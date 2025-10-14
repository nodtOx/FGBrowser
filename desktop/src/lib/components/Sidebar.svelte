<script lang="ts">
    import type { CategoryWithCount } from '$lib/stores/games';
    import { activeFilters, activeSizeFilter, activeStatusFilter, activeTimeFilter, applySizeFilter, applyStatusFilter, applyTimeFilter, categories, clearAllFilters, debouncedApplyFilters, removeFilter, selectedCategories, toggleCategorySelection } from '$lib/stores/games';
    
    // Sidebar with categories and filters
    export let totalGames = 0;
    
    let showAllCategories = false;
    const INITIAL_CATEGORIES_COUNT = 10;
    
    // Get categories to display based on show state
    $: visibleCategories = showAllCategories 
        ? $categories 
        : $categories.slice(0, INITIAL_CATEGORIES_COUNT);
    
    // Auto-apply filters when any selection changes (debounced)
    $: if ($selectedCategories || $activeTimeFilter || $activeSizeFilter || $activeStatusFilter) {
        debouncedApplyFilters();
    }
    
    function toggleShowAllCategories() {
        showAllCategories = !showAllCategories;
    }
    
    function isCategorySelected(category: CategoryWithCount): boolean {
        return $selectedCategories.some(c => c.id === category.id);
    }
    
    async function handleCategoryToggle(category: CategoryWithCount) {
        toggleCategorySelection(category);
        // Categories now work with all other filters - no need to clear them
    }
    
    async function handleClearAll() {
        await clearAllFilters();
    }
    
    async function selectRecent(recent: string) {
        console.log('Recent filter selected:', recent);
        await applyTimeFilter(recent);
    }
    
    async function selectSize(size: string) {
        console.log('Size filter selected:', size);
        await applySizeFilter(size);
    }
    
    async function selectStatus(status: string) {
        console.log('Status selected:', status);
        await applyStatusFilter(status);
    }
    
    function handleKeydown(e: KeyboardEvent, action: () => void) {
        if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            action();
        }
    }
</script>

<div class="sidebar">
    <!-- Unified Active Filters Display (Above all sections) -->
    {#if $activeFilters.length > 0}
        <div class="selected-categories">
            <div class="selected-header">
                <span class="selected-count">{$activeFilters.length} active filter{$activeFilters.length === 1 ? '' : 's'}</span>
                <button class="clear-all-btn" on:click={handleClearAll}>Clear All</button>
            </div>
            <div class="selected-chips">
                {#each $activeFilters as filter (filter.type + '-' + filter.value)}
                    <span class="category-chip" class:time-filter-chip={filter.type === 'time'} class:size-filter-chip={filter.type === 'size'} class:status-filter-chip={filter.type === 'status'}>
                        {filter.label}
                        <button class="chip-remove" on:click={() => removeFilter(filter.type, filter.value)}>×</button>
                    </span>
                {/each}
            </div>
        </div>
    {/if}
    
    <div class="sidebar-section">
        <div class="section-title">Categories</div>
        
        <!-- Show "All" when no filters are active -->
        {#if $activeFilters.length === 0}
            <div class="sidebar-item all-games">
                <span>All ({totalGames})</span>
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
                <span>{showAllCategories ? `Less (${$categories.length - INITIAL_CATEGORIES_COUNT} hidden)` : `More (${$categories.length - INITIAL_CATEGORIES_COUNT} more)`}</span>
            </div>
        {/if}
    </div>
    <div class="seperator"></div>
    <div class="sidebar-section">
        <div class="section-title">Recent</div>
        <div class="sidebar-item" class:active={$activeTimeFilter === 'Today'} on:click={() => selectRecent('Today')} on:keydown={(e) => handleKeydown(e, () => selectRecent('Today'))} role="button" tabindex="0">
            <span>Today</span>
        </div>
        <div class="sidebar-item" class:active={$activeTimeFilter === 'This Week'} on:click={() => selectRecent('This Week')} on:keydown={(e) => handleKeydown(e, () => selectRecent('This Week'))} role="button" tabindex="0">
            <span>This Week</span>
        </div>
        <div class="sidebar-item" class:active={$activeTimeFilter === 'This Month'} on:click={() => selectRecent('This Month')} on:keydown={(e) => handleKeydown(e, () => selectRecent('This Month'))} role="button" tabindex="0">
            <span>This Month</span>
        </div>
    </div>
    <div class="seperator"></div>
    <div class="sidebar-section">
        <div class="section-title">Size</div>
        <div class="sidebar-item" class:active={$activeSizeFilter === '< 1 GB'} on:click={() => selectSize('< 1 GB')} on:keydown={(e) => handleKeydown(e, () => selectSize('< 1 GB'))} role="button" tabindex="0">
            <span>{'<'} 1 GB</span>
        </div>
        <div class="sidebar-item" class:active={$activeSizeFilter === '1-10 GB'} on:click={() => selectSize('1-10 GB')} on:keydown={(e) => handleKeydown(e, () => selectSize('1-10 GB'))} role="button" tabindex="0">
            <span>1-10 GB</span>
        </div>
        <div class="sidebar-item" class:active={$activeSizeFilter === '10-25 GB'} on:click={() => selectSize('10-25 GB')} on:keydown={(e) => handleKeydown(e, () => selectSize('10-25 GB'))} role="button" tabindex="0">
            <span>10-25 GB</span>
        </div>
        <div class="sidebar-item" class:active={$activeSizeFilter === '25-40 GB'} on:click={() => selectSize('25-40 GB')} on:keydown={(e) => handleKeydown(e, () => selectSize('25-40 GB'))} role="button" tabindex="0">
            <span>25-40 GB</span>
        </div>
        <div class="sidebar-item" class:active={$activeSizeFilter === '40-60 GB'} on:click={() => selectSize('40-60 GB')} on:keydown={(e) => handleKeydown(e, () => selectSize('40-60 GB'))} role="button" tabindex="0">
            <span>40-60 GB</span>
        </div>
        <div class="sidebar-item" class:active={$activeSizeFilter === '> 60 GB'} on:click={() => selectSize('> 60 GB')} on:keydown={(e) => handleKeydown(e, () => selectSize('> 60 GB'))} role="button" tabindex="0">
            <span>{'>'} 60 GB</span>
        </div>
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
        display: flex;
        justify-content: space-between;
        align-items: center;
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
    .seperator {
        border-top: 1px solid var(--color-border);
        /* margin: 4px 0; */
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
    
    /* Filter Chip Variants */
    .time-filter-chip {
        background-color: var(--color-warning);
        color: var(--color-background);
    }
    
    .time-filter-chip .chip-remove {
        color: var(--color-background);
    }
    
    .time-filter-chip .chip-remove:hover {
        background-color: rgba(46, 52, 64, 0.2);
    }
    
    .size-filter-chip {
        background-color: var(--color-info);
        color: var(--color-selectedText);
    }
    
    .status-filter-chip {
        background-color: var(--color-success);
        color: var(--color-background);
    }
    
    .status-filter-chip .chip-remove {
        color: var(--color-background);
    }
    
    .status-filter-chip .chip-remove:hover {
        background-color: rgba(46, 52, 64, 0.2);
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

