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
    pub image_url: Option<String>,
    pub magnet_links: Vec<MagnetLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagnetLink {
    pub source: String,
    pub magnet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopularRepackEntry {
    pub url: String,
    pub title: String,
    pub image_url: Option<String>,
}

/// Cleans a game title by removing version numbers, DLC info, editions, and other clutter
/// while preserving the core game name
pub fn clean_game_title(title: &str) -> String {
    let mut cleaned = title.to_string();
    
    // Remove everything after the first slash (for cases like "v20220613/Build 8796429")
    let slash_regex = Regex::new(r"/.*").unwrap();
    cleaned = slash_regex.replace_all(&cleaned, "").to_string();
    
    // Remove everything after comma followed by version (for cases like ", v1.5.1 (26.09.25)")
    let comma_version_regex = Regex::new(r",\s*v\d+.*").unwrap();
    cleaned = comma_version_regex.replace_all(&cleaned, "").to_string();
    
    // Remove everything after comma followed by Build
    let comma_build_regex = Regex::new(r",\s*Build\s+\d+.*").unwrap();
    cleaned = comma_build_regex.replace_all(&cleaned, "").to_string();
    
    // Remove version patterns (v1.0.1, v1.2.2, etc.) - but only if they start with dash
    let version_regex = Regex::new(r"\s*[–\-–]\s*v\d+(?:\.\d+)*(?:\.\d+)*(?:\.\d+)*").unwrap();
    cleaned = version_regex.replace_all(&cleaned, "").to_string();
    
    // Remove build patterns (Build 12345, Build 20224620, etc.)
    let build_regex = Regex::new(r"\s*[–\-–]\s*Build\s+\d+").unwrap();
    cleaned = build_regex.replace_all(&cleaned, "").to_string();
    
    // Remove revision patterns (r34045, r49909, etc.) - handle both dash and dot cases
    let revision_regex = Regex::new(r"\s*[–\-–]\s*r\d+").unwrap();
    cleaned = revision_regex.replace_all(&cleaned, "").to_string();
    
    let revision_dot_regex = Regex::new(r"\.r\d+").unwrap();
    cleaned = revision_dot_regex.replace_all(&cleaned, "").to_string();
    
    // Remove standalone revision patterns (r34045 without prefix)
    let revision_standalone_regex = Regex::new(r"\s+r\d+").unwrap();
    cleaned = revision_standalone_regex.replace_all(&cleaned, "").to_string();
    
    // Remove date patterns (26.09.2025, 20250831_2044, etc.)
    let date_regex = Regex::new(r"\s*[–\-–]\s*\d{1,2}\.\d{1,2}\.\d{4}").unwrap();
    cleaned = date_regex.replace_all(&cleaned, "").to_string();
    
    let date_regex2 = Regex::new(r"\s*[–\-–]\s*\d{8}_\d{4}-\d+").unwrap();
    cleaned = date_regex2.replace_all(&cleaned, "").to_string();
    
    // Remove edition patterns (Deluxe Edition, Premium Edition, etc.) - but preserve "Complete Edition"
    // Remove editions that come after a dash, colon, or comma
    let edition_regex = Regex::new(r"\s*[–\-–:]\s*(?:Digital\s+)?(?:Deluxe|Premium|Ultimate|Gold|Special|Collector'?s?|Game\s+of\s+the\s+Year)\s+Edition").unwrap();
    cleaned = edition_regex.replace_all(&cleaned, "").to_string();
    
    // Also handle editions after commas
    let edition_comma_regex = Regex::new(r",\s*(?:Digital\s+)?(?:Deluxe|Premium|Ultimate|Gold|Special|Collector'?s?|Game\s+of\s+the\s+Year)\s+Edition").unwrap();
    cleaned = edition_comma_regex.replace_all(&cleaned, "").to_string();
    
    // Remove DLC and bonus content patterns
    let dlc_regex = Regex::new(r"\s*[–\-–]\s*v\d+(?:\.\d+)*(?:\.\d+)*\s*\+.*").unwrap();
    cleaned = dlc_regex.replace_all(&cleaned, "").to_string();
    
    let dlc_regex2 = Regex::new(r"\s*,\s*v\d+(?:\.\d+)*(?:\.\d+)*.*").unwrap();
    cleaned = dlc_regex2.replace_all(&cleaned, "").to_string();
    
    // Remove DLC counts and descriptions
    let dlc_content_regex = Regex::new(r"\s*\+.*").unwrap();
    cleaned = dlc_content_regex.replace_all(&cleaned, "").to_string();
    
    // Remove parenthetical content (Denuvoless, Campaign Only, etc.)
    let paren_regex = Regex::new(r"\s*\([^)]*\)").unwrap();
    cleaned = paren_regex.replace_all(&cleaned, "").to_string();
    
    // Remove platform indicators (GOG, Steam, etc.)
    let platform_regex = Regex::new(r"\s*[–\-–]\s*(?:GOG|Steam|GOG/Steam|MS|Epic|Origin|Uplay|Battle\.net)").unwrap();
    cleaned = platform_regex.replace_all(&cleaned, "").to_string();
    
    // Remove standalone platform indicators at the end
    let platform_end_regex = Regex::new(r"\s+(?:GOG|Steam|MS|Epic|Origin|Uplay|Battle\.net)$").unwrap();
    cleaned = platform_end_regex.replace_all(&cleaned, "").to_string();
    
    // Remove build numbers that come after version numbers
    let build_after_version_regex = Regex::new(r"\s+build\s+\d+").unwrap();
    cleaned = build_after_version_regex.replace_all(&cleaned, "").to_string();
    
    // Remove standalone build numbers
    let standalone_build_regex = Regex::new(r"\s*[–\-–]\s*build\s+\d+").unwrap();
    cleaned = standalone_build_regex.replace_all(&cleaned, "").to_string();
    
    // Remove hotfix patterns
    let hotfix_regex = Regex::new(r"\s*[–\-–]\s*Hotfix\s+\d+").unwrap();
    cleaned = hotfix_regex.replace_all(&cleaned, "").to_string();
    
    // Remove standalone "Release" at the end (with or without dash)
    let release_regex = Regex::new(r"\s*[–\-–]?\s*Release$").unwrap();
    cleaned = release_regex.replace_all(&cleaned, "").to_string();
    
    // Remove "Data Pack" patterns
    let datapack_regex = Regex::new(r"\s*Data\s+Pack\s+\d+\.\d+$").unwrap();
    cleaned = datapack_regex.replace_all(&cleaned, "").to_string();
    
    // Remove repack indicators
    let repack_regex = Regex::new(r"\s*[–\-–]\s*(?:Monkey|Turtle|Compressed|BetterRepack).*$").unwrap();
    cleaned = repack_regex.replace_all(&cleaned, "").to_string();
    
    // Remove specific edition patterns that weren't caught before
    let specific_edition_regex = Regex::new(r"\s*[–\-–]\s*(?:Jackdaw|Supporter|Anniversary|Limited|Collector's?|Special|Enhanced|Definitive|Remastered|Director's? Cut|Game\s+of\s+[Tt]he\s+Year|Master\s+Crafted|Khaos\s+Reigns\s+Kollection)\s+Edition").unwrap();
    cleaned = specific_edition_regex.replace_all(&cleaned, "").to_string();
    
    // Remove trailing punctuation and clean up spacing
    cleaned = cleaned.trim().to_string();
    
    // Remove multiple consecutive spaces
    let space_regex = Regex::new(r"\s+").unwrap();
    cleaned = space_regex.replace_all(&cleaned, " ").to_string();
    
    // Remove trailing commas, dashes, and colons
    let trailing_regex = Regex::new(r"[,:\-–\s]+$").unwrap();
    cleaned = trailing_regex.replace_all(&cleaned, "").to_string();
    
    cleaned
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

        // println!("Crawling page {}: {}", page_num, url);

        let html = self.fetch_page(&url).await?;
        let document = Html::parse_document(&html);
        
        // Extract total pages from pagination (only on first page)
        if page_num == 1 {
            if let Some(_total) = self.extract_total_pages(&document) {
                // println!("  [INFO] Total pages available: {}", _total);
            }
        }

        let article_selector = Selector::parse("article").unwrap();
        let mut repacks = Vec::new();

        for article in document.select(&article_selector) {
            if let Some(repack) = self.extract_repack_from_article(&article) {
                if self.is_blacklisted(&repack.url, &repack.title) {
                    // println!("  [SKIP] {}", repack.title);
                    continue;
                }
                // println!("  [+] {}", repack.title);
                repacks.push(repack);
            }
        }

        Ok(repacks)
    }
    
    pub async fn crawl_single_game(&self, url: &str) -> Result<Option<GameRepack>> {
        println!("Crawling single game: {}", url);
        
        let html = self.fetch_page(url).await?;
        let document = Html::parse_document(&html);
        
        let article_selector = Selector::parse("article").unwrap();
        
        if let Some(article) = document.select(&article_selector).next() {
            // Use specialized extraction for single game pages
            if let Some(repack) = self.extract_repack_from_single_page(&article, url) {
                if self.is_blacklisted(&repack.url, &repack.title) {
                    println!("  [SKIP] Blacklisted: {}", repack.title);
                    return Ok(None);
                }
                println!("  [✓] Extracted: {}", repack.title);
                return Ok(Some(repack));
            } else {
                println!("  [DEBUG] Failed to extract data from article");
            }
        } else {
            println!("  [DEBUG] No article element found");
        }
        
        Ok(None)
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
        let details = self.extract_game_details(&content);
        println!("    [DEBUG] Details - genres: {:?}, repack_size: {:?}", 
                 details.genres_tags.as_ref().map(|s| &s[..s.len().min(50)]), 
                 details.repack_size);

        // Extract cover image
        let image_url = self.extract_cover_image(&content);
        if let Some(ref img) = image_url {
            println!("    [DEBUG] Found cover image: {}...", &img[..img.len().min(60)]);
        }

        // Extract magnet links
        let magnet_links = self.extract_magnet_links(&content);
        
        println!("    [DEBUG] Found {} magnet links", magnet_links.len());
        if magnet_links.is_empty() {
            println!("    [WARNING] No magnet links found!");
        }

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
        })
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

        // Extract cover image
        let image_url = self.extract_cover_image(&content);

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
            image_url,
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
            
            // println!("[DEBUG] Combined text from siblings: {}", &full_text[..full_text.len().min(500)]);
            
            // Extract using simple string operations (like Python approach)
            // Genres/Tags
            if let Some(genres_start) = full_text.find("Genres/Tags:") {
                let after = &full_text[genres_start + 12..];
                let end = after.find("Companies:").or_else(|| after.find("Company:")).unwrap_or(after.len());
                details.genres_tags = Some(after[..end].trim().to_string());
                // println!("[DEBUG] Extracted Genres: {:?}", details.genres_tags);
            }
            
            // Company/Companies
            if let Some(start) = full_text.find("Companies:") {
                let after = &full_text[start + 10..];
                let end = after.find("Languages:").or_else(|| after.find("Language:")).unwrap_or(after.len());
                details.company = Some(after[..end].trim().to_string());
                // println!("[DEBUG] Extracted Company: {:?}", details.company);
            } else if let Some(start) = full_text.find("Company:") {
                let after = &full_text[start + 8..];
                let end = after.find("Languages:").or_else(|| after.find("Language:")).unwrap_or(after.len());
                details.company = Some(after[..end].trim().to_string());
                // println!("[DEBUG] Extracted Company: {:?}", details.company);
            }
            
            // Languages
            if let Some(start) = full_text.find("Languages:") {
                let after = &full_text[start + 10..];
                let end = after.find("Original Size:").or_else(|| after.find("This game")).unwrap_or(after.len());
                details.languages = Some(after[..end].trim().to_string());
                // println!("[DEBUG] Extracted Languages: {:?}", details.languages);
            } else if let Some(start) = full_text.find("Language:") {
                let after = &full_text[start + 9..];
                let end = after.find("Original Size:").or_else(|| after.find("This game")).unwrap_or(after.len());
                details.languages = Some(after[..end].trim().to_string());
                // println!("[DEBUG] Extracted Languages: {:?}", details.languages);
            }
            
            // Original Size
            if let Some(start) = full_text.find("Original Size:") {
                let after = &full_text[start + 14..];
                let end = after.find("Repack Size:").unwrap_or(after.len().min(100));
                details.original_size = Some(after[..end].trim().to_string());
                // println!("[DEBUG] Extracted Original Size: {:?}", details.original_size);
            }
            
            // Repack Size
            if let Some(start) = full_text.find("Repack Size:") {
                let after = &full_text[start + 12..];
                let end = after.find("Download").unwrap_or(after.len().min(100));
                details.repack_size = Some(after[..end].trim().to_string());
                // println!("[DEBUG] Extracted Repack Size: {:?}", details.repack_size);
            }
        }

        details
    }
    
    #[allow(dead_code)]
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

    fn extract_cover_image(&self, content: &scraper::ElementRef) -> Option<String> {
        // Look for the cover image - usually first img tag with width="150" or class="alignleft"
        let img_selector = Selector::parse("img").unwrap();
        
        for img in content.select(&img_selector) {
            // Check if it's likely a cover image (width=150 or alignleft class)
            let width = img.value().attr("width");
            let class = img.value().attr("class");
            let src = img.value().attr("src");
            
            if let Some(src_url) = src {
                // Cover images are usually 150px wide and alignleft
                if (width == Some("150") || class.map(|c| c.contains("alignleft")).unwrap_or(false))
                    && !src_url.is_empty() {
                    return Some(src_url.to_string());
                }
            }
        }
        
        None
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
        let url = match period {
            "year" => format!("{}/popular-repacks-of-the-year/", self.base_url),
            _ => format!("{}/popular-repacks/", self.base_url),
        };
        
        println!("Fetching popular repacks ({})...", period);
        
        let response = self.client.get(&url).send().await?;
        let html = response.text().await?;
        
        self.parse_popular_repacks_html(&html)
    }
    
    pub fn parse_popular_repacks_html(&self, html: &str) -> Result<Vec<PopularRepackEntry>> {
        let document = Html::parse_document(html);
        
        // Find the main content area (entry-content within article) to avoid sidebar items
        let article_selector = Selector::parse("article .entry-content").unwrap();
        let article_content = document.select(&article_selector).next()
            .ok_or_else(|| anyhow::anyhow!("Could not find article .entry-content"))?;
        
        let grid_item_selector = Selector::parse("div.widget-grid-view-image").unwrap();
        let link_selector = Selector::parse("a").unwrap();
        let img_selector = Selector::parse("img").unwrap();
        
        let mut popular_repacks = Vec::new();
        
        for grid_item in article_content.select(&grid_item_selector) {
            // Find the link
            if let Some(link) = grid_item.select(&link_selector).next() {
                let url = link.value().attr("href").unwrap_or("").to_string();
                let title = link.value().attr("title").unwrap_or("").to_string();
                
                // Find the image
                let image_url = if let Some(img) = grid_item.select(&img_selector).next() {
                    img.value().attr("src").map(|s| s.to_string())
                } else {
                    None
                };
                
                // Skip if URL or title is empty
                if url.is_empty() || title.is_empty() {
                    continue;
                }
                
                popular_repacks.push(PopularRepackEntry {
                    url,
                    title,
                    image_url,
                });
            }
        }
        
        println!("Found {} popular repacks (rank will be set during insertion)", popular_repacks.len());
        Ok(popular_repacks)
    }
    
    pub fn parse_popular_repacks_from_file(&self, file_path: &str) -> Result<Vec<PopularRepackEntry>> {
        let html = std::fs::read_to_string(file_path)?;
        self.parse_popular_repacks_html(&html)
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

