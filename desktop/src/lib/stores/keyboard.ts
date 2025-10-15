import { get, writable } from 'svelte/store';
import { moveSelection } from './games';
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

  // Check if user is typing in an input field
  const target = e.target as HTMLElement;
  const isTyping = target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable;

  // Allow Cmd+F and / to focus search even when in input
  if ((key === '/' || (ctrl && key === 'f')) && !shift) {
    focusSearch();
    return true;
  }

  // Navigation between games (arrow keys work everywhere, including in search)
  if (key === 'ArrowUp' && !ctrl) {
    moveSelection('up');
    return true;
  }

  if (key === 'ArrowDown' && !ctrl) {
    moveSelection('down');
    return true;
  }

  // Skip other shortcuts if user is typing
  if (isTyping) {
    return false;
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

  return false;
}

function focusSearch() {
  const searchInput = document.querySelector('input[type="search"], input[placeholder*="Search"]') as HTMLInputElement;
  if (searchInput) {
    searchInput.focus();
    searchInput.select();
  }
}
