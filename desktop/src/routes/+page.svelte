<script lang="ts">
  import { games, isCrawlingPopular, loadCategories, loadGames, popularCrawlProgress, totalGamesCount } from '$lib/stores/games';
  import { initKeyboardShortcuts } from '$lib/stores/keyboard';
  import { currentPage } from '$lib/stores/navigation';
  import { loadSavedTheme } from '$lib/stores/theme';
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  import CrawlerModal from '$lib/components/CrawlerModal.svelte';
  import GameDetailsModal from '$lib/components/GameDetailsModal.svelte';
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

  async function checkAndInitializeDatabase() {
    try {
      const isEmpty = await invoke('is_database_empty');
      
      if (isEmpty) {
        // Database is empty, show modal and initialize
        showCrawlerModal = true;
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
    } catch (error) {
      console.error('Error checking database:', error);
      // Try to load games anyway
      await loadGames(LOAD_ALL_GAMES);
    }
  }
  
  async function loadPopularRepacks() {
    try {
      // Load both month and year popular repacks from database
      const monthlyRepacks = await invoke<any[]>('get_popular_repacks', { period: 'month', limit: 50 });
      const yearlyRepacks = await invoke<any[]>('get_popular_repacks', { period: 'year', limit: 150 });
      console.log(`Loaded ${monthlyRepacks.length} monthly + ${yearlyRepacks.length} yearly popular repacks from database`);
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
      
      // Fetch both monthly and yearly
      console.log('Step 1/4: Fetching monthly popular repacks from website...');
      const monthCount = await invoke<number>('fetch_popular_repacks', { period: 'month' });
      console.log(`  ✅ Saved ${monthCount} monthly popular repacks`);
      
      console.log('Step 2/4: Fetching yearly popular repacks from website...');
      const yearCount = await invoke<number>('fetch_popular_repacks', { period: 'year' });
      console.log(`  ✅ Saved ${yearCount} yearly popular repacks`);
      
      console.log('Step 3/4: Crawling full game data for popular games...');
      
      // Set crawling state
      isCrawlingPopular.set(true);
      const totalCount = monthCount + yearCount;
      popularCrawlProgress.set({ crawled: 0, total: totalCount });
      
      // Crawl both periods (this may take a while)
      const monthCrawled = await invoke<number>('crawl_popular_games', { period: 'month' });
      console.log(`  ✅ Crawled ${monthCrawled} new monthly popular games`);
      
      const yearCrawled = await invoke<number>('crawl_popular_games', { period: 'year' });
      console.log(`  ✅ Crawled ${yearCrawled} new yearly popular games`);
      
      // Clear crawling state
      isCrawlingPopular.set(false);
      
      console.log('Step 4/4: Reloading popular repacks into UI...');
      await loadPopularRepacks();
      console.log('  ✅ UI updated');
      
      console.log('='.repeat(60));
      console.log('🎉 POPULAR REPACKS UPDATE COMPLETED');
      console.log(`   Total: ${monthCrawled + yearCrawled} games crawled`);
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
    {#if $currentPage === 'browse'}
      <div class="browse-page">
        <div class="browse-layout">
          <Sidebar totalGames={$totalGamesCount} />

          <div class="game-list-container">
            <VirtualizedGameList />
          </div>
        </div>
      </div>
    {:else if $currentPage === 'popular'}
      <Popular />
    {:else if $currentPage === 'downloads'}
      <div class="page-placeholder">
        <h2>Downloads Page</h2>
        <p>Torrent client integration coming soon...</p>
      </div>
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
  
  <GameDetailsModal />
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

  .game-list-container {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
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
</style>
