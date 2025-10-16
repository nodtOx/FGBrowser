// FitGirl Repacks specific crawler implementation

use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use scraper::{Html, Selector};
use std::time::Duration;
use tokio::time::sleep;

use super::extractors::Extractors;
use super::models::{GameRepack, PopularRepackEntry};
use super::site_crawler::SiteCrawler;

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
        // Default FitGirl blacklist patterns
        vec![
            "upcoming-repacks".to_string(),
            "updates-digest".to_string(),
        ]
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

    fn extract_repack_from_single_page(&self, article: &scraper::ElementRef, url: &str) -> Option<GameRepack> {
        // On single game pages, the title is in h1.entry-title WITHOUT a link
        let title_selector = Selector::parse("h1.entry-title").unwrap();
        let title_elem = article.select(&title_selector).next()?;
        
        let title = title_elem.text().collect::<String>().trim().to_string();
        
        if title.is_empty() {
            return None;
        }

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
}

#[async_trait]
impl SiteCrawler for FitGirlCrawler {
    fn site_id(&self) -> &str {
        "fitgirl"
    }

    fn site_name(&self) -> &str {
        "FitGirl Repacks"
    }

    fn base_url(&self) -> &str {
        &self.base_url
    }

    async fn crawl_page(&self, page_num: u32) -> Result<Vec<GameRepack>> {
        let url = if page_num == 1 {
            self.base_url.clone()
        } else {
            format!("{}/page/{}/", self.base_url, page_num)
        };

        let html = self.fetch_page(&url).await?;
        let document = Html::parse_document(&html);

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

    async fn crawl_single_game(&self, url: &str) -> Result<Option<GameRepack>> {
        let html = self.fetch_page(url).await?;
        let document = Html::parse_document(&html);
        
        // Find the article element
        let article_selector = Selector::parse("article").unwrap();
        if let Some(article) = document.select(&article_selector).next() {
            Ok(self.extract_repack_from_single_page(&article, url))
        } else {
            Ok(None)
        }
    }

    async fn fetch_popular_repacks(&self, period: &str) -> Result<Vec<PopularRepackEntry>> {
        let url = match period {
            "month" => format!("{}/popular-repacks/", self.base_url),
            "year" => format!("{}/popular-repacks-of-the-year/", self.base_url),
            "award" => format!("{}/games-with-my-personal-pink-paw-award/", self.base_url),
            _ => return Ok(Vec::new()),
        };

        let html = self.fetch_page(&url).await?;
        
        // For Pink Paw Award page, use special parser
        if period == "award" {
            return super::popular::PopularRepacks::parse_pink_paw_award_html(&html);
        }
        
        // For month/year, use the grid-based parser
        let document = Html::parse_document(&html);

        // Select only the main content area to avoid sidebar items
        let content_selector = Selector::parse("article .entry-content").unwrap();
        let grid_selector = Selector::parse("div.widget-grid-view-image").unwrap();

        let mut entries = Vec::new();

        if let Some(content_element) = document.select(&content_selector).next() {
            for element in content_element.select(&grid_selector) {
                if let Some(link) = element.select(&Selector::parse("a").unwrap()).next() {
                    if let Some(href) = link.value().attr("href") {
                        let title = link.value().attr("title").unwrap_or("").to_string();
                        
                        let image_url = element
                            .select(&Selector::parse("img").unwrap())
                            .next()
                            .and_then(|img| img.value().attr("src"))
                            .map(|s| s.to_string());

                        entries.push(PopularRepackEntry {
                            url: href.to_string(),
                            title,
                            image_url,
                        });
                    }
                }
            }
        }

        Ok(entries)
    }

    fn get_blacklist(&self) -> Vec<String> {
        self.blacklist.clone()
    }
}

