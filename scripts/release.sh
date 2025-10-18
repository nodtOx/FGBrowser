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
  echo "⚠️  Tag $TAG already exists"
  read -p "Delete and recreate tag? (y/n) " -n 1 -r
  echo
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
git push --tags

REPO=$(git remote get-url origin | sed -E 's#.*[:/]([^/]+/[^/]+).*#\1#' | sed 's/\.git$//')

echo ""
echo "✅ Tag pushed to GitHub!"
echo ""
echo "🚀 Release build will start automatically in a few seconds..."
echo ""
echo "📦 Check GitHub Actions: https://github.com/$REPO/actions"
echo "📋 View releases: https://github.com/$REPO/releases"
echo ""
echo "💡 Tip: The release will be created when all platform builds complete successfully."

