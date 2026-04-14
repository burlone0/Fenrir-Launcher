# Adding a New Store

Fenrir currently detects games from Steam-based sources (cracks, repacks, scene
releases). Multi-store support (GOG, Epic) is planned for Fase 4. This guide
documents the current extension points and what a new store integration needs.

## Current State

The codebase already has the scaffolding for multiple stores:

- `StoreOrigin` enum in `crates/fenrir-core/src/library/game.rs` defines
  `Steam`, `GOG`, `Epic`, `Unknown`
- Signatures can set `store = "GOG"` or `store = "Epic"` (the classifier
  parses these)
- The database stores and filters by `store_origin`

What's missing is store-specific detection signatures and any store-specific
launch behavior.

## What a New Store Needs

### 1. Detection signatures

Every store has characteristic files. For example:

**GOG:**
```toml
[gog]
name = "GOG"
store = "GOG"
required_files = ["goggame-*.info"]
confidence_boost = ["goglog/", "gog.ico"]
```

**Epic (with crack):**
```toml
[epic_generic]
name = "Epic Generic"
store = "Epic"
required_files = ["EOSSDK-Win64-Shipping.dll"]
optional_files = ["EasyAntiCheat/"]
```

These go in `data/signatures/` -- either in the existing `steam.toml` (renamed
to something more general) or in new store-specific files like `gog.toml`,
`epic.toml`.

### 2. Tuning profiles (if needed)

GOG games typically don't need Steam API DLL overrides. They might need their
own profile:

```toml
[profile]
name = "gog"
description = "Profile for GOG games"

[wine]
windows_version = "win10"
dll_overrides = []

[env]

[features]
dxvk = true
vkd3d = false
esync = true
fsync = true
```

### 3. Profile mapping

Update the crack-type-to-profile mapping in the CLI (or move it to the core
library) to handle the new store's games.

### 4. Discovery paths (optional)

Some stores have well-known install locations. If GOG Galaxy or Heroic Launcher
are installed, their game directories could be auto-discovered. This would go
in the scanner module as additional scan paths.

## What Doesn't Change

The core pipeline (scan -> classify -> store -> configure -> launch) doesn't
change. A GOG game goes through the exact same flow as a Steam game. The
differences are:
- Which signatures match
- Which profile gets applied
- Potentially different discovery paths

This is by design. The architecture handles multi-store through data (signatures
and profiles), not code branches.

## Roadmap

This work is tracked in the Fase 4 implementation plan. The rough sequence:

1. Add GOG/Epic signatures to `data/signatures/`
2. Add corresponding profiles to `data/profiles/`
3. Test detection against real game directories
4. Add store-specific discovery paths (if useful)
5. Update the profile mapping logic

Contributions are welcome -- if you have GOG or Epic games and want to help
build detection signatures, see the [Signatures Guide](signatures-guide.md)
and open a PR.
