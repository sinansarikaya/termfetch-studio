#!/usr/bin/env python3
"""Generate TermFetch Blocks: a TrueType font of the 256 octant patterns (2 x 4 cells),
used to draw image logos in terminals without an image protocol (macOS Terminal.app).

Glyph for pattern `m` sits at U+100000 + m (Supplementary Private Use Area-B, clear of
Nerd Fonts); bit `row * 2 + col` set means that eighth of the cell is foreground, row 0
on top. Metrics follow SF Mono, Terminal.app's default font, and the outer edges overshoot
slightly so neighbouring cells meet without hairline gaps. Python 3 stdlib only."""
import argparse
from pathlib import Path
import struct

ROOT = Path(__file__).resolve().parents[1]
UPM, ADVANCE, ASCENT, DESCENT = 2048, 1266, 1950, 494
# Terminals space rows a little wider than the font's own height (Terminal.app: ~4%),
# so the glyphs reach well past the top and bottom of the cell.
OVERSHOOT = 32
OVERSHOOT_Y = 128
BASE = 0x100000
FAMILY = "TermFetch Blocks"
VERSION = "Version 1.000"


def checksum(data):
    data += b"\0" * (-len(data) % 4)
    return sum(struct.unpack(f">{len(data) // 4}I", data)) & 0xFFFFFFFF


def rects(mask):
    """Foreground rectangles of a pattern: runs per row, merged down equal rows."""
    xs = [-OVERSHOOT, ADVANCE // 2, ADVANCE + OVERSHOOT]
    height = ASCENT + DESCENT
    ys = [ASCENT + OVERSHOOT_Y] + [ASCENT - height * r // 4 for r in (1, 2, 3)] + [-DESCENT - OVERSHOOT_Y]
    spans = []
    for row in range(4):
        on = [bool(mask >> (row * 2 + col) & 1) for col in range(2)]
        span = (0, 2) if all(on) else (0, 1) if on[0] else (1, 2) if on[1] else None
        spans.append(span)
    out, row = [], 0
    while row < 4:
        span, end = spans[row], row + 1
        while end < 4 and spans[end] == span:
            end += 1
        if span:
            out.append((xs[span[0]], ys[end], xs[span[1]], ys[row]))
        row = end
    return out


def glyph(mask):
    boxes = rects(mask)
    if not boxes:
        return b""
    points = []
    for x0, y0, x1, y1 in boxes:
        # Clockwise, as TrueType expects for outer contours.
        points.append([(x0, y0), (x0, y1), (x1, y1), (x1, y0)])
    flat = [p for contour in points for p in contour]
    xmin, ymin = min(p[0] for p in flat), min(p[1] for p in flat)
    xmax, ymax = max(p[0] for p in flat), max(p[1] for p in flat)
    data = struct.pack(">hhhhh", len(points), xmin, ymin, xmax, ymax)
    data += struct.pack(f">{len(points)}H", *[4 * i + 3 for i in range(len(points))])
    data += struct.pack(">H", 0) + b"\x01" * len(flat)
    px = py = 0
    xd, yd = b"", b""
    for x, y in flat:
        xd += struct.pack(">h", x - px)
        yd += struct.pack(">h", y - py)
        px, py = x, y
    return data + xd + yd


def name_table():
    strings = {1: FAMILY, 2: "Regular", 3: f"{FAMILY} Regular", 4: f"{FAMILY} Regular",
               5: VERSION, 6: "TermFetchBlocks-Regular"}
    records, storage = [], b""
    for platform, encoding, language, codec in ((1, 0, 0, "mac_roman"), (3, 1, 0x409, "utf-16-be")):
        for nid, text in strings.items():
            raw = text.encode(codec)
            records.append(struct.pack(">HHHHHH", platform, encoding, language, nid, len(raw), len(storage)))
            storage += raw
    return struct.pack(">HHH", 0, len(records), 6 + 12 * len(records)) + b"".join(records) + storage


def build():
    glyphs = [b""] + [glyph(m) for m in range(256)]
    glyf, loca = b"", []
    for g in glyphs:
        loca.append(len(glyf))
        glyf += g + b"\0" * (-len(g) % 4)
    loca.append(len(glyf))
    n = len(glyphs)
    max_points = max((len(rects(m)) * 4 for m in range(256)), default=0)
    max_contours = max((len(rects(m)) for m in range(256)), default=0)
    lo, hi = -OVERSHOOT, ADVANCE + OVERSHOOT
    tables = {
        "head": struct.pack(">IIIIHHqqhhhhHHhhh", 0x00010000, 0x00010000, 0, 0x5F0F3CF5, 0x000B, UPM,
                            0, 0, lo, -DESCENT - OVERSHOOT_Y, hi, ASCENT + OVERSHOOT_Y, 0, 8, 2, 1, 0),
        "hhea": struct.pack(">IhhhHhhhhhhhhhhhH", 0x00010000, ASCENT, -DESCENT, 0, ADVANCE, lo,
                            -OVERSHOOT, hi, 1, 0, 0, 0, 0, 0, 0, 0, n),
        "maxp": struct.pack(">IHHHHHHHHHHHHHH", 0x00010000, n, max_points, max_contours, 0, 0, 2,
                            0, 0, 0, 0, 0, 0, 0, 0),
        "OS/2": struct.pack(">HhHHHhhhhhhhhhhh10sIIIIIHHHhhhHHIIhhHHH", 4, ADVANCE, 400, 5, 0,
                            650, 700, 0, 140, 650, 700, 0, 480, 100, 600, 0,
                            bytes([2, 11, 6, 9, 0, 0, 0, 0, 0, 0]), 0, 0, 1 << 26, 0,
                            0x54465354, 0x0040, 0xFFFF, 0xFFFF, ASCENT, -DESCENT, 0,
                            ASCENT, DESCENT, 0, 0, ASCENT // 2, ASCENT * 3 // 4, 0, 32, 1),
        "hmtx": b"".join(struct.pack(">Hh", ADVANCE, min((r[0] for r in rects(m - 1)), default=0) if m else 0)
                         for m in range(n)),
        "cmap": None,
        "loca": struct.pack(f">{len(loca)}I", *loca),
        "glyf": glyf,
        "name": name_table(),
        "post": struct.pack(">IIhhIIIII", 0x00030000, 0, -100, 50, 1, 0, 0, 0, 0),
    }
    groups = struct.pack(">III", BASE, BASE + 255, 1)
    sub = struct.pack(">HHIII", 12, 0, 16 + len(groups), 0, 1) + groups
    tables["cmap"] = struct.pack(">HHHHIHHI", 0, 2, 0, 4, 20, 3, 10, 20) + sub
    tags = sorted(tables)
    count = len(tags)
    search = 1 << (count.bit_length() - 1)
    header = struct.pack(">IHHHH", 0x00010000, count, search * 16, search.bit_length() - 1, count * 16 - search * 16)
    offset = 12 + 16 * count
    directory, body = b"", b""
    for tag in tags:
        data = tables[tag]
        directory += struct.pack(">4sIII", tag.encode(), checksum(data), offset + len(body), len(data))
        body += data + b"\0" * (-len(data) % 4)
    font = bytearray(header + directory + body)
    head = offset + sum(len(tables[t]) + (-len(tables[t]) % 4) for t in tags[:tags.index("head")])
    struct.pack_into(">I", font, head + 8, (0xB1B0AFBA - checksum(bytes(font))) & 0xFFFFFFFF)
    return bytes(font)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "assets/fonts/TermFetchBlocks.ttf")
    out = parser.parse_args().output
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_bytes(build())
