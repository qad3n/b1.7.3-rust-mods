# Roughly Enough Items

Small port of Roughly Enough Items (Fabric 16.0.799) to the b1.7.3_rewrite mod loader: item list, search, recipe pages
and cheat mode. One `.bmod` runs on the client and the server.

## Assets

The lang strings belong to the original authors and are not in this repository. Supply them from your own copy of the
original mod jar before building:

```
python3 -I tools/extract_assets.py <path to RoughlyEnoughItems-16.0.799-fabric.jar>
```

This writes `assets/rei/lang/en_US.lang`, which stays untracked.

## Building

```
./build.sh
```

The release build lands in `out/rei-<version>.bmod`. Drop it into the `mods/` folder of the client and the server.

## License

MIT like the original mod.
