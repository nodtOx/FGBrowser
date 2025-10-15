import { writable } from 'svelte/store';

export type Page = 'browse' | 'popular' | 'downloads' | 'settings' | 'stats' | 'about';

export const currentPage = writable<Page>('browse');
export const showGameDetails = writable<boolean>(false);

export function navigateTo(page: Page) {
  currentPage.set(page);
}

export function openGameDetails() {
  showGameDetails.set(true);
}

export function closeGameDetails() {
  showGameDetails.set(false);
}
