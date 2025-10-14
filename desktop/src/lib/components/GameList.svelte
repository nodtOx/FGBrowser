<script lang="ts">
    import { games, selectedIndex, selectGame, type Game } from '$lib/stores/games';
    
    function formatSize(size: string | null): string {
        if (!size) return 'N/A';
        return size;
    }
    
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
    
    async function handleGameClick(index: number) {
        await selectGame(index);
    }
</script>

<div class="game-list">
    {#each $games as game, index (game.id)}
        <div 
            class="game-item"
            class:selected={index === $selectedIndex}
            on:click={() => handleGameClick(index)}
            on:keydown={(e) => e.key === 'Enter' && handleGameClick(index)}
            role="button"
            tabindex={index === $selectedIndex ? 0 : -1}
        >
            <div class="game-date">
                {formatDate(game.date)}
            </div>
            <div class="game-title">
                {game.title}
            </div>
            <div class="game-size">
                {formatSize(game.repack_size)}
            </div>
        </div>
    {/each}
</div>

<style>
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
        transition: var(--transition);
        border-bottom: 1px solid transparent;
    }
    
    .game-item:hover {
        background-color: var(--color-hover);
    }
    
    .game-item.selected {
        background-color: var(--color-primary);
        color: var(--color-selectedText);
        font-weight: 600;
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

