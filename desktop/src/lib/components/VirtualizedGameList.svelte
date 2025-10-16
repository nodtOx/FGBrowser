<script lang="ts">
    import { ITEM_HEIGHT, OVERSCAN, SEARCH_DEBOUNCE_MS } from '$lib/constants';
    import { formatSize, games, searchGames, searchQuery, selectedIndex, selectGame } from '$lib/stores/games';
    import { focusedPanel, openGameDetails } from '$lib/stores/navigation';
    import { onMount, tick } from 'svelte';
    
    let containerHeight: number = 0;
    let scrollTop: number = 0;
    let containerElement: HTMLElement;
    let searchInput: HTMLInputElement;
    let searchTimeout: any;
    let previousQuery: string = '';
    
    // Calculated values
    $: visibleCount = Math.ceil(containerHeight / ITEM_HEIGHT);
    $: startIndex = Math.max(0, Math.floor(scrollTop / ITEM_HEIGHT) - OVERSCAN);
    $: endIndex = Math.min($games.length, startIndex + visibleCount + 2 * OVERSCAN);
    $: visibleGames = $games.slice(startIndex, endIndex);
    $: totalHeight = $games.length * ITEM_HEIGHT;
    $: offsetY = startIndex * ITEM_HEIGHT;
    
    function formatDate(date: string | null): string {
        if (!date) return 'N/A';
        try {
            const dateObj = new Date(date);
            return dateObj.toLocaleDateString('en-US', { 
                month: '2-digit', 
                day: '2-digit',
                year: '2-digit'
            });
        } catch {
            return date.slice(0, 10); // Fallback to first 10 chars
        }
    }
    
    async function handleGameClick(gameIndex: number, event: MouseEvent) {
        focusedPanel.set('gamelist');
        await selectGame(startIndex + gameIndex);
        // Remove focus from clicked element to prevent residual focus state
        (event.currentTarget as HTMLElement).blur();
    }
    
    async function handleGameDoubleClick(gameIndex: number, event: MouseEvent) {
        await selectGame(startIndex + gameIndex);
        openGameDetails();
        // Remove focus from clicked element to prevent residual focus state
        (event.currentTarget as HTMLElement).blur();
    }
    
    function handleKeydown(e: KeyboardEvent, gameIndex: number) {
        // Keyboard shortcuts are now handled globally in keyboard.ts
        // This handler is kept for accessibility but doesn't preventDefault
        // to allow global handlers to work
    }
    
    // Scroll to selected item when selection changes externally
    $: if (containerElement && $selectedIndex >= 0) {
        scrollToIndex($selectedIndex);
    }
    
    function scrollToIndex(index: number) {
        if (!containerElement) return;
        
        const itemTop = index * ITEM_HEIGHT;
        const itemBottom = (index + 1) * ITEM_HEIGHT;
        const viewTop = scrollTop;
        const viewBottom = scrollTop + containerHeight;
        
        if (itemTop < viewTop) {
            containerElement.scrollTop = itemTop - ITEM_HEIGHT;
        } else if (itemBottom > viewBottom) {
            containerElement.scrollTop = itemBottom - containerHeight + ITEM_HEIGHT;
        }
    }
    
    function handleScroll(event: Event) {
        const target = event.target as HTMLElement;
        scrollTop = target.scrollTop;
    }
    
    function handleSearch() {
        // Clear previous timeout
        if (searchTimeout) {
            clearTimeout(searchTimeout);
        }
        
        const currentQuery = $searchQuery.trim();
        
        // Debounce search
        searchTimeout = setTimeout(async () => {
            // Only search if query has actually changed
            if (currentQuery !== previousQuery) {
                previousQuery = currentQuery;
                await searchGames(currentQuery);
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
    
    // Auto-focus/blur search input based on focused panel
    $: if (searchInput) {
        if ($focusedPanel === 'search') {
            searchInput.focus();
        } else {
            searchInput.blur();
        }
    }
    
    onMount(() => {
        // Scroll to selected item on mount
        if ($selectedIndex >= 0) {
            tick().then(() => scrollToIndex($selectedIndex));
        }
    });
</script>

<div class="list-container">
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
    </div>
    
    <div 
        class="virtualized-list"
        class:focused={$focusedPanel === 'gamelist'}
        bind:this={containerElement}
        bind:clientHeight={containerHeight}
        on:scroll={handleScroll}
    >
    <div class="scroll-area" style="height: {totalHeight}px;">
        <div class="visible-items" style="transform: translateY({offsetY}px);">
            {#each visibleGames as game, index (game.id)}
                {@const globalIndex = startIndex + index}
                <div 
                    class="game-item"
                    class:selected={globalIndex === $selectedIndex}
                    class:focused-panel={$focusedPanel === 'gamelist'}
                    on:click={(e) => handleGameClick(index, e)}
                    on:dblclick={(e) => handleGameDoubleClick(index, e)}
                    on:keydown={(e) => handleKeydown(e, index)}
                    role="button"
                    tabindex={globalIndex === $selectedIndex ? 0 : -1}
                    style="height: {ITEM_HEIGHT}px;"
                >
                    <div class="game-date">
                        {formatDate(game.date)}
                    </div>
                    <div class="game-title">
                        {game.title}
                    </div>
                    <div class="game-size">
                        {formatSize(game.size)}
                    </div>
                </div>
            {/each}
        </div>
    </div>
    </div>
</div>

<style>
    .list-container {
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
    }
    
    .search-bar.focused {
        border-left-color: var(--color-primary);
        background-color: rgba(var(--color-primary-rgb, 136, 192, 208), 0.05);
    }
    
    .search-input {
        width: 100%;
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
    
    .virtualized-list {
        flex: 1;
        overflow-y: auto;
        overflow-x: hidden;
        font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
        font-size: calc(var(--base-font-size) * 1);
        position: relative;
        border-left: 3px solid transparent;
        transition: all 0.15s ease;
    }
    
    .virtualized-list.focused {
        border-left-color: var(--color-primary);
        background-color: rgba(var(--color-primary-rgb, 136, 192, 208), 0.02);
    }
    
    .scroll-area {
        position: relative;
    }
    
    .visible-items {
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
    }
    
    .game-item {
        display: grid;
        grid-template-columns: auto 1fr auto;
        align-items: center;
        padding: 3px 12px;
        gap: 16px;
        cursor: pointer;
        border-bottom: 1px solid transparent;
        box-sizing: border-box;
        outline: none; /* Remove browser focus outline */
    }
    
    .game-item:focus {
        outline: none; /* Ensure no focus outline on click */
    }
    
    .game-item:hover {
        background-color: var(--color-hover);
    }
    
    .game-item.selected {
        background-color: var(--color-primary);
        color: var(--color-selectedText);
        opacity: 0.4;
    }
    
    .game-item.selected.focused-panel {
        opacity: 1;
    }
    
    .game-date {
        font-size: calc(var(--base-font-size) * 0.875);
        color: var(--color-textSecondary);
        font-weight: 500;
        text-align: left;
        white-space: nowrap;
        padding-right: 8px;
    }
    
    .game-item.selected.focused-panel .game-date {
        color: var(--color-selectedText);
    }
    
    .game-title {
        font-size: calc(var(--base-font-size) * 1);
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        padding: 0;
        min-width: 0;
    }
    
    .game-size {
        font-size: calc(var(--base-font-size) * 0.875);
        color: var(--color-textSecondary);
        text-align: right;
        font-weight: 500;
        white-space: nowrap;
        padding-left: 8px;
    }
    
    .game-item.selected.focused-panel .game-size {
        color: var(--color-selectedText);
    }
</style>
