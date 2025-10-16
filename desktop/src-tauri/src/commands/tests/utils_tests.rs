#[cfg(test)]
mod tests {
    use crate::commands::utils::{parse_size_to_mb, extract_info_hash, is_popular_blacklisted};

    // ========================================
    // parse_size_to_mb() tests
    // Pure function - easiest to test!
    // ========================================

    #[test]
    fn test_parse_size_mb() {
        assert_eq!(parse_size_to_mb(&Some("916 MB".to_string())), Some(916));
        assert_eq!(parse_size_to_mb(&Some("500 MB".to_string())), Some(500));
    }

    #[test]
    fn test_parse_size_gb() {
        assert_eq!(parse_size_to_mb(&Some("1.1 GB".to_string())), Some(1126));
        assert_eq!(parse_size_to_mb(&Some("2 GB".to_string())), Some(2048));
        assert_eq!(parse_size_to_mb(&Some("15.9 GB".to_string())), Some(16281));
    }

    #[test]
    fn test_parse_size_with_prefix() {
        assert_eq!(parse_size_to_mb(&Some("from 15.9 GB".to_string())), Some(16281));
        assert_eq!(parse_size_to_mb(&Some("~5 GB".to_string())), Some(5120));
    }

    #[test]
    fn test_parse_size_with_suffix() {
        assert_eq!(
            parse_size_to_mb(&Some("15.9 GB [Selective".to_string())),
            Some(16281)
        );
        assert_eq!(
            parse_size_to_mb(&Some("10 GB [Download".to_string())),
            Some(10240)
        );
    }

    #[test]
    fn test_parse_size_with_slash_different_units() {
        // "916 MB/1.1 GB" should take the smaller value (916 MB)
        assert_eq!(
            parse_size_to_mb(&Some("916 MB/1.1 GB".to_string())),
            Some(916)
        );
        assert_eq!(
            parse_size_to_mb(&Some("500 MB/2 GB".to_string())),
            Some(500)
        );
    }

    #[test]
    fn test_parse_size_with_slash_same_unit() {
        // "1.1/1.3 GB" should take the smaller value (1.1 GB)
        assert_eq!(
            parse_size_to_mb(&Some("1.1/1.3 GB".to_string())),
            Some(1126)
        );
        assert_eq!(
            parse_size_to_mb(&Some("5/10 GB".to_string())),
            Some(5120)
        );
    }

    #[test]
    fn test_parse_size_invalid() {
        assert_eq!(parse_size_to_mb(&Some("invalid".to_string())), None);
        assert_eq!(parse_size_to_mb(&Some("".to_string())), None);
        assert_eq!(parse_size_to_mb(&None), None);
    }

    #[test]
    fn test_parse_size_tb() {
        assert_eq!(parse_size_to_mb(&Some("1 TB".to_string())), Some(1048576));
        assert_eq!(parse_size_to_mb(&Some("0.5 TB".to_string())), Some(524288));
    }

    #[test]
    fn test_parse_size_edge_cases() {
        // Real-world examples from FitGirl repacks
        assert_eq!(
            parse_size_to_mb(&Some("from 35 GB".to_string())),
            Some(35840)
        );
        assert_eq!(
            parse_size_to_mb(&Some("~100 GB [Selective Download]".to_string())),
            Some(102400)
        );
    }

    // ========================================
    // extract_info_hash() tests
    // Pure function - very easy to test!
    // ========================================

    #[test]
    fn test_extract_info_hash_valid() {
        let magnet = "magnet:?xt=urn:btih:ABCDEF1234567890&dn=Test";
        assert_eq!(
            extract_info_hash(magnet),
            Some("ABCDEF1234567890".to_string())
        );
    }

    #[test]
    fn test_extract_info_hash_with_multiple_params() {
        let magnet = "magnet:?xt=urn:btih:1234567890ABCDEF&dn=TestGame&tr=http://tracker.com";
        assert_eq!(
            extract_info_hash(magnet),
            Some("1234567890ABCDEF".to_string())
        );
    }

    #[test]
    fn test_extract_info_hash_invalid() {
        assert_eq!(extract_info_hash("invalid"), None);
        assert_eq!(extract_info_hash("http://example.com"), None);
        assert_eq!(extract_info_hash(""), None);
    }

    #[test]
    fn test_extract_info_hash_no_btih() {
        let magnet = "magnet:?dn=Test&tr=http://tracker.com";
        assert_eq!(extract_info_hash(magnet), None);
    }

    // ========================================
    // is_popular_blacklisted() tests
    // Semi-pure function (depends on static data)
    // ========================================

    #[test]
    fn test_blacklist_contains() {
        assert!(is_popular_blacklisted(
            "https://fitgirl-repacks.site/the-genesis-order/"
        ));
        assert!(is_popular_blacklisted(
            "https://fitgirl-repacks.site/honey-select-2-libido/"
        ));
    }

    #[test]
    fn test_blacklist_case_insensitive() {
        assert!(is_popular_blacklisted(
            "https://fitgirl-repacks.site/THE-GENESIS-ORDER/"
        ));
        assert!(is_popular_blacklisted(
            "https://fitgirl-repacks.site/Honey-Select-2-Libido/"
        ));
    }

    #[test]
    fn test_blacklist_not_contained() {
        assert!(!is_popular_blacklisted(
            "https://fitgirl-repacks.site/cyberpunk-2077/"
        ));
        assert!(!is_popular_blacklisted(
            "https://fitgirl-repacks.site/elden-ring/"
        ));
    }

    #[test]
    fn test_blacklist_partial_match() {
        // Should match if blacklisted string is contained anywhere in URL
        assert!(is_popular_blacklisted(
            "https://fitgirl-repacks.site/some-prefix-the-genesis-order-suffix/"
        ));
    }
}

