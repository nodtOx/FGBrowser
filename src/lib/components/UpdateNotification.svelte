<script lang="ts">
  import { appUpdateStatus, checkForUpdates, downloadAndInstall } from '$lib/stores/appUpdater';
  import { onDestroy } from 'svelte';

  let showNotification = false;
  let dismissedVersion = '';

  const unsubscribe = appUpdateStatus.subscribe((status) => {
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
          <button class="btn-update" on:click={handleUpdate}> Update Now </button>
          <button class="btn-dismiss" on:click={dismiss}> Later </button>
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
  <div class="checking-notification">Checking for updates...</div>
{/if}

<style>
  .update-notification {
    position: fixed;
    bottom: 48px;
    right: 20px;
    background: var(--color-primary);
    color: var(--color-background);
    border-radius: var(--border-radius);
    border-color: var(--color-text);
    border-width: 2px;
    border-style: solid;
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
    background: var(--color-background);
    color: var(--color-primary);
    border: none;
    padding: 8px 16px;
    border-radius: var(--border-radius);
    font-weight: 600;
    cursor: pointer;
    font-size: 13px;
    transition: var(--transition);
  }

  .btn-update:hover {
    background: var(--color-hover);
    color: var(--color-text);
  }

  .btn-dismiss {
    background: transparent;
    color: var(--color-background);
    border: 1px solid var(--color-background);
    padding: 8px 16px;
    border-radius: var(--border-radius);
    cursor: pointer;
    font-size: 13px;
    transition: var(--transition);
  }

  .btn-dismiss:hover {
    background: var(--color-backgroundSecondary);
    color: var(--color-text);
  }

  .download-progress {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 150px;
  }

  .progress-bar {
    height: 6px;
    background: var(--color-backgroundSecondary);
    border-radius: var(--border-radius);
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: var(--color-background);
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
    background: var(--color-error);
    color: var(--color-background);
    border-radius: var(--border-radius);
    font-size: 12px;
  }

  .checking-notification {
    position: fixed;
    bottom: 20px;
    right: 20px;
    background: var(--color-backgroundTertiary);
    color: var(--color-text);
    padding: 12px 20px;
    border-radius: var(--border-radius);
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
