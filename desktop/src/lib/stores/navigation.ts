import { writable } from 'svelte/store';

export type Page = 'browse' | 'popular' | 'downloads' | 'settings' | 'stats' | 'about';
export type BrowseView = 'list' | 'details';

export const currentPage = writable<Page>('browse');
export const browseView = writable<BrowseView>('list');
export const showGameDetails = writable<boolean>(false); // Keep for backward compatibility

export function navigateTo(page: Page) {
  currentPage.set(page);
  // Reset browse view when navigating away from browse
  if (page !== 'browse') {
    browseView.set('list');
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
