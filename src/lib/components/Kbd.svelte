<script lang="ts">
    export let keys: string | string[];
    
    // Detect OS
    const isMac = typeof navigator !== 'undefined' && navigator.platform.toUpperCase().indexOf('MAC') >= 0;
    
    // Convert single string to array for consistent handling
    $: keyArray = Array.isArray(keys) ? keys : [keys];
    
    // Format key for platform
    // Use "Mod" as a special key that converts to Cmd on Mac, Ctrl elsewhere
    // Use "Ctrl" or "Cmd" directly if you need a specific key on all platforms
    function formatKey(key: string): string {
        if (key.toLowerCase() === 'mod') {
            return isMac ? 'Cmd' : 'Ctrl';
        }
        return key;
    }
</script>

<span class="kbd-group">
    {#each keyArray as key, index}
        {#if index > 0}
            <span class="kbd-separator">+</span>
        {/if}
        <kbd class="kbd">{formatKey(key)}</kbd>
    {/each}
</span>

<style>
    .kbd-group {
        display: inline-flex;
        align-items: center;
        gap: 4px;
    }
    
    .kbd {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        min-width: 20px;
        height: 18px;
        padding: 0 6px;
        font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
        font-size: calc(var(--base-font-size) * 0.75);
        font-weight: 600;
        line-height: 1;
        color: var(--color-text);
        background-color: var(--color-backgroundTertiary);
        border: 1px solid var(--color-border);
        border-radius: 3px;
        box-shadow: 
            0 1px 0 0 var(--color-border),
            0 2px 0 0 rgba(0, 0, 0, 0.08);
        text-transform: uppercase;
    }
    
    .kbd-separator {
        font-size: calc(var(--base-font-size) * 0.7);
        color: var(--color-textSecondary);
        font-weight: 500;
    }
</style>

