# Wireless Redstone

Port of ali4z's Wireless Redstone 1.4 for Minecraft beta, with its five addons (Wireless Clocker 0.2, Power
Configurator 0.5, Wireless Remote 0.4, Wireless Sniffer 0.5, Wireless Triangulator 0.4), to the b1.7.3_rewrite mod
loader as one mod. One `.bmod` runs on the client and the server.

## Assets

The textures belong to the original author and are not in this repository. Supply them from your own copies of the
six original zips before building:

```
python3 -I tools/extract_assets.py <folder with the original zips>
```

This fills `assets/wirelessredstone/`, which stays untracked.

## Building

```
./build.sh
```

The release build lands in `out/wirelessredstone-<version>.bmod`. Drop it into the `mods/` folder of the client and
the server.
