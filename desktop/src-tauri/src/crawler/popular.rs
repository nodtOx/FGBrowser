use anyhow::Result;
use scraper::{Html, Selector};
use super::models::PopularRepackEntry;

pub struct PopularRepacks;

impl PopularRepacks {
    pub fn parse_popular_repacks_html(html: &str) -> Result<Vec<PopularRepackEntry>> {
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
    
    pub fn parse_popular_repacks_from_file(file_path: &str) -> Result<Vec<PopularRepackEntry>> {
        let html = std::fs::read_to_string(file_path)?;
        Self::parse_popular_repacks_html(&html)
    }
}

