$ErrorActionPreference = "Stop"

# Navigate to the project root
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location -Path "$ScriptDir\.."

Write-Host "Starting Boundline VitePress dev server..." -ForegroundColor Cyan
npm run site:dev
