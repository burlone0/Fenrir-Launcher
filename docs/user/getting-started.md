# Getting Started

This walks you through the whole thing: from a folder of game files to
actually playing the game. Five minutes, tops.

## Before You Start

Make sure you have a runtime installed. Without Wine or Proton, `fenrir configure`
has nothing to work with. Check what's available:

```bash
fenrir runtime list
```

If the list is empty, install something:

```bash
fenrir runtime available            # see downloadable versions
fenrir runtime install GE-Proton9-20
```

GE-Proton is the recommended choice. See [Installation](installation.md) for
details on why and how to get Vulkan working if you haven't yet.

## Step 1: Scan Your Games

Point Fenrir at a directory:

```bash
fenrir scan --path /mnt/games/
```

Or set a permanent scan directory and run without `--path` every time:

```bash
fenrir config --set scan.game_dirs --value "/mnt/games/"
fenrir scan
```

Fenrir walks the directory recursively (up to 4 levels deep), looking for
folders that contain `.exe` files. It skips obvious non-game directories like
`_Redist`, `DirectX`, and `Redistributables`.

## Step 2: Review What It Found

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

### Handling low-confidence results

Sometimes Fenrir finds a directory that looks like a game but isn't sure --
this happens when the game has an unusual structure or uncommon crack type.
These show up separately in scan output as "needs confirmation."

If you recognize it as a real game, confirm it:

```bash
fenrir confirm "Some Game"
```

This adds it to the library so you can configure and launch it normally.

## Step 3: Configure the Game

This is where the Wine magic happens:

```bash
fenrir configure "Elden Ring"
```

What Fenrir does behind the scenes:

1. Finds an available Wine or Proton runtime on your system
2. Creates an isolated Wine prefix just for this game (no shared prefixes,
   no contamination between games)
3. Applies a tuning profile based on the detected crack type -- DLL overrides,
   DXVK, esync/fsync, and anything else the crack needs to work properly

You'll see output like:

```
configuring 'Elden Ring'...
  runtime: GE-Proton9-20 (Proton)
  creating prefix at /home/you/.local/share/fenrir/prefixes/a1b2c3d4-...
  applying profile 'onlinefix'...
  done! Run 'fenrir launch "Elden Ring"' to play.
```

### Cleaning up repack artifacts

If the game came from a FitGirl or DODI repack, the game folder might still
contain installer files, `.url` shortcuts, and redistributable directories that
aren't needed at runtime. Pass `--clean` to remove them:

```bash
fenrir configure "Elden Ring" --clean
```

Fenrir will show you what it plans to delete before doing anything. You'll be
asked to confirm.

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

## Something Went Wrong?

If the game crashes, shows a black screen, or doesn't start at all, check
[Troubleshooting](troubleshooting.md) before tearing your hair out. Most
issues have a one-line fix.

The game log is always at:

```
~/.local/share/fenrir/logs/<game-uuid>.log
```

## What's Next

- **[Commands Reference](commands.md)** -- full syntax and examples for every command
- **[Configuration](configuration.md)** -- every config option explained
- **[Troubleshooting](troubleshooting.md)** -- when things don't work as expected
