import { writable } from 'svelte/store';

export type Page = 'browse' | 'downloads' | 'settings' | 'stats' | 'about';

export const currentPage = writable<Page>('browse');

export function navigateTo(page: Page) {
  currentPage.set(page);
}
