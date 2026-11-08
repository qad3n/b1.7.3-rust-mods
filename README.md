# b1.7.3 Rust mods

Mods ported to the b1.7.3_rewrite mod loader. Each folder is a project of its own: its own `Cargo.toml`, `mod.toml`,
sources, tools and status notes. One `.b173mod` per folder runs on both the client and the server.

| Folder | Mod | Ported from |
|---|---|---|
| `aether/` | The Aether (Aether Legacy) | 1.7.10 v1.1.2.5 |
| `spc/` | Single Player Commands (modern, minimal redesign) | b1.7.3 v2.11.1 |
| `rei/` | Roughly Enough Items (item list, recipes and cheat mode only) | Fabric 16.0.799 |

## Building a mod

Each mod has a `build.sh` that runs `cargo-b173` from the modding SDK, either from `PATH` or from the SDK vendored in
the rewrite checkout next to this one (override with `B173_SDK=<sdk dir>`):

```
aether/build.sh
```

The release build lands in `<mod>/out/<id>-<version>.b173mod`, which holds only the latest build of that mod. Drop it
into the `mods/` folder of the client and the server.

`build-all.sh` in the root runs every mod's `build.sh`, each into its own `out/`:

```
./build-all.sh
```

## Assets

Textures, sounds and lang files belong to the original authors and are not in this repository. Each mod extracts them
from your own copy of the original jar, for example:

```
cd aether
python3 -I tools/extract_assets.py <path to aether-1.7.10-v1.1.2.5.jar>
```

## Licenses

Each mod keeps the license of the mod it was ported from; see its `mod.toml`.
