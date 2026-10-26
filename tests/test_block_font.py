"""TermFetch Blocks font: committed file matches the generator; one glyph per pattern."""
import importlib.util
from pathlib import Path
import struct
import unittest

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("blocks", ROOT / "scripts/generate-block-font.py")
blocks = importlib.util.module_from_spec(spec)
spec.loader.exec_module(blocks)


class BlockFont(unittest.TestCase):
    def test_committed_font_is_reproducible(self):
        self.assertEqual(blocks.build(), (ROOT / "assets/fonts/TermFetchBlocks.ttf").read_bytes())

    def test_patterns_cover_their_eighths(self):
        self.assertEqual(blocks.rects(0), [])
        # All eight: one box over the whole (overshooting) cell.
        full = blocks.rects(255)
        self.assertEqual(len(full), 1)
        x0, y0, x1, y1 = full[0]
        self.assertTrue(x0 < 0 < blocks.ADVANCE < x1 and y0 < -blocks.DESCENT and y1 > blocks.ASCENT)
        # Left column only: one tall box on the left half.
        (x0, _, x1, _), = blocks.rects(0b01010101)
        self.assertEqual((x0, x1), (-blocks.OVERSHOOT, blocks.ADVANCE // 2))

    def test_cmap_maps_the_private_use_range(self):
        font = blocks.build()
        count = struct.unpack(">H", font[4:6])[0]
        dirs = {font[12 + 16 * i:16 + 16 * i]: struct.unpack(">II", font[20 + 16 * i:28 + 16 * i]) for i in range(count)}
        cmap = dirs[b"cmap"][0]
        start, end, glyph = struct.unpack(">III", font[cmap + 20 + 16:cmap + 20 + 28])
        self.assertEqual((start, end, glyph), (blocks.BASE, blocks.BASE + 255, 1))


if __name__ == "__main__":
    unittest.main()
