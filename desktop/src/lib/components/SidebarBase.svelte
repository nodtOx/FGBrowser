<script lang="ts">
  // Reusable sidebar component with consistent styling
  // Can be used for categories, filters, download status, etc.
</script>

<div class="sidebar">
  <slot />
</div>

<style>
  .sidebar {
    width: var(--sidebar-width);
    background-color: var(--color-background);
    border-right: 1px solid var(--color-border);
    overflow-y: auto;
    padding: 8px 0;
    display: flex;
    flex-direction: column;
    gap: 16px;
    font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
    font-size: calc(var(--base-font-size) * 0.9);
  }

  /* Export as global styles for child components to use */
  :global(.sidebar-section) {
    display: flex;
    flex-direction: column;
    padding: 4px 0;
    margin: -4px 0;
    border-left: 3px solid transparent;
    transition: all 0.15s ease;
  }
  
  :global(.sidebar-section.focused) {
    border-left-color: var(--color-primary);
    background-color: rgba(var(--color-primary-rgb, 136, 192, 208), 0.05);
  }

  :global(.section-title) {
    font-size: calc(var(--base-font-size) * 0.85);
    font-weight: 600;
    color: var(--color-textSecondary);
    padding: 4px 12px 2px 12px;
    margin: 0;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  :global(.sidebar-item) {
    padding: 3px 12px;
    color: var(--color-textSecondary);
    cursor: pointer;
    transition: var(--transition);
    line-height: 1.4;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  :global(.sidebar-item:hover) {
    background-color: var(--color-hover);
    color: var(--color-text);
  }
  
  :global(.sidebar-item.keyboard-focused) {
    background-color: var(--color-hover);
    color: var(--color-text);
    outline: 2px solid var(--color-primary);
    outline-offset: -2px;
  }

  :global(.sidebar-item.selected),
  :global(.sidebar-item.active) {
    background-color: var(--color-primary);
    color: var(--color-selectedText);
    font-weight: 600;
  }

  :global(.sidebar-item.all-games) {
    color: var(--color-text);
    font-weight: 600;
    background-color: var(--color-backgroundTertiary);
    pointer-events: none;
  }

  :global(.sidebar-item.more-button) {
    color: var(--color-primary);
    font-style: italic;
    font-size: calc(var(--base-font-size) * 0.85);
    margin-top: 4px;
    padding-top: 6px;
  }

  :global(.sidebar-item.more-button:hover) {
    background-color: var(--color-hover);
    color: var(--color-primary);
    font-weight: 500;
  }

  /* Category-specific styles */
  :global(.category-item) {
    display: flex;
    align-items: center;
    justify-content: space-between;
    position: relative;
  }

  :global(.category-item.selected) {
    background-color: var(--color-backgroundTertiary);
    color: var(--color-primary);
    font-weight: 600;
    border-left: 3px solid var(--color-primary);
    padding-left: 9px;
  }

  :global(.category-item:hover:not(.selected)) {
    background-color: var(--color-hover);
    color: var(--color-text);
  }

  :global(.category-name) {
    flex: 1;
    min-width: 0;
  }

  :global(.category-count) {
    color: var(--color-textSecondary);
    font-size: calc(var(--base-font-size) * 0.85);
    margin-left: 4px;
  }

  :global(.category-item.selected .category-count) {
    color: var(--color-primary);
  }

  :global(.category-indicator) {
    color: var(--color-primary);
    font-weight: bold;
    margin-left: 8px;
    font-size: calc(var(--base-font-size) * 0.9);
  }

  /* Filter chip styles */
  :global(.selected-categories) {
    margin-bottom: 12px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--color-border);
  }

  :global(.selected-header) {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
    padding: 0 12px;
  }

  :global(.selected-count) {
    font-size: calc(var(--base-font-size) * 0.8);
    color: var(--color-primary);
    font-weight: 600;
  }

  :global(.clear-all-btn) {
    background: none;
    border: none;
    color: var(--color-textSecondary);
    font-size: calc(var(--base-font-size) * 0.75);
    cursor: pointer;
    padding: 2px 6px;
    border-radius: 3px;
    transition: var(--transition);
  }

  :global(.clear-all-btn:hover) {
    background-color: var(--color-hover);
    color: var(--color-text);
  }

  :global(.selected-chips) {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    padding: 0 12px;
  }

  :global(.seperator) {
    border-top: 1px solid var(--color-border);
  }

  :global(.category-chip) {
    display: inline-flex;
    align-items: center;
    background-color: var(--color-primary);
    color: var(--color-selectedText);
    font-size: calc(var(--base-font-size) * 0.75);
    padding: 2px 6px;
    border-radius: 0px;
    font-weight: 500;
  }

  :global(.chip-remove) {
    background: none;
    border: none;
    color: var(--color-selectedText);
    margin-left: 4px;
    cursor: pointer;
    font-size: 14px;
    line-height: 1;
    padding: 0;
    width: 14px;
    height: 14px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    transition: var(--transition);
  }

  :global(.chip-remove:hover) {
    background-color: rgba(255, 255, 255, 0.2);
  }

  /* Filter-specific styles */
  :global(.filter-name) {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  :global(.filter-count) {
    color: var(--color-textSecondary);
    font-size: calc(var(--base-font-size) * 0.85);
    margin-left: 4px;
  }

  :global(.sidebar-item.selected .filter-count) {
    color: var(--color-selectedText);
  }

  /* Error-specific styles */
  :global(.sidebar-item.error-item:hover) {
    background-color: rgba(244, 67, 54, 0.1);
  }

  :global(.sidebar-item.error-item.selected) {
    background-color: var(--color-error);
    color: white;
  }

  :global(.error-count) {
    color: var(--color-error);
    font-weight: 700;
  }

  :global(.sidebar-item.error-item.selected .error-count) {
    color: white;
  }

  /* Filter chip variants */
  :global(.time-filter-chip) {
    background-color: var(--color-warning);
    color: var(--color-background);
  }

  :global(.time-filter-chip .chip-remove) {
    color: var(--color-background);
  }

  :global(.time-filter-chip .chip-remove:hover) {
    background-color: rgba(46, 52, 64, 0.2);
  }

  :global(.size-filter-chip) {
    background-color: var(--color-info);
    color: var(--color-selectedText);
  }

  :global(.status-filter-chip) {
    background-color: var(--color-success);
    color: var(--color-background);
  }

  :global(.status-filter-chip .chip-remove) {
    color: var(--color-background);
  }

  :global(.status-filter-chip .chip-remove:hover) {
    background-color: rgba(46, 52, 64, 0.2);
  }
</style>

