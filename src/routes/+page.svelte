<script lang="ts">
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
  import VirtualizedGameGrid from '$lib/components/VirtualizedGameGrid.svelte';
  import VirtualizedGameList from '$lib/components/VirtualizedGameList.svelte';

  import { LOAD_ALL_GAMES } from '$lib/constants';
  import '../app.css';

  let isDownloadingDatabase = false;
  let downloadError = '';

  async function checkAndInitializeDatabase() {
    try {
      // First, check if database file exists
      const dbExists = await invoke<boolean>('check_database_exists');
      
      if (!dbExists) {
        // Database doesn't exist, download it from server
        console.log('Database not found locally, downloading from server...');
        isDownloadingDatabase = true;
        downloadError = '';
        
        try {
          const downloaded = await invoke<boolean>('download_database');
          
          if (downloaded) {
            console.log('Database downloaded successfully!');
            // Database downloaded, load games
            await loadGames(LOAD_ALL_GAMES);
            await loadCategories();
            
            // Load popular repacks in the background
            loadPopularRepacks().catch(err => 
              console.warn('Failed to load popular repacks:', err)
            );
            
            // Start background updates (non-blocking)
            startBackgroundUpdates();
          } else {
            console.log('Database already exists, skipped download');
          }
        } catch (error) {
          console.error('Failed to download database:', error);
          downloadError = `Failed to download database: ${error}`;
        } finally {
          isDownloadingDatabase = false;
        }
      } else {
        // Database exists, check if it's empty
        const isEmpty = await invoke<boolean>('is_database_empty');
        
        if (isEmpty) {
          // Database file exists but is empty, download fresh copy
          console.log('Database is empty, downloading fresh copy...');
          isDownloadingDatabase = true;
          downloadError = '';
          
          try {
            await invoke<boolean>('download_database');
            console.log('Database downloaded successfully!');
            await loadGames(LOAD_ALL_GAMES);
            await loadCategories();
            
            loadPopularRepacks().catch(err => 
              console.warn('Failed to load popular repacks:', err)
            );
            
            // Start background updates (non-blocking)
            startBackgroundUpdates();
          } catch (error) {
            console.error('Failed to download database:', error);
            downloadError = `Failed to download database: ${error}`;
          } finally {
            isDownloadingDatabase = false;
          }
        } else {
          // Database has data, check for updates
          await loadGames(LOAD_ALL_GAMES);
          await loadCategories();
          
          // Load popular repacks in the background (non-blocking)
          loadPopularRepacks().catch(err => 
            console.warn('Failed to load popular repacks:', err)
          );
          
          // Start background updates (non-blocking)
          startBackgroundUpdates();
        }
      }
    } catch (error) {
      console.error('Error checking database:', error);
      downloadError = `Error initializing database: ${error}`;
    }
  }
  
  function retryDownload() {
    downloadError = '';
    checkAndInitializeDatabase();
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

    // Check if database needs initialization and load games
    await checkAndInitializeDatabase();
  });

  onDestroy(() => {
    // Cleanup OS theme watcher
    if (unwatchOSTheme) {
      unwatchOSTheme();
    }
  });
</script>

<div class="app">
  <Header />

  <main class="main-content">
    {#if downloadError}
      <div class="page-placeholder">
        <div class="error-icon">✕</div>
        <h2>Database Download Failed</h2>
        <p class="error-message">{downloadError}</p>
        <div class="error-actions">
          <button class="retry-button" on:click={retryDownload}>
            Retry Download
          </button>
        </div>
        <p class="text-muted">Please check your internet connection and try again</p>
      </div>
    {:else if isDownloadingDatabase}
      <div class="page-placeholder">
        <div class="loading-spinner"></div>
        <h2>Downloading Database</h2>
        <p>Please wait while we download the game database from the server...</p>
        <p class="text-muted">This only happens once on first run</p>
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

  .loading-spinner {
    width: 48px;
    height: 48px;
    border: 4px solid var(--color-border);
    border-top-color: var(--color-primary);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
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

  .retry-button:hover {
    background: var(--color-primaryHover);
    transform: translateY(-1px);
  }

  .retry-button:active {
    transform: translateY(0);
  }
</style>
