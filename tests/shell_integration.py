#!/usr/bin/env python3
"""Bash/Zsh/Fish startup regression checks in an isolated Ubuntu container."""
import json
import shutil
import tempfile
import uuid
from pathlib import Path
from docker_matrix import ROOT, run, root_run, user_run, docker_user, setup, interactive

name = "tfs-shell-" + uuid.uuid4().hex[:12]
result = {"image": "ubuntu:24.04", "checks": {}}
with tempfile.TemporaryDirectory(prefix="tfs-shell-source-") as temporary:
    Path(temporary).chmod(0o755)
    shutil.copy2(ROOT / "install.sh", Path(temporary) / "install.sh")
    (Path(temporary) / "install.sh").chmod(0o644)
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
        result["fastfetch"] = user_run(name, "fastfetch --version").stdout.strip()
    finally:
        run("docker", "rm", "-f", name, check=False)
(ROOT / "tests/shell-results.json").write_text(json.dumps(result, indent=2) + "\n")
print(json.dumps(result, indent=2))
