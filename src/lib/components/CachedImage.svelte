<script lang="ts">
  import { convertFileSrc, invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  export let src: string | null | undefined;
  export let alt: string = '';
  export let className: string = '';

  let cachedSrc: string = '';
  let error: boolean = false;
  let imageLoaded: boolean = false;

  async function loadCachedImage() {
    if (!src) {
      return;
    }

    try {
      error = false;
      imageLoaded = false;

      // Check if image is already cached (fast, synchronous check)
      const cachedPath = await invoke<string | null>('check_image_cached', { url: src });

      if (cachedPath) {
        // Image is cached, use the cached version
        cachedSrc = convertFileSrc(cachedPath);
      } else {
        // Image not cached, show original URL immediately
        cachedSrc = src;
        // Start caching in the background (fire and forget)
        invoke('cache_image_background', { url: src }).catch((err) => {
          console.warn('Background cache failed:', err);
        });
      }
    } catch (err) {
      console.error('Failed to check cached image:', err);
      error = true;
      // Fallback to original URL if check fails
      cachedSrc = src;
    }
  }

  function handleImageLoad() {
    imageLoaded = true;
  }

  function handleImageError(e: Event) {
    console.error('Image load error for:', cachedSrc, e);
    error = true;
    imageLoaded = false;
  }

  $: if (src) {
    loadCachedImage();
  }

  onMount(() => {
    if (src) {
      loadCachedImage();
    }
  });
</script>

<div class="image-container {className}">
  {#if !imageLoaded && !error && src && cachedSrc}
    <div class="image-placeholder">
      <div class="loading-spinner"></div>
    </div>
  {/if}

  {#if error || !src || !cachedSrc}
    <slot name="fallback">
      <div class="image-error">
        <span>No Image</span>
      </div>
    </slot>
  {:else}
    <img
      src={cachedSrc}
      {alt}
      class="cached-image"
      class:loaded={imageLoaded}
      loading="lazy"
      on:load={handleImageLoad}
      on:error={handleImageError}
    />
  {/if}
</div>

<style>
  .image-container {
    position: relative;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  .image-placeholder {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: var(--color-backgroundTertiary);
    z-index: 1;
  }

  .loading-spinner {
    width: 24px;
    height: 24px;
    border: 3px solid var(--color-border);
    border-top-color: var(--color-primary);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .image-error {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background-color: var(--color-backgroundTertiary);
    color: var(--color-textMuted);
    font-size: 12px;
  }

  .cached-image {
    width: 100%;
    height: 100%;
    object-fit: cover;
    opacity: 0;
    transition: opacity 0.2s ease-in-out;
  }

  .cached-image.loaded {
    opacity: 1;
  }
</style>
