# Changelog

All notable changes to Fenrir Launcher will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

Nothing yet.

## [0.2.0] - 2026-04-16

Fase 2 complete. Runtime download management, expanded game detection
(GOG and Epic), four new tuning profiles, structured logging, and better
error messages with actionable hints.

### Added

- **Runtime download manager** -- fetch GE-Proton and Wine-GE directly from
  GitHub Releases, with SHA-512 checksum verification and per-chunk progress
  callbacks.
- **`runtime available` command** -- list downloadable runtimes by kind
  (`proton-ge` or `wine-ge`), queried live from GitHub.
- **`runtime install` command** -- download, verify, and extract a runtime
  to `~/.local/share/fenrir/runtimes/` with a progress bar.
- **`confirm` command** -- promote a low-confidence game (score 30-59) from
  "needs confirmation" to the library.
- **Global `--verbose` / `--quiet` flags** -- override log level for any
  command; `--verbose` sets debug/trace, `--quiet` suppresses everything
  except errors.
- **GOG detection signatures** (`data/signatures/gog.toml`) -- three patterns:
  `gog_info` (goggame-*.info), `gog_galaxy` (GalaxyClient.dll),
  `gog_installer` (game.id).
- **Epic detection signatures** (`data/signatures/epic.toml`) -- two patterns:
  `epic_emu` (EOSSDK-Win64-Shipping.dll + ScreamAPI) and `epic_generic`
  (EpicGamesLauncher.lnk).
- **New crack type: `SteamRip`** -- for Steam library rips distinct from
  scene/repack releases.
- **Four new tuning profiles** -- `dodi`, `fitgirl`, `scene`, `gog`
  (previously all fell back to `steam_generic`).
- **Error hints system** -- `FenrirError::suggestion()` returns a short
  actionable message for common errors; printed as `hint:` below the error
  line in the CLI.
- **Structured logging** -- `tracing` + `tracing-subscriber` throughout
  `fenrir-core` and `fenrir-cli`; log level controlled by `RUST_LOG` or
  the new global flags.
- **Cleanup module** -- `CleanupPlan` / `CleanupEntry` for dry-run and
  destructive removal of post-extract noise. Accessible via `--clean` in
  `fenrir configure`.

### Changed

- `fenrir configure` accepts `--clean` to remove repack artifacts
  (`.url` files, installer directories) after prefix setup.
- `fenrir runtime` subcommand expanded from `list | set-default` to
  `list | set-default | available | install`.

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
