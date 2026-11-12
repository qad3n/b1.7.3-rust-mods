# Roughly Enough Items port status

Source: RoughlyEnoughItems-16.0.799-fabric.jar (`reference/src`, Vineflower decompile, never committed).
Target: b1.7.3_rewrite mod loader, one `rei-<version>.bmod` for client and server. Needs loader API 1.33.
Assets: never committed, `assets.toml` copies the lang lines the port uses from the player's jar in `mods/rei_assets/`.

Not a 1:1 port: only the core concept, rebuilt small on the loader's overlay API. REI's plugin system, entry types,
favorites, config screens, transfer handlers, and the weather,
time and gamemode buttons are left out.

## Controls

| Input | Does |
|---|---|
| Left click an item | recipes making it, or in give mode a full stack on the cursor |
| Right click an item | recipes using it, or in give mode one item on the cursor |
| Click the list holding another item | deletes the held stack (give mode) |
| `R` / `U` over any item | recipes / usages, vanilla slots included |
| Wheel over the list / recipe page | page / recipe |
| `Left` / `Right` arrow keys | recipe on a recipe page, else list page |
| `Esc` / `Backspace` / Back button on a recipe page | close / back to the page an ingredient was opened from, else close. Any screen change closes the page, so the inventory always reopens on the plain container |
| `O` | hides or shows the panel |
| Search field | words match the name, `@word` the mod id or name, right click clears |
| Recipe page tooltips | name, owning mod, registry id and every item and block tag, a drop's count, chance and conditions |
| Tabs left of a recipe page | switch category: Crafting, Smelting, Mob Drops, Block Drops, Chest Loot, Tags. `R` shows how to get an item, `U` what it makes, what a block or spawner drops, a chest's loot |
| Crafting table button | craftable filter, from the 36 inventory slots |
| `R` button | REI's cheat toggle as recipe mode and give mode, the R turns red in give mode and its tooltip names the mode. The server grants it only to ops and checks op again on every grab and delete |

## Done

- [x] Item list beside every vanilla container screen (`REIRuntimeImpl`, `EntryListWidget` concept), paged grid sized to the screen
- [x] Search filter with name words and `@mod` id or name (`SearchFilter` concept)
- [x] Craftable filter (`CraftableFilter`), greedy ingredient match with exact damages first
- [x] Recipe and usage pages for crafting, shapeless and smelting (`DefaultCraftingDisplay`, `DefaultCookingDisplay`), history with back, wildcard ingredients cycle their subtypes
- [x] Cheat mode, grab method (`RoughlyEnoughItemsNetwork` create item and delete item packets), op checked server side on toggle, grab and delete
- [x] Loader API 1.2: overlay key and wheel input, `itemCatalog`, `recipeList`, `contentGeneration`, `screenSize`
- [x] Items and recipes come from the loader registries, so every mod's items and its recipes (`ctx.recipes()` or `data/<modid>/recipes/*.toml`) appear with no REI plugin. Recipe pages show the recipe registry name. The cache reloads on each content generation (remap, server recipe sync)
- [x] Mod name line: `§9§o` mod name under every item tooltip, REI list and vanilla slots (API 1.4 tooltip hook), behind the client option `tooltip.mod_name` (default on, mod menu "Mod Name")
- [x] Vertical category tabs on recipe pages (REI's category tabs, stacked down the left side): Crafting, Smelting, Mob Drops, Block Drops, Chest Loot, Tags, only those with pages for the item
- [x] Mob, block and chest loot from the loader's `lootList()` (API 1.33): beta mobs, every block's drops per metadata, the dungeon chest, mod spec drops and every mod loot table. Drops show count range, chance (`~` when measured from block code) and `*` for a condition the tooltip spells out. Drops rolled in a mod's own code are not listed until that mod declares a table
- [x] Tag pages: every item and block tag of the item with all members, members clickable
- [x] Loader API 1.5: the recipe page reports it covers the container, so vanilla slots under it show no highlight, tooltip or key lookup stack
- [x] Mob spawners: one per living entity in the loader entity registry (API 1.31 `mobList`), vanilla and every mod's mobs, damage = entity id as old item viewers did. Placing one sets the spawner's mob on the authority, vanilla slot tooltips name the mob. Beta difference: a multiplayer client mirrors spawners as pigs since beta never syncs the mob
- [x] Build, clippy and fmt clean for the mod and the rewrite

## Loader API 1.48 to 1.55 (2026-11-11)

- Builds against the current SDK unchanged. It registers no blocks, entities or loot of its own.

## Bugs to check at the end

- [ ] Live check: none run yet. Tabs, loot pages and tag pages unseen. Deploy, open the inventory, search, craftable, recipe pages, cheat as op and non-op on a dedicated server
- [x] Overlay items lit and depth tested as vanilla slots, tooltips (REI and vanilla slot) drawn over every overlay
- [x] Vanilla 20 high buttons and panels, search field, craftable and `R` square buttons in one row under the list
