# Changelog

All notable changes to Fenrir Launcher will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

Nothing yet.

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
