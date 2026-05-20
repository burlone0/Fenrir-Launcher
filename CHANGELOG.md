# Changelog

All notable changes to Fenrir Launcher will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

Nothing yet.

## [0.4.0] - 2026-05-20

Concurrent-launch hardening, kill-game support end-to-end, winetricks
component installation for mod-loader-aware profiles, and several scanner
and Wine-profile correctness fixes.

### Added

- **Concurrent launch protection + `kill_game` backend** — `AppState`'s DB
  and config are now wrapped in `Arc<Mutex<...>>` and a new
  `Arc<Mutex<HashMap<Uuid, u32>>>` tracks live game PIDs. `launch_game`
  rejects double-launching the same UUID up-front; the PID is registered
  before spawning the monitor task and removed on exit (including on
  monitor-thread panics, via a cloned `Arc` independent of state borrow).
  New `kill_game(id)` command sends `SIGTERM` via `libc::kill` (Unix), and
  `is_running(id)` provides a cheap query for the frontend to drive
  Launch/Kill button state without polling events. `scan_directory` now
  takes the DB lock per upsert instead of holding it for the whole batch,
  so `list_games` / `get_game` interleave correctly during a long scan.
  Adds `libc 0.2` as a Unix-only dependency.
- **Stop button in GameDetail** — wires `kill_game` and `is_running` to the
  UI. When a game is running, the GameDetail panel shows a Stop button in
  place of the disabled Launch button. Sends `SIGTERM`; `launchingId`
  clears automatically when the launch monitor observes the exit.
- **Winetricks component installation** — new `[winetricks]` section in
  `WineProfile` declares mandatory `.components` and `.optional`
  (best-effort) components installed idempotently via `winetricks -q`
  between `create_prefix` and `apply_profile`. New
  `CrackType::OnlineFixMelonLoader` variant with a matching signature and
  `onlinefix_melonloader.toml` profile (`dotnetdesktop6` + `corefonts`)
  covers OnlineFix bundles with MelonLoader-based multiplayer mods
  (trigger case: Megabonk + BonkWithFriends). GUI wraps the install in
  `spawn_blocking` and emits `configure:step` progress events; missing
  winetricks surfaces as a non-fatal warning, component install failures
  mark the game `Broken`. Existing profiles parse unchanged via
  `#[serde(default)]`. The new signature scores ~125 against MelonLoader
  installs vs plain `onlinefix`'s ~60.

### Fixed

- **Scanner skips version-numbered directories** — directories whose names
  look like bare version strings (e.g. `14.40`, `1.0.2`, `v2.3.1`) are now
  filtered in `is_ignored_entry`. Prevents false positives from game
  version archives like `Fortnite Versions/14.40` — real game roots never
  have pure version-number names.
- **Single-folder scan fallback + explicit Wine errors + Broken on configure
  failure** — `fenrir scan --path /path/to/extracted-game/` now finds the
  game when the regular walk produces zero candidates: the scan root is
  promoted to a candidate as a fallback (random folders with only
  `setup.exe` are still filtered by `is_installer_only`). `wine_for_prefix`
  / `wine_binary` return `Result<PathBuf, String>` instead of silently
  returning a missing `/usr/bin/wine`, so users without system Wine get a
  clear error pointing them at the Runtimes tab. When `apply_profile`
  fails inside `configure_game` after the prefix is already on disk, the
  game is marked `GameStatus::Broken` before returning, so the Library
  shows a recoverable failure state instead of leaving the game in
  `Detected` limbo with an orphan prefix.
- **`version.dll` override added to OnlineFix profile** — modern OnlineFix
  releases (and OnlineFix bundles with mod loaders like MelonLoader, e.g.
  Megabonk + BonkWithFriends) use `version.dll` as the DLL-hijacking entry
  point. Without the native override, Wine loaded its builtin
  `version.dll` and the hook never fired — base steam_api64 spacewar
  fakeapp still worked (overlay + AppId 480), but OnlineFix64.dll
  rich-interface emulation (Friends/Lobby/Invites) and MelonLoader-based
  multiplayer mods stayed unloaded. `version=n,b` is now set alongside
  `winmm=n,b` so both crack vintages and modded variants are covered.

## [0.3.1] - 2026-05-03

Patch release — backend hardening for the v0.3.0 GUI based on a post-release
code review. No frontend or feature changes; behaviour-only fixes.

### Fixed

- **GUI startup no longer panics** on a fresh install — corrupt config falls
  back to defaults with a warning, and `library.db`'s parent directory is
  created automatically.
- **`Configure → Clean files` checkbox now works.** The flag was previously
  discarded; it now invokes the same cleanup pipeline as the CLI's
  `configure --clean`, removing repack residue (`*.url`, `_Redist/`, etc.)
  and recording `cleanup_done` in `user_overrides`.
- **`set_config` and `set_default_runtime` update in-memory state**, so a
  follow-up `get_config` returns the new value instead of the stale one
  cached at app start.
- **Scanner walker runs off the Tauri async runtime** via
  `tokio::task::spawn_blocking`, so a long scan no longer freezes
  `list_games` / `get_game` and other commands.
- **Configured paths are honored consistently**: `prefix_dir` now comes from
  `config.general.prefix_dir` instead of being derived from
  `library_db.parent()`; `runtime_dir` is read from config in `list_runtimes`
  and `install_runtime`; the launch log directory respects the user's data
  dir customisation.
- **Network errors during runtime install** are surfaced rather than masked
  as "release not found" when both proton-ge and wine-ge fetches fail.
- **Progress channel teardown in `install_runtime`** explicitly drops the
  sender before awaiting the events task, removing a latent deadlock if
  `download_runtime` ever held a clone of the callback past return.

### Documentation

- `fenrir-gui/README.md` replaced the default Tauri template with
  Fenrir-specific dev setup instructions, including `tauri:x11` /
  `tauri:x11:wk` fallbacks for Wayland/WebKit issues on NVIDIA and certain
  KDE/Sway configurations.

## [0.3.0] - 2026-04-26

Fase 3 complete. Native desktop GUI built with Tauri v2 + React + TypeScript.

### Added

- **GUI — Tauri v2 frontend** in `fenrir-gui/` — React 18, TypeScript, Vite, Tailwind CSS v3, Zustand.
- **Library view** — game grid with status/store/crack-type badges, detail panel, filter by status.
- **ScanView** — three-phase overlay: path input → progress bar → results with confirm buttons.
- **RuntimeManager** — installed runtimes table, GE-Proton/Wine-GE fetch and install with progress.
- **Tauri commands** — `list_games`, `get_game`, `confirm_game`, `delete_game`, `configure_game`,
  `launch_game`, `scan_directory`, `list_runtimes`, `available_runtimes`, `install_runtime`,
  `set_default_runtime`, `get_config`, `set_config`.
- **Event system** — `configure:step/done`, `launch:started/ended`, `download:progress/done`.
- **Keyboard shortcuts** — `Ctrl+S` opens scan, `Enter` launches or configures selected game.
- **New crack types** — `CrackType::SmokeAPI`, `CrackType::Unsteam` with DB round-trip support.
- **New signatures** — SmokeAPI, unsteam (with crack_type), ColdClientLoader.
- **Scanner fixes** — UE deep scan (markers in Binaries/Win64), `promote_out_of_bin_dir`,
  AnkerGames glob, `max_depth` 4→6, SteamRIP dirname fallback.

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
