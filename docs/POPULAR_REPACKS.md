# Popular Repacks Feature

## Overview

The popular repacks feature allows the application to track and display the most popular game repacks from FitGirl's website. This data is stored in a separate database table and linked to the main repacks catalog.

## Database Schema

### `popular_repacks` Table

```sql
CREATE TABLE popular_repacks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL UNIQUE,           -- Game page URL
    title TEXT NOT NULL,                -- Full game title
    image_url TEXT,                     -- Cover image URL
    rank INTEGER NOT NULL,              -- Popularity rank (1 = most popular)
    repack_id INTEGER,                  -- Foreign key to repacks.id
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (repack_id) REFERENCES repacks (id) ON DELETE SET NULL
);
```

### Key Features

- **Separate tracking**: Popular repacks are stored independently from the main catalog
- **Automatic linking**: The system automatically links popular repacks to existing games by URL
- **Rank tracking**: Each entry has a rank indicating its popularity position
- **Image support**: Cover images are stored for display in the UI

## Backend API (Rust/Tauri Commands)

### 1. `fetch_popular_repacks()`

Fetches the latest popular repacks from the FitGirl website.

```rust
let count: usize = invoke("fetch_popular_repacks").await?;
```

**Returns**: Number of popular repacks saved

**Process**:

1. Fetches HTML from `https://fitgirl-repacks.site/popular-repacks/`
2. Parses the grid layout to extract URLs, titles, and images
3. Clears existing popular repacks
4. Saves new popular repacks to database
5. Links them to existing games in the repacks table

### 2. `parse_popular_repacks_from_file(file_path: String)`

Parses popular repacks from a local HTML file.

```rust
let count: usize = invoke("parse_popular_repacks_from_file", {
    filePath: "popular_repacks.html"
}).await?;
```

**Returns**: Number of popular repacks saved

**Use case**: Testing or offline parsing of saved HTML files

### 3. `get_popular_repacks(limit: i32)`

Retrieves popular repacks from the database.

```rust
let repacks: Vec<PopularRepack> = invoke("get_popular_repacks", {
    limit: 50
}).await?;
```

**Returns**:

```typescript
interface PopularRepack {
  id: number;
  url: string;
  title: string;
  image_url: string | null;
  rank: number;
  repack_id: number | null;
}
```

### 4. `get_popular_repacks_with_games(limit: i32)`

Retrieves popular repacks with full game details (if linked).

```rust
let repacks: Vec<PopularRepackWithGame> = invoke("get_popular_repacks_with_games", {
    limit: 50
}).await?;
```

**Returns**:

```typescript
interface PopularRepackWithGame {
  id: number;
  url: string;
  title: string;
  image_url: string | null;
  rank: number;
  game: Game | null; // Full game details if linked
}
```

### 5. `update_popular_repack_links()`

Updates the links between popular repacks and existing games.

```rust
let count: usize = invoke("update_popular_repack_links").await?;
```

**Returns**: Number of links updated

**Use case**: Run this after crawling new games to link them with popular repacks

## App Lifecycle Integration

The popular repacks feature is integrated into the application lifecycle at strategic points:

### 1. **App Startup** (Load from Database)

When the app starts and finds existing data:

```typescript
async function checkAndInitializeDatabase() {
  // ... load games and categories ...

  // Load popular repacks from database (non-blocking)
  loadPopularRepacks().catch((err) => console.warn('Failed to load popular repacks:', err));
}
```

**Why**: Loads previously fetched popular repacks from the database immediately on startup. This is a quick database query, not a network request.

### 2. **After Crawler Completes** (Fetch from Website)

When the main catalog crawler finishes:

```typescript
async function onCrawlerComplete() {
  await loadGames(100);
  await loadCategories();

  // Fetch fresh popular repacks from website
  fetchPopularRepacks().catch((err) => console.error('Failed to fetch popular repacks:', err));
}
```

**Why**: This is the **primary update point**. After crawling the main catalog, we fetch the latest popular repacks from the website and automatically link them to the newly crawled games.

### 3. **Manual Refresh** (User-Triggered)

Users can manually refresh popular repacks via Settings or a UI button:

```typescript
async function refreshPopularRepacks() {
  try {
    const count = await invoke('fetch_popular_repacks');
    console.log(`Refreshed ${count} popular repacks`);
  } catch (error) {
    console.error('Failed to refresh:', error);
  }
}
```

**Why**: Allows users to get the latest popularity rankings without running the full crawler.

### Lifecycle Summary

```
App Launch
    ├─ Database Empty? → Show Crawler Modal
    │                      └─ After Crawl → Fetch Popular Repacks ✅
    │
    └─ Database Has Data → Load Games & Categories
                          └─ Load Popular Repacks from DB ✅ (quick)
                          └─ Check for Updates (optional)
                               └─ After Update → Fetch Popular Repacks ✅
```

### Network vs Database Calls

- **App Startup**: `get_popular_repacks()` - Quick database query
- **After Crawling**: `fetch_popular_repacks()` - Network request to website
- **Manual Refresh**: `fetch_popular_repacks()` - Network request to website

## Usage Flow

### Initial Setup

```rust
// 1. First time: Crawl the main catalog
invoke("start_crawler").await?;

// 2. Fetch popular repacks
invoke("fetch_popular_repacks").await?;
```

### Regular Updates

```rust
// 1. Update main catalog
invoke("update_database").await?;

// 2. Update popular repacks
invoke("fetch_popular_repacks").await?;
```

### Manual Testing with HTML File

```bash
# 1. Download the HTML file
curl -L "https://fitgirl-repacks.site/popular-repacks/" -o popular_repacks.html

# 2. Parse from file
let count = invoke("parse_popular_repacks_from_file", {
    filePath: "./popular_repacks.html"
}).await?;
```

## HTML Structure

The parser expects the following HTML structure:

```html
<div class="widget-grid-view-image">
  <a href="[GAME_URL]" title="[GAME_TITLE]">
    <img src="[IMAGE_URL]" alt="[GAME_TITLE]" />
  </a>
</div>
```

## Implementation Details

### Crawler Module (`crawler.rs`)

```rust
pub struct PopularRepackEntry {
    pub url: String,
    pub title: String,
    pub image_url: Option<String>,
    pub rank: i32,
}

impl FitGirlCrawler {
    pub async fn fetch_popular_repacks() -> Result<Vec<PopularRepackEntry>>;
    pub fn parse_popular_repacks_html(html: &str) -> Result<Vec<PopularRepackEntry>>;
    pub fn parse_popular_repacks_from_file(file_path: &str) -> Result<Vec<PopularRepackEntry>>;
}
```

### Database Module (`database.rs`)

```rust
pub struct PopularRepack {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub image_url: Option<String>,
    pub rank: i32,
    pub repack_id: Option<i64>,
}

pub struct PopularRepackWithGame {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub image_url: Option<String>,
    pub rank: i32,
    pub game: Option<Game>,
}

impl Database {
    pub fn save_popular_repack(...) -> Result<i64>;
    pub fn get_popular_repacks(limit: i32) -> Result<Vec<PopularRepack>>;
    pub fn get_popular_repacks_with_games(limit: i32) -> Result<Vec<PopularRepackWithGame>>;
    pub fn clear_popular_repacks() -> Result<()>;
    pub fn update_popular_repack_links() -> Result<usize>;
}
```

## Testing

### Python Test Script

A test script is provided to validate the HTML parsing:

```bash
python3 test_popular_repacks.py
```

This will parse `popular_repacks.html` and display the first 10 entries.

### Expected Output

```
Found 50 popular repacks:

 1. Grand Theft Auto V / GTA 5 (Legacy) - v1.0.3411/1.70 + ...
    URL: https://fitgirl-repacks.site/grand-theft-auto-v/
    Image: https://i0.wp.com/i3.imageban.ru/out/2021/12/31/...

 2. Dying Light: The Beast - Deluxe Edition, v1.2.0 + ...
    URL: https://fitgirl-repacks.site/dying-light-the-beast/
    Image: https://i0.wp.com/i3.imageban.ru/out/2025/09/19/...
...
```

## Frontend Integration (TODO)

### TypeScript Interface

```typescript
interface PopularRepack {
  id: number;
  url: string;
  title: string;
  image_url: string | null;
  rank: number;
  repack_id: number | null;
}

interface PopularRepackWithGame {
  id: number;
  url: string;
  title: string;
  image_url: string | null;
  rank: number;
  game: Game | null;
}
```

### Usage Example

```typescript
import { invoke } from '@tauri-apps/api/core';

// Fetch and save popular repacks
const count = await invoke<number>('fetch_popular_repacks');
console.log(`Saved ${count} popular repacks`);

// Get popular repacks for display
const popularRepacks = await invoke<PopularRepackWithGame[]>('get_popular_repacks_with_games', { limit: 50 });
```

### Suggested UI Components

1. **Popular Games Section**: Display top 10-20 popular games with cover images
2. **Popular Badge**: Show a badge on games that are in the popular list
3. **Sorting Options**: Allow sorting by popularity rank
4. **Refresh Button**: Update popular repacks from the website

## Notes

- The popular repacks list updates independently from the main catalog
- If a game is removed from the main catalog, the link is set to NULL (not deleted)
- Cover images are stored as URLs, not downloaded locally
- The rank determines the order (1 = most popular)
- Currently supports up to 50 popular repacks (can be adjusted)

## Future Enhancements

- [ ] Cache cover images locally for faster loading
- [ ] Track popularity history over time
- [ ] Add trending indicator (games moving up in rank)
- [ ] Filter games by popularity in the UI
- [ ] Show popularity metrics (views, downloads if available)
