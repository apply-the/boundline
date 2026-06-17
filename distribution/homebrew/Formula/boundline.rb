# frozen_string_literal: true

class Boundline < Formula
  desc "Local delivery orchestrator for bounded engineering work"
  homepage "https://github.com/apply-the/boundline"
  url "https://github.com/apply-the/boundline", using: :git, tag: "0.80.0"
  version "0.80.0"
  license "MIT"

  head "https://github.com/apply-the/boundline", branch: "main", using: :git

  depends_on "rustup" => :build

  resource "canon-source" do
    url "https://github.com/apply-the/canon", using: :git, tag: "0.72.5"
  end

  def install
    rustup_bin = Formula["rustup"].opt_bin/"rustup"
    cargo_bin = Formula["rustup"].opt_bin/"cargo"

    canon_source = buildpath/"canon-source"
    resource("canon-source").stage canon_source

    versions = [toolchain_version_for(buildpath), toolchain_version_for(canon_source)].compact.uniq
    versions = ["stable"] if versions.empty?
    versions.each do |toolchain_version|
      install_toolchain(rustup_bin, toolchain_version)
    end

    ENV["CARGO_NET_GIT_FETCH_WITH_CLI"] = "true"

    system cargo_bin, "install",
           "--locked",
           "--path", ".",
           "--root", prefix

    Dir.chdir(canon_source) do
      system cargo_bin, "install",
             "--locked",
             "--path", "crates/canon-cli",
             "--root", prefix
    end
  end

  def caveats
    <<~EOS
      Run boundline doctor --install after install or upgrade to verify the Boundline 0.80.0 + Canon 0.72.5 pairing.
    EOS
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/boundline --version")
    assert_match "0.72.5", shell_output("#{bin}/canon --version")
  end

  private

  def toolchain_version_for(root)
    toolchain_file = root/"rust-toolchain.toml"
    return nil unless toolchain_file.exist?

    toolchain_file.read[/channel\s*=\s*"([^"]+)"/, 1]
  end

  def install_toolchain(rustup_bin, toolchain_version)
    system rustup_bin, "toolchain", "install", toolchain_version,
           "--profile", "minimal",
           "--component", "rustfmt",
           "--component", "clippy",
           "--no-self-update"
  end
end
