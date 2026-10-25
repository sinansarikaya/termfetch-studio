class Termfetch < Formula
  desc "Customize your terminal, your way: fastfetch themes, logos and terminal colors"
  homepage "https://github.com/sinansarikaya/termfetch-studio"
  version "2.1.0"
  license "MIT"

  on_macos do
    url "https://github.com/sinansarikaya/termfetch-studio/releases/download/v2.1.0/termfetch-studio-v2.1.0-apple-darwin-universal.tar.gz"
    sha256 "__SHA256_MACOS_UNIVERSAL__"
  end

  on_linux do
    url "https://github.com/sinansarikaya/termfetch-studio/releases/download/v2.1.0/termfetch-studio-v2.1.0-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "__SHA256_LINUX_GNU__"
  end

  depends_on "fastfetch"

  def install
    bin.install "termfetch-studio"
    bin.install_symlink bin/"termfetch-studio" => "termfetch"
    pkgshare.install "assets"
  end

  def caveats
    <<~EOS
      Open a new terminal and run `termfetch`.
      `termfetch settings` opens the Settings screen.
      Shell startup and the menu launcher are set up on first start.
    EOS
  end

  test do
    assert_match "2.1.0", shell_output("#{bin}/termfetch --version")
  end
end
