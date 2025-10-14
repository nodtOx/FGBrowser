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

export const availableThemes = [nord, dracula];

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
