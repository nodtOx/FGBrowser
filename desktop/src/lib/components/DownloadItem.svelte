<script lang="ts">
  import type { Download } from '$lib/stores/downloads';
  import {
    formatBytes,
    formatETA,
    formatProgress,
    formatSpeed,
    openDownloadFolder,
    pauseDownload,
    removeDownload,
    resumeDownload,
  } from '$lib/stores/downloads';

  export let download: Download;

  let showDetails = false;
  let showRemoveConfirm = false;

  function toggleDetails() {
    showDetails = !showDetails;
  }

  async function handlePause() {
    await pauseDownload(download.info_hash);
  }

  async function handleResume() {
    await resumeDownload(download.info_hash);
  }

  async function handleRemove(deleteFiles: boolean) {
    await removeDownload(download.info_hash, deleteFiles);
    showRemoveConfirm = false;
  }

  async function handleOpenFolder() {
    await openDownloadFolder(download.save_path);
  }

  function getStatusColor(status: string): string {
    switch (status) {
      case 'downloading':
        return '#4CAF50';
      case 'seeding':
        return '#2196F3';
      case 'completed':
        return '#00BCD4';
      case 'paused':
        return '#FF9800';
      case 'error':
        return '#F44336';
      case 'queued':
        return '#9E9E9E';
      default:
        return '#9E9E9E';
    }
  }

  $: statusColor = getStatusColor(download.status);
</script>

<div class="download-item">
  <div
    class="download-header"
    on:click={toggleDetails}
    on:keydown={(e) => e.key === 'Enter' && toggleDetails()}
    role="button"
    tabindex="0"
  >
    <div class="download-info">
      <div class="download-title-row">
        <span class="status-badge" style="background-color: {statusColor}">
          {download.status.toUpperCase()}
        </span>
        <h3 class="download-title">{download.game_title}</h3>
      </div>

      <div class="download-stats">
        {#if download.status === 'downloading'}
          <span class="stat">{formatBytes(download.downloaded_bytes)} / {formatBytes(download.total_size)}</span>
          <span class="stat">↓ {formatSpeed(download.download_speed)}</span>
          <span class="stat">↑ {formatSpeed(download.upload_speed)}</span>
          <span class="stat">Peers: {download.peers} ({download.seeds} seeds)</span>
          <span class="stat">ETA: {formatETA(download.eta_seconds)}</span>
        {:else if download.status === 'seeding'}
          <span class="stat">{formatBytes(download.total_size)}</span>
          <span class="stat">↑ {formatSpeed(download.upload_speed)}</span>
          <span class="stat">Peers: {download.peers}</span>
          <span class="stat">Ratio: {(download.uploaded_bytes / download.total_size).toFixed(2)}</span>
        {:else if download.status === 'completed'}
          <span class="stat">{formatBytes(download.total_size)}</span>
          <span class="stat">Completed: {new Date(download.completed_at || '').toLocaleString()}</span>
        {:else if download.status === 'error'}
          <span class="stat error-text">{download.error_message || 'Unknown error'}</span>
        {:else if download.status === 'paused'}
          <span class="stat"
            >{formatProgress(download.progress)} - {formatBytes(download.downloaded_bytes)} / {formatBytes(
              download.total_size,
            )}</span
          >
        {:else if download.status === 'queued'}
          <span class="stat">Waiting to start...</span>
        {/if}
      </div>
    </div>

    <div class="download-actions">
      {#if download.status === 'downloading' || download.status === 'seeding'}
        <button class="action-btn pause-btn" on:click|stopPropagation={handlePause} title="Pause">⏸</button>
      {:else if download.status === 'paused' || download.status === 'error'}
        <button class="action-btn resume-btn" on:click|stopPropagation={handleResume} title="Resume">▶</button>
      {/if}

      {#if download.status === 'completed' || download.status === 'seeding'}
        <button class="action-btn folder-btn" on:click|stopPropagation={handleOpenFolder} title="Open Folder">
          📁
        </button>
      {/if}

      {#if !showRemoveConfirm}
        <button
          class="action-btn remove-btn"
          on:click|stopPropagation={() => (showRemoveConfirm = true)}
          title="Remove"
        >
          🗑
        </button>
      {:else}
        <div class="remove-confirm">
          <button
            class="action-btn confirm-btn"
            on:click|stopPropagation={() => handleRemove(false)}
            title="Remove from list"
          >
            Keep Files
          </button>
          <button
            class="action-btn danger-btn"
            on:click|stopPropagation={() => handleRemove(true)}
            title="Delete files"
          >
            Delete Files
          </button>
          <button class="action-btn" on:click|stopPropagation={() => (showRemoveConfirm = false)} title="Cancel"
            >✕</button
          >
        </div>
      {/if}
    </div>
  </div>

  <div class="progress-container">
    <div class="progress-bar">
      <div class="progress-fill" style="width: {download.progress}%"></div>
    </div>
    <span class="progress-text">{formatProgress(download.progress)}</span>
  </div>

  {#if showDetails}
    <div class="download-details">
      <div class="detail-row">
        <span class="detail-label">Save Path:</span>
        <span class="detail-value">{download.save_path}</span>
      </div>
      <div class="detail-row">
        <span class="detail-label">Info Hash:</span>
        <span class="detail-value hash">{download.info_hash}</span>
      </div>
      <div class="detail-row">
        <span class="detail-label">Started:</span>
        <span class="detail-value">
          {download.started_at ? new Date(download.started_at).toLocaleString() : 'N/A'}
        </span>
      </div>
      {#if download.completed_at}
        <div class="detail-row">
          <span class="detail-label">Completed:</span>
          <span class="detail-value">{new Date(download.completed_at).toLocaleString()}</span>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .download-item {
    background-color: var(--color-backgroundSecondary);
    border: 1px solid var(--color-border);
    border-radius: 4px;
    overflow: hidden;
  }

  .download-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    cursor: pointer;
    transition: var(--transition);
  }

  .download-header:hover {
    background-color: var(--color-hover);
  }

  .download-info {
    flex: 1;
    min-width: 0;
  }

  .download-title-row {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 8px;
  }

  .status-badge {
    padding: 2px 8px;
    border-radius: 3px;
    font-size: 10px;
    font-weight: 700;
    color: white;
    letter-spacing: 0.5px;
  }

  .download-title {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    color: var(--color-text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .download-stats {
    display: flex;
    gap: 16px;
    flex-wrap: wrap;
  }

  .stat {
    font-size: 12px;
    color: var(--color-textSecondary);
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  }

  .stat.error-text {
    color: var(--color-error);
  }

  .download-actions {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .action-btn {
    padding: 6px 12px;
    background-color: var(--color-backgroundTertiary);
    border: 1px solid var(--color-border);
    color: var(--color-text);
    cursor: pointer;
    font-size: 14px;
    transition: var(--transition);
  }

  .action-btn:hover {
    background-color: var(--color-hover);
  }

  .pause-btn:hover {
    background-color: #ff9800;
    color: white;
    border-color: #ff9800;
  }

  .resume-btn:hover {
    background-color: #4caf50;
    color: white;
    border-color: #4caf50;
  }

  .remove-btn:hover {
    background-color: var(--color-error);
    color: white;
    border-color: var(--color-error);
  }

  .folder-btn:hover {
    background-color: var(--color-primary);
    color: white;
    border-color: var(--color-primary);
  }

  .remove-confirm {
    display: flex;
    gap: 4px;
  }

  .confirm-btn:hover {
    background-color: var(--color-primary);
    color: white;
  }

  .danger-btn {
    background-color: var(--color-error);
    color: white;
  }

  .danger-btn:hover {
    opacity: 0.9;
  }

  .progress-container {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 0 16px 12px 16px;
  }

  .progress-bar {
    flex: 1;
    height: 8px;
    background-color: var(--color-background);
    border-radius: 4px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, var(--color-primary) 0%, #4caf50 100%);
    transition: width 0.3s ease;
  }

  .progress-text {
    font-size: 12px;
    color: var(--color-textSecondary);
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
    min-width: 50px;
    text-align: right;
  }

  .download-details {
    padding: 12px 16px;
    background-color: var(--color-background);
    border-top: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .detail-row {
    display: flex;
    gap: 12px;
    font-size: 12px;
  }

  .detail-label {
    color: var(--color-textSecondary);
    font-weight: 600;
    min-width: 100px;
  }

  .detail-value {
    color: var(--color-text);
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .detail-value.hash {
    font-size: 10px;
    word-break: break-all;
  }
</style>
