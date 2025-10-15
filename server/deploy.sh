#!/bin/bash
# FitBoy Database Server Deployment Script
# Deploy to: 157.230.16.45

set -e  # Exit on any error

SERVER="157.230.16.45"
SERVER_USER="root"
DEPLOY_DIR="/opt/fitboy"
WEB_DIR="/var/www/fitboy"

echo "🚀 FitBoy Server Deployment"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Server: $SERVER"
echo ""

# Check SSH connection
echo "🔍 Testing SSH connection..."
if ! ssh -o ConnectTimeout=5 $SERVER_USER@$SERVER "echo 'OK'" &>/dev/null; then
    echo "❌ Cannot connect. Setup SSH key first:"
    echo "   ssh-copy-id $SERVER_USER@$SERVER"
    exit 1
fi
echo "✅ Connected"

# Get latest CLI from GitHub releases
echo ""
echo "📥 Downloading latest CLI from GitHub..."
LATEST_RELEASE=$(curl -s https://api.github.com/repos/yourusername/fit-boy/releases/latest | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST_RELEASE" ]; then
    echo "⚠️  No GitHub release found. Building locally instead..."
    echo ""
    echo "🔨 Building CLI locally..."
    cd ../desktop/src-tauri
    cargo build --bin cli --release
    CLI_PATH="target/release/cli"
else
    echo "✅ Found release: $LATEST_RELEASE"
    DOWNLOAD_URL="https://github.com/yourusername/fit-boy/releases/download/$LATEST_RELEASE/cli"
    curl -L -o /tmp/fitboy-cli "$DOWNLOAD_URL"
    chmod +x /tmp/fitboy-cli
    CLI_PATH="/tmp/fitboy-cli"
fi

# Deploy
echo ""
echo "📦 Deploying to server..."

# Create directories and install nginx
ssh $SERVER_USER@$SERVER << 'ENDSSH'
set -e
echo "  → Installing dependencies..."
mkdir -p /opt/fitboy /var/www/fitboy /var/log/fitboy
apt-get update -qq
apt-get install -y nginx curl &>/dev/null
ENDSSH

# Upload CLI binary
echo "  → Uploading CLI..."
scp "$CLI_PATH" $SERVER_USER@$SERVER:$DEPLOY_DIR/cli
ssh $SERVER_USER@$SERVER "chmod +x $DEPLOY_DIR/cli"

# Upload nginx config
cd ../server 2>/dev/null || cd server
echo "  → Uploading nginx config..."
scp nginx.conf $SERVER_USER@$SERVER:/tmp/fitboy.nginx

# Upload update script
echo "  → Uploading update script..."
scp update.sh $SERVER_USER@$SERVER:$DEPLOY_DIR/update.sh
ssh $SERVER_USER@$SERVER "chmod +x $DEPLOY_DIR/update.sh"

# Setup nginx
echo "  → Configuring nginx..."
ssh $SERVER_USER@$SERVER << 'ENDSSH'
mv /tmp/fitboy.nginx /etc/nginx/sites-available/fitboy
ln -sf /etc/nginx/sites-available/fitboy /etc/nginx/sites-enabled/fitboy
rm -f /etc/nginx/sites-enabled/default
nginx -t && systemctl reload nginx
ENDSSH

# Setup cron
echo "  → Setting up auto-update (every 6 hours)..."
ssh $SERVER_USER@$SERVER << 'ENDSSH'
(crontab -l 2>/dev/null | grep -v fitboy; echo "0 */6 * * * /opt/fitboy/update.sh >> /var/log/fitboy/update.log 2>&1") | crontab -
ENDSSH

# Run initial update
echo ""
echo "🔄 Running initial update..."
ssh $SERVER_USER@$SERVER "/opt/fitboy/update.sh"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Deployment Complete!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📍 Database URL:"
echo "   http://157.230.16.45/repacks.db"
echo ""
echo "📊 Metadata:"
echo "   http://157.230.16.45/metadata.json"
echo ""
echo "🔄 Updates: Every 6 hours"
echo ""
echo "📝 Server commands:"
echo "   ssh $SERVER_USER@$SERVER"
echo "   /opt/fitboy/update.sh              # Manual update"
echo "   tail -f /var/log/fitboy/update.log # Watch logs"
echo "   /opt/fitboy/cli stats              # View stats"
echo ""

# Test
if curl -s http://157.230.16.45/metadata.json | grep -q "games"; then
    echo "✅ Server is responding!"
    echo ""
    curl -s http://157.230.16.45/metadata.json | python3 -m json.tool 2>/dev/null
else
    echo "⚠️  Give it a minute for the initial update to complete"
fi

echo ""
echo "🎉 Done!"
