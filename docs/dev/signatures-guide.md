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
