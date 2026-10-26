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

def tile_glyph_rects():
    """The glyph cells of the tile variant, whose colours are made for its dark background."""
    root = ET.parse(ROOT / "assets/brand/termfetch-tile.svg").getroot()
    rects = []
    def visit(node, fill="#000000"):
        fill = node.get("fill", fill)
        if node.tag.endswith("}rect") and not node.get("rx"):
            rects.append((tuple(float(node.get(k, 0)) for k in ("x", "y", "width", "height")), bytes.fromhex(fill.lstrip("#"))))
        for child in node:
            visit(child, fill)
    visit(root)
    return rects

TILE = "0f172a"
# Apple's icon grid on a 1024 canvas, shared by every platform's app icon.
TILE0, TILE1, RADIUS = 100, 924, 185
GLYPH0, CELL = 192, 40

def app_png(size):
    """App icon raster: the glyph on the navy rounded tile. Pixels whose corners agree
    are filled directly; edge pixels are anti-aliased with 4x4 subsamples."""
    s = size / 1024
    tile0, tile1, radius = TILE0 * s, TILE1 * s, RADIUS * s
    rects = [(((GLYPH0 + x * CELL) * s, (GLYPH0 + y * CELL) * s, w * CELL * s, h * CELL * s), rgb) for (x, y, w, h), rgb in tile_glyph_rects()]
    tile = bytes.fromhex(TILE)
    def sample(px, py):
        if not (tile0 <= px < tile1 and tile0 <= py < tile1):
            return None
        cx = min(max(px, tile0 + radius), tile1 - radius)
        cy = min(max(py, tile0 + radius), tile1 - radius)
        if (px - cx) ** 2 + (py - cy) ** 2 > radius ** 2:
            return None
        for (rx, ry, rw, rh), rgb in rects:
            if rx <= px < rx + rw and ry <= py < ry + rh:
                return rgb
        return tile
    subs = [(i + .5) / 4 for i in range(4)]
    rows = bytearray()
    for y in range(size):
        rows.append(0)
        for x in range(size):
            corners = {sample(x + dx, y + dy) for dx in (.01, .99) for dy in (.01, .99)}
            if len(corners) == 1:
                rgb = corners.pop()
                rows.extend(rgb + b"\xff" if rgb else b"\x00\x00\x00\x00")
                continue
            acc, hits = [0, 0, 0], 0
            for sy in subs:
                for sx in subs:
                    rgb = sample(x + sx, y + sy)
                    if rgb:
                        hits += 1
                        for i in range(3):
                            acc[i] += rgb[i]
            if hits:
                rows.extend(bytes(round(a / hits) for a in acc) + bytes([round(255 * hits / 16)]))
            else:
                rows.extend(b"\x00\x00\x00\x00")
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(bytes(rows), 9)) + chunk(b"IEND", b"")

def app_svg():
    """The app icon as SVG (Linux icon theme / launchers), same geometry as the rasters."""
    cells = "".join(f'<rect x="{GLYPH0 + x * CELL:g}" y="{GLYPH0 + y * CELL:g}" width="{w * CELL:g}" height="{h * CELL:g}" fill="#{rgb.hex()}"/>' for (x, y, w, h), rgb in tile_glyph_rects())
    return (f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024" role="img" aria-label="TermFetch Studio"><title>TermFetch Studio app icon</title>\n'
            f'<rect x="{TILE0}" y="{TILE0}" width="{TILE1 - TILE0}" height="{TILE1 - TILE0}" rx="{RADIUS}" fill="#{TILE}"/>{cells}\n</svg>\n')

# Every ICNS slot macOS looks up, Retina (@2x) variants included.
MAC_ICNS = ((b"icp4", 16), (b"icp5", 32), (b"icp6", 64), (b"ic07", 128), (b"ic08", 256), (b"ic09", 512),
            (b"ic10", 1024), (b"ic11", 32), (b"ic12", 64), (b"ic13", 256), (b"ic14", 512))

def generate(output):
    output = Path(output)
    output.mkdir(parents=True, exist_ok=True)
    images = {s: app_png(s) for s in SIZES + (1024,)}
    for size in SIZES:
        (output / f"termfetch-{size}.png").write_bytes(images[size])
    ico_sizes = (16, 32, 48, 64, 128, 256)
    offset = 6 + 16 * len(ico_sizes)
    entries = bytearray()
    for size in ico_sizes:
        data = images[size]
        entries.extend(struct.pack("<BBBBHHII", size % 256, size % 256, 0, 0, 1, 32, len(data), offset))
        offset += len(data)
    (output / "termfetch.ico").write_bytes(struct.pack("<HHH", 0, 1, len(ico_sizes)) + entries + b"".join(images[s] for s in ico_sizes))
    # Modern PNG-backed ICNS representations; no platform tools needed.
    payload = b"".join(kind + struct.pack(">I", len(images[s]) + 8) + images[s] for kind, s in MAC_ICNS)
    (output / "termfetch.icns").write_bytes(b"icns" + struct.pack(">I", len(payload) + 8) + payload)
    (output / "termfetch-app-icon.svg").write_text(app_svg())

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "assets/brand")
    generate(parser.parse_args().output)
