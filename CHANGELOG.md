# Changelog

All notable changes to Fenrir Launcher will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

Nothing yet.

## [0.2.0] - 2026-04-16

Fase 2 complete. Runtime download manager, expanded detection, robust error
handling, structured logging.

### Added

- **Runtime download** -- fetch GE-Proton and Wine-GE releases directly from
  GitHub. Includes SHA-512 checksum verification and a progress bar.
- **`fenrir runtime available`** -- list available runtimes for download
  (`--kind proton-ge` or `wine-ge`).
- **`fenrir runtime install <VERSION>`** -- download and install a runtime to
  `~/.local/share/fenrir/runtimes/`.
- **`fenrir confirm <GAME>`** -- confirm and add a low-confidence detected game
  to the library.
- **Global flags** -- `--verbose` (`-v`) enables debug logging, `--quiet` (`-q`)
  suppresses everything except errors. Available on all commands.
- **GOG detection** -- three new signatures: `goggame-*.info`, `GalaxyClient.dll`,
  `game.id`. GOG profile with no Steam DLL overrides.
- **Epic detection** -- two new signatures: `EOSSDK-Win64-Shipping.dll` (ScreamAPI
  support), `EpicGamesLauncher.lnk`.
- **New tuning profiles** -- dedicated Wine profiles for FitGirl, DODI, Scene,
  and GOG game types.
- **Steam overlay injection** -- Fenrir injects the Steam overlay when launching
  through Proton.
- **Proton warning suppression** -- noisy Proton stderr output filtered at launch.
- **Error hints** -- actionable suggestions shown after errors (e.g. "no runtime
  found -- run `fenrir runtime install`").

### Changed

- Structured logging replaces raw `println!` in the CLI. Log level controlled
  by `RUST_LOG` or `--verbose`/`--quiet` flags.

## [0.1.0] - 2026-04-13

First release -- Fase 1 complete. Core library and CLI prototype, covering game
detection through launch.

### Added

- **Config module** -- TOML configuration with XDG Base Directory support,
  load/save, sensible defaults.
- **Library module** -- Game data model with SQLite storage, full CRUD operations,
  and fuzzy title search.
- **Scanner module** -- Recursive directory walk, TOML-based signature matching
  with confidence scoring (required +30, optional +15, boost +10), automatic
  title extraction and cleanup.
- **Runtime module** -- Wine and Proton discovery across system paths, Steam
  compatibility tools, and user-managed runtimes. Version parsing and comparison.
- **Prefix module** -- Isolated WINEPREFIX creation per game (wineboot --init),
  TOML profile loading, and auto-tuning (DLL overrides, DXVK/VKD3D, esync/fsync,
  Windows version).
- **Launcher module** -- Wine and Proton command composition, subprocess execution,
  stdout/stderr logging, playtime tracking, exit code recording.
- **CLI** -- Eight commands: `scan`, `list`, `info`, `add`, `config`, `configure`,
  `launch`, `runtime` (list, set-default).
- **Detection signatures** -- Five crack type patterns: Steam generic, OnlineFix,
  FitGirl, DODI, Scene.
- **Tuning profiles** -- Two Wine configuration profiles: steam_generic, onlinefix.
- **Integration tests** -- End-to-end test suite covering the full Fase 1 pipeline.
- **CI pipeline** -- GitHub Actions workflow: rustfmt, clippy (-D warnings),
  tests, release build.
