#!/bin/bash
# FitBoy Database Update Script
# Runs on server to update database and publish it

DEPLOY_DIR="/opt/fitboy"
WEB_DIR="/var/www/fitboy"
DB_PATH="$DEPLOY_DIR/repacks.db"

cd "$DEPLOY_DIR"

echo "[$(date)] Starting update..."

# Crawl latest pages
./cli -d "$DB_PATH" crawl -p 5 2>&1 | tail -5

# Update popular repacks
./cli -d "$DB_PATH" popular 2>&1 | tail -3

# Get stats
STATS=$(./cli -d "$DB_PATH" stats 2>/dev/null)
GAMES=$(echo "$STATS" | grep "Total Games:" | awk '{print $3}')
MAGNETS=$(echo "$STATS" | grep "Total Magnet" | awk '{print $4}')

if [ -f "$DB_PATH" ]; then
    # Get size
    SIZE=$(stat -c%s "$DB_PATH" 2>/dev/null || stat -f%z "$DB_PATH")
    
    # Create metadata
    cat > "$WEB_DIR/metadata.json" << EOF
{
  "updated": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "games": $GAMES,
  "magnets": $MAGNETS,
  "size": $SIZE,
  "url": "http://157.230.16.45/repacks.db",
  "checksum": "$(sha256sum "$DB_PATH" | cut -d' ' -f1)"
}
EOF
    
    # Copy to web directory
    cp "$DB_PATH" "$WEB_DIR/repacks.db"
    
    echo "[$(date)] ✅ Published: $GAMES games, $((SIZE/1024/1024))MB"
else
    echo "[$(date)] ❌ Database not found at: $DB_PATH"
    exit 1
fi

