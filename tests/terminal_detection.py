#!/usr/bin/env python3
"""Check terminal ancestry handling without using the host's terminal settings."""
import re
import subprocess
import tempfile
from pathlib import Path

root = Path(__file__).resolve().parents[1]
source = (root / "install.sh").read_text()
match = re.search(r"<< 'CORELIB'\n(.*?)\nCORELIB\n", source, re.S)
if not match:
    raise SystemExit("CORELIB not found")

with tempfile.TemporaryDirectory(prefix="tfs-terminal-") as directory:
    core = Path(directory) / "core.sh"
    core.write_text(match.group(1) + "\n")
    for process_name, environment, expected in [
        ("gnome-terminal-", "KITTY_WINDOW_ID=stale", "none"),
        ("kitty", "KITTY_WINDOW_ID=", "kitty"),
    ]:
        pid = subprocess.Popen(
            ["python3", "-c",
             "import ctypes,os,subprocess,sys; "
             "ctypes.CDLL(None).prctl(15, sys.argv[1].encode(), 0, 0, 0); "
             "os.environ.update(dict(x.split('=',1) for x in sys.argv[2].split(';'))); "
             "subprocess.run(['bash','-c', 'source \"'+sys.argv[3]+'\"; check_terminal_image_support'], check=True)",
             process_name, environment, str(core)],
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        output, error = pid.communicate(timeout=10)
        if pid.returncode or output.strip() != expected:
            raise AssertionError(f"{process_name}: expected {expected}, got {output!r} {error!r}")
    print("Terminal ancestry: passed (stale Kitty environment, Kitty parent)")
