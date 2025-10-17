<script lang="ts">
    import { isCrawlingPopular } from '$lib/stores/games';
    import { refreshPopularCounts } from '$lib/stores/popular';
    import { invoke } from '@tauri-apps/api/core';
    
    export let isOpen = false;
    export let isUpdating = false;
    export let totalGames = 0;
    export let onComplete: () => void = () => {};
    export let onStart: () => void = () => {};
    
    // Track new games found during updates
    let newGamesFound = 0;
    
    // Configuration
    const MAX_PAGES = 10;
    const GAMES_PER_PAGE = 10;
    const MODAL_CLOSE_DELAY_MS = 1500; // Delay before closing modal after completion
    
    let status = 'Initializing crawler...';
    let crawlerRunning = false;
    let estimatedTotal = MAX_PAGES * GAMES_PER_PAGE;
    
    interface CrawlResult {
        total_games: number;
        status: string;
    }
    
    // Watch for changes in isOpen - trigger appropriate action
    $: if (isOpen && !crawlerRunning) {
        if (isUpdating) {
            console.log('Starting database update from reactive statement');
            startUpdate();
        } else {
            console.log('Starting crawler from reactive statement');
            startCrawl();
        }
    }
    
    async function startCrawl() {
        if (crawlerRunning) return;
        
        crawlerRunning = true;
        status = 'Fetching games...';
        onStart(); // Start polling
        
        try {
            console.log('🎯 CrawlerModal calling start_crawler (no parameters - hardcoded in Rust)');
            const result = await invoke<CrawlResult>('start_crawler');
            status = `Completed! Found ${result.total_games} games`;
            
            // Wait before closing
            setTimeout(() => {
                isOpen = false;
                crawlerRunning = false;
                onComplete();
            }, MODAL_CLOSE_DELAY_MS);
        } catch (error) {
            console.error('Crawler error:', error);
            status = `Error: ${error}`;
            crawlerRunning = false;
            
            // Close on error
            setTimeout(() => {
                isOpen = false;
            }, 3000);
        }
    }
    
    async function startUpdate() {
        if (crawlerRunning) return;
        
        crawlerRunning = true;
        newGamesFound = 0; // Reset counter
        status = 'Checking for new games...';
        onStart(); // Start polling
        
        try {
            console.log('🎯 CrawlerModal calling update_database');
            const result = await invoke<CrawlResult>('update_database');
            
            newGamesFound = result.total_games;
            
            if (result.total_games > 0) {
                status = `Found ${result.total_games} new games!`;
            } else {
                status = 'Database is up to date';
            }
            
            // Fetch popular repacks after database update
            await fetchPopularRepacks();
            
            // Wait before closing
            setTimeout(() => {
                isOpen = false;
                crawlerRunning = false;
                newGamesFound = 0; // Reset when closing
                onComplete();
            }, MODAL_CLOSE_DELAY_MS);
        } catch (error) {
            console.error('Update error:', error);
            status = `Error: ${error}`;
            crawlerRunning = false;
            newGamesFound = 0;
            
            // Close on error
            setTimeout(() => {
                isOpen = false;
            }, 3000);
        }
    }
    
    async function fetchPopularRepacks() {
        try {
            console.log('[CrawlerModal] Fetching popular repacks...');
            status = 'Fetching popular repacks...';
            
            // Fetch all five periods: week, today, month, year, and award
            await invoke<number>('fetch_popular_repacks', { period: 'week' });
            await invoke<number>('fetch_popular_repacks', { period: 'today' });
            await invoke<number>('fetch_popular_repacks', { period: 'month' });
            await invoke<number>('fetch_popular_repacks', { period: 'year' });
            await invoke<number>('fetch_popular_repacks', { period: 'award' });
            
            status = 'Crawling popular games...';
            isCrawlingPopular.set(true);
            
            // Crawl all five periods
            await invoke<number>('crawl_popular_games', { period: 'week' });
            await invoke<number>('crawl_popular_games', { period: 'today' });
            await invoke<number>('crawl_popular_games', { period: 'month' });
            await invoke<number>('crawl_popular_games', { period: 'year' });
            await invoke<number>('crawl_popular_games', { period: 'award' });
            
            isCrawlingPopular.set(false);
            status = 'Popular repacks updated!';
            
            // Refresh badges
            console.log('[CrawlerModal] Refreshing popular badges');
            refreshPopularCounts();
        } catch (error) {
            console.error('[CrawlerModal] Failed to fetch popular repacks:', error);
            isCrawlingPopular.set(false);
            status = 'Popular repacks update failed';
        }
    }
</script>

{#if isOpen}
    <div class="modal-backdrop">
        <div class="modal">
            <div class="modal-header">
                <div class="modal-title">{isUpdating ? 'Checking for Updates' : 'Database Initialization'}</div>
            </div>
            
            <div class="modal-content">
                <div class="progress-container">
                    <h3 class="progress-title">{isUpdating ? 'Updating Database' : 'Initializing Database'}</h3>
                        
                        <div class="spinner">
                            <div class="spinner-circle"></div>
                        </div>
                        
                        <div class="stats">
                            {#if isUpdating}
                                <div class="stat-item">
                                    <div class="stat-value">{newGamesFound}</div>
                                    <div class="stat-label">New Games Found</div>
                                </div>
                                <div class="stat-divider">|</div>
                                <div class="stat-item">
                                    <div class="stat-value secondary">{totalGames}</div>
                                    <div class="stat-label">Total Games</div>
                                </div>
                            {:else}
                                <div class="stat-item">
                                    <div class="stat-value">{totalGames}</div>
                                    <div class="stat-label">Games Found</div>
                                </div>
                                
                                {#if estimatedTotal > 0 && totalGames > 0}
                                    <div class="stat-divider">/</div>
                                    <div class="stat-item">
                                        <div class="stat-value secondary">~{estimatedTotal}</div>
                                        <div class="stat-label">Estimated Total</div>
                                    </div>
                                {/if}
                            {/if}
                        </div>
                        
                        {#if !isUpdating && estimatedTotal > 0 && totalGames > 0}
                            <div class="progress-bar">
                                <div class="progress-fill" style="width: {Math.min((totalGames / estimatedTotal) * 100, 100)}%"></div>
                            </div>
                        {/if}
                        
                    <p class="status">{status}</p>
                </div>
            </div>
        </div>
    </div>
{/if}

<style>
    .modal-backdrop {
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background-color: rgba(0, 0, 0, 0.2);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 1000;
    }
    
    .modal {
        background-color: var(--color-background);
        border: 2px solid var(--color-border);
        width: 600px;
        max-width: 90vw;
        display: flex;
        flex-direction: column;
    }
    
    .modal-header {
        background-color: var(--color-backgroundTertiary);
        border-bottom: 2px solid var(--color-border);
        padding: 12px 16px;
    }
    
    .modal-title {
        font-size: 14px;
        font-weight: 600;
        color: var(--color-text);
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }
    
    .modal-content {
        padding: 24px;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 20px;
        min-height: 280px;
    }
    
    .progress-container {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 20px;
        width: 100%;
    }
    
    .progress-title {
        font-size: 14px;
        font-weight: 600;
        color: var(--color-text);
        margin: 0;
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }
    
    .spinner {
        width: 60px;
        height: 60px;
        position: relative;
    }
    
    .spinner-circle {
        width: 100%;
        height: 100%;
        border: 4px solid var(--color-border);
        border-top-color: var(--color-primary);
        border-radius: 50%;
        animation: spin 1s linear infinite;
    }
    
    @keyframes spin {
        0% { transform: rotate(0deg); }
        100% { transform: rotate(360deg); }
    }
    
    .status {
        font-size: 14px;
        color: var(--color-text);
        margin: 0;
        font-weight: 500;
    }
    
    .stats {
        display: flex;
        align-items: center;
        gap: 16px;
    }
    
    .stat-item {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 4px;
    }
    
    .stat-value {
        font-size: 32px;
        font-weight: 700;
        color: var(--color-text);
        line-height: 1;
        font-family: monospace;
    }
    
    .stat-value.secondary {
        color: var(--color-textSecondary);
        font-size: 24px;
    }
    
    .stat-label {
        font-size: 11px;
        color: var(--color-textSecondary);
        text-transform: uppercase;
        letter-spacing: 1px;
    }
    
    .stat-divider {
        font-size: 32px;
        color: var(--color-border);
        font-weight: 300;
    }
    
    .progress-bar {
        width: 100%;
        height: 4px;
        background-color: var(--color-backgroundTertiary);
        border: 1px solid var(--color-border);
    }
    
    .progress-fill {
        height: 100%;
        background-color: var(--color-primary);
    }
</style>

