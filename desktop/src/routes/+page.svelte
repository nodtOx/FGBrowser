<script lang="ts">
  import { onMount } from 'svelte';
  import { currentPage } from '$lib/stores/navigation';
  import { loadGames, games } from '$lib/stores/games';
  import { loadSavedTheme } from '$lib/stores/theme';
  import { initKeyboardShortcuts } from '$lib/stores/keyboard';

  import Header from '$lib/components/Header.svelte';
  import StatusBar from '$lib/components/StatusBar.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import GameList from '$lib/components/GameList.svelte';
  import Settings from '$lib/components/Settings.svelte';

  import '../app.css';

  onMount(async () => {
    // Load saved theme
    loadSavedTheme();

    // Initialize keyboard shortcuts
    initKeyboardShortcuts();

    // Load games from database
    await loadGames(100);
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
