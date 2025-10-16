import { writable } from 'svelte/store';

export interface Theme {
  name: string;
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
    hover: '#44475a',
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
    hover: '#292e42',
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
    hover: '#eee8d5',
    selected: '#268bd2',
    selectedText: '#fdf6e3',
    success: '#859900',
    warning: '#b58900',
    error: '#dc322f',
    info: '#2aa198',
  },
};

export const availableThemes = [nord, dracula, tokyoNight, gruvboxDark, gruvboxLight, solarizedLight];

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

export function loadSavedTheme() {
  if (typeof localStorage !== 'undefined') {
    const savedTheme = localStorage.getItem('theme');
    if (savedTheme) {
      const theme = availableThemes.find((t) => t.name === savedTheme);
      if (theme) {
        applyTheme(theme);
      }
    }
  }
}
