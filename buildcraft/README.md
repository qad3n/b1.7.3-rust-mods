# BuildCraft

Port of BuildCraft 2.1.1 (Core, Transport, Energy, Factory and Builders) to the b1.7.3_rewrite mod loader as one mod.
One `.bmod` runs on the client and the server. Needs loader API 1.33.

## Assets

The textures and gui images belong to the original authors and are not in this repository. Supply them from your own
copy of the BuildCraft 2.1.1 zips before building:

```
python3 -I tools/extract_assets.py "<folder holding the BuildCraft *.zip files>"
```

This fills `assets/buildcraft/`, which stays untracked.

## Build

```
./build.sh
```
