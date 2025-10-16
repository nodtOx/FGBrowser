import { writable } from 'svelte/store';

export type Page = 'browse' | 'popular' | 'downloads' | 'settings' | 'stats' | 'about';
export type BrowseView = 'list' | 'details';
export type BrowsePanel = 'categories' | 'recent' | 'size' | 'search' | 'gamelist';

const panelOrder: BrowsePanel[] = ['search', 'gamelist', 'categories', 'recent', 'size'];

export const currentPage = writable<Page>('browse');
export const browseView = writable<BrowseView>('list');
export const focusedPanel = writable<BrowsePanel>('search');
export const showGameDetails = writable<boolean>(false); // Keep for backward compatibility

export function navigateTo(page: Page) {
  currentPage.set(page);
  // Reset browse view when navigating away from browse
  if (page !== 'browse') {
    browseView.set('list');
    focusedPanel.set('search');
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
