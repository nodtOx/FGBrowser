#!/usr/bin/env pwsh

# Bump version script
# Usage: .\bump-version.ps1 [patch|minor|major]

param(
    [ValidateSet('patch', 'minor', 'major')]
    [string]$BumpType = 'patch'
)

$ErrorActionPreference = "Stop"

$VERSION_FILE = "VERSION"
$BUILD_FILE = "BUILD_NUMBER"

# Read current version
$CURRENT_VERSION = (Get-Content $VERSION_FILE).Trim()

# Read and increment build number
$CURRENT_BUILD = [int](Get-Content $BUILD_FILE).Trim()
$NEW_BUILD = $CURRENT_BUILD + 1

# Parse version components
$VERSION_PARTS = $CURRENT_VERSION.Split('.')
$MAJOR = [int]$VERSION_PARTS[0]
$MINOR = [int]$VERSION_PARTS[1]
$PATCH = [int]$VERSION_PARTS[2]

# Bump version based on type
switch ($BumpType) {
    'patch' {
        $PATCH++
    }
    'minor' {
        $MINOR++
        $PATCH = 0
    }
    'major' {
        $MAJOR++
        $MINOR = 0
        $PATCH = 0
    }
}

$NEW_VERSION = "$MAJOR.$MINOR.$PATCH"

Write-Host "Bumping version: $CURRENT_VERSION → $NEW_VERSION" -ForegroundColor Cyan
Write-Host "Bumping build: $CURRENT_BUILD → $NEW_BUILD" -ForegroundColor Cyan

# Update VERSION file
$NEW_VERSION | Out-File -FilePath $VERSION_FILE -Encoding UTF8 -NoNewline

# Update BUILD_NUMBER file
$NEW_BUILD | Out-File -FilePath $BUILD_FILE -Encoding UTF8 -NoNewline

# Update package.json
if (Get-Command node -ErrorAction SilentlyContinue) {
    $pkgJson = Get-Content "package.json" | ConvertFrom-Json
    $pkgJson.version = $NEW_VERSION
    $pkgJson | ConvertTo-Json -Depth 100 | Set-Content "package.json" -Encoding UTF8
    Write-Host "Updated package.json" -ForegroundColor Green
}

# Update Cargo.toml (only the package version, not dependencies)
if (Test-Path "src-tauri/Cargo.toml") {
    $cargoLines = Get-Content "src-tauri/Cargo.toml"
    $inPackageSection = $false
    $updated = $false
    
    for ($i = 0; $i -lt $cargoLines.Count; $i++) {
        if ($cargoLines[$i] -match '^\[package\]') {
            $inPackageSection = $true
        }
        elseif ($cargoLines[$i] -match '^\[') {
            $inPackageSection = $false
        }
        
        if ($inPackageSection -and $cargoLines[$i] -match '^version\s*=\s*"[\d\.]+"' -and -not $updated) {
            $cargoLines[$i] = "version = `"$NEW_VERSION`""
            $updated = $true
        }
    }
    
    $cargoLines | Set-Content "src-tauri/Cargo.toml" -Encoding UTF8
    Write-Host "Updated Cargo.toml" -ForegroundColor Green
}

# Update Cargo.lock
if (Test-Path "src-tauri/Cargo.lock") {
    Push-Location src-tauri
    cargo update -p fgbrowser 2>$null
    if ($LASTEXITCODE -ne 0) {
        cargo check --quiet
    }
    Pop-Location
    Write-Host "Updated Cargo.lock" -ForegroundColor Green
}

# Update tauri.conf.json (required for auto-updater)
if (Test-Path "src-tauri/tauri.conf.json") {
    if (Get-Command node -ErrorAction SilentlyContinue) {
        $tauriConf = Get-Content "src-tauri/tauri.conf.json" | ConvertFrom-Json
        $tauriConf.version = $NEW_VERSION
        $tauriConf | ConvertTo-Json -Depth 100 | Set-Content "src-tauri/tauri.conf.json" -Encoding UTF8
        Write-Host "Updated tauri.conf.json" -ForegroundColor Green
    }
}

Write-Host ""
Write-Host "✨ Version bumped to $NEW_VERSION (build $NEW_BUILD)" -ForegroundColor Green
Write-Host ""

# Auto-commit the version bump
Write-Host "📝 Committing version bump..." -ForegroundColor Cyan
git add .
git commit -m "chore: bump version to $NEW_VERSION (build $NEW_BUILD)"

Write-Host "Changes committed" -ForegroundColor Green
Write-Host ""
Write-Host "Next step:" -ForegroundColor Yellow
Write-Host "  Run: .\scripts\release.ps1" -ForegroundColor White
Write-Host ""
Write-Host "This will:" -ForegroundColor Yellow
Write-Host "  Create and push tag v$NEW_VERSION"
Write-Host "  Trigger release workflow"

