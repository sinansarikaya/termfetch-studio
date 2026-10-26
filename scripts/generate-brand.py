#!/usr/bin/env python3
"""Reproduce brand rasters from SVG rect geometry using Python 3 stdlib only."""
import argparse
import binascii
from pathlib import Path
import struct
import xml.etree.ElementTree as ET
import zlib

ROOT = Path(__file__).resolve().parents[1]
SIZES = (16, 32, 48, 64, 128, 256, 512)

def chunk(kind, data):
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", binascii.crc32(kind + data) & 0xffffffff)

def png(source, size):
    root = ET.parse(source).getroot()
    rects = []
    def visit(node, fill="#000000"):
        fill = node.get("fill", fill)
        if node.tag.endswith("}rect"):
            if node.get("rx"):
                raise ValueError("Raster source must use square cells")
            rects.append((tuple(float(node.get(k, 0)) for k in ("x", "y", "width", "height")), bytes.fromhex(fill.lstrip("#"))))
        for child in node:
            visit(child, fill)
    visit(root)
    rows = bytearray()
    for y in range(size):
        rows.append(0)
        for x in range(size):
            color = b"\x00\x00\x00\x00"
            for (rx, ry, rw, rh), rgb in rects:
                if rx <= (x + .5) * 16 / size < rx + rw and ry <= (y + .5) * 16 / size < ry + rh:
                    color = rgb + b"\xff"
            rows.extend(color)
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(bytes(rows), 9)) + chunk(b"IEND", b"")

def generate(output):
    output = Path(output)
    output.mkdir(parents=True, exist_ok=True)
    images = {s: png(ROOT / "assets/brand/termfetch-symbol.svg", s) for s in SIZES}
    for size, data in images.items():
        (output / f"termfetch-{size}.png").write_bytes(data)
    ico_sizes = (16, 32, 48, 64, 128, 256)
    offset = 6 + 16 * len(ico_sizes)
    entries = bytearray()
    for size in ico_sizes:
        data = images[size]
        entries.extend(struct.pack("<BBBBHHII", size % 256, size % 256, 0, 0, 1, 32, len(data), offset))
        offset += len(data)
    (output / "termfetch.ico").write_bytes(struct.pack("<HHH", 0, 1, len(ico_sizes)) + entries + b"".join(images[s] for s in ico_sizes))
    # Modern PNG-backed ICNS representations; no platform tools needed.
    payload = b"".join(kind + struct.pack(">I", len(images[s]) + 8) + images[s] for kind, s in ((b"icp4", 16), (b"icp5", 32), (b"icp6", 64), (b"ic07", 128), (b"ic08", 256), (b"ic09", 512)))
    (output / "termfetch.icns").write_bytes(b"icns" + struct.pack(">I", len(payload) + 8) + payload)

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "assets/brand")
    generate(parser.parse_args().output)
