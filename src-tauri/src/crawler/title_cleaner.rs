use regex::Regex;

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

