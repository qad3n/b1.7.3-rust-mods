# b1.7.3 Rust mods

Mods ported to the b1.7.3_rewrite mod loader. Each folder is a project of its own: its own `Cargo.toml`, `mod.toml`,
sources, tools and status notes. One `.bmod` per folder runs on both the client and the server.

| Folder | Mod | Ported from |
|---|---|---|
| `aether/` | The Aether (Aether Legacy) | 1.7.10 v1.1.2.5 |
| `spc/` | Single Player Commands (modern, minimal redesign) | b1.7.3 v2.11.1 |
| `rei/` | Roughly Enough Items (item list, recipes and cheat mode only) | Fabric 16.0.799 |
| `twilightforest/` | The Twilight Forest | 1.7.10 2.4.3 |

## Building a mod

Each mod has a `build.sh` that runs `cargo-b173` from the modding SDK, either from `PATH` or from the SDK vendored in
the rewrite checkout next to this one (override with `B173_SDK=<sdk dir>`):

```
aether/build.sh
```

The release build lands in `<mod>/out/<id>-<version>.bmod`, which holds only the latest build of that mod. Drop it
into the `mods/` folder of the client and the server.

`build-all.sh` in the root runs every mod's `build.sh`, each into its own `out/`:

```
./build-all.sh
```

## Assets

Mods made here commit their assets with the source.

Ported mods do not: their textures, sounds and lang files belong to the original authors and never go in a `.bmod`.
Each ported mod ships an `assets.toml` recipe instead, and its `README.md` names the download a player puts in the
game's `mods/<modid>_assets/` folder. The client converts that download once and serves it below the mod's own
assets. Text the port wrote itself, such as names a mod set in code, ships in the mod's own `lang/en_US.lang`.

Every mod uses the modern resource pack layout only: `assets/<modid>/textures/<path>.png` with one file per sprite
(an animated sprite is a strip of square frames with a `.png.mcmeta` beside it), `assets/<modid>/sounds/<path>.ogg`
(discs under `sounds/record/`) and `assets/<modid>/lang/en_US.lang`. Old grid sheets are split into sprites by
`[[slice]]` steps in the port's `assets.toml`. The loader serves each mod's `assets/` as a texture pack layer above the beta jar,
so a player's texture pack retextures a mod or replaces its sounds and lang keys at the same paths.

## Licenses

Each mod keeps the license of the mod it was ported from; see its `mod.toml`.
