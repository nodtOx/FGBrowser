<script lang="ts">
    // Sidebar with categories and filters
    export let totalGames = 0;
    
    let activeCategory = 'All Games';
    let activeRecent = '';
    let activeSize = '';
    let activeStatus = '';
    
    function selectCategory(category: string) {
        activeCategory = category;
        activeRecent = '';
        activeSize = '';
        activeStatus = '';
        console.log('Category selected:', category);
        // TODO: Filter games by category
    }
    
    function selectRecent(recent: string) {
        activeRecent = recent;
        activeCategory = '';
        activeSize = '';
        activeStatus = '';
        console.log('Recent filter selected:', recent);
        // TODO: Filter games by date
    }
    
    function selectSize(size: string) {
        activeSize = size;
        activeCategory = '';
        activeRecent = '';
        activeStatus = '';
        console.log('Size filter selected:', size);
        // TODO: Filter games by size
    }
    
    function selectStatus(status: string) {
        activeStatus = status;
        activeCategory = '';
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
        <div class="sidebar-item" class:active={activeCategory === 'All Games'} on:click={() => selectCategory('All Games')} on:keydown={(e) => handleKeydown(e, () => selectCategory('All Games'))} role="button" tabindex="0">
            All ({totalGames})
        </div>
        <div class="sidebar-item" class:active={activeCategory === 'Action'} on:click={() => selectCategory('Action')} on:keydown={(e) => handleKeydown(e, () => selectCategory('Action'))} role="button" tabindex="0">Action</div>
        <div class="sidebar-item" class:active={activeCategory === 'Adventure'} on:click={() => selectCategory('Adventure')} on:keydown={(e) => handleKeydown(e, () => selectCategory('Adventure'))} role="button" tabindex="0">Adventure</div>
        <div class="sidebar-item" class:active={activeCategory === 'Racing'} on:click={() => selectCategory('Racing')} on:keydown={(e) => handleKeydown(e, () => selectCategory('Racing'))} role="button" tabindex="0">Racing</div>
        <div class="sidebar-item" class:active={activeCategory === 'RPG'} on:click={() => selectCategory('RPG')} on:keydown={(e) => handleKeydown(e, () => selectCategory('RPG'))} role="button" tabindex="0">RPG</div>
        <div class="sidebar-item" class:active={activeCategory === 'Simulation'} on:click={() => selectCategory('Simulation')} on:keydown={(e) => handleKeydown(e, () => selectCategory('Simulation'))} role="button" tabindex="0">Simulation</div>
        <div class="sidebar-item" class:active={activeCategory === 'Sports'} on:click={() => selectCategory('Sports')} on:keydown={(e) => handleKeydown(e, () => selectCategory('Sports'))} role="button" tabindex="0">Sports</div>
        <div class="sidebar-item" class:active={activeCategory === 'Strategy'} on:click={() => selectCategory('Strategy')} on:keydown={(e) => handleKeydown(e, () => selectCategory('Strategy'))} role="button" tabindex="0">Strategy</div>
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
</style>

