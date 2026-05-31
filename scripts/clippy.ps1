Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

& cargo clippy --workspace --all-targets --all-features @args -- -D warnings
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}