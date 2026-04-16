# Fenrir Launcher

A native Linux game launcher that detects, configures, and launches Windows
games through Wine and Proton. Built in Rust, because life's too short for
slow launchers.

[![CI](https://github.com/burlone0/Fenrir-Launcher/actions/workflows/ci.yml/badge.svg)](https://github.com/burlone0/Fenrir-Launcher/actions/workflows/ci.yml)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](LICENSE)
![Version: 0.2.0](https://img.shields.io/badge/version-0.2.0-green.svg)

## What It Does

- **Scans your game folders** and automatically identifies games using
  signature-based pattern matching -- it knows what a Steam crack looks like,
  what an OnlineFix release looks like, what a FitGirl repack looks like
- **Creates isolated Wine prefixes** for each game -- no contamination, no
  shared state, no mysterious breakage
- **Auto-tunes Wine settings** based on the detected game type -- DLL overrides,
  DXVK, esync/fsync, environment variables, all handled
- **Launches games** with the right runtime and configuration, tracks playtime,
  logs output
- **Stays offline** -- zero network connections by default, no telemetry, no
  phoning home
- **Runs fast** -- native Rust binary, instant startup, low memory footprint
- **Downloads runtimes automatically** -- fetch GE-Proton or Wine-GE directly from GitHub, with SHA-512 checksum verification and progress tracking
- **Detects GOG and Epic games** -- in addition to Steam cracks, repacks, and scene releases

## Quick Start

```bash
# Build
git clone https://github.com/burlone0/Fenrir-Launcher.git
cd Fenrir-Launcher
cargo build --release

# Scan a game directory
./target/release/fenrir-cli scan --path /mnt/games/

# See what it found
./target/release/fenrir-cli list

# Set up a game (creates prefix, applies tuning)
./target/release/fenrir-cli configure "Elden Ring"

# Play
./target/release/fenrir-cli launch "Elden Ring"
```

## Requirements

- Linux (tested on Arch, Fedora, Ubuntu)
- Rust stable toolchain (for building)
- Wine or Proton (at least one installed)

## Commands

| Command | Description |
|---------|-------------|
| `fenrir scan [--path DIR]` | Scan for games in a directory |
| `fenrir list` | Show all games in library |
| `fenrir info <GAME>` | Show detailed game info |
| `fenrir add <PATH>` | Manually add a game |
| `fenrir config [--set K --value V]` | View or change settings |
| `fenrir configure <GAME>` | Create prefix and apply tuning |
| `fenrir launch <GAME>` | Launch a configured game |
| `fenrir runtime list\|set-default` | Manage Wine/Proton runtimes |

`<GAME>` accepts a title (fuzzy-matched) or UUID.

## Project Status

Fenrir is under active development. Here's where things stand:

- **Fase 1 -- Core + CLI** -- done. Scanning, detection, configuration, and
  launch all work from the terminal.
- **Fase 2 -- Runtime management** -- done. Automatic download of GE-Proton
  and Wine-GE, expanded detection (GOG, Epic, all major crack types), robust
  error handling with hints, structured logging.
- **Fase 3 -- GUI** -- next. Tauri-based visual launcher with game library,
  cover art, and configuration UI.
- **Fase 4 -- Multi-store** -- planned. Import from Lutris/Heroic,
  metadata fetching, community signatures.

## Documentation

**For users:**
- [Installation](docs/user/installation.md)
- [Getting Started](docs/user/getting-started.md)
- [Commands Reference](docs/user/commands.md)
- [Configuration](docs/user/configuration.md)

**For developers:**
- [Architecture](docs/dev/architecture.md)
- [Signatures Guide](docs/dev/signatures-guide.md)
- [Profiles Guide](docs/dev/profiles-guide.md)
- [Adding a Store](docs/dev/adding-a-store.md)

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for setup
instructions, commit conventions, and how to extend Fenrir's detection and
tuning systems without writing Rust.

## License

[GPL-3.0-only](LICENSE)
