#[cfg(test)]
mod tests {
    use crate::crawler::clean_game_title;

    // ========================================
    // clean_game_title() tests
    // Pure function with complex regex logic
    // EXCELLENT candidate for comprehensive testing!
    // ========================================

    #[test]
    fn test_clean_version_numbers() {
        assert_eq!(
            clean_game_title("Cyberpunk 2077 – v2.13"),
            "Cyberpunk 2077"
        );
        assert_eq!(
            clean_game_title("Elden Ring – v1.12.3"),
            "Elden Ring"
        );
    }

    #[test]
    fn test_clean_build_numbers() {
        assert_eq!(
            clean_game_title("Game Name – Build 12345"),
            "Game Name"
        );
        assert_eq!(
            clean_game_title("Another Game, Build 8796429"),
            "Another Game"
        );
    }

    #[test]
    fn test_clean_revision_numbers() {
        assert_eq!(
            clean_game_title("Game – r34045"),
            "Game"
        );
        assert_eq!(
            clean_game_title("Game.r49909"),
            "Game"
        );
        assert_eq!(
            clean_game_title("Game r12345"),
            "Game"
        );
    }

    #[test]
    fn test_clean_dates() {
        assert_eq!(
            clean_game_title("Game – 26.09.2025"),
            "Game"
        );
        assert_eq!(
            clean_game_title("Game – 20250831_2044-123"),
            "Game"
        );
    }

    #[test]
    fn test_clean_editions() {
        assert_eq!(
            clean_game_title("Cyberpunk 2077 – Deluxe Edition"),
            "Cyberpunk 2077"
        );
        assert_eq!(
            clean_game_title("Elden Ring: Premium Edition"),
            "Elden Ring"
        );
        assert_eq!(
            clean_game_title("Game, Ultimate Edition"),
            "Game"
        );
        assert_eq!(
            clean_game_title("Game – Gold Edition"),
            "Game"
        );
    }

    #[test]
    fn test_clean_platforms() {
        assert_eq!(
            clean_game_title("Cyberpunk 2077 – GOG"),
            "Cyberpunk 2077"
        );
        assert_eq!(
            clean_game_title("Elden Ring – Steam"),
            "Elden Ring"
        );
        assert_eq!(
            clean_game_title("Game GOG"),
            "Game"
        );
    }

    #[test]
    fn test_clean_parenthetical() {
        assert_eq!(
            clean_game_title("Cyberpunk 2077 (Denuvoless)"),
            "Cyberpunk 2077"
        );
        assert_eq!(
            clean_game_title("Game (Campaign Only)"),
            "Game"
        );
    }

    #[test]
    fn test_clean_dlc_content() {
        assert_eq!(
            clean_game_title("Game + 5 DLCs + Bonus Content"),
            "Game"
        );
        assert_eq!(
            clean_game_title("Game, v1.5 + All DLC"),
            "Game"
        );
    }

    #[test]
    fn test_clean_complex_title() {
        // Real-world example with multiple patterns
        assert_eq!(
            clean_game_title("Cyberpunk 2077: Ultimate Edition – v2.13 + 5 DLCs (GOG)"),
            "Cyberpunk 2077"
        );
        
        assert_eq!(
            clean_game_title("Elden Ring – Deluxe Edition – v1.12.3 – Build 12345"),
            "Elden Ring"
        );
    }

    #[test]
    fn test_clean_slash_version() {
        assert_eq!(
            clean_game_title("Game v20220613/Build 8796429"),
            "Game v20220613"
        );
    }

    #[test]
    fn test_clean_repack_indicators() {
        assert_eq!(
            clean_game_title("Game – Monkey Repack"),
            "Game"
        );
        assert_eq!(
            clean_game_title("Game – BetterRepack R18"),
            "Game"
        );
    }

    #[test]
    fn test_clean_multiple_spaces() {
        assert_eq!(
            clean_game_title("Game    With    Spaces"),
            "Game With Spaces"
        );
    }

    #[test]
    fn test_clean_trailing_punctuation() {
        assert_eq!(
            clean_game_title("Game Name - "),
            "Game Name"
        );
        assert_eq!(
            clean_game_title("Game Name : "),
            "Game Name"
        );
        assert_eq!(
            clean_game_title("Game Name , "),
            "Game Name"
        );
    }

    #[test]
    fn test_clean_specific_editions() {
        assert_eq!(
            clean_game_title("Game – Anniversary Edition"),
            "Game"
        );
        // Note: Director's Cut requires the apostrophe to be in the regex pattern
        // Current implementation may not catch all apostrophe variations
        assert_eq!(
            clean_game_title("Game – Game of the Year Edition"),
            "Game"
        );
        assert_eq!(
            clean_game_title("Game – Definitive Edition"),
            "Game"
        );
    }

    #[test]
    fn test_clean_simple_name_unchanged() {
        // Simple names should pass through unchanged
        assert_eq!(
            clean_game_title("Cyberpunk 2077"),
            "Cyberpunk 2077"
        );
        assert_eq!(
            clean_game_title("Elden Ring"),
            "Elden Ring"
        );
    }

    #[test]
    fn test_clean_hotfix() {
        assert_eq!(
            clean_game_title("Game – Hotfix 5"),
            "Game"
        );
    }

    #[test]
    fn test_clean_data_pack() {
        assert_eq!(
            clean_game_title("Game Data Pack 1.5"),
            "Game"
        );
    }

    #[test]
    fn test_real_world_examples() {
        // Real examples from FitGirl repacks
        assert_eq!(
            clean_game_title("S.T.A.L.K.E.R. 2: Heart of Chornobyl – v1.1.2 + Bonus Content"),
            "S.T.A.L.K.E.R. 2: Heart of Chornobyl"
        );

        assert_eq!(
            clean_game_title("Assassin's Creed IV: Black Flag – Jackdaw Edition"),
            "Assassin's Creed IV: Black Flag"
        );

        assert_eq!(
            clean_game_title("Red Dead Redemption 2: Ultimate Edition – v1491.50 + Bonus Content (Steam)"),
            "Red Dead Redemption 2"
        );
    }
}

