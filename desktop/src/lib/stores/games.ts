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

// Track active filter type to prevent conflicts
export const activeFilterType = writable<'none' | 'category' | 'time' | 'size' | 'status'>('none');

// Active size filter state
export const activeSizeFilter = writable<string>('');

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

// Helper function to convert time filter names to days
function getTimeFilterDays(timeFilter: string): number {
  switch (timeFilter) {
    case 'Today':
      return 1;
    case 'This Week':
      return 7;
    case 'This Month':
      return 30;
    default:
      return 0;
  }
}

// Apply time filter
export async function applyTimeFilter(timeFilter: string) {
  const daysAgo = getTimeFilterDays(timeFilter);
  if (daysAgo > 0) {
    activeFilterType.set('time');
    await loadGamesByDateRange(daysAgo);
    // When filtering by time, restore all categories
    await loadCategories();
  }
}

// Helper function to convert size filter names to MB ranges
function getSizeFilterRange(sizeFilter: string): { minSize?: number; maxSize?: number } {
  switch (sizeFilter) {
    case '< 1 GB':
      return { maxSize: 1024 }; // 1 GB in MB
    case '1-10 GB':
      return { minSize: 1024, maxSize: 10240 }; // 1-10 GB in MB
    case '10-25 GB':
      return { minSize: 10240, maxSize: 25600 }; // 10-25 GB in MB
    case '25-40 GB':
      return { minSize: 25600, maxSize: 40960 }; // 25-40 GB in MB
    case '40-60 GB':
      return { minSize: 40960, maxSize: 61440 }; // 40-60 GB in MB
    case '> 60 GB':
      return { minSize: 61440 }; // 60+ GB in MB
    default:
      return {};
  }
}

// Apply size filter (works with categories)
export async function applySizeFilter(sizeFilter: string) {
  activeSizeFilter.set(sizeFilter);
  const { minSize, maxSize } = getSizeFilterRange(sizeFilter);

  // Get current selected categories
  let currentSelected: CategoryWithCount[] = [];
  selectedCategories.subscribe((s) => (currentSelected = s))();

  if (currentSelected.length > 0) {
    // Combine category + size filters
    activeFilterType.set('category'); // Keep category as primary filter type
    const categoryIds = currentSelected.map((c) => c.id);
    await loadGamesByCategoriesAndSize(categoryIds, minSize, maxSize);
  } else {
    // Size filter only
    activeFilterType.set('size');
    await loadGamesBySize(minSize, maxSize);
    await loadCategories(); // Show all categories when no categories selected
  }
}

// Clear size filter
export function clearSizeFilter() {
  activeSizeFilter.set('');
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

// Load games by date range
export async function loadGamesByDateRange(daysAgo: number, limit: number = 1000, offset: number = 0) {
  isLoading.set(true);
  try {
    const result = await invoke<Game[]>('get_games_by_date_range', {
      daysAgo,
      limit,
      offset,
    });
    games.set(result);
    if (result.length > 0) {
      await selectGame(0);
    }
  } catch (error) {
    console.error('Failed to load games by date range:', error);
  } finally {
    isLoading.set(false);
  }
}

// Load games by size range
export async function loadGamesBySize(minSize?: number, maxSize?: number, limit: number = 1000, offset: number = 0) {
  isLoading.set(true);
  try {
    const result = await invoke<Game[]>('get_games_by_size_range', {
      minSize: minSize || null,
      maxSize: maxSize || null,
      limit,
      offset,
    });
    games.set(result);
    if (result.length > 0) {
      await selectGame(0);
    }
  } catch (error) {
    console.error('Failed to load games by size range:', error);
  } finally {
    isLoading.set(false);
  }
}

// Load games by categories AND size (combined filters)
export async function loadGamesByCategoriesAndSize(
  categoryIds: number[],
  minSize?: number,
  maxSize?: number,
  limit: number = 1000,
  offset: number = 0,
) {
  isLoading.set(true);
  try {
    const result = await invoke<Game[]>('get_games_by_categories_and_size', {
      categoryIds,
      minSize: minSize || null,
      maxSize: maxSize || null,
      limit,
      offset,
    });
    games.set(result);
    if (result.length > 0) {
      await selectGame(0);
    }
  } catch (error) {
    console.error('Failed to load games by categories and size:', error);
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

// Clear all filters and restore default state
export async function clearAllFilters() {
  activeFilterType.set('none');
  clearCategorySelection();
  clearSizeFilter();
  await Promise.all([loadGames(), loadCategories()]);
}

// Apply category filters (with faceted filtering and size combination)
export async function applyCategoryFilters() {
  let currentSelected: CategoryWithCount[] = [];
  selectedCategories.subscribe((s) => (currentSelected = s))();

  let currentSizeFilter: string = '';
  activeSizeFilter.subscribe((s) => (currentSizeFilter = s))();

  if (currentSelected.length === 0) {
    // No categories selected
    let currentActiveFilter: string = 'none';
    activeFilterType.subscribe((f) => (currentActiveFilter = f))();

    if (currentActiveFilter === 'category' || currentActiveFilter === 'none') {
      // Check if there's a size filter to maintain
      if (currentSizeFilter) {
        await applySizeFilter(currentSizeFilter);
      } else {
        activeFilterType.set('none');
        await Promise.all([loadGames(), loadCategories()]);
      }
    }
  } else {
    // Categories selected - check if we need to combine with size filter
    activeFilterType.set('category');
    const categoryIds = currentSelected.map((c) => c.id);

    if (currentSizeFilter) {
      // Combine categories + size filter
      const { minSize, maxSize } = getSizeFilterRange(currentSizeFilter);
      await Promise.all([
        loadGamesByCategoriesAndSize(categoryIds, minSize, maxSize),
        loadFilteredCategories(categoryIds),
      ]);
    } else {
      // Categories only
      await Promise.all([loadGamesByMultipleCategories(categoryIds), loadFilteredCategories(categoryIds)]);
    }
  }
}

// Search games
export async function searchGames(query: string, limit: number = 100) {
  // Clear selected categories when searching
  clearCategorySelection();

  if (!query.trim()) {
    // If search is cleared, restore normal state (all games and categories)
    activeFilterType.set('none');
    await Promise.all([loadGames(limit), loadCategories()]);
    return;
  }

  activeFilterType.set('none'); // Search overrides all filters
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
