# Getting Started

This walks you through the whole thing: from a folder of game files to
actually playing the game. Five minutes, tops.

## Step 1: Tell Fenrir Where Your Games Are

Fenrir needs to know where to look. Point it at a directory:

```bash
fenrir scan --path /mnt/games/
```

Or, if you want to set a permanent scan directory (so you don't need `--path`
every time):

```bash
fenrir config --set scan.game_dirs --value "/mnt/games/"
fenrir scan
```

Fenrir walks the directory recursively (up to 4 levels deep), looking for
folders that contain `.exe` files. It skips obvious non-game directories like
`_Redist`, `DirectX`, and `Redistributables`.

## Step 2: See What It Found

```bash
fenrir list
```

You'll get a table like this:

```
ID                                   TITLE                          STORE    STATUS     CRACK
a1b2c3d4-e5f6-...                    Elden Ring                     Steam    Detected   OnlineFix
f7e8d9c0-b1a2-...                    Cyberpunk 2077                 Steam    Detected   FitGirl
...
```

**Status: Detected** means Fenrir found the game but hasn't set up a Wine
prefix for it yet. That's the next step.

For more details on a specific game:

```bash
fenrir info "Elden Ring"
```

This shows the full breakdown: install directory, detected executable, crack
type, assigned runtime, play time, everything.

## Step 3: Configure the Game

This is where the Wine magic happens:

```bash
fenrir configure "Elden Ring"
```

What Fenrir does behind the scenes:

1. Finds an available Wine or Proton runtime on your system
2. Creates an isolated Wine prefix just for this game (no shared prefixes,
   no contamination between games)
3. Applies a tuning profile based on the detected crack type -- setting DLL
   overrides, enabling DXVK, configuring esync/fsync, and anything else the
   crack needs to work properly

You'll see output like:

```
configuring 'Elden Ring'...
  runtime: GE-Proton9-20 (Proton)
  creating prefix at /home/you/.local/share/fenrir/prefixes/a1b2c3d4-...
  applying profile 'onlinefix'...
  done! Run 'fenrir launch "Elden Ring"' to play.
```

## Step 4: Launch

```bash
fenrir launch "Elden Ring"
```

Fenrir composes the Wine/Proton command with all the right environment
variables, launches the game, and tracks the process. When you're done
playing, it logs the session time and exit code.

```
launching 'Elden Ring'...
game exited (code: Some(0), played: 47m)
```

## What's Next

- **[Commands Reference](commands.md)** -- all 8 CLI commands with full
  syntax and examples
- **[Configuration](configuration.md)** -- every config option explained,
  with common setup examples
