# Signatures Guide

This guide explains how Fenrir's game detection system works and how to add
new signatures. If you've ever looked at a game folder and thought "I know
what this is just by the files in it" -- that's exactly what signatures
formalize.

## How Detection Works

When you run `fenrir scan`, this happens:

1. **Directory walk** -- Fenrir recursively walks the target directory (max
   depth 4), looking for subdirectories that contain at least one `.exe` file.
   Directories like `_Redist`, `DirectX`, and `Redistributables` are skipped
   automatically.

2. **Candidate collection** -- Each directory with an `.exe` becomes a
   `GameCandidate`. If a directory has multiple `.exe` files, they're all
   recorded (Fenrir currently uses the first one as the launch executable).

3. **Signature matching** -- Every candidate is tested against every loaded
   signature. For each signature, Fenrir checks whether the required files
   exist in the candidate directory. If they do, it adds up a confidence
   score.

4. **Classification** -- The highest-scoring signature wins. If the score is
   >= 60, the game is auto-added with high confidence. Between 30 and 59, it's
   flagged for user confirmation. Below 30, it's ignored.

## Signature Format

Signatures are TOML files in `data/signatures/`. Currently there are three:
`steam.toml`, `gog.toml`, and `epic.toml`. Each file can contain multiple
signatures as top-level sections. Here's the anatomy:

```toml
[section_key]
name = "Human-readable name"          # Required. Displayed in scan output.
store = "Steam"                       # Optional. Sets StoreOrigin (Steam/GOG/Epic).
crack_type = "OnlineFix"              # Optional. Sets CrackType.
required_files = ["file.dll"]         # Required. ALL must exist or score is 0.
optional_files = ["other.dll"]        # Optional. Each match adds to score.
confidence_boost = ["bonus_dir/"]     # Optional. Each match adds to score.
```

### Fields

**name** (required) -- Display name for the signature. Shown in scan output
and debug logs.

**store** (optional) -- The store this game came from. Maps to the `StoreOrigin`
enum: `"Steam"`, `"GOG"`, `"Epic"`. If omitted, defaults to `Unknown`.

**crack_type** (optional) -- The crack/repack type. Maps to the `CrackType`
enum: `"OnlineFix"`, `"DODI"`, `"FitGirl"`, `"Scene"`, `"GOGRip"`. If omitted,
the game is detected but no specific crack type is assigned.

**required_files** (required) -- List of files or directories that MUST exist
in the game directory. If any are missing, the entire signature scores 0.
This is the critical filter.

**optional_files** (optional) -- Files that are commonly present but not
guaranteed. Each match adds points.

**confidence_boost** (optional) -- Extra files that increase confidence when
present. Lower weight than optional_files.

### File Pattern Syntax

- `"filename.ext"` -- Exact file match. Case-insensitive fallback (so
  `"steam_api.dll"` matches `STEAM_API.DLL`).
- `"dirname/"` -- Directory match. The trailing slash is required.
- `"*.ext"` -- Glob pattern. Matches any file with that extension.

## Confidence Scoring

| Match type | Points |
|------------|--------|
| Each `required_files` match | +30 |
| Each `optional_files` match | +15 |
| Each `confidence_boost` match | +10 |

**Important:** If ANY required file is missing, the score is 0. Required files
are an all-or-nothing gate.

| Score range | Result |
|-------------|--------|
| >= 60 | High confidence -- auto-added to library |
| 30-59 | Needs confirmation -- listed separately |
| < 30 | Ignored |

### Scoring examples

**Steam generic (all files present):**
- `steam_api.dll` (required): +30
- `steam_api64.dll` (optional): +15
- `steam_appid.txt` (optional): +15
- Total: 60 -> high confidence

**Steam generic (minimal):**
- `steam_api.dll` (required): +30
- Total: 30 -> needs confirmation

**OnlineFix (typical):**
- `OnlineFix.url` (required): +30
- `OnlineFix64.dll` (optional): +15
- `steam_settings/` (boost): +10
- Total: 55 -> needs confirmation (would need one more match for auto-add)

## Writing a New Signature

### Step 1: Analyze the game directory

Look at actual game directories for the type you want to detect. What files
are always there? What files are usually there? What files are specific to
this type and not others?

For example, if you're adding a GOG game signature:
```
MyGOGGame/
  goggame-1234567890.info    <- always present, unique to GOG
  goglog/                    <- usually present
  gog.ico                    <- sometimes present
  game.exe
```

### Step 2: Write the signature

Create a new section in an existing file, or create a new `.toml` file under
`data/signatures/`:

```toml
[gog]
name = "GOG"
store = "GOG"
required_files = ["goggame-*.info"]
confidence_boost = ["goglog/", "gog.ico"]
```

Things to think about:
- **Be specific with required_files** -- they should be files that only this
  type of game has. `steam_api.dll` is a good required file for Steam cracks
  because non-Steam games don't have it.
- **Don't over-require** -- if a file is only present 80% of the time, make
  it optional, not required. A missing required file means score = 0.
- **Use boost for weak signals** -- files that slightly suggest this type but
  aren't definitive.

### Step 3: Test it

Run a scan against a directory you know contains games of this type:

```bash
RUST_LOG=debug fenrir scan --path /path/to/your/games/
```

The `debug` log level shows per-candidate scoring. Look for your signature
name and check the scores make sense. You want:
- Games of this type to score >= 60 (or at least >= 30 for confirmation)
- Games of other types to score 0 against your signature

### Step 4: Run the test suite

Make sure you haven't broken anything:

```bash
cargo test --all
```

## Existing Signatures Walkthrough

Signatures are split across three files: `steam.toml` for Steam-based sources,
`gog.toml` for GOG, and `epic.toml` for Epic Games Store.

### steam.toml

**steam_generic** -- The broadest catch-all. Any game with `steam_api.dll` is
probably a Steam crack. Optional files (`steam_api64.dll`, `steam_appid.txt`)
are present in most but not all. Boost files (`steam_emu.ini`, `cream_api.ini`)
indicate specific crack tools.

**onlinefix** -- OnlineFix always drops an `OnlineFix.url` shortcut. The DLL
(`OnlineFix64.dll`) and `steam_settings/` directory are strong secondary signals.

**onlinefix_melonloader** -- A more specific variant of `onlinefix` for games
that ship MelonLoader on top of an OnlineFix crack (e.g. Megabonk +
BonkWithFriends). Requires both `OnlineFix.ini` and a `MelonLoader/`
directory, scoring ~125 against MelonLoader-installed OnlineFix games vs the
~60 of plain `onlinefix`. Maps to a dedicated profile that adds
`dotnetdesktop6` via winetricks. See the
[Modded Crack Pattern](#modded-crack-pattern) section below for the design
rationale.

**fitgirl** -- FitGirl leaves a `fitgirl-repacks.site` marker file. After
installation, the game looks like a standard Steam crack (hence `steam_api.dll`
as a boost).

**dodi** -- DODI creates a `DODI Repacks/` directory. Like FitGirl, the actual
game is a standard Steam crack underneath.

**scene** -- Scene releases always have an `.nfo` file. They sometimes split
across disc directories (`cd1/`, `cd2/`). No store is assumed since scene
releases aren't tied to a specific storefront.

### gog.toml

**gog_info** -- The most reliable GOG signal. GOG installers write a
`goggame-<appid>.info` file to the install directory for every game. The glob
`goggame-*.info` catches all of them. `goglog/` and `gog.ico` are common but
not universal, so they're optional rather than required.

**gog_galaxy** -- For games installed through the GOG Galaxy launcher. Galaxy
drops `GalaxyClient.dll` in the game directory. This is a fairly reliable
required file -- non-GOG games don't ship it. `Galaxy64.dll` is a boost for
extra confidence.

**gog_installer** -- For games installed from GOG's offline installers. The
installer writes a plain `game.id` file to the install root. The `start.sh`
script and `gameinfo` file are optional helpers that GOG installs include.

### epic.toml

**epic_emu** -- All Epic Games Store titles bundle the Epic Online Services
SDK as `EOSSDK-Win64-Shipping.dll`. This is a strong required file -- if it's
there, it's almost certainly an EGS game. ScreamAPI (a DLC/entitlement
unlocker) replaces the same DLL and ships `ScreamAPI.dll` or `ScreamAPI64.dll`
alongside it, so those are confidence boosts.

**epic_generic** -- A fallback for games launched via the EGS launcher that
leave an `EpicGamesLauncher.lnk` shortcut in the game directory. Less specific
than epic_emu but catches titles that don't bundle EOSSDK directly.

## Modded Crack Pattern

Sometimes a base crack type ships with an additional mod loader (MelonLoader,
BepInEx, UnityModManager, etc.) that requires extra Wine prefix setup --
typically a .NET runtime via winetricks. The base crack profile works for the
unmodded game, but the modded variant needs more.

The pattern is: **layer a more specific signature on top of the base, with a
dedicated crack type that maps to a dedicated profile.**

### How it works

The scanner picks the *highest-scoring* signature per candidate. If both the
base and the modded signature match, the modded one wins as long as it scores
higher. That's the lever we use.

Take OnlineFix + MelonLoader as the worked example:

| Signature | Required files | Typical score |
|-----------|----------------|---------------|
| `onlinefix` (base) | `OnlineFix.ini` | ~60 |
| `onlinefix_melonloader` (modded) | `OnlineFix.ini`, `MelonLoader/` | ~125 |

When a game ships both, the modded signature scores ~2x the base because it
has two required files (60 points from required) plus more optional/boost
matches that only modded games have (`Mods/`, `UserLibs/`,
`MelonLoader/net6/`). When a plain OnlineFix game scans, only the base
matches because `MelonLoader/` is missing, so the modded signature scores 0.

### When to use this pattern

- A base crack type already has a working profile.
- A subset of games using that crack ship an additional component (mod
  loader, runtime, framework) that needs setup the base profile can't
  provide.
- The additional component leaves a recognizable filesystem footprint -- a
  directory, a marker file, a DLL name.

If the modded variant just needs a tweak that the base profile could handle
unconditionally, don't fork. Just add it to the base. The modded pattern is
for cases where the extra setup would *hurt* unmodded games (e.g. installing
.NET 6 unnecessarily, taking 10 minutes, when 95% of games don't need it).

### How to write a modded signature

1. **Add a new variant to `CrackType`** in
   `crates/fenrir-core/src/library/game.rs`. Don't reuse the base variant --
   the whole point is to map to a different profile.

2. **Write the signature** in the same TOML file as the base. Include the
   base's required files plus the modded marker(s):

   ```toml
   [base_modded_variant]
   name = "Base + Modded"
   store = "Steam"
   crack_type = "BaseModdedVariant"
   auto_add_threshold = 30
   required_files = ["base_marker.ini", "ModLoader/"]
   optional_files = ["base_optional.dll", "Mods/", "Plugins/"]
   confidence_boost = ["ModLoader/runtime/", "config.json"]
   cleanup_files = ["base_marker.url", "_Redist/", "setup.exe"]
   ```

3. **Verify the scoring**. Run `fenrir --verbose scan` against a directory
   containing both a base-only and a modded version of the same game. The
   modded version should classify as the modded variant; the unmodded one
   should classify as the base.

4. **Write a dedicated profile** in `data/profiles/` that includes whatever
   extra setup the modded variant needs. For the `[winetricks]` section
   specifically, see the
   [Profiles Guide](profiles-guide.md#winetricks).

5. **Wire the crack type to the profile** in
   `crates/fenrir-cli/src/commands/configure.rs` via
   `crack_type_to_profile_name()`.

### Caveats

- **Both signatures must be present in the same file.** They're not
  hierarchical -- the scanner doesn't know the modded variant is "a
  subclass" of the base. It just compares scores. As long as the modded
  variant scores higher when both match, you're fine.
- **Don't make the modded variant's required files too narrow.** If the
  modder ships variants of the mod loader (e.g. MelonLoader 0.5 vs 0.6 in
  different directories), match the *common* path.
- **Cleanup files should be the same** as the base unless the modded variant
  ships extra installer junk that's different.
