# Multi-Site Crawler Architecture

## Overview

The crawler has been refactored to support multiple repack sites through a trait-based architecture. This allows adding new sites without modifying core logic.

## Architecture

### Core Components

```
src-tauri/src/crawler/
├── mod.rs                  # Main module, re-exports, registry creation
├── site_crawler.rs         # Trait definition and registry
├── fitgirl.rs             # FitGirl Repacks implementation
├── models.rs              # Shared data structures
├── extractors.rs          # Common extraction utilities
├── title_cleaner.rs       # Game title cleaning
└── popular.rs             # Popular repacks parsing
```

### Key Files

#### 1. `site_crawler.rs`

Defines the `SiteCrawler` trait that all site crawlers must implement:

```rust
#[async_trait]
pub trait SiteCrawler: Send + Sync {
    fn site_id(&self) -> &str;
    fn site_name(&self) -> &str;
    fn base_url(&self) -> &str;
    fn is_enabled(&self) -> bool;

    async fn crawl_page(&self, page_num: u32) -> Result<Vec<GameRepack>>;
    async fn crawl_single_game(&self, url: &str) -> Result<Option<GameRepack>>;
    async fn fetch_popular_repacks(&self, period: &str) -> Result<Vec<PopularRepackEntry>>;
}
```

Also contains `CrawlerRegistry` for managing multiple crawlers.

#### 2. `fitgirl.rs`

Implements `SiteCrawler` for FitGirl Repacks:

```rust
pub struct FitGirlCrawler {
    client: Client,
    base_url: String,
    crawl_delay: Duration,
    blacklist: Vec<String>,
}

#[async_trait]
impl SiteCrawler for FitGirlCrawler { ... }
```

#### 3. `mod.rs`

Contains:

- Legacy `FitGirlCrawler` (for backward compatibility)
- `create_crawler_registry()` helper function

```rust
pub fn create_crawler_registry() -> Result<CrawlerRegistry> {
    let mut registry = CrawlerRegistry::new();
    registry.register(Box::new(FitGirlSiteCrawler::new()?));
    // Add more sites here
    Ok(registry)
}
```

## Usage

### Using the Registry (Recommended)

```rust
use desktop_lib::crawler::{create_crawler_registry, CrawlerRegistry};

// Create registry with all available crawlers
let registry = create_crawler_registry()?;

// Crawl all enabled sites
let results = registry.crawl_all_sites(1).await?;
for (site_id, repacks) in results {
    println!("{}: {} repacks found", site_id, repacks.len());
}

// Use a specific crawler
if let Some(fitgirl) = registry.get_crawler("fitgirl") {
    let repacks = fitgirl.crawl_page(1).await?;
}
```

### Using Individual Crawlers

```rust
use desktop_lib::crawler::FitGirlSiteCrawler;

let crawler = FitGirlSiteCrawler::new()?;
let repacks = crawler.crawl_page(1).await?;
```

## Adding a New Site

### Step 1: Create Site Crawler Module

Create `src-tauri/src/crawler/dodi.rs`:

```rust
use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use scraper::{Html, Selector};
use std::time::Duration;

use super::models::{GameRepack, PopularRepackEntry};
use super::site_crawler::SiteCrawler;

pub struct DodiRepacksCrawler {
    client: Client,
    base_url: String,
}

impl DodiRepacksCrawler {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent("Mozilla/5.0...")
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            base_url: "https://dodi-repacks.site".to_string(),
        })
    }
}

#[async_trait]
impl SiteCrawler for DodiRepacksCrawler {
    fn site_id(&self) -> &str {
        "dodi"
    }

    fn site_name(&self) -> &str {
        "DODI Repacks"
    }

    fn base_url(&self) -> &str {
        &self.base_url
    }

    async fn crawl_page(&self, page_num: u32) -> Result<Vec<GameRepack>> {
        // Site-specific page crawling logic
        todo!()
    }

    async fn crawl_single_game(&self, url: &str) -> Result<Option<GameRepack>> {
        // Site-specific single game extraction
        todo!()
    }

    // Optional: implement if site supports popular repacks
    async fn fetch_popular_repacks(&self, period: &str) -> Result<Vec<PopularRepackEntry>> {
        Ok(Vec::new()) // Not supported
    }
}
```

### Step 2: Register in Module

Update `src-tauri/src/crawler/mod.rs`:

```rust
pub mod dodi;
pub use dodi::DodiRepacksCrawler;

pub fn create_crawler_registry() -> Result<CrawlerRegistry> {
    let mut registry = CrawlerRegistry::new();

    // FitGirl
    registry.register(Box::new(FitGirlSiteCrawler::new()?));

    // DODI
    registry.register(Box::new(DodiRepacksCrawler::new()?));

    Ok(registry)
}
```

### Step 3: Update Database Schema (if needed)

If you need to track which site a repack came from:

```rust
// In database/models.rs
pub struct Game {
    pub id: i64,
    pub site_id: String,  // "fitgirl", "dodi", etc.
    // ... other fields
}
```

## Site-Specific Considerations

### HTML Parsing

Each site has different HTML structure. Key selectors to identify:

**FitGirl Example:**

```rust
let article_selector = Selector::parse("article").unwrap();
let title_selector = Selector::parse("h1.entry-title, h2.entry-title").unwrap();
let content_selector = Selector::parse("div.entry-content").unwrap();
```

**Your Site:**

```rust
// Inspect the site's HTML and create appropriate selectors
let post_selector = Selector::parse("div.game-post").unwrap();
let title_selector = Selector::parse("h2.game-title").unwrap();
```

### Blacklisting

Each site may need different blacklist patterns:

```rust
fn load_blacklist() -> Vec<String> {
    vec![
        "site-specific-pattern-1".to_string(),
        "site-specific-pattern-2".to_string(),
    ]
}
```

### Rate Limiting

Respect each site's crawling policies:

```rust
impl DodiRepacksCrawler {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: Client::builder().build()?,
            crawl_delay: Duration::from_secs(2), // Adjust per site
        })
    }
}
```

## Database Integration

### Storing Multi-Site Data

Option 1: Add `site_id` column to `repacks` table:

```sql
ALTER TABLE repacks ADD COLUMN site_id TEXT DEFAULT 'fitgirl';
```

Option 2: Create separate tables per site:

```sql
CREATE TABLE fitgirl_repacks (...);
CREATE TABLE dodi_repacks (...);
```

### Commands Integration

Update `src-tauri/src/commands.rs`:

```rust
#[tauri::command]
pub async fn crawl_all_sites(db: tauri::State<'_, Database>) -> Result<String, String> {
    let registry = create_crawler_registry().map_err(|e| e.to_string())?;

    for crawler in registry.get_enabled_crawlers() {
        let repacks = crawler.crawl_page(1).await.map_err(|e| e.to_string())?;

        // Save to database with site_id
        for repack in repacks {
            db.save_repack_with_site(repack, crawler.site_id())
                .map_err(|e| e.to_string())?;
        }
    }

    Ok("Done".to_string())
}
```

## Feature Flags

You can gate new sites behind feature flags:

```rust
impl SiteCrawler for DodiRepacksCrawler {
    fn is_enabled(&self) -> bool {
        // Check feature flag or settings
        cfg!(feature = "dodi-crawler")
    }
}
```

## Testing New Crawlers

1. **Create test HTML files:**

   ```bash
   curl -o dodi_page.html https://dodi-repacks.site/
   ```

2. **Write unit tests:**

   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[tokio::test]
       async fn test_dodi_crawler() {
           let crawler = DodiRepacksCrawler::new().unwrap();
           let repacks = crawler.crawl_page(1).await.unwrap();
           assert!(!repacks.is_empty());
       }
   }
   ```

3. **Test with registry:**
   ```rust
   let registry = create_crawler_registry()?;
   let results = registry.crawl_all_sites(1).await?;
   println!("{:#?}", results);
   ```

## Migration Path

The old `FitGirlCrawler` in `mod.rs` is kept for backward compatibility. To migrate:

```rust
// Old code
let crawler = FitGirlCrawler::new()?;
let repacks = crawler.crawl_page(1).await?;

// New code (unchanged API, works the same)
let crawler = FitGirlCrawler::new()?;
let repacks = crawler.crawl_page(1).await?;

// OR use registry
let registry = create_crawler_registry()?;
let results = registry.crawl_all_sites(1).await?;
```

## Benefits

1. **Extensibility**: Add new sites without modifying existing code
2. **Maintainability**: Site-specific logic is isolated
3. **Testability**: Each crawler can be tested independently
4. **Flexibility**: Enable/disable crawlers at runtime
5. **Type Safety**: Trait ensures consistent API across all sites

## Future Enhancements

- Site settings (enabled/disabled) in database
- Per-site crawl scheduling
- Site health monitoring
- Automatic site detection from URLs
- Parallel crawling of multiple sites
- Site-specific parsers for different content types
