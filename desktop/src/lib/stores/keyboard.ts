import { get, writable } from 'svelte/store';
import { copyMagnetLink, moveSelection, openMagnetLink, selectedGame } from './games';
import { currentPage, navigateTo, type Page } from './navigation';

export const keyboardEnabled = writable<boolean>(true);

// Tab order for cycling
const TAB_ORDER: Page[] = ['browse', 'popular', 'downloads', 'settings', 'stats'];

export function initKeyboardShortcuts() {
  if (typeof window === 'undefined') return;

  window.addEventListener('keydown', async (e) => {
    const enabled = get(keyboardEnabled);
    if (!enabled) return;

    // Prevent default for handled keys
    const handled = handleKeyPress(e);
    if (handled) {
      e.preventDefault();
    }
  });
}

function cycleTab(direction: 'next' | 'previous') {
  const current = get(currentPage);
  const currentIndex = TAB_ORDER.indexOf(current);

  if (currentIndex === -1) return;

  let nextIndex: number;
  if (direction === 'next') {
    nextIndex = (currentIndex + 1) % TAB_ORDER.length;
  } else {
    nextIndex = (currentIndex - 1 + TAB_ORDER.length) % TAB_ORDER.length;
  }

  navigateTo(TAB_ORDER[nextIndex]);
}

function handleKeyPress(e: KeyboardEvent): boolean {
  const key = e.key;
  const ctrl = e.ctrlKey || e.metaKey;
  const shift = e.shiftKey;

  // Navigation between games
  if ((key === 'ArrowUp' || key === 'k') && !ctrl) {
    moveSelection('up');
    return true;
  }

  if ((key === 'ArrowDown' || key === 'j') && !ctrl) {
    moveSelection('down');
    return true;
  }

  // Page navigation - cycle through tabs
  if (ctrl && key === ']') {
    cycleTab('next');
    return true;
  }

  if (ctrl && key === '[') {
    cycleTab('previous');
    return true;
  }

  // Search
  if ((key === '/' || (ctrl && key === 'f')) && !shift) {
    focusSearch();
    return true;
  }

  // Open first magnet link
  if (key === 'Enter') {
    const game = get(selectedGame);
    if (game && game.magnet_links.length > 0) {
      openMagnetLink(game.magnet_links[0].magnet);
    }
    return true;
  }

  // Open specific magnet link (1-9)
  if (!ctrl && key >= '1' && key <= '9') {
    const game = get(selectedGame);
    const index = parseInt(key) - 1;
    if (game && game.magnet_links[index]) {
      openMagnetLink(game.magnet_links[index].magnet);
    }
    return true;
  }

  // Copy first magnet link
  if (key === 'c' && !ctrl) {
    const game = get(selectedGame);
    if (game && game.magnet_links.length > 0) {
      copyMagnetLink(game.magnet_links[0].magnet);
    }
    return true;
  }

  // Quit
  if ((ctrl && key === 'q') || key === 'Q') {
    // Handle quit
    return true;
  }

  // Fullscreen
  if (key === 'F11') {
    toggleFullscreen();
    return true;
  }

  return false;
}

function focusSearch() {
  const searchInput = document.querySelector('input[type="search"], input[placeholder*="Search"]') as HTMLInputElement;
  if (searchInput) {
    searchInput.focus();
    searchInput.select();
  }
}

function toggleFullscreen() {
  if (!document.fullscreenElement) {
    document.documentElement.requestFullscreen();
  } else {
    document.exitFullscreen();
  }
}
