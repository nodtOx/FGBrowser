// Application-wide constants
// These constants are shared between Rust backend and TypeScript frontend
// The source of truth is in src-tauri/src/constants.rs
// Frontend can load them via the get_app_constants() Tauri command

import { invoke } from '@tauri-apps/api/core';

// Default constants (used as fallback before backend loads)
// These match the values in src-tauri/src/constants.rs
export const LOAD_ALL_GAMES = 999999;
export const DEFAULT_OFFSET = 0;
export const POLLING_INTERVAL_MS = 500;
export const SEARCH_DEBOUNCE_MS = 300;
export const ITEM_HEIGHT = 30;
export const OVERSCAN = 5;
// Use high limits to effectively fetch all games (no artificial restrictions)
export const POPULAR_FETCH_LIMIT = 9999;
export const POPULAR_REFRESH_INTERVAL_MS = 3000;
export const DISK_INFO_REFRESH_INTERVAL_MS = 30000;

// Interface matching Rust AppConstants struct
export interface AppConstants {
  load_all_games: number;
  default_offset: number;
  polling_interval_ms: number;
  search_debounce_ms: number;
  item_height: number;
  overscan: number;
  popular_fetch_limit: number;
  popular_refresh_interval_ms: number;
  disk_info_refresh_interval_ms: number;
  database_url: string;
}

// Load constants from backend (optional - use for runtime updates)
export async function loadAppConstants(): Promise<AppConstants> {
  return await invoke<AppConstants>('get_app_constants');
}
