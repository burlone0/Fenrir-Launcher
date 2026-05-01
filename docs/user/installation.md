# Installation

## Requirements

| Requirement | Minimum | Recommended |
|-------------|---------|-------------|
| OS | Any Linux distro | Arch, Fedora, or Ubuntu (tested) |
| Kernel | 5.4 | 5.16+ (for fsync support) |
| Rust | Stable channel | Latest stable |
| Wine or Proton | Any version | GE-Proton (latest) |
| GPU | Any with OpenGL | Vulkan-capable (for DXVK) |
| RAM | 4 GB | 8 GB+ |

Fenrir itself is tiny. The RAM and GPU requirements come from the games you'll
be running through it.

## Installing Wine

You need at least one Wine-compatible runtime. The system Wine from your
package manager works for getting started:

```bash
# Ubuntu/Debian
sudo apt install wine

# Fedora
sudo dnf install wine

# Arch (btw)
sudo pacman -S wine
```

For serious gaming, system Wine is often not enough. See
[Getting a Better Runtime](#getting-a-better-runtime) below.

## Building Fenrir

```bash
git clone https://github.com/burlone0/Fenrir-Launcher.git
cd Fenrir-Launcher
cargo build --release
```

The binary lands at `target/release/fenrir-cli`. Copy it somewhere in your
`$PATH`:

```bash
cp target/release/fenrir-cli ~/.local/bin/fenrir
```

Or run it in place as `./target/release/fenrir-cli`.

## First Run

The first time you run any Fenrir command, it creates its config and data
directories following the XDG Base Directory spec:

```
~/.config/fenrir/
  config.toml          -- your settings (scan dirs, defaults, privacy)

~/.local/share/fenrir/
  library.db           -- SQLite database of your games
  prefixes/            -- isolated Wine prefixes (one per game)
  runtimes/            -- downloaded Wine/Proton runtimes
  logs/                -- per-game stdout/stderr logs
```

If the config file doesn't exist, Fenrir uses sensible defaults. You don't
need to set anything up manually before your first scan.

## Verifying the Install

```bash
fenrir config
```

This prints your current configuration. If you see a TOML block with sections
like `[general]`, `[scan]`, `[privacy]`, and `[defaults]`, you're good to go.

## Getting a Better Runtime

System Wine works, but [GE-Proton](https://github.com/GloriousEggroll/proton-ge-custom)
is significantly better for modern games. It includes patches and fixes that
upstream Proton doesn't ship yet. Fenrir can install it for you:

```bash
# See what's available
fenrir runtime available

# Install a specific version
fenrir runtime install GE-Proton9-20

# Make it the default
fenrir runtime set-default GE-Proton9-20
```

If you prefer to install manually, extract the GE-Proton archive to either:
- `~/.local/share/fenrir/runtimes/` (Fenrir-managed)
- `~/.steam/root/compatibilitytools.d/` (shared with Steam)

Fenrir scans both locations and picks it up automatically.

## Vulkan and DXVK

Fenrir enables DXVK by default for all games. DXVK translates DirectX 9/10/11
calls to Vulkan, which gives better performance and fewer rendering glitches
on Linux.

For DXVK to work, you need:
- A GPU that supports Vulkan (nearly all GPUs released since 2015 do)
- The Vulkan driver for your GPU:
  ```bash
  # NVIDIA (proprietary driver)
  sudo apt install libvulkan1 vulkan-tools   # Ubuntu
  sudo pacman -S vulkan-icd-loader           # Arch

  # AMD (open source -- usually included)
  sudo apt install mesa-vulkan-drivers       # Ubuntu
  sudo pacman -S vulkan-radeon               # Arch

  # Intel (open source -- usually included)
  sudo apt install intel-media-va-driver     # Ubuntu
  sudo pacman -S vulkan-intel                # Arch
  ```

If DXVK isn't working (black screen, missing textures), check the game log at
`~/.local/share/fenrir/logs/<uuid>.log` for Vulkan errors. You can also
disable DXVK per-game via user overrides, or globally in config:

```toml
[defaults]
enable_dxvk = false
```

## Troubleshooting the Install

If something goes wrong, see [Troubleshooting](troubleshooting.md). The most
common install-time issues are a missing Vulkan driver (game crashes
immediately on launch) and a Wine binary that isn't in `$PATH` (Fenrir reports
no runtimes found).

## What's Next

Head to [Getting Started](getting-started.md) to scan your first game.
