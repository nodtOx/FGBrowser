use anyhow::Result;
use reqwest::Client;
use scraper::{Html, Selector};
use std::time::Duration;
use tokio::time::sleep;

// Re-export all public modules
pub mod models;
pub mod title_cleaner;
pub mod extractors;
pub mod popular;
pub mod site_crawler;
pub mod fitgirl;
pub mod riotpixels;

// Re-export commonly used types for convenience
pub use models::*;
pub use title_cleaner::clean_game_title;
pub use popular::PopularRepacks;
pub use site_crawler::{SiteCrawler, CrawlerRegistry};
pub use fitgirl::FitGirlCrawler as FitGirlSiteCrawler;
use extractors::Extractors;

pub struct FitGirlCrawler {
    client: Client,
    base_url: String,
    crawl_delay: Duration,
    blacklist: Vec<String>,
    riotpixels_client: riotpixels::RiotPixelsClient,
}

impl FitGirlCrawler {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .timeout(Duration::from_secs(30))
            .build()?;

        let blacklist = Self::load_blacklist();
        let riotpixels_client = riotpixels::RiotPixelsClient::new()?;

        Ok(Self {
            client,
            base_url: "https://fitgirl-repacks.site".to_string(),
            crawl_delay: Duration::from_secs(1),
            blacklist,
            riotpixels_client,
        })
    }

    fn load_blacklist() -> Vec<String> {
        // Try to read blacklist from file
        if let Ok(content) = std::fs::read_to_string("config/blacklist.txt") {
            content
                .lines()
                .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
                .map(|line| line.trim().to_lowercase())
                .collect()
        } else {
            // Default blacklist patterns
            vec![
                "upcoming-repacks".to_string(),
                "updates-digest".to_string(),
                "russian movies".to_string(),
                "movies weekend".to_string(),
                "movies #".to_string(),
            ]
        }
    }


    fn is_blacklisted(&self, url: &str, title: &str) -> bool {
        let url_lower = url.to_lowercase();
        let title_lower = title.to_lowercase();

        self.blacklist.iter().any(|pattern| {
            url_lower.contains(pattern) || title_lower.contains(pattern)
        })
    }


    async fn fetch_page(&self, url: &str) -> Result<String> {
        sleep(self.crawl_delay).await;
        let response = self.client.get(url).send().await?;
        let text = response.text().await?;
        Ok(text)
    }

    pub async fn crawl_page(&self, page_num: u32) -> Result<Vec<GameRepack>> {
        let url = if page_num == 1 {
            self.base_url.clone()
        } else {
            format!("{}/page/{}/", self.base_url, page_num)
        };

        let html = self.fetch_page(&url).await?;
        let document = Html::parse_document(&html);
        
        // Extract total pages from pagination (only on first page)
        if page_num == 1 {
            if let Some(_total) = Extractors::extract_total_pages(&document) {
                // println!("  [INFO] Total pages available: {}", _total);
            }
        }

        let article_selector = Selector::parse("article").unwrap();
        let mut repacks = Vec::new();

        for article in document.select(&article_selector) {
            if let Some(repack) = self.extract_repack_from_article(&article) {
                if self.is_blacklisted(&repack.url, &repack.title) {
                    continue;
                }
                repacks.push(repack);
            }
        }

        Ok(repacks)
    }
    
    pub async fn crawl_single_game(&self, url: &str) -> Result<Option<GameRepack>> {
        println!("Crawling single game: {}", url);
        
        let html = self.fetch_page(url).await?;
        
        // Extract all data from document (sync operation, must complete before any await)
        let mut repack = {
            let document = Html::parse_document(&html);
            let article_selector = Selector::parse("article").unwrap();
            
            if let Some(article) = document.select(&article_selector).next() {
                self.extract_repack_from_single_page(&article, url)
            } else {
                println!("  [DEBUG] No article element found");
                None
            }
        }; // Html document is dropped here
        
        // Now handle screenshots asynchronously (after document is dropped)
        if let Some(ref mut repack) = repack {
            if self.is_blacklisted(&repack.url, &repack.title) {
                println!("  [SKIP] Blacklisted: {}", repack.title);
                return Ok(None);
            }
            
            // Fetch screenshots from RiotPixels if URL exists
            if !repack.screenshots.is_empty() {
                if let Some(riotpixels_url) = repack.screenshots.first().cloned() {
                    let cleaned_url = riotpixels::RiotPixelsClient::clean_screenshot_url(&riotpixels_url);
                    println!("    [DEBUG] Fetching screenshots from: {}", cleaned_url);
                    match self.riotpixels_client.fetch_screenshots(&cleaned_url).await {
                        Ok(urls) => {
                            println!("    [DEBUG] Found {} screenshots", urls.len());
                            repack.screenshots = urls;
                        },
                        Err(e) => {
                            println!("    [WARNING] Failed to fetch RiotPixels screenshots: {}", e);
                            repack.screenshots = Vec::new();
                        }
                    }
                }
            }
            
            println!("  [✓] Extracted: {}", repack.title);
        } else {
            println!("  [DEBUG] Failed to extract data from article");
        }
        
        Ok(repack)
    }
    
    fn extract_repack_from_single_page(&self, article: &scraper::ElementRef, url: &str) -> Option<GameRepack> {
        // On single game pages, the title is in h1.entry-title WITHOUT a link
        let title_selector = Selector::parse("h1.entry-title").unwrap();
        let title_elem = article.select(&title_selector).next();
        
        if title_elem.is_none() {
            println!("    [DEBUG] h1.entry-title not found");
            return None;
        }
        
        let title = title_elem.unwrap().text().collect::<String>().trim().to_string();
        
        if title.is_empty() {
            println!("    [DEBUG] Title is empty");
            return None;
        }
        
        println!("    [DEBUG] Found title: {}", title);

        // Extract date
        let date_selector = Selector::parse("time.entry-date").unwrap();
        let date = article.select(&date_selector).next().and_then(|elem| {
            elem.value()
                .attr("datetime")
                .map(|s| s.to_string())
                .or_else(|| {
                    let text: String = elem.text().collect();
                    Some(text.trim().to_string())
                })
        });
        
        if let Some(ref d) = date {
            println!("    [DEBUG] Found date: {}", d);
        }

        // Extract content
        let content_selector = Selector::parse("div.entry-content").unwrap();
        let content = article.select(&content_selector).next();
        
        if content.is_none() {
            println!("    [DEBUG] div.entry-content not found");
            return None;
        }
        
        let content = content.unwrap();
        println!("    [DEBUG] Found entry-content div");

        // Extract game details
        let details = Extractors::extract_game_details(&content);
        println!("    [DEBUG] Details - genres: {:?}, repack_size: {:?}", 
                 details.genres_tags.as_ref().map(|s| &s[..s.len().min(50)]), 
                 details.repack_size);

        // Extract cover image
        let image_url = Extractors::extract_cover_image(&content);
        if let Some(ref img) = image_url {
            println!("    [DEBUG] Found cover image: {}...", &img[..img.len().min(60)]);
        }

        // Extract magnet links
        let magnet_links = Extractors::extract_magnet_links(&content);
        
        println!("    [DEBUG] Found {} magnet links", magnet_links.len());
        if magnet_links.is_empty() {
            println!("    [WARNING] No magnet links found!");
        }

        // Store RiotPixels URL temporarily in screenshots vector (will be fetched async later)
        // This is a workaround to avoid holding ElementRef across await
        let screenshots = if let Some(riotpixels_url) = Extractors::extract_riotpixels_screenshot_url(&content) {
            vec![riotpixels_url]  // Will be replaced with actual screenshots later
        } else {
            Vec::new()
        };

        // Extract video/GIF URLs
        let videos = Extractors::extract_video_urls(&content);
        println!("    [DEBUG] Found {} video/GIF links", videos.len());

        Some(GameRepack {
            title,
            genres_tags: details.genres_tags,
            company: details.company,
            languages: details.languages,
            original_size: details.original_size,
            repack_size: details.repack_size,
            url: url.to_string(),
            date,
            image_url,
            magnet_links,
            screenshots,
            videos,
        })
    }

    fn extract_repack_from_article(&self, article: &scraper::ElementRef) -> Option<GameRepack> {
        // Extract title and URL
        let title_selector = Selector::parse("h1.entry-title, h2.entry-title").unwrap();
        let title_elem = article.select(&title_selector).next()?;
        
        let link_selector = Selector::parse("a").unwrap();
        let link = title_elem.select(&link_selector).next()?;
        
        let title = link.text().collect::<String>().trim().to_string();
        let url = link.value().attr("href")?.to_string();

        // Extract date
        let date_selector = Selector::parse("time.entry-date").unwrap();
        let date = article.select(&date_selector).next().and_then(|elem| {
            elem.value()
                .attr("datetime")
                .map(|s| s.to_string())
                .or_else(|| {
                    let text: String = elem.text().collect();
                    Some(text.trim().to_string())
                })
        });

        // Extract content
        let content_selector = Selector::parse("div.entry-content").unwrap();
        let content = article.select(&content_selector).next()?;

        // Extract game details
        let details = Extractors::extract_game_details(&content);

        // Extract cover image
        let image_url = Extractors::extract_cover_image(&content);

        // Extract magnet links
        let magnet_links = Extractors::extract_magnet_links(&content);

        // Screenshots can't be fetched here (sync context)
        // If needed, crawl_page will handle it
        let screenshots = Vec::new();

        // Extract video/GIF URLs
        let videos = Extractors::extract_video_urls(&content);

        Some(GameRepack {
            title,
            genres_tags: details.genres_tags,
            company: details.company,
            languages: details.languages,
            original_size: details.original_size,
            repack_size: details.repack_size,
            url,
            date,
            image_url,
            magnet_links,
            screenshots,
            videos,
        })
    }

    #[allow(dead_code)]
    pub async fn crawl_multiple_pages(&self, start_page: u32, max_pages: Option<u32>) -> Result<Vec<GameRepack>> {
        let mut all_repacks = Vec::new();
        let mut current_page = start_page;

        println!("\n{}", "=".repeat(80));
        if let Some(max) = max_pages {
            println!("CRAWLING PAGES {} to {}", start_page, start_page + max - 1);
        } else {
            println!("CRAWLING PAGES {} to END (infinite mode)", start_page);
        }
        println!("{}", "=".repeat(80));

        loop {
            // Check if we've reached max_pages
            if let Some(max) = max_pages {
                if current_page >= start_page + max {
                    break;
                }
            }

            match self.crawl_page(current_page).await {
                Ok(repacks) => {
                    if repacks.is_empty() {
                        println!("\nNo more content found at page {}", current_page);
                        println!("Reached end of available pages.");
                        break;
                    }

                    let count = repacks.len();
                    all_repacks.extend(repacks);
                    println!(
                        "[OK] Page {}: Found {} games (Total: {})",
                        current_page,
                        count,
                        all_repacks.len()
                    );
                }
                Err(e) => {
                    eprintln!("Error crawling page {}: {}", current_page, e);
                    break;
                }
            }

            current_page += 1;
        }

        println!("\n{}", "=".repeat(80));
        println!("CRAWLING COMPLETE: {} games found", all_repacks.len());
        println!("{}", "=".repeat(80));

        Ok(all_repacks)
    }
    
    pub async fn fetch_popular_repacks(&self, period: &str) -> Result<Vec<PopularRepackEntry>> {
        // For "week" and "today", we fetch from homepage (or any page) to get sidebar sections
        // For "month", "year", "award", we fetch from dedicated pages
        let url = match period {
            "year" => format!("{}/popular-repacks-of-the-year/", self.base_url),
            "award" => format!("{}/games-with-my-personal-pink-paw-award/", self.base_url),
            "week" | "today" => self.base_url.clone(), // Fetch homepage for sidebar sections
            _ => format!("{}/popular-repacks/", self.base_url), // month
        };
        
        // println!("Fetching popular repacks ({})...", period);
        
        let response = self.client.get(&url).send().await?;
        let html = response.text().await?;
        
        // Use appropriate parser based on period
        let mut entries = if period == "award" {
            // Award list is curated and doesn't need filtering
            PopularRepacks::parse_pink_paw_award_html(&html)?
        } else if period == "week" {
            PopularRepacks::parse_week_popular_repacks(&html)?
        } else if period == "today" {
            PopularRepacks::parse_today_popular_repacks(&html)?
        } else {
            // month/year
            PopularRepacks::parse_popular_repacks_html(&html)?
        };
        
        // Apply blacklist filtering for all periods except award
        if period != "award" {
            entries.retain(|entry| !is_popular_blacklisted(&entry.url));
            // println!("  Filtered out blacklisted games for period: {}", period);
        }
        
        Ok(entries)
    }
    
    pub fn parse_popular_repacks_from_file(&self, file_path: &str) -> Result<Vec<PopularRepackEntry>> {
        PopularRepacks::parse_popular_repacks_from_file(file_path)
    }
}

// Helper functions for popular games blacklist
fn is_popular_blacklisted(url: &str) -> bool {
    let url_lower = url.to_lowercase();
    let popular_blacklist = load_popular_blacklist();
    
    popular_blacklist.iter().any(|pattern| {
        url_lower.contains(pattern)
    })
}

fn load_popular_blacklist() -> Vec<String> {
    // Hardcoded blacklist for popular games (NSFW/adult content)
    vec![
        "the-genesis-order".to_string(),
        "one-more-night".to_string(),
        "honeycome-come-come-party".to_string(),
        "honey-select-2-libido".to_string(),
        "gym-manager".to_string(),
        "nymphomaniac-sex-addict".to_string(),
        "lust-n-dead".to_string(),
        "violet".to_string(),
        "roomgirl-paradise".to_string(),
        "house-party".to_string(),
        "venus-vacation-prism-dead-or-alive-xtreme".to_string(),
        "under-the-witch-heros-journey".to_string(),
        "taboo-trial".to_string(),
        "beach-club-simulator-2024".to_string(),
        "succubus".to_string(),
    ]
}

/// Create a crawler registry with all available sites
pub fn create_crawler_registry() -> Result<CrawlerRegistry> {
    let mut registry = CrawlerRegistry::new();
    
    // Register FitGirl Repacks crawler
    let fitgirl = FitGirlSiteCrawler::new()?;
    registry.register(Box::new(fitgirl));
    
    // Future sites can be added here:
    // let dodi = DodiRepacksCrawler::new()?;
    // registry.register(Box::new(dodi));
    
    Ok(registry)
}

