// Trait for site-specific crawlers
// This allows adding new repack sites without modifying core logic

use anyhow::Result;
use async_trait::async_trait;
use super::models::{GameRepack, PopularRepackEntry};

/// Trait that all site crawlers must implement
#[async_trait]
pub trait SiteCrawler: Send + Sync {
    /// Unique identifier for the site (e.g., "fitgirl", "dodi")
    fn site_id(&self) -> &str;
    
    /// Display name for the site
    fn site_name(&self) -> &str;
    
    /// Base URL for the site
    fn base_url(&self) -> &str;
    
    /// Whether this site is currently enabled
    fn is_enabled(&self) -> bool {
        true
    }
    
    /// Crawl a specific page number and return repacks
    async fn crawl_page(&self, page_num: u32) -> Result<Vec<GameRepack>>;
    
    /// Crawl a single game page by URL
    async fn crawl_single_game(&self, url: &str) -> Result<Option<GameRepack>>;
    
    /// Fetch popular repacks (if site supports it)
    async fn fetch_popular_repacks(&self, _period: &str) -> Result<Vec<PopularRepackEntry>> {
        // Default implementation: not supported
        Ok(Vec::new())
    }
    
    /// Get total number of pages available (optional)
    async fn get_total_pages(&self) -> Result<Option<u32>> {
        Ok(None)
    }
    
    /// Site-specific blacklist patterns
    fn get_blacklist(&self) -> Vec<String> {
        Vec::new()
    }
}

/// Registry for managing multiple site crawlers
pub struct CrawlerRegistry {
    crawlers: Vec<Box<dyn SiteCrawler>>,
}

impl CrawlerRegistry {
    pub fn new() -> Self {
        Self {
            crawlers: Vec::new(),
        }
    }
    
    /// Register a new site crawler
    pub fn register(&mut self, crawler: Box<dyn SiteCrawler>) {
        self.crawlers.push(crawler);
    }
    
    /// Get all enabled crawlers
    pub fn get_enabled_crawlers(&self) -> Vec<&dyn SiteCrawler> {
        self.crawlers
            .iter()
            .filter(|c| c.is_enabled())
            .map(|c| c.as_ref())
            .collect()
    }
    
    /// Get a specific crawler by ID
    pub fn get_crawler(&self, site_id: &str) -> Option<&dyn SiteCrawler> {
        self.crawlers
            .iter()
            .find(|c| c.site_id() == site_id)
            .map(|c| c.as_ref())
    }
    
    /// Crawl all enabled sites
    pub async fn crawl_all_sites(&self, page_num: u32) -> Result<Vec<(String, Vec<GameRepack>)>> {
        let mut results = Vec::new();
        
        for crawler in self.get_enabled_crawlers() {
            match crawler.crawl_page(page_num).await {
                Ok(repacks) => {
                    println!("  ✅ {} - Found {} repacks", crawler.site_name(), repacks.len());
                    results.push((crawler.site_id().to_string(), repacks));
                }
                Err(e) => {
                    eprintln!("  ❌ {} - Error: {}", crawler.site_name(), e);
                }
            }
        }
        
        Ok(results)
    }
}

impl Default for CrawlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

