<script lang="ts">
  import {
    activeDownloads,
    completedDownloads,
    downloads,
    errorDownloads,
    pausedDownloads,
    seedingDownloads,
  } from '$lib/stores/downloads';
  import SidebarBase from './SidebarBase.svelte';

  export let selectedFilter: 'all' | 'active' | 'completed' | 'seeding' | 'paused' | 'error' = 'all';

  function selectFilter(filter: typeof selectedFilter) {
    selectedFilter = filter;
  }

  function handleKeydown(e: KeyboardEvent, action: () => void) {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      action();
    }
  }
</script>

<SidebarBase>
  <div class="sidebar-section">
    <div class="section-title">Status</div>

    <div
      class="sidebar-item"
      class:selected={selectedFilter === 'all'}
      on:click={() => selectFilter('all')}
      on:keydown={(e) => handleKeydown(e, () => selectFilter('all'))}
      role="button"
      tabindex="0"
    >
      <span class="filter-name">All</span>
      <span class="filter-count">{$downloads.length}</span>
    </div>

    <div
      class="sidebar-item"
      class:selected={selectedFilter === 'active'}
      on:click={() => selectFilter('active')}
      on:keydown={(e) => handleKeydown(e, () => selectFilter('active'))}
      role="button"
      tabindex="0"
    >
      <span class="filter-name">Active</span>
      <span class="filter-count">{$activeDownloads.length}</span>
    </div>

    <div
      class="sidebar-item"
      class:selected={selectedFilter === 'seeding'}
      on:click={() => selectFilter('seeding')}
      on:keydown={(e) => handleKeydown(e, () => selectFilter('seeding'))}
      role="button"
      tabindex="0"
    >
      <span class="filter-name">Seeding</span>
      <span class="filter-count">{$seedingDownloads.length}</span>
    </div>

    <div
      class="sidebar-item"
      class:selected={selectedFilter === 'paused'}
      on:click={() => selectFilter('paused')}
      on:keydown={(e) => handleKeydown(e, () => selectFilter('paused'))}
      role="button"
      tabindex="0"
    >
      <span class="filter-name">Paused</span>
      <span class="filter-count">{$pausedDownloads.length}</span>
    </div>

    <div
      class="sidebar-item"
      class:selected={selectedFilter === 'completed'}
      on:click={() => selectFilter('completed')}
      on:keydown={(e) => handleKeydown(e, () => selectFilter('completed'))}
      role="button"
      tabindex="0"
    >
      <span class="filter-name">Completed</span>
      <span class="filter-count">{$completedDownloads.length}</span>
    </div>

    {#if $errorDownloads.length > 0}
      <div
        class="sidebar-item error-item"
        class:selected={selectedFilter === 'error'}
        on:click={() => selectFilter('error')}
        on:keydown={(e) => handleKeydown(e, () => selectFilter('error'))}
        role="button"
        tabindex="0"
      >
        <span class="filter-name">Errors</span>
        <span class="filter-count error-count">{$errorDownloads.length}</span>
      </div>
    {/if}
  </div>
</SidebarBase>


