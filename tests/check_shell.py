#!/usr/bin/env python3
"""Syntax check installer and generated Bash; run ShellCheck in Docker."""
import re
from collections import Counter
import subprocess
import tempfile
from pathlib import Path

root = Path(__file__).resolve().parents[1]
source = (root / "install.sh").read_text()
markers = ["MAINSCRIPT", "WRAPPER", "CORELIB", "FASTFETCHLIB", "UNINSTALL", "BASHRC"]
with tempfile.TemporaryDirectory(prefix="tfs-shell-") as temporary:
    output = Path(temporary)
    generated = []
    for marker in markers:
        match = re.search(r"<< '" + marker + r"'\n(.*?)\n" + marker + r"\n", source, re.S)
        if not match:
            raise SystemExit(f"Missing generated script: {marker}")
        path = output / (marker.lower() + ".sh")
        path.write_text(("# shellcheck shell=bash\n" if marker == "BASHRC" else "") + match.group(1) + "\n")
        generated.append(path)
    for path in [root / "install.sh", *generated]:
        subprocess.run(["bash", "-n", str(path)], check=True)
    print("Bash syntax: passed (installer, five generated scripts and Bash startup block)")
    command = ["docker", "run", "--rm", "-v", f"{root}:/repo:ro",
               "-v", f"{output}:/generated:ro", "koalaman/shellcheck:stable",
               "-S", "warning", "/repo/install.sh"]
    command += ["/generated/" + path.name for path in generated]
    result = subprocess.run(command, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    findings = Counter(re.findall(r"SC\d+ \((?:warning|error)\)", result.stdout))
    print("ShellCheck findings:", dict(sorted(findings.items())))
    # Existing advisory warnings are recorded for follow-up; errors fail this gate.
    command[command.index("warning")] = "error"
    errors = subprocess.run(command, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    if errors.returncode:
        print(errors.stdout)
    raise SystemExit(errors.returncode)
