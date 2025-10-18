#!/bin/bash

# Clean sensitive information from git history
# WARNING: This rewrites git history and requires force push

set -e

echo "⚠️  WARNING: This will rewrite git history!"
echo ""
echo "This will:"
echo "  1. Remove your signing identity from all commits"
echo "  2. Require force push to update remote"
echo "  3. Anyone who has cloned the repo will need to re-clone"
echo ""
read -p "Continue? (yes/no) " -r
echo

if [[ ! $REPLY == "yes" ]]; then
  echo "❌ Aborted"
  exit 1
fi

# Check if git-filter-repo is installed
if ! command -v git-filter-repo &> /dev/null; then
  echo "Installing git-filter-repo..."
  if command -v brew &> /dev/null; then
    brew install git-filter-repo
  elif command -v pip3 &> /dev/null; then
    pip3 install git-filter-repo
  else
    echo "❌ Please install git-filter-repo manually:"
    echo "   brew install git-filter-repo"
    echo "   or"
    echo "   pip3 install git-filter-repo"
    exit 1
  fi
fi

# Backup current branch
CURRENT_BRANCH=$(git branch --show-current)
echo "📦 Creating backup branch: backup-before-cleanup"
git branch -f backup-before-cleanup

# Create a replacement file
cat > /tmp/replace-signing-identity.txt << 'EOF'
"signingIdentity": "Apple Development: Ekin Ertac (XRG252LVC9)"==>"signingIdentity": null
EOF

echo ""
echo "🧹 Cleaning git history..."
git filter-repo --replace-text /tmp/replace-signing-identity.txt --force

# Clean up
rm /tmp/replace-signing-identity.txt

echo ""
echo "✅ Git history cleaned!"
echo ""
echo "⚠️  IMPORTANT: You need to force push to update remote:"
echo "   git push --force-with-lease origin $CURRENT_BRANCH"
echo ""
echo "💾 A backup branch 'backup-before-cleanup' was created"
echo ""
echo "⚠️  WARNING: Anyone who cloned this repo will need to re-clone after force push"

