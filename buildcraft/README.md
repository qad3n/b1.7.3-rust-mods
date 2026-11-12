# BuildCraft

Port of BuildCraft 2.1.1 (Core, Transport, Energy, Factory and Builders) to the b1.7.3_rewrite mod loader as one mod.
One `.bmod` runs on the client and the server. Needs loader API 1.33.

## Assets

The textures and gui images belong to the original authors and are not in this repository or in the `.bmod`. Put your own `BuildCraft Core 2.1.1.zip`,
`BuildCraft Builders 2.1.1.zip`, `BuildCraft Energy 2.1.1.zip` and `BuildCraft Transport 2.1.1.zip` in the game's
`mods/buildcraft_assets/` folder. On its next launch the client builds the mod's art from them, following
`assets.toml`. The names and screen text are the port's own and ship in `assets/buildcraft/lang/en_US.lang`.

## Build

```
./build.sh
```
