<script lang="ts">
  import { games, loadCategories, loadGames } from '$lib/stores/games';
  import { initKeyboardShortcuts } from '$lib/stores/keyboard';
  import { currentPage } from '$lib/stores/navigation';
  import { loadSavedTheme } from '$lib/stores/theme';
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  import CrawlerModal from '$lib/components/CrawlerModal.svelte';
  import GameDetailsModal from '$lib/components/GameDetailsModal.svelte';
  import Header from '$lib/components/Header.svelte';
  import Settings from '$lib/components/Settings.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import StatusBar from '$lib/components/StatusBar.svelte';
  import VirtualizedGameList from '$lib/components/VirtualizedGameList.svelte';

  import '../app.css';

  // Configuration
  const GAMES_LOAD_LIMIT = 100; // Number of games to load at once
  const POLLING_INTERVAL_MS = 2000; // How often to poll for new games during crawling

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
        await loadGames(GAMES_LOAD_LIMIT);
        await loadCategories();
        checkForUpdates();
      }
    } catch (error) {
      console.error('Error checking database:', error);
      // Try to load games anyway
      await loadGames(GAMES_LOAD_LIMIT);
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
      await loadGames(GAMES_LOAD_LIMIT);
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
    await loadGames(100);
    startPollingGames();
  }

  async function onCrawlerComplete() {
    stopPollingGames();
    // Reset updating state
    isUpdating = false;
    // Final load of all games and categories
    await loadGames(100);
    await loadCategories();
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
          <Sidebar totalGames={$games.length} />

          <div class="game-list-container">
            <VirtualizedGameList />
          </div>
        </div>
      </div>
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
