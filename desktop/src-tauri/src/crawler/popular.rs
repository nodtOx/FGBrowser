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
        
        // println!("Found {} popular repacks (rank will be set during insertion)", popular_repacks.len());
        Ok(popular_repacks)
    }
    
    pub fn parse_pink_paw_award_html(html: &str) -> Result<Vec<PopularRepackEntry>> {
        let document = Html::parse_document(html);
        
        // Find the main content area
        let article_selector = Selector::parse("article .entry-content").unwrap();
        let article_content = document.select(&article_selector).next()
            .ok_or_else(|| anyhow::anyhow!("Could not find article .entry-content"))?;
        
        // Find the list (ul.lcp_catlist)
        let list_selector = Selector::parse("ul.lcp_catlist").unwrap();
        let list = article_content.select(&list_selector).next()
            .ok_or_else(|| anyhow::anyhow!("Could not find ul.lcp_catlist"))?;
        
        let li_selector = Selector::parse("li").unwrap();
        let link_selector = Selector::parse("a").unwrap();
        
        let mut popular_repacks = Vec::new();
        
        for li in list.select(&li_selector) {
            // Find the link
            if let Some(link) = li.select(&link_selector).next() {
                let url = link.value().attr("href").unwrap_or("").to_string();
                let title = link.text().collect::<String>().trim().to_string();
                
                // Skip if URL or title is empty
                if url.is_empty() || title.is_empty() {
                    continue;
                }
                
                // No images in this format
                popular_repacks.push(PopularRepackEntry {
                    url,
                    title,
                    image_url: None,
                });
            }
        }
        
        // println!("Found {} Pink Paw Award games (rank will be set during insertion)", popular_repacks.len());
        Ok(popular_repacks)
    }
    
    pub fn parse_popular_repacks_from_file(file_path: &str) -> Result<Vec<PopularRepackEntry>> {
        let html = std::fs::read_to_string(file_path)?;
        Self::parse_popular_repacks_html(&html)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pink_paw_award() {
        let html = r#"
            <article>
                <div class="entry-content">
                    <ul class="lcp_catlist">
                        <li><a href="https://fitgirl-repacks.site/game-one/">Game One Title</a></li>
                        <li><a href="https://fitgirl-repacks.site/game-two/">Game Two Title</a></li>
                        <li><a href="https://fitgirl-repacks.site/game-three/">Game Three: Special Edition</a></li>
                    </ul>
                </div>
            </article>
        "#;

        let result = PopularRepacks::parse_pink_paw_award_html(html);
        assert!(result.is_ok());
        
        let entries = result.unwrap();
        assert_eq!(entries.len(), 3);
        
        assert_eq!(entries[0].title, "Game One Title");
        assert_eq!(entries[0].url, "https://fitgirl-repacks.site/game-one/");
        assert_eq!(entries[0].image_url, None);
        
        assert_eq!(entries[1].title, "Game Two Title");
        assert_eq!(entries[2].title, "Game Three: Special Edition");
    }

    #[test]
    fn test_parse_popular_repacks_with_images() {
        let html = r#"
            <article>
                <div class="entry-content">
                    <div class="widget-grid-view-image">
                        <a href="https://fitgirl-repacks.site/game-one/" title="Game One Title">
                            <img src="https://example.com/image1.jpg" />
                        </a>
                    </div>
                    <div class="widget-grid-view-image">
                        <a href="https://fitgirl-repacks.site/game-two/" title="Game Two Title">
                            <img src="https://example.com/image2.jpg" />
                        </a>
                    </div>
                </div>
            </article>
        "#;

        let result = PopularRepacks::parse_popular_repacks_html(html);
        assert!(result.is_ok());
        
        let entries = result.unwrap();
        assert_eq!(entries.len(), 2);
        
        assert_eq!(entries[0].title, "Game One Title");
        assert_eq!(entries[0].url, "https://fitgirl-repacks.site/game-one/");
        assert_eq!(entries[0].image_url, Some("https://example.com/image1.jpg".to_string()));
    }
}

