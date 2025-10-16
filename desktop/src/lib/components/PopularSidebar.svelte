<script lang="ts">
  import SidebarBase from './SidebarBase.svelte';

  export let selectedPeriod: 'month' | 'year' | 'award' = 'month';
  export let monthlyCount: number = 0;
  export let yearlyCount: number = 0;
  export let awardCount: number = 0;

  function selectPeriod(period: 'month' | 'year' | 'award') {
    selectedPeriod = period;
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
    <div class="section-title">Period</div>

    <div
      class="sidebar-item"
      class:selected={selectedPeriod === 'month'}
      on:click={() => selectPeriod('month')}
      on:keydown={(e) => handleKeydown(e, () => selectPeriod('month'))}
      role="button"
      tabindex="0"
    >
      <span class="filter-name">This Month</span>
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
      <span class="filter-name">This Year</span>
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
      <span class="filter-name">Pink Paw Award</span>
      <span class="filter-count">{awardCount}</span>
    </div>
  </div>

  <div class="sidebar-section">
    <div class="section-title">Info</div>
    
    <div class="info-item">
      <span class="info-label">Source:</span>
      <span class="info-value">FitGirl Repacks</span>
    </div>
    
    <div class="info-item">
      <span class="info-label">Updated:</span>
      <span class="info-value">Real-time</span>
    </div>
    
    <div class="info-item">
      <span class="info-label">Ranking:</span>
      <span class="info-value">By Downloads</span>
    </div>
  </div>
</SidebarBase>

<style>
  .info-item {
    padding: 6px 12px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .info-label {
    font-size: calc(var(--base-font-size) * 0.75);
    color: var(--color-textSecondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    font-weight: 600;
  }

  .info-value {
    font-size: calc(var(--base-font-size) * 0.85);
    color: var(--color-text);
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  }
</style>

