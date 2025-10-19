#!/bin/bash

# Wait for GitHub Actions to complete, download DMGs, calculate SHA256, and update Homebrew cask
# Usage: ./scripts/update-homebrew-after-release.sh [version]
#
# For authentication, either:
# 1. Set GH_TOKEN_NODTOX environment variable with a GitHub personal access token
# 2. Use default gh CLI authentication

set -e

VERSION=${1:-$(cat VERSION)}
TAG="v$VERSION"
CASK_FILE="homebrew/fgbrowser.rb"
TEMP_DIR=$(mktemp -d)

# Extract repo from git remote
REPO=$(git remote get-url origin | sed -E 's#.*[:/]([^/]+/[^/]+).*#\1#' | sed 's/\.git$//')

echo "🔍 Waiting for GitHub Actions to complete for $TAG..."
echo "📦 Repository: $REPO"
echo ""

# Check if gh CLI is installed
if ! command -v gh &> /dev/null; then
  echo "❌ GitHub CLI (gh) is not installed."
  echo "Install it with: brew install gh"
  exit 1
fi

# Use GH_TOKEN_NODTOX if available, otherwise use default gh auth
if [ -n "$GH_TOKEN_NODTOX" ]; then
  export GH_TOKEN="$GH_TOKEN_NODTOX"
  echo "✅ Using GH_TOKEN_NODTOX for authentication"
elif [ -z "$GH_TOKEN" ]; then
  echo "ℹ️  Using default gh CLI authentication"
  # Check if authenticated
  gh auth status &>/dev/null || {
    echo "❌ Not authenticated with GitHub CLI"
    echo "Either run 'gh auth login' or set GH_TOKEN_NODTOX environment variable"
    exit 1
  }
fi

# Wait for workflow to complete
echo "⏳ Checking workflow status..."
WORKFLOW_STATUS=""
ATTEMPTS=0
MAX_ATTEMPTS=60  # 10 minutes max (10 seconds * 60)

while [ "$WORKFLOW_STATUS" != "completed" ] && [ $ATTEMPTS -lt $MAX_ATTEMPTS ]; do
  # Get the latest workflow run for this tag
  WORKFLOW_STATUS=$(gh run list --repo "$REPO" --limit 1 --json status,headBranch --jq ".[] | select(.headBranch == \"$TAG\") | .status" 2>/dev/null || echo "")
  
  if [ -z "$WORKFLOW_STATUS" ]; then
    echo "⏳ Waiting for workflow to start... (attempt $((ATTEMPTS+1))/$MAX_ATTEMPTS)"
  elif [ "$WORKFLOW_STATUS" = "in_progress" ] || [ "$WORKFLOW_STATUS" = "queued" ]; then
    echo "⏳ Build in progress... (attempt $((ATTEMPTS+1))/$MAX_ATTEMPTS)"
  elif [ "$WORKFLOW_STATUS" = "completed" ]; then
    # Check if it succeeded
    CONCLUSION=$(gh run list --repo "$REPO" --limit 1 --json conclusion,headBranch --jq ".[] | select(.headBranch == \"$TAG\") | .conclusion")
    if [ "$CONCLUSION" != "success" ]; then
      echo "❌ Workflow failed with status: $CONCLUSION"
      echo "Check: https://github.com/$REPO/actions"
      exit 1
    fi
    echo "✅ Build completed successfully!"
    break
  fi
  
  sleep 10
  ATTEMPTS=$((ATTEMPTS+1))
done

if [ $ATTEMPTS -ge $MAX_ATTEMPTS ]; then
  echo "❌ Timeout waiting for workflow to complete"
  echo "Check status: https://github.com/$REPO/actions"
  exit 1
fi

echo ""
echo "📥 Downloading DMG files from release $TAG..."

# Download DMG files
cd "$TEMP_DIR"

AARCH64_DMG="FGBrowser_${VERSION}_aarch64.dmg"
X86_64_DMG="FGBrowser_${VERSION}_x86_64.dmg"

echo "Downloading $AARCH64_DMG..."
gh release download "$TAG" --repo "$REPO" --pattern "*_aarch64.dmg" 2>/dev/null || {
  echo "⚠️  ARM64 DMG not found, trying without pattern..."
  gh release download "$TAG" --repo "$REPO" 2>/dev/null || {
    echo "❌ Failed to download release assets"
    exit 1
  }
}

echo "Downloading $X86_64_DMG..."
gh release download "$TAG" --repo "$REPO" --pattern "*_x86_64.dmg" 2>/dev/null || true

cd -

echo ""
echo "🔐 Calculating SHA256 hashes..."
echo ""

# Find downloaded DMG files
AARCH64_FILE=$(find "$TEMP_DIR" -name "*_aarch64.dmg" | head -n 1)
X86_64_FILE=$(find "$TEMP_DIR" -name "*_x86_64.dmg" | head -n 1)

# Calculate and update SHA256 for ARM64
if [ -n "$AARCH64_FILE" ] && [ -f "$AARCH64_FILE" ]; then
  AARCH64_SHA=$(shasum -a 256 "$AARCH64_FILE" | awk '{print $1}')
  echo "ARM64 (aarch64):"
  echo "  File: $(basename "$AARCH64_FILE")"
  echo "  SHA256: $AARCH64_SHA"
  echo ""
  
  # Update in cask file
  if [[ "$OSTYPE" == "darwin"* ]]; then
    sed -i '' "/on_arm do/,/end/ s/sha256 \".*\"/sha256 \"$AARCH64_SHA\"/" "$CASK_FILE"
  else
    sed -i "/on_arm do/,/end/ s/sha256 \".*\"/sha256 \"$AARCH64_SHA\"/" "$CASK_FILE"
  fi
else
  echo "⚠️  ARM64 DMG not found, skipping..."
fi

# Calculate and update SHA256 for Intel
if [ -n "$X86_64_FILE" ] && [ -f "$X86_64_FILE" ]; then
  X86_64_SHA=$(shasum -a 256 "$X86_64_FILE" | awk '{print $1}')
  echo "Intel (x86_64):"
  echo "  File: $(basename "$X86_64_FILE")"
  echo "  SHA256: $X86_64_SHA"
  echo ""
  
  # Update in cask file
  if [[ "$OSTYPE" == "darwin"* ]]; then
    sed -i '' "/on_intel do/,/end/ s/sha256 \".*\"/sha256 \"$X86_64_SHA\"/" "$CASK_FILE"
  else
    sed -i "/on_intel do/,/end/ s/sha256 \".*\"/sha256 \"$X86_64_SHA\"/" "$CASK_FILE"
  fi
else
  echo "⚠️  Intel DMG not found, skipping..."
fi

# Cleanup
rm -rf "$TEMP_DIR"

echo "✅ Updated $CASK_FILE with new SHA256 hashes"
echo ""
echo "📋 Changes:"
git diff "$CASK_FILE"
echo ""
echo "Next steps:"
echo "  1. Review changes above"
echo "  2. Commit: git add $CASK_FILE && git commit -m \"chore: update Homebrew SHA256 for $TAG\""
echo "  3. Push: git push"
echo "  4. Copy to homebrew-fgbrowser repo"

