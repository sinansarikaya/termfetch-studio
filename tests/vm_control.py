#!/usr/bin/env python3
"""Send guest keyboard events and capture QEMU's real graphical framebuffer.

Example: vm_control.py --monitor /var/tmp/tfs/monitor.sock key:down key:ret
         vm_control.py --monitor /var/tmp/tfs/monitor.sock shot:/var/tmp/menu.png
This controls an already running isolated VM; it does not configure the host.
"""
import argparse
import json
import re
import socket
import subprocess
import time
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--monitor", required=True, type=Path)
parser.add_argument("actions", nargs="+", help="key:QEMU-key, text:command, wait:seconds, shot:/path.png")
args = parser.parse_args()
with socket.socket(socket.AF_UNIX) as monitor:
    monitor.connect(str(args.monitor))
    monitor.settimeout(1)
    try:
        monitor.recv(4096)
    except socket.timeout:
        pass

    def command(value):
        monitor.sendall((value + "\n").encode())

    def key(value):
        if not re.fullmatch(r"[a-z0-9_-]+", value):
            raise ValueError(f"Invalid QEMU key: {value}")
        command("sendkey " + value)

    for action in args.actions:
        kind, value = action.split(":", 1)
        if kind == "key":
            key(value)
            time.sleep(0.8)
        elif kind == "text":
            for character in value:
                key({"-": "minus", " ": "spc", "/": "slash", ".": "dot",
                     "_": "shift-minus"}.get(character, character))
                time.sleep(0.08)
        elif kind == "wait":
            delay = float(value)
            if not 0 <= delay <= 30:
                raise ValueError("Wait must be between 0 and 30 seconds")
            time.sleep(delay)
        elif kind == "shot":
            target = Path(value).resolve()
            if target.suffix != ".png":
                raise ValueError("Screenshot target must end in .png")
            temporary = target.with_suffix(".ppm")
            if target.exists() or temporary.exists():
                raise FileExistsError(target)
            command("screendump " + json.dumps(str(temporary)))
            time.sleep(1)
            subprocess.run(["magick", str(temporary), str(target)], check=True)
            temporary.unlink()
        else:
            raise ValueError(f"Unknown action: {kind}")
