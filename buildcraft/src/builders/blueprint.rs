use super::{TEMPLATE, dye, takeArea};
use crate::core::area::Area;
use crate::core::power::{Provider, Setup};
use crate::core::render::{self, Laser, Scene};
use crate::core::{AIR, dir, items, stacks, state};
use crate::factory::{Faces, facePlacer, soft, unbreakable};
use b173_api::prelude::*;
use std::collections::BTreeMap;

// BluePrint: a mask of filled and empty blocks with an anchor, saved with the world and named by the template's damage
const STORE: &str = "blueprints";
const TABLE: &str = "template_table";
const BUILDER: &str = "builder";
const TABLE_SLOTS: i32 = 2;
const BUILDER_SLOTS: i32 = 28;
const COMPUTE_TICKS: i32 = 200;
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
const TABLE_HARDNESS: f32 = 0.5;
const BUILDER_HARDNESS: f32 = 0.7;
const TOP_POWERED: &str = "buildcraft:block/template_top_powered";

const TABLE_FACES: Faces = Faces
{
    m_front: "buildcraft:block/template_front",
    m_top: "buildcraft:block/template_top",
    m_side: "buildcraft:block/template_side",
    m_bottom: None,
    m_back: None,
};

const BUILDER_FACES: Faces = Faces
{
    m_front: "buildcraft:block/builder_front",
    m_top: "buildcraft:block/builder_top",
    m_side: "buildcraft:block/builder_side",
    m_bottom: None,
    m_back: None,
};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, Default)]
pub(crate) struct Blueprint
{
    m_size: [i32; 3],
    m_anchor: [i32; 3],
    m_mask: Vec<bool>,
}

impl Blueprint
{
    fn index(&self, x: i32, y: i32, z: i32) -> usize { ((x * self.m_size[1] + y) * self.m_size[2] + z) as usize }

    fn filled(&self, x: i32, y: i32, z: i32) -> bool { self.m_mask[self.index(x, y, z)] }

    // BluePrint.rotateLeft, a quarter turn that swaps the x and z sizes
    fn rotateLeft(&self) -> Blueprint
    {
        let [sizeX, sizeY, sizeZ] = self.m_size;
        let mut turned: Blueprint =
            Blueprint { m_size: [sizeZ, sizeY, sizeX], m_anchor: [0; 3], m_mask: vec![false; self.m_mask.len()] };

        for x in 0..sizeZ
        {
            for y in 0..sizeY
            {
                for z in 0..sizeX
                {
                    let index: usize = turned.index(x, y, z);
                    turned.m_mask[index] = self.filled(z, y, sizeZ - 1 - x);
                }
            }
        }

        turned.m_anchor = [sizeZ - 1 - self.m_anchor[2], self.m_anchor[1], self.m_anchor[0]];
        turned
    }

    fn turned(&self, quarters: usize) -> Blueprint
    {
        (0..quarters).fold(self.clone(), |blueprint, _| blueprint.rotateLeft())
    }
}

// BuildCraftBuilders.bluePrints, kept outside the worlds so templates carry between saves as in buildcraft/blueprints
fn store() -> BTreeMap<i32, Blueprint> { sharedData(STORE) }

fn save(blueprint: Blueprint) -> i32
{
    let mut all: BTreeMap<i32, Blueprint> = store();
    let id: i32 = (1..).find(|id| !all.contains_key(id)).unwrap_or(1);
    all.insert(id, blueprint);

    if let Err(error) = setSharedData(STORE, &all)
    {
        warn(&format!("blueprint {id} could not be saved: {error}"));
    }

    id
}

// the face a builder or template table points at, down for one never placed by a player
fn facing(world: World, pos: BlockPos) -> i32 { state::faceAt(world, pos, state::FACING).unwrap_or(dir::DOWN) }

// the quarter turns that face a blueprint along a machine's facing, counted as BuildCraft's rotateLeft calls
fn quarters(front: i32, saving: bool) -> usize
{
    let toward: i32 = dir::reverse(front);
    let turns: usize = match toward
    {
        dir::SOUTH => 1,
        dir::WEST => 2,
        dir::NORTH => 3,
        _ => 0,
    };

    if saving { (4 - turns) % 4 } else { turns }
}

// BlockBuilder and BlockTemplate turn a quarter with a wrench
fn wrenchTurn(world: World, pos: BlockPos)
{
    let next: i32 = match facing(world, pos)
    {
        dir::WEST => dir::SOUTH,
        dir::EAST => dir::NORTH,
        dir::NORTH => dir::WEST,
        dir::SOUTH => dir::EAST,
        face => face,
    };
    state::set(world, pos, state::FACING, dir::name(next));
    world.markBlockForRender(pos);
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
struct TableState
{
    m_started: bool,
    m_area: Option<Area>,
    m_computing: bool,
    m_time: i32,
    m_last: i32,
}

fn templateIn(world: World, pos: BlockPos, slot: i32) -> Option<ItemStack>
{
    world.tileSlot(pos, slot).filter(|stack| stack.item.is(TEMPLATE))
}

// TileTemplate.createBluePrint: filled blocks in the box, inverted while the table is powered
fn draw(world: World, pos: BlockPos, area: &Area, last: i32) -> (i32, Blueprint)
{
    let inverted: bool = world.isPowered(pos);
    let size: [i32; 3] = [area.size(0), area.size(1), area.size(2)];
    let mut blueprint: Blueprint = Blueprint
    {
        m_size: size,
        m_anchor: [pos.x - area.m_min[0], pos.y - area.m_min[1], pos.z - area.m_min[2]],
        m_mask: vec![false; (size[0] * size[1] * size[2]) as usize],
    };

    for x in 0..size[0]
    {
        for y in 0..size[1]
        {
            for z in 0..size[2]
            {
                let solid: bool =
                    !world.is(BlockPos::new(area.m_min[0] + x, area.m_min[1] + y, area.m_min[2] + z), AIR);
                let index: usize = blueprint.index(x, y, z);
                blueprint.m_mask[index] = solid != inverted;
            }
        }
    }

    let blueprint: Blueprint = blueprint.turned(quarters(facing(world, pos), true));
    if store().get(&last) == Some(&blueprint)
    {
        return (last, blueprint);
    }

    (save(blueprint.clone()), blueprint)
}

// TileTemplate.updateEntity and initializeComputing, checked each tick since slot changes have no hook
fn tableTick(world: World, pos: BlockPos, state: &mut TableState)
{
    if !state.m_started
    {
        state.m_started = true;
        state.m_area = takeArea(world, pos);
    }

    let Some(area) = state.m_area
    else
    {
        return;
    };
    let ready: bool = templateIn(world, pos, 0).is_some() && world.tileSlot(pos, 1).is_none();

    if !ready
    {
        state.m_computing = false;
        state.m_time = 0;
        return;
    }

    if !state.m_computing
    {
        state.m_computing = true;
        state.m_time = 0;
    }

    if state.m_time < COMPUTE_TICKS
    {
        state.m_time += 1;
        return;
    }

    let (id, _) = draw(world, pos, &area, state.m_last);
    state.m_last = id;
    state.m_computing = false;
    state.m_time = 0;

    let _ = world.setTileSlot(pos, 0, None);
    let _ = world.setTileSlot(pos, 1, Some(ItemStack::of(TEMPLATE, 1).withDamage(id)));
}

struct TableTile;

#[tile_behavior]
impl TileBehavior for TableTile
{
    fn onTick(&self, ctx: &mut BlockCtx)
    {
        if !ctx.isAuthority()
        {
            return;
        }

        let before: TableState = ctx.tileData();
        let mut state: TableState = before.clone();
        tableTick(ctx.world(), ctx.pos(), &mut state);

        if state != before
        {
            let _ = ctx.setTileData(&state);
        }
    }

    fn pose(&self, ctx: &mut BlockCtx, _partial: f32, _ticks: i32) -> Pose
    {
        let state: TableState = ctx.tileData();
        boxPose(ctx, state.m_area)
    }
}

fn boxPose(ctx: &mut BlockCtx, area: Option<Area>) -> Pose
{
    let Some(area) = area
    else
    {
        return Pose::default();
    };

    let pos: BlockPos = ctx.pos();
    let mut scene: Scene = Scene::new();
    scene.laserBox(
        render::relative(pos, area.min()),
        render::relative(pos, area.max()),
        Laser::Stripes,
        render::lightAt(ctx.world(), pos),
    );
    scene.finish()
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
struct BuilderState
{
    m_blueprint: i32,
    m_area: Option<Area>,
    m_provider: Option<Provider>,
}

// BluePrintBuilder.findNextBlock in template mode: an empty spot the mask fills, or a block the mask clears
fn nextBlock(world: World, origin: BlockPos, blueprint: &Blueprint) -> Option<(BlockPos, bool)>
{
    let [sizeX, sizeY, sizeZ] = blueprint.m_size;
    let [anchorX, anchorY, anchorZ] = blueprint.m_anchor;

    for y in 0..sizeY
    {
        for x in 0..sizeX
        {
            for z in 0..sizeZ
            {
                let pos: BlockPos =
                    BlockPos::new(x + origin.x - anchorX, y + origin.y - anchorY, z + origin.z - anchorZ);
                if pos.y <= 0
                {
                    continue;
                }

                let filled: bool = blueprint.filled(x, y, z);
                if filled && soft(world, pos)
                {
                    return Some((pos, true));
                }

                if !filled && !soft(world, pos) && !unbreakable(world, pos)
                {
                    return Some((pos, false));
                }
            }
        }
    }

    None
}

fn placed(pos: BlockPos, blueprint: &Blueprint) -> Area
{
    let min: [i32; 3] = std::array::from_fn(|axis| [pos.x, pos.y, pos.z][axis] - blueprint.m_anchor[axis]);
    Area::new(min, std::array::from_fn(|axis| min[axis] + blueprint.m_size[axis] - 1))
}

// TileBuilder.initalizeBluePrint: the template in slot 0 picks the blueprint, turned to the builder's facing
fn current(world: World, pos: BlockPos, state: &mut BuilderState) -> Option<Blueprint>
{
    let Some(template) = templateIn(world, pos, 0).filter(|stack| stack.damage > 0)
    else
    {
        state.m_blueprint = 0;
        state.m_area = None;
        return None;
    };

    let blueprint: Blueprint = store().get(&template.damage)?.turned(quarters(facing(world, pos), false));
    if state.m_blueprint != template.damage
    {
        state.m_blueprint = template.damage;
        state.m_area = Some(placed(pos, &blueprint));
    }

    Some(blueprint)
}

// TileBuilder.doWork: clear what is in the way, then fill from the first block in its resources
fn build(world: World, pos: BlockPos, state: &mut BuilderState)
{
    let Some(blueprint) = current(world, pos, state)
    else
    {
        return;
    };
    let Some((target, fill)) = nextBlock(world, pos, &blueprint)
    else
    {
        state.m_area = None;
        return;
    };

    if !fill
    {
        stacks::breakBlock(world, target);
        return;
    }

    for slot in 1..BUILDER_SLOTS
    {
        let Some(mut stack) = world.tileSlot(pos, slot).filter(stacks::isBlock)
        else
        {
            continue;
        };

        if world.placeBlock(target, 0, &stack).unwrap_or(false)
        {
            stack.count -= 1;
            let _ = world.setTileSlot(pos, slot, (stack.count > 0).then_some(stack));
        }

        return;
    }
}

struct BuilderTile;

#[tile_behavior]
impl TileBehavior for BuilderTile
{
    fn onTick(&self, ctx: &mut BlockCtx)
    {
        if !ctx.isAuthority()
        {
            return;
        }

        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();
        let before: BuilderState = ctx.tileData();
        let mut state: BuilderState = before.clone();
        let mut provider: Provider = state.m_provider.take().unwrap_or_else(|| POWER.provider());

        if provider.update(world, pos) && provider.useEnergy(WORK_COST, WORK_COST, true) >= WORK_COST
        {
            build(world, pos, &mut state);
        }
        else if templateIn(world, pos, 0).is_none()
        {
            state.m_area = None;
            state.m_blueprint = 0;
        }

        provider.decay(world);
        state.m_provider = Some(provider);

        if state != before
        {
            let _ = ctx.setTileData(&state);
        }
    }

    fn pose(&self, ctx: &mut BlockCtx, _partial: f32, _ticks: i32) -> Pose
    {
        let state: BuilderState = ctx.tileData();
        boxPose(ctx, state.m_area)
    }

    fn energyDemand(&self, ctx: &mut BlockCtx, _face: i32) -> Option<EnergyDemand>
    {
        let state: BuilderState = ctx.tileData();
        state.m_provider.unwrap_or_else(|| POWER.provider()).demand()
    }

    fn receiveEnergy(&self, ctx: &mut BlockCtx, _face: i32, amount: i64, simulate: bool) -> i64
    {
        let mut state: BuilderState = ctx.tileData();
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

// BlockTemplate and BlockBuilder: a wrench turns them, otherwise they open their screens
struct TurnableBlock
{
    m_faces: &'static Faces,
    m_screen: &'static str,
    m_poweredTop: bool,
}

#[block_behavior]
impl BlockBehavior for TurnableBlock
{
    fn textureFor(&self, face: i32, state: &StateData) -> Option<String> { self.m_faces.sprite(face, state) }

    fn faceTexture(&self, ctx: &mut BlockCtx, face: i32) -> Option<i32>
    {
        (self.m_poweredTop && face == dir::UP && ctx.world().isPowered(ctx.pos())).then(|| render::slot(TOP_POWERED))
    }

    fn onPlacedBy(&self, ctx: &mut BlockCtx, entity: Entity) { facePlacer(ctx, entity); }

    fn onActivated(&self, ctx: &mut BlockCtx, player: Player) -> bool
    {
        if !ctx.isAuthority()
        {
            return true;
        }

        if items::holdsWrench(player)
        {
            wrenchTurn(ctx.world(), ctx.pos());
            return true;
        }

        let _ = player.openScreen(self.m_screen, Some(ctx.pos()), Vec::new());
        true
    }
}

mod gui
{
    pub const WIDTH: i32 = 175;
    pub const HEIGHT: i32 = 225;
    pub const STANDARD_HEIGHT: i32 = 166;
    pub const SLOT: i32 = 18;
    pub const COLUMNS: i32 = 9;
    pub const TEXT: u32 = 4210752;
    // GuiTemplate on template.png
    pub const TABLE_BACKGROUND: &str = "buildcraft:gui/template";
    pub const TABLE_SLOTS: [[i32; 2]; 2] = [[55, 35], [114, 35]];
    pub const TABLE_INVENTORY: [i32; 2] = [8, 84];
    pub const TABLE_TITLE: [i32; 2] = [8, 6];
    pub const TABLE_INVENTORY_LABEL: [i32; 2] = [8, 73];
    pub const ARROW: [i32; 4] = [79, 34, 176, 14];
    pub const ARROW_LENGTH: i32 = 24;
    pub const ARROW_HEIGHT: i32 = 16;
    // GuiBuilder on builder.png
    pub const BUILDER_BACKGROUND: &str = "buildcraft:gui/builder";
    pub const BUILDER_TEMPLATE: [i32; 2] = [80, 27];
    pub const BUILDER_RESOURCES: [i32; 2] = [8, 72];
    pub const BUILDER_INVENTORY: [i32; 2] = [8, 140];
    pub const BUILDER_TITLE: [i32; 2] = [67, 15];
    pub const BUILDER_RESOURCES_LABEL: [i32; 2] = [8, 60];
    pub const BUILDER_INVENTORY_LABEL: [i32; 2] = [8, 128];
}

fn worldOfPlayer() -> Option<World>
{
    localPlayer().and_then(|player| player.info()).map(|info| World { dimension: info.world })
}

struct TableScreen
{
    m_pos: Option<BlockPos>,
}

impl Screen for TableScreen
{
    fn init(&mut self, _width: i32, _height: i32, pos: Option<BlockPos>, _data: &[u8]) { self.m_pos = pos; }

    fn draw(&mut self, draw: &mut Draw, _mouse: [i32; 2], _partial: f32)
    {
        draw.text(gui::TABLE_TITLE[0], gui::TABLE_TITLE[1], &translate("buildcraft.gui.template"), gui::TEXT);
        draw.text(
            gui::TABLE_INVENTORY_LABEL[0],
            gui::TABLE_INVENTORY_LABEL[1],
            &translate("buildcraft.gui.inventory"),
            gui::TEXT,
        );

        let (Some(pos), Some(world)) = (self.m_pos, worldOfPlayer())
        else
        {
            return;
        };
        let state: TableState = world.tileData(pos);
        let [x, y, u, v] = gui::ARROW;
        let length: i32 = state.m_time * gui::ARROW_LENGTH / COMPUTE_TICKS;
        draw.texture(gui::TABLE_BACKGROUND, x, y, u, v, length + 1, gui::ARROW_HEIGHT);
    }
}

struct BuilderScreen;

impl Screen for BuilderScreen
{
    fn draw(&mut self, draw: &mut Draw, _mouse: [i32; 2], _partial: f32)
    {
        draw.text(
            gui::BUILDER_TITLE[0],
            gui::BUILDER_TITLE[1],
            &translate("buildcraft.gui.builder_template"),
            gui::TEXT,
        );
        draw.text(
            gui::BUILDER_RESOURCES_LABEL[0],
            gui::BUILDER_RESOURCES_LABEL[1],
            &translate("buildcraft.gui.building_resources"),
            gui::TEXT,
        );
        draw.text(
            gui::BUILDER_INVENTORY_LABEL[0],
            gui::BUILDER_INVENTORY_LABEL[1],
            &translate("buildcraft.gui.inventory"),
            gui::TEXT,
        );
    }
}

fn registerMachine(
    ctx: &mut RegisterCtx,
    name: &'static str,
    faces: &'static Faces,
    hardness: f32,
    slots: i32,
    tile: impl TileBehavior + 'static,
    poweredTop: bool,
) -> Result<()>
{
    faces.reserve();
    ctx.blocks().register(
        name,
        BlockSpecBuilder::template("minecraft:block")
            .material("iron")
            .sound("metal")
            .hardness(hardness)
            .texture(faces.m_front)
            .tile(Tile::new(tile).inventory(slots).ticks().synced().model(EntityModelSpec::default()))
            .property(crate::factory::facing())
            .behavior(TurnableBlock { m_faces: faces, m_screen: name, m_poweredTop: poweredTop }),
    )?;
    Ok(())
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    render::slot(TOP_POWERED);

    registerMachine(ctx, TABLE, &TABLE_FACES, TABLE_HARDNESS, TABLE_SLOTS, TableTile, true)?;
    ctx.tileEntities().stackLimit(TABLE, 1)?;
    let table: ScreenBuilder = ScreenBuilder::new("", gui::WIDTH, gui::STANDARD_HEIGHT)
        .background(gui::TABLE_BACKGROUND)
        .slot(0, gui::TABLE_SLOTS[0][0], gui::TABLE_SLOTS[0][1])
        .outputSlot(1, gui::TABLE_SLOTS[1][0], gui::TABLE_SLOTS[1][1])
        .playerInventory(gui::TABLE_INVENTORY[0], gui::TABLE_INVENTORY[1]);
    ctx.screens().register(TABLE, table, TableScreen { m_pos: None })?;

    registerMachine(ctx, BUILDER, &BUILDER_FACES, BUILDER_HARDNESS, BUILDER_SLOTS, BuilderTile, false)?;
    let mut builder: ScreenBuilder = ScreenBuilder::new("", gui::WIDTH, gui::HEIGHT)
        .background(gui::BUILDER_BACKGROUND)
        .slot(0, gui::BUILDER_TEMPLATE[0], gui::BUILDER_TEMPLATE[1])
        .playerInventory(gui::BUILDER_INVENTORY[0], gui::BUILDER_INVENTORY[1]);
    for slot in 1..BUILDER_SLOTS
    {
        let index: i32 = slot - 1;
        let (row, column): (i32, i32) = (index / gui::COLUMNS, index % gui::COLUMNS);
        builder = builder.slot(
            slot,
            gui::BUILDER_RESOURCES[0] + column * gui::SLOT,
            gui::BUILDER_RESOURCES[1] + row * gui::SLOT,
        );
    }
    ctx.screens().register(BUILDER, builder, BuilderScreen)?;

    dye::recipe(ctx, BUILDER, items::gear::DIAMOND, |shape| shape.keyTag('C', items::material::CHEST))?;
    dye::recipe(ctx, TABLE, items::gear::DIAMOND, |shape| shape.key('C', TEMPLATE))
}
