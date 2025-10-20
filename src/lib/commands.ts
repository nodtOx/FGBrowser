/**
 * Tauri Command Constants
 *
 * Organized by Rust source file for type-safe invoke() calls.
 * Usage: invoke(commands.game.get_all_games, { limit, offset })
 */

export const commands = {
  game: {
    search_games: 'search_games',
    get_all_games: 'get_all_games',
    get_game_details: 'get_game_details',
    get_database_stats: 'get_database_stats',
    get_categories_with_counts: 'get_categories_with_counts',
    get_categories_for_filtered_games: 'get_categories_for_filtered_games',
    get_categories_for_time_filtered_games: 'get_categories_for_time_filtered_games',
    get_categories_for_size_filtered_games: 'get_categories_for_size_filtered_games',
    get_categories_for_size_and_time_filtered_games: 'get_categories_for_size_and_time_filtered_games',
    get_categories_for_search: 'get_categories_for_search',
    get_games_by_category: 'get_games_by_category',
    get_games_by_date_range: 'get_games_by_date_range',
    get_games_by_size_range: 'get_games_by_size_range',
    get_games_by_categories_and_size: 'get_games_by_categories_and_size',
    get_games_by_categories_and_time: 'get_games_by_categories_and_time',
    get_games_by_size_and_time: 'get_games_by_size_and_time',
    get_games_by_categories_size_and_time: 'get_games_by_categories_size_and_time',
    get_games_by_multiple_categories: 'get_games_by_multiple_categories',
    clear_category_cache: 'clear_category_cache',
    is_database_empty: 'is_database_empty',
    mark_all_games_as_seen: 'mark_all_games_as_seen',
    mark_game_as_seen: 'mark_game_as_seen',
    get_new_games_count: 'get_new_games_count',
  },

  popular: {
    fetch_popular_repacks: 'fetch_popular_repacks',
    parse_popular_repacks_from_file: 'parse_popular_repacks_from_file',
    get_popular_repacks: 'get_popular_repacks',
    get_popular_repacks_with_games: 'get_popular_repacks_with_games',
    update_popular_repack_links: 'update_popular_repack_links',
    get_unseen_popular_count: 'get_unseen_popular_count',
    get_total_unseen_popular_count: 'get_total_unseen_popular_count',
    mark_popular_as_viewed: 'mark_popular_as_viewed',
    crawl_popular_games: 'crawl_popular_games',
    crawl_single_popular_game: 'crawl_single_popular_game',
  },

  crawler: {
    start_crawler: 'start_crawler',
    update_database: 'update_database',
  },

  settings: {
    get_settings: 'get_settings',
    save_settings: 'save_settings',
    reset_database: 'reset_database',
    download_database: 'download_database',
    check_database_exists: 'check_database_exists',
  },

  system: {
    get_app_constants: 'get_app_constants',
    open_magnet_link: 'open_magnet_link',
    copy_to_clipboard: 'copy_to_clipboard',
    get_disk_info: 'get_disk_info',
    select_download_folder: 'select_download_folder',
    open_download_folder: 'open_download_folder',
  },

  download: {
    get_downloads: 'get_downloads',
    add_download: 'add_download',
    pause_download: 'pause_download',
    resume_download: 'resume_download',
    remove_download: 'remove_download',
    set_speed_limits: 'set_speed_limits',
  },

  image: {
    get_cached_image: 'get_cached_image',
    check_image_cached: 'check_image_cached',
    cache_image_background: 'cache_image_background',
    clear_image_cache: 'clear_image_cache',
    get_image_cache_size: 'get_image_cache_size',
  },

  telemetry: {
    track_app_launch: 'track_app_launch',
    track_feature_usage: 'track_feature_usage',
    track_crawler_run: 'track_crawler_run',
    track_error: 'track_error',
    is_telemetry_enabled: 'is_telemetry_enabled',
    test_sentry_integration: 'test_sentry_integration',
    get_telemetry_user_id: 'get_telemetry_user_id',
    track_search: 'track_search',
  },
} as const;

// Type for all available commands
export type CommandCategory = keyof typeof commands;
export type GameCommand = keyof typeof commands.game;
export type PopularCommand = keyof typeof commands.popular;
export type CrawlerCommand = keyof typeof commands.crawler;
export type SettingsCommand = keyof typeof commands.settings;
export type SystemCommand = keyof typeof commands.system;
export type DownloadCommand = keyof typeof commands.download;
export type ImageCommand = keyof typeof commands.image;
export type TelemetryCommand = keyof typeof commands.telemetry;
