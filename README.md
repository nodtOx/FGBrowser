# FitGirl Repacks Browser

A desktop application and web crawler for browsing game repack information from fitgirl-repacks.site.

## Features

### Desktop App

- **Pre-populated Database**: Downloads a ready-to-use database on first run
- **Fast Browsing**: Search and filter thousands of games instantly
- **Auto-Updates**: Automatically fetches new games in the background
- **Beautiful UI**: Modern, responsive interface with dark/light themes
- **Popular Games**: View trending repacks by month and year
- **Download Management**: Track and manage your downloads (coming soon)
- **Offline Ready**: Browse games without internet after initial setup

### Web Crawler (Python)

- Crawls multiple pages of repacks
- Extracts comprehensive game information:
  - Title
  - Genres/Tags
  - Company/Developer
  - Languages
  - Original Size
  - Repack Size
  - Magnet Links (torrent download links)
  - Publication date
- Stores data in SQLite database for fast queries
- Command-line query interface
- Rate limiting to be respectful to the server

## Desktop App

The desktop application is built with Tauri (Rust) and Svelte.

### Quick Start

1. Navigate to desktop folder:

```bash
cd desktop
```

2. Install dependencies:

```bash
npm install
```

3. Run the app:

```bash
npm run tauri dev
```

On first run, the app will automatically download a pre-populated database from the server, so you can start browsing games immediately.

### How It Works

1. **First Run**: Downloads database from `http://157.230.16.45/repacks.db`
2. **Error Handling**: If download fails, shows error with retry button
3. **Auto-Update**: Checks for new games and updates database in background
4. **Browse**: Search, filter, and view game details
5. **Download**: Copy magnet links or open in your torrent client

### Build Desktop App

```bash
cd desktop
npm run tauri build
```

See [docs/DESKTOP_APP.md](docs/DESKTOP_APP.md) for detailed documentation.

## Python Crawler

The Python crawler is used to populate the database that the desktop app uses.

### Installation

1. Create a virtual environment:

```bash
py -m venv venv
```

2. Activate the virtual environment:

```bash
.\venv\Scripts\Activate.ps1
```

3. Install dependencies:

```bash
pip install -r requirements.txt
```

## Usage

### Crawl and populate database:

Simply run the crawler to fetch games and save to database:

```bash
python run_crawler.py
```

This will create a `repacks.db` SQLite database with all game information.

### Configure crawling:

Edit the constants at the top of `crawler/crawler.py`:

```python
# Configuration
MAX_PAGES = 50         # Set to None for infinite crawl (all available pages)
CRAWL_DELAY = 1.0      # Delay between requests in seconds
```

Examples:

- `MAX_PAGES = 50` - Crawl first 50 pages only
- `MAX_PAGES = None` - Crawl ALL available pages until the end
- `CRAWL_DELAY = 1.0` - Wait 1 second between requests (be respectful!)
- `CRAWL_DELAY = 0` - No delay (NOT RECOMMENDED - may cause IP bans!)

### Manage blacklist:

The crawler automatically skips pages matching patterns in `blacklist.txt`:

```bash
# List blacklist patterns
python -m crawler.manage_blacklist list

# Add a pattern (matches URL or title)
python -m crawler.manage_blacklist add "pattern-to-skip"

# Remove a pattern
python -m crawler.manage_blacklist remove "pattern-to-skip"

# Check if something is blacklisted
python -m crawler.manage_blacklist check "Some Game Title"
python -m crawler.manage_blacklist check "https://fitgirl-repacks.site/some-page/"

# Clear all patterns
python -m crawler.manage_blacklist clear
```

You can also edit `config/blacklist.txt` directly - add one pattern per line.

### Query the database:

Use the query utility to search and view games:

```bash
# Show database statistics
python -m crawler.query stats

# Search for games
python -m crawler.query search "Little Nightmares"

# Show recent repacks
python -m crawler.query recent 10

# Get detailed info about a specific game
python -m crawler.query detail "Arcane Path"

# Export database to JSON
python -m crawler.query export my_repacks.json
```

### Customize the crawler:

```python
from crawler import FitGirlCrawler

crawler = FitGirlCrawler()

# Change number of pages to crawl
repacks = crawler.crawl_multiple_pages(start_page=1, max_pages=5)

# Save directly to database
crawler.save_to_database(repacks)
```

## Database Schema

The SQLite database contains two tables:

**repacks table:**

- id (Primary Key)
- title
- genres_tags
- company
- languages
- original_size
- repack_size
- url
- date
- created_at
- updated_at

**magnet_links table:**

- id (Primary Key)
- repack_id (Foreign Key)
- source (e.g., "1337x", "RuTor")
- magnet (full magnet link)
- created_at

## Notes

- The crawler includes a 1-second delay between requests by default (configurable via `CRAWL_DELAY`)
- Respects the website's server by not overwhelming it with requests
- **Optimized crawling**: Only fetches listing pages, no need to visit individual game pages
- Crawl speed: ~50 pages in 2-3 minutes (depending on connection)
- Infinite crawl mode available (set `MAX_PAGES = None`)
- **Database uses URL as unique identifier** - games are identified by their page URL
- Re-running the crawler will update existing entries (based on URL)
- Entries without URLs and blacklisted items are automatically skipped
- Connection errors are handled gracefully - just re-run to resume
- You can export the database to JSON format using `python -m crawler.query export`

## Project Structure

```
fit-boy/
├── desktop/               # Desktop application (Tauri + Svelte)
│   ├── src/              # Frontend source code
│   │   ├── lib/          # Components, stores, utilities
│   │   └── routes/       # SvelteKit routes
│   ├── src-tauri/        # Rust backend
│   │   └── src/          # Rust source code
│   └── package.json      # Node dependencies
├── crawler/              # Python crawler package
│   ├── __init__.py       # Package initialization
│   ├── crawler.py        # Main crawler logic
│   ├── database.py       # Database operations
│   ├── query_db.py       # Query utilities
│   ├── query.py          # CLI query tool
│   ├── manage_blacklist.py # CLI blacklist tool
│   └── blacklist_manager.py # Blacklist management
├── docs/                 # Documentation
│   ├── DESKTOP_APP.md    # Desktop app documentation
│   ├── DATABASE_DOWNLOAD.md # Database download feature
│   └── ...
├── config/               # Configuration files
│   ├── blacklist.txt     # Patterns to skip during crawl
│   └── blacklist.txt.example # Example configuration
├── server/               # Server deployment scripts
├── run_crawler.py        # Python crawler entry point
├── requirements.txt      # Python dependencies
├── README.md            # This file
└── repacks.db           # SQLite database
```

## Blacklist System

The blacklist allows you to skip specific pages during crawling:

- **Pattern matching**: Patterns are case-insensitive and match both URLs and titles
- **Example patterns**:
  - `upcoming-repacks` - matches URLs containing this text
  - `updates-digest` - skips all update digest posts
- **Automatic filtering**: Blacklisted items are skipped during both page listing and detail crawling
- **Easy management**: Edit `blacklist.txt` directly or use the CLI tool
