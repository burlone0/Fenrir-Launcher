# Architecture

## Overview

Fenrir is a Cargo workspace with two crates:

```
crates/
  fenrir-core/    -- library: all core logic, zero CLI concerns
  fenrir-cli/     -- binary: thin clap wrapper that calls fenrir-core
```

The split is intentional. `fenrir-core` is a library that can be consumed by
any frontend -- the CLI today, a Tauri GUI in Fase 3, or anything else. The CLI
doesn't contain business logic; it parses arguments, calls core functions, and
prints results.

## Module Map

All modules live under `crates/fenrir-core/src/`. Each module has a `mod.rs`
(or is a single file) that re-exports its public API.

### config

**Purpose:** Load and save application settings.

**Key types:**
- `FenrirConfig` -- top-level config struct with sections for general, scan,
  privacy, and defaults
- `GeneralConfig`, `ScanConfig`, `PrivacyConfig`, `DefaultsConfig` -- section
  structs

**How it works:** Config is a TOML file at `~/.config/fenrir/config.toml`
(XDG). If the file doesn't exist, `FenrirConfig::default()` provides sensible
defaults. `load()` reads and parses, `save()` writes back.

**Depends on:** nothing (leaf module)

### library

**Purpose:** Game data model and persistent storage.

**Key types:**
- `Game` -- the central data struct (UUID, title, exe path, install dir, store,
  crack type, prefix path, runtime, status, playtime, timestamps, user overrides)
- `StoreOrigin` -- enum: Steam, GOG, Epic, Unknown
- `CrackType` -- enum: OnlineFix, DODI, FitGirl, Scene, GOGRip, Unknown
- `GameStatus` -- enum: Detected, Configured, Ready, Broken
- `Database` -- SQLite handle with CRUD operations and fuzzy search

**How it works:** `Database::open()` opens (or creates) the SQLite file and runs
migrations. Games are inserted after scanning, updated after configuration and
launch. `find_by_title()` does case-insensitive substring matching.

**Depends on:** nothing (leaf module)

### scanner

**Purpose:** Find games on disk and classify them.

**Key types:**
- `GameCandidate` -- a directory that contains at least one `.exe`
- `Signature` -- a TOML-defined detection pattern (name, store, crack_type,
  required/optional/boost files)
- `ClassifiedGame` -- a candidate that matched a signature (includes title,
  store, crack type, confidence score)
- `ScanResult` -- output of a scan: high confidence games, needs-confirmation
  games, total candidate count

**How it works:** Three-stage pipeline:
1. `detector::find_game_candidates()` -- recursive directory walk (max depth 4),
   finds directories containing `.exe` files, skips known non-game dirs
2. `signatures::load_signatures_from_dir()` -- loads TOML patterns from
   `data/signatures/`
3. `classifier::classify_candidate()` -- scores each candidate against all
   signatures, picks the highest-scoring match

**Depends on:** library (for StoreOrigin, CrackType enums)

### runtime

**Purpose:** Discover installed runtimes and download new ones from GitHub.

The runtime module has four files:

- `types.rs` -- data structures (`Runtime`, `RuntimeType`, `RuntimeSource`)
- `discovery.rs` -- filesystem scan (`discover_all()`)
- `github.rs` -- GitHub Releases API client
- `downloader.rs` -- archive download, checksum verification, extraction

**Key types:**
- `Runtime` -- a discovered runtime (ID, type, version, path, source)
- `RuntimeType` -- enum: Wine, Proton, ProtonGE, WineGE
- `RuntimeSource` -- enum: System, Steam, Downloaded
- `GitHubRelease` -- a release entry from the GloriousEggroll API
  (`tag_name`, list of `GitHubAsset`)
- `GitHubAsset` -- a single release file (`name`, `browser_download_url`,
  `size`)
- `ProgressCallback` -- `Arc<dyn Fn(u64, u64) + Send + Sync>` -- called
  periodically during download with `(bytes_received, total_bytes)`;
  used to drive the CLI progress bar

**How it works:**

Discovery (`discovery.rs`) scans a prioritized list of filesystem paths:
1. `~/.local/share/fenrir/runtimes/` (Fenrir-managed)
2. `~/.steam/root/compatibilitytools.d/` (GE-Proton)
3. Steam's common/ directory (Valve Proton)
4. System Wine (`/usr/bin/wine`, `/usr/share/wine/`)

Each discovered runtime gets a version parsed from directory name or binary
output.

GitHub client (`github.rs`) calls the GloriousEggroll releases API to list
available versions. The repo constants (`PROTON_GE_REPO`, `WINE_GE_REPO`) are
the only hard-coded references to an external service in the entire codebase.

Downloader (`downloader.rs`) handles the full install sequence:
1. Fetch the asset URL from `GitHubRelease`
2. Stream the download, calling the progress callback every chunk
3. Verify the SHA-512 checksum against the `.sha512sum` companion file
4. Extract the tarball to `~/.local/share/fenrir/runtimes/`

If the checksum fails, the partial download is deleted and an error is returned.

**Depends on:** nothing (leaf module)

### prefix

**Purpose:** Create and configure Wine prefixes.

**Key types:**
- `WineProfile` -- a TOML-defined tuning configuration (DLL overrides, env vars,
  DXVK/VKD3D toggles, esync/fsync, Windows version)
- `ProfileMeta`, `WineConfig`, `FeatureConfig` -- profile sub-structs

**How it works:**
1. `builder::create_prefix()` -- creates the directory, runs `wineboot --init`
2. `profile::load_profiles_from_dir()` -- loads TOML profiles from
   `data/profiles/`
3. `tuner::apply_profile()` -- applies DLL overrides via Wine registry, sets
   environment variables
4. `builder::build_wine_env()` -- composes the base env vars (WINEPREFIX,
   WINEDEBUG, WINEESYNC, WINEFSYNC)

**Depends on:** nothing (leaf module)

### launcher

**Purpose:** Build and execute Wine/Proton commands, monitor the running game.

**Key types:**
- `LaunchConfig` -- everything needed to launch: exe, wine binary, prefix path,
  env vars, is_proton flag, proton path
- `PreparedCommand` -- the fully composed command (program, args, env,
  working dir)
- `ProcessResult` -- outcome of a monitored process (exit code, play time in
  seconds)

**How it works:**
1. `runner::build_launch_command()` -- composes either a Wine command
   (`wine game.exe`) or a Proton command (`proton run game.exe`) with the
   appropriate environment variables
2. `runner::launch()` -- spawns the subprocess with piped stdout/stderr
3. `monitor::monitor_process()` -- waits for exit, logs output to file,
   calculates play time

**Depends on:** nothing (leaf module)

### error

**Purpose:** Centralized error types.

**Key types:**
- `FenrirError` -- top-level enum, wraps all module-specific errors
- `ConfigError`, `DatabaseError`, `ScannerError`, `RuntimeError`, `PrefixError`,
  `LauncherError` -- module-level error enums

All implemented with `thiserror` for ergonomic `?` propagation.

## Data Flow

The life of a game, from directory on disk to running process:

```
1. SCAN       User directory
              |
              v
2. DETECT     find_game_candidates() -> Vec<GameCandidate>
              |
              v
3. CLASSIFY   classify_candidate() -> ClassifiedGame (with confidence score)
              |
              v
4. STORE      Database::insert_game() -> Game { status: Detected }
              |
              v
5. CONFIGURE  create_prefix() + apply_profile() -> Game { status: Configured }
              |
              v
6. LAUNCH     build_launch_command() + launch() + monitor_process()
              |
              v
7. UPDATE     Update play_time, last_played, status in database
```

## Storage

**Why SQLite:** Embedded, zero setup, no daemon, file-level locking is fine for
a single-user CLI app. Gives us SQL queries (fuzzy search, filtering) without
the overhead of a server database.

**Schema:** Single `games` table. All fields from the `Game` struct map to
columns. `user_overrides` is stored as JSON text. UUIDs are stored as text.
Timestamps are ISO 8601 text.

**Location:** `~/.local/share/fenrir/library.db` (configurable)

## Design Decisions

### Why signature-based detection instead of hardcoded patterns?

Signatures live in TOML files under `data/signatures/`. New crack types can be
added without recompiling -- just drop a `.toml` file. This also makes community
contributions easier: you don't need to know Rust to add support for a new
game type.

### Why isolated prefixes instead of a shared one?

One WINEPREFIX per game means zero contamination. A DLL override for one game
can't break another. A corrupted prefix affects only one game. The downside
is disk space (1-5 GB per prefix), but disk is cheap and debugging shared
prefix issues is not.

### Why TOML for config and data files?

TOML is human-readable, hand-editable, and the de facto standard in the Rust
ecosystem. YAML has footguns (Norway problem, implicit typing). JSON doesn't
support comments. TOML is the obvious choice.

### Why library-first architecture?

Separating fenrir-core from fenrir-cli means the Tauri GUI (Fase 3) can import
the same library without duplicating logic. It also makes the core testable
in isolation -- no CLI parsing in the test path.
