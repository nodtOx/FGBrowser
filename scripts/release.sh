#!/bin/bash

# Create a release with git tag
# This will trigger GitHub Actions to build and deploy

set -e

VERSION=$(cat VERSION)
TAG="v$VERSION"

echo "🚀 Creating release $TAG"
echo ""

# Check if there are uncommitted changes
if [[ -n $(git status -s) ]]; then
  echo "⚠️  You have uncommitted changes:"
  git status -s
  echo ""
  read -p "Commit changes? (y/n) " -r
  if [[ $REPLY =~ ^[Yy]$ ]]; then
    git add -A
    git commit -m "chore: bump version to $VERSION"
  else
    echo "❌ Aborted. Please commit your changes first."
    exit 1
  fi
fi

# Check if tag already exists
if git rev-parse "$TAG" >/dev/null 2>&1; then
  echo "⚠️  Tag $TAG already exists"
  read -p "Delete and recreate tag? (y/n) " -r
  if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ Aborted"
    exit 1
  fi
  
  echo "Deleting old tag..."
  git tag -d "$TAG"
  git push origin ":refs/tags/$TAG" 2>/dev/null || true
fi

# Create tag
echo "Creating tag $TAG..."
git tag -a "$TAG" -m "Release $TAG"

echo ""
echo "✅ Tag created: $TAG"
echo ""
echo "Pushing tag to GitHub..."
git push
git push origin "$TAG"

REPO=$(git remote get-url origin | sed -E 's#.*[:/]([^/]+/[^/]+).*#\1#' | sed 's/\.git$//')

echo ""
echo "✅ Tag pushed to GitHub!"
echo ""
echo "🚀 Release build will start automatically in a few seconds..."
echo "📦 Repository: https://github.com/$REPO"
echo ""

# Check if gh CLI is installed
if ! command -v gh &> /dev/null; then
  echo "⚠️  GitHub CLI not installed. Build will run in background."
  echo "📦 Check progress: https://github.com/$REPO/actions"
  exit 0
fi

# Wait a moment for the workflow to start
echo "⏳ Waiting for workflow to start..."
sleep 5

# Watch the workflow run with live logs
echo ""
echo "📺 Watching build progress (live logs)..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Get the latest run and watch it
gh run watch --repo "$REPO" --exit-status || {
  echo ""
  echo "❌ Build failed!"
  echo "📦 Check logs: https://github.com/$REPO/actions"
  exit 1
}

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Build completed successfully!"
echo ""

# Now update Homebrew SHA256 automatically
echo "🔐 Updating Homebrew SHA256 hashes..."
echo ""

SKIP_WORKFLOW_WAIT=1 bash "$(dirname "$0")/update-homebrew-after-release.sh" "$VERSION" || {
  echo ""
  echo "❌ Failed to update Homebrew SHA256"
  echo "You can run manually: make update-homebrew-sha"
  exit 1
}

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎉 Release v$VERSION complete!"
echo ""
echo "Next steps:"
echo "  1. Review changes: git diff homebrew/fgbrowser.rb"
echo "  2. Commit: git add homebrew/fgbrowser.rb && git commit -m 'chore: update Homebrew SHA256 for v$VERSION'"
echo "  3. Push: git push"
echo "  4. Copy to homebrew-fgbrowser repo"

