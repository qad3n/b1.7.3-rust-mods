# IndustrialCraft 2 port status

Source: IndustrialCraft 2 v1.00 for b1.7.3 (ModLoader, Forge), `b1.7.3_files/IndustrialCraft2 v1.00.zip`. Its classes were
remapped from obfuscated beta names to MCP names with tiny-remapper and `b1.7.3_decomp/mappings/b1.7.3-client.tiny`,
then decompiled with Vineflower into `reference/src` (never committed).
Target: b1.7.3_rewrite loader API 1.39, one `ic2-1.00.bmod` for client and server.
Assets: `python3 -I tools/extract_assets.py "<IndustrialCraft2 v1.00.zip>"` cuts the block, cable and item sheets into
sprites, copies the guis and armor layers and writes the lang file. The zip has no sounds.
Build: `./build.sh`. Test: `../../tools/modport/test.sh deploy ports/ic2`.

## Design

- One block per IC2 metadata variant (machines, generators, storage, cables, metal blocks), named by what it is.
- Names, tags and states only, never numeric ids or metadata. Blocks are found with `world.is`, `blockIs`,
  `world.tagged` and `blockTagged`, placed with `world.place`, `setBlockState` or a `StateHandle` in world generation,
  and read with the per epoch `world.stateAt` / `ctx.stateAt` cache, so ticks and renders allocate nothing.
- Named block states replace every IC2 metadata value: machines `facing` and `active` (`TileEntityBlock`, the texture
  hooks draw IC2's per family looks from them), cable `color`, rubber wood `resin` (placed, plain, wet, dry) and
  `facing` of its hole, rubber sheet `supported`, iron fence `charge`, ITNT and nuke `lit`, dynamite `lean`,
  luminator `facing`, and the reinforced door's `half`, `open` and `facing`, listed so each state index is the
  metadata the vanilla door template reads. Vanilla blocks have no named states, so the terraformer still sets their
  `meta` state (tall grass type, ripe wheat, leaves of the log's wood type).
- Tags (`data/`): IC2 blocks and items join `c:ores/*`, `c:storage_blocks/*`, `minecraft:logs`, `leaves` and
  `saplings`. IC2's own lists are `ic2:` tags built from the seeded vanilla tags: `mineable/drill`, `laser_proof`,
  `burnable`, `meltable`, `defusable`, `dynamite_sticks`, `miner_outputs`, `terraformer_plants`, `flattenable`, item
  `reactor_components` and entity `monsters`. Scanner ore values go by `c:ores/*` tag, so other mods' ores count.
- Charge stays in item damage exactly as IC2 kept it (1 full, max-1 empty), so recipe damages carry over unchanged.
- The energy net (`energy/net.rs`) is a port of `EnergyNet`: cheapest-path discovery from each source with cable loss,
  packets split by 1/loss, shocks, cable burn-out, overload explosions. Paths are cached per source and dropped for the
  whole dimension when any IC2 energy block or any player-placed block changes. EU is registered as `ic2:eu` with worth
  4 (2.5 EU per BuildCraft MJ); IC2 sinks take foreign energy through `energyDemand`/`receiveEnergy` capped at their
  rating, and IC2 sources feed foreign sinks next to cables or generators.

## Registries

What each subsystem hands the loader, so other mods see and extend it without IC2 knowing them:

| Subsystem | Registers | Where |
|---|---|---|
| Blocks, items, tile entities, entities, screens | every block with its named states, items, tiles, the explosive and laser entities, every gui | `blocks.rs`, `items.rs`, `machines/`, `explosives.rs` |
| Tags | `c:ores/*`, `c:ingots/*`, `c:dusts/*`, `c:plates/*`, `c:circuits/*`, `c:rubbers`, `c:storage_blocks/*`, `minecraft:logs/leaves/saplings`, IC2's own block, item and entity lists | `data/`, item rows in `items.rs` |
| Crafting | all shaped and shapeless recipes; ingots, dusts, plates, circuits, rubber, redstone, glowstone, diamond, sticks and glass go in as tags | `recipes.rs` |
| Smelting and fuels | dust smelting, `mod_IC2.AddFuel` fuels; the iron furnace and generator burn what the fuel registry gives (lava 2000 and the fuel can stay iron furnace only) | `recipes.rs`, `machines/processing.rs` |
| Machine recipes | macerator, extractor and compressor jobs with tag inputs (`c:ores/*`, `c:ingots/*`, `minecraft:coals`, `minecraft:wool`, `minecraft:sand`, `c:sandstone_blocks`, `c:raw_materials/uranium`); their ticks look jobs up with `machineRecipe`, so another mod's macerator recipe runs in IC2's macerator. The scrap box's drops are its recipes and opening one draws from them. Generator, canner, recycler, electrolyzer, magnetizer, mass fabricator and reactor pages describe the rest | `machines/`, `items.rs` |
| Energy | `ic2:eu` with worth 4, demand and receive on every sink, foreign sinks fed beside cables and generators | `energy/` |
| Item moves | induction furnace and miner store output through `insertItem` into `c:chests` and personal safes | `machines/special.rs`, `machines/mining.rs` |
| Lang | `assets/ic2/lang/en_US.lang` | `assets/` |

Not in a registry yet, and why:
- Ore and rubber tree generation stay IC2's own chunk populate: `ctx.worldgen()` ores draw a flat height range and
  replace one block, while IC2's copper height is a sum of two draws, and there is no tree feature. Both need a new
  worldgen spec (a new ABI struct), so they wait for a loader feature.
- The canner's food and fuel quality tables, the recycler's any-item rule, the electrolyzer's cell swap and the
  reactor's component rules are logic, not input to output jobs; they only describe themselves in the machine registry.
- The pump takes buckets from a chest and the Trade-O-Mat pays into chest slots by slot, which `extractItem` cannot
  pick by item; the IC2 energy net walks IC2's own conductors and only meets other mods through the energy capability.
- Cells are not fluid containers: the fluid registry has no container entries.

## Loader additions (API 1.33, generic)

- `World::explosionResistance`, `World::blastBlock(pos, chance)`, `World::blockDensity(from, entity)` in
  `crates/modloader/src/blast.rs` and `b173_api/src/world/blast.rs`: mods run their own explosions the way
  `Explosion` does. Used for `ExplosionIC2`, `PointExplosion` and the mining laser.
- `minecraft:door` block template (`BlockDoor`), used by the reinforced door.
- SDK only, no API bump: `machineRecipe` / `machineRecipes` (registered machine recipes by machine, cached per
  epoch and content generation), and the loader now seeds item tags `c:ores/*`, `c:storage_blocks/*`, `c:gravels` and
  `minecraft:ice` like their block tags.
- SDK only, no API bump: `World::stateAt`, `BlockCtx::stateAt` (the per epoch `stateOf` cache at a spot), `World::clear`
  and `PropertySpec::new`.
- Fixed `itemInfo` in `tech.rs` reading the block table with item ids above 4096 (crashed the server thread).
- Kit: `MODTEST_HOME` and `MODTEST_PORT` give a parallel session its own test server; the server no longer holds the
  caller's stderr, so piped deploys return.

## Done

Verified on a test server (`tools/check_energy.sh`, `tools/check_reactor.sh`, own instance in
`target/modtest-ic2`): generator to macerator through copper cable smelts ore to dust; two uranium cells give 40 EU/t,
heat climbs 33 per cycle as IC2's formula gives, and an MFSU fills through glass fibre. No panics or errors.
Everything else is checked by build and clippy only.

| Feature | Source | Port |
|---|---|---|
| Ores, metal blocks, reinforced stone, glass and door | `BlockTex`, `BlockMetal`, `BlockTexGlass`, `BlockIC2Door`, `ItemIC2Door` | `blocks.rs` |
| Rubber trees: wood with resin holes and refill, leaves, sapling with bone meal | `BlockRubWood`, `BlockRubLeaves`, `BlockRubSapling`, `WorldGenRubTree` | `blocks.rs`, `world/tree.rs` |
| Resin puddle, rubber sheet trampoline, iron fence pole lift, mining pipe and tip | `BlockResin`, `BlockRubberSheet`, `BlockPoleFence`, `BlockMiningPipe`, `BlockMiningTip` | `blocks.rs` |
| Ore and rubber tree generation | `mod_IC2.GenerateSurface`, `WorldGenMinable` | `world/` |
| Resources, cells, cans, treetap, resin, painters, scrap box, bronze tools and armor, composite vest | `ItemIC2`, `ItemGradual`, `ItemCell`, `ItemTinCan`, `ItemTreetap`, `ItemResin`, `ItemToolPainter`, `ItemScrapbox`, `ItemIC2*`, `ItemArmorIC2` | `items.rs` |
| Batteries, charge rules of every electric item | `ItemBattery`, `ItemBatteryDischarged`, `ElectricItem`, `IChargeableItem` | `electric.rs` |
| Energy net, cables with paint and insulation, shocks, burn-out, overloads | `EnergyNet`, `TileEntityCable`, `BlockCable`, `RenderBlockCable` | `energy/` |
| BatBox, MFE, MFSU, LV/MV/HV transformers | `TileEntityElectricBlock`, `TileEntityTransformer`, `GuiElectricBlock` | `energy/storage.rs` |
| Generator, geothermal, water mill, solar panel, wind mill with drifting wind | `TileEntity*Generator`, `Gui*Generator` | `machines/generators.rs` |
| Iron and electric furnace, macerator, extractor, compressor, recycler, canning machine | `TileEntityIronFurnace`, `TileEntityElectricMachine` family, `TileEntityCanner` | `machines/processing.rs` |
| Electrolyzer, induction furnace (with this build's chest deposit), mass fabricator, magnetizer, tesla coil | `TileEntityElectrolyzer`, `TileEntityInduction`, `TileEntityMatter`, `TileEntityMagnetizer`, `TileEntityTesla` | `machines/special.rs` |
| Miner with scanners and pump link, pump with fountain and geothermal feed | `TileEntityMiner`, `TileEntityPump` | `machines/mining.rs` |
| Terraformer and its five blueprints | `TileEntityTerra`, `ItemTFBP*` | `machines/terraformer.rs` |
| Teleporter and frequency transmitter, luminator, personal safe, Trade-O-Mat | `TileEntityTeleporter`, `TileEntityLuminator`, `TileEntityPersonalChest`, `TileEntityTradeOMat` | `machines/` |
| Nuclear reactor, chambers, reactor screens for every width | `TileEntityNuclearReactor`, `BlockReactorChamber`, `GuiNuclearReactor` | `machines/reactor.rs` |
| Wrench, electric wrench, EU-reader, insulation cutter, drills, chainsaw, nano saber, scanners, mining laser | `ItemTool*`, `ItemElectricTool*`, `ItemNanoSaber`, `ItemScanner`, `ItemToolMiningLaser` | `tools.rs` |
| Industrial TNT, nuke, dynamite, sticky dynamite, Dynamite-O-Mote, laser beam | `BlockITNT`, `BlockDynamite`, `EntityIC2Explosive`, `EntityDynamite`, `ItemRemote`, `EntityMiningLaser`, `ExplosionIC2`, `PointExplosion` | `explosives.rs` |
| Rubber boots, jetpacks with hover, batpack, nano and quantum suits, suit, hover and laser keys | `ItemArmor*`, `mod_IC2.OnTickInGame` | `gear.rs`, `keys.rs` |
| All crafting, smelting and fuel recipes | `mod_IC2` | `recipes.rs` |

## Accepted differences

- IC2 sounds are played under `ic2.<name>` but the zip carries none; they are silent unless a pack adds them.
- The black painter paints nothing, as in IC2 (black is color 0, the unpainted look).
- Iron fences and mining pipes always collide as a thin pole.
- Rubber wood drops its bonus resin only when a player breaks it.
- The canner cans beta's own foods only.
- The luminator renders at normal brightness; IC2 v1.00 drew it full bright but never lit the world either.
- The reinforced door uses its own top and bottom sprites; IC2 v1.00 showed unfilled sheet slots in the world.
- Nano and quantum armor keep IC2's all-or-nothing absorption, but plain armor still counts once a hit gets through.
- Primed ITNT, nukes, dynamite and lasers draw as flat sprites, not as blocks and crossed quads.
- Energy paths are rebuilt whenever an IC2 block or a player-placed block changes, so the EU-reader's totals
  restart then.
- Machines accept and give items through other mods' pipes by the loader's chest rule, as IC2's plain
  inventories did.
- Machine and crafting inputs take tags, so other mods' ores, ingots and circuits work in IC2 and IC2's in theirs. A
  miner find stores into a chest whenever the chest takes all of it, split over slots or not; IC2 wanted one slot.
- Tags widen IC2's vanilla lists to other mods' content: any `c:chests` takes miner, induction furnace and
  Trade-O-Mat output, any `c:glass_blocks` lets the sun reach a solar panel, any `minecraft:water` turns a water
  mill. Lasers and hot reactors burn every wood material block (IC2 listed logs, leaves, planks and wool).
- The wrench gives defused explosives back with damage 0; IC2 copied the block metadata into the item damage.
- Worlds saved before the named states (2026-11-11) show machines, cables and rubber wood in their default state
  until they change; nothing else is lost.

## Code checks

`cargo test --lib` runs 22 native unit tests (no game needed; `src/host_stub.rs` stands in for the host):
- `energy/net.rs`: path search and packet split against `EnergyNet.discover` and `emitEnergyFrom`: cable loss per
  type, the 0.1 floor, paint rules, cheapest route, storage front only, search limit, 1/loss shares.
- `machines/reactor.rs`: one cycle of `processChambers` against hand computed IC2 values: pulses, heat, coolant and
  plating spread in slot order, redstone stop, lava, ice, enrichment, parts breaking past 10000.
- `electric.rs`: `giveEnergyTo` and `getEnergyFrom` for batteries, crystals, tools, the discharged RE-Battery and the
  electric wrench.

Review fixes against the Java: stale energy paths kept after a burn-out or overload, machines clamping battery draw
(IC2 ignores the request), the wrench sometimes giving back a plain machine block (IC2 always returns the machine),
the water mill dropping a generator, cables joining the electrolyzer.

## Needs a real run

- Client: machine textures and active looks, cable models and paint, every screen's gauges and slot layout (reactor
  for 3 to 9 columns), armor layers, door halves, flat entity sprites, the luminator.
- Player driven: wrench turning and dismantling, treetap, painter, cutter, scanners, mining laser modes, nano saber,
  dynamite throwing and sticking, remote, personal safe ownership, Trade-O-Mat trades, teleporter linking.
- Movement: jetpack thrust and hover, quantum run and jump, rubber and quantum boot fall absorption, nano and
  quantum hit absorption, and all of these for a second player in multiplayer.
- Server, quick console runs only: canner, induction furnace with a chest, mass fabricator with scrap, electrolyzer
  with an MFE, miner with a scanner and pump, terraformer blueprints, transformer redstone flip, cable shock and
  burn-out, overload explosions, ore and rubber tree generation in a fresh world.

## Bugs to check at the end

- Named states need a look: rubber wood resin holes on the right face, cable paint colors, machine fronts and
  active sprites after a reload, door halves and hinge side, dynamite leaning on walls.
- Client look of cables, machines, screens, armor layers, door, entities; multiplayer key and jetpack sync.

## Todo, in order

1. Client run: place and open every machine screen, check gauges and slot layouts against the IC2 guis.
2. Multiplayer: jetpack, quantum suit and laser key behaviour for a remote player.
