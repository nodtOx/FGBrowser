<script lang="ts">
    import { searchQuery, searchGames } from '$lib/stores/games';
    
    let query = '';
    let debounceTimer: number;
    
    function handleInput(e: Event) {
        const input = e.target as HTMLInputElement;
        query = input.value;
        
        // Debounce search
        clearTimeout(debounceTimer);
        debounceTimer = setTimeout(() => {
            searchGames(query);
            searchQuery.set(query);
        }, 300);
    }
</script>

<div class="search-bar">
    <div class="search-input-wrapper">
        <input
            type="search"
            placeholder="Search games... (Press / to focus)"
            value={query}
            on:input={handleInput}
            class="search-input"
        />
    </div>
    
    <select class="filter-select">
        <option>Sort: Date</option>
        <option>Sort: Title</option>
        <option>Sort: Size</option>
    </select>
    
    <select class="filter-select">
        <option>Filter: All</option>
        <option>With Magnets</option>
        <option>Recent</option>
    </select>
</div>

<style>
    .search-bar {
        display: flex;
        gap: 12px;
        padding: 12px 16px;
        background-color: var(--color-backgroundSecondary);
        border-bottom: 1px solid var(--color-border);
    }
    
    .search-input-wrapper {
        flex: 1;
        display: flex;
        align-items: center;
    }
    
    .search-input {
        width: 100%;
        padding: 10px 12px;
        background-color: var(--color-background);
        border: 1px solid var(--color-border);
        border-radius: var(--border-radius);
        color: var(--color-text);
        font-size: 14px;
        outline: none;
        transition: var(--transition);
    }
    
    .search-input:focus {
        border-color: var(--color-primary);
        box-shadow: 0 0 0 2px rgba(233, 69, 96, 0.2);
    }
    
    .filter-select {
        padding: 10px 16px;
        background-color: var(--color-background);
        border: 1px solid var(--color-border);
        border-radius: var(--border-radius);
        color: var(--color-text);
        font-size: 13px;
        cursor: pointer;
        outline: none;
        transition: var(--transition);
    }
    
    .filter-select:hover {
        border-color: var(--color-primary);
    }
    
    .filter-select:focus {
        border-color: var(--color-primary);
        box-shadow: 0 0 0 2px rgba(233, 69, 96, 0.2);
    }
</style>

