import { writable } from 'svelte/store';

export interface Theme {
  name: string;
  type: 'dark' | 'light';
  author?: string;
  version?: string;
  colors: {
    background: string;
    backgroundSecondary: string;
    backgroundTertiary: string;
    primary: string;
    secondary: string;
    text: string;
    textSecondary: string;
    textMuted: string;
    border: string;
    hover: string;
    selected: string;
    selectedText: string;
    success: string;
    warning: string;
    error: string;
    info: string;
  };
}

export const nord: Theme = {
  name: 'Nord',
  type: 'dark',
  colors: {
    background: '#2e3440',
    backgroundSecondary: '#3b4252',
    backgroundTertiary: '#434c5e',
    primary: '#88c0d0',
    secondary: '#5e81ac',
    text: '#eceff4',
    textSecondary: '#d8dee9',
    textMuted: '#4c566a',
    border: '#434c5e',
    hover: '#4c566a',
    selected: '#88c0d0',
    selectedText: '#2e3440',
    success: '#a3be8c',
    warning: '#ebcb8b',
    error: '#bf616a',
    info: '#81a1c1',
  },
};

export const dracula: Theme = {
  name: 'Dracula',
  type: 'dark',
  colors: {
    background: '#282a36',
    backgroundSecondary: '#44475a',
    backgroundTertiary: '#6272a4',
    primary: '#bd93f9',
    secondary: '#ff79c6',
    text: '#f8f8f2',
    textSecondary: '#f8f8f2',
    textMuted: '#6272a4',
    border: '#44475a',
    hover: '#6272a4',
    selected: '#bd93f9',
    selectedText: '#282a36',
    success: '#50fa7b',
    warning: '#f1fa8c',
    error: '#ff5555',
    info: '#8be9fd',
  },
};

export const tokyoNight: Theme = {
  name: 'Tokyo Night',
  type: 'dark',
  colors: {
    background: '#1a1b26',
    backgroundSecondary: '#24283b',
    backgroundTertiary: '#414868',
    primary: '#7aa2f7',
    secondary: '#bb9af7',
    text: '#c0caf5',
    textSecondary: '#a9b1d6',
    textMuted: '#565f89',
    border: '#3b4261',
    hover: '#3b4261',
    selected: '#7aa2f7',
    selectedText: '#1a1b26',
    success: '#9ece6a',
    warning: '#e0af68',
    error: '#f7768e',
    info: '#7dcfff',
  },
};

export const gruvboxDark: Theme = {
  name: 'Gruvbox Dark',
  type: 'dark',
  colors: {
    background: '#282828',
    backgroundSecondary: '#3c3836',
    backgroundTertiary: '#504945',
    primary: '#83a598',
    secondary: '#d3869b',
    text: '#ebdbb2',
    textSecondary: '#d5c4a1',
    textMuted: '#7c6f64',
    border: '#504945',
    hover: '#504945',
    selected: '#83a598',
    selectedText: '#282828',
    success: '#b8bb26',
    warning: '#fabd2f',
    error: '#fb4934',
    info: '#8ec07c',
  },
};

export const gruvboxLight: Theme = {
  name: 'Gruvbox Light',
  type: 'light',
  colors: {
    background: '#fbf1c7',
    backgroundSecondary: '#ebdbb2',
    backgroundTertiary: '#d5c4a1',
    primary: '#076678',
    secondary: '#8f3f71',
    text: '#3c3836',
    textSecondary: '#504945',
    textMuted: '#928374',
    border: '#d5c4a1',
    hover: '#d5c4a1',
    selected: '#076678',
    selectedText: '#fbf1c7',
    success: '#79740e',
    warning: '#b57614',
    error: '#cc241d',
    info: '#427b58',
  },
};

export const solarizedLight: Theme = {
  name: 'Solarized Light',
  type: 'light',
  colors: {
    background: '#fdf6e3',
    backgroundSecondary: '#eee8d5',
    backgroundTertiary: '#93a1a1',
    primary: '#268bd2',
    secondary: '#6c71c4',
    text: '#657b83',
    textSecondary: '#586e75',
    textMuted: '#93a1a1',
    border: '#eee8d5',
    hover: '#d9d2c4',
    selected: '#268bd2',
    selectedText: '#fdf6e3',
    success: '#859900',
    warning: '#b58900',
    error: '#dc322f',
    info: '#2aa198',
  },
};

export const catppuccinLatte: Theme = {
  name: 'Catppuccin Latte',
  type: 'light',
  colors: {
    background: '#eff1f5',
    backgroundSecondary: '#e6e9ef',
    backgroundTertiary: '#ccd0da',
    primary: '#1e66f5',
    secondary: '#8839ef',
    text: '#4c4f69',
    textSecondary: '#5c5f77',
    textMuted: '#9ca0b0',
    border: '#dce0e8',
    hover: '#dce0e8',
    selected: '#1e66f5',
    selectedText: '#eff1f5',
    success: '#40a02b',
    warning: '#df8e1d',
    error: '#d20f39',
    info: '#209fb5',
  },
};

export const availableThemes = [nord, dracula, tokyoNight, gruvboxDark, gruvboxLight, solarizedLight, catppuccinLatte];

export const darkThemes = availableThemes.filter((t) => t.type === 'dark');
export const lightThemes = availableThemes.filter((t) => t.type === 'light');

export const currentTheme = writable<Theme>(nord);

export function applyTheme(theme: Theme) {
  const root = document.documentElement;

  Object.entries(theme.colors).forEach(([key, value]) => {
    root.style.setProperty(`--color-${key}`, value);
  });

  currentTheme.set(theme);

  // Save to localStorage
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem('theme', theme.name);
  }
}

export function detectOSTheme(): 'dark' | 'light' {
  if (typeof window !== 'undefined' && window.matchMedia) {
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  }
  return 'dark'; // Default to dark
}

export function getThemeForOSPreference(): Theme {
  const osTheme = detectOSTheme();

  // Get all themes matching the OS preference
  const matchingThemes = availableThemes.filter((t) => t.type === osTheme);

  // Return the first matching theme (Nord for dark, Gruvbox Light for light)
  if (matchingThemes.length > 0) {
    return matchingThemes[0];
  }

  // Fallback to Nord
  return nord;
}

export function loadSavedTheme() {
  if (typeof localStorage !== 'undefined') {
    const savedTheme = localStorage.getItem('theme');
    if (savedTheme) {
      const theme = availableThemes.find((t) => t.name === savedTheme);
      if (theme) {
        applyTheme(theme);
        return;
      }
    }
  }

  // If no saved theme, use OS preference
  const osTheme = getThemeForOSPreference();
  applyTheme(osTheme);
}

// Optional: Listen for OS theme changes
export function watchOSThemeChanges(callback?: (theme: Theme) => void) {
  if (typeof window !== 'undefined' && window.matchMedia) {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');

    const handler = (e: MediaQueryListEvent) => {
      // Only auto-switch if user hasn't manually selected a theme
      if (typeof localStorage !== 'undefined') {
        const savedTheme = localStorage.getItem('theme');
        if (!savedTheme) {
          const newTheme = getThemeForOSPreference();
          applyTheme(newTheme);
          if (callback) {
            callback(newTheme);
          }
        }
      }
    };

    mediaQuery.addEventListener('change', handler);

    // Return cleanup function
    return () => mediaQuery.removeEventListener('change', handler);
  }

  return () => {}; // No-op cleanup
}
