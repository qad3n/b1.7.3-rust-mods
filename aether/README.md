# The Aether

Port of Aether Legacy 1.7.10 v1.1.2.5 to the b1.7.3_rewrite mod loader. One `.bmod` runs on the client and the
server.

## Assets

The textures, sounds and lang files are All Rights Reserved by the original authors and are not in this repository or in the `.bmod`. Put your own
`aether-1.7.10-v1.1.2.5.jar` in the game's `mods/aether_assets/` folder. On its next launch the client builds the
mod's art from it, following `assets.toml`.

## Building

```
./build.sh
```

The release build lands in `out/aether-<version>.bmod`. Drop it into the `mods/` folder of the client and the server.

## License

Code is LGPL-3.0 like the original mod. Assets stay with their authors.
