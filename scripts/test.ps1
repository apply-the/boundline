Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

& cargo nextest run --workspace --all-features --no-fail-fast @args
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}