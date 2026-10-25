class TermfetchStudio < Formula
  desc "Cross-platform terminal customization and Fastfetch theme studio"
  homepage "https://github.com/sinansarikaya/termfetch-studio"
  version "2.0.0"
  license "MIT"

  on_macos do
    url "https://github.com/sinansarikaya/termfetch-studio/releases/download/v2.0.0/termfetch-studio-v2.0.0-apple-darwin-universal.tar.gz"
    sha256 "PLACEHOLDER_APPLE_DARWIN_UNIVERSAL_SHA256"
  end

  on_linux do
    url "https://github.com/sinansarikaya/termfetch-studio/releases/download/v2.0.0/termfetch-studio-v2.0.0-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "PLACEHOLDER_X86_64_LINUX_GNU_SHA256"
  end

  depends_on "fastfetch"

  def install
    bin.install "termfetch-studio"
    pkgshare.install "assets"
  end

  test do
    assert_match "termfetch-studio 2.0.0", shell_output("#{bin}/termfetch-studio --version")
  end
end
