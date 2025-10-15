# 🎮 FitGirl Browser Desktop App - Quick Start

## What We've Built

A beautiful desktop application inspired by PSVita's PKGj, featuring:

### ✅ Completed Features

- **PKGj-Style UI** - Nostalgic Vita interface with sidebar, game list, and details panel
- **Theme System** - 3 built-in themes (PKGj Classic, Nord, Dracula) + custom theme support
- **Full Keyboard Navigation** - Arrow keys, vim keys (j/k), shortcuts for everything
- **Fast Search** - Real-time filtering with debounce
- **Database Integration** - Direct SQLite access to your crawled data
- **Magnet Link Handling** - Open in system torrent client or copy to clipboard
- **Responsive Layout** - Sidebar, scrollable game list, collapsible details

### 🚧 Coming Soon

- Integrated torrent client (downloads page)
- Settings page (configure paths, limits, etc.)
- Stats page (database analytics)

## 🚀 How to Run

### Step 1: Install Dependencies

```bash
cd desktop
npm install
```

### Step 2: Run the App

```bash
npm run tauri dev
```

This will:

1. Compile the Rust backend
2. Start the Svelte dev server
3. Launch the desktop window

**Note:** Make sure `repacks.db` exists in the parent directory!

## ⌨️ Essential Keyboard Shortcuts

```
Navigation:
  ↑/↓ or j/k     Navigate games
  /              Focus search
  Enter          Open magnet link
  1-9            Open specific magnet
  C              Copy magnet link

Pages:
  Ctrl+1         Browse
  Ctrl+2         Downloads (coming soon)
  Ctrl+3         Settings (coming soon)
  Ctrl+4         Stats (coming soon)

UI:
  T              Theme selector
  F11            Fullscreen
  Q              Quit
```

## 🎨 Themes

Click the 🎨 button in the top right or press `T` to switch themes:

1. **PKGj Classic** (Default) - Purple/pink, nostalgic Vita vibes
2. **Nord** - Arctic, cool blue theme
3. **Dracula** - Dark with purple accents

### Creating Custom Themes

1. Create a JSON file in `desktop/themes/custom/`
2. Use this template:

```json
{
  "name": "My Cool Theme",
  "author": "Your Name",
  "colors": {
    "background": "#1a1a2e",
    "backgroundSecondary": "#16213e",
    "backgroundTertiary": "#0f3460",
    "primary": "#e94560",
    "secondary": "#533483",
    "text": "#eaeaea",
    "textSecondary": "#a0a0a0",
    "textMuted": "#666666",
    "border": "#2a2a3e",
    "hover": "#253555",
    "selected": "#e94560",
    "selectedText": "#ffffff",
    "success": "#00d4aa",
    "warning": "#ffd369",
    "error": "#ff5252",
    "info": "#4fc3f7"
  }
}
```

3. Restart the app and select your theme

## 📁 Project Structure

```
fit-boy/
├── crawler/           # Python crawler (existing)
├── desktop/          # NEW: Desktop app
│   ├── src/
│   │   ├── lib/
│   │   │   ├── components/  # Svelte UI components
│   │   │   └── stores/      # State management
│   │   └── routes/
│   │       └── +page.svelte # Main page
│   ├── src-tauri/
│   │   └── src/
│   │       ├── database.rs  # SQLite queries
│   │       ├── commands.rs  # Tauri commands
│   │       └── lib.rs       # App initialization
│   └── themes/         # Theme JSON files
├── config/
├── repacks.db         # Your crawled database
└── run_crawler.py
```

## 🔧 Development Workflow

### Frontend Changes

Edit files in `desktop/src/` - hot reload is automatic!

### Backend Changes

Edit files in `desktop/src-tauri/src/` - restart `npm run tauri dev`

### Adding a New Tauri Command

1. **Define in Rust** (`desktop/src-tauri/src/commands.rs`):

```rust
#[tauri::command]
pub async fn my_command(param: String) -> Result<String, String> {
    Ok(format!("Hello {}", param))
}
```

2. **Register** (`desktop/src-tauri/src/lib.rs`):

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands
    my_command
])
```

3. **Call from Svelte**:

```typescript
import { invoke } from '@tauri-apps/api/core';

const result = await invoke<string>('my_command', { param: 'World' });
```

## 🏗️ Building for Production

```bash
cd desktop
npm run tauri build
```

Output:

- Windows: `.exe` in `src-tauri/target/release/bundle/`
- macOS: `.dmg` in `src-tauri/target/release/bundle/dmg/`
- Linux: `.AppImage` / `.deb` in `src-tauri/target/release/bundle/`

## 🐛 Troubleshooting

### "repacks.db not found"

Make sure the database exists at `../repacks.db` (relative to desktop folder).

### Compilation errors

```bash
# Update Rust
rustup update

# Clean and rebuild
cd desktop
rm -rf src-tauri/target
npm run tauri dev
```

### Theme not applying

Check browser console (F12) for errors. Make sure JSON is valid.

## 🎯 Next Steps

Want to contribute? Here are some ideas:

1. **Implement torrent client** - Add libtorrent integration in Rust
2. **Downloads page** - Show active/completed downloads with progress
3. **Settings page** - Configure download paths, speed limits, etc.
4. **Filters** - Add category/genre filtering in sidebar
5. **Search improvements** - Add fuzzy search, filters
6. **Game details** - Add screenshots, descriptions (from web scraping)
7. **Theme gallery** - Create a community theme sharing platform

## 📚 Resources

- [Tauri Docs](https://tauri.app/)
- [Svelte Tutorial](https://svelte.dev/tutorial)
- [Rust Book](https://doc.rust-lang.org/book/)
- [TailwindCSS](https://tailwindcss.com/)

## 💡 Tips

- Press `F12` in the app to open DevTools
- Edit `src/app.css` to tweak global styles
- Check `src/lib/stores/` to understand state management
- All keyboard shortcuts are in `src/lib/stores/keyboard.ts`

---

**Enjoy your new FitGirl Browser! 🎮✨**

Questions? Open an issue or check the README files.
