# Fenrir Launcher

A native Linux game launcher that detects, configures, and launches Windows
games through Wine and Proton. Built in Rust, because life's too short for
slow launchers.

[![CI](https://github.com/burlone0/Fenrir-Launcher/actions/workflows/ci.yml/badge.svg)](https://github.com/burlone0/Fenrir-Launcher/actions/workflows/ci.yml)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](LICENSE)
![Version: 0.5.0](https://img.shields.io/badge/version-0.5.0-green.svg)

## What It Does

**Core launcher:**

- **Scans your game folders** and automatically identifies games using
  signature-based pattern matching -- it knows what a Steam crack looks like,
  what an OnlineFix release looks like, what a FitGirl repack looks like
- **Detects games from multiple stores** -- Steam cracks, repacks, scene
  releases, GOG, and Epic Games Store, all in one library
- **Creates isolated Wine prefixes** for each game -- no contamination, no
  shared state, no mysterious breakage
- **Auto-tunes Wine settings** based on the detected game type -- DLL overrides,
  DXVK, esync/fsync, environment variables, all handled
- **Installs winetricks components** automatically when a profile requires them
  (e.g. .NET 6 for MelonLoader-modded games)
- **Launches games** with the right runtime and configuration, tracks playtime,
  logs output, and supports stopping a running game from the UI
- **Downloads runtimes automatically** -- fetch GE-Proton or Wine-GE directly
  from GitHub, with SHA-512 checksum verification and progress tracking
- **Stays offline** -- zero network connections by default, no telemetry, no
  phoning home
- **Runs fast** -- native Rust binary, instant startup, low memory footprint

**Desktop GUI** (Tauri v2 + React, since v0.3.0):

- **Library view** -- grid of game cards with cover art, status, store and
  crack-type badges, filterable and searchable
- **Settings screen** -- edit every config option in-app, no TOML editing
- **Per-game log viewer** -- read stdout/stderr from any past launch directly
  in the UI
- **First-run onboarding wizard** -- guides new users through scan paths and
  runtime install
- **Theme support** -- dark, light, or follow system
- **Keyboard shortcuts** -- `Ctrl+S` opens scan, `Enter` configures or launches
  the selected game

## Screenshots

Screenshots of the GUI live in [docs/assets/screenshots/](docs/assets/screenshots/).
That directory's [README](docs/assets/screenshots/README.md) lists what each
image should show and where it's referenced -- handy if you want to help
populate them.

## Quick Start

```bash
# Build
git clone https://github.com/burlone0/Fenrir-Launcher.git
cd Fenrir-Launcher
cargo build --release
```

### Using the GUI

```bash
# From the repo root
cd fenrir-gui
npm install
npm run tauri dev          # development mode
npm run tauri build        # production binary in src-tauri/target/release/
```

On first launch the onboarding wizard walks you through scan directories,
runtime installation, and theme. After that, everything is point-and-click.

### Using the CLI

```bash
# Copy to PATH (optional)
cp target/release/fenrir-cli ~/.local/bin/fenrir

# Scan a game directory
fenrir scan --path /mnt/games/

# See what it found
fenrir list

# Set up a game (creates prefix, applies tuning)
fenrir configure "Elden Ring"

# Play
fenrir launch "Elden Ring"
```

The CLI and the GUI share the same database -- changes made through one show
up in the other immediately.

## Requirements

- Linux (tested on Arch, Fedora, Ubuntu)
- Rust stable toolchain (for building)
- Wine or Proton (at least one installed)
- A GPU with Vulkan support (for DXVK -- most GPUs from 2015 onward qualify)

**Optional:**

- **winetricks** -- needed only for games whose profile requires runtime
  libraries (.NET, VC++ redistributables, Microsoft fonts). Modded OnlineFix
  releases that ship MelonLoader for multiplayer (e.g. Megabonk +
  BonkWithFriends) fall into this category. If winetricks is missing, Fenrir
  warns during `configure` instead of failing -- install it and re-run.
  Most distros ship it as `winetricks` in their package manager. See
  [MelonLoader Games](docs/user/troubleshooting/melonloader-games.md) for
  the worked case.

## Recommendations

**Runtime:** [GE-Proton](https://github.com/GloriousEggroll/proton-ge-custom)
gives the best game compatibility. You can install it directly from Fenrir:

```bash
fenrir runtime available          # see what's out there
fenrir runtime install GE-Proton9-20
```

**Kernel:** 5.16 or newer for fsync support. Any recent distro ships this.

**RAM:** 8 GB or more for typical modern games. Wine prefixes themselves are
lightweight; the game's own requirements are what matter.

**Storage:** Wine prefixes can be 1-5 GB each. Pointing `prefix_dir` at a
spacious drive is a good call. See [Configuration](docs/user/configuration.md).

## Commands

| Command | Description |
|---------|-------------|
| `fenrir scan [--path DIR]` | Scan for games in a directory |
| `fenrir list` | Show all games in library |
| `fenrir info <GAME>` | Show detailed game info |
| `fenrir add <PATH>` | Manually add a game |
| `fenrir confirm <GAME>` | Confirm a low-confidence detected game |
| `fenrir config [--set K --value V]` | View or change settings |
| `fenrir configure <GAME> [--clean]` | Create prefix and apply tuning |
| `fenrir launch <GAME>` | Launch a configured game |
| `fenrir runtime list\|available\|install\|set-default` | Manage Wine/Proton runtimes |

`<GAME>` accepts a title (fuzzy-matched) or UUID.

Global flags `--verbose` / `-v` and `--quiet` / `-q` work on every command.
Full syntax and examples: [Commands Reference](docs/user/commands.md).

## Project Status

Fenrir is under active development. Here's where things stand:

- **Fase 1 -- Core + CLI** -- done. Scanning, detection, configuration, and
  launch all work from the terminal.
- **Fase 2 -- Runtime management** -- done. Automatic download of GE-Proton
  and Wine-GE, expanded detection (GOG, Epic, all major crack types), robust
  error handling with hints, structured logging.
- **Fase 3 -- GUI** -- done. Tauri-based visual launcher (v0.3.0), concurrent
  launch protection and kill-game (v0.4.0), Settings + log viewer + theme +
  onboarding + cover art (v0.5.0).
- **Fase 4 -- Multi-store + community** -- planned. Native GOG and Epic
  Galaxy integration beyond filesystem detection, Lutris/Heroic import,
  metadata fetching, community-contributed signatures.

## Documentation

**For users:**
- [Installation](docs/user/installation.md)
- [Getting Started](docs/user/getting-started.md) -- CLI walkthrough
- [GUI Guide](docs/user/gui-guide.md) -- desktop app walkthrough
- [Commands Reference](docs/user/commands.md)
- [Configuration](docs/user/configuration.md)
- [Troubleshooting](docs/user/troubleshooting.md)
- [FAQ](docs/user/faq.md)

**For developers:**
- [Architecture](docs/dev/architecture.md)
- [Signatures Guide](docs/dev/signatures-guide.md)
- [Profiles Guide](docs/dev/profiles-guide.md)
- [Adding a Store](docs/dev/adding-a-store.md)

## Legal

Fenrir is a Wine launcher. It doesn't download, distribute, or unlock game
files -- it launches executables that already exist on your machine and
configures Wine to run them correctly.

Fenrir identifies game sources (Steam, GOG, Epic) and release types
(FitGirl, OnlineFix, etc.) because that information determines the right Wine
configuration. Knowing a game uses the OnlineFix DLL means you need specific
DLL overrides set. This is a technical classification, not an endorsement of
any particular method of obtaining software.

You are responsible for complying with the license terms of any software you
run through Fenrir.

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for setup
instructions, commit conventions, and how to extend Fenrir's detection and
tuning systems without writing Rust.

## License

[GPL-3.0-only](LICENSE)
