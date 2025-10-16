// Command modules organized by responsibility
pub mod utils;
pub mod game_commands;
pub mod crawler_commands;
pub mod popular_commands;
pub mod download_commands;
pub mod settings_commands;
pub mod image_commands;
pub mod system_commands;

// Re-export AppState for convenience
pub use utils::AppState;

// Re-export all command functions for use in lib.rs
// This allows lib.rs to import commands the same way as before

// Game commands
pub use game_commands::{
    search_games,
    get_all_games,
    get_game_details,
    get_database_stats,
    get_categories_with_counts,
    get_categories_for_filtered_games,
    get_categories_for_time_filtered_games,
    get_categories_for_size_filtered_games,
    get_categories_for_size_and_time_filtered_games,
    get_categories_for_search,
    get_games_by_date_range,
    get_games_by_size_range,
    get_games_by_categories_and_size,
    get_games_by_categories_and_time,
    get_games_by_size_and_time,
    get_games_by_categories_size_and_time,
    get_games_by_category,
    get_games_by_multiple_categories,
    clear_category_cache,
    is_database_empty,
};

// Crawler commands
pub use crawler_commands::{
    start_crawler,
    update_database,
    save_repacks_to_db,  // Also exported for CLI use
    CrawlProgress,
};

// Popular repacks commands
pub use popular_commands::{
    fetch_popular_repacks,
    parse_popular_repacks_from_file,
    get_popular_repacks,
    get_popular_repacks_with_games,
    get_unseen_popular_count,
    get_total_unseen_popular_count,
    mark_popular_as_viewed,
    update_popular_repack_links,
    crawl_popular_games,
    crawl_single_popular_game,
};

// Download management commands
pub use download_commands::{
    get_downloads,
    add_download,
    pause_download,
    resume_download,
    remove_download,
    set_speed_limits,
};

// Settings commands
pub use settings_commands::{
    get_settings,
    save_settings,
    reset_database,
    download_database,
    check_database_exists,
    DownloadProgress,
};

// Image cache commands
pub use image_commands::{
    get_cached_image,
    check_image_cached,
    cache_image_background,
    clear_image_cache,
    get_image_cache_size,
};

// System utilities
pub use system_commands::{
    get_app_constants,
    open_magnet_link,
    copy_to_clipboard,
    get_disk_info,
    select_download_folder,
    open_download_folder,
    DiskInfo,
};

