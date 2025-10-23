<script lang="ts">
  import { fetchGameMedia, type MediaResult } from '$lib/types/media';
  import { onMount, onDestroy } from 'svelte';

  export let gameId: number;

  let loading = true;
  let error = '';
  let media: MediaResult | null = null;

  // Lightbox state
  let lightboxOpen = false;
  let lightboxClosing = false;
  let currentImageIndex = 0;

  // Reactive statement - runs whenever gameId changes
  $: {
    console.log('GameMedia: gameId changed to', gameId);
    loadMedia(gameId);
  }

  async function loadMedia(id: number) {
    try {
      loading = true;
      error = '';
      media = null; // Clear previous media
      closeLightbox(); // Close lightbox when switching games

      console.log('Fetching media for game:', id);
      // Fetch media (will load from DB if available, otherwise scrape from FitGirl)
      media = await fetchGameMedia(id);

      console.log(`Loaded ${media.screenshots.length} screenshots and ${media.videos.length} videos for game ${id}`);

      // Preload full-resolution images in the background
      if (media && media.screenshots.length > 0) {
        const screenshotsToPreload = media.screenshots;
        setTimeout(() => {
          preloadFullResImages(screenshotsToPreload);
        }, 500); // Small delay to prioritize thumbnail loading
      }
    } catch (err) {
      console.error('Failed to fetch game media:', err);
      error = err instanceof Error ? err.message : String(err);
    } finally {
      loading = false;
    }
  }

  function preloadFullResImages(screenshots: Array<{ url: string; thumbnail_url?: string }>) {
    screenshots.forEach((screenshot) => {
      if (screenshot.url) {
        const img = new Image();
        img.src = screenshot.url;
        // Browser will cache the image
      }
    });
    console.log(`🖼️ Preloading ${screenshots.length} full-resolution images`);
  }

  function openLightbox(index: number) {
    currentImageIndex = index;
    lightboxOpen = true;
    lightboxClosing = false;
  }

  function closeLightbox() {
    if (lightboxClosing) return; // Prevent multiple close calls
    lightboxClosing = true;
    // Wait for fade-out animation to complete before actually closing
    setTimeout(() => {
      lightboxOpen = false;
      lightboxClosing = false;
    }, 250); // Slightly longer than animation to ensure completion
  }

  function nextImage() {
    if (media && media.screenshots.length > 0) {
      currentImageIndex = (currentImageIndex + 1) % media.screenshots.length;
    }
  }

  function prevImage() {
    if (media && media.screenshots.length > 0) {
      currentImageIndex = (currentImageIndex - 1 + media.screenshots.length) % media.screenshots.length;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    // Don't handle if typing in an input
    const target = e.target as HTMLElement;
    const isTyping = target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable;
    if (isTyping) return;

    // Don't handle if no screenshots
    if (!media || media.screenshots.length === 0) return;

    // When lightbox is open
    if (lightboxOpen) {
      switch (e.key) {
        case 'Escape':
          closeLightbox();
          break;
        case 'ArrowRight':
          e.preventDefault();
          nextImage();
          break;
        case 'ArrowLeft':
          e.preventDefault();
          prevImage();
          break;
      }
    }
    // When lightbox is closed, arrow keys open it
    else {
      switch (e.key) {
        case 'ArrowRight':
          e.preventDefault();
          // Move to next image and open lightbox
          currentImageIndex = (currentImageIndex + 1) % media.screenshots.length;
          openLightbox(currentImageIndex);
          break;
        case 'ArrowLeft':
          e.preventDefault();
          // Move to previous image and open lightbox
          currentImageIndex = (currentImageIndex - 1 + media.screenshots.length) % media.screenshots.length;
          openLightbox(currentImageIndex);
          break;
      }
    }
  }

  onMount(() => {
    window.addEventListener('keydown', handleKeydown);
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeydown);
  });
</script>

<div class="game-media">
  {#if loading}
    <div class="loading">
      <div class="spinner"></div>
      <p>Loading screenshots and videos...</p>
    </div>
  {:else if error}
    <div class="error">
      <p>Error loading media: {error}</p>
    </div>
  {:else if media}
    <!-- Screenshots Section -->
    {#if media.screenshots.length > 0}
      <section class="screenshots">
        <h3>Screenshots ({media.screenshots.length})</h3>
        <div class="screenshot-grid">
          {#each media.screenshots as screenshot, i}
            {#if screenshot.thumbnail_url}
              <button class="screenshot-item" on:click={() => openLightbox(i)} type="button">
                <img src={screenshot.thumbnail_url} alt="Game screenshot" loading="lazy" />
              </button>
            {/if}
          {/each}
        </div>
      </section>
    {/if}

    <!-- Videos Section -->
    {#if media.videos.length > 0}
      <section class="videos">
        <h3>Videos & GIFs ({media.videos.length})</h3>
        <div class="video-list">
          {#each media.videos as video}
            <div class="video-item">
              {#if video.url.includes('youtube.com') || video.url.includes('youtu.be')}
                <!-- YouTube embed -->
                <iframe
                  src={video.url.replace('watch?v=', 'embed/')}
                  title="Game video"
                  frameborder="0"
                  allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
                  allowfullscreen
                ></iframe>
              {:else if video.url.endsWith('.gif') || video.url.endsWith('.gifv')}
                <!-- GIF -->
                <img src={video.url} alt="Game GIF" loading="lazy" />
              {:else if video.url.endsWith('.mp4') || video.url.endsWith('.webm')}
                <!-- Direct video -->
                <!-- svelte-ignore a11y-media-has-caption -->
                <video controls>
                  <source src={video.url} type="video/{video.url.split('.').pop()}" />
                  Your browser does not support the video tag.
                </video>
              {:else}
                <!-- External link -->
                <a href={video.url} target="_blank" rel="noopener noreferrer"> Watch video </a>
              {/if}
            </div>
          {/each}
        </div>
      </section>
    {/if}

    {#if media.screenshots.length === 0 && media.videos.length === 0}
      <p class="no-media">No screenshots or videos available for this game.</p>
    {/if}
  {/if}
</div>

<!-- Lightbox Modal -->
{#if lightboxOpen && media && media.screenshots.length > 0}
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="lightbox-overlay" class:closing={lightboxClosing} on:click={closeLightbox}>
    <button class="lightbox-close" on:click={closeLightbox} aria-label="Close">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="18" y1="6" x2="6" y2="18"></line>
        <line x1="6" y1="6" x2="18" y2="18"></line>
      </svg>
    </button>

    <div class="lightbox-content">
      <div class="lightbox-image-container">
        <img
          src={media.screenshots[currentImageIndex].url}
          alt="Screenshot {currentImageIndex + 1}"
          class="lightbox-image"
        />

        <!-- Interactive zones overlaid on the image -->
        <div class="lightbox-zones">
          <!-- Left 25% - Previous -->
          <button class="lightbox-zone zone-prev" on:click|stopPropagation={prevImage} aria-label="Previous screenshot">
            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="15 18 9 12 15 6"></polyline>
            </svg>
          </button>

          <!-- Center 50% - Close -->
          <button class="lightbox-zone zone-close" on:click|stopPropagation={closeLightbox} aria-label="Close lightbox">
          </button>

          <!-- Right 25% - Next -->
          <button class="lightbox-zone zone-next" on:click|stopPropagation={nextImage} aria-label="Next screenshot">
            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="9 18 15 12 9 6"></polyline>
            </svg>
          </button>
        </div>
      </div>

      <!-- svelte-ignore a11y-click-events-have-key-events -->
      <!-- svelte-ignore a11y-no-static-element-interactions -->
      <div class="lightbox-counter" on:click|stopPropagation>
        {currentImageIndex + 1} / {media.screenshots.length}
      </div>
    </div>
  </div>
{/if}

<style>
  .game-media {
    margin-top: 2rem;
  }

  .loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    padding: 3rem;
    color: var(--color-textSecondary);
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 3px solid var(--color-border);
    border-top-color: var(--color-primary);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .error {
    padding: 1rem;
    background: rgba(255, 0, 0, 0.1);
    border: 1px solid rgba(255, 0, 0, 0.3);
    border-radius: 8px;
    color: #ff6b6b;
  }

  .screenshots,
  .videos {
    margin-top: 2rem;
  }

  h3 {
    font-size: 14px;
    font-weight: 600;
    color: var(--color-primary);
    margin-bottom: 12px;
  }

  .screenshot-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 16px;
  }

  .screenshot-item {
    position: relative;
    width: 100%;
    aspect-ratio: 16 / 9;
    overflow: hidden;
    border-radius: var(--border-radius);
    background-color: var(--color-backgroundTertiary);
    border: none;
    padding: 0;
    cursor: pointer;
    display: block;
  }

  .screenshot-item img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    border-radius: var(--border-radius);
    border: 1px solid var(--color-border);
    transition:
      transform 0.2s,
      box-shadow 0.2s;
    pointer-events: none;
  }

  .screenshot-item:hover img {
    transform: scale(1.05);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }

  /* Responsive grid */
  @media (max-width: 1400px) {
    .screenshot-grid {
      grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    }
  }

  @media (max-width: 1024px) {
    .screenshot-grid {
      grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    }
  }

  @media (max-width: 768px) {
    .screenshot-grid {
      grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
      gap: 12px;
    }
  }

  .video-list {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .video-item iframe,
  .video-item video,
  .video-item img {
    width: 100%;
    max-width: 800px;
    border-radius: 8px;
  }

  .video-item iframe {
    aspect-ratio: 16 / 9;
  }

  .video-item a {
    display: inline-block;
    padding: 8px 16px;
    background: var(--color-primary);
    color: white;
    text-decoration: none;
    border-radius: var(--border-radius);
    transition: background 0.2s;
    font-size: 14px;
  }

  .video-item a:hover {
    opacity: 0.9;
  }

  .no-media {
    padding: 2rem;
    text-align: center;
    color: var(--color-textSecondary);
    font-size: 14px;
  }

  /* Lightbox Styles */
  .lightbox-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.95);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 10000;
    cursor: pointer;
    animation: fadeIn 0.2s ease-out forwards;
  }

  .lightbox-overlay.closing {
    animation: fadeOut 0.2s ease-out forwards;
    pointer-events: none;
  }

  .lightbox-overlay.closing * {
    pointer-events: none;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  @keyframes fadeOut {
    from {
      opacity: 1;
    }
    to {
      opacity: 0;
    }
  }

  .lightbox-content {
    max-width: 95vw;
    max-height: 95vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    cursor: default;
  }

  .lightbox-image-container {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .lightbox-content img {
    max-width: 100%;
    max-height: 90vh;
    object-fit: contain;
    border-radius: 4px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
    display: block;
  }

  /* Interactive zones overlay */
  .lightbox-zones {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    display: flex;
  }

  .lightbox-zone {
    position: relative;
    background: transparent;
    border: none;
    cursor: pointer;
    transition: background 0.2s ease;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
  }

  .lightbox-zone svg {
    opacity: 0;
    color: white;
    transition: opacity 0.3s ease;
    filter: drop-shadow(0 2px 8px rgba(0, 0, 0, 0.8));
    position: relative;
    z-index: 1;
  }

  .lightbox-zone:hover svg {
    opacity: 0.9;
  }

  /* Left zone - 25% */
  .zone-prev {
    flex: 0 0 25%;
    justify-content: flex-start;
    padding-left: 20px;
    position: relative;
  }

  .zone-prev::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: linear-gradient(to right, rgba(0, 0, 0, 0.3), transparent);
    opacity: 0;
    transition: opacity 0.3s ease;
    pointer-events: none;
  }

  .zone-prev:hover::before {
    opacity: 1;
  }

  /* Center zone - 50% */
  .zone-close {
    flex: 0 0 50%;
  }

  /* No hover effect for center zone - keep it invisible */

  /* Right zone - 25% */
  .zone-next {
    flex: 0 0 25%;
    justify-content: flex-end;
    padding-right: 20px;
    position: relative;
  }

  .zone-next::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: linear-gradient(to left, rgba(0, 0, 0, 0.3), transparent);
    opacity: 0;
    transition: opacity 0.3s ease;
    pointer-events: none;
  }

  .zone-next:hover::before {
    opacity: 1;
  }

  .lightbox-close {
    position: absolute;
    top: 20px;
    right: 20px;
    background: rgba(255, 255, 255, 0.1);
    border: none;
    color: white;
    width: 48px;
    height: 48px;
    border-radius: 50%;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.2s;
    z-index: 10001;
  }

  .lightbox-close:hover {
    background: rgba(255, 255, 255, 0.2);
  }

  .lightbox-counter {
    color: white;
    font-size: 16px;
    font-weight: 500;
    background: rgba(0, 0, 0, 0.6);
    padding: 8px 16px;
    border-radius: 20px;
    user-select: none;
  }

  /* Mobile responsive */
  @media (max-width: 768px) {
    .lightbox-close {
      top: 10px;
      right: 10px;
      width: 40px;
      height: 40px;
    }

    .lightbox-zone svg {
      width: 32px;
      height: 32px;
    }
  }
</style>
