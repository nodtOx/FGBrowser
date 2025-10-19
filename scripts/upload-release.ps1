#!/usr/bin/env pwsh

# Upload release artifacts to GitHub
# Usage: .\upload-release.ps1

$ErrorActionPreference = "Stop"

$VERSION = (Get-Content "VERSION").Trim()
$TAG = "v$VERSION"

Write-Host "📤 Uploading release artifacts for $TAG" -ForegroundColor Cyan
Write-Host ""

$INSTALLER_PATH = "src-tauri\target\release\bundle\nsis\FGBrowser_${VERSION}_x64-setup.exe"
$SIG_PATH = "${INSTALLER_PATH}.sig"
$LATEST_JSON = "src-tauri\target\release\bundle\nsis\latest.json"

# Verify files exist
$files = @($INSTALLER_PATH, $SIG_PATH, $LATEST_JSON)
foreach ($file in $files) {
    if (-not (Test-Path $file)) {
        Write-Host "❌ File not found: $file" -ForegroundColor Red
        Write-Host "Run .\scripts\build-local.ps1 first" -ForegroundColor Yellow
        exit 1
    }
}

# Check if release exists
Write-Host "🔍 Checking if release $TAG exists..." -ForegroundColor Cyan
$releaseExists = gh release view $TAG 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Host "⚠️  Release $TAG does not exist. Creating it..." -ForegroundColor Yellow
    gh release create $TAG --title "FGBrowser $TAG" --notes "## FGBrowser $TAG`n`n**Windows Installer:** Download and run FGBrowser_${VERSION}_x64-setup.exe`n`nAuto-updates are enabled."
} else {
    Write-Host "✅ Release $TAG exists" -ForegroundColor Green
}

# Upload files
Write-Host ""
Write-Host "📤 Uploading files..." -ForegroundColor Cyan
gh release upload $TAG $INSTALLER_PATH $SIG_PATH $LATEST_JSON --clobber

Write-Host ""
Write-Host "✅ Upload complete!" -ForegroundColor Green
Write-Host ""
Write-Host "📦 Release URL:" -ForegroundColor Yellow
$remoteUrl = git remote get-url origin
$REPO = $remoteUrl -replace '.*[:/]([^/]+/[^/]+).*', '$1' -replace '\.git$', ''
Write-Host "  https://github.com/$REPO/releases/tag/$TAG"
Write-Host ""
Write-Host "🔄 Auto-updater endpoint:" -ForegroundColor Yellow
Write-Host "  https://github.com/$REPO/releases/latest/download/latest.json"

