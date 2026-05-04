#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

metadata_file="distribution/channel-metadata.toml"
boundline_version="$(sed -n 's/^version = "\([^"]*\)"$/\1/p' Cargo.toml | head -n 1)"
canon_version="$(sed -n 's/^canon_version = "\([^"]*\)"$/\1/p' "$metadata_file" | head -n 1)"

if [[ -z "$boundline_version" || -z "$canon_version" ]]; then
  echo "failed to resolve Boundline or Canon version" >&2
  exit 1
fi

windows_x86_64_sha="${WINDOWS_X86_64_SHA256:-REPLACE_WITH_WINDOWS_X86_64_SHA256}"

tag="v${boundline_version}"
homebrew_formula="distribution/homebrew/Formula/boundline.rb"
winget_root="distribution/winget/manifests/a/ApplyThe/Boundline/${boundline_version}"

mkdir -p "$(dirname "$homebrew_formula")" "$winget_root"

cat > "$homebrew_formula" <<EOF
# frozen_string_literal: true

class Boundline < Formula
  desc "Local delivery orchestrator for bounded engineering work"
  homepage "https://github.com/apply-the/boundline"
  url "https://github.com/apply-the/boundline", using: :git, tag: "${tag}"
  version "${boundline_version}"
  license "MIT"

  head "https://github.com/apply-the/boundline", branch: "main", using: :git

  depends_on "rustup" => :build

  resource "canon-source" do
    url "https://github.com/apply-the/canon", using: :git, tag: "v${canon_version}"
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
      Run boundline doctor --install after install or upgrade to verify the Boundline ${boundline_version} + Canon ${canon_version} pairing.
    EOS
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/boundline --version")
    assert_match "${canon_version}", shell_output("#{bin}/canon --version")
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
EOF

cat > "${winget_root}/ApplyThe.Boundline.yaml" <<EOF
PackageIdentifier: ApplyThe.Boundline
PackageVersion: ${boundline_version}
DefaultLocale: en-US
ManifestType: version
ManifestVersion: 1.6.0
EOF

cat > "${winget_root}/ApplyThe.Boundline.locale.en-US.yaml" <<EOF
PackageIdentifier: ApplyThe.Boundline
PackageVersion: ${boundline_version}
PackageLocale: en-US
Publisher: Apply The
PublisherUrl: https://github.com/apply-the/boundline
PublisherSupportUrl: https://github.com/apply-the/boundline/issues
Author: Apply The
PackageName: Boundline
PackageUrl: https://github.com/apply-the/boundline
ShortDescription: Boundline is a local delivery orchestrator for bounded engineering work.
Description: |
  Boundline is a local delivery orchestrator for bounded engineering work.
  It owns orchestration, bounded planning, execution,
  validation, and session continuity. The Windows release bundle installs both
  boundline and a compatible Canon companion so boundline doctor --install can verify
  the supported pairing after install or upgrade.
Moniker: boundline
License: MIT
LicenseUrl: https://github.com/apply-the/boundline/blob/main/LICENSE
ReleaseNotesUrl: https://github.com/apply-the/boundline/blob/main/CHANGELOG.md
ManifestType: defaultLocale
ManifestVersion: 1.6.0
EOF

cat > "${winget_root}/ApplyThe.Boundline.installer.yaml" <<EOF
PackageIdentifier: ApplyThe.Boundline
PackageVersion: ${boundline_version}
InstallerType: zip
NestedInstallerType: portable
Commands:
  - boundline
  - canon
Installers:
  - Architecture: x64
    InstallerUrl: https://github.com/apply-the/boundline/releases/download/${tag}/boundline-bundle-${boundline_version}-windows-x86_64.zip
    InstallerSha256: ${windows_x86_64_sha}
    NestedInstallerFiles:
      - RelativeFilePath: boundline.exe
        PortableCommandAlias: boundline
      - RelativeFilePath: canon.exe
        PortableCommandAlias: canon
ManifestType: installer
ManifestVersion: 1.6.0
EOF

echo "Synced distribution metadata for Boundline ${boundline_version} with Canon ${canon_version}."
echo "Homebrew formula: ${homebrew_formula}"
echo "winget manifests: ${winget_root}"