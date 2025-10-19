#!/bin/bash

# Bump version script
# Usage: ./bump-version.sh [patch|minor|major]

set -e

BUMP_TYPE=${1:-patch}
VERSION_FILE="VERSION"

# Read current version
CURRENT_VERSION=$(cat $VERSION_FILE)

# Parse version components
IFS='.' read -ra VERSION_PARTS <<< "$CURRENT_VERSION"
MAJOR=${VERSION_PARTS[0]}
MINOR=${VERSION_PARTS[1]}
PATCH=${VERSION_PARTS[2]}

# Bump version based on type
case $BUMP_TYPE in
  patch)
    PATCH=$((PATCH + 1))
    ;;
  minor)
    MINOR=$((MINOR + 1))
    PATCH=0
    ;;
  major)
    MAJOR=$((MAJOR + 1))
    MINOR=0
    PATCH=0
    ;;
  *)
    echo "❌ Invalid bump type: $BUMP_TYPE"
    echo "Usage: $0 [patch|minor|major]"
    exit 1
    ;;
esac

NEW_VERSION="$MAJOR.$MINOR.$PATCH"

echo "📦 Bumping version: $CURRENT_VERSION → $NEW_VERSION"

# Update VERSION file
echo $NEW_VERSION > $VERSION_FILE

# Update package.json
if command -v node &> /dev/null; then
  node -e "const pkg = require('./package.json'); pkg.version = '$NEW_VERSION'; require('fs').writeFileSync('package.json', JSON.stringify(pkg, null, 2) + '\n');"
  echo "✅ Updated package.json"
fi

# Update Cargo.toml
if [ -f "src-tauri/Cargo.toml" ]; then
  sed -i.bak "s/^version = \".*\"/version = \"$NEW_VERSION\"/" src-tauri/Cargo.toml
  rm -f src-tauri/Cargo.toml.bak
  echo "✅ Updated Cargo.toml"
fi

# Update Cargo.lock
if [ -f "src-tauri/Cargo.lock" ]; then
  cd src-tauri && cargo update -p fgbrowser --precise $NEW_VERSION 2>/dev/null || cargo check --quiet
  cd ..
  echo "✅ Updated Cargo.lock"
fi

# Update tauri.conf.json (required for auto-updater)
if [ -f "src-tauri/tauri.conf.json" ]; then
  if command -v node &> /dev/null; then
    node -e "const conf = require('./src-tauri/tauri.conf.json'); conf.version = '$NEW_VERSION'; require('fs').writeFileSync('src-tauri/tauri.conf.json', JSON.stringify(conf, null, 2) + '\n');"
    echo "✅ Updated tauri.conf.json"
  fi
fi

# Update Homebrew cask
if [ -f "homebrew/fgbrowser.rb" ]; then
  sed -i.bak "s/version \".*\"/version \"$NEW_VERSION\"/" homebrew/fgbrowser.rb
  rm -f homebrew/fgbrowser.rb.bak
  echo "✅ Updated homebrew/fgbrowser.rb"
fi

echo ""
echo "✨ Version bumped to $NEW_VERSION"
echo ""
echo "Next steps:"
echo "  1. Build locally: make build"
echo "  2. Update Homebrew SHA256: ./scripts/update-homebrew-sha.sh"
echo "  3. Review changes: git diff"
echo "  4. Commit: git add -A && git commit -m \"chore: bump version to $NEW_VERSION\""
echo "  5. Create release: make release"

