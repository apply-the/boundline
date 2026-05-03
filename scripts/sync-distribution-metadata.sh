#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

boundline_version="$(sed -n 's/^version = "\([^"]*\)"$/\1/p' Cargo.toml | head -n 1)"
canon_version="$(sed -n 's/^canon_version = "\([^"]*\)"$/\1/p' distribution/canon-bundle.toml | head -n 1)"

if [[ -z "$boundline_version" || -z "$canon_version" ]]; then
  echo "failed to resolve Boundline or Canon version" >&2
  exit 1
fi

macos_arm64_sha="${MACOS_ARM64_SHA256:-REPLACE_WITH_MACOS_ARM64_SHA256}"
macos_x86_64_sha="${MACOS_X86_64_SHA256:-REPLACE_WITH_MACOS_X86_64_SHA256}"
windows_x86_64_sha="${WINDOWS_X86_64_SHA256:-REPLACE_WITH_WINDOWS_X86_64_SHA256}"

tag="v${boundline_version}"
homebrew_formula="distribution/homebrew/Formula/boundline.rb"
winget_root="distribution/winget/manifests/a/ApplyThe/Boundline/${boundline_version}"

mkdir -p "$(dirname "$homebrew_formula")" "$winget_root"

cat > "$homebrew_formula" <<EOF
class Boundline < Formula
  desc "Local delivery orchestrator for bounded engineering work"
  homepage "https://github.com/apply-the/boundline"
  version "${boundline_version}"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/apply-the/boundline/releases/download/${tag}/boundline-bundle-${boundline_version}-macos-arm64.tar.gz"
      sha256 "${macos_arm64_sha}"
    else
      url "https://github.com/apply-the/boundline/releases/download/${tag}/boundline-bundle-${boundline_version}-macos-x86_64.tar.gz"
      sha256 "${macos_x86_64_sha}"
    end
  end

  def install
    bin.install "boundline"
    bin.install "canon"
  end

  def caveats
    <<~EOS
      Run boundline doctor --install after install or upgrade to verify the Boundline ${boundline_version} + Canon ${canon_version} pairing.
    EOS
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/boundline --version")
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