#!/usr/bin/env pwsh

# Complete Release Script for FGBrowser
# Usage: .\scripts\release.ps1 [patch|minor|major] [-notes "Release notes"]

param(
    [ValidateSet('patch', 'minor', 'major')]
    [string]$BumpType = 'patch',
    
    [string]$Notes = ""
)

$ErrorActionPreference = "Stop"

# Tauri signing keys
$TAURI_PRIVATE_KEY = "[REDACTED]"
$TAURI_PASSWORD = "[REDACTED]"

Write-Host "===============================================" -ForegroundColor Cyan
Write-Host "   FGBrowser Release Automation Script" -ForegroundColor Cyan
Write-Host "===============================================" -ForegroundColor Cyan
Write-Host ""

# ========================================
# STEP 1: Bump Version
# ========================================
Write-Host "[1/5] Bumping version..." -ForegroundColor Yellow

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

Write-Host "      Version: $CURRENT_VERSION → $NEW_VERSION" -ForegroundColor Cyan
Write-Host "      Build: $CURRENT_BUILD → $NEW_BUILD" -ForegroundColor Cyan

# Update VERSION file
$NEW_VERSION | Out-File -FilePath $VERSION_FILE -Encoding UTF8 -NoNewline

# Update BUILD_NUMBER file
$NEW_BUILD | Out-File -FilePath $BUILD_FILE -Encoding UTF8 -NoNewline

# Update package.json
if (Get-Command node -ErrorAction SilentlyContinue) {
    $pkgJson = Get-Content "package.json" | ConvertFrom-Json
    $pkgJson.version = $NEW_VERSION
    $pkgJson | ConvertTo-Json -Depth 100 | Set-Content "package.json" -Encoding UTF8
    Write-Host "      ✓ Updated package.json" -ForegroundColor Green
}

# Update Cargo.toml
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
    Write-Host "      ✓ Updated Cargo.toml" -ForegroundColor Green
}

# Update Cargo.lock
if (Test-Path "src-tauri/Cargo.lock") {
    Push-Location src-tauri
    cargo update -p fgbrowser 2>$null | Out-Null
    if ($LASTEXITCODE -ne 0) {
        cargo check --quiet | Out-Null
    }
    Pop-Location
    Write-Host "      ✓ Updated Cargo.lock" -ForegroundColor Green
}

# Update tauri.conf.json
if (Test-Path "src-tauri/tauri.conf.json") {
    if (Get-Command node -ErrorAction SilentlyContinue) {
        $tauriConf = Get-Content "src-tauri/tauri.conf.json" | ConvertFrom-Json
        $tauriConf.version = $NEW_VERSION
        $tauriConf | ConvertTo-Json -Depth 100 | Set-Content "src-tauri/tauri.conf.json" -Encoding UTF8
        Write-Host "      ✓ Updated tauri.conf.json" -ForegroundColor Green
    }
}

# Commit version bump
git add .
git commit -m "chore: bump version to $NEW_VERSION (build $NEW_BUILD)"
Write-Host "      ✓ Version and build number committed" -ForegroundColor Green
Write-Host ""

# ========================================
# STEP 2: Build & Sign Installer
# ========================================
Write-Host "[2/5] Building installer..." -ForegroundColor Yellow

$env:TAURI_PRIVATE_KEY = $TAURI_PRIVATE_KEY
$env:TAURI_PRIVATE_KEY_PASSWORD = $TAURI_PASSWORD

npx tauri build --bundles nsis

if ($LASTEXITCODE -ne 0) {
    Write-Host "      ✗ Build failed!" -ForegroundColor Red
    exit 1
}

$INSTALLER_PATH = "src-tauri\target\release\bundle\nsis\FGBrowser_${NEW_VERSION}_x64-setup.exe"

if (-not (Test-Path $INSTALLER_PATH)) {
    Write-Host "      ✗ Installer not found at: $INSTALLER_PATH" -ForegroundColor Red
    exit 1
}

Write-Host "      ✓ Installer built successfully" -ForegroundColor Green
Write-Host ""

# ========================================
# STEP 3: Sign the Installer
# ========================================
Write-Host "[3/5] Signing installer..." -ForegroundColor Yellow

npx @tauri-apps/cli signer sign $INSTALLER_PATH --password $TAURI_PASSWORD

if ($LASTEXITCODE -ne 0) {
    Write-Host "      ✗ Signing failed!" -ForegroundColor Red
    exit 1
}

$SIG_PATH = "${INSTALLER_PATH}.sig"

if (-not (Test-Path $SIG_PATH)) {
    Write-Host "      ✗ Signature file not found!" -ForegroundColor Red
    exit 1
}

Write-Host "      ✓ Installer signed successfully" -ForegroundColor Green
Write-Host ""

# ========================================
# STEP 4: Update latest.json
# ========================================
Write-Host "[4/5] Updating latest.json..." -ForegroundColor Yellow

$SIGNATURE = (Get-Content $SIG_PATH -Raw).Trim()
$PUB_DATE = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")

if ([string]::IsNullOrWhiteSpace($Notes)) {
    $Notes = "Update available for FGBrowser v$NEW_VERSION"
}

$latestJson = @{
    version   = $NEW_VERSION
    notes     = $Notes
    pub_date  = $PUB_DATE
    platforms = @{
        "windows-x86_64" = @{
            signature          = $SIGNATURE
            url                = "https://github.com/[REDACTED]/FGBrowser/releases/download/v${NEW_VERSION}/FGBrowser_${NEW_VERSION}_x64-setup.exe"
            with_elevated_task = $false
        }
    }
} | ConvertTo-Json -Depth 10

$LATEST_JSON_PATH = "src-tauri\target\release\bundle\nsis\latest.json"
$latestJson | Set-Content $LATEST_JSON_PATH -Encoding UTF8

Write-Host "      ✓ latest.json updated" -ForegroundColor Green
Write-Host ""

# ========================================
# STEP 5: Create GitHub Release
# ========================================
Write-Host "[5/5] Creating GitHub release..." -ForegroundColor Yellow

# Check if gh CLI is available
if (-not (Get-Command gh -ErrorAction SilentlyContinue)) {
    Write-Host "      ✗ GitHub CLI (gh) not found!" -ForegroundColor Red
    Write-Host "      Install it from: https://cli.github.com/" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "      Manual release command:" -ForegroundColor Yellow
    Write-Host "      gh release create `"v${NEW_VERSION}`" --title `"FGBrowser v${NEW_VERSION}`" --notes `"${Notes}`" `"${INSTALLER_PATH}`" `"${SIG_PATH}`" `"${LATEST_JSON_PATH}`"" -ForegroundColor White
    exit 1
}

# Create release
gh release create "v${NEW_VERSION}" `
    --title "FGBrowser v${NEW_VERSION}" `
    --notes $Notes `
    $INSTALLER_PATH `
    $SIG_PATH `
    $LATEST_JSON_PATH

if ($LASTEXITCODE -ne 0) {
    Write-Host "      ✗ GitHub release creation failed!" -ForegroundColor Red
    exit 1
}

Write-Host "      ✓ GitHub release created" -ForegroundColor Green
Write-Host ""

# ========================================
# Success!
# ========================================
Write-Host "===============================================" -ForegroundColor Green
Write-Host "   Release v$NEW_VERSION Complete!" -ForegroundColor Green
Write-Host "===============================================" -ForegroundColor Green
Write-Host ""
Write-Host "Release URL:" -ForegroundColor Cyan
Write-Host "  https://github.com/[REDACTED]/FGBrowser/releases/tag/v${NEW_VERSION}" -ForegroundColor White
Write-Host ""
Write-Host "Auto-updater endpoint:" -ForegroundColor Cyan
Write-Host "  https://github.com/[REDACTED]/FGBrowser/releases/latest/download/latest.json" -ForegroundColor White
Write-Host ""
