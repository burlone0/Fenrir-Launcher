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
- **Detects GOG and Epic games** -- in addition to Steam cracks, repacks, and
  scene releases
- **Creates isolated Wine prefixes** for each game -- no contamination, no
  shared state, no mysterious breakage
- **Auto-tunes Wine settings** based on the detected game type -- DLL overrides,
  DXVK, esync/fsync, environment variables, all handled
- **Launches games** with the right runtime and configuration, tracks playtime,
  logs output
- **Downloads runtimes automatically** -- fetch GE-Proton or Wine-GE directly
  from GitHub, with SHA-512 checksum verification and progress tracking
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

## Requirements

- Linux (tested on Arch, Fedora, Ubuntu)
- Rust stable toolchain (for building)
- Wine or Proton (at least one installed)
- A GPU with Vulkan support (for DXVK -- most GPUs from 2015 onward qualify)

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
