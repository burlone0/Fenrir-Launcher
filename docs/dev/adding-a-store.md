# Adding a New Store

Fenrir can detect games from Steam-based sources, GOG, and Epic Games Store.
This guide explains how that detection is structured and what you need to add
support for a new store.

## What's Already There

As of v0.2.0, three store families are fully wired up:

- **Steam** -- cracks, repacks, and scene releases (`data/signatures/steam.toml`)
- **GOG** -- GOG installer/Galaxy client/offline installs (`data/signatures/gog.toml`)
- **Epic** -- EGS titles with EOSSDK, ScreamAPI cracked (`data/signatures/epic.toml`)

The library model (`StoreOrigin` enum) already has `Steam`, `GOG`, `Epic`, and
`Unknown`. Adding a new store is purely a matter of adding data -- no changes to
the core pipeline are needed.

## What a New Store Needs

### 1. Detection signatures

Every store has characteristic files. Figure out what's always there, then
write a signature in `data/signatures/`:

```toml
# data/signatures/mystore.toml

[mystore_generic]
name = "MyStore"
store = "MyStore"
crack_type = "MyStoreCrack"    # or omit if DRM-free
required_files = ["mystore-sdk.dll"]
optional_files = ["mystore_settings/"]
confidence_boost = ["gameinfo.json"]
```

See the [Signatures Guide](signatures-guide.md) for the full format reference
and scoring rules.

### 2. Extend StoreOrigin and CrackType

Add the new variants to both enums in
`crates/fenrir-core/src/library/game.rs`:

```rust
pub enum StoreOrigin {
    Steam,
    GOG,
    Epic,
    MyStore,   // add this
    Unknown,
}

pub enum CrackType {
    OnlineFix,
    DODI,
    FitGirl,
    Scene,
    GOGRip,
    MyStoreCrack,  // add this
    Unknown,
}
```

The classifier in `scanner/classifier.rs` reads the `store` and `crack_type`
fields from signatures and parses them into these enums, so the string you put
in the TOML must match the enum variant name.

### 3. Tuning profile (if needed)

Some stores need different Wine settings. GOG games don't need Steam API DLL
overrides; Epic games don't either. If your store has specific requirements,
create a profile in `data/profiles/`:

```toml
# data/profiles/mystore.toml

[profile]
name = "mystore"
description = "Profile for MyStore games"

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

See the [Profiles Guide](profiles-guide.md) for the full format reference.

### 4. Profile mapping

Wire the new crack type to its profile in
`crates/fenrir-cli/src/commands/configure.rs`:

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
        Some(CrackType::MyStoreCrack) => "mystore",  // add this
        _                         => "steam_generic",
    }
}
```

### 5. Discovery paths (optional)

Some stores install to well-known locations. If the store has a launcher with
a fixed install root, you can add it to the default scan paths in config.
This is optional -- users can always point `fenrir scan --path` at the right
directory.

## What Doesn't Change

The core pipeline (scan -> classify -> store -> configure -> launch) is
store-agnostic. A GOG game goes through the exact same flow as a Steam crack.
The differences are entirely in:
- Which signatures match
- Which profile gets applied
- Potentially, where to look during discovery

This is by design. Multi-store support is a data problem, not a code problem.
