<script lang="ts">
    import { copyMagnetLink, formatSize, openMagnetLink, selectedGame } from '$lib/stores/games';
    
    async function handleOpenMagnet(magnet: string) {
        await openMagnetLink(magnet);
    }
    
    async function handleCopyMagnet(magnet: string) {
        await copyMagnetLink(magnet);
    }
</script>

<div class="details-panel">
    {#if $selectedGame}
        <div class="details-header">
            <h2 class="game-title">{$selectedGame.title}</h2>
        </div>
        
        <div class="details-content">
            <div class="info-grid">
                <div class="info-item">
                    <span class="info-label">Categories:</span>
                    <div class="info-value">
                        {#if $selectedGame.categories && $selectedGame.categories.length > 0}
                            <div class="categories-list">
                                {#each $selectedGame.categories as category}
                                    <span class="category-tag">{category.name}</span>
                                {/each}
                            </div>
                        {:else}
                            N/A
                        {/if}
                    </div>
                </div>
                
                <div class="info-item">
                    <span class="info-label">Company:</span>
                    <span class="info-value">{$selectedGame.company || 'N/A'}</span>
                </div>
                
                <div class="info-item">
                    <span class="info-label">Languages:</span>
                    <span class="info-value">{$selectedGame.languages || 'N/A'}</span>
                </div>
                
                <div class="info-item">
                    <span class="info-label">Original Size:</span>
                    <span class="info-value">{$selectedGame.original_size || 'N/A'}</span>
                </div>
                
                <div class="info-item">
                    <span class="info-label">Repack Size:</span>
                    <span class="info-value">{formatSize($selectedGame.size)}</span>
                </div>
                
                <div class="info-item">
                    <span class="info-label">Release Date:</span>
                    <span class="info-value">{$selectedGame.date || 'N/A'}</span>
                </div>
            </div>
            
            {#if $selectedGame.magnet_links.length > 0}
                <div class="magnets-section">
                    <h3>MAGNET LINKS ({$selectedGame.magnet_links.length}):</h3>
                    <div class="magnet-list">
                        {#each $selectedGame.magnet_links as link, index}
                            <div class="magnet-item">
                                <span class="magnet-index">[{index + 1}]</span>
                                <span class="magnet-source">{link.source}</span>
                                <div class="magnet-actions">
                                    <button 
                                        class="btn btn-primary"
                                        on:click={() => handleOpenMagnet(link.magnet)}
                                    >
                                        Open
                                    </button>
                                    <button 
                                        class="btn btn-secondary"
                                        on:click={() => handleCopyMagnet(link.magnet)}
                                    >
                                        Copy
                                    </button>
                                </div>
                            </div>
                        {/each}
                    </div>
                </div>
            {/if}
        </div>
    {:else}
        <div class="no-selection">
            <p>Select a game to view details</p>
        </div>
    {/if}
</div>

<style>
    .details-panel {
        display: flex;
        flex-direction: column;
        height: 100%;
        background-color: var(--color-backgroundSecondary);
        border-top: 1px solid var(--color-border);
        overflow-y: auto;
    }
    
    .details-header {
        padding: 16px;
        border-bottom: 1px solid var(--color-border);
    }
    
    .game-title {
        font-size: 18px;
        font-weight: 600;
        color: var(--color-text);
        margin: 0;
    }
    
    .details-content {
        padding: 16px;
        flex: 1;
    }
    
    .info-grid {
        display: grid;
        grid-template-columns: repeat(2, 1fr);
        gap: 12px;
        margin-bottom: 20px;
    }
    
    .info-item {
        display: flex;
        flex-direction: column;
        gap: 4px;
    }
    
    .info-label {
        font-size: 12px;
        color: var(--color-textSecondary);
        font-weight: 500;
    }
    
    .info-value {
        font-size: 14px;
        color: var(--color-text);
    }
    
    .magnets-section {
        margin-top: 20px;
    }
    
    .magnets-section h3 {
        font-size: 14px;
        font-weight: 600;
        color: var(--color-primary);
        margin-bottom: 12px;
    }
    
    .magnet-list {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }
    
    .magnet-item {
        display: flex;
        align-items: center;
        gap: 12px;
        padding: 12px;
        background-color: var(--color-backgroundTertiary);
    }
    
    .magnet-index {
        color: var(--color-primary);
        font-weight: 600;
        min-width: 30px;
    }
    
    .magnet-source {
        flex: 1;
        color: var(--color-text);
    }
    
    .magnet-actions {
        display: flex;
        gap: 8px;
    }
    
    .btn {
        padding: 6px 16px;
        border: none;
        cursor: pointer;
        font-size: 12px;
        font-weight: 500;
        transition: var(--transition);
    }
    
    .btn-primary {
        background-color: var(--color-primary);
        color: var(--color-selectedText);
    }
    
    .btn-primary:hover {
        opacity: 0.9;
        transform: translateY(-1px);
    }
    
    .btn-secondary {
        background-color: var(--color-secondary);
        color: var(--color-selectedText);
    }
    
    .btn-secondary:hover {
        opacity: 0.9;
        transform: translateY(-1px);
    }
    
    .no-selection {
        display: flex;
        align-items: center;
        justify-content: center;
        height: 100%;
        color: var(--color-textMuted);
    }
    
    .categories-list {
        display: flex;
        flex-wrap: wrap;
        gap: 6px;
    }
    
    .category-tag {
        background-color: var(--color-backgroundTertiary);
        color: var(--color-text);
        padding: 2px 8px;
        border-radius: 12px;
        font-size: 11px;
        font-weight: 500;
        border: 1px solid var(--color-border);
        white-space: nowrap;
    }
</style>

