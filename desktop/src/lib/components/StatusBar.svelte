<script lang="ts">
    import { games } from '$lib/stores/games';
    import { invoke } from '@tauri-apps/api/core';
    import { onMount } from 'svelte';
    
    interface DiskInfo {
        total: number;
        free: number;
        used: number;
    }
    
    let diskInfo: DiskInfo = { total: 0, free: 0, used: 0 };
    let downloadSpeed = '0 KB/s';
    let uploadSpeed = '0 KB/s';
    
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
        // Refresh disk info every 30 seconds
        const interval = setInterval(loadDiskInfo, 30000);
        return () => clearInterval(interval);
    });
</script>

<div class="status-bar">
    <div class="status-left">
        <div class="status-item">
            <i class="fas fa-database"></i>
            <span>{$games.length} games</span>
        </div>
        
        <div class="status-item">
            <i class="fas fa-download"></i>
            <span>{downloadSpeed}</span>
        </div>
        
        <div class="status-item">
            <i class="fas fa-upload"></i>
            <span>{uploadSpeed}</span>
        </div>
    </div>
    
    <div class="status-center">
        <div class="shortcuts">
            <span class="shortcut">Search: /</span>
            <span class="shortcut">Navigate: ↑↓</span>
            <span class="shortcut">Open: Enter</span>
            <span class="shortcut">Copy: C</span>
            <span class="shortcut">Refresh: R</span>
            <span class="shortcut">Quit: Q</span>
        </div>
    </div>
    
    <div class="status-right">
        <div class="status-item">
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
        font-size: calc(var(--base-font-size) * 0.8);
        width: 14px;
        text-align: center;
    }
    
    .shortcuts {
        display: flex;
        gap: 16px;
        align-items: center;
    }
    
    .shortcut {
        display: flex;
        align-items: center;
        color: var(--color-textSecondary);
        font-size: calc(var(--base-font-size) * 0.8);
        font-weight: 500;
    }
</style>

