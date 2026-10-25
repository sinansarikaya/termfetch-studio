#!/usr/bin/env python3
"""
TermFetch Studio Checksum Injector
Parses SHA256SUMS.txt and injects real checksums into:
- packaging/homebrew/termfetch-studio.rb
- packaging/aur/PKGBUILD-bin
- packaging/winget/manifests/.../SinanSarikaya.TermFetchStudio.installer.yaml
"""

import sys
import os
import re
from pathlib import Path

def main():
    if len(sys.argv) < 2:
        print("Usage: python3 inject_checksums.py <path_to_SHA256SUMS.txt>")
        sys.exit(1)

    sums_file = Path(sys.argv[1])
    if not sums_file.is_file():
        print(f"Error: Checksums file not found: {sums_file}")
        sys.exit(1)

    root_dir = Path(__file__).resolve().parent.parent
    packaging_dir = root_dir / "packaging"

    checksums = {}
    with open(sums_file, "r", encoding="utf-8") as f:
        for line in f:
            parts = line.strip().split()
            if len(parts) >= 2:
                sha = parts[0].strip()
                name = Path(parts[1].strip()).name
                checksums[name] = sha

    print(f"Loaded {len(checksums)} checksums from {sums_file.name}:")
    for name, sha in checksums.items():
        print(f"  {name} -> {sha[:12]}...")

    # 1. Update Homebrew formula
    hb_file = packaging_dir / "homebrew" / "termfetch-studio.rb"
    if hb_file.exists():
        content = hb_file.read_text(encoding="utf-8")
        mac_sha = next((sha for name, sha in checksums.items() if "apple-darwin" in name or "macos" in name), None)
        linux_sha = next((sha for name, sha in checksums.items() if "x86_64-unknown-linux-gnu.tar.gz" in name), None)
        if mac_sha:
            content = re.sub(r'(on_macos do\s+url\s+"[^"]+"\s+sha256\s+)"[^"]+"', rf'\1"{mac_sha}"', content)
            content = content.replace("PLACEHOLDER_APPLE_DARWIN_UNIVERSAL_SHA256", mac_sha)
        if linux_sha:
            content = re.sub(r'(on_linux do\s+url\s+"[^"]+"\s+sha256\s+)"[^"]+"', rf'\1"{linux_sha}"', content)
            content = content.replace("PLACEHOLDER_X86_64_LINUX_GNU_SHA256", linux_sha)
        hb_file.write_text(content, encoding="utf-8")
        print("Updated Homebrew formula.")

    # 2. Update AUR PKGBUILD-bin and termfetch-studio-bin/PKGBUILD
    aur_files = [
        packaging_dir / "aur" / "PKGBUILD-bin",
        packaging_dir / "aur" / "termfetch-studio-bin" / "PKGBUILD",
    ]
    for aur_file in aur_files:
        if aur_file.exists():
            content = aur_file.read_text(encoding="utf-8")
            for name, sha in checksums.items():
                if "x86_64-unknown-linux-gnu.tar.gz" in name:
                    content = re.sub(r"sha256sums=\('[^']+'\)", f"sha256sums=('{sha}')", content)
            aur_file.write_text(content, encoding="utf-8")
            print(f"Updated AUR {aur_file.name}.")

    # 3. Update WinGet installer manifest
    winget_files = list((packaging_dir / "winget").glob("**/*installer.yaml"))
    for wg_file in winget_files:
        content = wg_file.read_text(encoding="utf-8")
        for name, sha in checksums.items():
            if "windows-msvc.zip" in name:
                content = re.sub(r"InstallerSha256:\s+[a-f0-9A-F]{64}", f"InstallerSha256: {sha.upper()}", content)
        wg_file.write_text(content, encoding="utf-8")
        print(f"Updated WinGet manifest: {wg_file.name}")

    print("=== Checksum injection complete ===")

if __name__ == "__main__":
    main()
