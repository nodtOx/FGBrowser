import { invoke } from '@tauri-apps/api/core';
import { derived, writable } from 'svelte/store';

// Download status types
export type DownloadStatus = 'queued' | 'downloading' | 'seeding' | 'paused' | 'completed' | 'error';

// Download interface matching backend schema
export interface Download {
  id: number;
  repack_id: number;
  game_title: string; // Denormalized for display
  magnet_link: string;
  info_hash: string;
  status: DownloadStatus;
  save_path: string;
  total_size: number; // bytes
  downloaded_bytes: number;
  uploaded_bytes: number;
  download_speed: number; // bytes/sec
  upload_speed: number; // bytes/sec
  progress: number; // 0.0 to 100.0
  peers: number;
  seeds: number;
  eta_seconds: number | null;
  error_message: string | null;
  started_at: string | null;
  completed_at: string | null;
}

// Download settings
export interface DownloadSettings {
  download_path: string;
  max_download_speed: number; // KB/s, 0 = unlimited
  max_upload_speed: number; // KB/s, 0 = unlimited
  seed_ratio_limit: number; // Default 2.0
}

// Stores
export const downloads = writable<Download[]>([]);
export const downloadSettings = writable<DownloadSettings>({
  download_path: '',
  max_download_speed: 0,
  max_upload_speed: 0,
  seed_ratio_limit: 2.0,
});

// Derived stores for filtering
export const activeDownloads = derived(downloads, ($downloads) =>
  $downloads.filter((d) => d.status === 'downloading' || d.status === 'queued'),
);

export const completedDownloads = derived(downloads, ($downloads) =>
  $downloads.filter((d) => d.status === 'completed'),
);

export const seedingDownloads = derived(downloads, ($downloads) => $downloads.filter((d) => d.status === 'seeding'));

export const pausedDownloads = derived(downloads, ($downloads) => $downloads.filter((d) => d.status === 'paused'));

export const errorDownloads = derived(downloads, ($downloads) => $downloads.filter((d) => d.status === 'error'));

// Total download/upload speeds
export const totalDownloadSpeed = derived(downloads, ($downloads) =>
  $downloads.filter((d) => d.status === 'downloading').reduce((sum, d) => sum + d.download_speed, 0),
);

export const totalUploadSpeed = derived(downloads, ($downloads) =>
  $downloads
    .filter((d) => d.status === 'downloading' || d.status === 'seeding')
    .reduce((sum, d) => sum + d.upload_speed, 0),
);

// Utility functions
export function formatSpeed(bytesPerSec: number): string {
  if (bytesPerSec === 0) return '0 B/s';
  const k = 1024;
  const sizes = ['B/s', 'KB/s', 'MB/s', 'GB/s'];
  const i = Math.floor(Math.log(bytesPerSec) / Math.log(k));
  return parseFloat((bytesPerSec / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

export function formatETA(seconds: number | null): string {
  if (seconds === null || seconds === 0) return 'Unknown';

  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = Math.floor(seconds % 60);

  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  } else if (minutes > 0) {
    return `${minutes}m ${secs}s`;
  } else {
    return `${secs}s`;
  }
}

export function formatProgress(progress: number): string {
  return progress.toFixed(1) + '%';
}

// Actions
export async function loadDownloads() {
  try {
    const result = await invoke<Download[]>('get_downloads');
    downloads.set(result);
  } catch (error) {
    console.error('Failed to load downloads:', error);
  }
}

export async function addDownload(magnetLink: string, repackId: number, savePath: string) {
  try {
    const download = await invoke<Download>('add_download', {
      magnet: magnetLink,
      repackId,
      savePath,
    });
    // Reload downloads to get updated list
    await loadDownloads();
    return download;
  } catch (error) {
    console.error('Failed to add download:', error);
    throw error;
  }
}

export async function pauseDownload(infoHash: string) {
  try {
    await invoke('pause_download', { infoHash });
    await loadDownloads();
  } catch (error) {
    console.error('Failed to pause download:', error);
  }
}

export async function resumeDownload(infoHash: string) {
  try {
    await invoke('resume_download', { infoHash });
    await loadDownloads();
  } catch (error) {
    console.error('Failed to resume download:', error);
  }
}

export async function removeDownload(infoHash: string, deleteFiles: boolean) {
  try {
    await invoke('remove_download', { infoHash, deleteFiles });
    await loadDownloads();
  } catch (error) {
    console.error('Failed to remove download:', error);
  }
}

export async function setSpeedLimits(downloadKbps: number, uploadKbps: number) {
  try {
    await invoke('set_speed_limits', { downloadKbps, uploadKbps });
    downloadSettings.update((s) => ({
      ...s,
      max_download_speed: downloadKbps,
      max_upload_speed: uploadKbps,
    }));
  } catch (error) {
    console.error('Failed to set speed limits:', error);
  }
}

export async function openDownloadFolder(path: string) {
  try {
    await invoke('open_download_folder', { path });
  } catch (error) {
    console.error('Failed to open folder:', error);
  }
}
