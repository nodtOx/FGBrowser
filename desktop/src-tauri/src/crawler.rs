use anyhow::{Context, Result};
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameRepack {
    pub title: String,
    pub genres_tags: Option<String>,
    pub company: Option<String>,
    pub languages: Option<String>,
    pub original_size: Option<String>,
    pub repack_size: Option<String>,
    pub url: String,
    pub date: Option<String>,
    pub magnet_links: Vec<MagnetLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagnetLink {
    pub source: String,
    pub magnet: String,
}

pub struct FitGirlCrawler {
    client: Client,
    base_url: String,
    crawl_delay: Duration,
    blacklist: Vec<String>,
}

impl FitGirlCrawler {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .timeout(Duration::from_secs(30))
            .build()?;

        let blacklist = Self::load_blacklist();

        Ok(Self {
            client,
            base_url: "https://fitgirl-repacks.site".to_string(),
            crawl_delay: Duration::from_secs(1),
            blacklist,
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

        println!("Crawling page {}: {}", page_num, url);

        let html = self.fetch_page(&url).await?;
        let document = Html::parse_document(&html);

        let article_selector = Selector::parse("article").unwrap();
        let mut repacks = Vec::new();

        for article in document.select(&article_selector) {
            if let Some(repack) = self.extract_repack_from_article(&article) {
                if self.is_blacklisted(&repack.url, &repack.title) {
                    println!("  [SKIP] {}", repack.title);
                    continue;
                }
                println!("  [+] {}", repack.title);
                repacks.push(repack);
            }
        }

        Ok(repacks)
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
                .or_else(|| Some(elem.text().collect::<String>().trim()))
                .map(|s| s.to_string())
        });

        // Extract content
        let content_selector = Selector::parse("div.entry-content").unwrap();
        let content = article.select(&content_selector).next()?;

        // Extract game details
        let details = self.extract_game_details(content);

        // Extract magnet links
        let magnet_links = self.extract_magnet_links(content);

        Some(GameRepack {
            title,
            genres_tags: details.genres_tags,
            company: details.company,
            languages: details.languages,
            original_size: details.original_size,
            repack_size: details.repack_size,
            url,
            date,
            magnet_links,
        })
    }

    fn extract_game_details(&self, content: &scraper::ElementRef) -> GameDetails {
        let mut details = GameDetails::default();

        // Get all text from content
        let full_text: String = content.text().collect();

        // Extract genres/tags
        if let Ok(re) = Regex::new(r"(?i)Genres?[/\s]*Tags?:\s*(.+?)(?=Compan|Languages?|Original|$)") {
            if let Some(caps) = re.captures(&full_text) {
                details.genres_tags = Some(caps[1].trim().to_string());
            }
        }

        // Extract company
        if let Ok(re) = Regex::new(r"(?i)Compan(?:y|ies):\s*(.+?)(?=Languages?|Original|Repack|$)") {
            if let Some(caps) = re.captures(&full_text) {
                details.company = Some(caps[1].trim().to_string());
            }
        }

        // Extract languages
        if let Ok(re) = Regex::new(r"(?i)Languages?:\s*(.+?)(?=Original|Repack|This game|$)") {
            if let Some(caps) = re.captures(&full_text) {
                details.languages = Some(caps[1].trim().to_string());
            }
        }

        // Extract original size
        if let Ok(re) = Regex::new(r"(?i)Original Size:\s*(.+?)(?=Repack|$)") {
            if let Some(caps) = re.captures(&full_text) {
                details.original_size = Some(caps[1].trim().to_string());
            }
        }

        // Extract repack size
        if let Ok(re) = Regex::new(r"(?i)Repack Size:\s*(.+?)(?=Download|$)") {
            if let Some(caps) = re.captures(&full_text) {
                details.repack_size = Some(caps[1].trim().to_string());
            }
        }

        details
    }

    fn extract_magnet_links(&self, content: &scraper::ElementRef) -> Vec<MagnetLink> {
        let mut magnet_links = Vec::new();

        let link_selector = Selector::parse("a[href^='magnet:']").unwrap();
        
        for link in content.select(&link_selector) {
            let magnet = link.value().attr("href").unwrap_or("").to_string();
            
            // Find parent li element to get source
            let mut source = "Unknown".to_string();
            let mut current = Some(link);
            
            while let Some(elem) = current {
                if elem.value().name() == "li" {
                    let text: String = elem.text().collect();
                    // Extract source name (before | or [)
                    source = text
                        .split('|')
                        .next()
                        .unwrap_or(&text)
                        .split('[')
                        .next()
                        .unwrap_or(&text)
                        .trim()
                        .to_string();
                    break;
                }
                current = elem.parent().and_then(|p| scraper::ElementRef::wrap(p));
            }

            magnet_links.push(MagnetLink { source, magnet });
        }

        magnet_links
    }

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
}

#[derive(Default)]
struct GameDetails {
    genres_tags: Option<String>,
    company: Option<String>,
    languages: Option<String>,
    original_size: Option<String>,
    repack_size: Option<String>,
}

