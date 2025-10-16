import { featureFlags } from '$lib/featureFlags';
import { get, writable } from 'svelte/store';
import { moveSelection } from './games';
import {
  browseView,
  currentPage,
  cycleFocusPanel,
  focusedPanel,
  navigateTo,
  openGameDetails,
  type Page,
} from './navigation';

export const keyboardEnabled = writable<boolean>(true);

// Keyboard context types
type KeyboardContext = {
  page: Page;
  view?: string;
  isTyping: boolean;
  ctrl: boolean;
  shift: boolean;
};

// Key binding definition
type KeyBinding = {
  key: string;
  condition?: (ctx: KeyboardContext) => boolean;
  handler: (e: KeyboardEvent, ctx: KeyboardContext) => void;
};

// Utility functions
function isUserTyping(target: HTMLElement): boolean {
  return target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable;
}

function getTabOrder(): Page[] {
  const tabs: Page[] = ['browse', 'popular', 'pinkpaw'];
  if (featureFlags.torrentClient) tabs.push('downloads');
  tabs.push('settings');
  if (featureFlags.stats) tabs.push('stats');
  return tabs;
}

function focusSearch() {
  const searchInput = document.querySelector('input[type="search"], input[placeholder*="Search"]') as HTMLInputElement;
  if (searchInput) {
    searchInput.focus();
    searchInput.select();
  }
}

function cycleTab(direction: 'next' | 'previous') {
  const current = get(currentPage);
  const TAB_ORDER = getTabOrder();
  const currentIndex = TAB_ORDER.indexOf(current);
  if (currentIndex === -1) return;

  const nextIndex =
    direction === 'next'
      ? (currentIndex + 1) % TAB_ORDER.length
      : (currentIndex - 1 + TAB_ORDER.length) % TAB_ORDER.length;

  navigateTo(TAB_ORDER[nextIndex]);
}

// Global key bindings (work everywhere)
const globalBindings: KeyBinding[] = [
  {
    key: '/',
    handler: () => focusSearch(),
  },
  {
    key: 'f',
    condition: (ctx) => ctx.ctrl && !ctx.shift,
    handler: () => focusSearch(),
  },
  {
    key: 'r',
    condition: (ctx) => ctx.ctrl,
    handler: () => window.location.reload(),
  },
];

// Browse list view bindings
const browseListBindings: KeyBinding[] = [
  {
    key: 'ArrowUp',
    condition: (ctx) => !ctx.ctrl && !ctx.isTyping,
    handler: () => {
      const panel = get(focusedPanel);
      if (panel === 'gamelist') {
        moveSelection('up');
      }
      // Other panels handle their own arrow navigation
    },
  },
  {
    key: 'ArrowDown',
    condition: (ctx) => !ctx.ctrl && !ctx.isTyping,
    handler: () => {
      const panel = get(focusedPanel);
      if (panel === 'gamelist') {
        moveSelection('down');
      }
      // Other panels handle their own arrow navigation
    },
  },
  {
    key: 'PageUp',
    condition: (ctx) => !ctx.ctrl && !ctx.isTyping,
    handler: () => {
      const panel = get(focusedPanel);
      if (panel === 'gamelist') {
        moveSelection('up', 10);
      }
    },
  },
  {
    key: 'PageDown',
    condition: (ctx) => !ctx.ctrl && !ctx.isTyping,
    handler: () => {
      const panel = get(focusedPanel);
      if (panel === 'gamelist') {
        moveSelection('down', 10);
      }
    },
  },
  {
    key: 'Enter',
    condition: (ctx) => !ctx.ctrl && !ctx.shift && !ctx.isTyping,
    handler: () => {
      const panel = get(focusedPanel);
      if (panel === 'gamelist') {
        openGameDetails();
      }
    },
  },
  {
    key: 'Tab',
    condition: (ctx) => !ctx.ctrl,
    handler: (e, ctx) => {
      e.preventDefault();
      cycleFocusPanel(ctx.shift ? 'previous' : 'next');
    },
  },
];

// Browse details view bindings
const browseDetailsBindings: KeyBinding[] = [
  // Details view handles its own keys (Esc, Backspace, d)
  // We just prevent interference from list bindings
];

// Global navigation bindings
const navigationBindings: KeyBinding[] = [
  {
    key: ']',
    condition: (ctx) => ctx.ctrl && !ctx.isTyping,
    handler: () => cycleTab('next'),
  },
  {
    key: '[',
    condition: (ctx) => ctx.ctrl && !ctx.isTyping,
    handler: () => cycleTab('previous'),
  },
];

// Main keyboard handler
function handleKeyPress(e: KeyboardEvent): boolean {
  const ctx: KeyboardContext = {
    page: get(currentPage),
    view: get(browseView),
    isTyping: isUserTyping(e.target as HTMLElement),
    ctrl: e.ctrlKey || e.metaKey,
    shift: e.shiftKey,
  };

  // Try global bindings first (always available)
  for (const binding of globalBindings) {
    if (e.key === binding.key && (!binding.condition || binding.condition(ctx))) {
      binding.handler(e, ctx);
      return true;
    }
  }

  // Context-specific bindings
  let contextBindings: KeyBinding[] = [];

  if (ctx.page === 'browse' && ctx.view === 'list') {
    contextBindings = browseListBindings;
  } else if (ctx.page === 'browse' && ctx.view === 'details') {
    // Let details view handle its own shortcuts
    contextBindings = browseDetailsBindings;
  }

  // Try context bindings
  for (const binding of contextBindings) {
    if (e.key === binding.key && (!binding.condition || binding.condition(ctx))) {
      binding.handler(e, ctx);
      return true;
    }
  }

  // Try navigation bindings (work everywhere except when typing)
  if (!ctx.isTyping) {
    for (const binding of navigationBindings) {
      if (e.key === binding.key && (!binding.condition || binding.condition(ctx))) {
        binding.handler(e, ctx);
        return true;
      }
    }
  }

  return false;
}

export function initKeyboardShortcuts() {
  if (typeof window === 'undefined') return;

  window.addEventListener('keydown', async (e) => {
    const enabled = get(keyboardEnabled);
    if (!enabled) return;

    const handled = handleKeyPress(e);
    if (handled) {
      e.preventDefault();
    }
  });
}
