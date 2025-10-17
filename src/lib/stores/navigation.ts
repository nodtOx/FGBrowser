import { writable } from 'svelte/store';

export type Page = 'browse' | 'popular' | 'pinkpaw' | 'downloads' | 'settings' | 'stats' | 'about';
export type BrowseView = 'list' | 'details';
export type BrowsePanel = 'categories' | 'recent' | 'size' | 'search' | 'gamelist';
export type GameListViewMode = 'list' | 'grid';

const panelOrder: BrowsePanel[] = ['search', 'gamelist', 'categories', 'recent', 'size'];

export const currentPage = writable<Page>('browse');
export const browseView = writable<BrowseView>('list');
export const focusedPanel = writable<BrowsePanel>('gamelist');
export const showGameDetails = writable<boolean>(false); // Keep for backward compatibility
export const gameListViewMode = writable<GameListViewMode>('list');

export function navigateTo(page: Page) {
  currentPage.set(page);
  // Reset browse view when navigating away from browse
  if (page !== 'browse') {
    browseView.set('list');
    focusedPanel.set('gamelist');
  }
}

export function openGameDetails() {
  browseView.set('details');
  showGameDetails.set(true); // Keep for backward compatibility
}

export function closeGameDetails() {
  browseView.set('list');
  showGameDetails.set(false); // Keep for backward compatibility
}

export function goBack() {
  closeGameDetails();
}

export function cycleFocusPanel(direction: 'next' | 'previous' = 'next') {
  focusedPanel.update((current) => {
    const currentIndex = panelOrder.indexOf(current);
    if (currentIndex === -1) return 'gamelist';

    let nextIndex: number;
    if (direction === 'next') {
      nextIndex = (currentIndex + 1) % panelOrder.length;
    } else {
      nextIndex = (currentIndex - 1 + panelOrder.length) % panelOrder.length;
    }

    return panelOrder[nextIndex];
  });
}

export function setGameListViewMode(mode: GameListViewMode) {
  gameListViewMode.set(mode);
  saveGameListViewMode(mode);
}

export function toggleGameListViewMode() {
  gameListViewMode.update((current) => {
    const newMode = current === 'list' ? 'grid' : 'list';
    saveGameListViewMode(newMode);
    return newMode;
  });
}

// Persistence functions
const GAME_LIST_VIEW_MODE_KEY = 'gameListViewMode';

function saveGameListViewMode(mode: GameListViewMode) {
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem(GAME_LIST_VIEW_MODE_KEY, mode);
  }
}

export function loadSavedGameListViewMode() {
  if (typeof localStorage !== 'undefined') {
    const saved = localStorage.getItem(GAME_LIST_VIEW_MODE_KEY);
    if (saved === 'list' || saved === 'grid') {
      gameListViewMode.set(saved);
    }
  }
}
