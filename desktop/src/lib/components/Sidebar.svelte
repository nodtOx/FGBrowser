<script lang="ts">
    import type { CategoryWithCount } from '$lib/stores/games';
    import { categories, clearCategorySelection, debouncedApplyCategoryFilters, selectedCategories, toggleCategorySelection } from '$lib/stores/games';
    
    // Sidebar with categories and filters
    export let totalGames = 0;
    
    let activeRecent = '';
    let activeSize = '';
    let activeStatus = '';
    let showAllCategories = false;
    
    const INITIAL_CATEGORIES_COUNT = 10;
    
    // Get categories to display based on show state
    $: visibleCategories = showAllCategories 
        ? $categories 
        : $categories.slice(0, INITIAL_CATEGORIES_COUNT);
    
    // Auto-apply filters when selection changes (debounced)
    $: if ($selectedCategories) {
        debouncedApplyCategoryFilters();
    }
    
    function toggleShowAllCategories() {
        showAllCategories = !showAllCategories;
    }
    
    function isCategorySelected(category: CategoryWithCount): boolean {
        return $selectedCategories.some(c => c.id === category.id);
    }
    
    async function handleCategoryToggle(category: CategoryWithCount) {
        toggleCategorySelection(category);
        activeRecent = '';
        activeSize = '';
        activeStatus = '';
    }
    
    async function handleClearAll() {
        clearCategorySelection();
        activeRecent = '';
        activeSize = '';
        activeStatus = '';
    }
    
    function selectRecent(recent: string) {
        activeRecent = recent;
        clearCategorySelection();
        activeSize = '';
        activeStatus = '';
        console.log('Recent filter selected:', recent);
        // TODO: Filter games by date
    }
    
    function selectSize(size: string) {
        activeSize = size;
        clearCategorySelection();
        activeRecent = '';
        activeStatus = '';
        console.log('Size filter selected:', size);
        // TODO: Filter games by size
    }
    
    function selectStatus(status: string) {
        activeStatus = status;
        clearCategorySelection();
        activeRecent = '';
        activeSize = '';
        console.log('Status selected:', status);
        // TODO: Filter by download status
    }
    
    function handleKeydown(e: KeyboardEvent, action: () => void) {
        if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            action();
        }
    }
</script>

<div class="sidebar">
    <div class="sidebar-section">
        <div class="section-title">Categories</div>
        
        <!-- Selected Categories Display -->
        {#if $selectedCategories.length > 0}
            <div class="selected-categories">
                <div class="selected-header">
                    <span class="selected-count">{$selectedCategories.length} selected</span>
                    <button class="clear-all-btn" on:click={handleClearAll}>Clear All</button>
                </div>
                <div class="selected-chips">
                    {#each $selectedCategories as category (category.id)}
                        <span class="category-chip">
                            {category.name}
                            <button class="chip-remove" on:click={() => handleCategoryToggle(category)}>×</button>
                        </span>
                    {/each}
                </div>
            </div>
        {:else}
            <div class="sidebar-item all-games">
                All ({totalGames})
            </div>
        {/if}
        
        <!-- Category Selection -->
        {#each visibleCategories as category (category.id)}
            <div 
                class="sidebar-item category-item" 
                class:selected={isCategorySelected(category)}
                on:click={() => handleCategoryToggle(category)} 
                on:keydown={(e) => handleKeydown(e, () => handleCategoryToggle(category))} 
                role="button" 
                tabindex="0"
            >
                <span class="category-name">{category.name}</span>
                <span class="category-count">({category.game_count})</span>
                {#if isCategorySelected(category)}
                    <span class="category-indicator">✓</span>
                {/if}
            </div>
        {/each}
        
        {#if $categories.length > INITIAL_CATEGORIES_COUNT}
            <div class="sidebar-item more-button" on:click={toggleShowAllCategories} on:keydown={(e) => handleKeydown(e, toggleShowAllCategories)} role="button" tabindex="0">
                {showAllCategories ? `Less (${$categories.length - INITIAL_CATEGORIES_COUNT} hidden)` : `More (${$categories.length - INITIAL_CATEGORIES_COUNT} more)`}
            </div>
        {/if}
    </div>
    
    <div class="sidebar-section">
        <div class="section-title">Recent</div>
        <div class="sidebar-item" class:active={activeRecent === 'Today'} on:click={() => selectRecent('Today')} on:keydown={(e) => handleKeydown(e, () => selectRecent('Today'))} role="button" tabindex="0">Today</div>
        <div class="sidebar-item" class:active={activeRecent === 'This Week'} on:click={() => selectRecent('This Week')} on:keydown={(e) => handleKeydown(e, () => selectRecent('This Week'))} role="button" tabindex="0">This Week</div>
        <div class="sidebar-item" class:active={activeRecent === 'This Month'} on:click={() => selectRecent('This Month')} on:keydown={(e) => handleKeydown(e, () => selectRecent('This Month'))} role="button" tabindex="0">This Month</div>
    </div>
    
    <div class="sidebar-section">
        <div class="section-title">Size</div>
        <div class="sidebar-item" class:active={activeSize === '< 10 GB'} on:click={() => selectSize('< 10 GB')} on:keydown={(e) => handleKeydown(e, () => selectSize('< 10 GB'))} role="button" tabindex="0">{'<'} 10 GB</div>
        <div class="sidebar-item" class:active={activeSize === '10-50 GB'} on:click={() => selectSize('10-50 GB')} on:keydown={(e) => handleKeydown(e, () => selectSize('10-50 GB'))} role="button" tabindex="0">10-50 GB</div>
        <div class="sidebar-item" class:active={activeSize === '> 50 GB'} on:click={() => selectSize('> 50 GB')} on:keydown={(e) => handleKeydown(e, () => selectSize('> 50 GB'))} role="button" tabindex="0">{'>'} 50 GB</div>
    </div>
    
    <div class="sidebar-section">
        <div class="section-title">Status</div>
        <div class="sidebar-item" class:active={activeStatus === 'Available'} on:click={() => selectStatus('Available')} on:keydown={(e) => handleKeydown(e, () => selectStatus('Available'))} role="button" tabindex="0">Available</div>
        <div class="sidebar-item" class:active={activeStatus === 'Downloading'} on:click={() => selectStatus('Downloading')} on:keydown={(e) => handleKeydown(e, () => selectStatus('Downloading'))} role="button" tabindex="0">Downloading</div>
        <div class="sidebar-item" class:active={activeStatus === 'Completed'} on:click={() => selectStatus('Completed')} on:keydown={(e) => handleKeydown(e, () => selectStatus('Completed'))} role="button" tabindex="0">Completed</div>
    </div>
</div>

<style>
    .sidebar {
        width: var(--sidebar-width);
        background-color: var(--color-background);
        border-right: 1px solid var(--color-border);
        overflow-y: auto;
        padding: 8px 0;
        display: flex;
        flex-direction: column;
        gap: 16px;
        font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
        font-size: calc(var(--base-font-size) * 0.9);
    }
    
    .sidebar-section {
        display: flex;
        flex-direction: column;
    }
    
    .section-title {
        font-size: calc(var(--base-font-size) * 0.85);
        font-weight: 600;
        color: var(--color-textSecondary);
        padding: 4px 12px 2px 12px;
        margin: 0;
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }
    
    .sidebar-item {
        padding: 3px 12px;
        color: var(--color-textSecondary);
        cursor: pointer;
        transition: var(--transition);
        line-height: 1.4;
    }
    
    .sidebar-item:hover {
        background-color: var(--color-hover);
        color: var(--color-text);
    }
    
    .sidebar-item.active {
        background-color: var(--color-primary);
        color: var(--color-selectedText);
        font-weight: 600;
    }
    
    .sidebar-item.more-button {
        color: var(--color-primary);
        font-style: italic;
        font-size: calc(var(--base-font-size) * 0.85);
        margin-top: 4px;
        border-top: 1px solid var(--color-border);
        padding-top: 6px;
    }
    
    .sidebar-item.more-button:hover {
        background-color: var(--color-hover);
        color: var(--color-primary);
        font-weight: 500;
    }
    
    /* Selected Categories Styles */
    .selected-categories {
        margin-bottom: 12px;
        padding-bottom: 12px;
        border-bottom: 1px solid var(--color-border);
    }
    
    .selected-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 8px;
        padding: 0 12px;
    }
    
    .selected-count {
        font-size: calc(var(--base-font-size) * 0.8);
        color: var(--color-primary);
        font-weight: 600;
    }
    
    .clear-all-btn {
        background: none;
        border: none;
        color: var(--color-textSecondary);
        font-size: calc(var(--base-font-size) * 0.75);
        cursor: pointer;
        padding: 2px 6px;
        border-radius: 3px;
        transition: var(--transition);
    }
    
    .clear-all-btn:hover {
        background-color: var(--color-hover);
        color: var(--color-text);
    }
    
    .selected-chips {
        display: flex;
        flex-wrap: wrap;
        gap: 4px;
        padding: 0 12px;
    }
    
    .category-chip {
        display: inline-flex;
        align-items: center;
        background-color: var(--color-primary);
        color: var(--color-selectedText);
        font-size: calc(var(--base-font-size) * 0.75);
        padding: 2px 6px;
        border-radius: 0px;
        font-weight: 500;
    }
    
    .chip-remove {
        background: none;
        border: none;
        color: var(--color-selectedText);
        margin-left: 4px;
        cursor: pointer;
        font-size: 14px;
        line-height: 1;
        padding: 0;
        width: 14px;
        height: 14px;
        display: flex;
        align-items: center;
        justify-content: center;
        border-radius: 50%;
        transition: var(--transition);
    }
    
    .chip-remove:hover {
        background-color: rgba(255, 255, 255, 0.2);
    }
    
    .all-games {
        color: var(--color-text);
        font-weight: 600;
        background-color: var(--color-backgroundTertiary);
        pointer-events: none;
    }
    
    /* Category Item Styles */
    .category-item {
        display: flex;
        align-items: center;
        justify-content: space-between;
        position: relative;
    }
    
    .category-item.selected {
        background-color: var(--color-backgroundTertiary);
        color: var(--color-primary);
        font-weight: 600;
        border-left: 3px solid var(--color-primary);
        padding-left: 9px;
    }
    
    .category-item:hover:not(.selected) {
        background-color: var(--color-hover);
        color: var(--color-text);
    }
    
    .category-name {
        flex: 1;
        min-width: 0;
    }
    
    .category-count {
        color: var(--color-textSecondary);
        font-size: calc(var(--base-font-size) * 0.85);
        margin-left: 4px;
    }
    
    .category-item.selected .category-count {
        color: var(--color-primary);
    }
    
    .category-indicator {
        color: var(--color-primary);
        font-weight: bold;
        margin-left: 8px;
        font-size: calc(var(--base-font-size) * 0.9);
    }
</style>

