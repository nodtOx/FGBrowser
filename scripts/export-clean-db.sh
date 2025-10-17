#!/bin/bash

# Export database for server (with empty user-specific tables)
# This creates a clean database with game data but truncated settings/downloads

DB_FILE="repacks.db"
CLEAN_DB="repacks-clean.db"

echo "Creating clean database export..."

# Remove existing clean database if it exists
rm -f "$CLEAN_DB" "$CLEAN_DB-shm" "$CLEAN_DB-wal"

# Copy entire database
cp "$DB_FILE" "$CLEAN_DB"

# Truncate user-specific tables (keep schema, remove data)
sqlite3 "$CLEAN_DB" <<EOF
-- Keep tables but remove user-specific data
DELETE FROM settings;
DELETE FROM downloads;
VACUUM;
EOF

echo "✅ Clean database created: $CLEAN_DB"
echo ""
echo "📊 Database sizes:"
du -h "$DB_FILE" "$CLEAN_DB"
echo ""
echo "To upload to server, run:"
echo "  rsync -avz --progress --checksum $CLEAN_DB root@157.230.16.45:/var/www/fgbrowser/repacks.db"

