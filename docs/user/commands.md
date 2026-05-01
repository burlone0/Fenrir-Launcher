# Commands Reference

All commands accept `--help` for quick usage info. Game arguments accept either
a UUID or a title (fuzzy-matched).

## Global Flags

These flags work on every command. Pass them before the subcommand name.

| Flag | Short | Description |
|------|-------|-------------|
| `--verbose` | `-v` | Enable verbose output (sets log level to debug/trace) |
| `--quiet` | `-q` | Suppress all output except errors |

```bash
# See exactly what the scanner is doing
fenrir --verbose scan --path /mnt/games/

# Run silently, only print errors
fenrir --quiet launch "Elden Ring"
```

`--verbose` and `--quiet` are mutually exclusive -- if you pass both, quiet
wins. The default log level shows info-level messages from fenrir-core.

## Scanning & Discovery

### fenrir scan

Scan a directory for games.

```
fenrir scan [--path <DIR>]
```

| Option | Description |
|--------|-------------|
| `--path`, `-p` | Directory to scan (overrides configured scan dirs) |

Without `--path`, scans all directories listed in `scan.game_dirs` in your
config. If no directories are configured and no `--path` is given, it tells
you so.

Detected games are added to the library with status `Detected`. Games that
Fenrir isn't sure about (confidence score 30-59) are listed separately for
manual confirmation.

```bash
# Scan a specific directory
fenrir scan --path /mnt/games/

# Scan configured directories
fenrir scan
```

### fenrir list

Show all games in the library.

```
fenrir list
```

Prints a table with ID, title, store origin, status, and crack type. If the
library is empty, it suggests running `fenrir scan`.

### fenrir info

Show detailed information about a game.

```
fenrir info <GAME>
```

| Argument | Description |
|----------|-------------|
| `GAME` | Game title (fuzzy) or UUID |

Prints everything Fenrir knows: title, ID, store, crack type, status,
executable path, install directory, prefix path, assigned runtime, play time,
and last played date.

```bash
fenrir info "Elden Ring"
fenrir info a1b2c3d4-e5f6-7890-abcd-ef1234567890
```

### fenrir add

Manually add a game directory to the library.

```
fenrir add <PATH>
```

| Argument | Description |
|----------|-------------|
| `PATH` | Path to the game directory |

Useful when automatic detection doesn't pick up a game. Fenrir uses the
directory name as the title and tries to find a `.exe` in the top level.
The game is added with status `Detected` and store `Unknown`.

```bash
fenrir add /mnt/games/SomeObscureGame/
```

### fenrir confirm

Confirm a low-confidence game and add it to the library.

```
fenrir confirm <GAME>
```

| Argument | Description |
|----------|-------------|
| `GAME` | Game title or UUID |

When `fenrir scan` finds a game with a confidence score between 30 and 59, it
lists it separately as "needs confirmation" rather than adding it automatically.
Use this command to say "yes, that's a real game, add it."

```bash
fenrir confirm "Some Obscure Game"
```

After confirmation, the game gets status `Detected` and goes through the normal
configure-then-launch flow.

## Configuration & Setup

### fenrir config

View or modify the global configuration.

```
fenrir config [--set <KEY> --value <VALUE>]
```

| Option | Description |
|--------|-------------|
| `--set`, `-s` | Config key to set |
| `--value`, `-v` | Value to set |

Without arguments, prints the full config as TOML. With `--set` and `--value`,
modifies a specific setting and saves.

```bash
# View full config
fenrir config

# Set scan directories (comma-separated for multiple)
fenrir config --set scan.game_dirs --value "/mnt/games/,/home/user/Games/"

# Enable metadata fetching
fenrir config --set privacy.fetch_metadata --value true
```

See [Configuration](configuration.md) for all available keys.

### fenrir configure

Set up a game for launch: create prefix, apply tuning profile.

```
fenrir configure <GAME>
```

| Argument | Description |
|----------|-------------|
| `GAME` | Game title (fuzzy) or UUID |

This does three things:
1. Picks a Wine/Proton runtime (first available, or your configured default)
2. Creates an isolated WINEPREFIX at `~/.local/share/fenrir/prefixes/<game-uuid>/`
3. Applies the tuning profile for the game's crack type (DLL overrides, DXVK, etc.)

After this, the game status changes from `Detected` to `Configured`.

```bash
fenrir configure "Cyberpunk 2077"
```

## Running Games

### fenrir launch

Launch a configured game.

```
fenrir launch <GAME>
```

| Argument | Description |
|----------|-------------|
| `GAME` | Game title (fuzzy) or UUID |

The game must be in `Configured` or `Ready` status. If it's still `Detected`,
Fenrir tells you to run `configure` first.

Fenrir composes the Wine/Proton command with the right environment variables,
launches the game subprocess, pipes stdout/stderr to a log file at
`~/.local/share/fenrir/logs/<game-uuid>.log`, and tracks playtime.

```bash
fenrir launch "Elden Ring"
```

## Runtime Management

### fenrir runtime list

List all detected Wine/Proton runtimes.

```
fenrir runtime list
```

Prints a table with ID, type (Wine/Proton/ProtonGE/WineGE), version, source
(System/Steam/Downloaded), and filesystem path.

Fenrir scans these locations:
- `~/.local/share/fenrir/runtimes/`
- `~/.steam/root/compatibilitytools.d/`
- Steam's `compatibilitytools.d` and `common/` directories
- System Wine (`/usr/bin/wine`, `/usr/share/wine/`)

### fenrir runtime available

Show runtimes available to download from GitHub.

```
fenrir runtime available [--kind proton-ge|wine-ge]
```

| Option | Description |
|--------|-------------|
| `--kind`, `-k` | Which runtime family to list: `proton-ge` (default) or `wine-ge` |

Queries the GloriousEggroll GitHub releases API and prints recent versions with
their download size. Useful for finding the version string to pass to
`runtime install`.

```bash
# List recent GE-Proton releases
fenrir runtime available

# List recent Wine-GE releases
fenrir runtime available --kind wine-ge
```

### fenrir runtime install

Download and install a runtime.

```
fenrir runtime install <VERSION>
```

| Argument | Description |
|----------|-------------|
| `VERSION` | Version to install, e.g. `GE-Proton9-20` |

Downloads the runtime archive from GitHub, verifies the SHA-512 checksum, and
extracts it to `~/.local/share/fenrir/runtimes/`. Shows a progress bar during
download.

```bash
fenrir runtime install GE-Proton9-20
```

After installation, the runtime appears in `runtime list` with source
`Downloaded` and can be set as default with `runtime set-default`.

### fenrir runtime set-default

Set the default runtime for new game configurations.

```
fenrir runtime set-default <ID>
```

| Argument | Description |
|----------|-------------|
| `ID` | Runtime ID (from `runtime list`) |

```bash
fenrir runtime set-default GE-Proton9-20
```
