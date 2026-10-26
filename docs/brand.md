# TermFetch Studio brand

## Design and source

The TF cells monogram uses two aligned three-unit stems, a seven-unit T cap,
a cyan F arm, and a detached two-unit cursor cell. It connects terminal editing
with the color cells used to customize Fastfetch. A 16 × 16 grid, integer edges,
and open negative space keep the silhouette legible at favicon size and in one
color. This is an original drawing, not a trademark search or legal uniqueness
guarantee.

Source of truth lives in assets/brand:

- termfetch-symbol.svg: transparent emerald #059669 / cyan #0891b2 symbol,
  for light and dark surfaces. Use the tile for a consistently dark backdrop.
- termfetch-monochrome.svg: identical geometry with currentColor. Inline it
  and set color for the surrounding surface; external image files default to black.
- termfetch-wordmark.svg: symbol plus original grid-drawn uppercase lettering.
  Letter outlines are paths, with no installed-font dependency.
- termfetch-tile.svg: dark slate rounded tile with brighter emerald/cyan cells.
  This is an alternate SVG presentation, not the source for transparent rasters.
- termfetch-studio.desktop: package launcher, using the hicolor icon name.

Keep at least one grid unit clear around the mark. Do not add a terminal box or
prompt chevron. At very small sizes prefer the symbol over the wordmark. The TUI
uses the compact text approximation "TF·" in the existing header badge, with the
theme primary color and unchanged layout width; it needs no private-use font.

## Regeneration

Exact prerequisites: Python 3.9 or newer, standard library only. No Pillow,
CairoSVG, SVG renderer, native resource compiler, or runtime app dependency.

Run from any directory:

```sh
python3 scripts/generate-brand.py
python3 scripts/generate-brand.py --output /tmp/termfetch-brand
python3 -m unittest discover -s tests -p test_brand_assets.py
```

The generator reads rectangle geometry and inherited fills directly from
termfetch-symbol.svg. It deliberately supports square, solid-color rectangles;
rounded paths or gradients require extending and verifying the renderer first.
It uses deterministic center sampling of the 16-unit grid, RGBA PNG with
transparent margins, and lossless zlib compression. The unusual 48px size is
still an exact integer scale.

Generated outputs intended for version control: termfetch-{16,32,48,64,128,256,512}.png,
termfetch.ico (16/32/48/64/128/256 PNG frames), and termfetch.icns
(PNG representations icp4/icp5/icp6/ic07/ic08/ic09 through 512px).
Byte-identical regeneration assumes the same Python/zlib implementation;
geometry and formats do not depend on external fonts or raster libraries.
The unittest first generates into a temporary directory, verifies dimensions,
alpha, palette, ICO directory/frame payloads, ICNS structure, and repeatability,
then generates canonical assets. No host installation happens in tests.

## Usage and platform behavior

Linux setup embeds the SVG in the binary and writes it beside the user's
termfetch-studio.desktop in the XDG applications directory. The launcher uses
that absolute icon path, so a standalone executable also works. Repeated writes
are no-ops. A differing existing icon is refused. Existing launcher removal
also removes the adjacent icon only when its bytes match the embedded source;
modified icons stay in place. Older branding revisions may be left in place
rather than deleting or overwriting unrecognized bytes.

Debian release staging, RPM and both AUR recipe copies install the package
launcher into /usr/share/applications and the SVG into
/usr/share/icons/hicolor/scalable/apps/termfetch-studio.svg. Package managers
own removal of these system files. AUR/RPM recipes skip these two optional
files when building the existing v2.0.0 archives, which predate the brand. New
release archives are checked for mandatory brand assets. Recipes still reference their existing
release versions/checksums; they must be updated against real release artifacts
before submission. No package has been submitted or installed here.

Release archives already copy assets recursively. The local packaging script
and release workflow now check that brand assets are present. Source archives
use git archive HEAD, so newly generated assets must be committed before packaging.
The website leader should copy brand assets to website/public/brand; its shared
reference is /brand/termfetch-symbol.svg. This worker did not edit the website.

Windows archive installation locates assets/brand/termfetch.ico and copies it
into the stable install folder. Start Menu app/settings shortcuts use that ICO.
The embedded PowerShell installer replaces a private placeholder with base64
from include_bytes!, so an EXE alone carries its shortcut icon too. A directly
invoked script without bundled assets falls back to the platform shortcut icon.
Windows Terminal profile setup writes an embedded 64px PNG beside the selected
settings file and uses its absolute path. Other profiles and the existing backup
behavior remain intact. Icon collisions fail safely; removal deletes only an
exact embedded-image match. The install folder cleanup includes the ICO.

Windows builds embed the ICO into the executable through build.rs and the
build-only winresource crate. The target OS is checked at build-script runtime,
so Linux/macOS builds skip resource compilation. Windows builds require the
Windows SDK resource compiler (or a configured GNU windres toolchain). A missing
compiler fails the Windows build instead of silently shipping an unbranded EXE.
Native Windows Explorer, shortcut and Terminal rendering require Windows QA.

The current macOS source includes an application bundle and DMG build workflow
with the shared ICNS icon, launcher and bundled dependencies. These packaging
sources do not imply a published DMG asset: the verified public v2.0.0 release
still provides the universal tarball. Check actual release assets before
advertising a downloadable DMG. Signing, notarization and native Finder/Dock
display require their own macOS verification.

## Verification

Run the deterministic asset tests and Rust behavior tests from the repository:

```sh
python3 -m unittest discover -s tests -p test_brand_assets.py
cargo test --lib
cargo test --test brand_integration
cargo fmt --all -- --check
```

Tests use temporary directories; they do not install launchers or modify host
terminal settings. Native Windows/macOS rendering is verified separately from
these Linux source and behavior checks. See the release QA record for results.
