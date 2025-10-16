# Testing Database Download Feature

## Prerequisites

1. Ensure the database server is running at `http://157.230.16.45/repacks.db`
2. Make sure you have built the app at least once

## Test Scenarios

### Scenario 1: First-time installation (no local database)

1. Delete the local database file:

   ```bash
   rm desktop/repacks.db
   # or
   rm repacks.db
   ```

2. Start the app:

   ```bash
   cd desktop
   npm run tauri dev
   ```

3. **Expected behavior:**
   - App should show "Downloading Database" message with a spinner
   - Console should show:
     ```
     DATABASE DOWNLOAD STARTED
     Source: http://157.230.16.45/repacks.db
     ...
     DATABASE DOWNLOAD COMPLETED
     Total Games: [number]
     ```
   - After download, games should load immediately
   - Update crawler should run in background
   - "Downloading Database" message should disappear

### Scenario 2: Database exists and has data

1. Ensure `repacks.db` exists with data
2. Start the app

3. **Expected behavior:**
   - No download should occur
   - Console should show: "Database already exists with [X] games, skipping download"
   - Games load immediately
   - Update crawler runs to check for new games

### Scenario 3: Database file exists but is empty/corrupted

1. Create an empty or corrupted database file:

   ```bash
   echo "" > desktop/repacks.db
   ```

2. Start the app

3. **Expected behavior:**
   - App detects empty/corrupted database
   - Downloads fresh copy from server
   - Console shows download progress
   - Games load after download

### Scenario 4: Server is unreachable

1. Stop the database server or change URL to invalid one
2. Delete local database
3. Start the app

4. **Expected behavior:**
   - Download fails with error message
   - Error screen is displayed with:
     - Red error icon
     - Clear error message
     - "Retry Download" button
     - Suggestion to check internet connection
   - User can click "Retry Download" to try again
   - No fallback to crawler (user must fix connection or retry)

## Verification Checklist

- [ ] First run downloads database successfully
- [ ] Download progress is visible to user
- [ ] Downloaded database contains games
- [ ] Subsequent runs skip download
- [ ] Empty database triggers re-download
- [ ] Failed download shows error screen
- [ ] Retry button works and attempts re-download
- [ ] Update crawler runs after successful download
- [ ] Database integrity is verified after download

## Manual Verification

After download completes, verify database contents:

```bash
# Check database size
ls -lh desktop/repacks.db

# Query database
cd desktop/src-tauri
cargo run --bin cli -- stats
```

Expected output should show statistics like:

```
Total repacks: [number]
Total magnet links: [number]
Categories: [number]
```

## Performance Metrics

Track these metrics during testing:

- **Download time**: Time from start to completion
- **Database size**: Size of downloaded file
- **Verification time**: Time to verify database integrity
- **Total startup time**: From app launch to games loaded

Example benchmarks (will vary based on connection):

- Database size: ~100-200 MB
- Download time: 10-60 seconds (depending on connection)
- Verification: < 1 second
- Total first-run startup: 15-90 seconds

## Debugging

If issues occur, check:

1. **Console logs**: Look for error messages
2. **Network**: Verify server is accessible
   ```bash
   curl -I http://157.230.16.45/repacks.db
   ```
3. **Permissions**: Ensure app can write to directory
4. **Disk space**: Ensure sufficient space for database

## Common Issues

### Issue: "Failed to download database"

- **Cause**: Server unreachable or network error
- **Solution**:
  1. Check internet connection
  2. Verify server is running: `curl -I http://157.230.16.45/repacks.db`
  3. Click "Retry Download" button
  4. If persists, check firewall/proxy settings

### Issue: "Downloaded database is invalid"

- **Cause**: Corrupted download or invalid database file
- **Solution**:
  1. Check server file integrity
  2. Delete local database file
  3. Retry download

### Issue: Download stuck

- **Cause**: Large file, slow connection
- **Solution**:
  1. Wait patiently (may take 30-60 seconds on slow connections)
  2. Check network speed
  3. If truly stuck (>5 minutes), restart app

## Success Criteria

The feature is working correctly if:

1. ✅ Fresh install downloads database automatically
2. ✅ Download completes without errors
3. ✅ Downloaded database is verified as valid
4. ✅ Games load immediately after download
5. ✅ Update crawler runs after initial download
6. ✅ Subsequent app launches skip download
7. ✅ Error handling works (shows error screen with retry)
8. ✅ Retry button successfully re-attempts download
