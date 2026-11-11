# The Aether

Port of Aether Legacy 1.7.10 v1.1.2.5 to the b1.7.3_rewrite mod loader. One `.bmod` runs on the client and the
server.

## Assets

The textures, sounds and lang files are All Rights Reserved by the original authors and are not in this repository.
Supply them from your own copy of the original mod jar before building:

```
python3 -I tools/extract_assets.py <path to aether-1.7.10-v1.1.2.5.jar>
```

This fills `assets/aether/`, which stays untracked.

## Building

```
./build.sh
```

The release build lands in `out/aether-<version>.bmod`. Drop it into the `mods/` folder of the client and the server.

## License

Code is LGPL-3.0 like the original mod. Assets stay with their authors.
