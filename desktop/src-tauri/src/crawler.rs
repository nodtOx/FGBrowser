use anyhow::Result;
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
        
        // Extract total pages from pagination (only on first page)
        if page_num == 1 {
            if let Some(total) = self.extract_total_pages(&document) {
                println!("  [INFO] Total pages available: {}", total);
            }
        }

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
    
    fn extract_total_pages(&self, document: &Html) -> Option<u32> {
        let pagination_selector = Selector::parse(".pagination a.page-numbers").unwrap();
        let mut max_page = 1u32;
        
        for elem in document.select(&pagination_selector) {
            if let Some(href) = elem.value().attr("href") {
                // Extract page number from URL like "/page/673/"
                if let Some(page_str) = href.split("/page/").nth(1) {
                    if let Some(num_str) = page_str.split('/').next() {
                        if let Ok(page_num) = num_str.parse::<u32>() {
                            max_page = max_page.max(page_num);
                        }
                    }
                }
            }
        }
        
        if max_page > 1 {
            Some(max_page)
        } else {
            None
        }
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
        let details = self.extract_game_details(&content);

        // Extract magnet links
        let magnet_links = self.extract_magnet_links(&content);

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

        // EXACTLY like Python: Look for the game info section (usually in <h3>)
        let h3_selector = Selector::parse("h3").unwrap();
        if let Some(info_section) = content.select(&h3_selector).next() {
            // Get all text until next heading or download section (like Python)
            let mut text_parts = Vec::new();
            
            // Collect text from all siblings until we hit another h3
            for sibling in info_section.next_siblings() {
                if let Some(element) = scraper::ElementRef::wrap(sibling) {
                    // Stop if we hit another h3
                    if element.value().name() == "h3" {
                        break;
                    }
                    // Collect text from this element
                    let text: String = element.text().collect();
                    if !text.trim().is_empty() {
                        text_parts.push(text.trim().to_string());
                    }
                }
            }
            
            // Join and parse the combined text (like Python: " ".join(text_parts))
            let full_text = text_parts.join(" ");
            
            println!("[DEBUG] Combined text from siblings: {}", &full_text[..full_text.len().min(500)]);
            
            // Extract using simple string operations (like Python approach)
            // Genres/Tags
            if let Some(genres_start) = full_text.find("Genres/Tags:") {
                let after = &full_text[genres_start + 12..];
                let end = after.find("Companies:").or_else(|| after.find("Company:")).unwrap_or(after.len());
                details.genres_tags = Some(after[..end].trim().to_string());
                println!("[DEBUG] Extracted Genres: {:?}", details.genres_tags);
            }
            
            // Company/Companies
            if let Some(start) = full_text.find("Companies:") {
                let after = &full_text[start + 10..];
                let end = after.find("Languages:").or_else(|| after.find("Language:")).unwrap_or(after.len());
                details.company = Some(after[..end].trim().to_string());
                println!("[DEBUG] Extracted Company: {:?}", details.company);
            } else if let Some(start) = full_text.find("Company:") {
                let after = &full_text[start + 8..];
                let end = after.find("Languages:").or_else(|| after.find("Language:")).unwrap_or(after.len());
                details.company = Some(after[..end].trim().to_string());
                println!("[DEBUG] Extracted Company: {:?}", details.company);
            }
            
            // Languages
            if let Some(start) = full_text.find("Languages:") {
                let after = &full_text[start + 10..];
                let end = after.find("Original Size:").or_else(|| after.find("This game")).unwrap_or(after.len());
                details.languages = Some(after[..end].trim().to_string());
                println!("[DEBUG] Extracted Languages: {:?}", details.languages);
            } else if let Some(start) = full_text.find("Language:") {
                let after = &full_text[start + 9..];
                let end = after.find("Original Size:").or_else(|| after.find("This game")).unwrap_or(after.len());
                details.languages = Some(after[..end].trim().to_string());
                println!("[DEBUG] Extracted Languages: {:?}", details.languages);
            }
            
            // Original Size
            if let Some(start) = full_text.find("Original Size:") {
                let after = &full_text[start + 14..];
                let end = after.find("Repack Size:").unwrap_or(after.len().min(100));
                details.original_size = Some(after[..end].trim().to_string());
                println!("[DEBUG] Extracted Original Size: {:?}", details.original_size);
            }
            
            // Repack Size
            if let Some(start) = full_text.find("Repack Size:") {
                let after = &full_text[start + 12..];
                let end = after.find("Download").unwrap_or(after.len().min(100));
                details.repack_size = Some(after[..end].trim().to_string());
                println!("[DEBUG] Extracted Repack Size: {:?}", details.repack_size);
            }
        }

        details
    }
    
    fn strip_html_tags(&self, html: &str) -> String {
        // Simple HTML tag stripper - replace tags with spaces to preserve word boundaries
        let tag_re = Regex::new(r"<[^>]*>").unwrap();
        let text = tag_re.replace_all(html, " ");
        
        // Decode HTML entities
        let text = text.replace("&amp;", "&");
        let text = text.replace("&lt;", "<");
        let text = text.replace("&gt;", ">");
        let text = text.replace("&quot;", "\"");
        let text = text.replace("&#038;", "&");
        let text = text.replace("&nbsp;", " ");
        
        // Normalize whitespace
        let whitespace_re = Regex::new(r"\s+").unwrap();
        whitespace_re.replace_all(&text, " ").trim().to_string()
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

