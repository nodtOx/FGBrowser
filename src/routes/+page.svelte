<script lang="ts">
  import { checkForUpdatesOnStartup } from '$lib/stores/appUpdater';
  import { isCrawlingPopular, loadCategories, loadGames, totalGamesCount } from '$lib/stores/games';
  import { initKeyboardShortcuts } from '$lib/stores/keyboard';
  import { browseView, currentPage, gameListViewMode, loadSavedGameListViewMode } from '$lib/stores/navigation';
  import { refreshPopularCounts } from '$lib/stores/popular';
  import { loadSavedTheme, watchOSThemeChanges } from '$lib/stores/theme';
  import { popularStatus, updateStatus } from '$lib/stores/updates';
  import { invoke } from '@tauri-apps/api/core';
  import { onDestroy, onMount } from 'svelte';

  import Downloads from '$lib/components/Downloads.svelte';
  import GameDetails from '$lib/components/GameDetails.svelte';
  import Header from '$lib/components/Header.svelte';
  import PinkPawAward from '$lib/components/PinkPawAward.svelte';
  import Popular from '$lib/components/Popular.svelte';
  import Settings from '$lib/components/Settings.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import StatusBar from '$lib/components/StatusBar.svelte';
  import UpdateNotification from '$lib/components/UpdateNotification.svelte';
  import VirtualizedGameGrid from '$lib/components/VirtualizedGameGrid.svelte';
  import VirtualizedGameList from '$lib/components/VirtualizedGameList.svelte';

  import { LOAD_ALL_GAMES } from '$lib/constants';
  import '../app.css';

  let databaseError = '';
  let isRetrying = false;

  async function initializeApp() {
    try {
      // Track app launch (if telemetry enabled)
      invoke('track_app_launch').catch(() => {}); // Non-blocking, ignore errors
      
      // Check for app updates in the background
      checkForUpdatesOnStartup();
      
      // Database should be ready from Rust startup, check if it has data
      const isEmpty = await invoke<boolean>('is_database_empty');
      
      if (isEmpty) {
        // Database exists but is empty - Rust download likely failed
        databaseError = 'Database is empty. Please check your internet connection and retry.';
        return;
      }
      
      // Load the games and categories
      await loadGames(LOAD_ALL_GAMES);
      await loadCategories();
      
      // Load popular repacks in the background (non-blocking)
      loadPopularRepacks().catch(err => 
        console.warn('Failed to load popular repacks:', err)
      );
      
      // Start background updates (non-blocking)
      startBackgroundUpdates();
    } catch (error) {
      console.error('Failed to initialize app:', error);
      databaseError = `Failed to load database: ${error}`;
    }
  }

  async function retryDatabaseDownload() {
    isRetrying = true;
    databaseError = '';
    
    try {
      await invoke<boolean>('download_database');
      await initializeApp();
    } catch (error) {
      console.error('Failed to download database:', error);
      databaseError = `Failed to download database: ${error}`;
    } finally {
      isRetrying = false;
    }
  }
  
  async function loadPopularRepacks() {
    try {
      // Load all three periods from database (fetch all available games)
      const monthlyRepacks = await invoke<any[]>('get_popular_repacks', { period: 'month', limit: 9999 });
      const yearlyRepacks = await invoke<any[]>('get_popular_repacks', { period: 'year', limit: 9999 });
      const awardRepacks = await invoke<any[]>('get_popular_repacks', { period: 'award', limit: 9999 });
      console.log(`Loaded ${monthlyRepacks.length} monthly + ${yearlyRepacks.length} yearly + ${awardRepacks.length} award popular repacks from database`);
    } catch (error) {
      console.warn('No popular repacks in database yet');
    }
  }

  async function startBackgroundUpdates() {
    // Check for database updates
    checkForUpdates();
    
    // Fetch popular repacks (non-blocking)
    fetchPopularRepacks();
  }

  async function checkForUpdates() {
    try {
      updateStatus.set({ isUpdating: true, message: '', newGamesFound: 0 });
      
      const result = await invoke<{ total_games: number; status: string }>('update_database');
      
      if (result.total_games > 0) {
        await loadGames(LOAD_ALL_GAMES);
        await loadCategories();
      }
      
      updateStatus.set({ isUpdating: false, message: '', newGamesFound: 0 });
    } catch (error) {
      console.error('Update error:', error);
      updateStatus.set({ isUpdating: false, message: '', newGamesFound: 0 });
    }
  }

  async function fetchPopularRepacks() {
    try {
      popularStatus.set({ isFetching: true, message: '', currentPeriod: '' });
      
      // Fetch all five periods
      await invoke<number>('fetch_popular_repacks', { period: 'week' });
      await invoke<number>('fetch_popular_repacks', { period: 'today' });
      await invoke<number>('fetch_popular_repacks', { period: 'month' });
      await invoke<number>('fetch_popular_repacks', { period: 'year' });
      await invoke<number>('fetch_popular_repacks', { period: 'award' });
      
      isCrawlingPopular.set(true);
      
      // Crawl all five periods
      await invoke<number>('crawl_popular_games', { period: 'week' });
      await invoke<number>('crawl_popular_games', { period: 'today' });
      await invoke<number>('crawl_popular_games', { period: 'month' });
      await invoke<number>('crawl_popular_games', { period: 'year' });
      await invoke<number>('crawl_popular_games', { period: 'award' });
      
      isCrawlingPopular.set(false);
      
      // Reload data
      await loadPopularRepacks();
      
      // Refresh badges
      refreshPopularCounts();
      
      popularStatus.set({ isFetching: false, message: '', currentPeriod: '' });
    } catch (error) {
      console.error('Failed to fetch popular repacks:', error);
      isCrawlingPopular.set(false);
      popularStatus.set({ isFetching: false, message: '', currentPeriod: '' });
    }
  }

  let unwatchOSTheme: (() => void) | null = null;

  onMount(async () => {
    // Load saved theme (or detect from OS)
    loadSavedTheme();

    // Load saved game list view mode
    loadSavedGameListViewMode();

    // Watch for OS theme changes
    unwatchOSTheme = watchOSThemeChanges((newTheme) => {
      console.log(`OS theme changed, auto-switching to ${newTheme.name}`);
    });

    // Initialize keyboard shortcuts
    initKeyboardShortcuts();

    // Initialize app (database is already ready from Rust startup)
    await initializeApp();
  });

  onDestroy(() => {
    // Cleanup OS theme watcher
    if (unwatchOSTheme) {
      unwatchOSTheme();
    }
  });
</script>

<div class="app">
  <UpdateNotification />
  <Header />

  <main class="main-content">
    {#if databaseError}
      <div class="page-placeholder">
        <div class="error-icon">✕</div>
        <h2>Database Error</h2>
        <p class="error-message">{databaseError}</p>
        <div class="error-actions">
          <button class="retry-button" on:click={retryDatabaseDownload} disabled={isRetrying}>
            {isRetrying ? 'Downloading...' : 'Retry Download'}
          </button>
        </div>
        <p class="text-muted">The database may have failed to download during startup</p>
      </div>
    {:else if $currentPage === 'browse'}
      <div class="browse-page">
        <div class="browse-layout">
          <Sidebar totalGames={$totalGamesCount} />

          <div class="main-content-area">
            <div class="view-container" class:hidden={$browseView !== 'list'}>
              {#if $gameListViewMode === 'list'}
                <VirtualizedGameList />
              {:else}
                <VirtualizedGameGrid />
              {/if}
            </div>
            <div class="view-container" class:hidden={$browseView !== 'details'}>
              <GameDetails />
            </div>
          </div>
        </div>
      </div>
    {:else if $currentPage === 'popular'}
      <Popular />
    {:else if $currentPage === 'pinkpaw'}
      <PinkPawAward />
    {:else if $currentPage === 'downloads'}
      <Downloads />
    {:else if $currentPage === 'settings'}
      <Settings />
    {:else if $currentPage === 'stats'}
      <div class="page-placeholder">
        <h2>Stats Page</h2>
        <p>Statistics dashboard coming soon...</p>
      </div>
    {/if}
  </main>

  <StatusBar />
</div>

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
  }

  .main-content {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .browse-page {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .browse-layout {
    display: flex;
    flex: 1;
    overflow: hidden;
    min-height: 0;
  }

  .main-content-area {
    flex: 1;
    overflow: hidden;
    position: relative;
  }

  .view-container {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .view-container.hidden {
    visibility: hidden;
    pointer-events: none;
  }

  .page-placeholder {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--color-textMuted);
    gap: 16px;
    user-select: text;
  }

  .page-placeholder h2 {
    font-size: 24px;
    color: var(--color-primary);
  }

  .page-placeholder p {
    font-size: 16px;
  }

  .text-muted {
    color: var(--color-textMuted);
    font-size: 14px;
  }

  .error-icon {
    width: 64px;
    height: 64px;
    border-radius: 50%;
    background: rgba(239, 68, 68, 0.1);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 32px;
    color: #ef4444;
    margin-bottom: 8px;
  }

  .error-message {
    color: #ef4444;
    font-size: 14px;
    margin: 16px 0;
    max-width: 500px;
  }

  .error-actions {
    margin: 24px 0 16px 0;
  }

  .retry-button {
    background: var(--color-primary);
    color: white;
    border: none;
    padding: 12px 24px;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .retry-button:hover:not(:disabled) {
    background: var(--color-primaryHover);
    transform: translateY(-1px);
  }

  .retry-button:active:not(:disabled) {
    transform: translateY(0);
  }

  .retry-button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
</style>
