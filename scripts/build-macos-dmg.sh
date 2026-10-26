#!/usr/bin/env bash
set -euo pipefail

# Builds "TermFetch Studio.app" (universal: Apple Silicon + Intel) with fastfetch bundled
# inside, and wraps it in a drag-to-Applications DMG in dist/.
#
# Optional signing and notarization (without them the app is ad-hoc signed and users
# confirm it once under System Settings > Privacy & Security):
#   MACOS_SIGN_IDENTITY  "Developer ID Application: ..." from the login keychain
#   APPLE_ID, APPLE_TEAM_ID, APPLE_APP_PASSWORD  notarytool credentials
#
#   --skip-build  reuse target/apple-darwin-universal/release/termfetch-studio

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION=$(grep -m1 '^version =' "$ROOT_DIR/Cargo.toml" | cut -d '"' -f2)
DIST_DIR="$ROOT_DIR/dist"
WORK_DIR="$DIST_DIR/macos"
APP_NAME="TermFetch Studio"
APP="$WORK_DIR/$APP_NAME.app"
DMG="$DIST_DIR/termfetch-studio-v$VERSION-macos-universal.dmg"
UNIVERSAL="$ROOT_DIR/target/apple-darwin-universal/release/termfetch-studio"

# Pinned fastfetch release; checksums from the GitHub release assets.
FASTFETCH_VERSION="2.69.0"
FASTFETCH_SHA256_aarch64="3901a90b11f1823a00fea6f3c3c877e7f336465e9d9facc98fd6b8803e8fac7e"
FASTFETCH_SHA256_amd64="ec6ea2aed26df0561c9331526f1f4289f4285b11f7e3910c29f0b66766b635f1"
# Nerd Font symbols, so module icons render whatever font the terminal uses.
NERD_FONTS_VERSION="v3.5.1"
NERD_FONTS_SHA256="01172f37db8543edb102e5cb5c64101c9f4686630804d49b419aa07b23a69996"

[ "$(uname -s)" = "Darwin" ] || { echo "macOS is required (lipo, codesign, hdiutil)" >&2; exit 1; }

echo "=== TermFetch Studio v$VERSION macOS DMG ==="
rm -rf "$WORK_DIR"
mkdir -p "$WORK_DIR"

# 1. Universal termfetch-studio
if [ "${1:-}" != "--skip-build" ]; then
    if command -v rustup >/dev/null; then
        rustup target add aarch64-apple-darwin x86_64-apple-darwin
    fi
    for target in aarch64-apple-darwin x86_64-apple-darwin; do
        cargo build --release --locked --target "$target" --manifest-path "$ROOT_DIR/Cargo.toml"
    done
    mkdir -p "$(dirname "$UNIVERSAL")"
    lipo -create -output "$UNIVERSAL" \
        "$ROOT_DIR/target/aarch64-apple-darwin/release/termfetch-studio" \
        "$ROOT_DIR/target/x86_64-apple-darwin/release/termfetch-studio"
fi
lipo "$UNIVERSAL" -verify_arch arm64 x86_64

# 2. Universal fastfetch from the pinned release
FF_DIR="$WORK_DIR/fastfetch"
mkdir -p "$FF_DIR"
for arch in aarch64 amd64; do
    archive="$FF_DIR/fastfetch-macos-$arch.tar.gz"
    curl -fsSL -o "$archive" \
        "https://github.com/fastfetch-cli/fastfetch/releases/download/$FASTFETCH_VERSION/fastfetch-macos-$arch.tar.gz"
    expected="FASTFETCH_SHA256_$arch"
    echo "${!expected}  $archive" | shasum -a 256 -c -
    mkdir -p "$FF_DIR/$arch"
    tar -xzf "$archive" -C "$FF_DIR/$arch"
done
lipo -create -output "$FF_DIR/fastfetch" \
    "$FF_DIR/aarch64/fastfetch-macos-aarch64/usr/bin/fastfetch" \
    "$FF_DIR/amd64/fastfetch-macos-amd64/usr/bin/fastfetch"
lipo "$FF_DIR/fastfetch" -verify_arch arm64 x86_64

NF_DIR="$WORK_DIR/nerd-fonts"
mkdir -p "$NF_DIR"
curl -fsSL -o "$NF_DIR/symbols.tar.xz" \
    "https://github.com/ryanoasis/nerd-fonts/releases/download/$NERD_FONTS_VERSION/NerdFontsSymbolsOnly.tar.xz"
echo "$NERD_FONTS_SHA256  $NF_DIR/symbols.tar.xz" | shasum -a 256 -c -
tar -xJf "$NF_DIR/symbols.tar.xz" -C "$NF_DIR" SymbolsNerdFontMono-Regular.ttf LICENSE

# 3. App bundle
MACOS="$APP/Contents/MacOS"
RES="$APP/Contents/Resources"
mkdir -p "$MACOS" "$RES"
sed "s/__VERSION__/$VERSION/g" "$ROOT_DIR/packaging/macos/Info.plist" > "$APP/Contents/Info.plist"
install -m 755 "$ROOT_DIR/packaging/macos/launcher.sh" "$MACOS/$APP_NAME"
install -m 755 "$UNIVERSAL" "$MACOS/termfetch-studio"
install -m 755 "$FF_DIR/fastfetch" "$MACOS/fastfetch"
ln -s termfetch-studio "$MACOS/termfetch"
install -m 755 "$ROOT_DIR/packaging/macos/$APP_NAME.command" "$RES/$APP_NAME.command"
cp "$ROOT_DIR/assets/brand/termfetch.icns" "$RES/termfetch.icns"
cp -R "$ROOT_DIR/assets" "$RES/assets"
cp "$ROOT_DIR/LICENSE" "$RES/LICENSE"
cp "$FF_DIR/amd64/fastfetch-macos-amd64/usr/share/licenses/fastfetch/LICENSE" "$RES/LICENSE-fastfetch"
mkdir -p "$RES/fonts"
cp "$NF_DIR/SymbolsNerdFontMono-Regular.ttf" "$ROOT_DIR/assets/fonts/TermFetchBlocks.ttf" "$RES/fonts/"
cp "$NF_DIR/LICENSE" "$RES/LICENSE-nerd-fonts"
plutil -lint "$APP/Contents/Info.plist"

# 4. Signing
if [ -n "${MACOS_SIGN_IDENTITY:-}" ]; then
    for bin in fastfetch termfetch-studio; do
        codesign --force --timestamp --options runtime -s "$MACOS_SIGN_IDENTITY" "$MACOS/$bin"
    done
    codesign --force --timestamp --options runtime -s "$MACOS_SIGN_IDENTITY" "$APP"
else
    for bin in fastfetch termfetch-studio; do
        codesign --force -s - "$MACOS/$bin"
    done
    codesign --force -s - "$APP"
fi
codesign --verify --strict --verbose=2 "$APP"

# 5. DMG: the app beside an Applications shortcut
STAGE="$WORK_DIR/dmg"
mkdir -p "$STAGE"
cp -R "$APP" "$STAGE/"
ln -s /Applications "$STAGE/Applications"
rm -f "$DMG"
hdiutil create -volname "$APP_NAME" -srcfolder "$STAGE" -ov -format UDZO "$DMG"

if [ -n "${MACOS_SIGN_IDENTITY:-}" ]; then
    codesign --force --timestamp -s "$MACOS_SIGN_IDENTITY" "$DMG"
    if [ -n "${APPLE_ID:-}" ] && [ -n "${APPLE_TEAM_ID:-}" ] && [ -n "${APPLE_APP_PASSWORD:-}" ]; then
        xcrun notarytool submit "$DMG" --apple-id "$APPLE_ID" --team-id "$APPLE_TEAM_ID" \
            --password "$APPLE_APP_PASSWORD" --wait
        xcrun stapler staple "$DMG"
    fi
fi

rm -rf "$WORK_DIR"
echo "=== Done ==="
ls -lh "$DMG"
shasum -a 256 "$DMG"
