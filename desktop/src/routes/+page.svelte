<script lang="ts">
  import { games, loadGames } from '$lib/stores/games';
  import { initKeyboardShortcuts } from '$lib/stores/keyboard';
  import { currentPage } from '$lib/stores/navigation';
  import { loadSavedTheme } from '$lib/stores/theme';
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  import CrawlerModal from '$lib/components/CrawlerModal.svelte';
  import GameList from '$lib/components/GameList.svelte';
  import Header from '$lib/components/Header.svelte';
  import Settings from '$lib/components/Settings.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import StatusBar from '$lib/components/StatusBar.svelte';

  import '../app.css';

  let showCrawlerModal = false;
  let pollingInterval: any;

  interface UpdateResult {
    total_games: number;
    status: string;
  }

  let isUpdating = false;

  async function checkAndInitializeDatabase() {
    try {
      const isEmpty = await invoke('is_database_empty');
      
      if (isEmpty) {
        // Database is empty, show modal and initialize
        showCrawlerModal = true;
      } else {
        // Database has data, check for updates
        await loadGames(100);
        checkForUpdates();
      }
    } catch (error) {
      console.error('Error checking database:', error);
      // Try to load games anyway
      await loadGames(100);
    }
  }

  async function checkForUpdates() {
    try {
      console.log('Checking for new games...');
      
      // Show modal for update
      isUpdating = true;
      showCrawlerModal = true;
      onStartCrawl(); // Start polling
      
      const result = await invoke<UpdateResult>('update_database');
      
      if (result.total_games > 0) {
        console.log(`Updated database with ${result.total_games} new games`);
      } else {
        console.log('Database is up to date');
      }
      
      // Close modal after a brief delay
      setTimeout(() => {
        showCrawlerModal = false;
        isUpdating = false;
        stopPollingGames();
        loadGames(100);
      }, 1500);
    } catch (error) {
      console.error('Error updating database:', error);
      showCrawlerModal = false;
      isUpdating = false;
      stopPollingGames();
    }
  }

  function startPollingGames() {
    // Poll every 2 seconds to update game list while crawler is running
    pollingInterval = setInterval(async () => {
      await loadGames(100);
    }, 2000);
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
    // Final load of all games
    await loadGames(100);
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
            <GameList />
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
