# b1.7.3 Rust mods

Mods ported to the b1.7.3-rewrite mod loader. Each folder is a project of its own: its own `Cargo.toml`, `mod.toml`,
sources, tools and status notes. One `.b173mod` per folder runs on both the client and the server.

| Folder | Mod | Ported from |
|---|---|---|
| `aether/` | The Aether (Aether Legacy) | 1.7.10 v1.1.2.5 |

## Building a mod

Each mod builds with `cargo-b173` from the modding SDK:

```
cargo b173 build --path aether
```

The result lands in `<mod>/target/<id>-<version>.b173mod`. Drop it into the `mods/` folder of the client and the server.

## Assets

Textures, sounds and lang files belong to the original authors and are not in this repository. Each mod extracts them
from your own copy of the original jar, for example:

```
cd aether
python3 -I tools/extract_assets.py <path to aether-1.7.10-v1.1.2.5.jar>
```

## Licenses

Each mod keeps the license of the mod it was ported from; see its `mod.toml`.
