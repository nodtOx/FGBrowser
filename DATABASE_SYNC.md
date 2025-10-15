# Database Sync Strategy

## Overview

Instead of making users crawl the entire FitGirl Repacks site on first launch (which can take hours), provide a pre-populated database that they can download from your server.

## Architecture

### Server Side

**Your Server Setup:**

1. Run the CLI tool on your server with cron job to keep database updated
2. Host the database file via HTTP/CDN
3. Optionally host database metadata (size, last_updated, game_count)

**Update Script on Server:**

```bash
#!/bin/bash
# /opt/fitboy/update_and_publish.sh

cd /opt/fitboy/desktop/src-tauri

# Update database
./target/release/cli crawl -p 10
./target/release/cli popular

# Get stats for metadata
GAME_COUNT=$(./target/release/cli stats | grep "Total Games:" | awk '{print $3}')
DB_SIZE=$(stat -f%z ../../repacks.db)
LAST_UPDATED=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Create metadata file
cat > /var/www/fitboy/db_metadata.json <<EOF
{
  "version": "$(date +%Y%m%d%H%M%S)",
  "last_updated": "$LAST_UPDATED",
  "game_count": $GAME_COUNT,
  "size_bytes": $DB_SIZE,
  "url": "https://your-server.com/fitboy/repacks.db",
  "checksum": "$(sha256sum ../../repacks.db | awk '{print $1}')"
}
EOF

# Copy database to web directory
cp ../../repacks.db /var/www/fitboy/repacks.db

echo "Database published successfully!"
```

**Cron Job:**

```bash
# Update every 6 hours
0 */6 * * * /opt/fitboy/update_and_publish.sh >> /var/log/fitboy_sync.log 2>&1
```

### Client Side (Future Implementation)

**First Launch Flow:**

```
User opens app
  ↓
Check if repacks.db exists locally
  ↓ NO
Show "Download Database" dialog
  ├─ Option 1: Download pre-populated DB (~50-200 MB) [Recommended]
  │   └─ Progress bar with speed/ETA
  └─ Option 2: Crawl from scratch (slow, takes hours)
  ↓
Download complete
  ↓
Verify checksum
  ↓
App ready to use!
```

**Update Flow:**

```
App startup
  ↓
Check server metadata
  ↓
Compare with local version
  ↓ Newer version available
Show notification: "Database update available"
  ├─ Download in background (non-blocking)
  └─ Or prompt user to download
```

## Implementation Plan

### Phase 1: Server Setup (Do This First)

✅ **Already Done:**

- CLI tool with auto-detection
- Crawl and popular commands
- Stats command

**TODO:**

1. Deploy CLI binary to your server
2. Create update script
3. Setup cron job for automated updates
4. Setup static file hosting (nginx/apache/CDN)
5. Create metadata endpoint

### Phase 2: Client Implementation (Later)

**New Files:**

- `src/lib/stores/dbSync.ts` - Database sync state management
- `src/lib/components/DatabaseSyncModal.svelte` - Download UI
- `src-tauri/src/sync.rs` - Download and verification logic

**New Tauri Commands:**

```rust
// src-tauri/src/commands.rs
check_database_version() -> Result<DatabaseMetadata>
download_database(url: String) -> Result<()>
verify_database_checksum(expected: String) -> Result<bool>
```

**Features:**

- Progress tracking during download
- Resume support for interrupted downloads
- Checksum verification
- Automatic backup of old database before replacing
- Option to skip sync (use local DB)

### Phase 3: Auto-Update (Optional)

- Background sync every 24 hours
- Delta updates (only download changed records) - more complex
- Version comparison on startup
- User preference: auto/manual/never

## File Structure

```
Your Server:
/var/www/fitboy/
├── repacks.db              # Latest database
├── db_metadata.json        # Version info
└── archives/
    ├── repacks-20250115.db # Daily backups
    └── repacks-20250114.db
```

## Metadata Format

```json
{
  "version": "20250115120000",
  "last_updated": "2025-01-15T12:00:00Z",
  "game_count": 1500,
  "size_bytes": 52428800,
  "size_human": "50 MB",
  "url": "https://your-server.com/fitboy/repacks.db",
  "checksum": "sha256:abc123...",
  "schema_version": 1
}
```

## Benefits

### For Users:

- ✅ Instant start - no waiting for initial crawl
- ✅ Always up-to-date data from your server
- ✅ Reduced bandwidth on FitGirl's site
- ✅ No need to understand crawling/CLI

### For You:

- ✅ Centralized database maintenance
- ✅ Quality control - you verify data before publishing
- ✅ Can add custom data/curation
- ✅ Serve as CDN for all users
- ✅ Analytics on database downloads

### For FitGirl's Site:

- ✅ Reduced load - only your server crawls
- ✅ Respectful rate limiting from single source
- ✅ Less likely to trigger anti-bot measures

## Cost Considerations

**Storage:**

- Database size: ~50-200 MB (depends on game count)
- Monthly archives: ~6-12 GB/month
- Use S3/DigitalOcean Spaces/Backblaze B2 for cheap storage

**Bandwidth:**

- Per user download: ~50-200 MB (one-time)
- 1000 users = 50-200 GB
- Use Cloudflare (free tier) or BunnyCDN (cheap)

**Server:**

- Minimal CPU/RAM for CLI updates
- Can run on smallest VPS ($5/month)
- Or use GitHub Actions (free for public repos)

## Alternative: GitHub Releases

**Free Option:**
Instead of your own server, use GitHub Releases:

```bash
# On your development machine/CI
./cli crawl -p 20
./cli popular

# Create release with database
gh release create v$(date +%Y%m%d) \
  --title "Database Update $(date +%Y-%m-%d)" \
  --notes "Game count: $(./cli stats | grep 'Total Games')" \
  repacks.db

# Or use GitHub Actions to automate this
```

**Pros:**

- ✅ Free hosting and bandwidth
- ✅ Built-in version management
- ✅ CDN included
- ✅ Reliable infrastructure

**Cons:**

- ❌ Rate limits on API
- ❌ Less control over update frequency
- ❌ Requires GitHub account for automation

## Security Considerations

1. **Checksum Verification** - Always verify downloaded database
2. **HTTPS Only** - Prevent MITM attacks
3. **Signature** - Optionally sign database with GPG
4. **Rollback** - Keep local backup before replacing
5. **Schema Version** - Check compatibility before applying

## Current Status

**Completed:**

- ✅ CLI tool for automated database updates
- ✅ Database auto-detection
- ✅ Stats and health check commands
- ✅ Documentation

**Next Steps:**

1. Deploy CLI to your server
2. Setup automated update script
3. Host database file
4. Create metadata endpoint
5. (Later) Implement client-side sync

## Example Server Deployment

**Using DigitalOcean Droplet ($6/month):**

```bash
# Initial setup
ssh root@your-server

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/yourusername/fit-boy.git
cd fit-boy/desktop/src-tauri
cargo build --bin cli --release

# Setup directories
mkdir -p /opt/fitboy /var/www/fitboy
cp target/release/cli /opt/fitboy/
cp -r ../.. /opt/fitboy/

# Create update script (see above)
nano /opt/fitboy/update_and_publish.sh
chmod +x /opt/fitboy/update_and_publish.sh

# Setup cron
crontab -e
# Add: 0 */6 * * * /opt/fitboy/update_and_publish.sh

# Install and configure nginx
apt install nginx
nano /etc/nginx/sites-available/fitboy
# Configure static file serving

# Run first update
/opt/fitboy/update_and_publish.sh
```

**Using GitHub Actions (Free):**

```yaml
# .github/workflows/update-database.yml
name: Update Database

on:
  schedule:
    - cron: '0 */6 * * *' # Every 6 hours
  workflow_dispatch: # Manual trigger

jobs:
  update:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Build CLI
        run: |
          cd desktop/src-tauri
          cargo build --bin cli --release

      - name: Update Database
        run: |
          cd desktop/src-tauri
          ./target/release/cli crawl -p 20
          ./target/release/cli popular

      - name: Generate Metadata
        id: metadata
        run: |
          cd desktop/src-tauri
          GAME_COUNT=$(./target/release/cli stats | grep "Total Games:" | awk '{print $3}')
          echo "game_count=$GAME_COUNT" >> $GITHUB_OUTPUT

      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          tag_name: db-${{ github.run_number }}
          name: Database Update ${{ github.run_number }}
          body: |
            Database updated with latest games
            Total Games: ${{ steps.metadata.outputs.game_count }}
          files: desktop/repacks.db
```

This provides a complete path from your current CLI tool to a production database distribution system!
