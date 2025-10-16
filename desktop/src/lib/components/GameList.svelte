<script lang="ts">
    import { SEARCH_DEBOUNCE_MS } from '$lib/constants';
    import { formatSize, games, searchGames, searchQuery, selectedIndex, selectGame } from '$lib/stores/games';
    import { focusedPanel } from '$lib/stores/navigation';
    
    let searchInput: HTMLInputElement;
    let searchTimeout: any;
    let previousQuery: string = '';
    
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
    
    async function handleGameClick(index: number, event: MouseEvent) {
        await selectGame(index);
        // Remove focus from clicked element to prevent residual focus state
        (event.currentTarget as HTMLElement).blur();
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
    
    function handleKeydown(e: KeyboardEvent) {
        if (e.key === 'Escape') {
            searchQuery.set('');
            previousQuery = '';
            searchGames('');
            searchInput.blur();
        }
    }
</script>

<div class="game-list-container">
    <div class="search-bar">
        <input
            bind:this={searchInput}
            bind:value={$searchQuery}
            on:input={handleSearch}
            on:keydown={handleKeydown}
            type="search"
            placeholder="Search games... (press / to focus, Esc to clear)"
            class="search-input"
        />
    </div>
    <div class="game-list">
    {#each $games as game, index (game.id)}
        <div 
            class="game-item"
            class:selected={index === $selectedIndex}
            class:focused-panel={$focusedPanel === 'gamelist'}
            on:click={(e) => handleGameClick(index, e)}
            on:keydown={(e) => {
                // Keyboard shortcuts are now handled globally in keyboard.ts
            }}
            role="button"
            tabindex={index === $selectedIndex ? 0 : -1}
        >
            <div class="game-date">
                {formatDate(game.date)}
            </div>
            <div class="game-title">
                {game.clean_name || game.title}
            </div>
            <div class="game-size">
                {formatSize(game.size)}
            </div>
        </div>
    {/each}
    </div>
</div>

<style>
    .game-list-container {
        flex: 1;
        display: flex;
        flex-direction: column;
        overflow: hidden;
    }
    
    .search-bar {
        padding: 8px 12px;
        border-bottom: 1px solid var(--color-border);
        background-color: var(--color-backgroundSecondary);
    }
    
    .search-input {
        width: 100%;
        padding: 6px 12px;
        background-color: var(--color-backgroundSecondary);
        /* border: 1px solid var(--color-border); */
        color: var(--color-text);
        font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
        font-size: calc(var(--base-font-size) * 0.95);
        outline: none;
        transition: var(--transition);
    }
    
    .search-input:focus {
        /* border-color: var(--color-primary); */
        background-color: var(--color-background);
    }
    
    .search-input::placeholder {
        color: var(--color-textSecondary);
        opacity: 0.6;
    }
    
    .game-list {
        flex: 1;
        overflow-y: auto;
        overflow-x: hidden;
        font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
        font-size: calc(var(--base-font-size) * 1);
    }
    
    .game-item {
        display: grid;
        grid-template-columns: auto 1fr auto;
        align-items: center;
        height: 24px;
        padding: 3px 12px;
        gap: 16px;
        cursor: pointer;
        border-bottom: 1px solid transparent;
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

