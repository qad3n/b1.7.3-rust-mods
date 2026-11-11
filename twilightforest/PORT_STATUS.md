# The Twilight Forest port status

Source: TwilightForest-2.4.3.jar for 1.7.10 (`reference/src`, Vineflower decompile, SRG names, never committed).
Target: b1.7.3_rewrite mod loader, one `twilightforest-<version>.b173mod` for client and server.
Assets: never committed, `../../tools/modport/py.sh extract_assets.py <jar> twilightforest` fills `assets/twilightforest`.

## Build and test

```bash
../../tools/modport/test.sh deploy ports/twilightforest
../../tools/modport/test.sh errors
../../tools/modport/modtest.sh cmd loader map <dim> 0 0 8 tf.png
```

## Port decisions

- Every block variant that has its own item is its own block (loader block items place meta 0). In-world state
  (leaf decay, device activity, jet phases, facing) uses named states or meta.
- Twilight logs keep their 1.7 axis as an `axis` state (y, x, z, bark). Vanilla logs the source lays sideways stand
  upright, beta logs have no axis.
- Source class is cited here, never in code comments.
- Shared math comes from `b173_api::math` (`blockOf`, `floor`, `distance`, `length`, `Face`, `yawTowards` for
  faceEntity turns). `math.rs` keeps its own `normalize`: Vec3.normalize divides each axis where the SDK multiplies
  by the inverse. Angles the source computed with double pi (hydra head aim, redcap and goblin knight facing tests,
  naga segments) and all world generation keep their own math.
- Terrain is TF's own 256 high shape squished to 128 (sea level 31), as ChunkProviderTwilightForest does. Random
  heights the source draws over 256 (lakes, dungeons) keep that range so every later draw lines up.
- The dimension is 256 high (TFWorld.CHUNKHEIGHT, loader api 1.9 `Dimensions::height`) with beta's build limit of
  128: players place blocks only below 128, world generation and structures fill the full height, so the final
  castle stands from y 152 up on its plateau as in 2.4.3. Terrain handlers still fill the bottom 128.
- Beta substitutes, accepted differences:
  - no enchantments: efficiency is baked into tool speed, other enchantments dropped;
  - potions ride the loader's status effects (api 1.22, `effects.rs`): slowness, speed, strength, poison and
    regeneration as the source gives them (ice sword, ice bow, ice bombs, tome and nature bolts, life drain scepter,
    zombie wand, lich minions, charms of life, the snowy forest and glacier locks); hunger, blindness, jump boost,
    resistance and fire resistance have no beta stat and stay out (swamp, dark forest, mosquitoes, mist wolves,
    charm extras), the effect lengths are the normal difficulty's, regeneration four heals four every fifty ticks
    where 1.7 heals one every six; yeti armor's chill aura slows a mob that strikes the wearer one level per piece,
    and it and the fiery aura only answer a mob's own blow (damage type mob) as TFEventListener does;
  - biome locks: every locked biome may still bring a hint kobold for the feature that unlocks it;
  - foods heal their hunger value in half hearts and stack to one;
  - no vines: hollow trees climb with ladders, WorldGenVines only draws its randoms;
  - no lily pads, double plants or 1.7 flowers: lily pads only draw randoms, double plants place their single
    kin, flowers become rose or dandelion by colour;
  - no huge vanilla mushrooms: own `huge_brown_mushroom` and `huge_red_mushroom` drawn with tinted gloom textures;
  - no mycelium, podzol, coarse dirt, stone bricks, jungle or mega spruce wood: dirt, grass, mossy cobblestone or
    stone, oak, the tall taiga spruce;
  - no emerald ore: the emerald stalactite is a small diamond one;
  - swamp trees are plain oaks (WorldGenSwamp's vines are its only difference that beta could not show);
  - troll cave clouds sit at y 120 and hollow trees are cut to fit under 128;
  - no experience: uncrafting costs nothing, the recraft cost (enchantments) never applies;
  - no creative tabs: the sorting engine groups chests by item;
  - no spawn eggs: ItemTFSpawnEgg is left out;
  - no written books: the explorer's notes are their own items with beta's book icon and read out in chat;
  - no death messages in beta: TFDamageSources is left out;
  - no map copying in beta: TFMapCloningRecipe is left out;
  - maps: beta draws only its own map in the hand, so a held twilight map shows as a map panel in the top right
    corner of the screen (the loader's dynamic texture and HUD), the holder's pointer is drawn into the picture and
    hidden off a maze map's level, and the magic map skips pixels it already painted since biomes never change;
  - grass and leaves: beta colours them by climate only, so every twilight biome uses the colorizer colour of its
    temperature and rainfall; TF's per biome tints (dark forest, swamp, fire swamp, enchanted rings, dark forest
    center patches) are left out, and TF's own leaves use the same climate colours;
  - the ore meter of 2.4.3 counts but says nothing, the older meters' chat report is restored;
  - structures: no top slabs (bottom halves), stairs lose their upside down bit, iron bars and nether brick fences
    are fences, glass panes glass, cobblestone walls cobblestone, quartz sandstone, redstone lamps glowstone,
    trapped chests chests, vines, flower pots, skulls, tripwire and item frames are left out, stone bricks as above;
  - structure spawners keep beta's spawn range and counts (the dark tower mini ghast spawner's NBT is dropped);
  - the lich tower draws its paintings' spots and art from the random but places none (no hanging entity call).

## Inventory

- 729 classes, ~78k lines. 66 block classes (~150 blocks after splitting variants), ~130 items, 58 mobs, 18
  projectiles, 9 bosses (Naga, Lich, Minoshroom, Hydra, Knight Phantoms, Ur-Ghast, Alpha Yeti, Snow Queen,
  plus the Final Castle placeholder), 23 biomes, ~15 structure families, 2 GUIs (uncrafting, cinder furnace).

## Done

- Blocks (BlockTF*), items, tools, armor, food, recipes and smelting (TFRecipes, gear tiers in `items/gear.rs`).
- Loader: animated sprites, mod biome source with each biome's climate, held item damage, world
  seed, colorizer, mob spawners from mods, facing and axis state faces.
- Dimension: GenLayer stack (`world/layers.rs`), terrain (ChunkProviderTwilightForest), caves and ravines
  (TFGenCaves, TFGenRavine), sky, fog and fixed evening.
- Trees: TFGenSmallTwilightOak, SmallRainboak, LargeRainboak (shared WorldGenBigTree port), CanopyTree, CanopyOak,
  CanopyMushroom, MangroveTree, DarkCanopyTree, HollowTree, TreeOfTime, TreeOfTransformation, MinersTree,
  SortingTree, LargeWinter; saplings grow them.
- Decoration: TFBiomeDecorator, TFDarkForestBiomeDecorator, every biome's decorate, tree, grass and flower picks,
  vanilla 1.7 BiomeDecorator order, the ruins (StoneCircle, Well, WitchHut, OutsideStalagmite, Foundation,
  Monolith, GroveRuins, HollowStump, FallenHollowLog, FallenSmallLog) and the small generators.
- Portal (BlockTFPortal, TFTeleporter, TFTickHandler.PortalEntityItem): a diamond tossed into a flower ringed pool,
  lightning, travel both ways, portals remembered in world data instead of the source's 200 block scan.
- Progression (TFAchievementPage on the loader's mod achievements with beta toasts, old player data bits adopted once), biome enforcement
  (TFBiome*.enforceProgession), item pickups that advance it, config (TwilightForestMod.loadConfiguration).
- Damage types: hits carry the vanilla type of the source's DamageSource (mob, player, thrown, fireball, explosion,
  in_fire, cactus, magic), parts pass their hit on under its type, and the ur-ghast, knight phantoms, goblin knight
  riders, death tome, termites and naga segments (no blasts or fire) check the type as the source does. Wither and anvil hits have no beta type and
  stay generic. A hydra part stuck in a wall breaks the blocks it is caught in (damage type in_wall).
- Items with behaviors: ore meter, ore magnet, crumble horn, peacock fan, lamp of cinders, magic beans, tower key,
  charms of life and keeping (TFEventListener), magic tree cores (BlockTFMagicLogSpecial), uberous soil.
- Machines: uncrafting table (ContainerTFUncrafting on a tile inventory, ghost ingredients drawn over the assembly
  grid) and cinder furnace (TileEntityTFCinderFurnace).
- Loader api 1.20: the crumble horn crumbles every fifth tick while held, the lamp of cinders and ore magnet act on
  release after a short hold (ItemTFCrumbleHorn, ItemTFLampOfCinders, ItemTFOreMagnet); trophies are tile entities
  drawing the bosses' heads (TileEntityTFTrophyRenderer) on a floor turned in sixteenths or against a wall, and drop
  their trophy when broken.
- The cinder furnace reads the `minecraft:logs` tag for the source's logWood ore dictionary entry, so it chars and
  multiplies any mod's logs, not just names ending in log.
- Uberous soil joins the `minecraft:crop_soil` and `minecraft:flower_soil` tags, so crops, flowers and saplings stay on
  it as BlockTFUberousSoil.canSustainPlant allows.
- Fireflies, cicadas and moonworms are tile entity models (loader api 1.20) as TileEntityTFFireflyRenderer and kin:
  turned to the face they cling to, twitching every ten to twenty seconds, the firefly pulsing its glow and
  letting tiny fireflies drift when a player is near, the cicada singing with notes after each rest, and the
  moonworm crawling round with its pieces dipping. The glow blends by alpha where the source added it; the block
  itself draws a blank cuboid. Not seen in a client yet.
- Held use (loader api 1.20) as the source: the twilight wand fires a bolt every sixth tick of the hold, the life
  drain scepter works every fifth tick and heals every tenth, the zombie wand and peacock fan stay raised for 30
  and 20 ticks after use, the block and chain and cube of annihilation are held up while out (the cube returns
  when the use key is let go), and the moonworm queen fires a moonworm when let go after twelve ticks.
- Batch 3: the armored giant and redcaps wear their armor (RenderBiped's two armor layers over their own limb swing,
  `models/armor.rs`), chunks are populated with creature herds (SpawnerAnimals.performWorldGenSpawning with the
  creature list's SpawnListEntry group sizes, fireflies stay ambient), the tree of transformation turns columns to
  the enchanted forest (loader api 1.16 biome overrides, saved and synced).
- Loader api 1.13 and 1.14 features: castle runes glow a pixel off castle brick (RenderBlockTFCastleMagic), giant
  blocks show a quarter of a quarter of the vanilla face (GiantBlockIcon), nagastone heads and bodies refit to their
  neighbors with every source shape, sprite and turn (BlockTFNagastone, RenderBlockTFNagastone), the stronghold
  shield is dug only from its inside face at the player's stone speed, a used trophy pedestal cannot be broken.
- Loader api 1.12 features: TwilightForestSeed seeds the dimension (WorldProviderTwilightForest.getSeed), the
  ur-ghast's tantrum storms the overworld for 6000 ticks, the hydra dies over 200 ticks with a head falling every
  second, trollsteinn shows its lit face toward light, huge lily pad and water lily items lay onto the still water
  the player looks at, and the slide block keeps its orientation in flight.
- Block effects: knightmetal block pricks what touches it and shows its spikes, trollsteinn glints red toward dark
  faces, transformation leaves shed runes, broken logs, cores, huge stalks and live thorns start nearby leaves decaying
  (BlockLog.onBlockRemoval). Slider (BlockTFSlider, EntityTFSlideBlock on the loader's moving_block): wakes on its
  four second beat while a player is within 32 of its column, slides out and back, hurts and knocks back what it
  hits. Trophy pedestal: RenderBlockTFPedestal's base, column and cap with the four boss faces turned by meta, lit
  while latent; a trophy set on it rewards nearby players first, so the first worthy visit opens it, and only the
  access chamber's latent shields (Java meta 15, the port's facing "none") vanish.
- Mobs: 46 entities from one table (`entities/mobs.rs`), models converted from every ModelTF* class, spawn rules by
  biome and depth (`entities/spawns.rs`), throwable shots (`entities/shots.rs`), the shared AI pieces
  (`entities/ai.rs`).
- Structures: every StructureComponent class of `twilightforest.structures` (170 classes, 18k lines of Rust) is
  translated by `tools/structgen.py` (javalang AST to Rust) and fixed by hand afterwards, so the drafts are no longer
  regenerated (use `--only Class` for a fresh draft to diff against). One `Piece` struct holds every class's fields
  in Cells, `piece.rs` dispatches overridden methods by `Kind`, one module per class holds its methods as free
  functions. StructureComponent and StructureTFComponent helpers live in `component.rs`, 1.7 and TF blocks map to
  beta and port blocks in `legacy.rs`, world generators in `gens.rs`. Starts are laid out once per feature chunk and
  built chunk by chunk with the populate random (MapGenStructure, StructureTFMajorFeatureStart incl.
  moveToAvgGroundLevel). Checked generating: lich tower, dark tower, hedge maze, ice tower and aurora palace,
  mushroom tower, naga courtyard, quest grove, hollow hills, hydra lair, labyrinth, stronghold, troll caves, yeti
  cave. Boss spawner block (BlockTFBossSpawner, TileEntityTF*Spawner). Bosses in `bosses/`: Naga, Lich, Minoshroom,
  Alpha Yeti (with EntityTFIceBomb and its item, EntityTFFallingIce, rampage, rider throw), Hydra (HydraHeadContainer
  state machine, heads and necks as entities placed by the body, mortars, armor and open mouth damage), Knight
  Phantoms (formations, charges, thrown axes and picks, the six knight spawner), Snow Queen (summon, drop and beam
  phases, ice shields, crystal minions), Ur-Ghast (hand made ModelTFTowerBoss, trap course, aggro, triple fireballs,
  tantrums with mini ghast flocks, ghast trap hold through `entities::ghastTrapped`).
- Structure protection (TFEventListener.isAreaProtected through `protection.rs`): breaking, chests, levers,
  buttons and tower devices, and hurting monsters inside an unconquered protected piece are refused without the
  feature's progress gate, the piece's edges flash red. Trophies place as small head blocks; a trophy on the
  stronghold's latent pedestal opens it for worthy players (BlockTFTrophyPedestal). Giant hollow oaks as structures
  (MapGenTFHollowTree, StructureTFHollowTreeStart, one chunk in 48 in the oak biomes).
- Loader structure registry (api 1.10, `structures::register` and `record`): every major feature with a structure is a
  kind `twilightforest:<FeatureRow.m_name>` with a finder on `features::directlyAt`, the hollow oaks are
  `twilightforest:hollow_tree`. A start is recorded with its piece boxes the first time a chunk builds part of it, so
  `/locate` and other mods see them. Protection and spawn lists still use the mod's own start cache, which also works on
  clients.
- Cinder furnace idle and lit blocks (BlockTFCinderFurnace.updateFurnaceBlockState): the lit block gives light 1.0 and
  drops the idle furnace, the swap goes through loader api 1.11 `World::swapBlock` so the tile entity, its slots and
  an open screen stay.
- Spawning is ChunkProviderTwilightForest.getPossibleCreatures through the dimension's spawn list handler (loader api
  1.9, `entities/spawns.rs` `possible`): inside a feature's pieces its own TFFeature lists with the source weights,
  nothing in a conquered feature's footprint, the underground lists below y 31, the dark forest, snowy forest and swamp
  monster lists on their 1 in 20 or 10 rolls, no surface monsters in any other biome, TFBiomeBase's creatures (only
  ravens on the final plateau, none in the thornlands and streams, penguins on the glacier), squid in lakes. Endermen,
  witches, blazes and bats are not in beta and left out; fireflies stand in as creatures for the ambient list.
  `loader spawns 4 monster <x> <y> <z>` on the console prints a spot's list.
- Boss bars only for the bosses with IBossDisplayData in 2.4.3: Naga, Lich, Hydra, Ur-Ghast, Snow Queen.
- Uberous soil turns to dirt under a solid block on a scheduled tick: the source's direct neighbor change cascaded
  through a whole troll garden patch covered by falling sand and overflowed the wasm stack, which disabled the mod and
  left flat stone terrain behind.
- Scepters (twilight, life drain, zombie with EntityTFLoyalZombie) and bows (triple, seeker, ice, ender with their
  arrows in `entities/arrows.rs` on the loader's `arrow` template, so beta EntityArrow flight, sticking, pickup and
  RenderArrow), drawn while the use key is held and loosed on release with the draw's speed (loader api 1.20 item
  use, with 1.7's bow zoom, slowed walk and first person pull; beta arrows have no critical hit). The triple bow's side arrows are a `side_arrow` kind
  that drops its shooter once stuck, so only the middle one can be picked up. The ender bow swaps places on any arrow
  hit while held, as TFEventListener does. Beta arrows deal a flat 4, not 1.7's speed scaled 6, and the triple bow
  cannot clear the target's hurt cooldown (no SDK field), so a second arrow in the same moment bounces. Transformation powder (beta pairs), moonworm queen.
- Block and chain and cube of annihilation (`items/thrown.rs`, EntityTFChainBlock, EntityTFCubeOfAnnihilation): one
  out per player, the ball smashes soft blocks and swings back, the cube eats blocks where its holder looks.
- Giant miner and armored giant (RenderTFGiant: a four times biped with the steve skin), harbinger cube
  (ModelTFApocalypseCube), roving cube (`entities/roving_cube.rs`, seeks rings of eight redstone wires).
- Mob AI (`entities/`): quest ram (`quest_ram.rs`, wool bands, loose wool, reward), redcaps and sapper (`redcap.rs`,
  shy circling, lighting and planting tnt), kobold packs (`flock.rs`, panic, flocking, leap; helmet crab leaps),
  block goblin chain (`goblin.rs`), goblin knight pair (`goblin_knight.rs`, rider, shield, armor, heavy spear,
  remount after reload), yeti grab and throw, troll death ripening trollber and its rare magic beans, tower termite
  allies and burrowing, adherent nature bolts, lich minion and loyal zombie tints with zombie arms, bunny and tiny
  bird texture variants, spider, wolf and ghast animations (ghast tentacles rebuilt by hand, the converter missed
  the loop). The troll's rock throw is dead code in 2.4.3 (only set from NBT) and is not ported.
- Every custom mob model animates (`models/animated.rs`): beetles, helmet crab, bunny, squirrel, birds, raven,
  penguin, tower golem, mosquito swarm, death tome, ice cores and crystal; wraith uses zombie arms. Loop-built
  children the converter merged (each ice spike's cube) are rebuilt per spike.
- Charm effects (EntityTFCharmEffect, `charms.rs`): two used charms circle the player for ten seconds.
- TFWeatherRenderer locks (`weather.rs`, on the loader's world render, API 1.8): blizzard, mosquitoes, ash, dark
  streams and heavy rain over biomes the player has not earned, and a refused guarded piece glitters inside its box
  under EntityTFProtectionBox's scrolling shell. The server syncs each player's progress to their client for it.
  TF's normal rain and snow are vanilla 1.7 weather, which beta draws itself.
- Held items on the loader's held item pose op (API 1.8): redcaps' picks, minotaur's gold axe, skeleton druid's gold
  hoe, the giants' stone tools, the minoshroom's axe and each knight phantom's weapon.
- Breath attacks (EntityAITFBreathAttack, `entities/breath.rs`): winter wolf frost and fire beetle flames that burn.
  Pinch beetle charges, pinches a player up and runs off with them (EntityAITFKidnapRider). Naga mixes its rattle
  into its hissing. `/tfprogress <player> <boss>` (CommandTFProgress).
- Hint kobolds (TFFeature.trySpawnHintMonster, `hints.rs` with the texts in `hint_books.rs`): a refused break or use
  in a guarded piece brings a kobold carrying that feature's explorer's notes, once a minute per feature.
- Config: DisableUncrafting, PortalCreationItem, TwilightOakChance join the earlier options. Item and block names
  follow the 2.4.3 lang file, [WIP] and [NYI] tags included.
- Magic, maze and ore maps with their blank maps and recipes (`items/maps.rs`), shown as a HUD panel while held:
  the server paints a stripe a tick around the holder (biome per chunk with feature icons, or the blocks of the
  map's level with ores shaded by depth), saves the picture in world data and sends it to the holder.
- Dev command `/twilightforest:tffeature info|center [x z]` (CommandTFFeature), coordinates for the console, plus
  `locate [ring] <label words>` (port test aid) giving the nearest center of a feature kind.

## Bugs to check at the end

- TFClientProxy.spawnParticle classes are loader mod particles (api 1.24, `particles.rs`): a beta particle recolored, resized, retimed or weighted toward each EntityFX; their own textures and animated fading are not reproduced.
- The tree of transformation's enchanted forest columns color grass and leaves and steer loader spawning, but the
  port's own biome lookups (biome locks, its spawn lists) still read the generated biome; the SDK has no biome query.
- Death tome uses the 1.7 enchanting book texture, which beta and the jar do not have.
- Structure warnings from the translated Java (unused locals, never read assignments) are still to be cleaned.
- The stronghold shields from StructureTFMajorFeatureStart.addShieldFor are unused in 2.4.3 and not ported.
- Hydra: the mortar cannot be knocked back (projectile entities take no hits). Snow queen's triple and seeker bow drops need the bows (items).
- Boss AI is checked by code review only: dimension 3 keeps no chunks loaded without a player, so the test server
  cannot tick them.
- Not checked in a client yet: mob models, poses and textures, the uncrafting table ghosts, portal visuals.
- Cinder furnace: the title sits left like other mod screens instead of centred.
- A player placed trophy pedestal is never latent (beta block items place meta 0).
- Moss patches puff a gray spore one tick in ten (BlockTFPlant), a recolored smoke since beta has no townaura.

## Remaining, in order

1. The redcaps'
   swap to tnt or flint while planting or lighting is not shown; tower ghast eye textures by attack state; per
   variant block names (Blue Castle Rune Brick and the like need per state names in the loader).
2. Config OldMapGen (pre 1.7 map gen, "may not be fully supported" in 2.4.3) is not ported.
3. TFClientEvents' iced entity overlay (ice cubes around a slowed mob) is not drawn yet; slowness now exists but
   other mobs' effects may not reach clients.
4. Not checked in a client: the map panel, mob poses and animations, held items, weather locks, charm effects,
   multiplayer.

## Original plan

1. Blocks: wood (logs, roots, leaves, planks, saplings, magic logs and cores, magic leaves), plants, critters (firefly,
   cicada, moonworm, jar), stone families (mazestone, nagastone, knightbrick, deadrock, troll steinn, castle bricks,
   castle magic, aurora set), tower blocks (towerwood, devices, translucent), thorns, clouds, giant blocks, huge
   lily pad and water lily, huge stalk, gloom, troll roots and berries, force field, slider, shield, spawners,
   trophy and pedestal, fire jet, uberous soil, knightmetal block, castle doors, cinder log and furnace, portal.
2. Items: materials, food, ironwood/steeleaf/fiery/knightly/naga/phantom/yeti/arctic armor and tools, giant tools,
   ice and glass swords, minotaur axe, mazebreaker, bows (triple, seeker, ice, ender), scepters, maps and foci,
   ore meter, ore magnet, crumble horn, peacock fan, moonworm queen, charms, tower key, lamp of cinders, ice bomb,
   chain block, cube of annihilation, magic beans, transformation powder, trophies, spawn eggs.
3. Recipes (TFRecipes), smelting, uncrafting table GUI, cinder furnace GUI.
4. Loader: animated block textures (mcmeta strips), multi biome dimension source with per chunk grass and foliage
   colors, per biome spawn lists.
5. Dimension: TFWorldChunkManager GenLayer stack, ChunkProviderTwilightForest terrain (sea level 31), caves and
   ravines, biome decorators, trees (canopy, mangrove, dark, rainbow, hollow, giant, magic trees), small features
   (stones, wells, ruins, lampposts, roots, mushgloom, fallen logs), sky and fog, fixed evening time, portal and
   teleporter.
6. Passive mobs: deer, boar, bighorn, bunny, squirrel, raven, tiny bird, penguin, firefly, quest ram.
7. Hostile mobs: redcap, sapper, kobold, swarm and hedge and king spiders, wolves (hostile, mist, winter), wraith,
   skeleton druid, mosquito swarm, death tome, beetles, maze slime, minotaur, mini ghast, tower ghast, golem,
   termite, broodling, goblins and knights, helmet crab, yeti, snow guardian, ice cores, troll, giants, ice crystal,
   harbinger cube, adherent, roving cube.
8. Projectiles: nature, lich, wand, tome bolts, hydra mortar, lich bomb, moonworm shot, slime blob, charm effect,
   thrown axe and pick, falling ice, ice bomb, seeker and ice arrows, ice snowball, chain block, cube, slide block.
9. Structures: hollow hills, hedge maze, naga courtyard, lich tower, minotaur labyrinth, hydra lair, quest grove,
   dark tower, yeti cave, aurora palace, troll caves, knight stronghold, mushroom tower, final castle, hollow trees.
10. Bosses: Naga, Lich, Minoshroom, Hydra, Knight Phantoms, Ur-Ghast, Alpha Yeti, Snow Queen; boss spawners,
    trophies, progression gates (biome enforcement and shields).
11. Player systems: charms of life and keeping, magic and maze maps, ore magnet, crumble horn, peacock fan,
    lamp of cinders, transformation powder, quest ram rewards.
12. Sounds, particles, music, config.
