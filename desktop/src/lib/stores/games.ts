import { invoke } from '@tauri-apps/api/core';
import { writable } from 'svelte/store';

export interface Game {
  id: number;
  title: string;
  genres_tags: string | null;
  company: string | null;
  languages: string | null;
  original_size: string | null;
  repack_size: string | null; // Keep for backwards compatibility
  size: number | null; // Size in MB (parsed from repack_size)
  url: string;
  date: string | null;
}

export interface MagnetLink {
  id: number;
  repack_id: number;
  source: string;
  magnet: string;
}

export interface Category {
  id: number;
  name: string;
}

export interface CategoryWithCount {
  id: number;
  name: string;
  game_count: number;
}

export interface GameDetails extends Game {
  magnet_links: MagnetLink[];
  categories: Category[];
}

export const games = writable<Game[]>([]);
export const selectedGame = writable<GameDetails | null>(null);
export const selectedIndex = writable<number>(0);
export const searchQuery = writable<string>('');
export const isLoading = writable<boolean>(false);
export const categories = writable<CategoryWithCount[]>([]);
export const selectedCategories = writable<CategoryWithCount[]>([]);

// Optimization: Debouncing for category filtering
let filterDebounceTimer: number | null = null;
const FILTER_DEBOUNCE_MS = 200;

// Debounced category filtering
export function debouncedApplyCategoryFilters() {
  if (filterDebounceTimer) {
    clearTimeout(filterDebounceTimer);
  }

  filterDebounceTimer = setTimeout(() => {
    applyCategoryFilters();
    filterDebounceTimer = null;
  }, FILTER_DEBOUNCE_MS) as unknown as number;
}

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

// Load categories from database
export async function loadCategories() {
  try {
    const result = await invoke<CategoryWithCount[]>('get_categories_with_counts');
    categories.set(result);
  } catch (error) {
    console.error('Failed to load categories:', error);
  }
}

// Load categories filtered by selected categories (faceted filtering)
export async function loadFilteredCategories(selectedCategoryIds: number[]) {
  try {
    const result = await invoke<CategoryWithCount[]>('get_categories_for_filtered_games', {
      selectedCategoryIds,
    });
    categories.set(result);
  } catch (error) {
    console.error('Failed to load filtered categories:', error);
  }
}

// Load games by category
export async function loadGamesByCategory(categoryId: number, limit: number = 100, offset: number = 0) {
  isLoading.set(true);
  try {
    const result = await invoke<Game[]>('get_games_by_category', {
      categoryId,
      limit,
      offset,
    });
    games.set(result);
    if (result.length > 0) {
      await selectGame(0);
    }
  } catch (error) {
    console.error('Failed to load games by category:', error);
  } finally {
    isLoading.set(false);
  }
}

// Load games by multiple categories
export async function loadGamesByMultipleCategories(categoryIds: number[], limit: number = 1000, offset: number = 0) {
  isLoading.set(true);
  try {
    const result = await invoke<Game[]>('get_games_by_multiple_categories', {
      categoryIds,
      limit,
      offset,
    });
    games.set(result);
    if (result.length > 0) {
      await selectGame(0);
    }
  } catch (error) {
    console.error('Failed to load games by multiple categories:', error);
  } finally {
    isLoading.set(false);
  }
}

// Toggle category selection
export function toggleCategorySelection(category: CategoryWithCount) {
  selectedCategories.update((selected) => {
    const index = selected.findIndex((c) => c.id === category.id);
    if (index >= 0) {
      // Remove if already selected
      return selected.filter((c) => c.id !== category.id);
    } else {
      // Add if not selected
      return [...selected, category];
    }
  });
}

// Clear all selected categories
export function clearCategorySelection() {
  selectedCategories.set([]);
}

// Apply category filters (with faceted filtering)
export async function applyCategoryFilters() {
  let currentSelected: CategoryWithCount[] = [];
  selectedCategories.subscribe((s) => (currentSelected = s))();

  if (currentSelected.length === 0) {
    // No categories selected - load all games and all categories
    await Promise.all([loadGames(), loadCategories()]);
  } else {
    // Categories selected - filter both games and categories
    const categoryIds = currentSelected.map((c) => c.id);
    await Promise.all([loadGamesByMultipleCategories(categoryIds), loadFilteredCategories(categoryIds)]);
  }
}

// Search games
export async function searchGames(query: string, limit: number = 100) {
  // Clear selected categories when searching
  clearCategorySelection();

  if (!query.trim()) {
    // If search is cleared, restore normal state (all games and categories)
    await Promise.all([loadGames(limit), loadCategories()]);
    return;
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

// Format size from MB to human-readable string
export function formatSize(sizeInMB: number | null): string {
  if (!sizeInMB || sizeInMB <= 0) return 'N/A';

  if (sizeInMB < 1024) {
    return `${sizeInMB} MB`;
  } else if (sizeInMB < 1024 * 1024) {
    const sizeInGB = sizeInMB / 1024;
    return `${sizeInGB.toFixed(1)} GB`;
  } else {
    const sizeInTB = sizeInMB / (1024 * 1024);
    return `${sizeInTB.toFixed(1)} TB`;
  }
}
