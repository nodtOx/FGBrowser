// RiotPixels screenshot scraping functionality
// Fetches high-resolution screenshots from RiotPixels.com
// Uses curl subprocess to bypass Cloudflare (pre-installed on Windows 10+, macOS, Linux)

use anyhow::Result;
use scraper::{Html, Selector};
use serde_json::Value;
use tokio::process::Command;

#[derive(Debug, Clone)]
pub struct ScreenshotData {
    pub full_url: String,
    pub thumbnail_url: Option<String>,
}

pub struct RiotPixelsClient;

impl RiotPixelsClient {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    /// Fetch page using curl subprocess (bypasses Cloudflare bot detection)
    async fn fetch_with_curl(url: &str) -> Result<String> {
        let output = Command::new("curl")
            .arg("-A")
            .arg("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .arg(url)
            .arg("-s")  // Silent mode
            .arg("-L")  // Follow redirects
            .output()
            .await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("curl failed: {}", stderr));
        }
        
        let html = String::from_utf8(output.stdout)?;
        
        // Check if we got Cloudflare challenge page
        if html.contains("Just a moment") && html.contains("cloudflare") {
            return Err(anyhow::anyhow!("Got Cloudflare challenge page"));
        }
        
        Ok(html)
    }

    /// Clean RiotPixels URL to get base screenshot page
    /// Example: https://en.riotpixels.com/games/gta-5/screenshots/1214/?utm_source=...
    /// Becomes: https://en.riotpixels.com/games/gta-5/screenshots/?utm_source=...
    /// Preserves UTM params to credit FitGirl for the referral
    pub fn clean_screenshot_url(url: &str) -> String {
        if let Some(screenshots_pos) = url.find("/screenshots/") {
            let base_part = &url[..screenshots_pos + 13]; // Include "/screenshots/"
            
            // Check if there's a query string (UTM params)
            if let Some(query_pos) = url.find('?') {
                let query_part = &url[query_pos..];
                format!("{}{}", base_part, query_part)
            } else {
                base_part.to_string()
            }
        } else {
            url.to_string()
        }
    }

    /// Fetch and parse high-res screenshot URLs from RiotPixels page
    /// Parses the JSON data from onclick attributes as described in riotpixels_scraping.md
    /// Returns full URLs and thumbnails (240p)
    pub async fn fetch_screenshots(&self, screenshot_page_url: &str) -> Result<Vec<ScreenshotData>> {
        // Use Russian domain - it's more reliable and less likely to be blocked
        let url = screenshot_page_url
            .replace("en.riotpixels.com", "ru.riotpixels.com")
            .replace("://riotpixels.com", "://ru.riotpixels.com");
        
        println!("  [DEBUG] Fetching screenshots from RiotPixels");
        
        // Use curl directly (bypasses Cloudflare, pre-installed on modern systems)
        let html = match Self::fetch_with_curl(&url).await {
            Ok(html) => {
                println!("  [DEBUG] Successfully fetched ({} bytes)", html.len());
                html
            }
            Err(e) => {
                println!("  [WARNING] Failed to fetch screenshots: {}", e);
                return Ok(Vec::new());
            }
        };
        
        let document = Html::parse_document(&html);
        let mut screenshots = Vec::new();
        
        // Try multiple selectors - RiotPixels uses different structures
        let selectors = vec![
            "a.action-colorbox[onclick]",
            "section.gallery-list-more ul li a[onclick]",
            "section.gfx-holder ul li a[onclick]",
            "a[onclick]",
        ];
        
        for selector_str in selectors {
            if let Ok(link_selector) = Selector::parse(selector_str) {
                let links: Vec<_> = document.select(&link_selector).collect();
                
                if !links.is_empty() {
                
                for link in links {
                    if let Some(onclick_attr) = link.value().attr("onclick") {
                        // Extract JSON array from onclick='return [...]'
                        if let Some(json_start) = onclick_attr.find('[') {
                            if let Some(json_end) = onclick_attr.rfind(']') {
                                let json_str = &onclick_attr[json_start..=json_end];
                                
                                // Parse the JSON array
                                // Fields: "o" = original flag (1 = full res), "s" = file size in bytes, "u" = URL
                                if let Ok(json_array) = serde_json::from_str::<Vec<Value>>(json_str) {
                                    // Find full resolution: look for o=1 OR largest file size
                                    let full_res_item = json_array.iter()
                                        .find(|item| item.get("o").and_then(|o| o.as_i64()) == Some(1))
                                        .or_else(|| {
                                            // Fallback: find largest file size
                                            json_array.iter()
                                                .max_by_key(|item| item.get("s").and_then(|s| s.as_i64()).unwrap_or(0))
                                        });
                                    
                                    // Find thumbnail: smallest file size
                                    let thumbnail_item = json_array.iter()
                                        .min_by_key(|item| item.get("s").and_then(|s| s.as_i64()).unwrap_or(i64::MAX));
                                    
                                    if let Some(full_item) = full_res_item {
                                        if let Some(full_url) = full_item.get("u").and_then(|u| u.as_str()) {
                                            let full_url = full_url.replace("http:", "https:");
                                            
                                            let thumbnail_url = thumbnail_item
                                                .and_then(|item| item.get("u").and_then(|u| u.as_str()))
                                                .map(|url| url.replace("http:", "https:"));
                                            
                                            screenshots.push(ScreenshotData {
                                                full_url,
                                                thumbnail_url,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                    // If we found screenshots, don't try other selectors
                    break;
                }
            }
        }
        
        Ok(screenshots)
    }
}

