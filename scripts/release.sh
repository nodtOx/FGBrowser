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
  read -p "Commit changes? (y/n) " -n 1 -r
  echo
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
  echo "❌ Tag $TAG already exists"
  exit 1
fi

# Create and push tag
echo "Creating tag $TAG..."
git tag -a "$TAG" -m "Release $TAG"

echo ""
echo "✅ Tag created: $TAG"
echo ""
echo "Push tag? (y/n)"
read -p "> " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
  git push
  git push --tags
  
  REPO=$(git remote get-url origin | sed 's/.*github.com[:/]\(.*\)\.git/\1/')
  
  echo ""
  echo "✅ Tag pushed to GitHub!"
  echo ""
  echo "🚀 To trigger the release build:"
  echo ""
  echo "   1. Go to: https://github.com/$REPO/actions/workflows/release.yml"
  echo "   2. Click 'Run workflow'"
  echo "   3. Enter version: $TAG"
  echo "   4. Click 'Run workflow' button"
  echo ""
  echo "Or run this command:"
  echo "   gh workflow run Release -f version=$TAG"
  echo ""
  echo "📋 View releases: https://github.com/$REPO/releases"
else
  echo ""
  echo "📝 To push later:"
  echo "  git push && git push --tags"
fi

