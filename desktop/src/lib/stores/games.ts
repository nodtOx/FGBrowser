import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export interface Game {
  id: number;
  title: string;
  genres_tags: string | null;
  company: string | null;
  languages: string | null;
  original_size: string | null;
  repack_size: string | null;
  url: string;
  date: string | null;
}

export interface MagnetLink {
  id: number;
  repack_id: number;
  source: string;
  magnet: string;
}

export interface GameDetails extends Game {
  magnet_links: MagnetLink[];
}

export const games = writable<Game[]>([]);
export const selectedGame = writable<GameDetails | null>(null);
export const selectedIndex = writable<number>(0);
export const searchQuery = writable<string>('');
export const isLoading = writable<boolean>(false);

// Load games from database
export async function loadGames(limit: number = 100, offset: number = 0) {
  isLoading.set(true);
  try {
    const result = await invoke<Game[]>('get_all_games', { limit, offset });
    games.set(result);
    if (result.length > 0) {
      await selectGame(0);
    }
  } catch (error) {
    console.error('Failed to load games:', error);
  } finally {
    isLoading.set(false);
  }
}

// Search games
export async function searchGames(query: string, limit: number = 100) {
  if (!query.trim()) {
    return loadGames(limit);
  }

  isLoading.set(true);
  try {
    const result = await invoke<Game[]>('search_games', { query, limit });
    games.set(result);
    if (result.length > 0) {
      await selectGame(0);
    }
  } catch (error) {
    console.error('Failed to search games:', error);
  } finally {
    isLoading.set(false);
  }
}

// Select a game by index
export async function selectGame(index: number) {
  let currentGames: Game[] = [];
  games.subscribe((g) => (currentGames = g))();

  if (index < 0 || index >= currentGames.length) return;

  selectedIndex.set(index);
  const game = currentGames[index];

  // Fetch details for magnet links
  try {
    const details = await invoke<GameDetails>('get_game_details', { gameId: game.id });
    selectedGame.set(details);
  } catch (error) {
    console.error('Failed to get game details:', error);
  }
}

// Navigate selection
export async function moveSelection(direction: 'up' | 'down') {
  let currentIndex = 0;
  let currentGames: Game[] = [];

  selectedIndex.subscribe((i) => (currentIndex = i))();
  games.subscribe((g) => (currentGames = g))();

  if (direction === 'up' && currentIndex > 0) {
    await selectGame(currentIndex - 1);
  } else if (direction === 'down' && currentIndex < currentGames.length - 1) {
    await selectGame(currentIndex + 1);
  }
}

// Open magnet link
export async function openMagnetLink(magnet: string) {
  try {
    await invoke('open_magnet_link', { magnet });
  } catch (error) {
    console.error('Failed to open magnet link:', error);
  }
}

// Copy magnet link to clipboard
export async function copyMagnetLink(magnet: string) {
  try {
    await invoke('copy_to_clipboard', { text: magnet });
  } catch (error) {
    console.error('Failed to copy to clipboard:', error);
  }
}
