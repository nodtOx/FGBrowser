<script lang="ts">
    import { DISK_INFO_REFRESH_INTERVAL_MS } from '$lib/constants';
    import { totalGamesCount } from '$lib/stores/games';
    import { browseView, currentPage, focusedPanel } from '$lib/stores/navigation';
    import { invoke } from '@tauri-apps/api/core';
    import { onMount } from 'svelte';
    import Kbd from './Kbd.svelte';
    
    interface DiskInfo {
        total: number;
        free: number;
        used: number;
    }
    
    let diskInfo: DiskInfo = { total: 0, free: 0, used: 0 };
    
    function formatBytes(bytes: number): string {
        if (bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
    }
    
    async function loadDiskInfo() {
        try {
            diskInfo = await invoke<DiskInfo>('get_disk_info');
        } catch (error) {
            console.error('Failed to load disk info:', error);
        }
    }
    
    onMount(() => {
        loadDiskInfo();
        // Refresh disk info periodically
        const interval = setInterval(loadDiskInfo, DISK_INFO_REFRESH_INTERVAL_MS);
        return () => clearInterval(interval);
    });
</script>

<div class="status-bar">
    <div class="status-left">
        <div class="status-item">
            <i class="fas fa-database"></i>
            <span>{$totalGamesCount.toLocaleString()} games</span>
        </div>
        
        <!-- <div class="status-item">
            <i class="fas fa-download"></i>
            <span>{formatSpeed($totalDownloadSpeed)}</span>
        </div>
        
        <div class="status-item">
            <i class="fas fa-upload"></i>
            <span>{formatSpeed($totalUploadSpeed)}</span>
        </div> -->
    </div>
    
    <div class="status-center">
        <div class="shortcuts">
            {#if $currentPage === 'browse' && $browseView === 'details'}
                <div class="shortcut-item">
                    <span class="shortcut-label">Back</span>
                    <Kbd keys="Esc" />
                </div>
                <div class="shortcut-item">
                    <span class="shortcut-label">Navigate</span>
                    <Kbd keys="Up" />
                    <Kbd keys="Down" />
                </div>
                <div class="shortcut-item">
                    <span class="shortcut-label">Download</span>
                    <Kbd keys="Enter" />
                </div>
            {:else if $currentPage === 'browse'}
                <div class="shortcut-item">
                    <span class="shortcut-label">Navigate</span>
                    <Kbd keys="Up" />
                    <Kbd keys="Down" />
                </div>
                {#if $focusedPanel === 'gamelist' || $focusedPanel === 'categories'}
                <div class="shortcut-item">
                    <span class="shortcut-label">Jump</span>
                    <Kbd keys="PgUp" />
                    <Kbd keys="PgDn" />
                </div>
                {/if}
                <div class="shortcut-item">
                    <span class="shortcut-label">Switch Panel</span>
                    <Kbd keys="Tab" />
                </div>
                {#if $focusedPanel === 'categories'}
                <div class="shortcut-item">
                    <span class="shortcut-label">Toggle All</span>
                    <Kbd keys="M" />
                </div>
                {/if}
                {#if $focusedPanel === 'gamelist'}
                <div class="shortcut-item">
                    <span class="shortcut-label">Open</span>
                    <Kbd keys="Enter" />
                </div>
                {:else if $focusedPanel !== 'search'}
                <div class="shortcut-item">
                    <span class="shortcut-label">Select</span>
                    <Kbd keys="Enter" />
                </div>
                {/if}
            {/if}
        
            <div class="shortcut-item">
                <span class="shortcut-label">Navigate Tabs</span>
                <Kbd keys={['Mod', '[']} />
                <Kbd keys={['Mod', ']']} />
            </div>
        
        </div>
    </div>
    
    <div class="status-right">
        <div class="status-item free-disk-space">
            <i class="fas fa-hdd"></i>
            <span>Free: {formatBytes(diskInfo.free)}</span>
        </div>
    </div>
</div>

<style>
    .status-bar {
        height: 28px;
        background-color: var(--color-backgroundTertiary);
        border-top: 1px solid var(--color-border);
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0 16px;
        font-size: calc(var(--base-font-size) * 0.85);
        font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
    }
    
    .status-left {
        display: flex;
        gap: 20px;
        align-items: center;
    }
    
    .status-center {
        flex: 1;
        display: flex;
        justify-content: center;
        align-items: center;
    }
    
    .status-right {
        display: flex;
        align-items: center;
    }
    
    .status-item {
        display: flex;
        align-items: center;
        gap: 6px;
        color: var(--color-text);
        font-weight: 500;
    }
    
    .status-item i {
        color: var(--color-textSecondary);
        /* font-size: calc(var(--base-font-size) * 0.8); */
        width: 14px;
        text-align: center;
    }
    
    .shortcuts {
        display: flex;
        gap: 20px;
        align-items: center;
    }
    
    .shortcut-item {
        display: flex;
        align-items: center;
        gap: 6px;
    }
    
    .shortcut-label {
        color: var(--color-textSecondary);
        /* font-size: calc(var(--base-font-size) * 0.8); */
        font-weight: 500;
    }
</style>

