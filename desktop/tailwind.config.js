/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  theme: {
    extend: {
      colors: {
        // Theme colors will be CSS variables
        primary: 'var(--color-primary)',
        secondary: 'var(--color-secondary)',
        background: 'var(--color-background)',
        backgroundSecondary: 'var(--color-backgroundSecondary)',
        backgroundTertiary: 'var(--color-backgroundTertiary)',
        text: 'var(--color-text)',
        textSecondary: 'var(--color-textSecondary)',
        textMuted: 'var(--color-textMuted)',
        border: 'var(--color-border)',
        hover: 'var(--color-hover)',
        selected: 'var(--color-selected)',
        selectedText: 'var(--color-selectedText)',
        success: 'var(--color-success)',
        warning: 'var(--color-warning)',
        error: 'var(--color-error)',
        info: 'var(--color-info)',
      },
    },
  },
  plugins: [],
};
