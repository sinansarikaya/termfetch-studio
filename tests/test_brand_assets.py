"""Brand verification and canonical artifact generation; no host integration runs."""
import binascii
import xml.etree.ElementTree as ET
import importlib.util
from pathlib import Path
import struct
import tempfile
import unittest
import zlib

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("brand", ROOT / "scripts/generate-brand.py")
brand = importlib.util.module_from_spec(spec)
spec.loader.exec_module(brand)

class BrandAssets(unittest.TestCase):
    def test_regeneration_formats_dimensions_and_determinism(self):
        with tempfile.TemporaryDirectory() as directory:
            out = Path(directory)
            brand.generate(out)
            first = {p.name: p.read_bytes() for p in out.iterdir()}
            brand.generate(out)
            self.assertEqual(first, {p.name: p.read_bytes() for p in out.iterdir()})
            for size in brand.SIZES:
                data = first[f"termfetch-{size}.png"]
                self.assertEqual(data[:8], b"\x89PNG\r\n\x1a\n")
                self.assertEqual(struct.unpack(">II", data[16:24]), (size, size))
                offset, raw = 8, b""
                while offset < len(data):
                    length = struct.unpack(">I", data[offset:offset+4])[0]
                    kind = data[offset+4:offset+8]
                    payload = data[offset+8:offset+8+length]
                    crc = struct.unpack(">I", data[offset+8+length:offset+12+length])[0]
                    self.assertEqual(crc, binascii.crc32(kind + payload) & 0xffffffff)
                    if kind == b"IDAT":
                        raw += data[offset+8:offset+8+length]
                    offset += length + 12
                pixels = zlib.decompress(raw)
                self.assertEqual(len(pixels), size * (size * 4 + 1))
                self.assertEqual(pixels[4], 0)  # transparent margin
                self.assertIn(b"\x05\x96\x69\xff", pixels)
                self.assertIn(b"\x08\x91\xb2\xff", pixels)
            ico = first["termfetch.ico"]
            self.assertEqual(struct.unpack("<HHH", ico[:6]), (0, 1, 6))
            for i, size in enumerate((16, 32, 48, 64, 128, 256)):
                w, h, _, _, planes, bits, length, offset = struct.unpack("<BBBBHHII", ico[6+i*16:22+i*16])
                self.assertEqual((w or 256, h or 256, planes, bits), (size, size, 1, 32))
                self.assertEqual(ico[offset:offset+length], first[f"termfetch-{size}.png"])
            icns = first["termfetch.icns"]
            self.assertEqual(icns[:4], b"icns")
            self.assertEqual(struct.unpack(">I", icns[4:8])[0], len(icns))
            offset = 8
            for kind, size in ((b"icp4",16),(b"icp5",32),(b"icp6",64),(b"ic07",128),(b"ic08",256),(b"ic09",512)):
                self.assertEqual(icns[offset:offset+4], kind)
                length = struct.unpack(">I", icns[offset+4:offset+8])[0]
                self.assertEqual(icns[offset+8:offset+length], first[f"termfetch-{size}.png"])
                offset += length
            self.assertEqual(offset, len(icns))
            # Canonical assets are written only after their temporary equivalents pass.
            brand.generate(ROOT / "assets/brand")
            for name, data in first.items():
                self.assertEqual((ROOT / "assets/brand" / name).read_bytes(), data)

    def test_svg_variants_keep_grid_geometry_and_font_independence(self):
        source = ET.parse(ROOT / "assets/brand/termfetch-symbol.svg").getroot()
        mono = ET.parse(ROOT / "assets/brand/termfetch-monochrome.svg").getroot()
        rects = lambda root: [tuple(n.get(k, "0") for k in ("x","y","width","height")) for n in root.iter() if n.tag.endswith("}rect")]
        self.assertEqual(rects(source), rects(mono))
        self.assertEqual(source.get("viewBox"), "0 0 16 16")
        for x, y, w, h in rects(source):
            x, y, w, h = map(int, (x, y, w, h))
            self.assertTrue(0 <= x < x+w <= 16 and 0 <= y < y+h <= 16)
        wordmark = ET.parse(ROOT / "assets/brand/termfetch-wordmark.svg").getroot()
        self.assertFalse(any(n.tag.endswith("}text") for n in wordmark.iter()))
        self.assertTrue(any(n.tag.endswith("}path") for n in wordmark.iter()))
        tile = ET.parse(ROOT / "assets/brand/termfetch-tile.svg").getroot()
        self.assertEqual(tile.get("viewBox"), "0 0 16 16")

    def test_packaging_and_embedded_asset_references_exist(self):
        desktop = (ROOT / "assets/brand/termfetch-studio.desktop").read_text()
        self.assertIn("Icon=termfetch-studio\n", desktop)
        self.assertIn("Terminal=true\n", desktop)
        for recipe in ("packaging/aur/PKGBUILD", "packaging/aur/PKGBUILD-bin",
                       "packaging/aur/termfetch-studio/PKGBUILD", "packaging/aur/termfetch-studio-bin/PKGBUILD",
                       "packaging/rpm/termfetch-studio.spec", ".github/workflows/release.yml"):
            text = (ROOT / recipe).read_text()
            self.assertIn("assets/brand/termfetch-symbol.svg", text)
            self.assertIn("/icons/hicolor/scalable/apps/termfetch-studio.svg", text)
            self.assertIn("/applications/termfetch-studio.desktop", text)
        installer = (ROOT / "src/adapters/installer.rs").read_text()
        self.assertIn('include_bytes!("../../assets/brand/termfetch.ico")', installer)
        self.assertIn('SCRIPT.replace("__TERMFETCH_ICON_BASE64__", &icon)', installer)
        ps = (ROOT / "scripts/install-windows.ps1").read_text()
        self.assertIn('$shortcut.IconLocation = "$iconPath,0"', ps)
        self.assertIn("assets\\brand\\termfetch.ico", ps)
