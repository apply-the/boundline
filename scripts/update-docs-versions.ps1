# Update documentation version references based on Cargo.toml version

if (-not (Test-Path "Cargo.toml")) {
    Write-Error "Cargo.toml not found in the current directory."
    exit 1
}

if (-not (Test-Path "docs")) {
    Write-Warning "docs/ directory not found. Skipping."
    exit 0
}

$cargoToml = Get-Content -Raw -Path "Cargo.toml"
$version = $null

# Try to find version in workspace.package first
if ($cargoToml -match '(?s)\[workspace\.package\].*?version\s*=\s*"([^"]+)"') {
    $version = $Matches[1]
} elseif ($cargoToml -match '(?s)\[package\].*?version\s*=\s*"([^"]+)"') {
    $version = $Matches[1]
}

if (-not $version) {
    Write-Error "Could not extract version from Cargo.toml."
    exit 1
}

Write-Host "Updating documentation references in docs/ to version: $version"

$files = Get-ChildItem -Path "docs" -Filter "*.md" -Recurse
foreach ($file in $files) {
    $content = Get-Content -Raw -Path $file.FullName
    
    # Check if we have work to do before writing
    if ($content -match 'blob/\d+\.\d+\.\d+|tree/\d+\.\d+\.\d+|Canon \d+\.\d+\.\d+|Boundline \d+\.\d+\.\d+') {
        $newContent = $content -replace 'blob/\d+\.\d+\.\d+', "blob/$version"
        $newContent = $newContent -replace 'tree/\d+\.\d+\.\d+', "tree/$version"
        $newContent = $newContent -replace 'Canon \d+\.\d+\.\d+', "Canon $version"
        $newContent = $newContent -replace 'Boundline \d+\.\d+\.\d+', "Boundline $version"
        
        if ($content -ne $newContent) {
            # Write back using UTF-8 (No BOM)
            $utf8NoBom = New-Object System.Text.UTF8Encoding($false)
            [System.IO.File]::WriteAllText($file.FullName, $newContent, $utf8NoBom)
            Write-Host "  Updated: $($file.FullName)"
        }
    }
}

Write-Host "Done!"
