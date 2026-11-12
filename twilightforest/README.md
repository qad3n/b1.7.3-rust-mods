# The Twilight Forest

Port of The Twilight Forest to the b1.7.3_rewrite mod loader. One `.bmod` runs on the client and the server.

## Assets

The textures, sounds and lang files belong to the original authors and are not in this repository or in the `.bmod`. Put your own
`TwilightForest-2.4.3.jar` in the game's `mods/twilightforest_assets/` folder. On its next launch the client builds the
mod's art from it, following `assets.toml`.

`assets.toml` is generated from the jar: `python3 -I tools/recipe.py <path to TwilightForest-2.4.3.jar>`.
