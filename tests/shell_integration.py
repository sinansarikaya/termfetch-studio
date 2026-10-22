#!/usr/bin/env python3
"""Bash/Zsh/Fish startup regression checks in an isolated Ubuntu container."""
import json
import argparse
import hashlib
import shutil
import tempfile
import uuid
from pathlib import Path
from docker_matrix import ROOT, run, root_run, user_run, docker_user, setup, interactive

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--output', type=Path, default=ROOT / 'tests/shell-results.json')
parser.add_argument('--installer', type=Path, default=ROOT / 'install.sh')
args = parser.parse_args()
name = "tfs-shell-" + uuid.uuid4().hex[:12]
result = {"image": "ubuntu:24.04", "installer_sha256": hashlib.sha256(args.installer.read_bytes()).hexdigest(), "checks": {}}
with tempfile.TemporaryDirectory(prefix="tfs-shell-source-") as temporary:
    Path(temporary).chmod(0o755)
    shutil.copy2(args.installer, Path(temporary) / "install.sh")
    (Path(temporary) / "install.sh").chmod(0o644)
    prefix=args.installer.read_text().rsplit('\nmain "$@"',1)[0]
    (Path(temporary) / 'integration.sh').write_text(prefix)
    (Path(temporary) / 'integration.sh').chmod(0o644)
    run("docker", "run", "-d", "--name", name, "-v", f"{temporary}:/src:ro",
        "ubuntu:24.04", "sleep", "infinity")
    try:
        setup(name, "ubuntu:24.04", "missing")
        root_run(name, "apt-get install -y -qq zsh fish")
        user_run(name, "touch ~/.zshrc; mkdir -p ~/.config/fish; touch ~/.config/fish/config.fish")
        interactive(docker_user(name, "bash", "/src/install.sh"),
                    [("Install Nerd Fonts? (Y/n)", "n"),
                     ("Would you like to start TermFetch Studio now? (Y/n):", "n")])
        for shell, config in [("bash", "~/.bashrc"), ("zsh", "~/.zshrc"),
                              ("fish", "~/.config/fish/config.fish")]:
            user_run(name, f"{shell} -n {config}")
            quiet = user_run(name, f"{shell} -c 'source {config}; printf QUIET_OK'")
            if quiet.stdout != "QUIET_OK":
                raise AssertionError(f"{shell} wrote output in noninteractive startup: {quiet.stdout!r}")
            command = docker_user(name, shell, "-i", "-c", "printf SHELL_READY")
            command.remove("-i")  # Docker stdin is not needed for this fixed command.
            command.insert(2, "-t")
            startup = run(*command)
            if "SHELL_READY" not in startup.stdout or "Ubuntu" not in startup.stdout:
                raise AssertionError(f"{shell} interactive startup missing fastfetch or prompt")
            result["checks"][shell] = "passed: syntax, quiet noninteractive startup, PTY fastfetch startup"
        # Count actual invocations without exposing machine-specific fastfetch output.
        user_run(name, "cp ~/.local/bin/tfs-fastfetch ~/.local/bin/tfs-fastfetch.real; printf '#!/bin/sh\\nprintf STARTUP_FETCH\\n' > ~/.local/bin/tfs-fastfetch; chmod +x ~/.local/bin/tfs-fastfetch")
        def startup_count(shell, command):
            argv=docker_user(name,shell,'-i','-c',command)
            argv.remove('-i'); argv.insert(2,'-t')
            output=run(*argv).stdout
            if output.count('STARTUP_FETCH') != 1: raise AssertionError((shell,command,output))
        for shell, config in [('bash','~/.bashrc'),('zsh','~/.zshrc'),('fish','~/.config/fish/config.fish')]:
            startup_count(shell, 'source '+config+'; source '+config+'; printf READY')
        startup_count('fish', "bash -i -c 'printf CHILD_READY'")
        startup_count('bash', "fish -i -c 'printf CHILD_READY'")
        result['checks']['managed_dedup']='passed: repeated source and Fish/Bash nesting emit once per terminal'
        # External personal startup is kept byte-for-byte. Reinstall must not add a second fetch.
        legacy='fish_add_path ~/.local/bin\n# PERSONAL_VISUALS\nif status is-interactive; and test -t 1\n    fastfetch\nend\nfunction personal_function; echo preserved; end\n'
        user_run(name, "cp ~/.local/bin/tfs-fastfetch ~/.local/bin/fastfetch")
        user_run(name, "python3 - <<'PY'\nfrom pathlib import Path\np=Path.home()/'.config/fish/config.fish'\np.write_text("+repr(legacy)+"+p.read_text())\nPY")
        for _ in range(2): user_run(name, 'source /src/integration.sh; setup_shell_integration')
        user_run(name, "python3 - <<'PY'\nfrom pathlib import Path\ns=(Path.home()/'.config/fish/config.fish').read_text()\nassert "+repr(legacy)+" in s\nassert s.count('# TermFetch Studio - START')==1\nassert '_tfs_external_startup true' in s\nPY")
        startup_count('fish', 'printf LEGACY_READY')
        startup_count('fish', "bash -i -c 'printf LEGACY_CHILD_READY'")
        result['checks']['legacy_preserved']='passed: external Fish block preserved after two integrations; single fetch including nested Bash'
        user_run(name, 'mv ~/.local/bin/tfs-fastfetch.real ~/.local/bin/tfs-fastfetch; rm ~/.local/bin/fastfetch')
        result["fastfetch"] = user_run(name, "fastfetch --version").stdout.strip()
    finally:
        run("docker", "rm", "-f", name, check=False)
args.output.write_text(json.dumps(result, indent=2) + "\n")
print(json.dumps(result, indent=2))
