<script lang="ts">
    import { featureFlags } from '$lib/featureFlags';
    import { addDownload } from '$lib/stores/downloads';
    import { formatSize, openMagnetLink, selectedGame } from '$lib/stores/games';
    import { invoke } from '@tauri-apps/api/core';
    import { onDestroy } from 'svelte';
    
    async function handleDownload(magnet: string) {
        if (!$selectedGame) return;
        
        try {
            // Open folder picker dialog using Tauri command
            const selectedPath = await invoke<string | null>('select_download_folder');
            
            if (!selectedPath) {
                // User cancelled
                return;
            }
            
            // Start download
            await addDownload(magnet, $selectedGame.id, selectedPath);
            alert('Download started!');
        } catch (error) {
            console.error('Failed to start download:', error);
            alert('Failed to start download: ' + error);
        }
    }
    
    async function handleOpenMagnet(magnet: string) {
        await openMagnetLink(magnet);
    }
    
    function handleKeydown(event: KeyboardEvent) {
        // Check if we're typing in an input field
        const target = event.target as HTMLElement;
        const isTyping = target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable;
        
        if (isTyping || !$selectedGame || $selectedGame.magnet_links.length === 0) {
            return;
        }
        
        const firstMagnet = $selectedGame.magnet_links[0].magnet;
        
        // Press 'd' or 'Enter' to download/open the first magnet link
        if (event.key === 'd' || event.key === 'Enter') {
            event.preventDefault();
            if (featureFlags.torrentClient) {
                handleDownload(firstMagnet);
            } else {
                handleOpenMagnet(firstMagnet);
            }
        }
    }
    
    // Add keyboard listener when component mounts
    $: {
        if ($selectedGame) {
            window.addEventListener('keydown', handleKeydown);
        } else {
            window.removeEventListener('keydown', handleKeydown);
        }
    }
    
    onDestroy(() => {
        window.removeEventListener('keydown', handleKeydown);
    });
</script>

<div class="details-panel">
    {#if $selectedGame}
        <div class="details-header">
            <h2 class="game-title">{$selectedGame.title}</h2>
        </div>
        
        <div class="details-content">
            {#if $selectedGame.image_url}
                <div class="cover-art">
                    <img src={$selectedGame.image_url} alt={$selectedGame.title} />
                </div>
            {/if}
            
            <div class="details-info">
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
                                    {#if featureFlags.torrentClient}
                                    <button 
                                        class="btn btn-download"
                                        on:click={() => handleDownload(link.magnet)}
                                        title="Download with built-in torrent client{index === 0 ? ' (d or Enter)' : ''}"
                                    >
                                        Download Torrent
                                    </button>
                                    {:else}
                                    <button 
                                        class="btn btn-primary"
                                        on:click={() => handleOpenMagnet(link.magnet)}
                                        title="Open in your default torrent client{index === 0 ? ' (d or Enter)' : ''}"
                                    >
                                        Download Torrent
                                    </button>
                                    {/if}
                                </div>
                            </div>
                        {/each}
                    </div>
                </div>
            {/if}
            </div>
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
        display: flex;
        gap: 20px;
    }
    
    .cover-art {
        flex-shrink: 0;
        text-align: center;
        width: 200px;
    }
    
    .cover-art img {
        width: 200px;
        height: auto;
        border: 1px solid var(--color-border);
        border-radius: 4px;
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
        display: block;
    }
    
    .details-info {
        flex: 1;
        min-width: 0;
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
        grid-column: 1 / -1;
        border-top: 1px solid var(--color-border);
        padding-top: 16px;
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
        padding: 8px 16px;
        border: none;
        border-radius: var(--border-radius);
        cursor: pointer;
        font-size: 13px;
        font-weight: 500;
        transition: all 0.15s ease;
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
    }
    
    .btn-primary {
        background-color: var(--color-primary);
        color: var(--color-selectedText);
    }
    
    .btn-primary:hover {
        background-color: color-mix(in srgb, var(--color-primary) 80%, white 40%);
    }
    
    .btn-download {
        background-color: #4CAF50;
        color: white;
        font-weight: 600;
    }
    
    .btn-download:hover {
        background-color: #45a049;
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

