<script lang="ts">
    import { formatSize, games, selectedIndex, selectGame } from '$lib/stores/games';
    import { onMount, tick } from 'svelte';
    
    // Virtualization parameters
    const ITEM_HEIGHT = 30; // Height of each game item in pixels
    const OVERSCAN = 5; // Extra items to render outside visible area for smooth scrolling
    
    let containerHeight: number = 0;
    let scrollTop: number = 0;
    let containerElement: HTMLElement;
    
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
    
    async function handleGameClick(gameIndex: number) {
        await selectGame(startIndex + gameIndex);
    }
    
    function handleKeydown(e: KeyboardEvent, gameIndex: number) {
        if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            handleGameClick(gameIndex);
        }
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
    
    onMount(() => {
        // Scroll to selected item on mount
        if ($selectedIndex >= 0) {
            tick().then(() => scrollToIndex($selectedIndex));
        }
    });
</script>

<div 
    class="virtualized-list" 
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
                    on:click={() => handleGameClick(index)}
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

<style>
    .virtualized-list {
        flex: 1;
        overflow-y: auto;
        overflow-x: hidden;
        font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
        font-size: calc(var(--base-font-size) * 1);
        position: relative;
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
    }
    
    .game-item:hover {
        background-color: var(--color-hover);
    }
    
    .game-item.selected {
        background-color: var(--color-primary);
        color: var(--color-selectedText);
    }
    
    .game-date {
        font-size: calc(var(--base-font-size) * 0.875);
        color: var(--color-textSecondary);
        font-weight: 500;
        text-align: left;
        white-space: nowrap;
        padding-right: 8px;
    }
    
    .game-item.selected .game-date {
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
    
    .game-item.selected .game-size {
        color: var(--color-selectedText);
    }
</style>
