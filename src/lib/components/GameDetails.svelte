<script lang="ts">
  import { featureFlags } from '$lib/featureFlags';
  import { addDownload } from '$lib/stores/downloads';
  import { formatSize, openMagnetLink, selectedGame } from '$lib/stores/games';
  import { goBack } from '$lib/stores/navigation';
  import { invoke } from '@tauri-apps/api/core';
  import { onDestroy } from 'svelte';
  import CachedImage from './CachedImage.svelte';

  // Optional callback for custom back behavior (e.g., from Popular page)
  export let onBack: (() => void) | undefined = undefined;

  let selectedMagnetIndex = 0;

  // Reset selection when game changes
  $: if ($selectedGame) {
    selectedMagnetIndex = 0;
  }

  async function handleDownload(magnet: string) {
    if (!$selectedGame) return;

    try {
      // Open folder picker dialog using Tauri command
      const selectedPath = await invoke<string | null>('select_download_folder');

      if (!selectedPath) {
        // User cancelled
        return;
      }

      // Start download
      await addDownload(magnet, $selectedGame.id, selectedPath);
      alert('Download started!');
    } catch (error) {
      console.error('Failed to start download:', error);
      alert('Failed to start download: ' + error);
    }
  }

  async function handleOpenMagnet(magnet: string) {
    await openMagnetLink(magnet);
  }

  function handleBackAction() {
    if (onBack) {
      onBack();
    } else {
      goBack();
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    // Check if we're typing in an input field
    const target = event.target as HTMLElement;
    const isTyping = target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable;

    // Handle back navigation
    if (event.key === 'Escape' || event.key === 'Backspace') {
      if (!isTyping) {
        event.preventDefault();
        handleBackAction();
        return;
      }
    }

    if (isTyping || !$selectedGame || $selectedGame.magnet_links.length === 0) {
      return;
    }

    const magnetCount = $selectedGame.magnet_links.length;

    // Navigate between magnet links with arrow keys
    if (event.key === 'ArrowUp') {
      event.preventDefault();
      selectedMagnetIndex = selectedMagnetIndex > 0 ? selectedMagnetIndex - 1 : magnetCount - 1;
    } else if (event.key === 'ArrowDown') {
      event.preventDefault();
      selectedMagnetIndex = selectedMagnetIndex < magnetCount - 1 ? selectedMagnetIndex + 1 : 0;
    }
  }

  // Add keyboard listener when a game is selected
  // Only handles Escape/Backspace for back navigation and arrow keys for magnet selection
  $: {
    if ($selectedGame) {
      window.addEventListener('keydown', handleKeydown);
    } else {
      window.removeEventListener('keydown', handleKeydown);
    }
  }

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeydown);
  });
</script>

<div class="details-panel">
  {#if $selectedGame}
    <div class="details-header">
      <button class="back-button" on:click={handleBackAction} title="Go back (Esc or Backspace)"> ← Back </button>
      <h2 class="game-title">
        <a
          href={$selectedGame.url}
          target="_blank"
          rel="noopener noreferrer"
          class="game-title-link"
          title="Open FitGirl page"
        >
          {$selectedGame.title}
        </a>
      </h2>
    </div>

    <div class="details-content">
      {#if $selectedGame.image_url}
        <div class="cover-art">
          <CachedImage src={$selectedGame.image_url} alt={$selectedGame.title} className="cover-image" />
        </div>
      {/if}

      <div class="details-info">
        <div class="info-grid">
          <div class="info-item">
            <span class="info-label">Categories:</span>
            <div class="info-value">
              {#if $selectedGame.categories && $selectedGame.categories.length > 0}
                <div class="categories-list">
                  {#each $selectedGame.categories as category}
                    <span class="category-tag">{category.name}</span>
                  {/each}
                </div>
              {:else}
                N/A
              {/if}
            </div>
          </div>

          <div class="info-item">
            <span class="info-label">Company:</span>
            <span class="info-value">{$selectedGame.company || 'N/A'}</span>
          </div>

          <div class="info-item">
            <span class="info-label">Languages:</span>
            <span class="info-value">{$selectedGame.languages || 'N/A'}</span>
          </div>

          <div class="info-item">
            <span class="info-label">Original Size:</span>
            <span class="info-value">{$selectedGame.original_size || 'N/A'}</span>
          </div>

          <div class="info-item">
            <span class="info-label">Repack Size:</span>
            <span class="info-value">{formatSize($selectedGame.size)}</span>
          </div>

          <div class="info-item">
            <span class="info-label">Release Date:</span>
            <span class="info-value">{$selectedGame.date || 'N/A'}</span>
          </div>
        </div>

        {#if $selectedGame.magnet_links.length > 0}
          <div class="magnets-section">
            <h3>MAGNET LINKS ({$selectedGame.magnet_links.length}):</h3>
            <div class="magnet-table">
              {#each $selectedGame.magnet_links as link, index}
                <div
                  class="magnet-row"
                  class:selected={index === selectedMagnetIndex}
                  on:click={() => {
                    selectedMagnetIndex = index;
                    if (featureFlags.torrentClient) {
                      handleDownload(link.magnet);
                    } else {
                      handleOpenMagnet(link.magnet);
                    }
                  }}
                  on:keydown={(e) => {
                    if (e.key === 'Enter' || e.key === ' ') {
                      e.preventDefault();
                      selectedMagnetIndex = index;
                      if (featureFlags.torrentClient) {
                        handleDownload(link.magnet);
                      } else {
                        handleOpenMagnet(link.magnet);
                      }
                    }
                  }}
                  role="button"
                  tabindex={index === selectedMagnetIndex ? 0 : -1}
                >
                  <span class="magnet-index">{index + 1}</span>
                  <span class="magnet-source">{link.source}</span>
                  <span class="magnet-hint">
                    {index === selectedMagnetIndex ? 'Selected' : ''}
                  </span>
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    </div>
  {:else}
    <div class="no-selection">
      <p>Select a game to view details</p>
    </div>
  {/if}
</div>

<style>
  .details-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    background-color: var(--color-background);
    overflow-y: auto;
  }

  .details-header {
    padding: 16px;
    border-bottom: 1px solid var(--color-border);
    background-color: var(--color-backgroundSecondary);
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .back-button {
    background: none;
    border: none;
    color: var(--color-primary);
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    padding: 8px 12px;
    border-radius: var(--border-radius);
    transition: all 0.15s ease;
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .back-button:hover {
    background-color: var(--color-hover);
  }

  .game-title {
    font-size: 18px;
    font-weight: 600;
    color: var(--color-text);
    margin: 0;
    flex: 1;
    user-select: text;
  }

  .game-title-link {
    color: var(--color-text);
    text-decoration: none;
    cursor: pointer;
    transition: var(--transition);
    display: inline-block;
    text-decoration: underline;
  }

  .game-title-link:hover {
    color: var(--color-primary);
  }

  .details-content {
    padding: 16px;
    flex: 1;
    display: flex;
    gap: 20px;
  }

  .cover-art {
    flex-shrink: 0;
    text-align: center;
    width: 200px;
  }

  :global(.cover-art .cover-image) {
    width: 200px;
    height: auto;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
    display: block;
  }

  .details-info {
    flex: 1;
    min-width: 0;
  }

  .info-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 12px;
    margin-bottom: 20px;
  }

  .info-item {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .info-label {
    font-size: 12px;
    color: var(--color-textSecondary);
    font-weight: 500;
  }

  .info-value {
    font-size: 14px;
    color: var(--color-text);
    user-select: text;
    cursor: text;
  }

  .magnets-section {
    margin-top: 20px;
    grid-column: 1 / -1;
    border-top: 1px solid var(--color-border);
    padding-top: 16px;
  }

  .magnets-section h3 {
    font-size: 14px;
    font-weight: 600;
    color: var(--color-primary);
    margin-bottom: 12px;
  }

  .magnet-table {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--color-border);
    border-radius: var(--border-radius);
    overflow: hidden;
  }

  .magnet-row {
    display: grid;
    grid-template-columns: 40px 1fr auto;
    align-items: center;
    gap: 16px;
    padding: 12px 16px;
    background-color: var(--color-backgroundTertiary);
    border-bottom: 1px solid var(--color-border);
    cursor: pointer;
    transition: all 0.15s ease;
    outline: none;
  }

  .magnet-row:last-child {
    border-bottom: none;
  }

  .magnet-row:hover {
    background-color: var(--color-hover);
  }

  .magnet-row.selected {
    background-color: var(--color-primary);
    color: var(--color-selectedText);
  }

  .magnet-row:focus {
    outline: none;
  }

  .magnet-index {
    color: var(--color-textSecondary);
    font-weight: 600;
    font-size: 13px;
    text-align: center;
  }

  .magnet-row.selected .magnet-index {
    color: var(--color-selectedText);
  }

  .magnet-source {
    color: var(--color-text);
    font-size: 14px;
    font-weight: 500;
    user-select: text;
  }

  .magnet-row.selected .magnet-source {
    color: var(--color-selectedText);
  }

  .magnet-hint {
    color: var(--color-textSecondary);
    font-size: 12px;
    font-style: italic;
  }

  .magnet-row.selected .magnet-hint {
    color: rgba(255, 255, 255, 0.8);
  }

  .no-selection {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--color-textMuted);
  }

  .categories-list {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .category-tag {
    background-color: var(--color-backgroundTertiary);
    color: var(--color-text);
    padding: 2px 8px;
    border-radius: 12px;
    font-size: 11px;
    font-weight: 500;
    border: 1px solid var(--color-border);
    white-space: nowrap;
    user-select: text;
    cursor: text;
  }
</style>
