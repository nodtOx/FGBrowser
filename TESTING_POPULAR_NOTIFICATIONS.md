# Testing Popular Game Notifications Feature

This document explains how to test the new popular game notifications feature.

## Feature Overview

The application now tracks when users view popular games and shows notifications for new additions:

### 1. **NEW Badge on Game Cards**

- Games added in the last 7 days show a green "NEW" badge
- Badge appears on the game card (top right corner)
- Automatically disappears after 7 days

### 2. **Unseen Count Badges**

- **Header "Popular" Tab**: Shows total unseen count across all periods
- **Sidebar Period Items**: Shows unseen count per period (Month/Year/Award)
- Badges are red circles with white numbers
- Disappear when you view that period

### 3. **Auto-Mark as Viewed**

- When you switch to a period (Month/Year/Award), it's automatically marked as viewed
- The unseen count resets for that period
- Badge colors invert when the tab is active

## How to Test

### Method 1: Using the SQL Test Script (Recommended)

1. **Make sure the app is closed**

2. **Run the test script:**

   ```bash
   cd /Users/ekinertac/Code/fit-boy
   sqlite3 desktop/repacks.db < desktop/test_popular_notifications.sql
   ```

3. **Open the app**

4. **Expected behavior:**

   - Header "Popular" tab shows a badge (e.g., "8")
   - Popular sidebar shows badges on "This Month" and "This Year"
   - Games with rank 1-5 (month) and 1-3 (year) have "NEW" badges
   - Game #10 (month) also has a "NEW" badge (6 days old)

5. **Click "This Month":**

   - The badge on "This Month" should disappear after ~500ms
   - Header badge count should decrease
   - "NEW" badges still visible on recent games

6. **Click "This Year":**

   - The badge on "This Year" should disappear
   - Header badge count should update

7. **Restart the app:**
   - Badges should stay gone (timestamps are saved)
   - Only unviewed periods show badges

### Method 2: Manual Testing with Real Data

1. **Clear existing timestamps (optional):**

   ```bash
   sqlite3 desktop/repacks.db "DELETE FROM settings WHERE key = 'app_settings';"
   ```

2. **Crawl popular games:**

   ```bash
   cd /Users/ekinertac/Code/fit-boy/desktop/src-tauri
   ./target/release/cli popular --period all
   ```

3. **Open the app:**

   - All popular games should show badges (first time viewing)
   - All games are marked as "NEW" (just crawled)

4. **View each period:**

   - Badges should disappear as you view them

5. **Wait 7 days or manually age some games:**

   ```sql
   -- Age half the games to 10 days old
   UPDATE popular_repacks
   SET created_at = datetime('now', '-10 days')
   WHERE id % 2 = 0;
   ```

6. **Restart app:**
   - Only recently added games show "NEW" badges
   - Unseen counts show games added since last view

## Database Schema

### Settings (JSON in `settings` table):

```json
{
  "popular_month_last_viewed": "2025-01-15T18:30:00Z",
  "popular_year_last_viewed": "2025-01-15T18:30:00Z",
  "popular_award_last_viewed": "2025-01-15T18:30:00Z"
}
```

### Popular Repacks Table:

```sql
SELECT
  id,
  repack_id,
  rank,
  period,
  created_at,
  CASE WHEN julianday('now') - julianday(created_at) <= 7
    THEN 'NEW' ELSE 'old' END as status
FROM popular_repacks
LIMIT 10;
```

## Troubleshooting

### Badges not showing?

1. Check if popular games are crawled: `sqlite3 desktop/repacks.db "SELECT COUNT(*) FROM popular_repacks;"`
2. Check timestamps: `sqlite3 desktop/repacks.db "SELECT period, COUNT(*) FROM popular_repacks GROUP BY period;"`
3. Clear settings and restart: `sqlite3 desktop/repacks.db "DELETE FROM settings;"`

### Badges not disappearing?

1. Check browser console for errors
2. Wait 500ms after clicking (there's a small delay)
3. Check settings are saving: `sqlite3 desktop/repacks.db "SELECT value FROM settings WHERE key = 'app_settings';" | python3 -m json.tool`

### "NEW" badges on wrong games?

1. Check created_at timestamps: Run the SQL query from the test script
2. Ensure games were added in the last 7 days
3. Use the test script to manually set timestamps

## Resetting for Fresh Tests

```bash
# Reset all timestamps to now (all games "new")
sqlite3 desktop/repacks.db "UPDATE popular_repacks SET created_at = CURRENT_TIMESTAMP;"

# Reset all timestamps to 30 days ago (all games "old")
sqlite3 desktop/repacks.db "UPDATE popular_repacks SET created_at = datetime('now', '-30 days');"

# Clear last viewed timestamps (all games "unseen")
sqlite3 desktop/repacks.db "UPDATE settings SET value = json_set(value, '$.popular_month_last_viewed', NULL, '$.popular_year_last_viewed', NULL, '$.popular_award_last_viewed', NULL) WHERE key = 'app_settings';"

# Nuclear option: delete everything and re-crawl
sqlite3 desktop/repacks.db "DELETE FROM popular_repacks; DELETE FROM settings;"
./desktop/src-tauri/target/release/cli popular --period all
```

## Code Locations

- **Backend Commands**: `desktop/src-tauri/src/commands.rs` (lines 904-958)
- **Database Queries**: `desktop/src-tauri/src/database/popular_queries.rs` (lines 139-173)
- **Settings Model**: `desktop/src-tauri/src/database/models.rs` (lines 132-136)
- **Header Badge**: `desktop/src/lib/components/Header.svelte` (lines 89-91, 227-242)
- **Sidebar Badges**: `desktop/src/lib/components/PopularSidebar.svelte` (lines 11-45, 62-98, 146-156)
- **NEW Badges**: `desktop/src/lib/components/Popular.svelte` (lines 223-225, 367-379)
- **Test Script**: `desktop/test_popular_notifications.sql`
