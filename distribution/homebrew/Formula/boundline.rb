class Boundline < Formula
  desc "Local delivery orchestrator for bounded engineering work"
  homepage "https://github.com/apply-the/boundline"
  version "0.40.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/apply-the/boundline/releases/download/v0.40.0/boundline-bundle-0.40.0-macos-arm64.tar.gz"
      sha256 "REPLACE_WITH_MACOS_ARM64_SHA256"
    else
      url "https://github.com/apply-the/boundline/releases/download/v0.40.0/boundline-bundle-0.40.0-macos-x86_64.tar.gz"
      sha256 "REPLACE_WITH_MACOS_X86_64_SHA256"
    end
  end

  def install
    bin.install "boundline"
    bin.install "canon"
  end

  def caveats
    <<~EOS
      Run boundline doctor --install after install or upgrade to verify the Boundline 0.40.0 + Canon 0.39.0 pairing.
    EOS
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/boundline --version")
  end
end
