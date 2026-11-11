use crate::core::{dir, items, render, stacks};
use b173_api::prelude::*;
use std::rc::Rc;

// TileAutoWorkbench: a crafting grid that only refills stacks already in it and crafts whenever a pipe pulls,
// taking the last item of a slot from the inventories around it
const NAME: &str = "auto_workbench";
const SLOTS: i32 = 9;
const HARDNESS: f32 = 1.0;
const TOP: &str = "buildcraft:block/auto_workbench_top";
const SIDE: &str = "buildcraft:block/auto_workbench_side";
// getNearbyItem looks west, east, down, up, north and south
const SOURCES: [i32; 6] = [dir::WEST, dir::EAST, dir::DOWN, dir::UP, dir::NORTH, dir::SOUTH];

mod gui
{
    // GuiAutoCrafting on the vanilla crafting table background
    pub const WIDTH: i32 = 176;
    pub const HEIGHT: i32 = 166;
    pub const BACKGROUND: &str = "minecraft:gui/crafting";
    pub const GRID: [i32; 2] = [30, 17];
    pub const SLOT: i32 = 18;
    pub const SIDE: i32 = 3;
    pub const RESULT: [i32; 2] = [124, 35];
    pub const INVENTORY: [i32; 2] = [8, 84];
    pub const TITLE: [i32; 2] = [28, 6];
    pub const INVENTORY_LABEL_DROP: i32 = 94;
    pub const TEXT: u32 = 4210752;
}

fn same(first: &ItemStack, second: &ItemStack) -> bool
{
    first.item.id().ok() == second.item.id().ok() && first.damage == second.damage
}

fn grid(world: World, pos: BlockPos) -> [Option<ItemStack>; 9]
{
    std::array::from_fn(|slot| world.tileSlot(pos, slot as i32))
}

// where one ingredient comes from: a slot of the table or of a neighbouring inventory
struct Source
{
    m_pos: BlockPos,
    m_slot: i32,
}

// TileAutoWorkbench.getNearbyItemFromOrientation: plain inventories only, never a special one, a chest with its
// other half
fn nearby(world: World, pos: BlockPos, wanted: &ItemStack, used: &[Source]) -> Option<Source>
{
    for face in SOURCES
    {
        let target: BlockPos = dir::step(pos, face);
        if !stacks::plain(world, target, dir::reverse(face))
        {
            continue;
        }

        for (at, slot) in stacks::slotsOf(world, target)
        {
            let taken: i32 = used.iter().filter(|source| source.m_pos == at && source.m_slot == slot).count() as i32;
            let available: bool =
                world.tileSlot(at, slot).is_some_and(|stack| same(&stack, wanted) && stack.count > taken);
            if available
            {
                return Some(Source { m_pos: at, m_slot: slot });
            }
        }
    }

    None
}

// the ingredients one craft takes, None when a last item has no spare nearby
fn plan(world: World, pos: BlockPos, slots: &[Option<ItemStack>; 9]) -> Option<Vec<Source>>
{
    let mut sources: Vec<Source> = Vec::new();

    for (slot, stack) in slots.iter().enumerate()
    {
        let Some(stack) = stack
        else
        {
            continue;
        };

        let source: Source = if stack.count <= 1
        {
            nearby(world, pos, stack, &sources)?
        }
        else
        {
            Source { m_pos: pos, m_slot: slot as i32 }
        };
        sources.push(source);
    }

    Some(sources)
}

// TileAutoWorkbench.extractItem: one craft, the ingredients' containers such as buckets left in their place
fn craft(world: World, pos: BlockPos, simulate: bool) -> Option<ItemStack>
{
    let slots: [Option<ItemStack>; 9] = grid(world, pos);
    let sources: Vec<Source> = plan(world, pos, &slots)?;
    let result: ItemStack = craftResult(&slots)?;
    if simulate
    {
        return Some(result);
    }

    for source in &sources
    {
        let Some(stack) = world.tileSlot(source.m_pos, source.m_slot)
        else
        {
            continue;
        };

        let left: Option<ItemStack> = (stack.count > 1).then(|| ItemStack { count: stack.count - 1, ..stack.clone() });
        let container: Option<ItemStack> = containerItem(&stack);
        let (kept, extra): (Option<ItemStack>, Option<ItemStack>) = match (left, container)
        {
            (None, container) => (container, None),
            (left, container) => (left, container),
        };

        let _ = world.setTileSlot(source.m_pos, source.m_slot, kept);

        if let Some(extra) = extra
        {
            stacks::drop(world, pos, extra);
        }
    }

    Some(result)
}

// TileAutoWorkbench.addItem: one item at a time into the smallest matching stack of the grid
fn fill(world: World, pos: BlockPos, stack: &ItemStack, simulate: bool) -> i32
{
    let mut counts: [Option<ItemStack>; 9] = grid(world, pos);
    let limit: i32 = stacks::info(stack).map_or(stack.count, |info| info.max_stack);
    let mut left: i32 = stack.count;

    while left > 0
    {
        let smallest: Option<usize> = counts
            .iter()
            .enumerate()
            .filter_map(|(slot, held)|
            {
                held.as_ref().filter(|held| held.count > 0 && same(held, stack)).map(|held| (slot, held.count))
            })
            .min_by_key(|(_, count)| *count)
            .map(|(slot, _)| slot);
        let Some(slot) = smallest
        else
        {
            break;
        };
        let Some(held) = counts[slot].as_mut().filter(|held| held.count < limit)
        else
        {
            break;
        };

        held.count += 1;
        left -= 1;
    }

    if !simulate && left < stack.count
    {
        for (slot, held) in counts.into_iter().enumerate()
        {
            let _ = world.setTileSlot(pos, slot as i32, held);
        }
    }

    left
}

struct WorkbenchTile;

#[tile_behavior]
impl TileBehavior for WorkbenchTile
{
    fn insertItem(&self, ctx: &mut BlockCtx, _face: i32, stack: ItemStack, simulate: bool) -> i32
    {
        fill(ctx.world(), ctx.pos(), &stack, simulate)
    }

    fn extractItem(&self, ctx: &mut BlockCtx, _face: i32, _max: i32, simulate: bool) -> Option<ItemStack>
    {
        craft(ctx.world(), ctx.pos(), simulate)
    }
}

struct WorkbenchBlock;

#[block_behavior]
impl BlockBehavior for WorkbenchBlock
{
    fn texture(&self, face: i32, _meta: i32) -> i32 { render::slot(if face <= dir::UP { TOP } else { SIDE }) }

    fn onActivated(&self, ctx: &mut BlockCtx, player: Player) -> bool
    {
        if ctx.isAuthority()
        {
            let _ = player.openScreen(NAME, Some(ctx.pos()), Vec::new());
        }

        true
    }
}

// GuiAutoCrafting: the vanilla crafting screen over the table's grid
struct WorkbenchScreen;

impl Screen for WorkbenchScreen
{
    fn draw(&mut self, draw: &mut Draw, _mouse: [i32; 2], _partial: f32)
    {
        draw.text(gui::TITLE[0], gui::TITLE[1], &translate("buildcraft.gui.crafting"), gui::TEXT);
        draw.text(
            gui::INVENTORY[0],
            gui::HEIGHT - gui::INVENTORY_LABEL_DROP,
            &translate("buildcraft.gui.inventory"),
            gui::TEXT,
        );
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    render::slot(TOP);
    render::slot(SIDE);

    let tile: Rc<dyn TileBehavior> = Rc::new(WorkbenchTile);
    ctx.tileEntities().register(
        NAME,
        TileEntitySpec { inventory: SLOTS, ticks: false, sync: false, behavior: None },
        Some(tile),
    )?;
    ctx.blocks().register(
        NAME,
        BlockSpecBuilder::template("minecraft:block")
            .material("wood")
            .sound("wood")
            .hardness(HARDNESS)
            .texture(SIDE)
            .tileEntity(NAME)
            .behavior(WorkbenchBlock),
    )?;
    ctx.machines().workstation("minecraft:crafting", NAME)?;

    let mut screen: ScreenBuilder = ScreenBuilder::new("", gui::WIDTH, gui::HEIGHT)
        .background(gui::BACKGROUND)
        .playerInventory(gui::INVENTORY[0], gui::INVENTORY[1]);
    for slot in 0..SLOTS
    {
        let (row, column): (i32, i32) = (slot / gui::SIDE, slot % gui::SIDE);
        screen = screen.slot(slot, gui::GRID[0] + column * gui::SLOT, gui::GRID[1] + row * gui::SLOT);
    }
    ctx.screens().register(NAME, screen, WorkbenchScreen)?;
    ctx.screens().craftResult(NAME, 0, gui::RESULT[0], gui::RESULT[1])?;

    ctx.recipes()
        .shaped(NAME, ItemStack::of("buildcraft:auto_workbench", 1))
        .pattern([" g ", "gwg", " g "])
        .keyTag('w', items::material::CRAFTING_TABLE)
        .keyTag('g', items::gear::WOOD)
        .build()
}
