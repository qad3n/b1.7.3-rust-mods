# Aether port status

Source: Aether Legacy 1.7.10 v1.1.2.5 (`reference/src`, Vineflower decompile of the user's jar, SRG names).
Target: b1.7.3-rewrite mod loader, one `aether-<version>.b173mod` for client and server.
Assets: never committed, `python3 -I tools/extract_assets.py <jar>` fills `assets/aether`.

## Build and test

```bash
cargo build --release --target wasm32-unknown-unknown
../../b1.7.3-rewrite/vendor/b1.7.3-modding-sdk/target/release/cargo-b173 b173 build --path .
../../tools/rewrite/modtest.sh install target/aether-0.1.0.b173mod
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
  puff and float), phyg and flying cow (wings, saddles, steering with one air jump), aerwhale (course AI), moa (four types, eggs,
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
- Repulsion shield damage cancel is a heuristic: b1.7.3 blames a projectile hit on its shooter, so hits from farther
  than 3.5 blocks count as projectiles. Deflection itself matches the source.
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

## Todo, in order

1. Aether main menu and menu button (config, off by default in the source), achievements.
2. Bug pass (list above), cargo fmt and clippy, client screenshots of the panel, layers, boss bar, wings, hearts and
   the lore book.
