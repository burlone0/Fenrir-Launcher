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
```

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

## Profile-to-Game Matching

Fenrir maps crack types to profile names with a simple lookup:

| CrackType | Profile name |
|-----------|-------------|
| `OnlineFix` | `onlinefix` |
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

Currently, the profile-to-crack-type mapping is in
`crates/fenrir-cli/src/commands/configure.rs` in the
`crack_type_to_profile_name()` function. Add your mapping there:

```rust
fn crack_type_to_profile_name(
    crack_type: Option<fenrir_core::library::game::CrackType>,
) -> &'static str {
    use fenrir_core::library::game::CrackType;
    match crack_type {
        Some(CrackType::OnlineFix) => "onlinefix",
        Some(CrackType::YourType) => "your_profile_name",  // add this
        _ => "steam_generic",
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
- `OPENSSL_ia32cap`: disables an AVX CPU instruction that causes crashes in
  some OnlineFix configurations
- Everything else: same as steam_generic

## User Overrides

Players can override profile settings per game through the `user_overrides`
field in the game database. These are stored as JSON and take priority over
profile defaults. The override chain is:

```
Profile defaults -> User overrides -> Final configuration
```

User overrides always win.
