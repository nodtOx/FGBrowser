# FGBrowser

A desktop application for browsing and organizing FitGirl Repack information.

![FGBrowser](.github/assets/FGBrowser-03.png)

> **Disclaimer**: This application does not host, create, or distribute any game files. It only aggregates and displays publicly available information and torrent magnet links from FitGirl Repacks website. Users are responsible for ensuring they have legal rights to download any content.

## Features

- **Fast Browsing**: Search and filter thousands of games instantly
- **Built-in Crawler**: Rust-based web crawler integrated into the app
- **Auto-Updates**: Automatically fetches new games in the background
- **Beautiful UI**: Modern, responsive interface with dark/light themes
- **Popular Games**: View trending repacks by month and year
- **Download Management**: Track and manage your downloads
- **Offline Ready**: Browse games without internet after initial crawl
- **Image Caching**: Automatic caching of game cover images
- **Multiple Views**: Grid and list view options

## Technology Stack

- **Frontend**: Svelte + SvelteKit
- **Backend**: Rust + Tauri
- **Database**: SQLite with Rusqlite
- **Web Crawler**: Custom Rust implementation using scraper and reqwest
- **Styling**: Tailwind CSS

## Quick Start

### Development

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

### Building

Build a production version:

```bash
cd desktop
npm run tauri build
```

The built application will be available in `desktop/src-tauri/target/release/bundle/`

## How It Works

1. **Crawler**: Built-in Rust crawler fetches publicly available game information and torrent magnet links from FitGirl Repacks website
2. **Database**: SQLite database stores game metadata and magnet links locally for easy browsing
3. **Image Cache**: Automatically downloads and caches game cover images
4. **Browse**: Search, filter, and view game information offline
5. **Magnet Links**: Copy magnet links to use with your torrent client
6. **Popular Repacks**: Tracks and displays trending repacks by time period

**Note**: This application only stores metadata and magnet links. No game files are downloaded, hosted, or distributed by this application.

## CLI Tool

FGBrowser includes a command-line interface for database operations:

```bash
# Build the CLI
cd desktop/src-tauri
cargo build --release --bin cli

# Use the CLI
./target/release/cli --help

# Query games
./target/release/cli query --search "game name"

# Run crawler
./target/release/cli crawl --pages 5

# Export database
./target/release/cli export output.json
```

See [CLI_USAGE.md](CLI_USAGE.md) for detailed CLI documentation.

## Database Schema

The SQLite database contains the following main tables:

**games table:**

- id, title, url, description, image_url
- genres, languages, developer, publisher
- release_date, original_size, repack_size
- created_at, updated_at

**magnet_links table:**

- id, game_id, magnet_link, source
- created_at

**popular_repacks table:**

- id, game_id, rank, month, year
- created_at

**downloads table:**

- id, game_id, status, progress, speed
- downloaded, total_size, eta, added_at

See [docs/DATABASE_DOWNLOAD.md](docs/DATABASE_DOWNLOAD.md) for detailed schema documentation.

## Project Structure

```
FGBrowser/
├── desktop/                    # Desktop application
│   ├── src/                   # Frontend (Svelte)
│   │   ├── lib/
│   │   │   ├── components/   # UI components
│   │   │   └── stores/       # State management
│   │   └── routes/           # SvelteKit routes
│   ├── src-tauri/            # Backend (Rust)
│   │   ├── src/
│   │   │   ├── commands/     # Tauri commands
│   │   │   ├── crawler/      # Web crawler
│   │   │   ├── database/     # Database layer
│   │   │   └── bin/          # CLI binary
│   │   └── Cargo.toml
│   └── package.json
├── docs/                      # Documentation
├── server/                    # Deployment scripts
└── README.md
```

## Documentation

- [Desktop App Documentation](docs/DESKTOP_APP.md)
- [CLI Usage Guide](CLI_USAGE.md)
- [Database Sync](DATABASE_SYNC.md)
- [Popular Repacks Feature](docs/POPULAR_REPACKS.md)
- [Crawler Implementation](docs/CRAWLER_RUST.md)

## Important Notes

- **No Content Hosting**: FGBrowser does not host, store, or distribute any game files
- **Information Only**: The application only aggregates publicly available metadata and magnet links
- **User Responsibility**: Users are solely responsible for ensuring they have legal rights to download any content
- **Educational Purpose**: This project is intended for educational purposes and to demonstrate web scraping and desktop application development

## License

This project is for educational purposes only. The developers are not responsible for how this software is used.
