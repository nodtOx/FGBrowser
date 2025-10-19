#!/usr/bin/env pwsh

# Build and sign Windows installer locally
# Usage: .\build-local.ps1

param(
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"

$VERSION = (Get-Content "VERSION").Trim()

Write-Host "🔨 Building FGBrowser v$VERSION locally" -ForegroundColor Cyan
Write-Host ""

# Check for signing keys in environment
if (-not $env:TAURI_PRIVATE_KEY) {
    Write-Host "⚠️  TAURI_PRIVATE_KEY not set" -ForegroundColor Yellow
    Write-Host "Please set the signing keys:" -ForegroundColor Yellow
    Write-Host '  $env:TAURI_PRIVATE_KEY = "..."' -ForegroundColor White
    Write-Host '  $env:TAURI_PRIVATE_KEY_PASSWORD = "..."' -ForegroundColor White
    Write-Host ""
    Write-Host "You can find them in .notes/tauri_update_keys.md" -ForegroundColor Cyan
    exit 1
}

if (-not $SkipBuild) {
    Write-Host "📦 Building frontend..." -ForegroundColor Cyan
    npm run build
    
    Write-Host ""
    Write-Host "🦀 Building Rust application..." -ForegroundColor Cyan
    Push-Location src-tauri
    cargo build --release
    Pop-Location
}

Write-Host ""
Write-Host "📦 Creating NSIS installer..." -ForegroundColor Cyan
npx tauri build --bundles nsis

$INSTALLER_PATH = "src-tauri\target\release\bundle\nsis\FGBrowser_${VERSION}_x64-setup.exe"
$SIG_PATH = "${INSTALLER_PATH}.sig"

# Sign the installer
Write-Host ""
Write-Host "✍️  Signing installer..." -ForegroundColor Cyan
npx @tauri-apps/cli signer sign $INSTALLER_PATH --password $env:TAURI_PRIVATE_KEY_PASSWORD

if (Test-Path $SIG_PATH) {
    Write-Host "✅ Installer signed successfully" -ForegroundColor Green
} else {
    Write-Host "❌ Signing failed!" -ForegroundColor Red
    exit 1
}

# Read signature
$SIGNATURE = Get-Content $SIG_PATH -Raw

# Create latest.json
Write-Host ""
Write-Host "📝 Creating latest.json..." -ForegroundColor Cyan

$latestJson = @{
    version = $VERSION
    notes = "Update available for FGBrowser"
    pub_date = (Get-Date -Format "yyyy-MM-ddTHH:mm:ssZ")
    platforms = @{
        "windows-x86_64" = @{
            signature = $SIGNATURE.Trim()
            url = "https://github.com/nodtOx/FGBrowser/releases/download/v${VERSION}/FGBrowser_${VERSION}_x64-setup.exe"
            with_elevated_task = $false
        }
    }
} | ConvertTo-Json -Depth 10

$latestJson | Set-Content "src-tauri\target\release\bundle\nsis\latest.json" -Encoding UTF8

Write-Host "✅ latest.json created" -ForegroundColor Green
Write-Host ""
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
Write-Host "✅ Build complete!" -ForegroundColor Green
Write-Host ""
Write-Host "📦 Files ready for upload:" -ForegroundColor Yellow
Write-Host "  - $INSTALLER_PATH"
Write-Host "  - $SIG_PATH"
Write-Host "  - src-tauri\target\release\bundle\nsis\latest.json"
Write-Host ""
Write-Host "Next step:" -ForegroundColor Yellow
Write-Host "  Run: .\scripts\upload-release.ps1" -ForegroundColor White

