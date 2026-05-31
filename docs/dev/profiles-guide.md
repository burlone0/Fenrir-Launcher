# Profiles Guide

Profiles are TOML files that tell Fenrir how to configure a Wine prefix for
a specific type of game. Different cracks need different Wine settings --
DLL overrides, environment variables, feature flags. Profiles encode that
knowledge so users don't have to figure it out themselves.

## What Profiles Do

When you run `fenrir configure <game>`, Fenrir:

1. Creates an empty Wine prefix for the game
2. Looks at the game's detected `crack_type`
3. Loads the matching profile from `data/profiles/`
4. Applies the profile's settings to the prefix

The result is a correctly configured Wine prefix, ready to launch.

## Profile Format

Profiles live in `data/profiles/` as `.toml` files. One profile per file.

```toml
[profile]
name = "profile_key"
description = "Human-readable description"

[wine]
windows_version = "win10"
dll_overrides = ["dllname=type", "other=type"]

[env]
SOME_VAR = "value"

[features]
dxvk = true
vkd3d = false
esync = true
fsync = true

[winetricks]
components = ["dotnetdesktop6"]
optional = ["corefonts"]
```

`[winetricks]` is optional. If your profile doesn't need any winetricks
components, leave the section out entirely (or write `[winetricks]` with
nothing under it).

### Sections

#### [profile]

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Unique identifier. Used to match profiles to crack types. |
| `description` | string | Human-readable description. |

#### [wine]

| Field | Type | Description |
|-------|------|-------------|
| `windows_version` | string | Windows version to emulate. Usually `"win10"`. |
| `dll_overrides` | list of strings | DLL overrides in `"name=type"` format. |

**DLL override types:**
- `n` -- native: use the DLL from the game directory
- `b` -- builtin: use Wine's built-in DLL
- `n,b` -- try native first, fall back to builtin

For cracked games, you almost always want `n` (native) for the crack's DLLs.
This tells Wine "use the DLL the crack shipped, not your own version."

#### [env]

Key-value pairs for environment variables injected at launch time. Leave
empty (`[env]` with nothing below it) if no extra vars are needed.

#### [features]

| Field | Type | Description |
|-------|------|-------------|
| `dxvk` | bool | Enable DXVK (DX9/10/11 -> Vulkan). |
| `vkd3d` | bool | Enable VKD3D (DX12 -> Vulkan). |
| `esync` | bool | Enable eventfd-based synchronization. |
| `fsync` | bool | Enable futex-based synchronization. |

**DXVK** translates DirectX 9, 10, and 11 calls to Vulkan. This is almost
always a win on Linux -- better performance, fewer rendering glitches.

**VKD3D** does the same for DirectX 12. Only needed for DX12 games.

**esync/fsync** are Wine synchronization improvements. esync uses eventfd
(widely supported), fsync uses futex (needs kernel >= 5.16 or so, but faster).
You can enable both -- Wine picks the best available.

#### [winetricks]

Some games need runtime libraries that aren't part of a base Wine prefix --
.NET runtimes, Visual C++ redistributables, Microsoft fonts, and so on.
Listing them here makes Fenrir install them with `winetricks -q <name>` during
`configure`, between prefix creation and DLL override application.

| Field | Type | Description |
|-------|------|-------------|
| `components` | list of strings | Mandatory. If any fail to install, the game is marked `Broken`. |
| `optional` | list of strings | Best-effort. If install fails (no network, package outdated, etc.) configure continues with a warning. |

Component names are passed verbatim to `winetricks -q <name>`. Run
`winetricks list-all` for valid identifiers -- common ones include
`dotnetdesktop6`, `dotnet48`, `vcrun2019`, `vcrun2022`, `corefonts`,
`d3dcompiler_47`.

Winetricks must be installed on the user's system. If it isn't, Fenrir emits a
non-fatal warning during configure rather than crashing -- the user can
install winetricks and re-run configure without losing state. See the
[Megabonk example](#onlinefix_melonloader) below for a worked case.

Notes:

- Component installs are **idempotent**. Subsequent configures detect installed
  components and skip them.
- First-time installs of large components (`dotnetdesktop6`, `vcrun2022`) can
  take 5-10 minutes. The GUI emits `configure:step` progress events during
  install so the user knows something is happening.
- Don't list components for things Wine already handles. DXVK and VKD3D are
  enabled via `[features]`, not winetricks.

## Profile-to-Game Matching

Fenrir maps crack types to profile names with a simple lookup:

| CrackType | Profile name |
|-----------|-------------|
| `OnlineFix` | `onlinefix` |
| `OnlineFixMelonLoader` | `onlinefix_melonloader` |
| `DODI` | `dodi` |
| `FitGirl` | `fitgirl` |
| `Scene` | `scene` |
| `GOGRip` | `gog` |
| Everything else | `steam_generic` |

The profile `name` field must match the expected profile name. If no matching
profile is found, Fenrir uses defaults from the global config.

## Creating a New Profile

### Step 1: Understand what the game type needs

Figure out what Wine configuration this type of game requires. Common
questions:
- Does the crack ship custom DLLs that need native overrides?
- Does it need specific environment variables?
- Is it DX11 (DXVK) or DX12 (VKD3D)?
- Are there known compatibility issues with esync/fsync?

### Step 2: Create the profile file

Create a new `.toml` file in `data/profiles/`:

```toml
[profile]
name = "your_profile_name"
description = "Profile for YourType games"

[wine]
windows_version = "win10"
dll_overrides = ["relevant_dll=n"]

[env]
# Add environment variables if needed

[features]
dxvk = true
vkd3d = false
esync = true
fsync = true
```

### Step 3: Wire it up

The profile-to-crack-type mapping lives in
`crates/fenrir-cli/src/commands/configure.rs` in the
`crack_type_to_profile_name()` function. Add your mapping there:

```rust
fn crack_type_to_profile_name(
    crack_type: Option<fenrir_core::library::game::CrackType>,
) -> &'static str {
    use fenrir_core::library::game::CrackType;
    match crack_type {
        Some(CrackType::OnlineFix) => "onlinefix",
        Some(CrackType::DODI)     => "dodi",
        Some(CrackType::FitGirl)  => "fitgirl",
        Some(CrackType::Scene)    => "scene",
        Some(CrackType::GOGRip)   => "gog",
        Some(CrackType::YourType) => "your_profile_name",  // add this
        _                         => "steam_generic",
    }
}
```

### Step 4: Test

```bash
cargo test --all
```

Then test manually with a game of that type:

```bash
fenrir configure "Some Game Of That Type"
```

## Existing Profiles Explained

### steam_generic

The fallback profile. Applied to any Steam crack that doesn't match a more
specific type.

- `dll_overrides`: `steam_api=n`, `steam_api64=n` -- use the crack's Steam API
  DLLs, not Wine's stubs
- `dxvk`: enabled -- most games benefit from Vulkan translation
- `vkd3d`: disabled -- most cracked games are DX11, not DX12
- `esync/fsync`: both enabled -- let Wine pick the best sync method

### onlinefix

For OnlineFix cracks that enable LAN/online multiplayer via Steam emulation.

- `dll_overrides`: adds `steamclient=n`, `steamclient64=n` on top of the
  standard Steam API overrides. OnlineFix ships custom steamclient DLLs for
  its multiplayer emulation.
- `version=n,b`: modern OnlineFix loader hook -- the DLL-hijacking entry
  point that reads `dlllist.txt` and pulls in `OnlineFix64.dll` +
  `SteamOverlay64.dll`. Without this, Wine uses its builtin `version.dll`
  and the multiplayer patch never loads.
- `winmm=n,b`: legacy OnlineFix loader hook for older crack vintages.
  Both kept to cover both eras.
- `OPENSSL_ia32cap`: disables an AVX CPU instruction that causes crashes in
  some OnlineFix configurations.
- Everything else: same as steam_generic.

### onlinefix_melonloader

For OnlineFix games that ship MelonLoader and a mod providing the actual
multiplayer functionality. The trigger case is **Megabonk + BonkWithFriends**,
but the same pattern applies to any OnlineFix game where the multiplayer
features live in a MelonLoader mod instead of base OnlineFix.

What's different from plain `onlinefix`:

- Same DLL override set (both crack vintages covered).
- Adds `[winetricks]` with `dotnetdesktop6` as a mandatory component.
  MelonLoader 0.6+ needs the .NET 6 Desktop Runtime to bootstrap
  `MelonLoader.NativeHost.dll`. Without it, mods silently fail to load and
  the game runs as if no mods existed -- overlay and Spacewar AppId still
  work via base OnlineFix `steam_api64`, but invites/lobbies/rich-presence
  via the modded multiplayer are dead.
- `corefonts` in `optional` -- improves the legibility of the MelonLoader
  console window on Wine setups without bundled MS fonts.

Full profile (`data/profiles/onlinefix_melonloader.toml`):

```toml
[profile]
name = "onlinefix_melonloader"
description = "OnlineFix with MelonLoader mod-based multiplayer (.NET 6 required)"

[wine]
windows_version = "win10"
dll_overrides = [
    "steam_api=n,b", "steam_api64=n,b",
    "steamclient=n,b", "steamclient64=n,b",
    "OnlineFix64=n,b", "SteamOverlay64=n,b",
    "version=n,b", "winmm=n,b",
    "winhttp=n,b", "dnet=n",
]

[env]
OPENSSL_ia32cap = "~0x20000000"

[features]
dxvk = true
vkd3d = false
esync = true
fsync = true

[winetricks]
components = ["dotnetdesktop6"]
optional = ["corefonts"]
```

This profile is selected when the scanner detects both `OnlineFix.ini` and a
`MelonLoader/` directory in the game root -- see the
[Signatures Guide](signatures-guide.md#modded-crack-pattern) for how the
"modded crack" detection layer works.

### dodi

For DODI repacks. After installation the game directory is a standard Steam
crack, so the profile mirrors steam_generic exactly: `steam_api=n` and
`steam_api64=n`, DXVK on, esync/fsync on. DODI-specific files (`_Redist/`,
`DODI Repacks/`) are not present at runtime so they don't affect Wine setup.

### fitgirl

For FitGirl repacks. Same situation as DODI -- post-install the game is a
standard Steam crack. The FitGirl-specific marker file (`fitgirl-repacks.site`)
and setup executables are installer artifacts, not runtime artifacts. Profile
is steam_generic-equivalent.

### scene

For Scene releases (CODEX, PLAZA, EMPRESS, generic `.nfo` releases). Scene
cracks ship a patched `steam_api.dll`, so `steam_api=n` and `steam_api64=n`
are both set. No store is assigned since scene releases aren't tied to a
storefront.

### gog

For GOG games and rips. GOG ships DRM-free so there's no Steam API to deal with
-- `dll_overrides` is empty. Galaxy DRM (`GalaxyClient.dll`) is handled
transparently by Wine's translation layer. Everything else (DXVK, esync/fsync)
is the same as any other profile.

### dodi

For DODI repacks. After installation the game directory is a standard Steam
crack, so the profile mirrors steam_generic exactly: `steam_api=n` and
`steam_api64=n`, DXVK on, esync/fsync on. DODI-specific files (`_Redist/`,
`DODI Repacks/`) are not present at runtime so they don't affect Wine setup.

### fitgirl

For FitGirl repacks. Same situation as DODI -- post-install the game is a
standard Steam crack. The FitGirl-specific marker file (`fitgirl-repacks.site`)
and setup executables are installer artifacts, not runtime artifacts. Profile
is steam_generic-equivalent.

### scene

For Scene releases (CODEX, PLAZA, EMPRESS, generic `.nfo` releases). Scene
cracks ship a patched `steam_api.dll`, so `steam_api=n` and `steam_api64=n`
are both set. No store is assigned since scene releases aren't tied to a
storefront.

### gog

For GOG games and rips. GOG ships DRM-free so there's no Steam API to deal with
-- `dll_overrides` is empty. Galaxy DRM (`GalaxyClient.dll`) is handled
transparently by Wine's translation layer. Everything else (DXVK, esync/fsync)
is the same as any other profile.

## User Overrides

Players can override profile settings per game through the `user_overrides`
field in the game database. These are stored as JSON and take priority over
profile defaults. The override chain is:

```
Profile defaults -> User overrides -> Final configuration
```

User overrides always win.
