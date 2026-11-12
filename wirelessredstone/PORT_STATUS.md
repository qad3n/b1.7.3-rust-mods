# Wireless Redstone port status

Source: ali4z's Wireless Redstone 1.4 for b1.7.3 (ModLoader) and its five addons, from `b1.7.3_files/Wireless Redstone/`:
`WirelessRedstone14.zip`, `WirelessRedstoneClocker02.zip`, `WirelessRedstonePowerConfigurator05.zip`,
`WirelessRemote04.zip`, `WirelessSniffer05.zip`, `WirelessTriangulator04.zip`. The zips were merged into one jar and
decompiled with Vineflower into `reference/src` (never committed). Obfuscated names resolve through
`b1.7.3_decomp/mappings/b1.7.3-client.tiny`.
Target: b1.7.3_rewrite loader, one `wirelessredstone-<version>.bmod` for client and server. All six mods are one mod.
Assets: `python3 -I tools/extract_assets.py "<folder with the six zips>"` copies the sprites and gui sheets into the
modern layout and writes the lang file.
Build: `./build.sh`. Test: `../../tools/modport/test.sh deploy ports/wirelessredstone`.

## Design

- The ether (`RedstoneEther`, `RedstoneEtherFrequency`, `RedstoneEtherNode`) is server state per dimension in
  `ether.rs`: frequency to transmitter and receiver sets. It is rebuilt from tile ticks after a load, as the source
  rebuilt it from `updateEntity`. The source reset it on every world hash change; per dimension maps do that here.
- Receiver, transmitter and clocker keep the on state as the named `powered` state, read with `StateData::flag` and
  written with `setBlockState`, so the textures are state variants and the client needs no tile data for the look.
  Blocks are found by name (`World::is`), recipes key vanilla items by name, and a neighbour change counts as a real
  block unless it is `minecraft:air`. The only numeric ids left are the engine handles passed through to
  `notifyNeighbors`: the source tells the neighbours "block 0" and, from its own add and remove hooks, its own id, so
  the port passes 0 and the callback's handle. `notifyAround` would pass the block now there, which is air once the
  receiver is removed.
- Tile data (`Tile` in `tile.rs`) holds the frequency, the six direct and six indirect power routes, and for the
  clocker its period and running flag. It is synced so screens and the frequency labels read it on the client.
- Every gui button is a serverbound message, validated by distance and block on the server, as the source changed
  the tile from the gui on the single player client.
- Threads become ticks. The clocker thread checked every 10 ms; the port adds 50 ms per tick and toggles each time the
  elapsed time passes the period (minimum 200 ms, so 4 ticks). The remote's 1 second pulse is 20 ticks.

## Registered with the loader

- Blocks `receiver`, `transmitter`, `clocker` with the named `powered` state and its textures (`ctx.blocks()`
  `register` and `states`), tile entities of the same names with synced tile data (`ctx.tileEntities()`): `blocks.rs`.
- Items `remote`, `power_configurator`, `sniffer`, `triangulator` (`ctx.items()`), the frequency on a stack as the
  component `wirelessredstone:freq`: `items.rs`.
- Seven shaped recipes (`ctx.recipes()`). Iron and redstone keys are the tags `c:ingots/iron` and `c:dusts/redstone`
  (`keyTag`), so other mods' iron and redstone craft them too; lever, torch, repeater and compass have no tag and stay
  by name: `recipes.rs`.
- Screens (`ctx.screens()`): `screens.rs`. Channels for every gui button and the sniffer and triangulator answers
  (`ctx.channels()`): `net.rs`. Tick, chunk unload and world unload events (`ctx.events()`): `lib.rs`.
- Service `wirelessredstone:ether` version 1 with `state`, `active` and `closest`, and lookup `wirelessredstone:node`
  with `frequency` on all three blocks (`ctx.services()`): `service.rs`. Other mods depend on the `api/` crate
  `wirelessredstone_api` for the names and message types, as the SDK's energy example does.
- Names and screen text come from `assets/wirelessredstone/lang/en_US.lang`.

Kept in the mod on purpose:
- The ether is an index of the loaded transmitters and receivers, rebuilt from tile ticks as the source rebuilt it from
  `updateEntity`. Its truth is the tile data and the `powered` state, which the loader already saves and syncs. Saving
  the index as world data would write the save on every clock toggle and keep entries for unloaded chunks. Other
  mods read it through the service.
- Remote pulses are timers of at most 20 ticks and are not saved, as in the source. Screen and triangulator needle
  state is client display state.
- No config: the source's properties file held only block and item ids, which the loader assigns by name.

## Beta and multiplayer adaptations

- The source is single player only. The remote's pulse, the sniffer grid and the triangulator needle need the server
  ether, so the server answers them over channels.
- The remote kept its frequency per `ItemStack.hashCode` in memory, lost on reload; it is a stack component now.
- The triangulator kept one frequency for every triangulator in a static field; it is a stack component now, and the
  needle follows the held triangulator, else the first one in the inventory.
- `isOpaqueCube` false on the three blocks: the loader has no flag for it alone, so they set light opacity 0 instead.
  Faces of neighbours against them are culled as against any full cube, which looks the same.
- Item and block ids from `wirelessRedstone.properties` are gone: the loader assigns ids by name.

## Loader additions (API 1.38, generic)

- `setBlockState` with another state of the block already there rewrites only its metadata
  (`setBlockMetadataWithNotify`), so the tile entity and its frequency stay, as the source's metadata write did.
  Before, the write replaced the block and reset the tile.

## Loader additions (API 1.35, generic)

Added next to another session's uncommitted 1.35 machine viewer work in the SDK and loader; both must ship in the same
API version, or these move to 1.36 and `mod.toml` follows.

- `spritePixels(texture)` (`HostCall::SpritePixels`, `HostReply::Pixels`, loader `atlas::pixels`): a sheet sprite's
  pixels as the active packs give it, the base the triangulator needle is painted over.
- `ModelOp::Text` and `ModelOp::DepthMask` (client `entity_model::glOp`): font text in a pose, for the frequency labels.

## Done

Verified on an own test server (`MODTEST_HOME=b1.7.3_rewrite/target/modtest-wireless MODTEST_PORT=25571`), no errors
in the logs:
- `tools/check_ether.sh`: a torch beside a transmitter powers a receiver 10 blocks away on frequency 0, which lights
  redstone wire beside it; removing the torch turns all three off.
- `tools/check_clocker.sh`: a powered clocker toggles the receiver once per second (default 1000 ms), and off once
  unpowered.
- `tools/check_reload.sh`: after a server restart the ether is rebuilt from tile ticks and the saved pair still works.

Verified in a test client (single player, `tools/check_client.sh` with `tools/client_lib.sh` and `tools/dev.sh`,
screenshots compared with the source's gui code), no wasm traps in the log:
- Blocks: receiver, transmitter and clocker faces, on and off textures, frequency labels upright on all four sides
  just under the top edge, labels follow a frequency change at once.
- Block items and the four tool icons in the hotbar; remote and configurator wear per use.
- Frequency gui (small sheet), clocker gui (large sheet cut at 166 rows as the source drew it): every step button,
  wraparound, the retuned receiver goes dark at once and lights when it matches a powered transmitter.
- Remote: frequency on the stack, Pulse closes the gui and powers the matching receiver for about a second.
- Power configurator: opens only on a receiver, on/off button skins, U direct off also clears U indirect as
  `flipPowerDirection` does, D indirect off alone.
- Sniffer: 125 by 80 grid, the one active frequency (1000) green at row 8 column 0.
- Triangulator: frequency gui, needle up with the transmitter ahead, left with it to the left, down with it behind.
- Recipes: all seven registered with the source's patterns and ingredients (registry report); not crafted by hand.

Not run: multiplayer with two players, crafting in a table, a texture pack replacing the triangulator sprite.

| Feature | Source | Port |
|---|---|---|
| Ether: frequencies, nodes, closest active transmitter | `RedstoneEther*` | `ether.rs` |
| Wireless Receiver: power routes, indirect power, updates from the ether | `BlockRedstoneWirelessR`, `TileEntityRedstoneWirelessR` | `blocks.rs` |
| Wireless Transmitter: powered by redstone, sets its node | `BlockRedstoneWirelessT`, `TileEntityRedstoneWirelessT` | `blocks.rs` |
| Wireless Clocker: toggles its node every period while powered | `BlockRedstoneWirelessClocker`, `TileEntityRedstoneWirelessClocker`, `ThreadWirelessClocker` | `blocks.rs` |
| Frequency gui (small sheet), clocker gui (large sheet) | `GuiRedstoneWireless`, `GuiRedstoneWirelessR/T/Clocker` | `screens.rs`, `net.rs` |
| Frequency labels on the four sides | `TileEntityRedstoneWirelessRenderer` | `labels.rs` |
| Wireless Remote: 64 uses, frequency gui, pulse for 1 s where it was used | `ItemRedstoneWirelessRemote`, `GuiRedstoneWirelessRemote`, `ThreadWirelessRemote`, `MemRedstoneWirelessRemote`, `GuiButtonBoolean` | `items.rs`, `screens.rs`, `net.rs` |
| Power Configurator: 64 uses, direct and indirect routes per side | `ItemRedstoneWirelessPowerDirector`, `GuiRedstoneWirelessPowerDirector`, `GuiButtonPowerDirection` | `items.rs`, `screens.rs`, `net.rs` |
| Wireless Sniffer: 125 by 80 grid of all 10000 frequencies | `ItemRedstoneWirelessSniffer`, `GuiRedstoneWirelessSniffer`, `ThreadWirelessSniffer` | `items.rs`, `screens.rs` |
| Wireless Triangulator: frequency gui, needle sprite to the closest active transmitter | `ItemRedstoneWirelessTriangulator`, `GuiRedstoneWirelessTriangulator`, `TextureTriangulatorFX` | `items.rs`, `triangulator.rs` |
| Recipes of all six mods | `mod_*.AddRecipes` | `recipes.rs` |

## Fixed source bugs

- `getClosestActiveTransmitter` kept the first node's distance while replacing the position, so the needle pointed at
  the last transmitter nearer than the first instead of the nearest.
- The transmitter set its metadata without a block update, so its texture changed only when the chunk redrew.
- Receivers of unloaded chunks leave the ether (they join again when loaded) so updates never load chunks.
- A frequency update on a removed receiver wrote metadata into whatever block stood there; it checks the block now.

## Bugs found in the client and fixed

- The receiver's texture did not change after a gui retune: `setBlockMetadataWithNotify` redraws only self notifying
  blocks, and the source called `markBlockNeedsUpdate` after it. `blocks::setMeta` marks the block now, which also
  sends the change to multiplayer clients.
- Pulse closed the remote gui from inside its own mouse callback; the host reentered the screen with `Close` and the
  wasm trapped (`panic_already_borrowed`), which killed the mod. The close waits for the next client tick now.

## Loader API 1.48 to 1.55 (registries and data pass, 2026-11-11)

- Receiver, transmitter and clocker declare their tile, `powered` property and lit faces in one block builder.
- No per-entity state, loot or `c:` material tags to move. Not run: build and clippy only.

## Bugs to check at the end

- Anaglyph colors of the triangulator needle are not ported (the source mixed them when 3D anaglyph was on).

## Todo, in order

- Multiplayer: two players, guis on a dedicated server.
