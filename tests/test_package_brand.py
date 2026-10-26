"""Exercise packaging of pre-brand releases and new branded archives in temp dirs."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
RECIPES = (
    "packaging/aur/PKGBUILD",
    "packaging/aur/PKGBUILD-bin",
    "packaging/aur/termfetch-studio/PKGBUILD",
    "packaging/aur/termfetch-studio-bin/PKGBUILD",
)


def fixture(base, branded):
    base.mkdir(parents=True)
    (base / "assets").mkdir()
    (base / "assets" / "sample.txt").write_text("existing assets")
    (base / "termfetch-studio").write_text("binary fixture")
    (base / "target" / "release").mkdir(parents=True)
    (base / "target" / "release" / "termfetch-studio").write_text("binary fixture")
    for name in ("LICENSE", "README.md"):
        (base / name).write_text(name)
    if branded:
        brand = base / "assets" / "brand"
        brand.mkdir()
        for name in ("termfetch-app-icon.svg", "termfetch-studio.desktop"):
            (brand / name).write_bytes((ROOT / "assets" / "brand" / name).read_bytes())


class PackageBrandTests(unittest.TestCase):
    def assert_icons(self, output, branded):
        for path in (
            "usr/share/icons/hicolor/scalable/apps/termfetch-studio.svg",
            "usr/share/applications/termfetch-studio.desktop",
        ):
            self.assertEqual((output / path).is_file(), branded, path)
        self.assertTrue((output / "usr/bin/termfetch-studio").is_file())
        self.assertTrue((output / "usr/share/termfetch-studio/assets/sample.txt").is_file())

    def test_aur_current_and_future_archives(self):
        for recipe in RECIPES:
            for branded in (False, True):
                with self.subTest(recipe=recipe, branded=branded), tempfile.TemporaryDirectory() as temp:
                    temp = Path(temp)
                    name = "termfetch-studio-v2.0.0-x86_64-unknown-linux-gnu" if "-bin" in recipe else "termfetch-studio-2.0.0"
                    fixture(temp / name, branded)
                    output = temp / "package"
                    subprocess.run(
                        ["bash", "-c", 'set -euo pipefail; source "$1"; package', "check", str(ROOT / recipe)],
                        cwd=temp, env={**os.environ, "pkgdir": str(output)}, check=True,
                        capture_output=True, text=True,
                    )
                    self.assert_icons(output, branded)

    def test_rpm_install_section_and_file_list(self):
        spec = (ROOT / "packaging/rpm/termfetch-studio.spec").read_text()
        install = spec.split("%install\n", 1)[1].split("\n%files", 1)[0]
        for branded in (False, True):
            with self.subTest(branded=branded), tempfile.TemporaryDirectory() as temp:
                temp = Path(temp)
                source, output, build = temp / "archive", temp / "package", temp / "build"
                fixture(source, branded)
                build.mkdir()
                script = install
                for key, value in {"buildroot": output, "_bindir": "/usr/bin", "_datadir": "/usr/share", "name": "termfetch-studio", "_builddir": build}.items():
                    script = script.replace("%{" + key + "}", str(value))
                subprocess.run(["bash", "-euc", script], cwd=source, check=True, capture_output=True, text=True)
                self.assert_icons(output, branded)
                entries = (build / "termfetch-studio-brand-files").read_text().splitlines()
                self.assertEqual(len(entries), 2 if branded else 0)
                for entry in entries:
                    self.assertTrue((output / entry.lstrip("/")).is_file())


if __name__ == "__main__":
    unittest.main()
