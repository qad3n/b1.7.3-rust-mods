# IndustrialCraft 2

Port of IndustrialCraft 2 v1.00 for beta 1.7.3 to the b1.7.3_rewrite mod loader. One `.bmod` runs on the client
and the server. Needs loader API 1.33.

## Assets

The textures, gui images and armor layers belong to the original authors and are not in this repository. Supply them
from your own copy of the IndustrialCraft 2 v1.00 zip before building:

```
python3 -I tools/extract_assets.py "<path to IndustrialCraft2 v1.00.zip>"
```

This fills `assets/ic2/`, which stays untracked. The zip carries no sounds; IC2 played them from a separate resource
download, so a texture pack may add them under `assets/ic2/sounds/`.

## Build

```
./build.sh
```
