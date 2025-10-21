import { invoke } from '@tauri-apps/api/core';

export interface Screenshot {
  url: string;
}

export interface Video {
  url: string;
}

export interface MediaResult {
  screenshots: Screenshot[];
  videos: Video[];
}

/**
 * Fetch screenshots and videos for a game (lazy loading)
 * First checks database, if not found, fetches from FitGirl page and saves to DB
 * @param gameId - The game ID
 * @returns Promise with screenshots and videos
 */
export async function fetchGameMedia(gameId: number): Promise<MediaResult> {
  return await invoke<MediaResult>('fetch_game_media', { gameId });
}

/**
 * Get screenshots and videos from database only (no fetching)
 * @param gameId - The game ID
 * @returns Promise with screenshots and videos from database
 */
export async function getGameMedia(gameId: number): Promise<MediaResult> {
  return await invoke<MediaResult>('get_game_media', { gameId });
}
