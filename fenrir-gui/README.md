# Fenrir GUI

Tauri v2 + React 18 + TypeScript desktop frontend for Fenrir Launcher.

The Rust backend (`src-tauri/`) wraps `fenrir-core` and exposes Tauri commands
+ events to the React frontend (`src/`).

## Development

```bash
cd fenrir-gui
npm install
npm run tauri dev
```

The first build compiles `fenrir-core` and Tauri (~1-2 min); subsequent runs
are cached.

### Wayland / WebKit caveats

WebKit2GTK has known issues on certain Wayland compositors (NVIDIA proprietary
driver, some KDE/Sway/Hyprland configurations). If you hit a Wayland protocol
error or a black/blank window, fall back to XWayland:

```bash
npm run tauri:x11        # forces GDK_BACKEND=x11
npm run tauri:x11:wk     # also disables WebKit GPU compositing
```

Use `tauri:x11:wk` if the X11-only mode shows a black window — that's the
WebKit hardware compositor failing on your driver. The CPU compositor is
slower but always works.

## Project layout

```
fenrir-gui/
  src-tauri/          Rust backend (Tauri commands wrapping fenrir-core)
    src/
      lib.rs          AppState (DB + config) + command registration
      commands/       games / scan / runtime / config commands
  src/                React frontend
    views/            Library, ScanView, RuntimeManager
    components/       GameCard, StatusBadge, etc.
    stores/           Zustand stores (games, scan, runtimes, ui)
    lib/              types.ts, commands.ts, events.ts
```

## Production build

```bash
npm run tauri build
```

Produces a self-contained binary in `target/release/`.
