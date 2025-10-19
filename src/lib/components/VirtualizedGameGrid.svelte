<script lang="ts">
  import {
    formatSize,
    games,
    searchGames,
    searchQuery,
    selectedIndex,
    newGamesCount,
    markAllGamesAsSeen,
  } from '$lib/stores/games';
  import { focusedPanel, gameListViewMode, openGameDetails, setGameListViewMode } from '$lib/stores/navigation';
  import { onMount, tick } from 'svelte';
  import CachedImage from './CachedImage.svelte';

  const CARD_WIDTH = 170;
  const CARD_HEIGHT = 280;
  const GAP = 16;
  const PADDING = 16;
  const SEARCH_DEBOUNCE_MS = 300;

  let containerElement: HTMLElement;
  let containerHeight = 0;
  let containerWidth = 0;
  let scrollTop = 0;
  let searchInput: HTMLInputElement;
  let searchTimeout: any;
  let previousQuery = '';

  $: columnsPerRow = Math.max(1, Math.floor((containerWidth - PADDING * 2 + GAP) / (CARD_WIDTH + GAP)));
  $: totalRows = Math.ceil($games.length / columnsPerRow);
  $: totalHeight = totalRows * (CARD_HEIGHT + GAP) + PADDING * 2;

  $: startRow = Math.max(0, Math.floor((scrollTop - PADDING) / (CARD_HEIGHT + GAP)) - 1);
  $: endRow = Math.min(totalRows, Math.ceil((scrollTop + containerHeight - PADDING) / (CARD_HEIGHT + GAP)) + 1);
  $: visibleRows = endRow - startRow;

  $: startIndex = startRow * columnsPerRow;
  $: endIndex = Math.min($games.length, endRow * columnsPerRow);
  $: visibleGames = $games.slice(startIndex, endIndex);

  $: offsetY = startRow * (CARD_HEIGHT + GAP) + PADDING;

  function handleScroll(e: Event) {
    const target = e.target as HTMLElement;
    scrollTop = target.scrollTop;
  }

  function handleGameClick(localIndex: number) {
    const globalIndex = startIndex + localIndex;
    selectedIndex.set(globalIndex);
    openGameDetails();
  }

  function handleSearch() {
    if (searchTimeout) {
      clearTimeout(searchTimeout);
    }

    searchTimeout = setTimeout(() => {
      const query = $searchQuery.trim();
      if (query !== previousQuery) {
        previousQuery = query;
        searchGames(query);
      }
    }, SEARCH_DEBOUNCE_MS);
  }

  function handleSearchKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      searchQuery.set('');
      previousQuery = '';
      searchGames('');
      searchInput.blur();
    }
  }

  function formatDate(dateStr: string | null): string {
    if (!dateStr) return 'N/A';
    const date = new Date(dateStr);
    return date.toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    });
  }

  $: if (searchInput) {
    if ($focusedPanel === 'search') {
      searchInput.focus();
    } else {
      searchInput.blur();
    }
  }

  onMount(() => {
    if ($selectedIndex >= 0) {
      tick().then(() => {
        const row = Math.floor($selectedIndex / columnsPerRow);
        const targetScrollTop = row * (CARD_HEIGHT + GAP);
        if (containerElement) {
          containerElement.scrollTop = targetScrollTop;
        }
      });
    }
  });
</script>

<div class="grid-container">
  <div class="search-bar" class:focused={$focusedPanel === 'search'}>
    <input
      bind:this={searchInput}
      bind:value={$searchQuery}
      on:input={handleSearch}
      on:keydown={handleSearchKeydown}
      on:focus={() => focusedPanel.set('search')}
      type="search"
      placeholder="Search games... (press / to focus, Esc to clear)"
      class="search-input"
    />
    <div class="view-toggles">
      <button
        class="view-toggle-btn"
        class:active={$gameListViewMode === 'list'}
        title="List View"
        on:click={() => setGameListViewMode('list')}
      >
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path d="M2 3H14M2 8H14M2 13H14" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
        </svg>
      </button>
      <button
        class="view-toggle-btn"
        class:active={$gameListViewMode === 'grid'}
        title="Grid View"
        on:click={() => setGameListViewMode('grid')}
      >
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
          <rect x="2" y="2" width="5" height="5" stroke="currentColor" stroke-width="1.5" />
          <rect x="9" y="2" width="5" height="5" stroke="currentColor" stroke-width="1.5" />
          <rect x="2" y="9" width="5" height="5" stroke="currentColor" stroke-width="1.5" />
          <rect x="9" y="9" width="5" height="5" stroke="currentColor" stroke-width="1.5" />
        </svg>
      </button>
    </div>
    {#if $newGamesCount > 0}
      <button class="mark-seen-btn" on:click={markAllGamesAsSeen} title="Mark all games as seen">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path
            d="M13 3L6 10L3 7"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
        Mark All as Seen ({$newGamesCount})
      </button>
    {/if}
  </div>

  <div
    class="virtualized-grid"
    bind:this={containerElement}
    bind:clientHeight={containerHeight}
    bind:clientWidth={containerWidth}
    on:scroll={handleScroll}
  >
    <div class="scroll-area" style="height: {totalHeight}px;">
      <div class="visible-items" style="transform: translateY({offsetY}px);">
        <div
          class="grid-wrapper"
          style="grid-template-columns: repeat({columnsPerRow}, {CARD_WIDTH}px); gap: {GAP}px; padding: 0 {PADDING}px;"
        >
          {#each visibleGames as game, index (game.id)}
            {@const globalIndex = startIndex + index}
            <div
              class="game-card"
              class:selected={globalIndex === $selectedIndex}
              on:click={() => handleGameClick(index)}
              on:keydown={(e) => e.key === 'Enter' && handleGameClick(index)}
              role="button"
              tabindex={globalIndex === $selectedIndex ? 0 : -1}
            >
              <div class="game-image">
                <CachedImage src={game.image_url} alt={game.clean_name || game.title} />
              </div>
              <div class="game-info">
                <h3 class="game-title">
                  {game.clean_name || game.title}
                  {#if game.is_new}
                    <span class="new-badge">NEW</span>
                  {/if}
                </h3>
                <div class="game-meta">
                  <span class="game-date">{formatDate(game.date)}</span>
                  <span class="game-size">{formatSize(game.size)}</span>
                </div>
              </div>
            </div>
          {/each}
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .grid-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .search-bar {
    padding: 8px 12px;
    border-bottom: 1px solid var(--color-border);
    background-color: var(--color-backgroundSecondary);
    border-left: 3px solid transparent;
    transition: all 0.15s ease;
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .search-bar.focused {
    border-left-color: var(--color-primary);
    background-color: rgba(var(--color-primary-rgb, 136, 192, 208), 0.05);
  }

  .search-input {
    flex: 1;
    padding: 6px 12px;
    background-color: var(--color-backgroundSecondary);
    border: 1px solid var(--color-border);
    color: var(--color-text);
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
    font-size: calc(var(--base-font-size) * 0.95);
    outline: none;
    transition: var(--transition);
  }

  .search-input:focus {
    border-color: var(--color-primary);
    background-color: var(--color-background);
  }

  .search-input::placeholder {
    color: var(--color-textSecondary);
    opacity: 0.6;
  }

  .view-toggles {
    display: flex;
    gap: 4px;
    flex-shrink: 0;
  }

  .view-toggle-btn {
    padding: 6px 8px;
    background-color: var(--color-backgroundSecondary);
    border: 1px solid var(--color-border);
    color: var(--color-textSecondary);
    cursor: pointer;
    transition: var(--transition);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .view-toggle-btn:hover {
    background-color: var(--color-hover);
    color: var(--color-text);
    border-color: var(--color-primary);
  }

  .view-toggle-btn.active {
    background-color: var(--color-primary);
    color: var(--color-selectedText);
    border-color: var(--color-primary);
  }

  .view-toggle-btn svg {
    display: block;
  }

  .mark-seen-btn {
    padding: 6px 12px;
    background-color: var(--color-backgroundSecondary);
    border: 1px solid var(--color-border);
    color: var(--color-text);
    cursor: pointer;
    transition: var(--transition);
    display: flex;
    align-items: center;
    gap: 6px;
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
    font-size: calc(var(--base-font-size) * 0.85);
    white-space: nowrap;
  }

  .mark-seen-btn:hover {
    background-color: var(--color-hover);
    border-color: var(--color-primary);
  }

  .mark-seen-btn svg {
    display: block;
    flex-shrink: 0;
  }

  .virtualized-grid {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    background-color: var(--color-background);
  }

  .scroll-area {
    position: relative;
    width: 100%;
  }

  .visible-items {
    position: absolute;
    width: 100%;
    will-change: transform;
  }

  .grid-wrapper {
    display: grid;
    justify-content: center;
  }

  .game-card {
    width: 170px;
    height: 280px;
    display: flex;
    flex-direction: column;
    background-color: var(--color-backgroundSecondary);
    border: 1px solid var(--color-border);
    cursor: pointer;
    transition: var(--transition);
    overflow: hidden;
    position: relative;
  }

  .new-badge {
    display: inline-block;
    margin-left: 4px;
    padding: 2px 4px;
    background-color: #10b981;
    color: white;
    font-size: 8px;
    font-weight: 600;
    border-radius: 2px;
    letter-spacing: 0.3px;
    vertical-align: middle;
  }

  .game-card:hover {
    border-color: var(--color-primary);
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2);
    transform: translateY(-2px);
  }

  .game-card.selected {
    border-color: var(--color-primary);
    box-shadow: 0 0 0 2px var(--color-primary);
  }

  .game-card:focus {
    outline: 2px solid var(--color-primary);
    outline-offset: 2px;
  }

  .game-image {
    width: 100%;
    height: 200px;
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: var(--color-backgroundTertiary);
  }

  .game-info {
    flex: 1;
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .game-title {
    margin: 0;
    font-size: 12px;
    font-weight: 500;
    color: var(--color-text);
    line-height: 1.3;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
  }

  .game-meta {
    display: flex;
    justify-content: space-between;
    gap: 8px;
    font-size: 10px;
  }

  .game-date {
    color: var(--color-textSecondary);
  }

  .game-size {
    color: var(--color-primary);
    font-weight: 600;
  }
</style>
