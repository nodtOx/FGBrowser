<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import SidebarBase from './SidebarBase.svelte';

  export let selectedPeriod: 'week' | 'today' | 'month' | 'year' | 'award' = 'week';
  export let weeklyCount: number = 0;
  export let todayCount: number = 0;
  export let monthlyCount: number = 0;
  export let yearlyCount: number = 0;
  export let awardCount: number = 0;

  let weekUnseen: number = 0;
  let todayUnseen: number = 0;
  let monthUnseen: number = 0;
  let yearUnseen: number = 0;
  let awardUnseen: number = 0;

  onMount(async () => {
    await loadUnseenCounts();
  });

  async function loadUnseenCounts() {
    try {
      weekUnseen = await invoke<number>('get_unseen_popular_count', { period: 'week' });
      todayUnseen = await invoke<number>('get_unseen_popular_count', { period: 'today' });
      monthUnseen = await invoke<number>('get_unseen_popular_count', { period: 'month' });
      yearUnseen = await invoke<number>('get_unseen_popular_count', { period: 'year' });
      awardUnseen = await invoke<number>('get_unseen_popular_count', { period: 'award' });
    } catch (err) {
      console.error('Failed to load unseen counts:', err);
    }
  }

  async function selectPeriod(period: 'week' | 'today' | 'month' | 'year' | 'award') {
    selectedPeriod = period;
    // Reload counts after selection (will update after mark_as_viewed is called)
    setTimeout(() => loadUnseenCounts(), 500);
  }

  function handleKeydown(e: KeyboardEvent, action: () => void) {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      action();
    }
  }

  // Export function to allow parent to refresh counts
  export function refreshUnseenCounts() {
    loadUnseenCounts();
  }
</script>

<SidebarBase>
  <div class="sidebar-section">
    <div class="section-title">Period</div>

    <div
      class="sidebar-item"
      class:selected={selectedPeriod === 'today'}
      on:click={() => selectPeriod('today')}
      on:keydown={(e) => handleKeydown(e, () => selectPeriod('today'))}
      role="button"
      tabindex="0"
    >
      <span class="filter-name">
        Today
        {#if todayUnseen > 0}
          <span class="unseen-badge">{todayUnseen}</span>
        {/if}
      </span>
      <span class="filter-count">{todayCount}</span>
    </div>
  
    <div
    class="sidebar-item"
    class:selected={selectedPeriod === 'week'}
    on:click={() => selectPeriod('week')}
    on:keydown={(e) => handleKeydown(e, () => selectPeriod('week'))}
    role="button"
    tabindex="0"
  >
    <span class="filter-name">
      This Week
      {#if weekUnseen > 0}
        <span class="unseen-badge">{weekUnseen}</span>
      {/if}
    </span>
    <span class="filter-count">{weeklyCount}</span>
  </div>

    <div
      class="sidebar-item"
      class:selected={selectedPeriod === 'month'}
      on:click={() => selectPeriod('month')}
      on:keydown={(e) => handleKeydown(e, () => selectPeriod('month'))}
      role="button"
      tabindex="0"
    >
      <span class="filter-name">
        This Month
        {#if monthUnseen > 0}
          <span class="unseen-badge">{monthUnseen}</span>
        {/if}
      </span>
      <span class="filter-count">{monthlyCount}</span>
    </div>

    <div
      class="sidebar-item"
      class:selected={selectedPeriod === 'year'}
      on:click={() => selectPeriod('year')}
      on:keydown={(e) => handleKeydown(e, () => selectPeriod('year'))}
      role="button"
      tabindex="0"
    >
      <span class="filter-name">
        This Year
        {#if yearUnseen > 0}
          <span class="unseen-badge">{yearUnseen}</span>
        {/if}
      </span>
      <span class="filter-count">{yearlyCount}</span>
    </div>

    <div
      class="sidebar-item"
      class:selected={selectedPeriod === 'award'}
      on:click={() => selectPeriod('award')}
      on:keydown={(e) => handleKeydown(e, () => selectPeriod('award'))}
      role="button"
      tabindex="0"
    >
      <span class="filter-name">
        Pink Paw Award
        <span class="pink-paw-icon">🐾</span>
        {#if awardUnseen > 0}
          <span class="unseen-badge">{awardUnseen}</span>
        {/if}
      </span>
      <span class="filter-count">{awardCount}</span>
    </div>
  </div>

  
</SidebarBase>

<style>
  .unseen-badge {
    display: inline-block;
    background-color: var(--color-primary);
    color: white;
    font-size: 10px;
    font-weight: 700;
    padding: 2px 6px;
    border-radius: 10px;
    margin-left: 6px;
    vertical-align: middle;
  }
  .pink-paw-icon {
    color: transparent;
    text-shadow: 0 0 0 #ff69b4;
  }
</style>

