import { relaunch } from '@tauri-apps/plugin-process';
import { check } from '@tauri-apps/plugin-updater';
import { writable } from 'svelte/store';

export interface AppUpdateStatus {
  checking: boolean;
  available: boolean;
  downloading: boolean;
  downloaded: boolean;
  error: string | null;
  currentVersion: string;
  latestVersion: string;
  downloadProgress: number;
}

export const appUpdateStatus = writable<AppUpdateStatus>({
  checking: false,
  available: false,
  downloading: false,
  downloaded: false,
  error: null,
  currentVersion: '',
  latestVersion: '',
  downloadProgress: 0,
});

export async function checkForUpdates(silent = false) {
  if (!silent) {
    appUpdateStatus.update((s) => ({ ...s, checking: true, error: null }));
  }

  try {
    const update = await check();

    if (update) {
      console.log(`Update available: ${update.currentVersion} -> ${update.version}`);

      appUpdateStatus.update((s) => ({
        ...s,
        checking: false,
        available: true,
        currentVersion: update.currentVersion,
        latestVersion: update.version,
      }));

      return update;
    } else {
      console.log('No updates available');

      if (!silent) {
        appUpdateStatus.update((s) => ({
          ...s,
          checking: false,
          available: false,
        }));
      }

      return null;
    }
  } catch (error) {
    console.error('Failed to check for updates:', error);

    appUpdateStatus.update((s) => ({
      ...s,
      checking: false,
      error: error instanceof Error ? error.message : 'Unknown error',
    }));

    return null;
  }
}

export async function downloadAndInstall() {
  appUpdateStatus.update((s) => ({ ...s, downloading: true, error: null }));

  try {
    const update = await check();

    if (!update) {
      appUpdateStatus.update((s) => ({
        ...s,
        downloading: false,
        error: 'No update available',
      }));
      return;
    }

    // Download with progress tracking
    let downloaded = 0;
    let contentLength = 0;

    await update.downloadAndInstall((event) => {
      switch (event.event) {
        case 'Started':
          contentLength = event.data.contentLength || 0;
          console.log(`Download started, size: ${contentLength} bytes`);
          break;
        case 'Progress':
          downloaded += event.data.chunkLength;
          const progress = contentLength > 0 ? Math.round((downloaded / contentLength) * 100) : 0;

          appUpdateStatus.update((s) => ({
            ...s,
            downloadProgress: progress,
          }));

          console.log(`Downloaded ${downloaded}/${contentLength} bytes (${progress}%)`);
          break;
        case 'Finished':
          console.log('Download finished');
          appUpdateStatus.update((s) => ({
            ...s,
            downloading: false,
            downloaded: true,
            downloadProgress: 100,
          }));
          break;
      }
    });

    console.log('Update downloaded and installed, restarting...');

    // Relaunch the app to apply the update
    await relaunch();
  } catch (error) {
    console.error('Failed to download update:', error);

    appUpdateStatus.update((s) => ({
      ...s,
      downloading: false,
      downloaded: false,
      error: error instanceof Error ? error.message : 'Download failed',
    }));
  }
}

// Check for updates on startup (silent check)
export async function checkForUpdatesOnStartup() {
  // Wait a few seconds after startup to avoid blocking
  setTimeout(async () => {
    await checkForUpdates(true);
  }, 5000);
}
