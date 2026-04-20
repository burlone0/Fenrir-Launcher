# Frequently Asked Questions

## General

### What distros are supported?

Fenrir is tested on Arch, Fedora, and Ubuntu. It should work on any Linux
distro that runs Wine, which is pretty much all of them. If something breaks on
your distro, open an issue.

### Does it work with native Linux games?

No. Fenrir is specifically for Windows games running through Wine or Proton.
Native Linux games don't need Wine configuration, so there's nothing for Fenrir
to do.

### What's the difference between Fenrir and Lutris or Heroic?

Lutris and Heroic are general-purpose launchers that handle downloading games,
store integration, and lots of edge cases for a wide variety of games.

Fenrir does one thing: configure and launch games that are already on your
disk. It's smaller, faster, completely offline by default, and designed for
people who already have games and just want them to work. If you need a
launcher that downloads from GOG or interfaces with the Epic store, use Heroic.

### Is Fenrir production-ready?

It's v0.2.0. The core pipeline (scan, configure, launch) works reliably. The
scanner has known rough edges with games in unusual directory structures. The
GUI doesn't exist yet. Use it, report issues, manage expectations.

---

## Detection

### Fenrir found my game but classified it wrong. What now?

Use `fenrir info "Game Name"` to see what it detected. If the crack type or
store is wrong, you can still configure and launch the game -- the wrong
classification might just mean a slightly suboptimal Wine profile.

If you want to fix it properly, [open an issue](https://github.com/burlone0/Fenrir-Launcher/issues)
or improve the detection signatures yourself. See the
[Signatures Guide](../dev/signatures-guide.md).

### A game isn't being detected at all. What can I do?

Run with verbose output:

```bash
fenrir --verbose scan --path /path/to/that/game/
```

Look for why the candidate is being scored as 0 or discarded. The most common
reasons are:

- The game's `.exe` is more than 4 directory levels deep
- The game doesn't match any known signature (no `steam_api.dll`,
  `goggame-*.info`, `EOSSDK-*.dll`, etc.)

If the game has none of the standard signature files, add it manually:

```bash
fenrir add /path/to/game/
```

Then configure and launch it normally.

### Does Fenrir support GOG and Epic games?

Yes, as of v0.2.0. Fenrir detects:
- GOG games via `goggame-*.info`, `GalaxyClient.dll`, or `game.id`
- Epic games via `EOSSDK-Win64-Shipping.dll` or `EpicGamesLauncher.lnk`

If your GOG or Epic game isn't being detected, see the answer above.

---

## Wine and Runtimes

### Can I use Steam's Proton?

Yes. If Steam is installed, Fenrir scans its compatibility tools directory
(`~/.steam/root/compatibilitytools.d/`) and picks up any Proton versions
there automatically. They appear in `fenrir runtime list`.

### What runtime should I use?

[GE-Proton](https://github.com/GloriousEggroll/proton-ge-custom) for most
games. It includes patches for game-specific issues and generally has better
compatibility than stock Wine or vanilla Proton.

For simple or older games (pre-2010, minimal DRM), system Wine is often
enough and uses less disk space.

### Why isolated Wine prefixes instead of one shared prefix?

A single shared prefix means a DLL override for one game can break another.
A corrupted prefix takes down everything. Debugging is a nightmare because you
don't know which game caused the state that broke yours.

Isolated prefixes are slightly wasteful with disk space (1-5 GB each), but
that's a worthwhile trade. Modern drives have the space. Modern humans don't
have the patience for shared prefix debugging.

### Can I reuse an existing Wine prefix?

Not directly through Fenrir. Fenrir creates and manages its own prefixes. If
you have a working prefix from Lutris or a manual Wine setup, the cleanest path
is to let Fenrir create a fresh one -- the profiles it applies are usually
sufficient to get the game running without manual prefix tuning.

---

## Privacy and Network

### Does Fenrir phone home?

No. By default, Fenrir makes zero network connections. The only time it touches
the network is when you explicitly run `fenrir runtime available` or
`fenrir runtime install`, which query GitHub's releases API.

All privacy-related options (`fetch_metadata`, `fetch_covers`) are `false` by
default and will remain so.

### Does Fenrir track my playtime anywhere outside my machine?

No. Playtime is stored locally in `~/.local/share/fenrir/library.db` and
nowhere else.

---

## Legal

### Is Fenrir legal to use?

Fenrir is a launcher. Running Wine to execute a Windows binary is legal in
virtually every jurisdiction. Fenrir doesn't download, distribute, or unlock
any software.

What Fenrir detects (FitGirl repacks, scene releases, OnlineFix cracks) are
common sources of pirated software. That's a fact worth naming honestly.
Fenrir identifies these release types because they require specific Wine
configuration -- not to endorse them. Whether obtaining software this way
complies with your local laws or the software's license is your responsibility,
not Fenrir's.

### Why does Fenrir explicitly name crack types like FitGirl and OnlineFix?

Because Wine configuration depends on it. A game that uses OnlineFix's custom
steamclient DLL needs different overrides than a vanilla Steam crack. Fenrir
needs to know the release type to apply the right profile. The names are
technical classifiers, not endorsements.

---

## Contributing

### I want to add support for a game type I have. Where do I start?

The [Signatures Guide](../dev/signatures-guide.md) and
[Profiles Guide](../dev/profiles-guide.md). Both use TOML files -- no Rust
required.

### I found a bug. Where do I report it?

[GitHub Issues](https://github.com/burlone0/Fenrir-Launcher/issues). Include
your distro, kernel, Wine/Proton version, and the output of
`fenrir --verbose <failing-command>`.
