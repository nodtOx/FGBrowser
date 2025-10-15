# App Lifecycle - Popular Repacks Integration

## Overview

The popular repacks feature is strategically integrated into the FitBoy application lifecycle to balance freshness, performance, and user experience.

## Integration Points

### 1. App Startup (Database Load)

**Trigger**: When app launches with existing data  
**Action**: Load popular repacks from database  
**Command**: `get_popular_repacks(limit: 50)`  
**Network**: No (database query only)  
**Blocking**: No (runs in background)  
**Duration**: <50ms typically

```typescript
// +page.svelte - checkAndInitializeDatabase()
loadPopularRepacks().catch((err) => console.warn('Failed to load popular repacks:', err));
```

**Purpose**:

- Show previously fetched popular games immediately
- No delay in app startup
- User sees data from last update

---

### 2. After Crawler Completes (Website Fetch)

**Trigger**: When main catalog crawler finishes  
**Action**: Fetch fresh popular repacks from FitGirl website  
**Command**: `fetch_popular_repacks()`  
**Network**: Yes (HTTP request to fitgirl-repacks.site)  
**Blocking**: No (runs in background)  
**Duration**: 1-3 seconds typically

```typescript
// +page.svelte - onCrawlerComplete()
async function onCrawlerComplete() {
  await loadGames(100);
  await loadCategories();

  // Fetch fresh data from website
  fetchPopularRepacks().catch((err) => console.error('Failed to fetch popular repacks:', err));
}
```

**Purpose**:

- Get the latest popularity rankings
- Automatically link to newly crawled games
- Ensures data is fresh after main catalog update

**Flow**:

1. Fetch HTML from `https://fitgirl-repacks.site/popular-repacks/`
2. Parse grid layout to extract ~50 games
3. Clear old popular repacks in database
4. Save new popular repacks
5. Link to existing games by URL matching

---

### 3. Manual Refresh (User-Triggered)

**Trigger**: User clicks refresh button in Settings/UI  
**Action**: Re-fetch popular repacks on demand  
**Command**: `fetch_popular_repacks()`  
**Network**: Yes  
**Blocking**: Yes (shows loading indicator)  
**Duration**: 1-3 seconds

```typescript
async function refreshPopularRepacks() {
  try {
    isLoading.set(true);
    const count = await invoke('fetch_popular_repacks');
    console.log(`Refreshed ${count} popular repacks`);
    await loadPopularRepacks(); // Reload into UI
  } catch (error) {
    console.error('Failed to refresh:', error);
  } finally {
    isLoading.set(false);
  }
}
```

**Purpose**:

- User wants latest rankings without full crawl
- Quick way to update popular list
- Can be done daily/weekly

---

## Lifecycle Diagram

```
┌─────────────────────────────────────────────────────────┐
│                     APP LAUNCHES                         │
└─────────────────┬───────────────────────────────────────┘
                  │
                  ▼
         ┌────────────────┐
         │  Database      │
         │  Empty?        │
         └───┬────────┬───┘
             │        │
        YES  │        │ NO
             │        │
             ▼        ▼
    ┌────────────┐  ┌──────────────────────┐
    │   Show     │  │ Load Games &         │
    │  Crawler   │  │ Categories           │
    │   Modal    │  │                      │
    └─────┬──────┘  └──────┬───────────────┘
          │                │
          │                ▼
          │         ┌──────────────────────┐
          │         │ loadPopularRepacks() │
          │         │ (from DB, quick)     │
          │         └──────────────────────┘
          │
          ▼
    ┌────────────┐
    │   Start    │
    │  Crawler   │
    └─────┬──────┘
          │
          ▼
    ┌────────────┐
    │ Crawling   │
    │ 10 pages...│
    └─────┬──────┘
          │
          ▼
    ┌──────────────────────────┐
    │  onCrawlerComplete()     │
    │  - Load games (100)      │
    │  - Load categories       │
    │  - fetchPopularRepacks() │ ← Network request
    └──────────────────────────┘
          │
          ▼
    ┌──────────────────────────┐
    │ Popular repacks updated! │
    │ Games auto-linked by URL │
    └──────────────────────────┘
```

---

## Data Flow

### fetch_popular_repacks() Flow

```
Frontend                     Backend (Rust)              Website
   │                              │                         │
   │  invoke('fetch_popular...')  │                         │
   ├─────────────────────────────>│                         │
   │                              │                         │
   │                              │  GET /popular-repacks/  │
   │                              ├────────────────────────>│
   │                              │                         │
   │                              │      HTML response      │
   │                              │<────────────────────────┤
   │                              │                         │
   │                              │ Parse HTML              │
   │                              │ (extract 50 games)      │
   │                              │                         │
   │                              │ Clear old DB records    │
   │                              │ Save new records        │
   │                              │ Link to existing games  │
   │                              │                         │
   │        count: 50             │                         │
   │<─────────────────────────────┤                         │
   │                              │                         │
   │  invoke('get_popular...')    │                         │
   ├─────────────────────────────>│                         │
   │                              │                         │
   │    Vec<PopularRepack>        │                         │
   │<─────────────────────────────┤                         │
   │                              │                         │
```

---

## Best Practices

### ✅ DO

- **Load from database on startup** (quick, no network)
- **Fetch from website after crawling** (ensures freshness)
- **Run fetches in background** (non-blocking)
- **Handle errors gracefully** (app works without popular repacks)
- **Cache in database** (reduces network requests)

### ❌ DON'T

- **Don't fetch on every app startup** (unnecessary network traffic)
- **Don't block app launch** (bad UX)
- **Don't fail if fetch fails** (not critical feature)
- **Don't fetch too frequently** (respect website)
- **Don't fetch without user consent** (privacy)

---

## Performance Considerations

| Operation                 | Duration | Network | User Impact          |
| ------------------------- | -------- | ------- | -------------------- |
| `get_popular_repacks()`   | <50ms    | No      | None (instant)       |
| `fetch_popular_repacks()` | 1-3s     | Yes     | Minimal (background) |
| Linking to games          | <10ms    | No      | None (automatic)     |

---

## Future Enhancements

1. **Periodic Auto-Refresh**: Check for updates every 24 hours
2. **Last Updated Indicator**: Show when popular list was last updated
3. **Popularity Trends**: Track rank changes over time
4. **Notification**: Alert user when new popular games are added
5. **Offline Mode**: Always work with cached data if network fails

---

## Testing

To test the lifecycle integration:

```typescript
// 1. Test initial load (from DB)
await invoke('get_popular_repacks', { limit: 10 });

// 2. Test fetch from website
const count = await invoke('fetch_popular_repacks');
console.log(`Fetched ${count} games`);

// 3. Test with games
const withGames = await invoke('get_popular_repacks_with_games', { limit: 10 });
console.log(`${withGames.length} popular games loaded`);

// 4. Test linking
const linked = await invoke('update_popular_repack_links');
console.log(`${linked} games linked to catalog`);
```

---

## Summary

The popular repacks feature intelligently integrates into the app lifecycle:

1. **Fast startup**: Loads from database, no network delay
2. **Smart updates**: Fetches after main crawler finishes
3. **User control**: Manual refresh option available
4. **Non-blocking**: Runs in background, never blocks UI
5. **Resilient**: App works fine if popular repacks fail to load

This design balances freshness, performance, and user experience.
