#!/usr/bin/env pwsh

# Prepare Changelog Helper
# Shows commits since last release to help write changelog entries

$ErrorActionPreference = "Stop"

# Get latest tag
$LATEST_TAG = git describe --tags --abbrev=0 2>$null

if (-not $LATEST_TAG) {
    Write-Host "No tags found. Showing all commits." -ForegroundColor Yellow
    $LATEST_TAG = ""
    $range = "HEAD"
} else {
    Write-Host "Latest release: $LATEST_TAG" -ForegroundColor Cyan
    $range = "$LATEST_TAG..HEAD"
}

Write-Host ""
Write-Host "==================================================" -ForegroundColor Cyan
Write-Host "  Commits since $LATEST_TAG" -ForegroundColor Cyan
Write-Host "==================================================" -ForegroundColor Cyan
Write-Host ""

# Get all commits
$commits = git log $range --oneline --no-merges

if (-not $commits) {
    Write-Host "No new commits since last release." -ForegroundColor Green
    Write-Host ""
    exit 0
}

# Count commits by type
$features = @()
$fixes = @()
$docs = @()
$perf = @()
$others = @()

foreach ($commit in $commits) {
    if ($commit -match "feat:") {
        $features += $commit
    } elseif ($commit -match "fix:") {
        $fixes += $commit
    } elseif ($commit -match "docs:") {
        $docs += $commit
    } elseif ($commit -match "perf:") {
        $perf += $commit
    } else {
        $others += $commit
    }
}

# Display categorized commits
if ($features) {
    Write-Host "FEATURES ($($features.Count)):" -ForegroundColor Green
    foreach ($commit in $features) {
        Write-Host "  $commit" -ForegroundColor White
    }
    Write-Host ""
}

if ($fixes) {
    Write-Host "FIXES ($($fixes.Count)):" -ForegroundColor Yellow
    foreach ($commit in $fixes) {
        Write-Host "  $commit" -ForegroundColor White
    }
    Write-Host ""
}

if ($perf) {
    Write-Host "PERFORMANCE ($($perf.Count)):" -ForegroundColor Magenta
    foreach ($commit in $perf) {
        Write-Host "  $commit" -ForegroundColor White
    }
    Write-Host ""
}

if ($docs) {
    Write-Host "DOCUMENTATION ($($docs.Count)):" -ForegroundColor Blue
    foreach ($commit in $docs) {
        Write-Host "  $commit" -ForegroundColor White
    }
    Write-Host ""
}

if ($others) {
    Write-Host "OTHER CHANGES ($($others.Count)):" -ForegroundColor Gray
    foreach ($commit in $others) {
        Write-Host "  $commit" -ForegroundColor White
    }
    Write-Host ""
}

Write-Host "==================================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "  1. Review the commits above" -ForegroundColor White
Write-Host "  2. Edit CHANGELOG.md and add user-facing descriptions" -ForegroundColor White
Write-Host "  3. Focus on what users will notice, not technical details" -ForegroundColor White
Write-Host ""
Write-Host "Example changelog entries:" -ForegroundColor Yellow
Write-Host "  ✓ Added search result empty state" -ForegroundColor Green
Write-Host "  ✓ Fixed crash when opening game details" -ForegroundColor Green
Write-Host "  ✗ fix: resolve null pointer in game_commands.rs" -ForegroundColor Red
Write-Host "  ✗ refactor: update database query logic" -ForegroundColor Red
Write-Host ""
Write-Host "When ready to release, run:" -ForegroundColor Yellow
Write-Host "  .\scripts\release.ps1 patch -notes `"See CHANGELOG.md`"" -ForegroundColor White
Write-Host ""

