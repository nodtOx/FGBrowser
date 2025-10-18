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
echo "Push tag to GitHub:"
echo "  git push origin $TAG"
echo ""
echo "Or push everything:"
echo "  git push && git push --tags"

