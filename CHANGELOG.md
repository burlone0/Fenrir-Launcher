# Changelog

All notable changes to Fenrir Launcher will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

Nothing yet.

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
