use super::database_service::SqliteDatabaseService;
use std::sync::Arc;

/// Shared application state with injected database service
/// Using concrete type for simplicity while maintaining SOLID architecture
pub struct AppState {
    pub db_service: Arc<SqliteDatabaseService>,
}

/// Parse size string to MB (integer)
/// Handles patterns like:
/// - "916 MB" -> Some(916)
/// - "1.1 GB" -> Some(1100) 
/// - "from 15.9 GB [Selective" -> Some(15900)
/// - "916 MB/1.1 GB" -> Some(916) (takes smaller value)
/// - "1.1/1.3 GB" -> Some(1100) (takes smaller value)
pub fn parse_size_to_mb(size_str: &Option<String>) -> Option<i64> {
    let original = size_str.as_ref()?.trim();
    
    // Remove common prefixes and suffixes
    let size_str = original
        .strip_prefix("from ")
        .unwrap_or(original);
    let size_str = size_str
        .strip_prefix("~")
        .unwrap_or(size_str);
    
    // Remove trailing brackets and content
    let size_str = size_str.split('[').next().unwrap_or(size_str).trim();
    
    // Handle different slash patterns
    if size_str.contains('/') {
        let parts: Vec<&str> = size_str.split('/').collect();
        if parts.len() >= 2 {
            // Handle cases like "1.1/1.3 GB" where unit is only at the end
            let first_part = parts[0].trim();
            let second_part = parts[1].trim();
            
            // Check if first part has no unit but second part does
            let first_size = if first_part.split_whitespace().count() == 1 && second_part.split_whitespace().count() >= 2 {
                // Extract unit from second part and apply to first
                let second_parts: Vec<&str> = second_part.split_whitespace().collect();
                if second_parts.len() >= 2 {
                    let unit = second_parts[1];
                    parse_single_size(&format!("{} {}", first_part, unit))
                } else {
                    None
                }
            } else {
                parse_single_size(first_part)
            };
            
            let second_size = parse_single_size(second_part);
            
            match (first_size, second_size) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                _ => {
                    println!("❌ SIZE PARSE FAILED: '{}'", original);
                    None
                }
            }
        } else {
            println!("❌ SIZE PARSE FAILED: '{}'", original);
            None
        }
    } else {
        // Single size
        let result = parse_single_size(size_str);
        if result.is_none() {
            println!("❌ SIZE PARSE FAILED: '{}'", original);
        }
        result
    }
}

/// Parse a single size string like "1.4 GB" or "916 MB"
fn parse_single_size(size_str: &str) -> Option<i64> {
    let parts: Vec<&str> = size_str.split_whitespace().collect();
    
    if parts.len() < 2 {
        return None;
    }
    
    let number_str = parts[0];
    let unit_str = parts[1];
    
    // Extract just the unit part (remove any trailing characters)
    let unit = if unit_str.starts_with("MB") {
        "MB"
    } else if unit_str.starts_with("GB") {
        "GB" 
    } else if unit_str.starts_with("TB") {
        "TB"
    } else {
        return None;
    };
    
    // Parse the number
    let number: f64 = number_str.parse().ok()?;
    
    // Convert to MB
    let mb = match unit {
        "MB" => number,
        "GB" => number * 1024.0,
        "TB" => number * 1024.0 * 1024.0,
        _ => return None,
    };
    
    Some(mb as i64)
}

/// Helper function to extract info hash from magnet link
pub fn extract_info_hash(magnet: &str) -> Option<String> {
    // magnet:?xt=urn:btih:INFO_HASH&...
    if !magnet.starts_with("magnet:?") {
        return None;
    }
    
    for part in magnet.split('&') {
        if let Some(xt) = part.strip_prefix("xt=urn:btih:") {
            return Some(xt.to_string());
        }
        if part.contains("xt=urn:btih:") {
            if let Some(hash) = part.split("xt=urn:btih:").nth(1) {
                return Some(hash.to_string());
            }
        }
    }
    
    None
}

/// Helper functions for popular games blacklist
pub fn is_popular_blacklisted(url: &str) -> bool {
    let url_lower = url.to_lowercase();
    let popular_blacklist = load_popular_blacklist();
    
    popular_blacklist.iter().any(|blacklisted| {
        url_lower.contains(blacklisted)
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
        "lost-chapter".to_string(),
        "koikatsu".to_string(),
        "av-director".to_string(),
    ]
}
