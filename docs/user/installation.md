# Installation

## What You Need

- **Linux** -- any recent distro should work. Tested on Arch, Fedora, Ubuntu.
- **Rust toolchain** -- stable channel. Install via [rustup](https://rustup.rs/)
  if you don't have it.
- **Wine or Proton** -- at least one must be installed. Fenrir doesn't ship its
  own (yet -- that's coming in a future release).

If you're running an Arch-based distro, chances are you already have Wine. On
Ubuntu/Fedora, grab it from your package manager:

```bash
# Ubuntu/Debian
sudo apt install wine

# Fedora
sudo dnf install wine

# Arch (btw)
sudo pacman -S wine
```

For better compatibility with modern games, consider
[GE-Proton](https://github.com/GloriousEggroll/proton-ge-custom) -- it
includes patches that upstream Proton and Wine don't. Extract it to
`~/.local/share/fenrir/runtimes/` or `~/.steam/root/compatibilitytools.d/`
and Fenrir will find it automatically.

## Building from Source

```bash
git clone https://github.com/burlone0/Fenrir-Launcher.git
cd Fenrir-Launcher
cargo build --release
```

The binary lands at `target/release/fenrir-cli`. You can copy it somewhere
in your `$PATH` or run it directly:

```bash
# Option 1: run in place
./target/release/fenrir-cli --help

# Option 2: copy to a directory in PATH
cp target/release/fenrir-cli ~/.local/bin/fenrir
fenrir --help
```

## First Run

The first time you run any Fenrir command, it creates its config and data
directories following the XDG Base Directory spec:

```
~/.config/fenrir/
  config.toml          -- your settings (scan dirs, defaults, privacy)

~/.local/share/fenrir/
  library.db           -- SQLite database of your games
  prefixes/            -- isolated Wine prefixes (one per game)
  runtimes/            -- downloaded Wine/Proton runtimes (Fase 2)
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

Head to [Getting Started](getting-started.md) to scan your first game.
