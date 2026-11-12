# IndustrialCraft 2

Port of IndustrialCraft 2 v1.00 for beta 1.7.3 to the b1.7.3_rewrite mod loader. One `.bmod` runs on the client
and the server. Needs loader API 1.33.

## Assets

The textures, gui images and armor layers belong to the original authors and are not in this repository or in the `.bmod`. Put your own
`IndustrialCraft2 v1.00.zip` in the game's `mods/ic2_assets/` folder. On its next launch the client builds the mod's
art from it, following `assets.toml`. The names and screen text are the port's own and ship in
`assets/ic2/lang/en_US.lang`. The zip carries no sounds; IC2 played them from a separate resource download, so a
texture pack may add them under `assets/ic2/sounds/`.

## Build

```
./build.sh
```
