import { writable } from 'svelte/store';

export const popularViewedTrigger = writable(0);

export function triggerPopularViewed() {
  popularViewedTrigger.update((n) => n + 1);
}
