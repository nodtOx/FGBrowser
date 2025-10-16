<script lang="ts">
    import type { CategoryWithCount } from '$lib/stores/games';
    import { activeFilters, activeSizeFilter, activeStatusFilter, activeTimeFilter, applySizeFilter, applyStatusFilter, applyTimeFilter, categories, clearAllFilters, debouncedApplyFilters, removeFilter, selectedCategories, toggleCategorySelection } from '$lib/stores/games';
    import { focusedPanel } from '$lib/stores/navigation';
    import { onDestroy, onMount } from 'svelte';
    import SidebarBase from './SidebarBase.svelte';
    
    // Sidebar with categories and filters
    export let totalGames = 0;
    
    let showAllCategories = false;
    const INITIAL_CATEGORIES_COUNT = 10;
    
    // Track focused item index in each section
    let focusedCategoryIndex = 0;
    let focusedRecentIndex = 0;
    let focusedSizeIndex = 0;
    
    // Element refs for scrolling
    let categoryElements: HTMLElement[] = [];
    let moreButtonElement: HTMLElement | null = null;
    
    const recentOptions = ['Today', 'This Week', 'This Month'];
    const sizeOptions = ['< 1 GB', '1-10 GB', '10-25 GB', '25-40 GB', '40-60 GB', '> 60 GB'];
    
    // Scroll element into view
    function scrollIntoView(element: HTMLElement | null) {
        if (element) {
            element.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
        }
    }
    
    // Get categories to display based on show state
    $: visibleCategories = showAllCategories 
        ? $categories 
        : $categories.slice(0, INITIAL_CATEGORIES_COUNT);
    
    // Calculate total items in categories section (including More button if present)
    $: totalCategoryItems = $categories.length > INITIAL_CATEGORIES_COUNT 
        ? visibleCategories.length + 1  // +1 for More/Less button
        : visibleCategories.length;
    
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
    
    // Handle keyboard navigation within focused panels
    function handlePanelKeydown(e: KeyboardEvent) {
        const panel = $focusedPanel;
        
        if (panel === 'categories') {
            if (e.key === 'ArrowUp') {
                e.preventDefault();
                focusedCategoryIndex = focusedCategoryIndex > 0 ? focusedCategoryIndex - 1 : totalCategoryItems - 1;
                // Scroll to focused item
                if (focusedCategoryIndex === visibleCategories.length) {
                    scrollIntoView(moreButtonElement);
                } else {
                    scrollIntoView(categoryElements[focusedCategoryIndex]);
                }
            } else if (e.key === 'ArrowDown') {
                e.preventDefault();
                focusedCategoryIndex = focusedCategoryIndex < totalCategoryItems - 1 ? focusedCategoryIndex + 1 : 0;
                // Scroll to focused item
                if (focusedCategoryIndex === visibleCategories.length) {
                    scrollIntoView(moreButtonElement);
                } else {
                    scrollIntoView(categoryElements[focusedCategoryIndex]);
                }
            } else if (e.key === 'PageUp') {
                e.preventDefault();
                focusedCategoryIndex = Math.max(0, focusedCategoryIndex - 5);
                scrollIntoView(categoryElements[focusedCategoryIndex]);
            } else if (e.key === 'PageDown') {
                e.preventDefault();
                focusedCategoryIndex = Math.min(totalCategoryItems - 1, focusedCategoryIndex + 5);
                if (focusedCategoryIndex === visibleCategories.length) {
                    scrollIntoView(moreButtonElement);
                } else {
                    scrollIntoView(categoryElements[focusedCategoryIndex]);
                }
            } else if (e.key === 'Enter') {
                e.preventDefault();
                // Check if we're on the More/Less button (last item)
                if (focusedCategoryIndex === visibleCategories.length && $categories.length > INITIAL_CATEGORIES_COUNT) {
                    toggleShowAllCategories();
                    // Reset to first item after toggling
                    focusedCategoryIndex = 0;
                } else if (visibleCategories[focusedCategoryIndex]) {
                    handleCategoryToggle(visibleCategories[focusedCategoryIndex]);
                }
            } else if (e.key === 'm' || e.key === 'M') {
                // Quick toggle with 'm' key
                e.preventDefault();
                if ($categories.length > INITIAL_CATEGORIES_COUNT) {
                    toggleShowAllCategories();
                    focusedCategoryIndex = 0;
                }
            }
        } else if (panel === 'recent') {
            if (e.key === 'ArrowUp') {
                e.preventDefault();
                focusedRecentIndex = focusedRecentIndex > 0 ? focusedRecentIndex - 1 : recentOptions.length - 1;
            } else if (e.key === 'ArrowDown') {
                e.preventDefault();
                focusedRecentIndex = focusedRecentIndex < recentOptions.length - 1 ? focusedRecentIndex + 1 : 0;
            } else if (e.key === 'Enter') {
                e.preventDefault();
                selectRecent(recentOptions[focusedRecentIndex]);
            }
        } else if (panel === 'size') {
            if (e.key === 'ArrowUp') {
                e.preventDefault();
                focusedSizeIndex = focusedSizeIndex > 0 ? focusedSizeIndex - 1 : sizeOptions.length - 1;
            } else if (e.key === 'ArrowDown') {
                e.preventDefault();
                focusedSizeIndex = focusedSizeIndex < sizeOptions.length - 1 ? focusedSizeIndex + 1 : 0;
            } else if (e.key === 'Enter') {
                e.preventDefault();
                selectSize(sizeOptions[focusedSizeIndex]);
            }
        }
    }
    
    // Set up keyboard event listener
    onMount(() => {
        window.addEventListener('keydown', handlePanelKeydown);
    });
    
    onDestroy(() => {
        window.removeEventListener('keydown', handlePanelKeydown);
    });
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
    
    <div class="sidebar-section" class:focused={$focusedPanel === 'categories'}>
        <div class="section-title">Categories</div>
        
        <!-- Show "All" when no filters are active -->
        {#if $activeFilters.length === 0}
            <div class="sidebar-item all-games">
                <span>All ({totalGames})</span>
            </div>
        {/if}
        
        <!-- Category Selection -->
        {#each visibleCategories as category, index (category.id)}
            <div 
                bind:this={categoryElements[index]}
                class="sidebar-item category-item" 
                class:selected={isCategorySelected(category)}
                class:keyboard-focused={$focusedPanel === 'categories' && index === focusedCategoryIndex}
                on:click={() => handleCategoryToggle(category)} 
                on:keydown={(e) => handleKeydown(e, () => handleCategoryToggle(category))} 
                role="button" 
                tabindex="-1"
            >
                <span class="category-name">{category.name}</span>
                <span class="category-count">({category.game_count})</span>
                {#if isCategorySelected(category)}
                    <span class="category-indicator">✓</span>
                {/if}
            </div>
        {/each}
        
        {#if $categories.length > INITIAL_CATEGORIES_COUNT}
            <div 
                bind:this={moreButtonElement}
                class="sidebar-item more-button" 
                class:keyboard-focused={$focusedPanel === 'categories' && focusedCategoryIndex === visibleCategories.length}
                on:click={toggleShowAllCategories} 
                on:keydown={(e) => handleKeydown(e, toggleShowAllCategories)} 
                role="button" 
                tabindex="-1"
                title="Press 'm' to toggle or navigate with arrows and press Enter"
            >
                <span>{showAllCategories ? `Less (${$categories.length - INITIAL_CATEGORIES_COUNT} hidden)` : `More (${$categories.length - INITIAL_CATEGORIES_COUNT} more)`}</span>
            </div>
        {/if}
    </div>
    <div class="seperator"></div>
    <div class="sidebar-section" class:focused={$focusedPanel === 'recent'}>
        <div class="section-title">Recent</div>
        {#each recentOptions as option, index}
            <div 
                class="sidebar-item" 
                class:active={$activeTimeFilter === option} 
                class:keyboard-focused={$focusedPanel === 'recent' && index === focusedRecentIndex}
                on:click={() => selectRecent(option)} 
                on:keydown={(e) => handleKeydown(e, () => selectRecent(option))} 
                role="button" 
                tabindex="-1"
            >
                <span>{option}</span>
            </div>
        {/each}
    </div>
    <div class="seperator"></div>
    <div class="sidebar-section" class:focused={$focusedPanel === 'size'}>
        <div class="section-title">Size</div>
        {#each sizeOptions as option, index}
            <div 
                class="sidebar-item" 
                class:active={$activeSizeFilter === option} 
                class:keyboard-focused={$focusedPanel === 'size' && index === focusedSizeIndex}
                on:click={() => selectSize(option)} 
                on:keydown={(e) => handleKeydown(e, () => selectSize(option))} 
                role="button" 
                tabindex="-1"
            >
                <span>{option}</span>
            </div>
        {/each}
    </div>
    
    
</SidebarBase>

