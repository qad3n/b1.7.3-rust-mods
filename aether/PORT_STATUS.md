# Aether port status

Source: Aether Legacy 1.7.10 v1.1.2.5 (`reference/src`, Vineflower decompile of the user's jar, SRG names).
Target: b1.7.3_rewrite mod loader, one `aether-<version>.bmod` for client and server.
Assets: never committed, `python3 -I tools/extract_assets.py <jar>` fills `assets/aether`.

## Build and test

```bash
cargo build --release --target wasm32-unknown-unknown
../../b1.7.3_rewrite/vendor/b1.7.3-modding-sdk/target/release/cargo-b173 b173 build --path .
../../tools/rewrite/modtest.sh install target/aether-0.1.0.bmod
../../tools/rewrite/modtest.sh server
../../tools/rewrite/modtest.sh cmd loader map 2 0 0 8 aether.png
```

Console dev tools (`loader ...`): `gen`, `map`, `column`, `fill`, `tp`, `use`, `entities`. Aether is dimension 2 on a fresh world.
Teleporting a test player into terrain suffocates it: check `loader column` first.

## Done (milestone 1, verified on a dedicated server and a multiplayer client)

- Blocks: holystone set, aether dirt and grass (spreads, dies under cover), enchanted grass, quicksoil (slippery), quicksoil glass,
  icestone, ambrosium/zanite/gravitite ores and enchanted gravitite, zanite block, aerogel, cold/blue/golden aerclouds (slow fall,
  blue launches), skyroot/golden oak logs, planks, leaves (decay through loader leaves components), crystal and crystal fruit leaves,
  berry bush, flowers, saplings (grow Aether trees), ambrosium torch, dungeon stones and lit variants, pillars, skyroot bookshelf
  and fence, Aether portal.
- Items: materials, ambrosium shard (enchants grass), foods, skyroot/holystone/zanite/gravitite/valkyrie tools, swords,
  zanite/gravitite/neptune/obsidian/phoenix/valkyrie armor and sentry boots (tinted sheets).
- Recipes from AetherRegistries that use ported items, plank and charcoal recipes.
- Dimension: ChunkProviderAether terrain, surfaces, quicksoil, large cold aerclouds, AetherBiomeDecorator (clouds, crystal
  islands, ores, flowers, berry bushes, skyroot and golden oak trees, lakes), sky and fog colors.
- Portal: water in a glowstone frame lights it, 80 tick travel both ways, arrival portal built or reused, no bounce back.

## Done (milestone 2, entities, verified alive and moving on a dedicated server)

- Models: `tools/convert_models.sh` regenerates `src/models/generated.rs` from the decompiled models plus `reference/vanilla/*.java`
  (pig, pig saddle, slime, moa and flying cow saddle variants written out by hand). Animation is hand ported into each entity's
  `pose` draw program.
- Mobs: aerbunny (hops, rides the player's head, slow fall and double jump), sheepuff (fleece colors, shearing, dyeing, grazing,
  puff and float), phyg and flying cow (wings, saddles, mod-side flight on `onTravel` with one air jump), aerwhale (course AI), moa (four types, eggs,
  feeding babies, sitting, type jumps and speed), cockatrice (needles), zephyr (snowballs, charge squash, overlay), swet (hunts,
  swallows, three bounces, bursts), aechor plant (rooted, needles, size, sway).
- Projectiles: golden, poison and enchanted darts, poison needle, zephyr snowball. Poison (inebriation) damage and wobble.

## Rules from the user

- Optimize, reorganize or rewrite freely when look, feel and gameplay stay identical; fix source bugs.
- Never replace vanilla screens; extra UI goes in toggleable overlays (see accessories).

## Done (milestone 3, dungeons and bosses)

- Machines: enchanter, freezer, incubator (tile data synced, lit state, GUIs).
- Valkyrie, fire minion, floating gravitite, crystals (fire, ice, thunder, cloud).
- Bronze, silver and golden dungeons (heights compressed into 128), locked stone, traps, mimic chests, treasure chests with
  keys and loot, chest loot tables.
- Slider, Valkyrie Queen (dialogue screen, medal check over a channel), Sun Spirit (dialogue lines, fire and ice balls,
  minions), boss bar HUD, dungeon unlock on death.
- Loader fixes found on the way: tile entities on any template, mod entity tracking waits for the mod handshake, death
  screen keep-alive (vanilla 30 second timeout).

## Bugs to check at the end

- Keep-alive fix on the death screen not verified yet (test running when paused).
- Boss bar HUD not seen in a screenshot yet; Slider untested in a client; Sun Spirit arm offsets vs original unverified.
- Huge mob counts in the Aether (100+ zephyrs and aechor plants near a dungeon); check spawn caps and weights.
- Resolved: whirlwinds falling off island edges match the source (EntityMob gravity, vanilla void kill at y -64).
- Resolved: Slider crush now bounces and stops only when its damage lands, as applyEntityCollision does.
- Clippy: clean except five `module_inception` warnings (constant modules named after their file) and one complex type.
- Valkyrie and queen arm swing on attack not shown (b1.7.3 mobs have no swing).
- Sun Spirit ice slowness and crystal crashes on hard difficulty skipped.
- Repulsion shield cancels hits whose damage type is a projectile (arrow, thrown, fireball). The source asks whether
  the hit's attacker is a projectile, which for vanilla arrows is the shooter, so its cancel almost never fired; the
  port follows its intent. Deflection itself matches the source.
- Inebriation poison spits red dye icon bits through the new `iconcrack_351_1` particle; check it renders in a client.
- Life shard hearts past ten, sun altar slider, lore book pages, valkyrie wings, invisibility and record playback are
  built but not seen in a client yet.
- Beta differences, accepted: doors and signs craft one at a time (beta stacks them to one).
- Accessory dye recipe (RecipeAccessoryDyes) needs NBT colors, not ported; red, blue and yellow capes come only from
  creative or commands, as in the source.
- Client window loses focus during tests and opens the pause menu (test harness only).

## Done (milestone 4, gear)

- Accessories: 31 accessory items, eight worn slots kept in player synced data, a toggle button over the vanilla
  inventory's player preview that shows a 2x4 equipment panel beside the window (vanilla screen untouched), click to swap
  with the cursor, render layers for pendants, capes, gloves and the repulsion shield, abilities (zanite wear and mining
  speed, ice freezing, iron bubble air, agility step, golden feather and valkyrie cape glide, regeneration stone,
  phoenix gloves cooling to obsidian).
- Armor sets: gravitite jump, neptune and phoenix swimming, phoenix fire immunity and quenching into obsidian.
- Weapons: dart shooters, flaming, lightning and holy swords, vampire blade, pig slayer, candy cane sword, hammer of
  notch (projectile), lightning knife (thrown, lightning), phoenix bow (flaming arrows).
- Tools: zanite speed by wear, gravitite lifting blocks, holystone ambrosium drops.
- Skyroot buckets (fill, pour, poison, remedy, milk), aechor plant poison draws, white apple cure, parachutes, cloud
  staff with mini clouds, falling out of the Aether into the overworld sky.

## Done (milestone 5, completion)

- Recipes: every `AetherRegistries.initializeRecipes` and `initializeShapelessRecipes` row (slabs, walls, stairs, gloves,
  pendants, darts, machines, bed, bookshelf, parachutes, lore book), metadata keys, any-color wool for the bed.
- Enchanter and freezer tables completed (rings, pendants, darts, discs, vanilla armor and hoes; the source repairs the
  iron sword at 550), skyroot water bucket leaves its bucket.
- Sun altar block and slider screen, eternal day (noon until the Sun Spirit falls, then the sky walks back to the world
  time), op check with `gameplay.sun_altar_multiplayer`, `gameplay.disable_eternal_day`.
- Life shards (`gameplay.max_life_shards`, hearts past ten in rows), Book of Lore container screen with all 209 lore
  entries, four music discs with their artists, first trip into the Aether gives a lore book and golden parachute.
- Valkyrie set flight and wings, invisibility cape, repulsion shield deflection, obsidian set halves damage.
- Skyroot tools double drops of grown holystone, dirt, grass, quicksoil, logs and ores; strong axes free golden amber
  from golden oak; berry bushes drop their berries again (the port dropped none).
- Aether music: the four Aether tracks replace vanilla music in the dimension, a minute to a minute and a quarter
  apart (loader `DimensionSpec.music`).
- Golden oak, crystal and holiday leaf motes and Aether portal motes and hum (loader `tintportal_RRGGBB` particle).
  The source has no lit machine faces, only smoke and flame, already ported.
- Inebriation follows PotionInebriation: gaussian sway and view shake on the poisoned player's own client, aechor
  plants hold two to five poison draws.

## Done (config)

- `src/config.rs`: `config/aether.toml` with the AetherConfig options this port has features for, editable in the mod menu:
  `loot.valkyrie_cape` (on) and `loot.golden_feather` (off, the Java default; the port had it on) in silver dungeon loot,
  `spawn_rates.<mob>` for the eleven mobs with the Java defaults (zephyr 65, aechor plant 55, whirlwind 55, swet 40,
  cockatrice 45, the rest 1). The missing spawn rate roll is the likely cause of the huge zephyr and aechor plant counts; not
  verified in game yet.
- Later: `gameplay.max_life_shards`, `gameplay.sun_altar_multiplayer`, `gameplay.disable_eternal_day`.
- Not ported yet, add their options with the features: Aether menu and menu button, trivia, holiday trees, tall grass,
  Sun Spirit dialogue repeat, legacy visuals, legacy altar name, portal frame start, first-entry gifts toggle.

## Done (SDK shared math)

- Block positions, distances, face offsets and faceEntity turns use `b173_api::math` (`blockOf`, `distance`,
  `Face`, `yawTowards`). Turns now divide by float pi as EntityLiving.faceEntity does; the earlier f64 `to_degrees`
  differed in the last bits.
- Left on their own math: dungeon and island generation, the f32 yaw quarter in `shapes.rs`, and angles the source
  wrote by hand with other constants (whirlwind and zephyr turns, sun spirit rotary, valkyrie lunges).

## Done (loader API 1.25, achievements)

- `mod.toml` needs API 1.25. AchievementsAether's ten achievements are loader mod achievements (`src/progress.rs`),
  each with the source's icon, en_US title and description, and `aether:enter_aether` as every other one's parent.
- Unlocked where the source triggers them: dimension change into the Aether (`enter_aether`), the Slider's killer and
  players within ten blocks of the dungeon point, the Valkyrie Queen's target, the Sun Spirit's target and players within
  20 x 3 x 20 of its origin, crafting the altar or a gravitite shovel, axe or pickaxe, bouncing on a blue aercloud,
  riding a phyg, a hatching incubator's owner, and a book of lore placed in the lore book's slot.
- A first unlock plays the source's achievement jingle (bronze and silver variants for those bosses) at the player.
  It is a world sound, so players nearby hear it too, where the source played it only to that player's client.
- The incubator owner is the last player who opened it, kept in memory as the source did; the source took the player
  who put the egg in. The loader has no slot callback, so the lore book slot is polled each tick for players who
  opened the book; crafting a book of lore in the inventory grid after closing it without one can also unlock it.
- The welcome gifts still follow the `entered_aether` player flag, not the achievement.

## Done (loader API 1.23, damage types, attributes, effects and held use)

- Regeneration and weakness are loader effects beside inebriation (`src/effects.rs`): the healing stone gives 610
  ticks of regeneration on eating, the regeneration stone eighty ticks whenever a hurt wearer has none, and ice
  and cloud crystals ten ticks of weakness, as the source's potions.
- The full phoenix set cancels fire damage (in_fire, on_fire, lava, fireball) as PlayerAetherEvents does, and the
  Slider only takes a player's own blow, never a projectile, magic, explosion or fire hit, as EntitySlider.
- Poison, remedy and milk skyroot buckets are drunk over 32 ticks with the drink pose (ItemSkyrootBucket). The remedy
  and the white apple cure inebriation, milk clears regeneration and weakness but not inebriation, as the source's
  curative items; the port had milk curing inebriation. The remedy's cure vignette overlay is not ported.
- The zanite sword hits harder the more worn it is (ItemZaniteSword's 3 to 7 against the iron sword's 6), added to
  a player's own melee blow in LivingHurt since the loader has no per stack attack attribute.
- Golden oak, crystal and holiday leaf motes and whirlwind puffs are loader mod particles (`src/particles.rs`) with
  the source's colours, sizes and lifetimes. A whirlwind's own tint and rainbow puffs are one fixed grey; portal
  motes keep the loader `tintportal_RRGGBB` particle for their random brightness.
- Fixed mob drops are entity spec drops as dropFewItems: aechor petals and the Slider's seven to nine carved stone
  moved off `onDeath`, the zephyr now drops its cold aercloud (the port dropped nothing) and the moa three
  feathers (the port dropped none to two). The bosses' dropFewItems drops are spec drops too, so recipe viewers list
  them through `lootList()`: the Slider's bronze key, the Valkyrie Queen's silver key and golden sword, the Sun
  Spirit's golden key and sun altar. They now honour the mob drops game rule like the source and spawn at the boss's
  feet instead of the source's half block lift (a spec drop has no height offset).
- State dependent drops are loot pools with loot conditions (API 1.42), so recipe viewers list them: sheepuff wool
  (`aether:unsheared` and one `aether:fleece/<colour>` condition per wool damage), the swet's glowstone or blue
  aercloud and swet balls (`aether:golden_swet`, one `aether:swet_half` count rule per death so every count agrees),
  and the saddles of phygs, flying cows (`aether:saddled`) and moas (`aether:moa_saddled`). They roll from the
  entity's random instead of the world's. ChunkProviderAether.populate's seventeen steps are named placed features
  (`aether:dungeons`, the three aerclouds, crystal islands, ores, flowers, trees, dungeon oaks, lakes, bronze
  dungeons) the dimension's generator places in order on its chunk random, so the sequence is unchanged and each can be
  turned off. The highlands biome is in `aether:is_aether` and `c:is_magical`. Dungeon chest loot (`src/world/dungeons/loot.rs`) also stays in
  code: each pick is one weighted outcome with nested rolls, and `rollLoot` rolls every entry on its own, so a chest
  table would change the odds.
- Aether grass joins the `minecraft:flower_soil` tag, so flowers and saplings stay on it as BlockAetherGrass allows.
- The phoenix bow draws while the use key is held (bow pose, the three `bow_pulling` icons after 0, 13 and 17
  ticks) and looses its flaming arrow on release at three times the draw power, as ItemPhoenixBow. Power, Punch,
  Flame and Infinity and the full draw critical have no beta counterpart.
- Inebriation is the loader status effect `aether:inebriation` (PotionInebriation's colour and inventory icon, one
  magic damage every fifty ticks), saved with the entity and shown on the HUD. Its effect behavior's `onTick`
  sways mobs and spits on the server and sways the poisoned player on that player's own client, so poisoned entities
  are no longer tracked in the mod; the custom poison channel is gone.
  The source's "was inebriated" death message has no beta counterpart. HUD icon not seen in a client yet.
- Life shard hearts are an `aether:life_shards` add modifier on `minecraft:max_health`, as the source's health
  attribute modifier, set again only when the shard count changes. Older saves that raised the base health get it
  reset to twenty once.
- `mod.toml` needs API 1.23. Hits carry the vanilla damage type closest to the source's DamageSource (`src/damage.rs`):
  darts, needles and flaming arrows as arrows, crystals as thrown, zephyr snowballs as fireballs, mob, boss, slider,
  sentry, pig slayer and hammer hits as mob, the holy sword's undead bonus as drown. Custom source names (incineration,
  crush, dart) have no beta type and take the nearest one.

## Done (loader API 1.10, structure registry)

- `mod.toml` needs API 1.10. The dungeons are loader structure kinds `aether:bronze_dungeon`, `aether:silver_dungeon` and
  `aether:golden_dungeon` (`world/dungeons/mod.rs` `KINDS`). Silver and golden starts are recorded with their component
  box when they build into a chunk and have finders on `canSpawn`, so `/locate` finds them before they generate. A bronze
  dungeon is recorded with every room and tube `Writer` built (`m_rooms`) once its boss room is placed.
- Generation is unchanged: recording draws no random numbers.

## Done (loader API 1.1, ABI 3)

- `mod.toml` needs API 1.1. The loader still accepts ABI 2 builds of this port.
- Named block states, so worlds save names instead of bare metadata:
  - `lit` on the enchanter, freezer and incubator, in `machines.rs` `registerBlock`.
  - `placed` on aether grass, aether dirt, holystone, quicksoil and both logs, in `harvest.rs` `registerStates`.
  - Each property lists its values in metadata order, so old saves keep their meaning. Since API 1.39 the code reads
    and writes the names too (see Names, tags and states).
- Synced player data now gets its own `aether` slot on every entity. Another mod syncing data on the player can no longer overwrite it. Old saves hand their single blob to the first mod that reads it, which is Aether.
- Not changed:
  - Stack damage is only durability here, so no data components were needed.
  - World data was already per mod.
  - The `thread_local` maps are runtime AI state, not saved data.
- Not verified in game: chunk palettes with these states, and an old world loading with this build.

## Audit against 1.1.2.5 (2026-11-11)

Checked one to one with `reference/src`: every ItemsAether and BlocksAether registration and every entity; tool and
sword materials, damage, speed, harvest level and durability (beta's sword formula, as for vanilla beta swords); zanite
wear bands for speed and attack; gravitite lift, holystone shards, elemental, vampire, pig slayer and candy cane hits;
armor values; darts, phoenix bow and lightning knife; present crates; slider block crushing; machine particles; every
`aether_legacy` sound key.

Fixed:
- Presents set off a primed TNT present (`tnt_present`, a moving block drawing the present) that pops up, falls,
  bounces, smokes for ten ticks and explodes at power 0.4; before, a power 1 blast went off at once. The candy cane
  sword chance belongs to the gingerbread crate only.
- Phoenix arrows trail flames, two a tick in flight and one every fifth tick once stuck.
- Darts and phoenix arrows play bowhit where they strike or stick, slow to 0.8 drag with bubbles in water and put out
  their fire, and a player who shot one picks it back up once it stops quivering (seven ticks) if it fits, with the
  pop sound.
- The slider drops the blocks it crushes, as destroyBlock calls dropBlockAsItem.
- The lightning knife plays the mod's own `projectile.shoot` sound instead of the bow's.

Still differs or unverified:
- Valkyrie reach rides the loader's `block_interaction_range` and `entity_interaction_range` attributes (rewrite
  3503004): the pickaxe, axe and shovel reach eight blocks for blocks, they and the lance point at entities eight
  blocks off. The source's extended attack picks the nearest entity in a narrow cone by line of sight and wears the
  item one more; the port uses the normal pick ray and wear.
- A dart pickup has no item fly animation (beta has no creative players, so the source's pickup mode 2 never
  applies).
- The slider's crush shows the break effect and smoke the source spawns only on the server, where 1.7 never draws
  them.
- Everything above is checked by build and code review only, not in a game.

Second pass (every entity, ability and machine class against the port by its numbers and logic):
- Fixed: the enchanter turns skyroot poison buckets into remedy buckets (1000 ticks); zephyrs, sentries and mini
  clouds have 1.7's default 20 health; sentries never fall asleep again once awake, search every ninth tick and keep
  a target they can see within 16 blocks, with a random hop pitch; whirlwinds that swallow another turn evil with
  half their life and fizz, tear leaves with mob griefing on, lose 50 life for each entity they hold while inside a
  block, and spit creepers with a little drift; aerbunny jumps puff five clouds spread over the bunny; a moa jumping
  by itself rises 0.05 more each tick.
- Phygs, flying cows and moas seat their rider at 0.65, 1.15 and 1.25 (getMountedYOffset) through the loader's
  api 1.36 `EntityField::Seat`, set each tick on both sides.
- Checked and matching: every other mob's health, size, speed and attack, the valkyrie (whose teleport never
  succeeds in the source), queen, slider, sun spirit (its body parts only take player hits, which it ignores), fire
  minion, crystals, armor abilities, repulsion, accessories and the freezer table.

## Done (loader API 1.39, names, tags and states)

The port no longer keeps numeric ids or reads metadata bits. `ids.rs` is gone, `names.rs` holds registry names and
the state helpers, `tags.rs` holds every tag the port adds.
- Identity by name: `world.is`, `ctx.is`, `blockIs`, `ItemRef::is`; placing with `world.place`, a `StateHandle`, or
  `GenWorld::put`/`set` with a `Pick` (a name plus named properties). World generation and dungeons keep one handle
  per pick, resolved once per content epoch, so a chunk writes numbers without a host lookup per block.
- Groups by tag (`tags.rs`):
  - `aether:soil` (aether grass, enchanted aether grass, aether dirt) decides where plants, saplings and trees root,
    replacing `Palette::isSoil`. The same blocks join `minecraft:dirt`.
  - `aether:amber_axes` (zanite, gravitite and valkyrie axes, diamond axe) chip golden amber from golden oak.
  - `aether:gravitite_tools` grants the gravitite tools achievement, `aether:valkyrie_tools` gets the long reach.
  - `aether:dungeon_stone` is BlockDungeonBase, the stone the slider never crushes.
  - Entity tags `minecraft:undead` (zombie, skeleton, pig zombie) for the holy sword and `aether:pigs` (pig, pig
    zombie, phyg) for the pig slayer. The source matched "pig", "phyg" and "taegore" in the entity name; another
    mod's pig now counts when it joins the tag. Fixed: the holy sword missed zombie pigmen, the old list named
    `zombie_pigman` where beta's id is `pig_zombie`.
  - Aether content in the shared tags: tools in `minecraft:pickaxes`, `axes`, `shovels`, swords and blades in
    `minecraft:swords`, armor in `minecraft:head_armor` and the other slots, logs, leaves, planks, saplings and
    flowers as items and blocks, ores in `c:ores` and `c:ores/ambrosium`, `zanite`, `gravitite`, `c:gems/zanite`,
    `c:storage_blocks/zanite`, skyroot sticks in `c:rods/wooden`.
- Variants by state:
  - Aether grass gains `dungeon` before `placed`: the golden dungeon's tree spots (the source's metadata 3) are
    `dungeon=true,placed=true`, and only grass with both false doubles under a skyroot shovel.
  - Stairs `facing` (east, west, south, north), the skyroot fence gate `open` and `facing`, the skyroot bed `head`,
    `occupied` and `facing`, all in metadata order so placed blocks keep their shape.
  - Reads go through `ctx.stateAt()`, so harvest code sees the state of a block already removed.
- Vanilla textures by sprite name: the dungeon chest faces are `minecraft:block/chest_*`.
- A floating block carries the named state it lifted instead of an id and metadata. One in flight while the world
  saves in an older build of the port lands as nothing.

Left numeric, by the loader's design:
- Air is id 0 (`isAir`, `block == 0`).
- Block model cuboids pick states by index (`Cuboid.metas`), which equals the metadata order above.
- `dropBlockItems`, `breakEffect`, `blockInfo` and `scheduleTick` take the id and metadata a callback or a world read
  just handed over, and the engine's `EntityField::Block` takes a block id for drawing (`names::numeric`).
- Beta wool and trapdoors have no named states, so blue wool and the boss trapdoors set the loader's `meta` property.
- Terrain fills the chunk buffer with ids resolved from names once per chunk.

Name, tag and state checks hit the SDK's per epoch caches without allocating, so ticks and generation call them
freely. Not verified in game: every change above is checked by build and code review only.

## Done (loader API 1.40, every subsystem through loader registries)

| Subsystem | Registers | Where |
|---|---|---|
| Blocks, items, block states | `ctx.blocks()`, `ctx.items()`, named states | `blocks.rs`, `items.rs`, `shapes.rs`, `harvest.rs`, `machines.rs` |
| Tags | `ctx.tags()`: soil, amber axes, gravitite and valkyrie tools, dungeon stone, dungeon loot rarity, undead and pig entities, shared vanilla and `c:` tags, `c:accessories/*` | `tags.rs`, `accessories.rs`, `items.rs` |
| Crafting | `ctx.recipes()` | `recipes.rs` |
| Enchanter, freezer, incubator | machine jobs and fuels (`ctx.machines().job/fuel`), run with `matchJob` and `machineFuel`, so other mods add enchanter or freezer work and fuels | `machines.rs` |
| Container leftovers | item `container_item` (skyroot buckets), read with `containerItem` | `items.rs`, `machines.rs` |
| Dungeon chest loot | weighted chest pools `aether:bronze_room`, `bronze_treasure`, `silver_room`, `silver_treasure`, `gold_treasure`; each weight gives the source's nested odds exactly; a pick draws one seed from the generator, so a seed builds other chests than before | `world/dungeons/loot.rs` |
| Accessories | eight loader accessory slots (`aether:pendant`, `cape`, `shield`, `misc`, `ring`, `second_ring`, `gloves`, `second_misc`) taking `c:accessories/<kind>`, saved and synced by the loader; the panel overlay swaps through `swapAccessory`. Old saves move their slots into the loader on first read | `accessories.rs` |
| Dimension, biome, terrain | `ctx.dimensions()` | `world/mod.rs` |
| Dungeons | `ctx.structures()` | `world/dungeons/mod.rs` |
| Spawns | `SpawnSpec` on each entity | `entities/mod.rs` |
| Effects, particles, achievements, config, text | `ctx.effects()`, `ctx.particles()`, `ctx.achievements()`, `ctx.config()`, lang file | `effects.rs`, `particles.rs`, `progress.rs`, `config.rs` |

Kept in the port on purpose: ores stay in the Aether decorator, since moving them to `ctx.worldgen()` would change the
random order that places clouds, plants and trees after them. The silver treasure's cape and feather follow the
config as it stands when the game starts. Not verified in game.

## Todo, in order

1. Aether main menu and menu button (config, off by default in the source).
2. Bug pass (list above), cargo fmt and clippy, client screenshots of the panel, layers, boss bar, wings, hearts and
   the lore book.
