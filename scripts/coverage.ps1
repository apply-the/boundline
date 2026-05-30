Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

$env:DISABLE_AUTO_UPDATE = 'true'

& cargo llvm-cov clean --workspace
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}

& cargo llvm-cov --workspace --all-features --lcov --output-path lcov.workspace.info @args
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}