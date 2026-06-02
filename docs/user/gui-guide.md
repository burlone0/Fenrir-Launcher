# GUI Guide

A walkthrough of the Fenrir desktop app. If you're more of a terminal person,
the same operations are available from the CLI -- see
[Getting Started](getting-started.md) and the
[Commands Reference](commands.md). The two share a database, so anything you
do in one shows up immediately in the other.

This guide is written against v0.5.0. Older versions may be missing some of
the screens described below.

## Starting the app

From a built tree:

```bash
cd fenrir-gui
npm install                # only once
npm run tauri dev          # development mode (live reload)
npm run tauri build        # builds a production binary
```

The production binary lands in `fenrir-gui/src-tauri/target/release/`. Copy
it somewhere convenient or run it directly.

On Wayland sessions with WebKit hiccups (NVIDIA proprietary driver, some
KDE/Sway setups) try `npm run tauri:x11` or `npm run tauri:x11:wk`. See
[fenrir-gui/README.md](../../fenrir-gui/README.md) for the details.

## First-run onboarding

The very first time you open the app, a four-step wizard pops up:

1. **Welcome** -- just a hello, no input needed.
2. **Install a Wine/Proton runtime** -- jumps you to the Runtimes tab. You
   need at least one runtime installed before any game can launch. If you
   already have system Wine or a GE-Proton you set up manually, you can
   skip this step.
3. **Scan your game directories** -- opens the Library and triggers the
   scan dialog. Point it at a folder; Fenrir does the rest.
4. **You're all set** -- closes the wizard.

You can dismiss the wizard at any step with the "Skip" button in the bottom
left. The wizard remembers that it's been seen (via `localStorage`) and
won't show again. If you want to replay it, clear browser storage from the
Tauri devtools or reset `fenrir.onboarding_done` in your app data.

## Library

The main screen. A grid of game cards, plus a detail panel that slides in
from the right when you click a card.

**Filter bar (top):**

- **All / Detected / Configured / Ready / Broken** -- show only games in
  that state. The same status enum the database stores.
- **Scan button** -- opens the [ScanView](#scanview) overlay. Keyboard
  shortcut: `Ctrl+S`.

**Game card:**

Each card shows the title, cover art (if present), a store badge (Steam,
GOG, Epic), a crack-type chip (OnlineFix, FitGirl, etc. when known), and a
status badge. Hover effects make the focused card a bit brighter; the
selected card has a sky-blue ring.

**Game detail panel (right side, when a card is selected):**

Shows everything the database knows about the game -- ID, paths,
runtime assignment, play time, last played -- plus action buttons:

- **Configure** / **Reconfigure** -- creates or rebuilds the Wine prefix,
  applies the matching tuning profile. If a `[winetricks]` section is in
  the profile (e.g. MelonLoader games), this is where .NET 6 gets
  installed. Optionally tick **Clean files** to remove repack residue
  before configuring.
- **Launch** -- starts the game. Shows as **Launching…** while the
  subprocess spawns, then a Stop button while the game is alive.
- **Stop** -- sends SIGTERM to the running game. Wine/Proton shuts the
  game down cleanly within a second or two.
- **Delete** -- removes the game from the library. The Wine prefix on
  disk stays untouched in case you change your mind.

The log viewer sits at the bottom of the detail panel -- see
[Log viewer](#log-viewer).

**Keyboard shortcuts:**

- `Ctrl+S` -- open the scan dialog
- `Enter` on a selected game -- launches a Configured/Ready game, or
  configures a Detected one

## ScanView

Opens as an overlay over the Library. Three phases:

### 1. Input

A text field for a path, or leave it empty to use the directories you
configured in Settings. Click **Scan** to start.

Tip: paste a path that's already mounted -- network shares, external
drives, whatever. Fenrir walks up to 4 levels deep, skipping obvious
non-game folders (`_Redist`, `DirectX`, `Redistributables`, version-numbered
directories).

### 2. Scanning

A progress bar. For most libraries this takes seconds; very large
collections on slow disks can take a minute or two. The bar shows the
current directory being walked.

### 3. Results

Two lists:

- **High confidence** -- games Fenrir is sure about (score ≥ 60, or
  whatever the signature's `auto_add_threshold` was). Already added to your
  library, marked with a green `✓ Added`.
- **Needs confirmation** -- games Fenrir found but isn't 100% sure about
  (score 30-59). Each has a **Confirm** button. Click to add to the
  library, skip to ignore.

Each row shows: title, store badge, crack-type chip, confidence score
in points (the raw scanner output -- useful for debugging signatures).

When you're done, close the overlay. The Library refreshes automatically.

## RuntimeManager

The Runtimes tab. Two sections.

### Installed runtimes

A table with one row per discovered runtime:

| Column | What it means |
|--------|---------------|
| Version | The runtime's version string (`GE-Proton9-20`, `8.5-staging`, etc.) |
| Type | Wine / Proton / ProtonGE / WineGE |
| Source | System (from your package manager), Steam (from Steam's compatibilitytools.d), or Downloaded (installed by Fenrir) |
| Default | Green "Default" badge if it's the current default; otherwise a "Set default" link |

The default runtime is used when a game's `runtime_id` is `auto` (the
default). Per-game overrides aren't editable from the GUI yet -- you'd
need to set `user_overrides` in the database for that.

### Available runtimes (downloadable)

A dropdown to pick **proton-ge** or **wine-ge**, plus a **Fetch** button
that queries the GloriousEggroll GitHub releases API. Lists recent
versions with their download size.

Click **Install** next to a version to download it. A confirm dialog
appears with the version and size; confirming starts the download.
A progress bar shows bytes received / total bytes. SHA-512 checksum is
verified before extraction; failed checksums abort the install and the
partial download is deleted.

Installed downloads land in `~/.local/share/fenrir/runtimes/` and appear
in the table above with `Source: Downloaded`.

## Settings

In-app editor for the same TOML config you'd find at
`~/.config/fenrir/config.toml`. Sections:

### General (read-only display)

Shows the resolved paths Fenrir is currently using:

- `library_db` -- the SQLite database file
- `prefix_dir` -- where Wine prefixes get created
- `runtime_dir` -- where downloaded runtimes land

These are read-only in the GUI -- they're path-shaped and easy to break
by typo. If you need to change them, edit the TOML file directly and
restart the app.

### Defaults (editable)

The Wine settings applied to every new game configuration, unless
overridden by a profile or user override:

- **Default runtime** -- dropdown populated from your installed runtimes.
  `auto` picks the first available; otherwise pick a specific version
  by ID.
- **DXVK** -- toggle. On by default. Translates DX9/10/11 to Vulkan.
- **VKD3D** -- toggle. Off by default. Translates DX12 to Vulkan. Enable
  for DX12 games.
- **esync** -- toggle. On by default. eventfd-based Wine synchronization.
- **fsync** -- toggle. On by default. Futex-based Wine synchronization
  (needs kernel 5.16+).

### Scan and Privacy

The `[scan]` and `[privacy]` sections of config are not currently
editable from the GUI. The `[privacy]` options are no-ops anyway -- they
gate features that aren't implemented yet (metadata fetching, cover
art download). Scan directories can be configured via the CLI:

```bash
fenrir config --set scan.game_dirs --value "/mnt/games/,/home/user/Games/"
```

Changes saved in the Settings screen update both the in-memory app state
and `~/.config/fenrir/config.toml`, so subsequent `get_config` calls
return the new values and CLI use sees them too.

## Log viewer

A panel at the bottom of the [Game detail panel](#library). Shows the
contents of `~/.local/share/fenrir/logs/<game-uuid>.log`, which is the
combined stdout/stderr of the most recent launch.

- **Refresh** -- re-reads the file. Useful while a game is running.
- The view auto-scrolls to the bottom on load.
- If the file is empty or doesn't exist, you see "Play this game to
  generate log output."

Logs are accumulative for the *current launch only* -- a new launch
overwrites the file. If you want persistent history, copy the log
elsewhere before launching again.

For grep-friendly access, the log file is just a text file. From a
terminal:

```bash
tail -f ~/.local/share/fenrir/logs/<uuid>.log
```

You can find the UUID by clicking the game in the GUI and looking at the
`id` field, or by running `fenrir info "Game Name"` from the CLI.

## Theme

Three options, chosen at first launch via the onboarding wizard and
changeable later from Settings:

- **Dark** -- default. The whole app uses Fenrir's zinc/sky palette.
- **Light** -- inverted palette. Same layout, lighter background.
- **System** -- follows your desktop environment's preference via
  `prefers-color-scheme`.

Theme switching is instant -- no app restart needed.

## Notifications

Toast-style messages slide in from the bottom-right when something
finishes:

- **Success** (green) -- configure done, runtime installed, etc.
- **Error** (red) -- something failed. The full error message is in the
  toast and stays until dismissed.
- **Info** (zinc) -- neutral state changes.

Toasts auto-dismiss after a few seconds for success/info; errors stick
until you click them.

## What's still CLI-only

The GUI covers the everyday flow -- scan, configure, launch, manage
runtimes, edit defaults. A few power-user operations are still
CLI-exclusive:

- Editing `[scan]` and `[privacy]` config sections (the Settings screen
  doesn't expose them yet)
- Per-game `user_overrides` -- override DXVK/runtime/etc. on a single game
- Manual `fenrir add <PATH>` for games that scan can't find
- Running with `--verbose` for signature debugging output

If a CLI-only feature is one you'd want a button for, open an issue.
