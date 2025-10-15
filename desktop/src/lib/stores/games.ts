import { invoke } from '@tauri-apps/api/core';
import { derived, writable } from 'svelte/store';

export interface Game {
  id: number;
  title: string;
  clean_name: string | null;
  genres_tags: string | null;
  company: string | null;
  languages: string | null;
  original_size: string | null;
  repack_size: string | null; // Keep for backwards compatibility
  size: number | null; // Size in MB (parsed from repack_size)
  url: string;
  date: string | null;
  image_url: string | null;
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

// Popular games crawling state
export const isCrawlingPopular = writable<boolean>(false);
export const popularCrawlProgress = writable<{ crawled: number; total: number }>({ crawled: 0, total: 0 });

// All active filters (unified system)
export const activeTimeFilter = writable<string>('');
export const activeSizeFilter = writable<string>('');
export const activeStatusFilter = writable<string>('');

// Combined active filters for display
export const activeFilters = derived(
  [selectedCategories, activeTimeFilter, activeSizeFilter, activeStatusFilter],
  ([$categories, $time, $size, $status]) => {
    const filters: Array<{ type: string; value: string; label: string }> = [];

    // Add category filters
    $categories.forEach((cat) => {
      filters.push({ type: 'category', value: cat.name, label: cat.name });
    });

    // Add time filter
    if ($time) {
      filters.push({ type: 'time', value: $time, label: $time });
    }

    // Add size filter
    if ($size) {
      filters.push({ type: 'size', value: $size, label: $size });
    }

    // Add status filter
    if ($status) {
      filters.push({ type: 'status', value: $status, label: $status });
    }

    return filters;
  },
);

// Optimization: Debouncing for category filtering
let filterDebounceTimer: number | null = null;
const FILTER_DEBOUNCE_MS = 200;

// Debounced filter application (unified)
export function debouncedApplyFilters() {
  if (filterDebounceTimer) {
    clearTimeout(filterDebounceTimer);
  }

  filterDebounceTimer = setTimeout(() => {
    applyAllFilters();
    filterDebounceTimer = null;
  }, FILTER_DEBOUNCE_MS) as unknown as number;
}

// Legacy function for backward compatibility
export function debouncedApplyCategoryFilters() {
  debouncedApplyFilters();
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

// Apply time filter (works with other filters)
export async function applyTimeFilter(timeFilter: string) {
  activeTimeFilter.set(timeFilter);
  await applyAllFilters();
}

// Apply size filter (works with other filters)
export async function applySizeFilter(sizeFilter: string) {
  activeSizeFilter.set(sizeFilter);
  await applyAllFilters();
}

// Apply status filter (works with other filters)
export async function applyStatusFilter(statusFilter: string) {
  activeStatusFilter.set(statusFilter);
  await applyAllFilters();
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

// Unified filter application - combines all active filters
export async function applyAllFilters() {
  isLoading.set(true);

  try {
    // Get current filter states
    let currentCategories: CategoryWithCount[] = [];
    selectedCategories.subscribe((s) => (currentCategories = s))();

    let currentTimeFilter: string = '';
    activeTimeFilter.subscribe((s) => (currentTimeFilter = s))();

    let currentSizeFilter: string = '';
    activeSizeFilter.subscribe((s) => (currentSizeFilter = s))();

    let currentStatusFilter: string = '';
    activeStatusFilter.subscribe((s) => (currentStatusFilter = s))();

    // If no filters active, show all games
    if (currentCategories.length === 0 && !currentTimeFilter && !currentSizeFilter && !currentStatusFilter) {
      await Promise.all([loadGames(), loadCategories()]);
      return;
    }

    // Apply combined filters
    const categoryIds = currentCategories.map((c) => c.id);
    const { minSize, maxSize } = currentSizeFilter ? getSizeFilterRange(currentSizeFilter) : {};
    const daysAgo = currentTimeFilter ? getTimeFilterDays(currentTimeFilter) : undefined;

    // Call the appropriate backend function based on active filters
    if (categoryIds.length > 0 && currentSizeFilter && currentTimeFilter && daysAgo) {
      // Categories + Size + Time (triple combination)
      await loadGamesByCategoriesSizeAndTime(categoryIds, minSize, maxSize, daysAgo);
    } else if (categoryIds.length > 0 && currentSizeFilter) {
      // Categories + Size
      await loadGamesByCategoriesAndSize(categoryIds, minSize, maxSize);
    } else if (categoryIds.length > 0 && currentTimeFilter && daysAgo) {
      // Categories + Time
      await loadGamesByCategoriesAndTime(categoryIds, daysAgo);
    } else if (currentSizeFilter && currentTimeFilter && daysAgo) {
      // Size + Time
      await loadGamesBySizeAndTime(minSize, maxSize, daysAgo);
    } else if (categoryIds.length > 0) {
      // Categories only
      await loadGamesByMultipleCategories(categoryIds);
    } else if (currentSizeFilter) {
      // Size only
      await loadGamesBySize(minSize, maxSize);
    } else if (currentTimeFilter && daysAgo) {
      // Time only
      await loadGamesByDateRange(daysAgo);
    }

    // Always update categories for faceted filtering
    if (categoryIds.length > 0) {
      // Categories are selected - use category-based faceted filtering
      await loadFilteredCategories(categoryIds);
    } else if (currentTimeFilter && currentSizeFilter && daysAgo) {
      // Only time + size filters are active
      await loadCategoriesForSizeAndTimeFilter(minSize, maxSize, daysAgo);
    } else if (currentTimeFilter && daysAgo) {
      // Only time filter is active
      await loadCategoriesForTimeFilter(daysAgo);
    } else if (currentSizeFilter) {
      // Only size filter is active
      await loadCategoriesForSizeFilter(minSize, maxSize);
    } else {
      // No filters active - show all categories
      await loadCategories();
    }
  } catch (error) {
    console.error('Failed to apply combined filters:', error);
  } finally {
    isLoading.set(false);
  }
}

// Clear specific filters
export async function clearTimeFilter() {
  activeTimeFilter.set('');
  await applyAllFilters();
}

export async function clearSizeFilter() {
  activeSizeFilter.set('');
  await applyAllFilters();
}

export async function clearStatusFilter() {
  activeStatusFilter.set('');
  await applyAllFilters();
}

// Remove specific filter by type and value
export async function removeFilter(type: string, value: string) {
  switch (type) {
    case 'category':
      // Find and remove the category
      selectedCategories.update((selected) => selected.filter((cat) => cat.name !== value));
      break;
    case 'time':
      activeTimeFilter.set('');
      break;
    case 'size':
      activeSizeFilter.set('');
      break;
    case 'status':
      activeStatusFilter.set('');
      break;
  }
  await applyAllFilters();
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

// Load categories filtered by time
export async function loadCategoriesForTimeFilter(daysAgo: number) {
  try {
    const result = await invoke<CategoryWithCount[]>('get_categories_for_time_filtered_games', {
      daysAgo,
    });
    categories.set(result);
  } catch (error) {
    console.error('Failed to load categories for time filter:', error);
  }
}

// Load categories filtered by size
export async function loadCategoriesForSizeFilter(minSize?: number, maxSize?: number) {
  try {
    const result = await invoke<CategoryWithCount[]>('get_categories_for_size_filtered_games', {
      minSize: minSize || null,
      maxSize: maxSize || null,
    });
    categories.set(result);
  } catch (error) {
    console.error('Failed to load categories for size filter:', error);
  }
}

// Load categories filtered by size AND time
export async function loadCategoriesForSizeAndTimeFilter(minSize?: number, maxSize?: number, daysAgo?: number) {
  if (daysAgo === undefined) return;

  try {
    const result = await invoke<CategoryWithCount[]>('get_categories_for_size_and_time_filtered_games', {
      minSize: minSize || null,
      maxSize: maxSize || null,
      daysAgo,
    });
    categories.set(result);
  } catch (error) {
    console.error('Failed to load categories for size and time filter:', error);
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

// Load games by categories AND time (combined filters)
export async function loadGamesByCategoriesAndTime(
  categoryIds: number[],
  daysAgo: number,
  limit: number = 1000,
  offset: number = 0,
) {
  isLoading.set(true);
  try {
    const result = await invoke<Game[]>('get_games_by_categories_and_time', {
      categoryIds,
      daysAgo,
      limit,
      offset,
    });
    games.set(result);
    if (result.length > 0) {
      await selectGame(0);
    }
  } catch (error) {
    console.error('Failed to load games by categories and time:', error);
  } finally {
    isLoading.set(false);
  }
}

// Load games by size AND time (combined filters)
export async function loadGamesBySizeAndTime(
  minSize?: number,
  maxSize?: number,
  daysAgo?: number,
  limit: number = 1000,
  offset: number = 0,
) {
  if (daysAgo === undefined) return;

  isLoading.set(true);
  try {
    const result = await invoke<Game[]>('get_games_by_size_and_time', {
      minSize: minSize || null,
      maxSize: maxSize || null,
      daysAgo,
      limit,
      offset,
    });
    games.set(result);
    if (result.length > 0) {
      await selectGame(0);
    }
  } catch (error) {
    console.error('Failed to load games by size and time:', error);
  } finally {
    isLoading.set(false);
  }
}

// Load games by categories, size AND time (triple combination)
export async function loadGamesByCategoriesSizeAndTime(
  categoryIds: number[],
  minSize?: number,
  maxSize?: number,
  daysAgo?: number,
  limit: number = 1000,
  offset: number = 0,
) {
  if (daysAgo === undefined) return;

  isLoading.set(true);
  try {
    const result = await invoke<Game[]>('get_games_by_categories_size_and_time', {
      categoryIds,
      minSize: minSize || null,
      maxSize: maxSize || null,
      daysAgo,
      limit,
      offset,
    });
    games.set(result);
    if (result.length > 0) {
      await selectGame(0);
    }
  } catch (error) {
    console.error('Failed to load games by categories, size and time:', error);
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
  selectedCategories.set([]);
  activeTimeFilter.set('');
  activeSizeFilter.set('');
  activeStatusFilter.set('');
  await Promise.all([loadGames(), loadCategories()]);
}

// Apply category filters (now uses unified system)
export async function applyCategoryFilters() {
  await applyAllFilters();
}

// Search games (clears all filters)
export async function searchGames(query: string, limit: number = 100) {
  // Clear all filters when searching
  selectedCategories.set([]);
  activeTimeFilter.set('');
  activeSizeFilter.set('');
  activeStatusFilter.set('');

  if (!query.trim()) {
    // If search is cleared, restore normal state
    await Promise.all([loadGames(limit), loadCategories()]);
    return;
  }

  // Search overrides all filters
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
