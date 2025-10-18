<script lang="ts">
  import { appUpdateStatus, checkForUpdates, downloadAndInstall } from '$lib/stores/appUpdater';
  import { onDestroy } from 'svelte';

  let showNotification = false;
  let dismissedVersion = '';

  const unsubscribe = appUpdateStatus.subscribe(status => {
    if (status.available && status.latestVersion !== dismissedVersion) {
      showNotification = true;
    }
  });

  onDestroy(() => {
    unsubscribe();
  });

  function dismiss() {
    dismissedVersion = $appUpdateStatus.latestVersion;
    showNotification = false;
  }

  async function handleUpdate() {
    await downloadAndInstall();
  }

  async function handleCheckNow() {
    await checkForUpdates(false);
  }
</script>

{#if showNotification && $appUpdateStatus.available}
  <div class="update-notification">
    <div class="update-content">
      <div class="update-icon">🎉</div>
      <div class="update-info">
        <div class="update-title">Update Available</div>
        <div class="update-version">
          v{$appUpdateStatus.latestVersion} is ready to install
        </div>
      </div>
      <div class="update-actions">
        {#if $appUpdateStatus.downloading}
          <div class="download-progress">
            <div class="progress-bar">
              <div class="progress-fill" style="width: {$appUpdateStatus.downloadProgress}%"></div>
            </div>
            <div class="progress-text">{$appUpdateStatus.downloadProgress}%</div>
          </div>
        {:else if $appUpdateStatus.downloaded}
          <span class="ready-text">Ready to install!</span>
        {:else}
          <button class="btn-update" on:click={handleUpdate}>
            Update Now
          </button>
          <button class="btn-dismiss" on:click={dismiss}>
            Later
          </button>
        {/if}
      </div>
    </div>
    {#if $appUpdateStatus.error}
      <div class="update-error">
        Error: {$appUpdateStatus.error}
      </div>
    {/if}
  </div>
{/if}

{#if $appUpdateStatus.checking}
  <div class="checking-notification">
    Checking for updates...
  </div>
{/if}

<style>
  .update-notification {
    position: fixed;
    top: 20px;
    right: 20px;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    border-radius: 12px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
    padding: 16px 20px;
    z-index: 10000;
    min-width: 400px;
    animation: slideIn 0.3s ease-out;
  }

  @keyframes slideIn {
    from {
      transform: translateX(120%);
      opacity: 0;
    }
    to {
      transform: translateX(0);
      opacity: 1;
    }
  }

  .update-content {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .update-icon {
    font-size: 32px;
    flex-shrink: 0;
  }

  .update-info {
    flex: 1;
  }

  .update-title {
    font-size: 16px;
    font-weight: 600;
    margin-bottom: 4px;
  }

  .update-version {
    font-size: 13px;
    opacity: 0.9;
  }

  .update-actions {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .btn-update {
    background: white;
    color: #667eea;
    border: none;
    padding: 8px 16px;
    border-radius: 6px;
    font-weight: 600;
    cursor: pointer;
    font-size: 13px;
    transition: all 0.2s;
  }

  .btn-update:hover {
    transform: scale(1.05);
    box-shadow: 0 4px 12px rgba(255, 255, 255, 0.3);
  }

  .btn-dismiss {
    background: transparent;
    color: white;
    border: 1px solid rgba(255, 255, 255, 0.3);
    padding: 8px 16px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
    transition: all 0.2s;
  }

  .btn-dismiss:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  .download-progress {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 150px;
  }

  .progress-bar {
    height: 6px;
    background: rgba(255, 255, 255, 0.3);
    border-radius: 3px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: white;
    transition: width 0.3s ease;
  }

  .progress-text {
    font-size: 12px;
    text-align: center;
    opacity: 0.9;
  }

  .ready-text {
    font-weight: 600;
    font-size: 14px;
  }

  .update-error {
    margin-top: 12px;
    padding: 8px 12px;
    background: rgba(255, 0, 0, 0.2);
    border-radius: 6px;
    font-size: 12px;
  }

  .checking-notification {
    position: fixed;
    bottom: 20px;
    right: 20px;
    background: rgba(0, 0, 0, 0.8);
    color: white;
    padding: 12px 20px;
    border-radius: 8px;
    font-size: 14px;
    z-index: 9999;
    animation: fadeIn 0.3s ease-out;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }
</style>

