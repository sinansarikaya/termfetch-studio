#!/usr/bin/env python3
"""Prove unknown-distribution behavior with and without preinstalled fastfetch."""
import json
import shutil
import tempfile
import uuid
from pathlib import Path

from docker_matrix import ROOT, docker_user, interactive, root_run, run, setup, user_run


def probe(state, source):
    name = "tfs-unknown-" + uuid.uuid4().hex[:12]
    result = {"scenario": state}
    try:
        run("docker", "run", "-d", "--name", name, "-v", f"{source}:/src:ro",
            "debian:12", "sleep", "infinity")
        setup(name, "debian:12", state)
        root_run(name, "printf 'ID=unlisted\\nID_LIKE=\\nVERSION_ID=1\\n' > /etc/os-release")
        detect = "source <(sed -n '/^detect_distro_base()/,/^}/p' /src/install.sh); detect_distro_base"
        result["detection"] = user_run(name, detect).stdout.strip()
        if state == "missing":
            attempt = user_run(name, "bash /src/install.sh", check=False)
            result["install"] = "rejected" if attempt.returncode and "Unsupported distribution" in attempt.stdout else "unexpected"
        else:
            try:
                interactive(docker_user(name, "bash", "/src/install.sh"),
                            [("Install Nerd Fonts? (Y/n)", "n"),
                             ("Would you like to start TermFetch Studio now? (Y/n):", "n")])
                present = user_run(name, "test -x ~/.local/bin/termfetch-studio && termfetch-studio --version").stdout.strip()
                result["install"] = "passed" if "1.1.0" in present else "unexpected"
            except Exception as exc:
                result["install"] = f"failed: {exc}"
    finally:
        run("docker", "rm", "-f", name, check=False)
    return result


def main():
    with tempfile.TemporaryDirectory(prefix="tfs-unknown-src-") as temporary:
        source = Path(temporary)
        source.chmod(0o755)
        shutil.copy2(ROOT / "install.sh", source / "install.sh")
        (source / "install.sh").chmod(0o644)
        shutil.copytree(ROOT / "preset-logos", source / "preset-logos")
        results = [probe("missing", source), probe("preinstalled", source)]
    Path("tests/unknown-results.json").write_text(json.dumps(results, indent=2) + "\n")
    print(json.dumps(results, indent=2))
    return int(any(r["detection"] != "unknown" or r["install"] not in ("rejected", "passed") for r in results))


if __name__ == "__main__":
    raise SystemExit(main())
