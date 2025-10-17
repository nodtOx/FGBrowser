<script lang="ts">
  import {
    activeDownloads,
    completedDownloads,
    downloads,
    errorDownloads,
    formatSpeed,
    loadDownloads,
    pausedDownloads,
    seedingDownloads,
    totalDownloadSpeed,
    totalUploadSpeed,
  } from '$lib/stores/downloads';
  import { onDestroy, onMount } from 'svelte';
  import DownloadItem from './DownloadItem.svelte';
  import DownloadsSidebar from './DownloadsSidebar.svelte';

  let selectedFilter: 'all' | 'active' | 'completed' | 'seeding' | 'paused' | 'error' = 'all';

  $: filteredDownloads =
    selectedFilter === 'all'
      ? $downloads
      : selectedFilter === 'active'
        ? $activeDownloads
        : selectedFilter === 'completed'
          ? $completedDownloads
          : selectedFilter === 'seeding'
            ? $seedingDownloads
            : selectedFilter === 'paused'
              ? $pausedDownloads
              : $errorDownloads;

  function startPolling() {
    // Poll every 2 seconds for active downloads
    pollInterval = setInterval(async () => {
      if ($activeDownloads.length > 0 || $seedingDownloads.length > 0) {
        await loadDownloads();
      }
    }, 2000);
  }

  function stopPolling() {
    if (pollInterval) {
      clearInterval(pollInterval);
      pollInterval = null;
    }
  }

  let pollInterval: any;

  onMount(async () => {
    // Try to load from backend, if it fails or is empty, use dummy data
    try {
      await loadDownloads();
    } catch (error) {
      console.log('Backend not ready, using dummy data');
    }
    
    // Add dummy downloads for testing if no downloads exist
    if ($downloads.length === 0) {
      downloads.set([
        {
          id: 1,
          repack_id: 1,
          game_title: 'Cyberpunk 2077',
          magnet_link: 'magnet:?xt=urn:btih:123456',
          info_hash: '123456',
          status: 'downloading',
          save_path: '/Users/Downloads/Cyberpunk 2077',
          total_size: 60 * 1024 * 1024 * 1024, // 60 GB
          downloaded_bytes: 25 * 1024 * 1024 * 1024, // 25 GB
          uploaded_bytes: 10 * 1024 * 1024 * 1024, // 10 GB
          download_speed: 5 * 1024 * 1024, // 5 MB/s
          upload_speed: 1 * 1024 * 1024, // 1 MB/s
          progress: 41.7,
          peers: 45,
          seeds: 12,
          eta_seconds: 7000,
          error_message: null,
          started_at: new Date().toISOString(),
          completed_at: null,
        },
        {
          id: 2,
          repack_id: 2,
          game_title: 'Elden Ring',
          magnet_link: 'magnet:?xt=urn:btih:789012',
          info_hash: '789012',
          status: 'seeding',
          save_path: '/Users/Downloads/Elden Ring',
          total_size: 50 * 1024 * 1024 * 1024, // 50 GB
          downloaded_bytes: 50 * 1024 * 1024 * 1024, // 50 GB
          uploaded_bytes: 120 * 1024 * 1024 * 1024, // 120 GB (2.4 ratio)
          download_speed: 0,
          upload_speed: 500 * 1024, // 500 KB/s
          progress: 100.0,
          peers: 8,
          seeds: 0,
          eta_seconds: null,
          error_message: null,
          started_at: new Date(Date.now() - 7200000).toISOString(),
          completed_at: new Date(Date.now() - 3600000).toISOString(),
        },
        {
          id: 3,
          repack_id: 3,
          game_title: 'Red Dead Redemption 2',
          magnet_link: 'magnet:?xt=urn:btih:345678',
          info_hash: '345678',
          status: 'paused',
          save_path: '/Users/Downloads/Red Dead Redemption 2',
          total_size: 120 * 1024 * 1024 * 1024, // 120 GB
          downloaded_bytes: 45 * 1024 * 1024 * 1024, // 45 GB
          uploaded_bytes: 5 * 1024 * 1024 * 1024, // 5 GB
          download_speed: 0,
          upload_speed: 0,
          progress: 37.5,
          peers: 0,
          seeds: 0,
          eta_seconds: null,
          error_message: null,
          started_at: new Date(Date.now() - 10800000).toISOString(),
          completed_at: null,
        },
        {
          id: 4,
          repack_id: 4,
          game_title: 'The Witcher 3: Wild Hunt',
          magnet_link: 'magnet:?xt=urn:btih:901234',
          info_hash: '901234',
          status: 'completed',
          save_path: '/Users/Downloads/The Witcher 3',
          total_size: 40 * 1024 * 1024 * 1024, // 40 GB
          downloaded_bytes: 40 * 1024 * 1024 * 1024, // 40 GB
          uploaded_bytes: 15 * 1024 * 1024 * 1024, // 15 GB
          download_speed: 0,
          upload_speed: 0,
          progress: 100.0,
          peers: 0,
          seeds: 0,
          eta_seconds: null,
          error_message: null,
          started_at: new Date(Date.now() - 86400000).toISOString(),
          completed_at: new Date(Date.now() - 80000000).toISOString(),
        },
        {
          id: 5,
          repack_id: 5,
          game_title: 'Grand Theft Auto V',
          magnet_link: 'magnet:?xt=urn:btih:567890',
          info_hash: '567890',
          status: 'error',
          save_path: '/Users/Downloads/GTA V',
          total_size: 95 * 1024 * 1024 * 1024, // 95 GB
          downloaded_bytes: 12 * 1024 * 1024 * 1024, // 12 GB
          uploaded_bytes: 0,
          download_speed: 0,
          upload_speed: 0,
          progress: 12.6,
          peers: 0,
          seeds: 0,
          eta_seconds: null,
          error_message: 'Tracker connection failed: timeout',
          started_at: new Date(Date.now() - 3600000).toISOString(),
          completed_at: null,
        },
      ]);
    }
    
    // Only start polling if we have real backend data
    // Comment out for now with dummy data
    // startPolling();
  });

  onDestroy(() => {
    stopPolling();
  });
</script>

<div class="downloads-page">
  <div class="downloads-layout">
    <DownloadsSidebar bind:selectedFilter />

    <div class="downloads-main">
      <div class="downloads-header">
        <h1>Downloads</h1>
        <div class="stats-summary">
          <span class="stat">↓ {formatSpeed($totalDownloadSpeed)}</span>
          <span class="stat">↑ {formatSpeed($totalUploadSpeed)}</span>
          <span class="stat">{$activeDownloads.length} active</span>
        </div>
      </div>

      <div class="downloads-content">
        {#if filteredDownloads.length === 0}
          <div class="empty-state">
            <p>No downloads {selectedFilter === 'all' ? 'yet' : `in ${selectedFilter}`}</p>
            <p class="hint">Add downloads from the game details modal</p>
          </div>
        {:else}
          <div class="downloads-list">
            {#each filteredDownloads as download (download.id)}
              <DownloadItem {download} />
            {/each}
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .downloads-page {
    height: 100%;
    display: flex;
    flex-direction: column;
    background-color: var(--color-background);
  }

  .downloads-layout {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .downloads-main {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .downloads-header {
    padding: 20px 24px;
    background-color: var(--color-backgroundSecondary);
    border-bottom: 1px solid var(--color-border);
  }

  .downloads-header h1 {
    margin: 0 0 12px 0;
    font-size: 24px;
    font-weight: 600;
    color: var(--color-text);
  }

  .stats-summary {
    display: flex;
    gap: 16px;
    align-items: center;
  }

  .stat {
    font-size: 14px;
    color: var(--color-textSecondary);
    font-weight: 500;
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  }

  .downloads-content {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--color-textSecondary);
    gap: 8px;
  }

  .empty-state p {
    margin: 0;
    font-size: 16px;
  }

  .empty-state .hint {
    font-size: 14px;
    opacity: 0.7;
  }

  .downloads-list {
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
</style>

