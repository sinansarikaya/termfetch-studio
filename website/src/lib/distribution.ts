// Update this registry only after verifying the release or package publication.
export const repository = "https://github.com/sinansarikaya/termfetch-studio";
export const publicOrigin = "https://termfetch-studio.sinansarikaya.dev";
export const release = {
  version: "2.0.0",
  tag: "v2.0.0",
  publishedAt: "2026-10-25",
  url: `${repository}/releases/tag/v2.0.0`,
  checksums: `${repository}/releases/download/v2.0.0/SHA256SUMS.txt`,
};
export const releaseAssets = [
  {
    platform: "Windows",
    title: "Windows x64",
    format: "ZIP",
    file: "termfetch-studio-v2.0.0-x86_64-pc-windows-msvc.zip",
    note: "Extract the ZIP, keep the bundled assets and run termfetch-studio.exe. Install Fastfetch separately.",
  },
  {
    platform: "macOS",
    title: "macOS Universal",
    format: "TAR.GZ",
    file: "termfetch-studio-v2.0.0-apple-darwin-universal.tar.gz",
    note: "Apple Silicon and Intel. Install Fastfetch separately.",
  },
  {
    platform: "Linux",
    title: "Linux x64 · glibc",
    format: "TAR.GZ",
    file: "termfetch-studio-v2.0.0-x86_64-unknown-linux-gnu.tar.gz",
    note: "For distributions using glibc.",
  },
  {
    platform: "Linux",
    title: "Linux x64 · musl",
    format: "TAR.GZ",
    file: "termfetch-studio-v2.0.0-x86_64-unknown-linux-musl.tar.gz",
    note: "Static binary. Install Fastfetch separately.",
  },
  {
    platform: "Linux",
    title: "Debian / Ubuntu x64",
    format: "DEB",
    file: "termfetch-studio_2.0.0_amd64.deb",
    note: "A downloadable package; no APT repository is published here.",
  },
  {
    platform: "Linux",
    title: "Fedora / openSUSE x64",
    format: "RPM",
    file: "termfetch-studio-2.0.0-1.x86_64.rpm",
    note: "A downloadable package; no DNF or Zypper repository is published here.",
  },
].map((asset) => ({
  ...asset,
  url: `${repository}/releases/download/${release.tag}/${asset.file}`,
}));
export type PackageChannel = {
  name: string;
  platform: string;
  format: string;
} & (
  | { status: "pending" }
  | { status: "available"; url: string; command: string }
);
export const packageChannels: PackageChannel[] = [
  {
    name: "Homebrew",
    platform: "macOS",
    format: "Tap / formula",
    status: "pending",
  },
  {
    name: "WinGet",
    platform: "Windows",
    format: "Package manifest",
    status: "pending",
  },
  {
    name: "Scoop",
    platform: "Windows",
    format: "Bucket manifest",
    status: "pending",
  },
  {
    name: "AUR",
    platform: "Arch Linux",
    format: "Source / binary recipe",
    status: "pending",
  },
];
export const sourceCommands = `git clone ${repository}.git
cd termfetch-studio
cargo build --release
# Linux / macOS
./target/release/termfetch-studio
# Windows PowerShell
.\\target\\release\\termfetch-studio.exe`;
