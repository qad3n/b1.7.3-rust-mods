# Roughly Enough Items port status

Source: RoughlyEnoughItems-16.0.799-fabric.jar (`reference/src`, Vineflower decompile, never committed).
Target: b1.7.3-rewrite mod loader, one `rei-<version>.b173mod` for client and server. Needs loader API 1.2.
Assets: never committed, `python3 -I tools/extract_assets.py <jar>` copies the lang lines the port uses into `assets/rei`.

Not a 1:1 port: only the core concept, rebuilt small on the loader's overlay API. REI's plugin system, entry types,
favorites, config screens, transfer handlers, display categories other than crafting and smelting, and the weather,
time and gamemode buttons are left out.

## Controls

| Input | Does |
|---|---|
| Left click an item | recipes making it, or with cheats on a full stack on the cursor |
| Right click an item | recipes using it, or with cheats on one item on the cursor |
| Click the list holding another item | deletes the held stack (cheats on) |
| `R` / `U` over any item | recipes / usages, vanilla slots included |
| Wheel over the list / recipe page | page / recipe |
| `Esc` / `Backspace` on a recipe page | close / back |
| `O` | hides or shows the panel |
| Search field | words match the name, `@word` the mod id, right click clears |
| Crafting table button | craftable filter, from the 36 inventory slots |
| Cheat button | asks the server, which grants it only to ops and checks op again on every grab and delete |

## Done

- [x] Item list beside every vanilla container screen (`REIRuntimeImpl`, `EntryListWidget` concept), paged grid sized to the screen
- [x] Search filter with name words and `@modid` (`SearchFilter` concept)
- [x] Craftable filter (`CraftableFilter`), greedy ingredient match with exact damages first
- [x] Recipe and usage pages for crafting, shapeless and smelting (`DefaultCraftingDisplay`, `DefaultCookingDisplay`), history with back, wildcard ingredients cycle their subtypes
- [x] Cheat mode, grab method (`RoughlyEnoughItemsNetwork` create item and delete item packets), op checked server side on toggle, grab and delete
- [x] Loader API 1.2: overlay key and wheel input, `itemCatalog`, `recipeList`, `contentGeneration`, `screenSize`
- [x] Items and recipes come from the loader registries, so every mod's items and its recipes (`ctx.recipes()` or `data/<modid>/recipes/*.toml`) appear with no REI plugin. Recipe pages show the recipe registry name. The cache reloads on each content generation (remap, server recipe sync)
- [x] Build, clippy and fmt clean for the mod and the rewrite

## Bugs to check at the end

- [ ] Live check: none run yet. Deploy, open the inventory, search, craftable, recipe pages, cheat as op and non-op on a dedicated server
- [ ] Block items drawn by overlay `DrawOp::Item` may render without item lighting (flat) since the overlay draws after the container resets GL state
- [ ] Tooltip over 3D block items may be hidden behind them if depth test is on during overlay draw
