use regex::Regex;
use scraper::{ElementRef, Selector};
use super::models::{GameDetails, MagnetLink};

pub struct Extractors;

impl Extractors {
    pub fn extract_total_pages(document: &scraper::Html) -> Option<u32> {
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

    pub fn extract_game_details(content: &ElementRef) -> GameDetails {
        let mut details = GameDetails::default();

        // EXACTLY like Python: Look for the game info section (usually in <h3>)
        let h3_selector = Selector::parse("h3").unwrap();
        if let Some(info_section) = content.select(&h3_selector).next() {
            // Get all text until next heading or download section (like Python)
            let mut text_parts = Vec::new();
            
            // Collect text from all siblings until we hit another h3
            for sibling in info_section.next_siblings() {
                if let Some(element) = ElementRef::wrap(sibling) {
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
            
            // Extract using simple string operations (like Python approach)
            // Genres/Tags
            if let Some(genres_start) = full_text.find("Genres/Tags:") {
                let after = &full_text[genres_start + 12..];
                let end = after.find("Companies:").or_else(|| after.find("Company:")).unwrap_or(after.len());
                details.genres_tags = Some(after[..end].trim().to_string());
            }
            
            // Company/Companies
            if let Some(start) = full_text.find("Companies:") {
                let after = &full_text[start + 10..];
                let end = after.find("Languages:").or_else(|| after.find("Language:")).unwrap_or(after.len());
                details.company = Some(after[..end].trim().to_string());
            } else if let Some(start) = full_text.find("Company:") {
                let after = &full_text[start + 8..];
                let end = after.find("Languages:").or_else(|| after.find("Language:")).unwrap_or(after.len());
                details.company = Some(after[..end].trim().to_string());
            }
            
            // Languages
            if let Some(start) = full_text.find("Languages:") {
                let after = &full_text[start + 10..];
                let end = after.find("Original Size:").or_else(|| after.find("This game")).unwrap_or(after.len());
                details.languages = Some(after[..end].trim().to_string());
            } else if let Some(start) = full_text.find("Language:") {
                let after = &full_text[start + 9..];
                let end = after.find("Original Size:").or_else(|| after.find("This game")).unwrap_or(after.len());
                details.languages = Some(after[..end].trim().to_string());
            }
            
            // Original Size
            if let Some(start) = full_text.find("Original Size:") {
                let after = &full_text[start + 14..];
                let end = after.find("Repack Size:").unwrap_or(after.len().min(100));
                details.original_size = Some(after[..end].trim().to_string());
            }
            
            // Repack Size
            if let Some(start) = full_text.find("Repack Size:") {
                let after = &full_text[start + 12..];
                let end = after.find("Download").unwrap_or(after.len().min(100));
                details.repack_size = Some(after[..end].trim().to_string());
            }
        }

        details
    }
    
    #[allow(dead_code)]
    pub fn strip_html_tags(html: &str) -> String {
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

    pub fn extract_cover_image(content: &ElementRef) -> Option<String> {
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
    
    pub fn extract_magnet_links(content: &ElementRef) -> Vec<MagnetLink> {
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
                current = elem.parent().and_then(ElementRef::wrap);
            }

            magnet_links.push(MagnetLink { source, magnet });
        }

        magnet_links
    }

    /// Extract RiotPixels screenshot page URL from FitGirl content
    /// Returns the raw URL (will be cleaned by RiotPixels module)
    pub fn extract_riotpixels_screenshot_url(content: &ElementRef) -> Option<String> {
        // Look for <h3>Screenshots (Click to enlarge)</h3>
        let h3_selector = Selector::parse("h3").unwrap();
        
        for h3 in content.select(&h3_selector) {
            let h3_text: String = h3.text().collect();
            
            // Check if this is the Screenshots heading
            if h3_text.contains("Screenshots") && h3_text.contains("Click to enlarge") {
                // Find the first <p> element after this <h3>
                for sibling in h3.next_siblings() {
                    if let Some(element) = ElementRef::wrap(sibling) {
                        if element.value().name() == "p" {
                            // Extract the first <a> tag with riotpixels.com URL
                            let link_selector = Selector::parse("a").unwrap();
                            
                            for link in element.select(&link_selector) {
                                if let Some(href) = link.value().attr("href") {
                                    if href.contains("riotpixels.com") && href.contains("/screenshots/") {
                                        return Some(href.to_string());
                                    }
                                }
                            }
                            
                            // Stop after processing the first <p> element
                            break;
                        }
                    }
                }
                
                // Stop after finding the Screenshots heading
                break;
            }
        }
        
        None
    }

    /// Extract video/GIF URLs from FitGirl content (YouTube, Streamable, GIFs, etc.)
    /// These are in the same <p> section as screenshots but from different sources
    pub fn extract_video_urls(content: &ElementRef) -> Vec<String> {
        let mut video_urls = Vec::new();
        
        // Look for <h3>Screenshots (Click to enlarge)</h3>
        let h3_selector = Selector::parse("h3").unwrap();
        
        for h3 in content.select(&h3_selector) {
            let h3_text: String = h3.text().collect();
            
            // Check if this is the Screenshots heading
            if h3_text.contains("Screenshots") && h3_text.contains("Click to enlarge") {
                // Find the first <p> element after this <h3>
                for sibling in h3.next_siblings() {
                    if let Some(element) = ElementRef::wrap(sibling) {
                        if element.value().name() == "p" {
                            // Extract all <a> tags that are NOT riotpixels
                            let link_selector = Selector::parse("a").unwrap();
                            
                            for link in element.select(&link_selector) {
                                if let Some(href) = link.value().attr("href") {
                                    let href_lower = href.to_lowercase();
                                    
                                    // Skip RiotPixels links (they're screenshots)
                                    if href.contains("riotpixels.com") {
                                        continue;
                                    }
                                    
                                    // Check if it's a video/gif link
                                    // Common video platforms and direct media links
                                    if href_lower.contains("youtube.com") ||
                                       href_lower.contains("youtu.be") ||
                                       href_lower.contains("streamable.com") ||
                                       href_lower.contains("vimeo.com") ||
                                       href_lower.contains("gfycat.com") ||
                                       href_lower.contains("imgur.com/") ||
                                       href_lower.ends_with(".gif") ||
                                       href_lower.ends_with(".gifv") ||
                                       href_lower.ends_with(".mp4") ||
                                       href_lower.ends_with(".webm") ||
                                       href_lower.contains(".gif?") ||
                                       href_lower.contains(".mp4?") ||
                                       href_lower.contains(".webm?") {
                                        video_urls.push(href.to_string());
                                    }
                                }
                            }
                            
                            // Stop after processing the first <p> element
                            break;
                        }
                    }
                }
                
                // Stop after finding the Screenshots heading
                break;
            }
        }
        
        video_urls
    }
}

