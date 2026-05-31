# MelonLoader Games

Some OnlineFix-cracked games ship with a separate mod loader called
**MelonLoader** that provides the actual multiplayer functionality. Megabonk
+ BonkWithFriends is the canonical example: the game itself runs fine
without any mod, but invites, lobbies, and rich-presence live in a mod that
MelonLoader has to load.

If you've configured one of these games with Fenrir and the game launches but
multiplayer features quietly don't work, you're in the right place.

## How to tell if your game uses MelonLoader

Look in the game's install directory for:

- A `MelonLoader/` folder
- A `Mods/` folder (often with `.dll` files inside)
- A `version.dll` in the root next to the game's `.exe`

If you have all three, the game uses MelonLoader. Fenrir should detect this
automatically and classify it with crack type `OnlineFixMelonLoader`. Verify
with:

```bash
fenrir info "Your Game Name"
```

If the `crack_type` field shows `OnlineFix` instead of `OnlineFixMelonLoader`,
the scanner missed the MelonLoader directory. Re-run scan and confirm the
game. If it still misclassifies, the MelonLoader folder might be nested
deeper than expected -- open an issue with the game's directory structure.

## What Fenrir does for these games

When you run `fenrir configure "Game Name"` for a MelonLoader game:

1. Creates the Wine prefix as usual
2. **Installs `dotnetdesktop6` (the .NET 6 Desktop Runtime) via winetricks**
   inside the prefix -- MelonLoader needs this to bootstrap
   `MelonLoader.NativeHost.dll`
3. Applies the `onlinefix_melonloader` profile (DLL overrides including
   `version.dll`, environment variables, esync/fsync)
4. Optionally installs `corefonts` for better MelonLoader console
   readability

The .NET 6 install takes **5-10 minutes** on the first configure. The GUI
shows progress messages; the CLI prints `installing component
dotnetdesktop6...`. Don't kill it -- subsequent configures of any game
detect the already-installed runtime and skip this step.

## You need winetricks installed

Winetricks is a separate tool that Fenrir uses to install runtime
dependencies into Wine prefixes. It's not bundled with Fenrir.

Check if you have it:

```bash
which winetricks
```

If the command returns a path, you're good. If not, install it:

```bash
# Ubuntu / Debian
sudo apt install winetricks

# Fedora
sudo dnf install winetricks

# Arch
sudo pacman -S winetricks
```

If you ran `fenrir configure` *before* installing winetricks, you'll see a
warning like:

```
warn: winetricks not found in PATH; .NET 6 Desktop Runtime not installed
hint: install winetricks (sudo apt install winetricks / sudo pacman -S winetricks) and re-run configure
```

Install winetricks and re-run `fenrir configure "Game Name"`. Fenrir won't
lose any state -- it picks up where it left off.

## Common problems

### "Game launches but multiplayer mods don't load"

The most common symptom. The game runs because base OnlineFix
`steam_api64.dll` is working (Steam overlay shows AppId 480 / Spacewar), but
invites and lobbies don't work because MelonLoader didn't bootstrap.

Causes, in order of likelihood:

1. **No .NET 6 Desktop Runtime in the prefix.** Either winetricks wasn't
   installed when you configured, or the install failed silently. Re-run
   `fenrir configure "Game Name"` -- Fenrir will retry the winetricks step.

2. **Game was classified as plain OnlineFix.** Check `fenrir info` -- if
   `crack_type` is `OnlineFix` (not `OnlineFixMelonLoader`), the modded
   profile wasn't applied. Re-scan with verbose output to see why the
   `MelonLoader/` directory wasn't matched:
   ```bash
   fenrir --verbose scan --path /path/to/game/
   ```

3. **`version.dll` not in the game directory.** MelonLoader installs itself
   via DLL hijacking through `version.dll`. If the game directory doesn't
   have one, MelonLoader isn't installed correctly -- the game was extracted
   without its mod files, or only the base OnlineFix release was applied.

### "Configure fails: cannot install component dotnetdesktop6"

The winetricks install of .NET 6 failed. The game is marked `Broken`.

Reasons this happens:

- **No network.** winetricks downloads .NET 6 from Microsoft's servers.
  Check your connection.
- **winetricks version is outdated.** Old winetricks versions have stale URLs
  for .NET installers. Update winetricks:
  ```bash
  # Update to the latest from upstream (works on any distro)
  sudo sh -c "wget -O /usr/local/bin/winetricks https://raw.githubusercontent.com/Winetricks/winetricks/master/src/winetricks && chmod +x /usr/local/bin/winetricks"
  ```
  Then re-run `fenrir configure`.
- **Wine prefix architecture mismatch.** .NET 6 Desktop is 64-bit only. If
  your prefix was accidentally created as 32-bit, .NET won't install. Delete
  the prefix and re-configure:
  ```bash
  rm -rf ~/.local/share/fenrir/prefixes/<game-uuid>
  fenrir configure "Game Name"
  ```

### "First configure takes 10 minutes and looks frozen"

It's not frozen. .NET 6 Desktop Runtime is ~75 MB and includes a full
runtime installer with its own setup wizard that runs in the background.

The CLI doesn't show real-time progress because winetricks runs in
non-interactive mode. The GUI emits `configure:step` events but the
individual winetricks steps don't surface granular progress.

If it's been more than 15 minutes, check if winetricks is still running:

```bash
pgrep -fa winetricks
```

If there's no winetricks process and `fenrir configure` is still running,
something hung. Cancel with Ctrl+C and check
`~/.cache/winetricks/dotnetdesktop6/` for partial downloads.

## Verifying MelonLoader loaded correctly

After a successful configure and launch, the game's log file should mention
MelonLoader bootstrapping:

```bash
tail -100 ~/.local/share/fenrir/logs/<game-uuid>.log | grep -i melonloader
```

Look for lines like:
```
[MelonLoader] Loading <mod-name>.dll
[MelonLoader] Mod loaded successfully
```

If you see those, the mod-based multiplayer should work. If the log only
shows base OnlineFix init and no MelonLoader output, something in the chain
broke -- check the sections above.

## Reporting issues

If your MelonLoader game still doesn't work after going through this guide,
open an issue with:

- The game's name and where you got it from
- Output of `fenrir info "Your Game Name"`
- Output of `fenrir --verbose configure "Your Game Name" 2>&1 | tail -100`
- Output of the `grep -i melonloader` on the game log above
- The list of files in the game's root directory (top level only):
  ```bash
  ls /path/to/game/
  ```
