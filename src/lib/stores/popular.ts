import { writable } from 'svelte/store';

export const popularViewedTrigger = writable(0);

export function triggerPopularViewed() {
  popularViewedTrigger.update((n) => n + 1);
}

// Trigger to refresh counts after database initialization
export function refreshPopularCounts() {
  console.log('[Store] refreshPopularCounts() called');
  popularViewedTrigger.update((n) => {
    console.log(`[Store] Updating trigger from ${n} to ${n + 1}`);
    return n + 1;
  });
}
