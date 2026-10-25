class TermfetchStudio < Formula
  desc "Cross-platform terminal customization and Fastfetch theme studio"
  homepage "https://github.com/sinansarikaya/termfetch-studio"
  version "2.0.0"
  license "MIT"

  on_macos do
    url "https://github.com/sinansarikaya/termfetch-studio/releases/download/v2.0.0/termfetch-studio-v2.0.0-apple-darwin-universal.tar.gz"
    sha256 "f04f8d822bdd3b0b02c2753ffc684dfa3e8895dccf7ff8a9fe2f02c22b0d8f91"
  end

  on_linux do
    url "https://github.com/sinansarikaya/termfetch-studio/releases/download/v2.0.0/termfetch-studio-v2.0.0-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "6ce95895c551679e40386ffc4cbc7e7e4aadcfbe993f8acbfa5c6acb4256047c"
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
