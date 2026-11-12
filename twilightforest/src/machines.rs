use crate::blocks::{texture, vanillaTexture};
use crate::names::{self, block};
use crate::{recipes, tags};
use b173_api::prelude::*;
use b173_api::screen::Draw;

// ContainerTFUncrafting on a tile inventory: the input, the result, nine assembly slots, and the uncrafted
// ingredients kept in tile data and drawn as ghosts over the assembly grid
mod table
{
    pub const INPUT: i32 = 0;
    pub const RESULT: i32 = 1;
    pub const ASSEMBLY: i32 = 2;
    pub const CELLS: i32 = 9;
    pub const SLOTS: i32 = ASSEMBLY + CELLS;
    pub const HARDNESS: f32 = 2.5;
    pub const CHANNEL: &str = "twilightforest:uncraft";
}

mod gui
{
    pub const WIDTH: i32 = 176;
    pub const HEIGHT: i32 = 166;
    pub const TEXT: u32 = 0x404040;
    pub const INVENTORY_LABEL_Y: i32 = 72;
    pub const INVENTORY: [i32; 2] = [8, 84];
    pub const TABLE_INPUT: [i32; 2] = [13, 35];
    pub const TABLE_RESULT: [i32; 2] = [147, 35];
    pub const ASSEMBLY: [i32; 2] = [62, 17];
    pub const CELL: i32 = 18;
    pub const ITEM: i32 = 16;
    // GuiTFGoblinCrafting greys ghost ingredients and tints broken ones red
    pub const GHOST: u32 = 0x9F8B8B8B;
    pub const BROKEN: u32 = 0x80FF8B8B;
    pub const FURNACE_INPUT: [i32; 2] = [56, 17];
    pub const FURNACE_FUEL: [i32; 2] = [56, 53];
    pub const FURNACE_OUTPUT: [i32; 2] = [116, 35];
    // GuiTFCinderFurnace draws the flame and the arrow from the right of its sheet
    pub const FLAME: [i32; 4] = [56, 36, 176, 0];
    pub const ARROW: [i32; 4] = [79, 34, 176, 14];
    pub const FLAME_WIDTH: i32 = 14;
    pub const FLAME_SCALE: i32 = 13;
    pub const FLAME_BOTTOM: i32 = 12;
    pub const ARROW_LENGTH: i32 = 24;
    pub const ARROW_HEIGHT: i32 = 16;
}

const TABLE_NAME: &str = "uncrafting_table";
const TABLE_TITLE: &str = "Uncrafting Table";
const FURNACE_NAME: &str = "cinder_furnace";
const FURNACE_LIT: &str = "cinder_furnace_lit";
const FURNACE_TITLE: &str = "Cinder Furnace";
// the block keeps its 2.4.3 name, the screen the furnace title
const FURNACE_LABEL: &str = "Cinder Furnace [NYI]";
const FURNACE_SHEET: &str = "minecraft:gui/furnace";
const TABLE_SHEET: &str = "twilightforest:gui/guigoblintinkering";
const STICK: &str = "minecraft:stick";

#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
struct Uncraft
{
    // the input this grid was made for, the ingredients, and how many input items one uncraft eats
    m_input: Option<StackData>,
    m_grid: Vec<Option<StackData>>,
    m_uses: i32,
    // the result the table placed and whether it recrafts the input
    m_result: Option<StackData>,
    m_combined: bool,
}

fn raw(stack: &Option<ItemStack>) -> Option<StackData> { stack.as_ref().and_then(|stack| stack.toRaw().ok()) }

fn cooked(stack: Option<StackData>) -> Option<ItemStack> { stack.map(ItemStack::fromRaw) }

fn named(stack: &ItemStack) -> String { stack.item.name().map(|name| name.path().to_owned()).unwrap_or_default() }

// ContainerTFUncrafting.isValidMatchForInput, a recraft keeps the kind of tool or the armor slot
fn kind(stack: &ItemStack) -> Option<&'static str>
{
    let name: String = named(stack);
    ["pickaxe", "axe", "shovel", "hoe", "sword", "bow", "helmet", "chestplate", "leggings", "boots"]
        .into_iter()
        .find(|kind| name == *kind || name.ends_with(&format!("_{kind}")))
}

// Item.hasContainerItem, buckets and any mod item that leaves something behind never come out of the table
fn problematic(stack: &StackData) -> bool { containerItem(&ItemStack::fromRaw(stack.clone())).is_some() }

// ContainerTFUncrafting.getRecipeFor, the first shaped recipe that makes the input
fn recipeFor(input: &ItemStack) -> Option<(i32, Vec<Option<StackData>>, i32)>
{
    let id: i32 = input.item.id().ok()?;
    let wears: bool = maxDamage(&input.item) > 0;

    recipeList().into_iter().find_map(|recipe| match recipe.shape
    {
        RecipeShape::Shaped { width, inputs }
            if recipe.output.id == id
                && input.count >= recipe.output.count
                && (wears || recipe.output.damage == input.damage) =>
        {
            Some((width, inputs, recipe.output.count))
        }

        _ => None,
    })
}

// ContainerTFUncrafting.onCraftMatrixChanged for the input, its ingredients one each, worn parts lost
fn uncraftGrid(input: &ItemStack) -> (Vec<Option<StackData>>, i32)
{
    let mut grid: Vec<Option<StackData>> = vec![None; table::CELLS as usize];

    let Some((width, inputs, uses)) = recipeFor(input)
    else
    {
        return (grid, 0);
    };

    let height: i32 = inputs.len() as i32 / width.max(1);
    for y in 0..height
    {
        for x in 0..width
        {
            grid[(x + y * 3) as usize] = inputs[(x + y * width) as usize].as_ref().map(|part| StackData
            {
                id: part.id,
                count: 1,
                damage: part.damage.max(0),
            });
        }
    }

    let damageable = |part: &Option<StackData>|
    {
        part.as_ref().is_some_and(|part| !b173_api::named(RegistryKind::Item, part.id, STICK))
    };
    let most: i32 = maxDamage(&input.item);
    if most > 0 && input.damage > 0
    {
        let parts: i32 = grid.iter().filter(|part| damageable(part)).count().max(4) as i32;
        let mut broken: i32 = (parts as f32 * input.damage as f32 / most as f32).ceil() as i32;

        for part in grid.iter_mut().filter(|part| damageable(part))
        {
            if broken <= 0
            {
                break;
            }

            if let Some(part) = part
            {
                part.count = 0;
            }

            broken -= 1;
        }
    }

    for part in grid.iter_mut().flatten()
    {
        if problematic(part)
        {
            part.count = 0;
        }
    }

    (grid, uses)
}

fn slots(world: World, pos: BlockPos, first: i32, count: i32) -> Vec<Option<ItemStack>>
{
    (first..first + count).map(|slot| world.tileSlot(pos, slot)).collect()
}

fn setSlot(world: World, pos: BlockPos, slot: i32, stack: Option<ItemStack>)
{
    if let Err(error) = world.setTileSlot(pos, slot, stack)
    {
        warn(&format!("uncrafting table slot {slot}: {}", error.0));
    }
}

fn shrink(world: World, pos: BlockPos, slot: i32, by: i32)
{
    let Some(stack) = world.tileSlot(pos, slot)
    else
    {
        return;
    };

    setSlot(world, pos, slot, (stack.count > by).then(|| ItemStack { count: stack.count - by, ..stack }));
}

// SlotCrafting.onPickupFromSlot, one of every assembly item is used and buckets come back
fn useAssembly(world: World, pos: BlockPos)
{
    for slot in table::ASSEMBLY..table::SLOTS
    {
        let Some(stack) = world.tileSlot(pos, slot)
        else
        {
            continue;
        };

        let empty: Option<ItemStack> =
            raw(&Some(stack.clone())).filter(problematic).map(|_| ItemStack::of("minecraft:bucket", 1));

        if stack.count <= 1 && empty.is_some()
        {
            setSlot(world, pos, slot, empty);
            continue;
        }

        shrink(world, pos, slot, 1);
    }
}

// the result for what is in the table now, either crafted from the assembly or the input recrafted
fn result(world: World, pos: BlockPos, state: &Uncraft) -> (Option<ItemStack>, bool)
{
    let assembly: Vec<Option<ItemStack>> = slots(world, pos, table::ASSEMBLY, table::CELLS);
    let input: Option<ItemStack> = world.tileSlot(pos, table::INPUT);
    let Some(input) = input
    else
    {
        return (recipes::craft(&assembly), false);
    };
    if assembly.iter().all(Option::is_none) || state.m_grid.iter().all(Option::is_none)
    {
        return (None, false);
    }

    let combined: Vec<Option<ItemStack>> = assembly
        .iter()
        .zip(&state.m_grid)
        .map(|(placed, ghost)|
        {
            placed.clone().or_else(|| ghost.clone().filter(|part| part.count > 0).map(ItemStack::fromRaw))
        })
        .collect();
    let made: Option<ItemStack> =
        recipes::craft(&combined).filter(|made| kind(made).is_some() && kind(made) == kind(&input));
    let recraft: bool = made.is_some();
    (made, recraft)
}

// the table's tick, which notices what players took and keeps the grid and the result up to date
fn tableTick(ctx: &mut BlockCtx)
{
    if !ctx.isAuthority()
    {
        return;
    }

    let (world, pos): (World, BlockPos) = (ctx.world(), ctx.pos());
    let mut state: Uncraft = ctx.tileData();
    let taken: bool = state.m_result.is_some() && raw(&world.tileSlot(pos, table::RESULT)) != state.m_result;
    if taken
    {
        if state.m_combined
        {
            setSlot(world, pos, table::INPUT, None);
        }

        useAssembly(world, pos);
        state.m_result = None;
    }

    let input: Option<StackData> = raw(&world.tileSlot(pos, table::INPUT));
    if input != state.m_input || state.m_grid.len() != table::CELLS as usize
    {
        let (grid, uses): (Vec<Option<StackData>>, i32) = cooked(input.clone())
            .filter(|_| !crate::config::uncraftingDisabled())
            .map_or((vec![None; table::CELLS as usize], 0), |stack| uncraftGrid(&stack));
        state = Uncraft { m_input: input, m_grid: grid, m_uses: uses, ..state };
    }

    let (made, combined): (Option<ItemStack>, bool) = result(world, pos, &state);
    let placed: Option<StackData> = raw(&made);
    if placed != state.m_result
    {
        setSlot(world, pos, table::RESULT, made);
    }

    state.m_result = placed;
    state.m_combined = combined;

    let before: Uncraft = ctx.tileData();
    if state != before
    {
        let _ = ctx.setTileData(&state);
    }
}

// a click on a ghost ingredient, the player's cursor takes it and the rest fill the assembly grid
#[derive(Serialize, Deserialize, Default)]
struct Pick
{
    m_pos: BlockPos,
    m_cell: i32,
}

fn pick(message: Pick, player: Player)
{
    let Some(info) = player.info()
    else
    {
        return;
    };

    let world: World = World { dimension: info.world };
    let pos: BlockPos = message.m_pos;
    let reach: f64 = 8.0;
    let near: bool = (0..3).all(|axis| (info.pos[axis] - crate::math::center(pos)[axis]).abs() <= reach);
    if !near || !world.is(pos, TABLE_NAME) || !(0..table::CELLS).contains(&message.m_cell) || player.cursor().is_some()
    {
        return;
    }
    if slots(world, pos, table::ASSEMBLY, table::CELLS).iter().any(Option::is_some)
    {
        return;
    }
    let state: Uncraft = world.tileData(pos);
    let input: Option<StackData> = raw(&world.tileSlot(pos, table::INPUT));

    let Some(clicked) = state.m_grid.get(message.m_cell as usize).cloned().flatten().filter(|part| part.count > 0)
    else
    {
        return;
    };
    if input.is_none() || input != state.m_input
    {
        return;
    }

    for (cell, part) in state.m_grid.iter().enumerate()
    {
        if cell as i32 != message.m_cell
            && let Some(part) = part.clone().filter(|part| part.count > 0)
        {
            setSlot(world, pos, table::ASSEMBLY + cell as i32, Some(ItemStack::fromRaw(part)));
        }
    }

    shrink(world, pos, table::INPUT, state.m_uses);

    if let Err(error) = player.setCursor(Some(ItemStack::fromRaw(clicked)))
    {
        warn(&format!("uncraft cursor: {}", error.0));
    }
}

// ContainerTFUncrafting.onContainerClosed, the table keeps nothing once its user walks away
#[derive(Serialize, Deserialize, Default)]
struct Leave
{
    m_pos: BlockPos,
}

fn leave(message: Leave, player: Player)
{
    let Some(info) = player.info()
    else
    {
        return;
    };
    let world: World = World { dimension: info.world };
    let pos: BlockPos = message.m_pos;
    if !world.is(pos, TABLE_NAME)
    {
        return;
    }

    for slot in (table::ASSEMBLY..table::SLOTS).chain([table::INPUT])
    {
        if let Some(stack) = world.tileSlot(pos, slot)
        {
            setSlot(world, pos, slot, None);

            if player.give(stack.clone()).is_err()
            {
                let _ = world.spawnItem(info.pos, stack);
            }
        }
    }

    setSlot(world, pos, table::RESULT, None);
}

const LEAVE_CHANNEL: &str = "twilightforest:uncraft_leave";

struct TableTile;

#[tile_behavior]
impl TileBehavior for TableTile
{
    fn onTick(&self, ctx: &mut BlockCtx) { tableTick(ctx); }
}

struct OpensScreen
{
    m_screen: &'static str,
}

#[block_behavior]
impl BlockBehavior for OpensScreen
{
    fn onActivated(&self, ctx: &mut BlockCtx, player: Player) -> bool
    {
        if ctx.isAuthority()
        {
            let _ = player.openScreen(self.m_screen, Some(ctx.pos()), Vec::new());
        }

        true
    }
}

fn localTile<T: serde::de::DeserializeOwned + Default>(pos: Option<BlockPos>) -> Option<T>
{
    let pos: BlockPos = pos?;
    let local: EntityInfo = localPlayer()?.info()?;
    Some(World { dimension: local.world }.tileData(pos))
}

// the loader draws the screen title
fn labels(draw: &mut Draw) { draw.text(gui::INVENTORY[0], gui::INVENTORY_LABEL_Y, "Inventory", gui::TEXT); }

// GuiTFGoblinCrafting, beta has no experience so uncrafting is free and no cost is shown
struct TableScreen
{
    m_pos: Option<BlockPos>,
    m_origin: [i32; 2],
}

impl TableScreen
{
    fn cell(&self, mouse: [i32; 2]) -> Option<i32>
    {
        let x: i32 = mouse[0] - self.m_origin[0] - gui::ASSEMBLY[0];
        let y: i32 = mouse[1] - self.m_origin[1] - gui::ASSEMBLY[1];
        let span: i32 = gui::CELL * 3;
        ((0..span).contains(&x) && (0..span).contains(&y)).then(|| x / gui::CELL + y / gui::CELL * 3)
    }
}

impl Screen for TableScreen
{
    fn init(&mut self, width: i32, height: i32, pos: Option<BlockPos>, _data: &[u8])
    {
        self.m_pos = pos;
        self.m_origin = [(width - gui::WIDTH) / 2, (height - gui::HEIGHT) / 2];
    }

    fn draw(&mut self, draw: &mut Draw, _mouse: [i32; 2], _partial: f32)
    {
        labels(draw);

        let Some(state) = localTile::<Uncraft>(self.m_pos)
        else
        {
            return;
        };
        let pos: BlockPos = self.m_pos.unwrap_or_default();

        let Some(local) = localPlayer().and_then(Player::info)
        else
        {
            return;
        };

        let world: World = World { dimension: local.world };

        for (cell, part) in state.m_grid.iter().enumerate()
        {
            let Some(part) = part
            else
            {
                continue;
            };
            if world.tileSlot(pos, table::ASSEMBLY + cell as i32).is_some()
            {
                continue;
            }

            let x: i32 = gui::ASSEMBLY[0] + cell as i32 % 3 * gui::CELL;
            let y: i32 = gui::ASSEMBLY[1] + cell as i32 / 3 * gui::CELL;
            draw.item(x, y, &ItemStack { count: 1, ..ItemStack::fromRaw(part.clone()) });

            let color: u32 = if part.count == 0
            {
                gui::BROKEN
            }
            else
            {
                gui::GHOST
            };
            draw.rect(x, y, gui::ITEM, gui::ITEM, color);
        }
    }

    fn mouse(&mut self, x: i32, y: i32, button: i32)
    {
        let (Some(pos), Some(cell)) = (self.m_pos, self.cell([x, y]))
        else
        {
            return;
        };

        if button == 0
        {
            let _ = send(table::CHANNEL, Target::Server, &Pick { m_pos: pos, m_cell: cell });
        }
    }

    fn close(&mut self)
    {
        if let Some(pos) = self.m_pos
        {
            let _ = send(LEAVE_CHANNEL, Target::Server, &Leave { m_pos: pos });
        }
    }
}

// TileEntityTFCinderFurnace, a furnace that runs faster and doubles ores and logs with cinder logs around it
mod furnace
{
    pub const INPUT: i32 = 0;
    pub const FUEL: i32 = 1;
    pub const OUTPUT: i32 = 2;
    pub const SLOTS: i32 = 3;
    pub const COOK: i32 = 200;
    pub const SPEED_FACTOR: i32 = 2;
    pub const SMELT_FACTOR: i32 = 10;
    pub const CINDER_EVERY: i32 = 5;
    pub const HARDNESS: f32 = 7.0;
    pub const MAX_STACK: i32 = 64;
    pub const LIGHT: f32 = 1.0;
    // beta's wooden slab is the stone slab's third kind
    pub const WOODEN_SLAB: i32 = 2;
    pub const SLAB_FUEL: i32 = 150;
    pub const TOOL_FUEL: i32 = 200;
}

#[derive(Serialize, Deserialize, Default, Clone, Copy, PartialEq)]
struct Burn
{
    m_burn: i32,
    m_fuelTime: i32,
    m_cook: i32,
}

// the wooden tools 1.7's furnace burns and beta's does not, cinder furnace fuels in the machine registry so other
// mods add their own
const WOODEN_TOOLS: [&str; 5] = [
    "minecraft:wooden_pickaxe",
    "minecraft:wooden_axe",
    "minecraft:wooden_shovel",
    "minecraft:wooden_sword",
    "minecraft:wooden_hoe",
];

// TileEntityTFCinderFurnace.getItemBurnTime is 1.7's furnace fuel: the machine's own fuels over beta's furnace
// fuels, and the wooden slab, which beta keeps as a stone slab's metadata the fuel registry cannot name
fn burnTime(stack: &ItemStack) -> i32
{
    let slab: bool = stack.item.is("minecraft:stone_slab") && stack.damage == furnace::WOODEN_SLAB;
    if slab
    {
        return furnace::SLAB_FUEL;
    }

    machineFuel(FURNACE_NAME, stack, true)
}

// the source's logWood ore dictionary entry, any mod's logs included
fn logLike(block: i32) -> bool { blockTagged(block, tags::LOGS) && !blockIs(block, block::CINDER_LOG) }

// TileEntityTFCinderFurnace.canMultiply, ores and logs come out multiplied
fn multiplies(input: &ItemStack) -> bool { input.item.tagged("c:ores") || input.item.tagged(tags::LOGS) }

fn cinders(world: World, pos: BlockPos) -> i32
{
    let around = || -1..=1;
    around()
        .flat_map(|dx| around().flat_map(move |dy| around().map(move |dz| [dx, dy, dz])))
        .filter(|[dx, dy, dz]| world.is(pos.offset(*dx, *dy, *dz), block::CINDER_LOG))
        .count() as i32
}

// TileEntityTFCinderFurnace.getCurrentMultiplier
fn multiplier(world: World, logs: i32, factor: i32) -> i32
{
    if logs < factor
    {
        return 1;
    }

    logs / factor + i32::from(world.random(factor) < logs % factor)
}

fn canSmelt(world: World, pos: BlockPos) -> Option<ItemStack>
{
    let input: ItemStack = world.tileSlot(pos, furnace::INPUT)?;
    let output: ItemStack = recipes::smelt(&input)?;

    let Some(held) = world.tileSlot(pos, furnace::OUTPUT)
    else
    {
        return Some(output);
    };
    if held.item.id().ok() != output.item.id().ok() || held.damage != output.damage
    {
        return None;
    }

    let most: i32 = (f64::from(cinders(world, pos)) / f64::from(furnace::SMELT_FACTOR)).ceil() as i32;

    let adds: i32 = if multiplies(&input)
    {
        output.count * most.max(1)
    }
    else
    {
        output.count
    };
    (held.count + adds <= furnace::MAX_STACK).then_some(output)
}

fn smeltOne(world: World, pos: BlockPos, output: ItemStack)
{
    let Some(input) = world.tileSlot(pos, furnace::INPUT)
    else
    {
        return;
    };

    let times: i32 = if multiplies(&input)
    {
        multiplier(world, cinders(world, pos), furnace::SMELT_FACTOR)
    }
    else
    {
        1
    };
    let made: ItemStack = ItemStack { count: output.count * times, ..output };

    let total: ItemStack = match world.tileSlot(pos, furnace::OUTPUT)
    {
        Some(held) => ItemStack { count: held.count + made.count, ..held },
        None => made,
    };
    let _ = world.setTileSlot(pos, furnace::OUTPUT, Some(total));
    let _ = world.setTileSlot(
        pos,
        furnace::INPUT,
        (input.count > 1).then(|| ItemStack { count: input.count - 1, ..input }),
    );
}

// TileEntityTFCinderFurnace.cinderizeNearbyLog, a burning furnace chars a log beside it
fn cinderize(world: World, pos: BlockPos)
{
    let step = || world.random(2) - world.random(2);
    let (dx, dy, dz): (i32, i32, i32) = (step(), step(), step());
    let at: BlockPos = pos.offset(dx, dy, dz);
    if !logLike(world.block(at))
    {
        return;
    }

    // TileEntityTFCinderFurnace.getCinderMetaFor, the log turns to face the furnace
    let axis: &str = if dz == 0 && dx != 0
    {
        if dy == 0 { "x" } else { "z" }
    }
    else if dx == 0 && dz != 0
    {
        if dy == 0 { "z" } else { "x" }
    }
    else if dx == 0 || dy == 0
    {
        "y"
    }
    else
    {
        "none"
    };
    let _ = world.setBlockState(at, names::state(block::CINDER_LOG).with("axis", axis));

    world.playSound(crate::math::center(at), "fire.fire", 1.0, 1.0);

    for _ in 0..2
    {
        world.particle("flame", crate::math::center(at), [0.0; 3]);
        world.particle("smoke", crate::math::center(at), [0.0; 3]);
    }
}

fn furnaceTick(ctx: &mut BlockCtx)
{
    if !ctx.isAuthority()
    {
        return;
    }

    let (world, pos): (World, BlockPos) = (ctx.world(), ctx.pos());
    let before: Burn = ctx.tileData();
    let mut state: Burn = before;
    let wasLit: bool = state.m_burn > 0;

    if state.m_burn > 0
    {
        state.m_burn -= 1;
    }

    let fueled: bool = world.tileSlot(pos, furnace::FUEL).is_some() && world.tileSlot(pos, furnace::INPUT).is_some();
    if state.m_burn != 0 || fueled
    {
        if state.m_burn == 0
            && canSmelt(world, pos).is_some()
            && let Some(fuel) = world.tileSlot(pos, furnace::FUEL)
        {
            state.m_burn = burnTime(&fuel);
            state.m_fuelTime = state.m_burn;

            if state.m_burn > 0
            {
                let left: Option<ItemStack> = if fuel.count > 1
                {
                    Some(ItemStack { count: fuel.count - 1, ..fuel })
                }
                else
                {
                    containerItem(&fuel)
                };
                let _ = world.setTileSlot(pos, furnace::FUEL, left);
            }
        }

        match canSmelt(world, pos).filter(|_| state.m_burn > 0)
        {
            Some(output) =>
            {
                state.m_cook += multiplier(world, cinders(world, pos), furnace::SPEED_FACTOR);
                if state.m_cook >= furnace::COOK
                {
                    state.m_cook = 0;
                    smeltOne(world, pos, output);
                }
            }

            None => state.m_cook = 0,
        }
    }

    let lit: bool = state.m_burn > 0;
    if wasLit != lit
    {
        let target: &str = if lit { FURNACE_LIT } else { FURNACE_NAME };
        let swapped: Result<bool> = crate::world::palette::withPlain(target, |state| world.swapState(pos, state));
        if let Err(error) = swapped
        {
            warn(&format!("cinder furnace at {pos:?}: {}", error.0));
        }
    }

    if lit && state.m_burn % furnace::CINDER_EVERY == 0
    {
        cinderize(world, pos);
    }

    if state != before
    {
        let _ = ctx.setTileData(&state);
    }
}

struct FurnaceTile;

#[tile_behavior]
impl TileBehavior for FurnaceTile
{
    fn onTick(&self, ctx: &mut BlockCtx) { furnaceTick(ctx); }
}

// BlockTFCinderFurnace, idle and lit are two blocks since beta keeps light per block id
struct FurnaceBlock
{
    m_lit: bool,
}

#[block_behavior]
impl BlockBehavior for FurnaceBlock
{
    fn onActivated(&self, ctx: &mut BlockCtx, player: Player) -> bool
    {
        if ctx.isAuthority()
        {
            let _ = player.openScreen(FURNACE_NAME, Some(ctx.pos()), Vec::new());
        }

        true
    }

    // BlockTFCinderFurnace.randomDisplayTick, smoke and flame from a random side
    fn onRandomDisplayTick(&self, ctx: &mut BlockCtx)
    {
        if !self.m_lit
        {
            return;
        }

        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();
        let (out, across): (f64, f64) = (0.52, f64::from(crate::math::fraction(world)) * 0.6 - 0.3);
        let height: f64 = f64::from(pos.y) + f64::from(crate::math::fraction(world)) * 6.0 / 16.0;
        let [cx, cz] = [f64::from(pos.x) + 0.5, f64::from(pos.z) + 0.5];

        let at: [f64; 3] = match world.random(4) + 2
        {
            4 => [cx - out, height, cz + across],
            5 => [cx + out, height, cz + across],
            2 => [cx + across, height, cz - out],
            _ => [cx + across, height, cz + out],
        };
        world.particle("smoke", at, [0.0; 3]);
        world.particle("flame", at, [0.0; 3]);
    }
}

// GuiTFCinderFurnace, the vanilla furnace screen
struct FurnaceScreen
{
    m_pos: Option<BlockPos>,
}

impl Screen for FurnaceScreen
{
    fn init(&mut self, _width: i32, _height: i32, pos: Option<BlockPos>, _data: &[u8]) { self.m_pos = pos; }

    fn draw(&mut self, draw: &mut Draw, _mouse: [i32; 2], _partial: f32)
    {
        labels(draw);

        let Some(state) = localTile::<Burn>(self.m_pos)
        else
        {
            return;
        };
        if state.m_burn <= 0
        {
            return;
        }

        let [x, y, u, v] = gui::FLAME;

        let full: i32 = if state.m_fuelTime > 0
        {
            state.m_fuelTime
        }
        else
        {
            furnace::COOK
        };
        let flame: i32 = state.m_burn * gui::FLAME_SCALE / full;
        let top: i32 = gui::FLAME_BOTTOM - flame;
        draw.texture(FURNACE_SHEET, x, y + top, u, v + top, gui::FLAME_WIDTH, flame + 1);

        let [x, y, u, v] = gui::ARROW;
        let arrow: i32 = state.m_cook * gui::ARROW_LENGTH / furnace::COOK;
        draw.texture(FURNACE_SHEET, x, y, u, v, arrow + 1, gui::ARROW_HEIGHT);
    }
}

fn registerTable(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.blocks().register(
        TABLE_NAME,
        BlockSpecBuilder::template("minecraft:block")
            .name(TABLE_TITLE)
            .material("wood")
            .sound("wood")
            .hardness(table::HARDNESS)
            .faces(FaceTextures
            {
                top: Some(texture("uncrafting_top")),
                bottom: Some(texture("uncrafting_top")),
                side: Some(texture("uncrafting_side")),
                front: None,
            })
            .tool(ToolClass::Axe, 0)
            .tile(Tile::new(TableTile).inventory(table::SLOTS).ticks().synced())
            .behavior(OpensScreen { m_screen: TABLE_NAME }),
    )?;

    let mut screen: ScreenBuilder = ScreenBuilder::new(TABLE_TITLE, gui::WIDTH, gui::HEIGHT)
        .background(TABLE_SHEET)
        .playerInventory(gui::INVENTORY[0], gui::INVENTORY[1])
        .slot(table::INPUT, gui::TABLE_INPUT[0], gui::TABLE_INPUT[1])
        .outputSlot(table::RESULT, gui::TABLE_RESULT[0], gui::TABLE_RESULT[1]);

    for cell in 0..table::CELLS
    {
        screen = screen.slot(
            table::ASSEMBLY + cell,
            gui::ASSEMBLY[0] + cell % 3 * gui::CELL,
            gui::ASSEMBLY[1] + cell / 3 * gui::CELL,
        );
    }

    ctx.screens().register(TABLE_NAME, screen, TableScreen { m_pos: None, m_origin: [0; 2] })?;
    ctx.channels().register::<Pick>(table::CHANNEL, Direction::ToServer, pick)?;
    ctx.channels().register::<Leave>(LEAVE_CHANNEL, Direction::ToServer, leave)?;
    Ok(())
}

fn registerFurnace(ctx: &mut RegisterCtx) -> Result<()>
{
    let spec: TileEntitySpec = TileEntitySpec { inventory: furnace::SLOTS, ticks: true, sync: true, behavior: None };
    ctx.tileEntities().register(FURNACE_NAME, spec, Some(std::rc::Rc::new(FurnaceTile)))?;

    let (top, off, on): (&str, &str, &str) = ("furnace_top", "furnace_front", "furnace_front_on");

    let look = |front: &str| FaceTextures
    {
        top: Some(vanillaTexture(top)),
        bottom: Some(vanillaTexture(top)),
        side: Some(vanillaTexture(front)),
        front: None,
    };

    let spec = |front: &str, lit: bool| -> Result<BlockSpecBuilder>
    {
        Ok(BlockSpecBuilder::template("minecraft:block")
            .name(FURNACE_LABEL)
            .material("wood")
            .sound("stone")
            .hardness(furnace::HARDNESS)
            .faces(look(front))
            .drops(vec![crate::blocks::drop(FURNACE_NAME, vec![1], 1.0)?])
            .tileEntity(FURNACE_NAME)
            .behavior(FurnaceBlock { m_lit: lit }))
    };
    ctx.blocks().register(FURNACE_NAME, spec(off, false)?)?;
    ctx.blocks().register(FURNACE_LIT, spec(on, true)?.light(furnace::LIGHT))?;
    ctx.screens().register(
        FURNACE_NAME,
        ScreenBuilder::new(FURNACE_TITLE, gui::WIDTH, gui::HEIGHT)
            .background(FURNACE_SHEET)
            .playerInventory(gui::INVENTORY[0], gui::INVENTORY[1])
            .slot(furnace::INPUT, gui::FURNACE_INPUT[0], gui::FURNACE_INPUT[1])
            .slot(furnace::FUEL, gui::FURNACE_FUEL[0], gui::FURNACE_FUEL[1])
            .outputSlot(furnace::OUTPUT, gui::FURNACE_OUTPUT[0], gui::FURNACE_OUTPUT[1]),
        FurnaceScreen { m_pos: None },
    )
}

// the uncrafting table as a recipe viewer page and the cinder furnace beside the vanilla furnace
fn registerPages(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.machines().register(TABLE_NAME, Some(TABLE_TITLE), &[TABLE_NAME])?;
    ctx.machines()
        .recipe(TABLE_NAME)
        .note("Takes a crafted item apart into its recipe")
        .note("Off when the server config disables uncrafting")
        .build()?;
    ctx.machines().workstation("minecraft:smelting", FURNACE_NAME)?;

    ctx.machines().register(FURNACE_NAME, Some(FURNACE_TITLE), &[FURNACE_NAME])?;
    ctx.machines()
        .recipe(FURNACE_NAME)
        .note("Smelts as a furnace does, ores and logs come out multiplied beside cinder logs")
        .build()?;

    for tool in WOODEN_TOOLS
    {
        ctx.machines().fuel(FURNACE_NAME, tool, furnace::TOOL_FUEL)?;
    }

    Ok(())
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    registerTable(ctx)?;
    registerFurnace(ctx)?;
    registerPages(ctx)
}
