# Configuration

Fenrir stores its configuration at:

```
~/.config/fenrir/config.toml
```

If this file doesn't exist, Fenrir creates one with default values on first
run. You can edit it by hand or use `fenrir config --set` from the command line.

## Full Default Config

```toml
[general]
library_db = "/home/you/.local/share/fenrir/library.db"
prefix_dir = "/home/you/.local/share/fenrir/prefixes"
runtime_dir = "/home/you/.local/share/fenrir/runtimes"

[scan]
game_dirs = []
auto_scan = false

[privacy]
fetch_metadata = false
fetch_covers = false
metadata_source = "igdb"

[defaults]
runtime = "auto"
enable_dxvk = true
enable_vkd3d = false
esync = true
fsync = true
```

(Paths shown use a placeholder home directory. Fenrir resolves these using
XDG Base Directory paths on your system.)

## Options Reference

### [general]

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `library_db` | path | `~/.local/share/fenrir/library.db` | Path to the SQLite database file |
| `prefix_dir` | path | `~/.local/share/fenrir/prefixes` | Where game Wine prefixes are created |
| `runtime_dir` | path | `~/.local/share/fenrir/runtimes` | Where downloaded runtimes are stored |

### [scan]

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `game_dirs` | list of paths | `[]` | Directories to scan when running `fenrir scan` without `--path` |
| `auto_scan` | bool | `false` | Reserved for future use |

### [privacy]

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `fetch_metadata` | bool | `false` | Allow fetching game metadata from the internet |
| `fetch_covers` | bool | `false` | Allow fetching cover art from the internet |
| `metadata_source` | string | `"igdb"` | Metadata provider (currently only IGDB planned) |

Fenrir makes zero network connections by default. These options are opt-in and
will be functional in a future release.

### [defaults]

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `runtime` | string | `"auto"` | Default runtime ID, or `"auto"` to pick the first available |
| `enable_dxvk` | bool | `true` | Enable DXVK (DirectX 9/10/11 to Vulkan translation) |
| `enable_vkd3d` | bool | `false` | Enable VKD3D (DirectX 12 to Vulkan translation) |
| `esync` | bool | `true` | Enable Wine esync (eventfd-based synchronization) |
| `fsync` | bool | `true` | Enable Wine fsync (futex-based synchronization) |

## Common Setups

### Games on an external drive

```toml
[scan]
game_dirs = ["/mnt/external/Games/"]
```

### Multiple game directories

```toml
[scan]
game_dirs = ["/mnt/games/", "/home/user/Games/", "/mnt/ssd/SteamLibrary/"]
```

### Custom prefix location (e.g., on a larger disk)

```toml
[general]
prefix_dir = "/mnt/ssd/fenrir-prefixes"
```

Wine prefixes can get large (1-5 GB each depending on the game and DXVK
installation), so putting them on a spacious drive is a good idea.

### Using a specific runtime by default

First, check what's available:

```bash
fenrir runtime list
```

Then set your preferred one:

```bash
fenrir config --set defaults.runtime --value "GE-Proton9-20"
```

Or in `config.toml`:

```toml
[defaults]
runtime = "GE-Proton9-20"
```
