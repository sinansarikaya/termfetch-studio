#!/usr/bin/env bash
set -euo pipefail

# TermFetch Studio Release Manifest Checker
# Checks packaging manifests for placeholder values, ensuring incomplete
# manifests are never submitted to upstream package managers by accident.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PACKAGING_DIR="$ROOT_DIR/packaging"

FAIL_ON_PLACEHOLDER=false
if [ "${1:-}" = "--fail-on-placeholder" ]; then
    FAIL_ON_PLACEHOLDER=true
fi

echo "=== Checking Packaging Manifests for Placeholders ==="
FOUND_PLACEHOLDERS=0

# 1. Homebrew
echo -n "Checking Homebrew formula... "
HB_PLACEHOLDERS=$(grep -E 'PLACEHOLDER_[A-Z0-9_]+' "$PACKAGING_DIR/homebrew/termfetch-studio.rb" || true)
if [ -n "$HB_PLACEHOLDERS" ]; then
    echo "FOUND PLACEHOLDERS:"
    echo "$HB_PLACEHOLDERS" | sed 's/^/  /'
    FOUND_PLACEHOLDERS=$((FOUND_PLACEHOLDERS + 1))
else
    echo "OK (Clean)"
fi

# 2. AUR
echo -n "Checking AUR PKGBUILDs... "
AUR_PLACEHOLDERS=$(grep -E '(PLACEHOLDER|SKIP)' "$PACKAGING_DIR/aur/"* || true)
if [ -n "$AUR_PLACEHOLDERS" ]; then
    echo "FOUND PLACEHOLDERS/SKIP:"
    echo "$AUR_PLACEHOLDERS" | sed 's/^/  /'
    FOUND_PLACEHOLDERS=$((FOUND_PLACEHOLDERS + 1))
else
    echo "OK (Clean)"
fi

# 3. WinGet
echo -n "Checking WinGet installer manifest... "
WG_FILE=$(find "$PACKAGING_DIR/winget" -name "*installer.yaml")
WG_PLACEHOLDERS=$(grep -E '0000000000000000000000000000000000000000000000000000000000000000' "$WG_FILE" || true)
if [ -n "$WG_PLACEHOLDERS" ]; then
    echo "FOUND ZERO-SHA256 PLACEHOLDER"
    FOUND_PLACEHOLDERS=$((FOUND_PLACEHOLDERS + 1))
else
    echo "OK (Clean)"
fi

echo "-----------------------------------------------------"
if [ "$FOUND_PLACEHOLDERS" -gt 0 ]; then
    echo "Notice: $FOUND_PLACEHOLDERS packaging manifests contain placeholder values."
    echo "Run 'python3 scripts/inject_checksums.py /path/to/SHA256SUMS.txt' after release build."
    if [ "$FAIL_ON_PLACEHOLDER" = true ]; then
        echo "Error: Release gate failed due to unpopulated manifests (--fail-on-placeholder active)."
        exit 1
    fi
else
    echo "All manifests are verified and populated with genuine release checksums."
fi
