# Wireless Redstone

Port of ali4z's Wireless Redstone 1.4 for Minecraft beta, with its five addons (Wireless Clocker 0.2, Power
Configurator 0.5, Wireless Remote 0.4, Wireless Sniffer 0.5, Wireless Triangulator 0.4), to the b1.7.3_rewrite mod
loader as one mod. One `.bmod` runs on the client and the server.

## Assets

The textures belong to the original author and are not in this repository or in the `.bmod`. Put your own copies of the six original zips
(`WirelessRedstone14.zip`, `WirelessRedstoneClocker02.zip`, `WirelessRedstonePowerConfigurator05.zip`,
`WirelessRemote04.zip`, `WirelessSniffer05.zip`, `WirelessTriangulator04.zip`) in the game's
`mods/wirelessredstone_assets/` folder. On its next launch the client builds the mod's art from them, following
`assets.toml`. The names and screen text are the port's own and ship in `assets/wirelessredstone/lang/en_US.lang`.

## Building

```
./build.sh
```

The release build lands in `out/wirelessredstone-<version>.bmod`. Drop it into the `mods/` folder of the client and
the server.
