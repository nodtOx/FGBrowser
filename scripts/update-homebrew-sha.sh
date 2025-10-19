#!/bin/bash

# Update Homebrew SHA256 hashes
# Run this after `make build` to update the SHA256 hashes in the Homebrew cask

set -e

CASK_FILE="homebrew/fgbrowser.rb"
DMG_DIR="src-tauri/target/release/bundle/dmg"

if [ ! -d "$DMG_DIR" ]; then
  echo "❌ DMG directory not found: $DMG_DIR"
  echo "Please run 'make build' first to create the DMG files."
  exit 1
fi

# Find DMG files
AARCH64_DMG=$(find "$DMG_DIR" -name "*_aarch64.dmg" | head -n 1)
X86_64_DMG=$(find "$DMG_DIR" -name "*_x86_64.dmg" | head -n 1)

if [ -z "$AARCH64_DMG" ] && [ -z "$X86_64_DMG" ]; then
  echo "❌ No DMG files found in $DMG_DIR"
  echo "Please run 'make build' first to create the DMG files."
  exit 1
fi

echo "📦 Calculating SHA256 hashes for Homebrew Cask..."
echo ""

# Calculate SHA256 for ARM64
if [ -n "$AARCH64_DMG" ]; then
  AARCH64_SHA=$(shasum -a 256 "$AARCH64_DMG" | awk '{print $1}')
  echo "ARM64 (aarch64):"
  echo "  File: $(basename "$AARCH64_DMG")"
  echo "  SHA256: $AARCH64_SHA"
  echo ""
  
  # Update in cask file
  if [[ "$OSTYPE" == "darwin"* ]]; then
    sed -i '' "/on_arm do/,/end/ s/sha256 \".*\"/sha256 \"$AARCH64_SHA\"/" "$CASK_FILE"
  else
    sed -i "/on_arm do/,/end/ s/sha256 \".*\"/sha256 \"$AARCH64_SHA\"/" "$CASK_FILE"
  fi
fi

# Calculate SHA256 for Intel
if [ -n "$X86_64_DMG" ]; then
  X86_64_SHA=$(shasum -a 256 "$X86_64_DMG" | awk '{print $1}')
  echo "Intel (x86_64):"
  echo "  File: $(basename "$X86_64_DMG")"
  echo "  SHA256: $X86_64_SHA"
  echo ""
  
  # Update in cask file
  if [[ "$OSTYPE" == "darwin"* ]]; then
    sed -i '' "/on_intel do/,/end/ s/sha256 \".*\"/sha256 \"$X86_64_SHA\"/" "$CASK_FILE"
  else
    sed -i "/on_intel do/,/end/ s/sha256 \".*\"/sha256 \"$X86_64_SHA\"/" "$CASK_FILE"
  fi
fi

echo "✅ Updated $CASK_FILE with new SHA256 hashes"
echo ""
echo "Next steps:"
echo "  1. Review changes: git diff $CASK_FILE"
echo "  2. Commit: git add $CASK_FILE && git commit -m \"chore: update Homebrew SHA256 hashes\""
echo "  3. Push to homebrew-fgbrowser repo"

