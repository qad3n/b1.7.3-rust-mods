# Roughly Enough Items

Small port of Roughly Enough Items (Fabric 16.0.799) to the b1.7.3_rewrite mod loader: item list, search, recipe pages
and cheat mode. One `.bmod` runs on the client and the server.

## Assets

The lang strings belong to the original authors and are not in this repository or in the `.bmod`. Put your own
`RoughlyEnoughItems-16.0.799-fabric.jar` in the game's `mods/rei_assets/` folder. On its next launch the client copies
the strings the port shows out of it, following `assets.toml`.

## Building

```
./build.sh
```

The release build lands in `out/rei-<version>.bmod`. Drop it into the `mods/` folder of the client and the server.

## License

MIT like the original mod.
