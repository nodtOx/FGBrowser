# Crawler Conversion: Python to Rust

## Overview

The Python crawler has been successfully converted to Rust and integrated into the Tauri desktop app. No Python dependency is required anymore.

## What Was Converted

### Python Components → Rust Equivalents

1. **HTTP Requests**

   - Python: `requests`
   - Rust: `reqwest`

2. **HTML Parsing**

   - Python: `BeautifulSoup` + `lxml`
   - Rust: `scraper`

3. **Regular Expressions**

   - Python: `re`
   - Rust: `regex`

4. **Database**
   - Already in Rust via `rusqlite`

## File Structure

```
desktop/src-tauri/src/
├── crawler.rs          # NEW: Rust crawler implementation
├── commands.rs         # Updated: Added crawler commands
├── database.rs         # Existing: Already in Rust
└── lib.rs              # Updated: Registered crawler
```

## Features

### Implemented

- Crawl multiple pages from fitgirl-repacks.site
- Extract game information (title, genres, company, languages, sizes)
- Extract magnet links with sources
- Blacklist system (reads from `config/blacklist.txt`)
- Rate limiting (1 second delay between requests)
- Save to SQLite database with deduplication
- Progress reporting

### How It Works

1. **Crawler Module** (`crawler.rs`)

   - `FitGirlCrawler::new()` - Creates crawler instance
   - `crawl_page(page_num)` - Crawls single page
   - `crawl_multiple_pages(start, max)` - Crawls multiple pages
   - Extracts game details using regex patterns
   - Extracts magnet links from HTML

2. **Tauri Command** (`commands.rs`)

   - `start_crawler(max_pages)` - Exposed to frontend
   - Runs crawler asynchronously
   - Saves results to database
   - Returns progress information

3. **Frontend Integration** (Settings page)
   - Input for number of pages
   - "Run Crawler Now" button
   - Progress display
   - Auto-disables during crawl

## Usage

### From UI (Settings Page)

1. Open Settings (Ctrl+3)
2. Navigate to "Database" section
3. Set number of pages to crawl (default: 50)
4. Click "Run Crawler Now"
5. Wait for completion message

### Programmatically

```typescript
import { invoke } from '@tauri-apps/api/core';

const result = await invoke('start_crawler', {
  maxPages: 50, // or null for all pages
});

console.log(`Found ${result.total_games} games`);
```

## Configuration

### Blacklist

The crawler respects `config/blacklist.txt`:

```
# Blacklist patterns (one per line)
upcoming-repacks
updates-digest
```

### Crawler Settings

In `crawler.rs`:

- `crawl_delay`: Duration between requests (default: 1 second)
- `base_url`: FitGirl repacks site URL
- User agent: Mozilla/5.0 (standard browser)

## Performance

- **Speed**: ~1 page per second (with 1s delay)
- **50 pages**: ~50-60 seconds
- **Memory**: Minimal (streaming parser)
- **Database**: Incremental updates (no duplicates)

## Advantages Over Python

1. **No Python Dependency**: Self-contained binary
2. **Better Performance**: Native code execution
3. **Type Safety**: Compile-time error checking
4. **Smaller Bundle**: No Python runtime needed
5. **Better Integration**: Direct Tauri commands
6. **Cross-Platform**: Single codebase for all platforms

## Dependencies Added

In `Cargo.toml`:

```toml
reqwest = { version = "0.12", features = ["json"] }
scraper = "0.21"
regex = "1.11"
```

## Future Improvements

- [ ] Real-time progress updates (using Tauri events)
- [ ] Pause/resume crawler
- [ ] Selective page range crawling
- [ ] Parallel page fetching (with rate limiting)
- [ ] Automatic retry on network errors
- [ ] Crawler scheduling (cron-like)
- [ ] Export crawled data to JSON

## Migration Notes

### Python Code Removal

The following Python files are **no longer needed** for the desktop app:

- `crawler/crawler.py`
- `crawler/database.py`
- `crawler/blacklist_manager.py`
- `run_crawler.py`
- `requirements.txt` (for desktop app)

You can keep them for reference or standalone usage, but they're not bundled with the app.

### Database Compatibility

The Rust crawler uses the **same database schema** as the Python version:

- Same table structures
- Same column names
- Same indexes
- Fully compatible

You can mix and match Python and Rust crawlers on the same database.

## Testing

1. Run the app in dev mode:

   ```bash
   cd desktop
   npm run tauri dev
   ```

2. Navigate to Settings → Database

3. Click "Run Crawler Now"

4. Check console output for crawl progress

5. Verify database was updated in Browse page

## Troubleshooting

### Crawler Fails to Start

- Check internet connection
- Verify `config/blacklist.txt` exists (or will use defaults)
- Check console for Rust panic messages

### No Games Found

- Site might be down
- Check if blacklist is too restrictive
- Verify HTML structure hasn't changed

### TypeScript Errors

The `@tauri-apps/plugin-store` type error is a known issue with the language server. Run:

```bash
cd desktop
npm install
```

Then reload VS Code.

## Summary

The Python crawler has been fully replaced with a native Rust implementation that's faster, more reliable, and doesn't require bundling Python. The user experience remains the same, but the app is now truly self-contained and production-ready.
