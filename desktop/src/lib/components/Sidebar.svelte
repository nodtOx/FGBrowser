<script lang="ts">
    import type { CategoryWithCount } from '$lib/stores/games';
    import { activeFilters, activeSizeFilter, activeStatusFilter, activeTimeFilter, applySizeFilter, applyStatusFilter, applyTimeFilter, categories, clearAllFilters, debouncedApplyFilters, removeFilter, selectedCategories, toggleCategorySelection } from '$lib/stores/games';
    import SidebarBase from './SidebarBase.svelte';
    
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

<SidebarBase>
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
    
    
</SidebarBase>

