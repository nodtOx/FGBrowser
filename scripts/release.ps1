#!/usr/bin/env pwsh

# Create a release with git tag
# This will trigger GitHub Actions to build and deploy

$ErrorActionPreference = "Stop"

$VERSION = (Get-Content "VERSION").Trim()
$TAG = "v$VERSION"

Write-Host "🚀 Creating release $TAG" -ForegroundColor Cyan
Write-Host ""

# Check if there are uncommitted changes
$gitStatus = git status -s
if ($gitStatus) {
    Write-Host "⚠️  You have uncommitted changes:" -ForegroundColor Yellow
    git status -s
    Write-Host ""
    $response = Read-Host "Commit changes? (y/n)"
    if ($response -match '^[Yy]$') {
        git add -A
        git commit -m "chore: bump version to $VERSION"
    } else {
        Write-Host "❌ Aborted. Please commit your changes first." -ForegroundColor Red
        exit 1
    }
}

# Check if tag already exists
$tagExists = git rev-parse "$TAG" 2>$null
if ($LASTEXITCODE -eq 0) {
    Write-Host "⚠️  Tag $TAG already exists" -ForegroundColor Yellow
    $response = Read-Host "Delete and recreate tag? (y/n)"
    if ($response -notmatch '^[Yy]$') {
        Write-Host "❌ Aborted" -ForegroundColor Red
        exit 1
    }
    
    Write-Host "Deleting old tag..." -ForegroundColor Cyan
    git tag -d "$TAG"
    git push origin ":refs/tags/$TAG" 2>$null
}

# Create tag
Write-Host "Creating tag $TAG..." -ForegroundColor Cyan
git tag -a "$TAG" -m "Release $TAG"

Write-Host ""
Write-Host "✅ Tag created: $TAG" -ForegroundColor Green
Write-Host ""
Write-Host "Pushing tag to GitHub..." -ForegroundColor Cyan
git push
git push origin "$TAG"

$remoteUrl = git remote get-url origin
$REPO = $remoteUrl -replace '.*[:/]([^/]+/[^/]+).*', '$1' -replace '\.git$', ''

Write-Host ""
Write-Host "✅ Tag pushed to GitHub!" -ForegroundColor Green
Write-Host ""
Write-Host "🚀 Release build will start automatically in a few seconds..." -ForegroundColor Cyan
Write-Host "📦 Repository: https://github.com/$REPO"
Write-Host ""

# Check if gh CLI is installed
if (-not (Get-Command gh -ErrorAction SilentlyContinue)) {
    Write-Host "⚠️  GitHub CLI not installed. Build will run in background." -ForegroundColor Yellow
    Write-Host "📦 Check progress: https://github.com/$REPO/actions"
    exit 0
}

# Wait a moment for the workflow to start
Write-Host "⏳ Waiting for Release workflow to start..." -ForegroundColor Cyan
Start-Sleep -Seconds 10

# Get the latest Release workflow run ID
Write-Host "🔍 Finding Release workflow run..." -ForegroundColor Cyan
$RUN_ID = (gh run list --repo $REPO --workflow "Release" --limit 1 --json databaseId --jq '.[0].databaseId')

if (-not $RUN_ID) {
    Write-Host "❌ Could not find Release workflow run" -ForegroundColor Red
    Write-Host "📦 Check manually: https://github.com/$REPO/actions"
    exit 1
}

# Watch the Release workflow with live logs
Write-Host ""
Write-Host "📺 Watching Release workflow (Run ID: $RUN_ID)..." -ForegroundColor Cyan
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Write-Host ""

gh run watch $RUN_ID --repo $REPO --exit-status
if ($LASTEXITCODE -ne 0) {
    Write-Host ""
    Write-Host "❌ Release build failed!" -ForegroundColor Red
    Write-Host "📦 Check logs: https://github.com/$REPO/actions/runs/$RUN_ID"
    exit 1
}

Write-Host ""
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
Write-Host "✅ Build completed successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "🎉 Release v$VERSION complete!" -ForegroundColor Green
Write-Host ""
Write-Host "📦 Download: https://github.com/$REPO/releases/tag/v$VERSION"

