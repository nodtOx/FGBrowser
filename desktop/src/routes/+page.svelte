<script lang="ts">
  import { games, isCrawlingPopular, loadCategories, loadGames, popularCrawlProgress, totalGamesCount } from '$lib/stores/games';
  import { initKeyboardShortcuts } from '$lib/stores/keyboard';
  import { browseView, currentPage } from '$lib/stores/navigation';
  import { loadSavedTheme } from '$lib/stores/theme';
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  import CrawlerModal from '$lib/components/CrawlerModal.svelte';
  import Downloads from '$lib/components/Downloads.svelte';
  import GameDetails from '$lib/components/GameDetails.svelte';
  import Header from '$lib/components/Header.svelte';
  import Popular from '$lib/components/Popular.svelte';
  import Settings from '$lib/components/Settings.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import StatusBar from '$lib/components/StatusBar.svelte';
  import VirtualizedGameList from '$lib/components/VirtualizedGameList.svelte';

  import { LOAD_ALL_GAMES, POLLING_INTERVAL_MS } from '$lib/constants';
  import '../app.css';

  let showCrawlerModal = false;
  let pollingInterval: any;
  let isUpdating = false;
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
            
            // Check for updates after downloading
            checkForUpdates();
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
            
            checkForUpdates();
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
          
          checkForUpdates();
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
      // TODO: Store in a store for UI display
    } catch (error) {
      console.warn('No popular repacks in database yet');
    }
  }

  async function checkForUpdates() {
    console.log('Checking for new games...');
    
    // Show modal for update - let CrawlerModal handle the update logic
    isUpdating = true;
    showCrawlerModal = true;
  }

  function startPollingGames() {
    // Poll every 2 seconds to update game list while crawler is running
    pollingInterval = setInterval(async () => {
      await loadGames(LOAD_ALL_GAMES);
    }, POLLING_INTERVAL_MS);
  }

  function stopPollingGames() {
    if (pollingInterval) {
      clearInterval(pollingInterval);
      pollingInterval = null;
    }
  }

  async function onStartCrawl() {
    // Crawler started, start polling
    await loadGames(LOAD_ALL_GAMES);
    startPollingGames();
  }

  async function onCrawlerComplete() {
    stopPollingGames();
    // Reset updating state
    isUpdating = false;
    // Final load of all games and categories
    await loadGames(LOAD_ALL_GAMES);
    await loadCategories();
    
    // Fetch popular repacks from website after crawling
    // This runs after both:
    // 1. Initial database crawl (start_crawler)
    // 2. Database updates (update_database)
    fetchPopularRepacks().catch(err => 
      console.error('Failed to fetch popular repacks:', err)
    );
  }
  
  async function fetchPopularRepacks() {
    try {
      console.log('\n' + '='.repeat(60));
      console.log('🌟 POPULAR REPACKS UPDATE STARTED');
      console.log('='.repeat(60));
      
      // Fetch all three periods: monthly, yearly, and award
      console.log('Step 1/6: Fetching monthly popular repacks from website...');
      const monthCount = await invoke<number>('fetch_popular_repacks', { period: 'month' });
      console.log(`  ✅ Saved ${monthCount} monthly popular repacks`);
      
      console.log('Step 2/6: Fetching yearly popular repacks from website...');
      const yearCount = await invoke<number>('fetch_popular_repacks', { period: 'year' });
      console.log(`  ✅ Saved ${yearCount} yearly popular repacks`);
      
      console.log('Step 3/6: Fetching Pink Paw Award games from website...');
      const awardCount = await invoke<number>('fetch_popular_repacks', { period: 'award' });
      console.log(`  ✅ Saved ${awardCount} Pink Paw Award games`);
      
      console.log('Step 4/6: Crawling full game data for popular games...');
      
      // Set crawling state
      isCrawlingPopular.set(true);
      const totalCount = monthCount + yearCount + awardCount;
      popularCrawlProgress.set({ crawled: 0, total: totalCount });
      
      // Crawl all three periods (this may take a while)
      const monthCrawled = await invoke<number>('crawl_popular_games', { period: 'month' });
      console.log(`  ✅ Crawled ${monthCrawled} new monthly popular games`);
      
      const yearCrawled = await invoke<number>('crawl_popular_games', { period: 'year' });
      console.log(`  ✅ Crawled ${yearCrawled} new yearly popular games`);
      
      const awardCrawled = await invoke<number>('crawl_popular_games', { period: 'award' });
      console.log(`  ✅ Crawled ${awardCrawled} new Pink Paw Award games`);
      
      // Clear crawling state
      isCrawlingPopular.set(false);
      
      console.log('Step 5/6: Reloading popular repacks into UI...');
      await loadPopularRepacks();
      console.log('  ✅ UI updated');
      
      console.log('Step 6/6: Complete');
      console.log('='.repeat(60));
      console.log('🎉 POPULAR REPACKS UPDATE COMPLETED');
      console.log(`   Total: ${monthCrawled + yearCrawled + awardCrawled} games crawled`);
      console.log('='.repeat(60) + '\n');
    } catch (error) {
      console.error('❌ Failed to fetch popular repacks:', error);
      isCrawlingPopular.set(false);
      throw error;
    }
  }

  onMount(async () => {
    // Load saved theme
    loadSavedTheme();

    // Initialize keyboard shortcuts
    initKeyboardShortcuts();

    // Check if database needs initialization and load games
    await checkAndInitializeDatabase();
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
              <VirtualizedGameList />
            </div>
            <div class="view-container" class:hidden={$browseView !== 'details'}>
              <GameDetails />
            </div>
          </div>
        </div>
      </div>
    {:else if $currentPage === 'popular'}
      <Popular />
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
  
  <CrawlerModal 
    bind:isOpen={showCrawlerModal}
    bind:isUpdating={isUpdating}
    totalGames={$games.length}
    onComplete={onCrawlerComplete}
    onStart={onStartCrawl}
  />
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
