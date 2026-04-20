# Troubleshooting

Most problems fall into one of these categories. Check the relevant section
and look at the game's log file for details:

```bash
cat ~/.local/share/fenrir/logs/<game-uuid>.log
```

You can find the UUID with `fenrir info "Game Name"`.

For more verbose output during any operation:

```bash
fenrir --verbose <command>
```

---

## Fenrir doesn't find any games

**Scan returns 0 results or misses games you know are there.**

Things to check:

- Make sure you're pointing at the right directory. The path must contain game
  folders as direct or nested subdirectories (up to 4 levels deep).
- The game folder must contain at least one `.exe` file. If all the executables
  are in a deeply nested subdirectory (more than 4 levels down), Fenrir won't
  find them.
- Directories named `_Redist`, `DirectX`, `Redistributables`, `CommonRedist`,
  and similar are skipped automatically. If your game folder has an unusual
  name that looks like one of these, it'll be ignored.

Run with verbose output to see exactly what's being scanned:

```bash
fenrir --verbose scan --path /your/games/
```

---

## Game is detected but ends up in "needs confirmation"

**Fenrir found the game but didn't add it automatically.**

Fenrir's confidence score for that game is between 30 and 59 -- it's fairly
sure it's a game, but not certain enough to add it without asking. This usually
happens with games that have unusual directory structures or don't match any
known crack type.

If it's a real game, just confirm it:

```bash
fenrir confirm "Game Name"
```

If Fenrir consistently misclassifies a game type you have a lot of, consider
[contributing a signature](../dev/signatures-guide.md) for it.

---

## Configure fails: "no runtime found"

**`fenrir configure` exits with an error about no available runtime.**

Fenrir can't find Wine or Proton anywhere. Fix options:

1. Install system Wine: `sudo apt install wine` (Ubuntu) or equivalent
2. Install GE-Proton via Fenrir:
   ```bash
   fenrir runtime available
   fenrir runtime install GE-Proton9-20
   ```
3. If you have Wine installed but Fenrir doesn't see it, check that the Wine
   binary is in your `$PATH`:
   ```bash
   which wine
   ```
   If it's somewhere non-standard (e.g., `/opt/wine/bin/wine`), Fenrir won't
   find it via system discovery. For now, copy or symlink it into a standard
   location.

---

## Game crashes immediately on launch

**Game process starts but exits with a non-zero code right away.**

Check the log first:

```bash
cat ~/.local/share/fenrir/logs/<uuid>.log
```

Common causes:

**Missing Vulkan / DXVK error:**
Log contains `DXVK: No Vulkan adapter found` or similar. Your GPU doesn't
support Vulkan or the driver isn't installed. See
[Installation](installation.md#vulkan-and-dxvk). As a workaround, disable
DXVK:
```bash
fenrir config --set defaults.enable_dxvk --value false
fenrir configure "Game Name"   # reconfigure with DXVK off
```

**DLL not found:**
Log contains `err:module:import_dll Library steam_api.dll not found`. The
crack's DLL overrides aren't working. Try reconfiguring:
```bash
fenrir configure "Game Name"
```
If the problem persists, the game might need a profile that hasn't been written
yet. Check the [Profiles Guide](../dev/profiles-guide.md) and consider
contributing one.

**Prefix corruption:**
Sometimes a prefix gets into a bad state. Recreate it:
```bash
fenrir configure "Game Name"
```
Fenrir detects an already-configured game and asks if you want to reconfigure.
Confirm yes.

---

## Black screen on launch

**Game process is running (no crash) but the screen is black.**

This is almost always a DXVK or rendering issue.

1. Check if the game uses DirectX 12 -- if so, try enabling VKD3D:
   ```bash
   fenrir config --set defaults.enable_vkd3d --value true
   fenrir configure "Game Name"
   ```

2. If the game uses DirectX 11 or older and DXVK is enabled, try disabling
   DXVK as a test to see if it's the cause:
   ```bash
   fenrir config --set defaults.enable_dxvk --value false
   fenrir configure "Game Name"
   ```

3. Check Wine version -- older Wine handles some games better. Try switching to
   a different runtime:
   ```bash
   fenrir runtime list
   fenrir runtime set-default <other-runtime-id>
   fenrir configure "Game Name"
   ```

---

## No audio

**Game runs but there's no sound.**

This is usually a Wine audio configuration issue, not a Fenrir issue. Things
to try:

- Make sure PulseAudio or PipeWire is running
- Check that `winecfg` (inside the game's prefix) has the right audio driver
  selected: `WINEPREFIX=~/.local/share/fenrir/prefixes/<uuid> winecfg`
- Some games need `PULSE_LATENCY_MSEC` set. You can add it via user overrides
  in the database, or wait for per-game environment variable support in a
  future release

---

## Poor performance

**Game runs but is slow or stutters.**

1. Make sure GE-Proton is in use -- it has Fsync, better shader compilation
   handling, and other performance improvements over stock Wine.
2. Make sure DXVK is enabled (`fenrir config` should show `enable_dxvk = true`).
3. Enable esync and fsync if they're off:
   ```bash
   fenrir config --set defaults.esync --value true
   fenrir config --set defaults.fsync --value true
   fenrir configure "Game Name"
   ```
4. Check your kernel version -- fsync requires kernel 5.16+.
5. On the first few launches, DXVK compiles shaders on-demand, which causes
   stuttering. This is normal. It gets better after a few sessions.

---

## Runtime download fails

**`fenrir runtime install` fails mid-download or with a checksum error.**

- Checksum mismatch: the downloaded file is corrupt. Delete the partial file
  in `~/.local/share/fenrir/runtimes/` and try again.
- Network timeout: try again. GitHub's releases CDN is generally reliable but
  sometimes hiccups.
- If it consistently fails, check if `https://github.com` is reachable from
  your machine and that you're not behind a proxy that strips TLS.

---

## Still stuck?

Run the operation with `--verbose` and open a GitHub issue with the output.
Include:
- Your distro and kernel version
- Wine/Proton runtime version (`fenrir runtime list`)
- The full verbose output
- The game log from `~/.local/share/fenrir/logs/`
