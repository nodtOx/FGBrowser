// Feature flags for enabling/disabling features
// Useful for hiding incomplete features or A/B testing

export interface FeatureFlags {
  torrentClient: boolean;
  popularRepacks: boolean;
  advancedSearch: boolean;
  stats: boolean;
  settings: boolean;
}

// Default feature flags
const defaultFlags: FeatureFlags = {
  torrentClient: false, // Hide torrent client until fully implemented
  popularRepacks: true, // Popular repacks is complete
  advancedSearch: true, // Search feature is complete
  stats: false, // Stats page not yet implemented
  settings: false, // Settings page not yet implemented
};

// Load feature flags from localStorage (allows dev override)
function loadFeatureFlags(): FeatureFlags {
  if (typeof window === 'undefined') return defaultFlags;

  try {
    const stored = localStorage.getItem('featureFlags');
    if (stored) {
      return { ...defaultFlags, ...JSON.parse(stored) };
    }
  } catch (error) {
    console.warn('Failed to load feature flags:', error);
  }

  return defaultFlags;
}

// Save feature flags to localStorage
export function saveFeatureFlags(flags: Partial<FeatureFlags>) {
  if (typeof window === 'undefined') return;

  try {
    const current = loadFeatureFlags();
    const updated = { ...current, ...flags };
    localStorage.setItem('featureFlags', JSON.stringify(updated));
    // Reload to apply changes
    window.location.reload();
  } catch (error) {
    console.warn('Failed to save feature flags:', error);
  }
}

// Export loaded flags
export const featureFlags = loadFeatureFlags();

// Helper to check if a feature is enabled
export function isFeatureEnabled(feature: keyof FeatureFlags): boolean {
  return featureFlags[feature];
}

// Dev helper: Enable a feature in console
// Usage: window.enableFeature('torrentClient')
if (typeof window !== 'undefined') {
  (window as any).enableFeature = (feature: keyof FeatureFlags) => {
    console.log(`Enabling feature: ${feature}`);
    saveFeatureFlags({ [feature]: true });
  };

  (window as any).disableFeature = (feature: keyof FeatureFlags) => {
    console.log(`Disabling feature: ${feature}`);
    saveFeatureFlags({ [feature]: false });
  };

  (window as any).showFeatureFlags = () => {
    console.log('Current feature flags:', featureFlags);
  };
}
