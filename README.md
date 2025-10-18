# FGBrowser

A desktop application for browsing and organizing FitGirl Repack information.

![FGBrowser](.github/assets/FGBrowser-03.png)

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
- **Auto-Update**: Automatic updates via GitHub releases

## Download

Download the latest version from [GitHub Releases](https://github.com/ekinertac/fit-boy/releases)

**macOS:**

- Apple Silicon (M1/M2/M3/M4): Download `*_aarch64.dmg`
- Intel Macs: Download `*_x86_64.dmg`

### macOS Installation Instructions

If you see **"FGBrowser is damaged and can't be opened"** - don't worry, the app is not actually damaged. This is macOS blocking unsigned apps downloaded from the internet.

**Option 1 - Terminal (Recommended):**

```bash
xattr -cr /Applications/FGBrowser.app
```

Then open the app normally. This removes the quarantine flag.

**Option 2 - Right-click Method:**

1. Right-click (or Control-click) on FGBrowser.app in Applications
2. Select "Open" from the menu
3. Click "Open" in the security dialog
4. App will open normally from then on

**Windows:**

- Download `*_x64-setup.exe`

## Development

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
make build
```

## How It Works

1. Browse thousands of games with instant search and filtering
2. View detailed information including size, languages, and descriptions
3. Copy magnet links to use with your torrent client
4. Track trending games by month and year
5. All data is stored locally for offline browsing

## Privacy

Anonymous telemetry is enabled to help improve the app. No personal data or game information is collected - only crash reports and usage statistics.

## Disclaimer

This application does not host, create, or distribute any game files. It only aggregates and displays publicly available information and torrent magnet links from FitGirl Repacks website. Users are responsible for ensuring they have legal rights to download any content.
