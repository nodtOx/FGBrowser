<script lang="ts">
  import { POPULAR_FETCH_LIMIT, POPULAR_REFRESH_INTERVAL_MS } from '$lib/constants';
  import { formatSize, isCrawlingPopular, selectedGame } from '$lib/stores/games';
  import { invoke } from '@tauri-apps/api/core';
  import { onDestroy, onMount } from 'svelte';
  import CachedImage from './CachedImage.svelte';
  import GameDetails from './GameDetails.svelte';

  interface Game {
    id: number;
    title: string;
    clean_name: string | null;
    genres_tags: string | null;
    company: string | null;
    languages: string | null;
    original_size: string | null;
    repack_size: string | null;
    size: number | null;
    url: string;
    date: string | null;
    image_url: string | null;
  }

  interface PopularRepackWithGame {
    id: number;
    url: string;
    title: string;
    image_url: string | null;
    rank: number;
    game: Game | null;
    created_at: string | null;
    is_new: boolean;
  }

  let awardGames: PopularRepackWithGame[] = [];
  let isLoading = true;
  let error = '';
  let refreshInterval: any;
  let mounted = false;
  let showingDetails = false;

  async function loadAwardGames() {
    try {
      isLoading = true;
      error = '';
      awardGames = await invoke<PopularRepackWithGame[]>('get_popular_repacks_with_games', { 
        period: 'award', 
        limit: POPULAR_FETCH_LIMIT 
      });
      console.log(`Loaded ${awardGames.length} Pink Paw Award games with game data`);
    } catch (err) {
      console.error('Failed to load Pink Paw Award games:', err);
      error = 'Failed to load Pink Paw Award games. Try running the crawler first.';
      awardGames = [];
    } finally {
      isLoading = false;
    }
  }

  async function markAsViewed() {
    try {
      await invoke('mark_popular_as_viewed', { period: 'award' });
      console.log('Marked Pink Paw Award as viewed');
    } catch (err) {
      console.error('Failed to mark as viewed:', err);
    }
  }

  async function handleGameClick(repack: PopularRepackWithGame) {
    if (repack.game) {
      try {
        const details = await invoke('get_game_details', { gameId: repack.game.id });
        selectedGame.set(details as any);
        showingDetails = true;
      } catch (err) {
        console.error('Failed to load game details:', err);
      }
    }
  }

  function handleBackToList() {
    showingDetails = false;
    selectedGame.set(null);
  }

  function startAutoRefresh() {
    refreshInterval = setInterval(async () => {
      if ($isCrawlingPopular) {
        await loadAwardGames();
      }
    }, POPULAR_REFRESH_INTERVAL_MS);
  }

  function stopAutoRefresh() {
    if (refreshInterval) {
      clearInterval(refreshInterval);
      refreshInterval = null;
    }
  }

  $: if ($isCrawlingPopular) {
    startAutoRefresh();
  } else {
    stopAutoRefresh();
  }

  onMount(async () => {
    await loadAwardGames();
    await markAsViewed();
    mounted = true;
  });

  onDestroy(() => {
    stopAutoRefresh();
  });
</script>

<div class="pinkpaw-page">
  <div class="pinkpaw-content">
    <div class="view-container" class:hidden={showingDetails}>
      <div class="pinkpaw-main">
        {#if $isCrawlingPopular}
          <div class="crawling-banner">
            <div class="crawling-content">
              <div class="spinner"></div>
              <div class="crawling-text">
                <strong>Crawling Pink Paw Award games...</strong>
                <p class="crawling-status">
                  Fetching full game data from website. This may take a few minutes.
                </p>
                <p class="crawling-hint">The list will update automatically as games are crawled.</p>
              </div>
            </div>
          </div>
        {/if}

        {#if isLoading && !$isCrawlingPopular}
          <div class="loading">
            <p>Loading Pink Paw Award games...</p>
          </div>
        {:else if error}
          <div class="error">
            <p>{error}</p>
          </div>
        {:else if awardGames.length === 0}
          <div class="empty">
            <p>No Pink Paw Award games found.</p>
            <p class="hint">Run the crawler to fetch award-winning games.</p>
          </div>
        {:else}
          <div class="pinkpaw-grid">
            <div class="pinkpaw-alert">
              <h2>Pink Paw Awards</h2>
              <p class="alert-description">
                Games awarded FitGirl's personal Pink Paw for unusual audio or graphics design or non-standard gameplay. 
                When you don't know what to play - try one of those gems.
              </p>
              <p class="alert-subtitle">
                {awardGames.length} games sorted by date of release, from newer to older titles
              </p>
            </div>

            <div class="games-wrapper">
            {#each awardGames as repack (repack.id)}
              <div 
                class="game-card"
                class:linked={repack.game !== null}
                on:click={() => handleGameClick(repack)}
                on:keydown={(e) => e.key === 'Enter' && handleGameClick(repack)}
                role="button"
                tabindex="0"
              >
                <div class="award-badge">Pink Paw</div>
                <div class="game-image">
                  <CachedImage 
                    src={repack.image_url} 
                    alt={repack.game?.clean_name || repack.title} 
                  />
                </div>
                <div class="game-info">
                  <div class="game-title-row">
                    <h3 class="game-title">{repack.game?.clean_name || repack.title}</h3>
                    {#if repack.is_new}
                      <span class="new-badge">NEW</span>
                    {/if}
                  </div>
                  {#if repack.game === null}
                    <span class="not-in-catalog">Not in catalog</span>
                  {:else if repack.game.size}
                    <span class="game-size">{formatSize(repack.game.size)}</span>
                  {/if}
                </div>
              </div>
            {/each}
            </div>
          </div>
        {/if}
      </div>
    </div>

    <div class="view-container" class:hidden={!showingDetails}>
      <GameDetails onBack={handleBackToList} />
    </div>
  </div>
</div>

<style>
  .pinkpaw-page {
    height: 100%;
    display: flex;
    flex-direction: column;
    background-color: var(--color-background);
  }

  .pinkpaw-content {
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

  .pinkpaw-main {
    flex: 1;
    overflow-y: auto;
  }

  .loading,
  .error,
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 8px;
    color: var(--color-textMuted);
  }

  .error {
    color: var(--color-error);
  }

  .hint {
    font-size: 12px;
    color: var(--color-textMuted);
  }

  .pinkpaw-grid {
    padding: 24px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 24px;
  }

  .pinkpaw-alert {
    max-width: 700px;
    width: 100%;
    padding: 20px 24px;
    background-color: rgba(255, 105, 180, 0.1);
    border: 1px solid rgba(255, 105, 180, 0.3);
    border-radius: 6px;
    text-align: center;
  }

  .pinkpaw-alert h2 {
    margin: 0 0 12px 0;
    font-size: 20px;
    font-weight: 600;
    color: #ff69b4;
  }

  .alert-description {
    margin: 0 0 12px 0;
    font-size: 14px;
    line-height: 1.6;
    color: var(--color-text);
  }

  .alert-subtitle {
    margin: 0;
    font-size: 12px;
    color: var(--color-textSecondary);
    font-style: italic;
  }

  .games-wrapper {
    display: flex;
    flex-wrap: wrap;
    gap: 20px;
    justify-content: center;
    width: 100%;
  }

  .game-card {
    width: 150px;
    display: flex;
    flex-direction: column;
    background-color: var(--color-backgroundSecondary);
    border: 1px solid var(--color-border);
    cursor: pointer;
    position: relative;
    overflow: hidden;
  }

  .game-card:hover {
    border-color: #ff69b4;
    box-shadow: 0 4px 8px rgba(255, 105, 180, 0.3);
  }

  .game-card:focus {
    outline: 2px solid #ff69b4;
    outline-offset: 2px;
  }

  .game-card:not(.linked) {
    opacity: 0.6;
    cursor: default;
  }

  .game-card:not(.linked):hover {
    transform: none;
    border-color: var(--color-border);
    box-shadow: none;
  }

  .award-badge {
    position: absolute;
    top: 8px;
    left: 8px;
    background-color: #ff69b4;
    color: white;
    font-size: 11px;
    font-weight: 700;
    padding: 4px 8px;
    border-radius: 2px;
    z-index: 1;
    letter-spacing: 0.5px;
  }

  .new-badge {
    background-color: #10b981;
    color: white;
    font-size: 9px;
    font-weight: 700;
    padding: 2px 5px;
    border-radius: 2px;
    letter-spacing: 0.5px;
    white-space: nowrap;
    flex-shrink: 0;
  }

  .game-image {
    width: 150px;
    height: 200px;
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: var(--color-backgroundTertiary);
  }


  .game-info {
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .game-title-row {
    display: flex;
    align-items: flex-start;
    gap: 6px;
    width: 100%;
  }

  .game-title {
    margin: 0;
    font-size: 12px;
    font-weight: 500;
    color: var(--color-text);
    line-height: 1.3;
    display: -webkit-box;
    -webkit-line-clamp: 3;
    line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
    min-width: 0;
  }

  .not-in-catalog {
    font-size: 10px;
    color: var(--color-textMuted);
    font-style: italic;
  }

  .game-size {
    font-size: 10px;
    color: var(--color-primary);
    font-weight: 600;
  }

  .crawling-banner {
    background-color: var(--color-backgroundSecondary);
    border-bottom: 2px solid #ff69b4;
    padding: 16px 24px;
    animation: pulse 2s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% {
      background-color: var(--color-backgroundSecondary);
    }
    50% {
      background-color: var(--color-backgroundTertiary);
    }
  }

  .crawling-content {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .spinner {
    width: 32px;
    height: 32px;
    border: 3px solid var(--color-border);
    border-top-color: #ff69b4;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .crawling-text {
    flex: 1;
  }

  .crawling-text strong {
    color: var(--color-text);
    font-size: 14px;
  }

  .crawling-status {
    margin: 4px 0 0 0;
    font-size: 12px;
    color: var(--color-textSecondary);
  }

  .crawling-hint {
    margin: 4px 0 0 0;
    font-size: 11px;
    color: var(--color-textMuted);
    font-style: italic;
  }
</style>

