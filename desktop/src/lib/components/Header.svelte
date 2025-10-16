<script lang="ts">
    import { featureFlags } from '$lib/featureFlags';
    import { currentPage, navigateTo, type Page } from '$lib/stores/navigation';
    import { applyTheme, currentTheme, darkThemes, lightThemes } from '$lib/stores/theme';
    import { invoke } from '@tauri-apps/api/core';
    import { onDestroy, onMount } from 'svelte';
    
    let isDev = false;
    let totalUnseenPopular = 0;
    let unseenCheckInterval: any;
    let themeDropdownRef: HTMLElement | null = null;
    
    onMount(() => {
        // Check if we're in development mode
        isDev = window.location.hostname === 'localhost' || window.location.hostname === 'tauri.localhost';
        
        // Load unseen count initially
        loadUnseenPopularCount();
        
        // Check for unseen popular games every 30 seconds
        unseenCheckInterval = setInterval(loadUnseenPopularCount, 30000);
        
        return () => {
            if (unseenCheckInterval) {
                clearInterval(unseenCheckInterval);
            }
        };
    });

    onDestroy(() => {
        if (typeof document !== 'undefined') {
            document.removeEventListener('click', handleClickOutside);
        }
    });
    
    async function loadUnseenPopularCount() {
        try {
            totalUnseenPopular = await invoke<number>('get_total_unseen_popular_count');
        } catch (err) {
            console.error('Failed to load unseen popular count:', err);
        }
    }
    
    function handleNavClick(page: Page) {
        navigateTo(page);
    }
    
    let showThemeSelector = false;
    
    function toggleThemeSelector(event: MouseEvent) {
        event.stopPropagation();
        showThemeSelector = !showThemeSelector;
        
        if (showThemeSelector) {
            // Add click outside listener when opening
            setTimeout(() => {
                document.addEventListener('click', handleClickOutside);
            }, 0);
        } else {
            // Remove listener when closing
            document.removeEventListener('click', handleClickOutside);
        }
    }
    
    function handleClickOutside(event: MouseEvent) {
        if (themeDropdownRef && !themeDropdownRef.contains(event.target as Node)) {
            showThemeSelector = false;
            document.removeEventListener('click', handleClickOutside);
        }
    }
    
    function selectTheme(theme: any) {
        applyTheme(theme);
        showThemeSelector = false;
        document.removeEventListener('click', handleClickOutside);
    }
    
    async function handleReset() {
        if (!confirm('This will delete the database and restart the app. Continue?')) {
            return;
        }
        
        try {
            await invoke('reset_database');
            // Reload the window to restart the app
            window.location.reload();
        } catch (error) {
            console.error('Failed to reset:', error);
            alert('Failed to reset database: ' + error);
        }
    }
</script>

<header class="header">
    <div class="header-content">
        <nav class="nav-tabs">
                <button 
                    class="nav-tab"
                    class:active={$currentPage === 'browse'}
                    on:click={() => handleNavClick('browse')}
                >
                    Browse
                </button>
                <button 
                    class="nav-tab"
                    class:active={$currentPage === 'popular'}
                    on:click={() => handleNavClick('popular')}
                >
                    Popular
                    {#if totalUnseenPopular > 0}
                        <span class="unseen-badge">{totalUnseenPopular}</span>
                    {/if}
                </button>
                <button 
                    class="nav-tab pink-paw-tab"
                    class:active={$currentPage === 'pinkpaw'}
                    on:click={() => handleNavClick('pinkpaw')}
                >
                Pink Paw Award <span class="paw-emoji">🐾</span> 
                </button>
                {#if featureFlags.torrentClient}
                <button 
                    class="nav-tab"
                    class:active={$currentPage === 'downloads'}
                    on:click={() => handleNavClick('downloads')}
                >
                    Downloads
                </button>
                {/if}
                {#if featureFlags.settings}
                <button 
                    class="nav-tab"
                    class:active={$currentPage === 'settings'}
                    on:click={() => handleNavClick('settings')}
                >
                    Settings
                </button>
                {/if}
                {#if featureFlags.stats}
                <button 
                    class="nav-tab"
                    class:active={$currentPage === 'stats'}
                    on:click={() => handleNavClick('stats')}
                >
                    Stats
                </button>
                {/if}
            </nav>
        
        <div class="header-right">
            {#if isDev}
                <button class="reset-btn" on:click={handleReset} title="Reset Database & Restart (Dev Only)">
                    R
                </button>
            {/if}
            <a 
                href="https://fitgirl-repacks.site/donations/" 
                target="_blank" 
                rel="noopener noreferrer"
                class="donate-btn"
                title="Support FitGirl"
            >
                Donate to fitgirl
            </a>
            <div class="theme-selector-wrapper" bind:this={themeDropdownRef}>
                <button class="theme-btn" on:click={toggleThemeSelector} title="Change Theme (T)">
                    T
                </button>
                {#if showThemeSelector}
                    <div class="theme-dropdown">
                        <div class="theme-section-header">Dark Themes</div>
                        {#each darkThemes as theme}
                            <button 
                                class="theme-option"
                                class:active={$currentTheme.name === theme.name}
                                on:click={() => selectTheme(theme)}
                            >
                                {theme.name}
                            </button>
                        {/each}
                        <div class="theme-section-header">Light Themes</div>
                        {#each lightThemes as theme}
                            <button 
                                class="theme-option"
                                class:active={$currentTheme.name === theme.name}
                                on:click={() => selectTheme(theme)}
                            >
                                {theme.name}
                            </button>
                        {/each}
                    </div>
                {/if}
            </div>
        </div>
    </div>
</header>

<style>
    .header {
        height: 31px;
        background-color: var(--color-backgroundTertiary);
        border-bottom: 1px solid var(--color-border);
        display: flex;
        align-items: center;
        padding: 0;
        font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
        font-size: calc(var(--base-font-size) * 1);
    }
    
    .header-content {
        display: flex;
        align-items: center;
        justify-content: space-between;
        width: 100%;
        height: 100%;
        padding: 0 12px;
    }
    
    .nav-tabs {
        display: flex;
        gap: 0;
        align-items: center;
        height: 100%;
    }
    
    .nav-tab {
        background: none;
        border: none;
        color: var(--color-textSecondary);
        padding: 4px 16px;
        cursor: pointer;
        font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
        font-size: calc(var(--base-font-size) * 0.9);
        font-weight: 600;
        transition: var(--transition);
    }
    
    .nav-tab:hover {
        color: var(--color-text);
        background-color: var(--color-hover);
    }
    
    .nav-tab.active {
        color: var(--color-selectedText);
        background-color: var(--color-primary);
        
        border-bottom: none;
    }

    .nav-tab {
        position: relative;
    }

    .unseen-badge {
        display: inline-block;
        background-color: var(--color-primary);
        color: white;
        font-size: 9px;
        font-weight: 700;
        padding: 2px 5px;
        border-radius: 10px;
        margin-left: 4px;
        vertical-align: middle;
    }

    .nav-tab.active .unseen-badge {
        background-color: white;
        color: var(--color-primary);
    }
    
    .header-right {
        display: flex;
        align-items: center;
        gap: 8px;
        position: relative;
    }
    
    .reset-btn {
        color: var(--color-error);
        cursor: pointer;
        padding: 2px 6px;
        border: 1px solid var(--color-border);
        background: none;
        font-size: 11px;
        font-weight: 600;
    }
    
    .reset-btn:hover {
        background-color: var(--color-error);
        color: var(--color-background);
        border-color: var(--color-error);
    }

    .donate-btn {
        color: var(--color-background);
        background-color: #ff69b4;
        cursor: pointer;
        padding: 3px 8px;
        border: 1px solid #ff1493;
        font-size: 11px;
        font-weight: 600;
        text-decoration: none;
        display: inline-block;
        white-space: nowrap;
        transition: var(--transition);
    }
    
    .donate-btn:hover {
        background-color: #ff1493;
        color: var(--color-background);
        border-color: #ff1493;
        
    }

    .theme-selector-wrapper {
        position: relative;
    }
    
    .theme-btn {
        color: var(--color-textSecondary);
        cursor: pointer;
        padding: 2px 6px;
        border: 1px solid var(--color-border);
        background: none;
        font-size: 11px;
        font-weight: 600;
    }
    
    .theme-btn:hover {
        background-color: var(--color-hover);
        color: var(--color-text);
    }
    
    .theme-dropdown {
        position: absolute;
        top: 100%;
        right: 0;
        margin-top: 2px;
        background-color: var(--color-backgroundSecondary);
        border: 1px solid var(--color-textMuted);
        z-index: 100;
        min-width: 150px;

    }
    
    .theme-option {
        display: block;
        width: 100%;
        padding: 6px 12px;
        background: none;
        border: none;
        color: var(--color-text);
        font-size: 11px;
        text-align: left;
        cursor: pointer;
        transition: var(--transition);
        font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
    }
    
    .theme-option:hover {
        background-color: var(--color-hover);
    }
    
    .theme-option.active {
        background-color: var(--color-primary);
        color: var(--color-selectedText);
        font-weight: 600;
    }

    .theme-section-header {
        padding: 8px 12px 4px 12px;
        font-size: 10px;
        font-weight: 700;
        color: var(--color-textMuted);
        text-transform: uppercase;
        letter-spacing: 0.5px;
        user-select: none;
    }

    .theme-section-header:not(:first-child) {
        margin-top: 8px;
    }

    .pink-paw-tab.active {
        background-color: #ff69b4;
        color: white;
    }

    .pink-paw-tab:hover:not(.active) {
        color: #ff69b4;
    }

    .paw-emoji {
        color: transparent;
        text-shadow: 0 0 0 #ff69b4;
        font-size: 13px;
        vertical-align: middle;
        display: inline-block;
        margin-right: 3px;
    }

    .pink-paw-tab.active .paw-emoji {
        color: transparent;
        text-shadow: 0 0 0 white;
    }
</style>

