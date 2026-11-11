# BuildCraft port status

Source: BuildCraft 2.1.1 for b1.7.3 (ModLoader and Forge): Core, Transport, Energy, Factory and Builders zips merged
into one jar, remapped to MCP names with tiny-remapper and the decomp mappings, decompiled with Vineflower into
`reference/src` (never committed). `reference/src_obf` holds the obfuscated decompile, `reference/textures` the raw
sheets.
Target: b1.7.3_rewrite loader API 1.33, one `buildcraft-2.1.1.bmod` for client and server.
Assets: `python3 -I tools/extract_assets.py "<folder with the BuildCraft zips>"` fills `assets/buildcraft`. It cuts the
block and item sheets, copies the guis and engine textures, and writes a first oil frame the live effect paints over.
Build: `./build.sh`. Test: `../../tools/modport/test.sh deploy ports/buildcraft --game` (the loader changed).

## Loader additions (API 1.33, generic, documented in tools/rewrite/modloader-progress.md)

Item, fluid and energy capabilities on mod tiles; an energy registry with conversion between mods' kinds; capability
query; block drops, block placing without a player, autocrafting, container items, fuel ticks, item info, item drops
with motion, item entity stack changes, entities in a box, inventory size, biome at a column; six face connection
masks for model cuboids; block item models; a stack draw op; startup material unification from `c:` tags and names;
`blockDropStack`, the single idDropped/quantityDropped stack ModLoader era machines take from a block.

## Done

| Feature | Source | Port |
|---|---|---|
| Gears, wrench | `BuildCraftCore` | `core/items.rs`; gears in `c:gears/*`, any `c:tools/wrench` item turns machines |
| Power providers, pneumatic and redstone frameworks, continuous current | `PowerProvider`, `RedstonePowerProvider`, `PneumaticPowerProvider` | `core/power.rs`, config `power.framework`, `current.continuous`; energy kind `buildcraft:mj`, worth 10 |
| Pipes: wooden, stone, cobblestone, iron, golden, diamond, obsidian | `transport/*` | `transport/`; connection rules, speeds, routing, wooden extraction with energy and switching, iron output and wrench and redstone, gold boost and powered look, diamond filters and screen, obsidian suction and collision pickup |
| Items in pipes | `TilePipe`, `EntityPassiveItem`, `RenderPipe` | `transport/travel.rs`, `pipe.rs`, `render.rs`; item packets to tracking clients, client side movement |
| Liquids in pipes | `TilePipe.moveLiquids`, `RenderPipe` | `transport/liquids.rs`, `render.rs`; one fluid per pipe |
| Engines: redstone, steam, combustion | `energy/*Engine*`, `RenderEngine`, guis | `energy/engine.rs`, `model.rs`, `mod.rs`; stages, piston stroke, explosions, fuel, oil, chaining, wrench, red dust, screens, 3d items |
| Oil, oil bucket, deposits | `BlockOil*`, `ItemBucketOil`, `OilPopulate`, `TextureOil*FX` | `energy/oil.rs`; loader fluid, desert springs and underground pools with wells and geysers, seeded per chunk by name |
| Mining well, mining pipe, frame | `factory/*` | `factory/well.rs`, `mod.rs` |
| Quarry | `TileQuarry`, `EntityMechanicalArm`, `BluePrintBuilder` | `factory/quarry.rs`; landmark or default area, frame building with stripe lasers, arm movement and digging, output to inventories, pipes or the air |
| Pump, tank | `TilePump`, `TileTank`, `RenderTank` | `factory/pump.rs`, `tank.rs`; tube, layer by layer pumping, tank columns |
| Automatic crafting table | `TileAutoWorkbench` | `factory/workbench.rs`; refill rules, crafting on extraction, nearby ingredients, containers |
| Landmarks | `TileMarker`, `BlockMarker` | `builders/marker.rs`; linking, red box, blue signals while powered, area for machines |
| Filler and its six patterns | `TileFiller`, `Filler*`, `FillerRegistry` | `builders/filler.rs`, `stairs.rs`; pattern recipes, side icons, screen |
| Template drawing table, builder, templates | `TileTemplate`, `TileBuilder`, `BluePrint`, `ItemTemplate` | `builders/blueprint.rs`; computing time, inverted masks while powered, rotations, building and clearing; blueprints shared by every save in `moddata/buildcraft/blueprints.dat`, templates named "Template #N" |
| Landmark block | `BlockMarker`, `renderMarkerWithMeta` | `builders/landmark.rs`; any full block face including ceilings, BlockMarker's selection sticks, the marker model drawn full bright |
| Crafting from the automatic table | `ContainerAutoWorkbench` | crafting result slot over the table's grid |
| Pipe routing and packets | `TilePipe.resolveDestination`, `createItemPacket` | random in single player, deterministic on servers with item packets at most once a second |
| Obsidian pipe extras | `TileEntityPickupFX`, chest minecarts | `transport/pickup.rs`, `obsidian.rs` |
| Machine glow | `BlockMachineRoot.getBlockBrightness` | quarry and mining well blocks half again as bright next to a working quarry, mining well or pump |
| Live oil textures | `TextureOilFX`, `TextureOilFlowFX` | `energy/oil_fx.rs`, painted each client tick |

## Additions beyond the original

BuildCraft plays as 2.1.1 did. These only add compatibility with other mods:

- Gears sit in `c:gears/*`, and any item in `c:tools/wrench` turns machines.
- Machines take `buildcraft:mj` (worth 10 base units) and other mods' energy that converts to it; engines power
  any mod's machine that takes a convertible kind.
- Pipes and machines reach every mod's inventories and tanks through the loader, BuildCraft's own slot rules kept
  for two and three slot inventories such as the furnace.
- A pipe carries one fluid at a time by name; 2.1.1 had only oil.
- The pump also fills other mods' fluid blocks next to it; BuildCraft's own tanks and engines still need a pipe.

## BuildCraft bugs fixed

Behaviour follows the 2.1.1 code line by line (audited 2026-11-11 against `reference/src`), except where it broke:

- `MangoPackUtils.isBlockUnbreakable` is not in the zips; it is read as `Utils.unbreakableBlock` (bedrock, lava)
  plus any block with negative hardness.
- The pump filed sideways oil under the previous block's upper layer, draining out of order; each block keeps its
  own layer.
- A wooden pipe could pick a diamond pipe's filter slots as its source and then pull nothing; pipes never count.
- The builder used up a block before trying to place it; it is used only when placed.
- A chest minecart's item could enter an obsidian pipe as an empty stack; it stays on the ground instead.
- Large oil geysers that would reach y 128 vanished through a byte cast; they stop at the top layer.
- An empty tank on top of a column stopped drains from the tanks below it.
- Engines drop their fuel when broken, as every other BuildCraft machine drops its contents.

## Bugs to check at the end

- Checked live on the test server: a redstone engine powers a wooden pipe that pulls from a chest into stone pipes
  and a second chest. That run found power loss applied before work in `Provider::update` (now `update` then
  `decay`, as PowerProvider.update calls doWork first), which starved wooden pipes, builders and fillers.
- Everything changed after that run is checked by build and clippy only.

## Todo, in order

1. Deploy with `--game` (the loader gained `blockDropStack`), then console runs of: wooden pipe to chest chains and
   double chests, iron, golden, diamond and obsidian pipes, steam and combustion engines with fuel, mining well and
   quarry output into chests and pipes, pump draining an oil pool, tank columns, auto workbench fed by a pipe,
   filler patterns, builder with a template, landmarks, oil generation in a fresh world.
2. Client visual check of pipes and items in them, engines, quarry, pump tube, landmark lasers and the screens.
