use super::{dye, takeArea};
use crate::core::area::Area;
use crate::core::power::{Provider, Setup};
use crate::core::render::{self, Laser, Scene};
use crate::core::{items, stacks};
use crate::factory::{soft, unbreakable};
use b173_api::prelude::*;
use std::rc::Rc;

// TileFiller: a pattern laid in its 3 by 3 grid, worked over the landmark box a block at a time from its resources
pub(crate) const NAME: &str = "filler";
const SLOTS: i32 = 36;
const GRID: i32 = 9;
const POWER: Setup = Setup
{
    m_latency: 10,
    m_minReceived: 25,
    m_maxReceived: 25,
    m_minActivation: 25,
    m_maxStored: 25,
    m_loss: Some((25, 1)),
};
const WORK_COST: i32 = 25;
const HARDNESS: f32 = 0.5;
const SIDE: &str = "buildcraft:block/filler_side";
const TOP_ON: &str = "buildcraft:block/filler_top_on";
const TOP_OFF: &str = "buildcraft:block/filler_top_off";
const BRICK: &str = "minecraft:brick_block";
const GLASS: &str = "minecraft:glass";

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub(crate) enum Pattern
{
    FillAll,
    Flattener,
    Remover,
    Walls,
    Pyramid,
    Stairs,
}

// FillerRegistry rows as BuildCraftBuilders adds them: the grid, the ingredient per letter and the icon
struct PatternRow
{
    m_pattern: Pattern,
    m_rows: [&'static str; 3],
    m_brick: Option<&'static str>,
    m_glass: Option<&'static str>,
    m_icon: &'static str,
}

const PATTERNS: [PatternRow; 6] = [
    PatternRow
    {
        m_pattern: Pattern::FillAll,
        m_rows: ["bbb", "bbb", "bbb"],
        m_brick: Some(BRICK),
        m_glass: None,
        m_icon: "buildcraft:block/filler_fill_all",
    },
    PatternRow
    {
        m_pattern: Pattern::Flattener,
        m_rows: ["   ", "ggg", "bbb"],
        m_brick: Some(BRICK),
        m_glass: Some(GLASS),
        m_icon: "buildcraft:block/filler_flattener",
    },
    PatternRow
    {
        m_pattern: Pattern::Remover,
        m_rows: ["ggg", "ggg", "ggg"],
        m_brick: None,
        m_glass: Some(GLASS),
        m_icon: "buildcraft:block/filler_remover",
    },
    PatternRow
    {
        m_pattern: Pattern::Walls,
        m_rows: ["bbb", "b b", "bbb"],
        m_brick: Some(BRICK),
        m_glass: None,
        m_icon: "buildcraft:block/filler_walls",
    },
    PatternRow
    {
        m_pattern: Pattern::Pyramid,
        m_rows: ["   ", " b ", "bbb"],
        m_brick: Some(BRICK),
        m_glass: None,
        m_icon: "buildcraft:block/filler_pyramid",
    },
    PatternRow
    {
        m_pattern: Pattern::Stairs,
        m_rows: ["  b", " bb", "bbb"],
        m_brick: Some(BRICK),
        m_glass: None,
        m_icon: "buildcraft:block/filler_stairs",
    },
];

mod gui
{
    // GuiFiller on filler.png
    pub const WIDTH: i32 = 175;
    pub const HEIGHT: i32 = 240;
    pub const BACKGROUND: &str = "buildcraft:gui/filler";
    pub const GRID: [i32; 2] = [31, 16];
    pub const RESOURCES: [i32; 2] = [8, 85];
    pub const INVENTORY: [i32; 2] = [8, 153];
    pub const SLOT: i32 = 18;
    pub const COLUMNS: i32 = 9;
    pub const SIDE: i32 = 3;
    pub const ICON: [i32; 2] = [125, 34];
    pub const ICON_SIZE: i32 = 16;
    pub const TITLE: [i32; 2] = [75, 6];
    pub const RESOURCES_LABEL: [i32; 2] = [8, 74];
    pub const INVENTORY_LABEL: [i32; 2] = [8, 142];
    pub const TEXT: u32 = 4210752;
    pub const TERRAIN: &str = "minecraft:terrain";
    pub const SHEET_TILES: i32 = 16;
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub(crate) struct FillerState
{
    m_started: bool,
    pub(crate) m_area: Option<Area>,
    pub(crate) m_pattern: Option<Pattern>,
    pub(crate) m_done: bool,
    m_forceDone: bool,
    m_provider: Option<Provider>,
}

fn iconOf(pattern: Pattern) -> &'static str
{
    PATTERNS.iter().find(|row| row.m_pattern == pattern).map_or(SIDE, |row| row.m_icon)
}

// ShapedPatternRecipe.matches: the grid fits the pattern at some offset, straight or mirrored
fn ingredient(row: &PatternRow, letter: u8) -> Option<&'static str>
{
    match letter
    {
        b'b' => row.m_brick,
        b'g' => row.m_glass,
        _ => None,
    }
}

fn fits(row: &PatternRow, grid: &[Option<ItemStack>], mirror: bool) -> bool
{
    (0..3).all(|y|
    {
        (0..3).all(|x|
        {
            let column: usize = if mirror { 2 - x } else { x };
            let wanted: Option<&str> = ingredient(row, row.m_rows[y].as_bytes()[column]);
            match (wanted, &grid[y * 3 + x])
            {
                (None, None) => true,
                (Some(name), Some(stack)) => stack.item.is(name),
                _ => false,
            }
        })
    })
}

fn findPattern(world: World, pos: BlockPos) -> Option<Pattern>
{
    let grid: Vec<Option<ItemStack>> = (0..GRID).map(|slot| world.tileSlot(pos, slot)).collect();
    PATTERNS.iter().find(|row| fits(row, &grid, false) || fits(row, &grid, true)).map(|row| row.m_pattern)
}

// FillerPattern.fill: the first open block from the bottom gets the stack's block, true once nothing is open
pub(crate) fn fillLayer(world: World, min: [i32; 3], max: [i32; 3], stack: Option<&mut ItemStack>) -> bool
{
    for y in min[1]..=max[1]
    {
        for x in min[0]..=max[0]
        {
            for z in min[2]..=max[2]
            {
                let pos: BlockPos = BlockPos::new(x, y, z);
                if !soft(world, pos)
                {
                    continue;
                }

                if let Some(stack) = stack
                    && world.placeBlock(pos, 0, stack).unwrap_or(false)
                {
                    stack.count -= 1;
                }

                return false;
            }
        }
    }

    true
}

// FillerPattern.empty: the lowest layer with something solid loses its last block, true once the box is clear
fn empty(world: World, min: [i32; 3], max: [i32; 3]) -> bool
{
    for y in min[1]..=max[1]
    {
        let mut last: Option<BlockPos> = None;

        for x in min[0]..=max[0]
        {
            for z in min[2]..=max[2]
            {
                let pos: BlockPos = BlockPos::new(x, y, z);
                if !soft(world, pos) && !unbreakable(world, pos)
                {
                    last = Some(pos);
                }
            }
        }

        if let Some(pos) = last
        {
            stacks::breakBlock(world, pos);
            return false;
        }
    }

    true
}

// FillerFlattener: fill every hole under the box down to the first solid layer, then clear everything above
fn flatten(world: World, area: &Area, stack: Option<&mut ItemStack>) -> bool
{
    let [x0, y0, z0] = area.m_min;
    let [x1, _, z1] = area.m_max;
    let mut blocked: Vec<bool> = vec![false; (area.size(0) * area.size(2)) as usize];
    let mut last: Option<BlockPos> = None;

    for y in (0..y0).rev()
    {
        let mut found: bool = false;

        for x in x0..=x1
        {
            for z in z0..=z1
            {
                let index: usize = ((x - x0) * area.size(2) + (z - z0)) as usize;
                if blocked[index]
                {
                    continue;
                }

                let pos: BlockPos = BlockPos::new(x, y, z);
                if soft(world, pos)
                {
                    found = true;
                    last = Some(pos);
                }
                else
                {
                    blocked[index] = true;
                }
            }
        }

        if !found
        {
            break;
        }
    }

    if let Some(pos) = last
    {
        if let Some(stack) = stack
            && world.placeBlock(pos, 0, stack).unwrap_or(false)
        {
            stack.count -= 1;
        }

        return false;
    }

    const TOP: i32 = 128;
    empty(world, area.m_min, [x1, TOP, z1])
}

// FillerFillPyramid: shrinking layers from the side of the box nearest the filler
fn pyramid(world: World, pos: BlockPos, area: &Area, mut stack: Option<&mut ItemStack>) -> bool
{
    let up: bool = pos.y <= area.m_min[1];
    let mut height: i32 = if up { area.m_min[1] } else { area.m_max[1] };
    let mut step: i32 = 0;

    while step <= area.size(0) / 2 && step <= area.size(2) / 2 && (area.m_min[1]..=area.m_max[1]).contains(&height)
    {
        let min: [i32; 3] = [area.m_min[0] + step, height, area.m_min[2] + step];
        let max: [i32; 3] = [area.m_max[0] - step, height, area.m_max[2] - step];
        if !fillLayer(world, min, max, stack.as_deref_mut())
        {
            return false;
        }

        step += 1;
        height += if up { 1 } else { -1 };
    }

    true
}

// FillerFillWalls: the floor, the four walls and the roof, each finished before the next
fn walls(world: World, area: &Area, mut stack: Option<&mut ItemStack>) -> bool
{
    let [x0, y0, z0] = area.m_min;
    let [x1, y1, z1] = area.m_max;
    let faces: [([i32; 3], [i32; 3]); 6] = [
        ([x0, y0, z0], [x1, y0, z1]),
        ([x0, y0, z0], [x0, y1, z1]),
        ([x0, y0, z0], [x1, y1, z0]),
        ([x1, y0, z0], [x1, y1, z1]),
        ([x0, y0, z1], [x1, y1, z1]),
        ([x0, y1, z0], [x1, y1, z1]),
    ];

    faces.into_iter().all(|(min, max)| fillLayer(world, min, max, stack.as_deref_mut()))
}

fn iterate(pattern: Pattern, world: World, pos: BlockPos, area: &Area, stack: Option<&mut ItemStack>) -> bool
{
    match pattern
    {
        Pattern::FillAll => fillLayer(world, area.m_min, area.m_max, stack),
        Pattern::Flattener => flatten(world, area, stack),
        Pattern::Remover => empty(world, area.m_min, area.m_max),
        Pattern::Walls => walls(world, area, stack),
        Pattern::Pyramid => pyramid(world, pos, area, stack),
        Pattern::Stairs => super::stairs::iterate(world, pos, area, stack),
    }
}

// TileFiller.computeRecipe on every tick, since a slot change has no hook of its own
fn recompute(world: World, pos: BlockPos, state: &mut FillerState)
{
    let pattern: Option<Pattern> = findPattern(world, pos);
    if pattern == state.m_pattern
    {
        return;
    }

    state.m_pattern = pattern;
    state.m_done = pattern.is_none() || state.m_forceDone;
    state.m_forceDone = false;
    world.markBlockForRender(pos);
}

// TileFiller.doWork: the first block in its resources is what it places
fn work(world: World, pos: BlockPos, state: &mut FillerState)
{
    if state.m_provider.get_or_insert_with(|| POWER.provider()).useEnergy(WORK_COST, WORK_COST, true) < WORK_COST
    {
        return;
    }

    let (Some(area), Some(pattern)) = (state.m_area, state.m_pattern)
    else
    {
        return;
    };
    if state.m_done
    {
        return;
    }

    let resource: Option<(i32, ItemStack)> =
        (GRID..SLOTS).find_map(|slot| world.tileSlot(pos, slot).filter(stacks::isBlock).map(|stack| (slot, stack)));

    match resource
    {
        Some((slot, mut stack)) =>
        {
            state.m_done = iterate(pattern, world, pos, &area, Some(&mut stack));
            let _ = world.setTileSlot(pos, slot, (stack.count > 0).then_some(stack));
        }

        None => state.m_done = iterate(pattern, world, pos, &area, None),
    }

    if state.m_done
    {
        world.markBlockForRender(pos);
    }
}

struct FillerTile;

#[tile_behavior]
impl TileBehavior for FillerTile
{
    fn onTick(&self, ctx: &mut BlockCtx)
    {
        if !ctx.isAuthority()
        {
            return;
        }

        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();
        let before: FillerState = ctx.tileData();
        let mut state: FillerState = before.clone();

        // TileFiller.initialize takes the box of a landmark next to it
        if !state.m_started
        {
            state.m_started = true;
            state.m_area = takeArea(world, pos);
            state.m_done = true;
        }

        recompute(world, pos, &mut state);

        let mut provider: Provider = state.m_provider.take().unwrap_or_else(|| POWER.provider());
        let ready: bool = provider.update(world, pos);
        state.m_provider = Some(provider);

        if ready
        {
            work(world, pos, &mut state);
        }

        if let Some(provider) = state.m_provider.as_mut()
        {
            provider.decay(world);
        }

        if state.m_area.is_none()
        {
            state.m_done = true;
        }

        if state != before
        {
            let _ = ctx.setTileData(&state);
        }
    }

    fn pose(&self, ctx: &mut BlockCtx, _partial: f32, _ticks: i32) -> Pose
    {
        let world: World = ctx.world();
        let state: FillerState = ctx.tileData();
        let Some(area) = state.m_area
        else
        {
            return Pose::default();
        };

        let mut draw: Scene = Scene::new();
        let pos: BlockPos = ctx.pos();
        draw.laserBox(
            render::relative(pos, area.min()),
            render::relative(pos, area.max()),
            Laser::Stripes,
            render::lightAt(world, pos),
        );
        draw.finish()
    }

    // ISpecialInventory.addItem, resources only, joining stacks before filling empty slots
    fn insertItem(&self, ctx: &mut BlockCtx, _face: i32, stack: ItemStack, simulate: bool) -> i32
    {
        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();
        let mut left: i32 = stack.count;
        let limit: i32 = stack.info().map_or(stack.count, |info| info.max_stack);

        for joining in [true, false]
        {
            for slot in GRID..SLOTS
            {
                if left <= 0
                {
                    return 0;
                }

                let held: Option<ItemStack> = world.tileSlot(pos, slot);
                let room: i32 = match &held
                {
                    Some(held)
                        if joining && held.item.id().ok() == stack.item.id().ok() && held.damage == stack.damage =>
                    {
                        limit - held.count
                    }
                    None if !joining => limit,
                    _ => 0,
                };
                let moved: i32 = room.clamp(0, left);
                if moved == 0
                {
                    continue;
                }

                left -= moved;
                if !simulate
                {
                    let count: i32 = held.map_or(0, |held| held.count) + moved;
                    let _ = world.setTileSlot(pos, slot, Some(ItemStack { count, ..stack.clone() }));
                }
            }
        }

        left
    }

    // ISpecialInventory.extractItem, one resource at a time
    fn extractItem(&self, ctx: &mut BlockCtx, _face: i32, _max: i32, simulate: bool) -> Option<ItemStack>
    {
        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();
        let (slot, held): (i32, ItemStack) =
            (GRID..SLOTS).find_map(|slot| world.tileSlot(pos, slot).map(|held| (slot, held)))?;

        if !simulate
        {
            let _ = world.setTileSlot(
                pos,
                slot,
                (held.count > 1).then(|| ItemStack { count: held.count - 1, ..held.clone() }),
            );
        }

        Some(ItemStack { count: 1, ..held })
    }

    fn energyDemand(&self, ctx: &mut BlockCtx, _face: i32) -> Option<EnergyDemand>
    {
        let state: FillerState = ctx.tileData();
        state.m_provider.unwrap_or_else(|| POWER.provider()).demand()
    }

    fn receiveEnergy(&self, ctx: &mut BlockCtx, _face: i32, amount: i64, simulate: bool) -> i64
    {
        let mut state: FillerState = ctx.tileData();
        let mut provider: Provider = state.m_provider.unwrap_or_else(|| POWER.provider());
        let taken: i64 = provider.take(amount, simulate);

        if !simulate
        {
            state.m_provider = Some(provider);
            let _ = ctx.setTileData(&state);
        }

        taken
    }
}

// BlockFiller.getBlockTexture: the pattern's icon on the sides, the top lit while it works
struct FillerBlock;

#[block_behavior]
impl BlockBehavior for FillerBlock
{
    fn faceTexture(&self, ctx: &mut BlockCtx, face: i32) -> Option<i32>
    {
        let state: FillerState = ctx.tileData();
        let sprite: &'static str = match face
        {
            0 | 1 if state.m_done => TOP_OFF,
            0 | 1 => TOP_ON,
            _ => state.m_pattern.map_or(SIDE, iconOf),
        };
        Some(render::slot(sprite))
    }

    fn texture(&self, face: i32, _meta: i32) -> i32 { render::slot(if face <= 1 { TOP_ON } else { SIDE }) }

    fn onActivated(&self, ctx: &mut BlockCtx, player: Player) -> bool
    {
        if ctx.isAuthority()
        {
            let _ = player.openScreen(NAME, Some(ctx.pos()), Vec::new());
        }

        true
    }
}

struct FillerScreen
{
    m_pos: Option<BlockPos>,
}

impl Screen for FillerScreen
{
    fn init(&mut self, _width: i32, _height: i32, pos: Option<BlockPos>, _data: &[u8]) { self.m_pos = pos; }

    fn draw(&mut self, draw: &mut Draw, _mouse: [i32; 2], _partial: f32)
    {
        draw.text(gui::TITLE[0], gui::TITLE[1], &translate("buildcraft.gui.filler"), gui::TEXT);
        draw.text(
            gui::RESOURCES_LABEL[0],
            gui::RESOURCES_LABEL[1],
            &translate("buildcraft.gui.filling_resources"),
            gui::TEXT,
        );
        draw.text(gui::INVENTORY_LABEL[0], gui::INVENTORY_LABEL[1], &translate("buildcraft.gui.inventory"), gui::TEXT);

        let Some(pos) = self.m_pos
        else
        {
            return;
        };
        let Some(world) = localPlayer().and_then(|player| player.info()).map(|info| World { dimension: info.world })
        else
        {
            return;
        };
        let state: FillerState = world.tileData(pos);
        let Some(pattern) = state.m_pattern
        else
        {
            return;
        };

        let slot: i32 = render::slot(iconOf(pattern));
        let (u, v): (i32, i32) = (slot % gui::SHEET_TILES * gui::ICON_SIZE, slot / gui::SHEET_TILES * gui::ICON_SIZE);
        draw.texture(gui::TERRAIN, gui::ICON[0], gui::ICON[1], u, v, gui::ICON_SIZE, gui::ICON_SIZE);
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for sprite in [SIDE, TOP_ON, TOP_OFF].into_iter().chain(PATTERNS.iter().map(|row| row.m_icon))
    {
        render::slot(sprite);
    }

    let tile: Rc<dyn TileBehavior> = Rc::new(FillerTile);
    ctx.tileEntities().register(
        NAME,
        TileEntitySpec { inventory: SLOTS, ticks: true, sync: true, behavior: None },
        Some(tile),
    )?;
    ctx.tileEntities().model(NAME, EntityModelSpec::default())?;
    ctx.blocks().register(
        NAME,
        BlockSpecBuilder::template("minecraft:block")
            .material("iron")
            .sound("metal")
            .hardness(HARDNESS)
            .texture(SIDE)
            .tileEntity(NAME)
            .behavior(FillerBlock),
    )?;

    let mut screen: ScreenBuilder = ScreenBuilder::new("", gui::WIDTH, gui::HEIGHT)
        .background(gui::BACKGROUND)
        .playerInventory(gui::INVENTORY[0], gui::INVENTORY[1]);
    for slot in 0..GRID
    {
        let (row, column): (i32, i32) = (slot / gui::SIDE, slot % gui::SIDE);
        screen = screen.slot(slot, gui::GRID[0] + column * gui::SLOT, gui::GRID[1] + row * gui::SLOT);
    }
    for slot in GRID..SLOTS
    {
        let index: i32 = slot - GRID;
        let (row, column): (i32, i32) = (index / gui::COLUMNS, index % gui::COLUMNS);
        screen = screen.slot(slot, gui::RESOURCES[0] + column * gui::SLOT, gui::RESOURCES[1] + row * gui::SLOT);
    }
    ctx.screens().register(NAME, screen, FillerScreen { m_pos: None })?;

    dye::recipe(ctx, NAME, items::gear::GOLD, |shape| shape.keyTag('C', items::material::CHEST))
}
