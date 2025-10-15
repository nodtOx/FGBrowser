# FitBoy CLI Tool

Command-line interface for crawling repack sites and managing the database without running the GUI application.

## Building the CLI

```bash
cd desktop/src-tauri
cargo build --bin cli --release
```

The binary will be located at: `desktop/src-tauri/target/release/cli`

## Usage

```bash
./cli [OPTIONS] <COMMAND>
```

### Global Options

- `-d, --database <PATH>` - Path to database file (auto-detected if not specified)
- `-v, --verbose` - Enable verbose output
- `-h, --help` - Show help

### Database Auto-Detection

The CLI automatically finds your database in these locations (in order):

1. `../repacks.db` (when running from `src-tauri/`)
2. `repacks.db` (when running from project root)
3. Next to the CLI binary
4. `desktop/repacks.db` (workspace location)

You only need `-d` flag if your database is in a custom location!

## Commands

### 1. Crawl Pages

Crawl FitGirl Repacks pages and save games to database.

```bash
./cli crawl [OPTIONS]
```

**Options:**

- `-p, --pages <N>` - Number of pages to crawl (default: 5)
- `-m, --max-pages <N>` - Maximum pages, 0 = no limit (default: 0)

**Examples:**

```bash
# Crawl first 5 pages (database auto-detected)
./cli crawl

# Crawl first 10 pages with verbose output
./cli crawl -p 10 -v

# Crawl 20 pages with custom database
./cli -d /path/to/mydb.db crawl -p 20

# Crawl all available pages (use max-pages)
./cli crawl --max-pages 100
```

### 2. Update Popular Repacks

Fetch and update popular repacks lists.

```bash
./cli popular [OPTIONS]
```

**Options:**

- `-p, --period <PERIOD>` - Period: "month", "year", or "both" (default: "both")

**Examples:**

```bash
# Update both monthly and yearly popular repacks
./cli popular

# Update only monthly popular repacks
./cli popular -p month

# Update yearly popular repacks with verbose output
./cli popular -p year -v
```

### 3. Show Statistics

Display database statistics including game count, categories, and top genres.

```bash
./cli stats
```

**Example output:**

```
📊 Database Statistics

Total Games: 1234
Total Magnet Links: 3456
Total Categories: 45

📈 Top Categories:
  1. Adventure (234)
  2. Action (189)
  3. RPG (156)
  ...
```

### 4. Check Database Health

Verify database integrity and data quality.

```bash
./cli check
```

**Example output:**

```
🔍 Checking database health...

✅ Database is accessible
   Total Games: 1234
   Total Categories: 45
   Total Magnet Links: 3456

🔍 Checking data quality...
  ✅ All games have clean names

✅ Database check complete
```

## Complete Examples

### Initial Setup - Crawl First 10 Pages

```bash
# Build CLI
cd desktop/src-tauri
cargo build --bin cli --release

# Crawl first 10 pages (database auto-detected!)
./target/release/cli crawl -p 10 -v

# Update popular repacks
./target/release/cli popular -v

# Check stats
./target/release/cli stats
```

### Daily Update Script

Create a script `update_db.sh`:

```bash
#!/bin/bash
cd /path/to/fit-boy/desktop/src-tauri

echo "Updating FitBoy database..."

# Crawl recent pages (database auto-detected)
./target/release/cli crawl -p 3

# Update popular repacks
./target/release/cli popular

# Show stats
./target/release/cli stats

echo "Update complete!"
```

Make it executable:

```bash
chmod +x update_db.sh
```

### Cron Job

Add to crontab to run daily at 2 AM:

```bash
0 2 * * * /path/to/fit-boy/update_db.sh >> /var/log/fitboy_update.log 2>&1
```

## Integration with Tauri App

The CLI tool uses the **exact same code** as the Tauri application:

- ✅ Same crawler logic (`FitGirlCrawler`)
- ✅ Same database operations (`save_repacks_to_db`)
- ✅ Same data validation and cleaning
- ✅ Same error handling

This means:

- No code duplication
- Consistent behavior
- Changes to crawler benefit both CLI and GUI
- Database populated by CLI is immediately usable in GUI

## Features

### What the CLI Does

1. **Crawls pages** - Fetches game listings from FitGirl Repacks
2. **Extracts data** - Parses game info, sizes, magnets, genres
3. **Cleans titles** - Automatically generates clean game names
4. **Saves to database** - Stores all data with proper relations
5. **Handles duplicates** - Updates existing games intelligently
6. **Fetches popular** - Gets top monthly/yearly repacks
7. **Links data** - Connects popular repacks to full game data

### What's NOT Duplicated

- ❌ No separate HTML parsers
- ❌ No separate database schemas
- ❌ No separate title cleaning logic
- ❌ No separate category handling

Everything reuses the library code from `desktop_lib`!

## Troubleshooting

### Database locked

If you get "database is locked" error:

- Make sure the GUI application is closed
- Check no other CLI instances are running

### Permission denied

```bash
chmod +x ./target/release/cli
```

### Can't find database

Use absolute path:

```bash
./cli -d /Users/yourname/Code/fit-boy/repacks.db crawl
```

## Advanced Usage

### Custom Database Location

```bash
# Use different database for testing
./cli -d test.db crawl -p 2

# Production database
./cli -d /var/lib/fitboy/repacks.db crawl -p 10
```

### Verbose Mode

Get detailed output about every operation:

```bash
./cli -v crawl -p 3
```

Output will show:

- Each game title being processed
- Fetch progress
- Save confirmations
- Error details

### Combining with Other Tools

Pipe output for logging:

```bash
./cli crawl -p 5 2>&1 | tee crawl.log
```

Run with notification:

```bash
./cli crawl -p 10 && notify-send "Crawl complete"
```

## Performance

- **Crawl speed**: ~1 page per 2-3 seconds (respects rate limiting)
- **Memory usage**: ~50-100 MB
- **Disk I/O**: Minimal, SQLite handles it efficiently
- **Network**: ~1-5 MB per page depending on images

## Future Enhancements

Possible additions (not yet implemented):

- Multi-site support via the new crawler registry
- Parallel page crawling
- Resume from interruption
- Export/import functionality
- Incremental updates only
