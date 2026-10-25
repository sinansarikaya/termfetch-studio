#!/usr/bin/env bash
set -euo pipefail

# TermFetch Studio Release Packaging Tool
# Packages release binaries, assets, licenses, checksums, and distribution archives.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION=$(grep -m1 '^version =' "$ROOT_DIR/Cargo.toml" | cut -d '"' -f2)
DIST_DIR="$ROOT_DIR/dist"

echo "=== TermFetch Studio v$VERSION Release Packaging ==="
mkdir -p "$DIST_DIR"

# 1. Build release binary if not present
if [ ! -f "$ROOT_DIR/target/release/termfetch-studio" ]; then
    echo "Building release binary..."
    cargo build --release --locked --manifest-path "$ROOT_DIR/Cargo.toml"
fi

# 2. Package Linux x86_64 GNU Archive
echo "Packaging Linux x86_64 GNU tarball..."
LINUX_DIR="$DIST_DIR/termfetch-studio-v$VERSION-x86_64-unknown-linux-gnu"
rm -rf "$LINUX_DIR"
mkdir -p "$LINUX_DIR"
cp "$ROOT_DIR/target/release/termfetch-studio" "$LINUX_DIR/"
cp -r "$ROOT_DIR/assets" "$LINUX_DIR/"
cp "$ROOT_DIR/LICENSE" "$LINUX_DIR/"
cp "$ROOT_DIR/README.md" "$LINUX_DIR/"

tar -czf "$DIST_DIR/termfetch-studio-v$VERSION-x86_64-unknown-linux-gnu.tar.gz" -C "$DIST_DIR" "termfetch-studio-v$VERSION-x86_64-unknown-linux-gnu"
rm -rf "$LINUX_DIR"

# 3. Package Source Archive (matches GitHub release archive format)
echo "Packaging Source tarball..."
git archive --format=tar.gz --prefix="termfetch-studio-$VERSION/" -o "$DIST_DIR/termfetch-studio-$VERSION.tar.gz" HEAD

# 4. Generate SHA256 Checksums
echo "Generating SHA256SUMS.txt..."
cd "$DIST_DIR"
sha256sum termfetch-studio-*.tar.gz* > "$DIST_DIR/SHA256SUMS.txt" 2>/dev/null || true

echo "=== Packaging Complete ==="
ls -lh "$DIST_DIR"
cat "$DIST_DIR/SHA256SUMS.txt"
