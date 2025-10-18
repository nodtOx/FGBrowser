#!/bin/bash

# Export Apple Developer certificate for GitHub Actions
# This script helps you export your certificate and prepare it for GitHub secrets

set -e

echo "🍎 Export Apple Developer Certificate for GitHub Actions"
echo ""
echo "This script will help you:"
echo "  1. Export your Apple Developer certificate as .p12"
echo "  2. Convert it to base64 for GitHub secrets"
echo "  3. Show you what secrets to add to GitHub"
echo ""

# Find available signing identities
echo "📋 Available signing identities:"
security find-identity -v -p codesigning

echo ""
read -p "Enter the certificate name (e.g., 'Apple Development: Your Name'): " CERT_NAME
read -sp "Enter a password for the .p12 file (you'll use this in GitHub secrets): " CERT_PASSWORD
echo ""

# Export certificate
OUTPUT_FILE="$HOME/Desktop/apple-dev-cert.p12"

echo ""
echo "🔐 Exporting certificate..."
security find-certificate -c "$CERT_NAME" -p > /tmp/cert.pem
security find-certificate -c "$CERT_NAME" -a -Z | grep ^SHA-1 | awk '{print $3}' > /tmp/cert-hash.txt
CERT_HASH=$(cat /tmp/cert-hash.txt)

security export -k ~/Library/Keychains/login.keychain-db -t identities -f pkcs12 -o "$OUTPUT_FILE" -P "$CERT_PASSWORD"

echo "✅ Certificate exported to: $OUTPUT_FILE"
echo ""

# Convert to base64
echo "📝 Converting to base64..."
BASE64_CERT=$(base64 -i "$OUTPUT_FILE")

# Create temporary file with instructions
INSTRUCTIONS_FILE="$HOME/Desktop/github-secrets-instructions.txt"

cat > "$INSTRUCTIONS_FILE" << EOF
🔐 GitHub Secrets Setup Instructions
====================================

Add these secrets to your GitHub repository:
Settings → Secrets and variables → Actions → New repository secret

1. APPLE_CERTIFICATE
   Value: (paste the base64 certificate below)
   
$BASE64_CERT

2. APPLE_CERTIFICATE_PASSWORD
   Value: $CERT_PASSWORD

3. KEYCHAIN_PASSWORD
   Value: (create a secure random password, e.g., $(openssl rand -base64 32))

4. APPLE_SIGNING_IDENTITY
   Value: $CERT_NAME

⚠️  Important:
- Keep these secrets secure
- Delete this file after adding secrets to GitHub
- You can delete $OUTPUT_FILE after this is done

🔗 Add secrets here:
https://github.com/YOUR_USERNAME/YOUR_REPO/settings/secrets/actions
EOF

echo ""
echo "✅ Instructions saved to: $INSTRUCTIONS_FILE"
echo ""
echo "📋 Next steps:"
echo "  1. Open the instructions file: open $INSTRUCTIONS_FILE"
echo "  2. Add the secrets to GitHub (link in the file)"
echo "  3. Delete both files when done for security"
echo ""
echo "🗑️  Cleanup commands (run after adding secrets):"
echo "  rm $OUTPUT_FILE"
echo "  rm $INSTRUCTIONS_FILE"

# Cleanup temp files
rm -f /tmp/cert.pem /tmp/cert-hash.txt

