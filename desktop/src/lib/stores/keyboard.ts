import { featureFlags } from '$lib/featureFlags';
import { get, writable } from 'svelte/store';
import { moveSelection } from './games';
import { currentPage, navigateTo, openGameDetails, type Page } from './navigation';

export const keyboardEnabled = writable<boolean>(true);

// Tab order for cycling (filtered by enabled features)
function getTabOrder(): Page[] {
  const tabs: Page[] = ['browse', 'popular'];

  if (featureFlags.torrentClient) {
    tabs.push('downloads');
  }

  tabs.push('settings');

  if (featureFlags.stats) {
    tabs.push('stats');
  }

  return tabs;
}

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
  const TAB_ORDER = getTabOrder();
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

  // Open game details with Space (only on browse page, not when typing)
  if (key === ' ' && !ctrl && !shift && !isTyping) {
    const current = get(currentPage);
    if (current === 'browse') {
      openGameDetails();
      return true;
    }
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
