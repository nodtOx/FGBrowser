<script lang="ts">
  import { POPULAR_FETCH_LIMIT, POPULAR_REFRESH_INTERVAL_MS } from '$lib/constants';
  import { formatSize, isCrawlingPopular, selectedGame } from '$lib/stores/games';
  import { invoke } from '@tauri-apps/api/core';
  import { onDestroy, onMount } from 'svelte';
  import GameDetails from './GameDetails.svelte';
  import PopularSidebar from './PopularSidebar.svelte';

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

  let popularRepacks: PopularRepackWithGame[] = [];
  let isLoading = true;
  let error = '';
  let refreshInterval: any;
  let selectedPeriod: 'week' | 'today' | 'month' | 'year' | 'award' = 'month';
  let mounted = false;
  let showingDetails = false;
  
  // Track counts for each period
  let weekCount = 0;
  let todayCount = 0;
  let monthCount = 0;
  let yearCount = 0;
  let awardCount = 0;

  async function loadPopularRepacks() {
    try {
      isLoading = true;
      error = '';
      // Fetch all games for the period (no artificial limit)
      popularRepacks = await invoke<PopularRepackWithGame[]>('get_popular_repacks_with_games', { 
        period: selectedPeriod, 
        limit: POPULAR_FETCH_LIMIT 
      });
      console.log(`Loaded ${popularRepacks.length} popular repacks (${selectedPeriod}) with game data`);
      
      // Update the count for the current period
      if (selectedPeriod === 'week') weekCount = popularRepacks.length;
      else if (selectedPeriod === 'today') todayCount = popularRepacks.length;
      else if (selectedPeriod === 'month') monthCount = popularRepacks.length;
      else if (selectedPeriod === 'year') yearCount = popularRepacks.length;
      else if (selectedPeriod === 'award') awardCount = popularRepacks.length;
    } catch (err) {
      console.error('Failed to load popular repacks:', err);
      error = 'Failed to load popular repacks. Try running the crawler first.';
      popularRepacks = [];
    } finally {
      isLoading = false;
    }
  }
  
  // Reactive statement: reload when period changes (but not on initial mount)
  $: if (mounted && selectedPeriod) {
    loadPopularRepacks();
    // Mark as viewed when switching periods
    markAsViewed();
  }

  async function markAsViewed() {
    try {
      await invoke('mark_popular_as_viewed', { period: selectedPeriod });
      console.log(`Marked ${selectedPeriod} as viewed`);
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
    // Refresh periodically while crawling to show new games
    refreshInterval = setInterval(async () => {
      if ($isCrawlingPopular) {
        await loadPopularRepacks();
      }
    }, POPULAR_REFRESH_INTERVAL_MS);
  }
  
  function stopAutoRefresh() {
    if (refreshInterval) {
      clearInterval(refreshInterval);
      refreshInterval = null;
    }
  }
  
  // Watch for crawling state changes
  $: if ($isCrawlingPopular) {
    startAutoRefresh();
  } else {
    stopAutoRefresh();
  }

  onMount(async () => {
    // Load all counts on mount (fetch all games for each period)
    try {
      const weeklyRepacks = await invoke<PopularRepackWithGame[]>('get_popular_repacks_with_games', { 
        period: 'week', 
        limit: POPULAR_FETCH_LIMIT 
      });
      weekCount = weeklyRepacks.length;
      
      const todayRepacks = await invoke<PopularRepackWithGame[]>('get_popular_repacks_with_games', { 
        period: 'today', 
        limit: POPULAR_FETCH_LIMIT 
      });
      todayCount = todayRepacks.length;
      
      const monthlyRepacks = await invoke<PopularRepackWithGame[]>('get_popular_repacks_with_games', { 
        period: 'month', 
        limit: POPULAR_FETCH_LIMIT 
      });
      monthCount = monthlyRepacks.length;
      
      const yearlyRepacks = await invoke<PopularRepackWithGame[]>('get_popular_repacks_with_games', { 
        period: 'year', 
        limit: POPULAR_FETCH_LIMIT 
      });
      yearCount = yearlyRepacks.length;
      
      const awardRepacks = await invoke<PopularRepackWithGame[]>('get_popular_repacks_with_games', { 
        period: 'award', 
        limit: POPULAR_FETCH_LIMIT 
      });
      awardCount = awardRepacks.length;
      
      console.log(`Loaded counts: week=${weekCount}, today=${todayCount}, month=${monthCount}, year=${yearCount}, award=${awardCount}`);
    } catch (err) {
      console.warn('Failed to load all counts:', err);
    }
    
    // Load the selected period's data
    await loadPopularRepacks();
    
    // Now enable the reactive statement for future period changes
    mounted = true;
  });
  
  onDestroy(() => {
    stopAutoRefresh();
  });
</script>

<div class="popular-page">
  <div class="popular-layout">
    <PopularSidebar 
      bind:selectedPeriod 
      weeklyCount={weekCount}
      todayCount={todayCount}
      monthlyCount={monthCount}
      yearlyCount={yearCount}
      awardCount={awardCount}
    />

    <div class="popular-content-area">
      <div class="view-container" class:hidden={showingDetails}>
        <div class="popular-main">
      <div class="popular-header">
        <h1>
          {selectedPeriod === 'week' ? 'Most Popular Repacks of the Week' : 
           selectedPeriod === 'today' ? "Today's Most Popular Repacks" :
           selectedPeriod === 'month' ? 'Most Popular Repacks of the Month' : 
           selectedPeriod === 'year' ? 'Most Popular Repacks of the Year' : 
           'Games with Pink Paw Award'}
        </h1>
        <p class="subtitle">
          {selectedPeriod === 'award' ? 
            `${popularRepacks.length} games with exceptional audio/graphics/gameplay design` :
            `Top ${popularRepacks.length} most downloaded games`}
        </p>
      </div>

  {#if $isCrawlingPopular}
    <div class="crawling-banner">
      <div class="crawling-content">
        <div class="spinner"></div>
        <div class="crawling-text">
          <strong>Crawling popular games...</strong>
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
      <p>Loading popular repacks...</p>
    </div>
  {:else if error}
    <div class="error">
      <p>{error}</p>
    </div>
  {:else if popularRepacks.length === 0}
    <div class="empty">
      <p>No popular repacks found.</p>
      <p class="hint">Run the crawler to fetch popular games.</p>
    </div>
  {:else}
    <div class="popular-grid">
      {#each popularRepacks as repack (repack.id)}
        <div 
          class="game-card"
          class:linked={repack.game !== null}
          on:click={() => handleGameClick(repack)}
          on:keydown={(e) => e.key === 'Enter' && handleGameClick(repack)}
          role="button"
          tabindex="0"
        >
          <div class="rank-badge">#{repack.rank}</div>
          <div class="game-image">
            {#if repack.image_url}
              <img src={repack.image_url} alt={repack.game?.clean_name || repack.title} loading="lazy" />
            {:else}
              <div class="no-image">No Image</div>
            {/if}
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
  {/if}
        </div>
      </div>

      <div class="view-container" class:hidden={!showingDetails}>
        <GameDetails onBack={handleBackToList} />
      </div>
    </div>
  </div>
</div>

<style>
  .popular-page {
    height: 100%;
    display: flex;
    flex-direction: column;
    background-color: var(--color-background);
  }

  .popular-layout {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .popular-content-area {
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

  .popular-main {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow-y: auto;
  }

  .popular-header {
    padding: 24px;
    border-bottom: 1px solid var(--color-border);
    background-color: var(--color-backgroundSecondary);
  }


  .popular-header h1 {
    margin: 0 0 8px 0;
    font-size: 24px;
    font-weight: 600;
    color: var(--color-text);
  }

  .subtitle {
    margin: 0;
    font-size: 14px;
    color: var(--color-textSecondary);
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

  .popular-grid {
    flex: 1;
    overflow-y: auto;
    padding: 24px;
    display: flex;
    flex-wrap: wrap;
    gap: 20px;
    align-content: flex-start;
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
    border-color: var(--color-primary);
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2);
  }

  .game-card:focus {
    outline: 2px solid var(--color-primary);
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

  .rank-badge {
    position: absolute;
    top: 8px;
    left: 8px;
    background-color: var(--color-primary);
    color: var(--color-selectedText);
    font-size: 12px;
    font-weight: 600;
    padding: 4px 8px;
    border-radius: 2px;
    z-index: 1;
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

  .game-image img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .no-image {
    color: var(--color-textMuted);
    font-size: 12px;
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
    border-bottom: 2px solid var(--color-primary);
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
    border-top-color: var(--color-primary);
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

