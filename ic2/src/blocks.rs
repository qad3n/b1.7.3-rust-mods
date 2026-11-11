use crate::core::{FLAG, blockTexture, dir, own, random, sound, stacks};
use crate::world::tree;
use b173_api::prelude::*;

pub(crate) const RUBBER_WOOD: &str = "ic2:rubber_wood";
pub(crate) const RUBBER_LEAVES: &str = "ic2:rubber_leaves";
pub(crate) const RUBBER_SAPLING: &str = "ic2:rubber_sapling";
pub(crate) const RESIN: &str = "ic2:resin";
pub(crate) const IRON_FENCE: &str = "ic2:iron_fence";
pub(crate) const MINING_PIPE: &str = "ic2:mining_pipe";
pub(crate) const MINING_TIP: &str = "ic2:mining_pipe_tip";
pub(crate) const COPPER_ORE: &str = "ic2:copper_ore";
pub(crate) const TIN_ORE: &str = "ic2:tin_ore";
pub(crate) const URANIUM_ORE: &str = "ic2:uranium_ore";

// a plain cube of the IC2 sheet: BlockTex, BlockMetal and the reinforced blocks
struct Cube
{
    m_name: &'static str,
    m_material: &'static str,
    m_sound: &'static str,
    m_hardness: f32,
    m_resistance: f32,
    m_tool: Option<(ToolClass, i32)>,
    m_tag: Option<&'static str>,
}

const CUBES: [Cube; 8] = [
    cube("copper_ore", "rock", "stone", 3.0, 5.0, Some((ToolClass::Pickaxe, 1)), Some("c:ores/copper")),
    cube("tin_ore", "rock", "stone", 3.0, 5.0, Some((ToolClass::Pickaxe, 1)), Some("c:ores/tin")),
    cube("uranium_ore", "rock", "stone", 4.0, 6.0, Some((ToolClass::Pickaxe, 2)), Some("c:ores/uranium")),
    cube("reinforced_stone", "iron", "metal", 4.0, 100.0, Some((ToolClass::Pickaxe, 3)), None),
    cube("copper_block", "iron", "metal", 2.0, 10.0, Some((ToolClass::Pickaxe, 0)), Some("c:storage_blocks/copper")),
    cube("tin_block", "iron", "metal", 2.0, 10.0, Some((ToolClass::Pickaxe, 0)), Some("c:storage_blocks/tin")),
    cube("bronze_block", "iron", "metal", 2.0, 10.0, Some((ToolClass::Pickaxe, 0)), Some("c:storage_blocks/bronze")),
    cube("uranium_block", "iron", "metal", 2.0, 10.0, Some((ToolClass::Pickaxe, 0)), Some("c:storage_blocks/uranium")),
];

const fn cube(
    name: &'static str,
    material: &'static str,
    sound: &'static str,
    hardness: f32,
    resistance: f32,
    tool: Option<(ToolClass, i32)>,
    tag: Option<&'static str>,
) -> Cube
{
    Cube
    {
        m_name: name,
        m_material: material,
        m_sound: sound,
        m_hardness: hardness,
        m_resistance: resistance,
        m_tool: tool,
        m_tag: tag,
    }
}

fn drop(item: &str, chance: f32) -> Result<DropSpec>
{
    Ok(DropSpec { item: Identifier::parse(item)?, count: vec![1], meta: 0, chance })
}

fn registerCubes(ctx: &mut RegisterCtx) -> Result<()>
{
    for row in &CUBES
    {
        let mut spec: BlockSpecBuilder = BlockSpecBuilder::template("minecraft:block")
            .material(row.m_material)
            .sound(row.m_sound)
            .hardness(row.m_hardness)
            .resistance(row.m_resistance)
            .texture(&blockTexture(row.m_name));

        if let Some((class, level)) = row.m_tool
        {
            spec = spec.tool(class, level);
        }

        if let Some(tag) = row.m_tag
        {
            spec = spec.tag(tag);
        }

        // BlockTex.idDropped gives the uranium ore item instead of the block
        if row.m_name == "uranium_ore"
        {
            spec = spec.drops(vec![drop(&own("uranium_chunk"), 1.0)?]);
        }

        ctx.blocks().register(row.m_name, spec)?;
    }

    Ok(())
}

mod wood
{
    pub const REFILL_CHANCE: i32 = 200;
    pub const REFILL_TICKS: i32 = 100;
    // dropBlockAsItemWithChance adds a resin one time in six from a log the world grew
    pub const RESIN_CHANCE: i32 = 6;
    pub const RESIN: &str = "resin";
    pub const FACING: &str = "facing";
    // a resin hole only ever faces sideways
    pub const HOLE_FACES: [&str; 4] = ["north", "south", "west", "east"];
}

// BlockRubWood metadata as a named state: a player's log, a grown log without a hole, or a hole full or tapped dry
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Sap
{
    Placed,
    Plain,
    Wet,
    Dry,
}

// the resin values in state order, the first is the default a placed log takes
const RESINS: [(Sap, &str); 4] = [(Sap::Placed, "placed"), (Sap::Plain, "plain"), (Sap::Wet, "wet"), (Sap::Dry, "dry")];

fn resinName(resin: Sap) -> &'static str
{
    RESINS.iter().find(|(known, _)| *known == resin).map_or(RESINS[0].1, |(_, name)| name)
}

fn resinOf(state: &StateData) -> Sap
{
    let value: Option<&str> = state.get(wood::RESIN);
    RESINS.iter().find(|(_, name)| Some(*name) == value).map_or(Sap::Placed, |(resin, _)| *resin)
}

pub(crate) fn rubberWoodTickRate() -> i32 { wood::REFILL_TICKS }

// a rubber log's resin and the face its hole looks at, None for any other block
pub(crate) fn resinAt(world: World, pos: BlockPos) -> Option<(Sap, i32)>
{
    if !world.is(pos, RUBBER_WOOD)
    {
        return None;
    }

    let state: std::rc::Rc<StateData> = world.stateAt(pos)?;
    let face: i32 = state.get(wood::FACING).and_then(dir::parse).unwrap_or(dir::NORTH);
    Some((resinOf(&state), face))
}

// a rubber log with this resin, the hole facing face
pub(crate) fn rubberWood(resin: Sap, face: i32) -> Result<StateData>
{
    Ok(StateData::of(Identifier::parse(RUBBER_WOOD)?)
        .with(wood::RESIN, resinName(resin))
        .with(wood::FACING, dir::name(face)))
}

// the resin of the log at pos changes, its hole keeps its face
pub(crate) fn setResin(world: World, pos: BlockPos, resin: Sap)
{
    let Some(state) = world.stateAt(pos)
    else
    {
        return;
    };

    if let Err(error) = world.setBlockState(pos, (*state).clone().with(wood::RESIN, resinName(resin)))
    {
        warn(&format!("rubber wood at {pos:?} could not change its resin: {error}"));
    }
}

struct RubberWood;

#[block_behavior]
impl BlockBehavior for RubberWood
{
    // BlockRubWood.updateTick: a tapped hole fills again one tick in two hundred
    fn onUpdateTick(&self, ctx: &mut BlockCtx)
    {
        let dry: bool = ctx.stateAt().is_some_and(|state| resinOf(&state) == Sap::Dry);
        if !ctx.isAuthority() || !dry
        {
            return;
        }

        let world: World = ctx.world();
        if world.random(wood::REFILL_CHANCE) == 0
        {
            setResin(world, ctx.pos(), Sap::Wet);
            return;
        }

        if let Err(error) = world.scheduleTick(ctx.pos(), ctx.block(), wood::REFILL_TICKS)
        {
            warn(&format!("rubber wood could not schedule its refill: {error}"));
        }
    }

    fn onHarvest(&self, ctx: &mut BlockCtx, _player: Player)
    {
        let world: World = ctx.world();
        let placed: bool = ctx.stateAt().is_none_or(|state| resinOf(&state) == Sap::Placed);
        if !ctx.isAuthority() || placed || world.random(wood::RESIN_CHANCE) != 0
        {
            return;
        }

        stacks::drop(world, ctx.pos(), &ItemStack::of(own("sticky_resin").as_str(), 1));
    }
}

mod sapling
{
    pub const LIGHT: i32 = 9;
    pub const CHANCE: i32 = 30;
    pub const BONE_MEAL: i32 = 15;
}

// BlockFlower.canThisPlantGrowOnThisBlockID
fn rooted(ctx: &BlockCtx) -> bool { ctx.world().tagged(ctx.pos().offset(0, -1, 0), "minecraft:flower_soil") }

// BlockRubSapling: grows a rubber tree with light above, or at once from bone meal
struct RubberSapling;

#[block_behavior]
impl BlockBehavior for RubberSapling
{
    fn canPlaceAt(&self, ctx: &mut BlockCtx) -> bool { ctx.world().isAir(ctx.pos()) && rooted(ctx) }

    fn canStay(&self, ctx: &mut BlockCtx) -> bool { rooted(ctx) }

    fn onUpdateTick(&self, ctx: &mut BlockCtx)
    {
        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();
        if !ctx.isAuthority() || world.light(pos.offset(0, 1, 0)) < sapling::LIGHT || world.random(sapling::CHANCE) != 0
        {
            return;
        }

        tree::grow(world, pos);
    }

    fn onActivated(&self, ctx: &mut BlockCtx, player: Player) -> bool
    {
        if !ctx.isAuthority()
        {
            return false;
        }

        let Some(held) = player.held()
        else
        {
            return false;
        };

        if !held.item.is("minecraft:dye") || held.damage != sapling::BONE_MEAL
        {
            return false;
        }

        tree::grow(ctx.world(), ctx.pos());
        if let Err(error) = player.setHeld(stacks::less(held, 1))
        {
            warn(&format!("could not use up the bone meal: {error}"));
        }

        false
    }
}

mod sheet
{
    pub const NAME: &str = "rubber_sheet";
    pub const THICKNESS: f32 = 2.0;
    // BlockRubberSheet keeps in its metadata whether the last check found it spanned
    pub const SUPPORTED: &str = "supported";
    // BlockRubberSheet.onEntityCollidedWithBlock bounces what falls faster than this
    pub const FALLING: f64 = -0.4;
    pub const SIDEWAYS: f64 = 1.1;
    pub const JUMP_BOUNCE: f64 = -1.3;
    pub const SNEAK_BOUNCE: f64 = -0.1;
    pub const BOUNCE: f64 = -0.8;
}

// air reports no block info, so it is never opaque
fn opaque(world: World, pos: BlockPos) -> bool { World::blockInfo(world.block(pos)).opaque }

// BlockRubberSheet.isBlockSupporter: an opaque cube or another sheet
fn supporter(world: World, pos: BlockPos) -> bool { opaque(world, pos) || world.is(pos, sheet::NAME) }

fn sheetHeld(world: World, pos: BlockPos) -> bool
{
    [dir::WEST, dir::EAST, dir::NORTH, dir::SOUTH].iter().any(|face| supporter(world, dir::step(pos, *face)))
}

fn setSupported(ctx: &BlockCtx, supported: bool)
{
    let Some(state) = ctx.stateAt()
    else
    {
        return;
    };

    if let Err(error) = ctx.setState((*state).clone().with(sheet::SUPPORTED, supported))
    {
        warn(&format!("a rubber sheet could not keep its support: {error}"));
    }
}

// one direction of canSupportWeight: along the row of sheets until an opaque wall or a sheet resting on one
fn anchored(world: World, pos: BlockPos, step: [i32; 3]) -> bool
{
    let mut at: BlockPos = pos;

    loop
    {
        if opaque(world, at)
        {
            return true;
        }

        if !world.is(at, sheet::NAME)
        {
            return false;
        }

        if opaque(world, at.offset(0, -1, 0))
        {
            return true;
        }

        at = at.offset(step[0], step[1], step[2]);
    }
}

// BlockRubberSheet.canSupportWeight, a sheet spanned between two anchors on either axis holds a living thing
fn supportsWeight(ctx: &mut BlockCtx) -> bool
{
    if ctx.stateAt().is_some_and(|state| state.flag(sheet::SUPPORTED))
    {
        return true;
    }

    let world: World = ctx.world();
    let pos: BlockPos = ctx.pos();
    let spans: bool = (anchored(world, pos, [-1, 0, 0]) && anchored(world, pos, [1, 0, 0]))
        || (anchored(world, pos, [0, 0, -1]) && anchored(world, pos, [0, 0, 1]));
    if spans
    {
        setSupported(ctx, true);
    }

    spans
}

struct RubberSheet;

#[block_behavior]
impl BlockBehavior for RubberSheet
{
    fn canPlaceAt(&self, ctx: &mut BlockCtx) -> bool { sheetHeld(ctx.world(), ctx.pos()) }

    fn onNeighborChanged(&self, ctx: &mut BlockCtx, _neighbor: i32)
    {
        if ctx.stateAt().is_some_and(|state| state.flag(sheet::SUPPORTED))
        {
            setSupported(ctx, false);
        }

        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();
        if sheetHeld(world, pos)
        {
            return;
        }

        for stack in world.blockDrops(pos)
        {
            stacks::drop(world, pos, &stack);
        }

        let _ = world.clear(pos);
    }

    fn onEntityCollided(&self, ctx: &mut BlockCtx, entity: Entity)
    {
        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();
        if opaque(world, pos.offset(0, -1, 0))
        {
            return;
        }

        let Some(info) = entity.info()
        else
        {
            return;
        };
        let living: bool = info.health.is_some();

        if living && !supportsWeight(ctx)
        {
            let _ = world.clear(pos);
            return;
        }

        let [x, y, z] = info.motion;
        if y > sheet::FALLING
        {
            return;
        }

        entity.setField(EntityField::FallDistance, 0.0);
        let bounce: f64 = match living
        {
            true if entity.flag(EntityField::Jumping) => sheet::JUMP_BOUNCE,
            true if info.player.is_some() && entity.flag(EntityField::Sneaking) => sheet::SNEAK_BOUNCE,
            _ => sheet::BOUNCE,
        };
        let _ = entity.setMotion([x * sheet::SIDEWAYS, y * bounce, z * sheet::SIDEWAYS]);
    }
}

mod resin
{
    pub const THICKNESS: f32 = 2.0;
    pub const DROP_CHANCE: f32 = 0.8;
    pub const FALL: f32 = 0.75;
    pub const SLOW_SIDEWAYS: f64 = 0.6;
    pub const SLOW_DOWN: f64 = 0.85;
}

// BlockResin.canPlaceBlockAt: an opaque solid block below
pub(crate) fn resinHolds(world: World, pos: BlockPos) -> bool
{
    let below: BlockInfo = World::blockInfo(world.block(pos.offset(0, -1, 0)));
    below.opaque && below.solid
}

// BlockResin: a sticky puddle on an opaque solid block that slows whatever walks through
struct Resin;

#[block_behavior]
impl BlockBehavior for Resin
{
    fn canPlaceAt(&self, ctx: &mut BlockCtx) -> bool { resinHolds(ctx.world(), ctx.pos()) }

    fn canStay(&self, ctx: &mut BlockCtx) -> bool { self.canPlaceAt(ctx) }

    fn onEntityCollided(&self, _ctx: &mut BlockCtx, entity: Entity)
    {
        crate::gear::multiplyFall(entity, resin::FALL);
        let Some(info) = entity.info()
        else
        {
            return;
        };
        let [x, y, z] = info.motion;
        let _ = entity.setMotion([x * resin::SLOW_SIDEWAYS, y * resin::SLOW_DOWN, z * resin::SLOW_SIDEWAYS]);
    }
}

mod pole
{
    pub const NAME: &str = "iron_fence";
    // BlockPoleFence metadata is the magnetizer's charge, one lift per point
    pub const CHARGE: &str = "charge";
    pub const LEVELS: [&str; 16] =
        ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15"];
    pub const LIFT: f64 = 0.075;
    pub const BOOST: f64 = 1.03;
    pub const SNEAK_TOP: f64 = 0.3;
    pub const TOP: f64 = 1.5;
    pub const SNEAK_FALL: f64 = -0.25;
    pub const SNEAK_BRAKE: f64 = 0.9;
    pub const NEAR: f32 = 6.0;
    pub const FAR: f32 = 10.0;
}

const METAL_SHOES: [&str; 6] = [
    "minecraft:iron_boots",
    "minecraft:golden_boots",
    "minecraft:chainmail_boots",
    "ic2:bronze_boots",
    "ic2:nano_boots",
    "ic2:quantum_boots",
];

// the boots armor slot of the player's inventory
const BOOTS_SLOT: i32 = 36;

fn isPole(world: World, pos: BlockPos) -> bool
{
    [dir::WEST, dir::EAST, dir::NORTH, dir::SOUTH].iter().all(|face| !world.is(dir::step(pos, *face), pole::NAME))
}

fn charge(state: &StateData) -> i32 { state.get(pole::CHARGE).and_then(|charge| charge.parse().ok()).unwrap_or(0) }

// an iron fence's magnetic charge, None for any other block
pub(crate) fn fenceCharge(world: World, pos: BlockPos) -> Option<i32>
{
    if !world.is(pos, IRON_FENCE)
    {
        return None;
    }

    world.stateAt(pos).map(|state| charge(&state))
}

pub(crate) fn setFenceCharge(world: World, pos: BlockPos, charge: i32)
{
    let Some(state) = world.stateAt(pos)
    else
    {
        return;
    };

    if let Err(error) = world.setBlockState(pos, (*state).clone().with(pole::CHARGE, charge))
    {
        warn(&format!("the iron fence at {pos:?} could not change its charge: {error}"));
    }
}

// BlockPoleFence: a lone iron fence charged by a magnetizer lifts a player in metal boots, sneaking slides down
struct IronFence;

#[block_behavior]
impl BlockBehavior for IronFence
{
    fn onEntityCollided(&self, ctx: &mut BlockCtx, entity: Entity)
    {
        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();
        let Some(info) = entity.info()
        else
        {
            return;
        };

        if info.player.is_none() || !isPole(world, pos)
        {
            return;
        }

        let player: Player = Player { id: info.id };
        let metal: bool =
            player.slot(BOOTS_SLOT).is_some_and(|boots| METAL_SHOES.iter().any(|name| boots.item.is(name)));
        let charge: i32 = ctx.stateAt().map_or(0, |state| charge(&state));
        let sneaking: bool = entity.flag(EntityField::Sneaking);
        let [x, mut y, z] = info.motion;

        if charge > 0 && metal
        {
            setFenceCharge(world, pos, charge - 1);
            y += pole::LIFT;

            if y > 0.0
            {
                y *= pole::BOOST;
                entity.setField(EntityField::FallDistance, 0.0);
            }

            y = y.min(if sneaking { pole::SNEAK_TOP } else { pole::TOP });
        }
        else if sneaking
        {
            if y < pole::SNEAK_FALL
            {
                y *= pole::SNEAK_BRAKE;
            }
            else
            {
                entity.setField(EntityField::FallDistance, 0.0);
            }
        }

        let _ = entity.setMotion([x, y, z]);
    }
}

// BlockMiningPipe and BlockMiningTip only appear under a miner
struct Unplaceable;

#[block_behavior]
impl BlockBehavior for Unplaceable
{
    fn canPlaceAt(&self, _ctx: &mut BlockCtx) -> bool { false }
}

fn poleBox() -> Vec<Cuboid>
{
    vec![Cuboid { from: [pole::NEAR, 0.0, pole::NEAR], to: [pole::FAR, 16.0, pole::FAR], ..Cuboid::default() }]
}

fn slab(height: f32) -> Vec<Cuboid> { vec![Cuboid { from: [0.0; 3], to: [16.0, height, 16.0], ..Cuboid::default() }] }

// the log's front face is the resin hole, the facing state turns it to the hole's side
fn woodFaces(hole: Option<&str>) -> FaceTextures
{
    FaceTextures
    {
        top: Some(blockTexture("rubber_wood_top")),
        bottom: Some(blockTexture("rubber_wood_top")),
        side: Some(blockTexture("rubber_wood")),
        front: hole.map(blockTexture),
    }
}

fn woodStates() -> BlockStatesSpec
{
    let resins: Vec<&str> = RESINS.iter().map(|(_, name)| *name).collect();
    let hole = |resin: Sap, sprite: &str| -> StateVariantSpec
    {
        StateVariantSpec
        {
            when: [(wood::RESIN.to_owned(), resinName(resin).to_owned())].into(),
            texture: None,
            textures: Some(woodFaces(Some(sprite))),
        }
    };

    BlockStatesSpec
    {
        properties: vec![PropertySpec::new(wood::RESIN, &resins), PropertySpec::new(wood::FACING, &wood::HOLE_FACES)],
        variants: vec![hole(Sap::Wet, "rubber_wood_resin"), hole(Sap::Dry, "rubber_wood_tapped")],
        ..BlockStatesSpec::default()
    }
}

fn registerRubberTree(ctx: &mut RegisterCtx) -> Result<()>
{
    const WOOD_HARDNESS: f32 = 1.0;
    const LEAVES_HARDNESS: f32 = 0.2;
    // ColorizerFoliage.getFoliageColorBirch
    const BIRCH: u32 = 0x80A755;
    const SAPLING_CHANCE: f32 = 1.0 / 35.0;

    ctx.blocks().register(
        "rubber_wood",
        BlockSpecBuilder::template("minecraft:block")
            .material("wood")
            .sound("wood")
            .hardness(WOOD_HARDNESS)
            .faces(woodFaces(None))
            .tool(ToolClass::Axe, 0)
            .tickRandomly()
            .sustainsLeaves()
            .tag("minecraft:logs")
            .drops(vec![DropSpec { item: Identifier::parse(RUBBER_WOOD)?, count: vec![1], meta: 0, chance: 1.0 }])
            .behavior(RubberWood),
    )?;

    ctx.blocks().states("rubber_wood", woodStates())?;

    ctx.blocks().register(
        "rubber_leaves",
        BlockSpecBuilder::template("minecraft:leaves")
            .texture("minecraft:block/oak_leaves")
            .color(BIRCH)
            .hardness(LEAVES_HARDNESS)
            .opacity(1)
            .sound("grass")
            .drops(vec![drop(RUBBER_SAPLING, SAPLING_CHANCE)?]),
    )?;

    // BlockRubSapling keeps the vanilla sapling picture
    ctx.blocks().register(
        "rubber_sapling",
        BlockSpecBuilder::template("minecraft:red_flower")
            .texture("minecraft:block/oak_sapling")
            .hardness(0.0)
            .sound("grass")
            .tickRandomly()
            .tag("minecraft:saplings")
            .behavior(RubberSapling),
    )?;
    Ok(())
}

fn registerSpecial(ctx: &mut RegisterCtx) -> Result<()>
{
    const RESIN_HARDNESS: f32 = 1.6;
    const RESIN_RESISTANCE: f32 = 0.5;
    const SHEET_HARDNESS: f32 = 0.8;
    const SHEET_RESISTANCE: f32 = 2.0;
    const FENCE_HARDNESS: f32 = 1.5;
    const FENCE_RESISTANCE: f32 = 5.0;
    const PIPE_HARDNESS: f32 = 6.0;
    const PIPE_RESISTANCE: f32 = 10.0;
    const GLASS_HARDNESS: f32 = 1.0;
    const GLASS_RESISTANCE: f32 = 100.0;

    ctx.blocks().register(
        "resin",
        BlockSpecBuilder::template("minecraft:block")
            .material("circuits")
            .sound("cloth")
            .hardness(RESIN_HARDNESS)
            .resistance(RESIN_RESISTANCE)
            .texture(&blockTexture("resin"))
            .opacity(0)
            .model(BlockModelSpec { cuboids: slab(resin::THICKNESS), ..BlockModelSpec::default() })
            .collision(Vec::new())
            .drops(vec![drop(&own("sticky_resin"), resin::DROP_CHANCE)?])
            .behavior(Resin),
    )?;

    ctx.blocks().register(
        "rubber_sheet",
        BlockSpecBuilder::template("minecraft:block")
            .material("cloth")
            .sound("cloth")
            .hardness(SHEET_HARDNESS)
            .resistance(SHEET_RESISTANCE)
            .texture(&blockTexture("rubber_sheet"))
            .opacity(0)
            .model(BlockModelSpec { cuboids: slab(sheet::THICKNESS), ..BlockModelSpec::default() })
            .collision(slab(sheet::THICKNESS))
            .behavior(RubberSheet),
    )?;
    ctx.blocks().states(
        sheet::NAME,
        BlockStatesSpec { properties: vec![PropertySpec::new(sheet::SUPPORTED, &FLAG)], ..BlockStatesSpec::default() },
    )?;

    // BlockPoleFence takes the iron furnace top sprite and lets entities into its pole
    ctx.blocks().register(
        "iron_fence",
        BlockSpecBuilder::template("minecraft:fence")
            .material("iron")
            .sound("metal")
            .hardness(FENCE_HARDNESS)
            .resistance(FENCE_RESISTANCE)
            .texture(&blockTexture("iron_furnace_top"))
            .tool(ToolClass::Pickaxe, 0)
            .collision(poleBox())
            .behavior(IronFence),
    )?;
    ctx.blocks().states(
        pole::NAME,
        BlockStatesSpec
        {
            properties: vec![PropertySpec::new(pole::CHARGE, &pole::LEVELS)],
            ..BlockStatesSpec::default()
        },
    )?;

    ctx.blocks().register(
        "mining_pipe",
        BlockSpecBuilder::template("minecraft:fence")
            .material("iron")
            .sound("metal")
            .hardness(PIPE_HARDNESS)
            .resistance(PIPE_RESISTANCE)
            .texture(&blockTexture("mining_pipe"))
            .tool(ToolClass::Pickaxe, 0)
            .collision(poleBox())
            .behavior(Unplaceable),
    )?;

    ctx.blocks().register(
        "mining_pipe_tip",
        BlockSpecBuilder::template("minecraft:block")
            .material("iron")
            .sound("metal")
            .hardness(PIPE_HARDNESS)
            .resistance(PIPE_RESISTANCE)
            .texture(&blockTexture("mining_pipe_tip"))
            .tool(ToolClass::Pickaxe, 0)
            .behavior(Unplaceable),
    )?;

    ctx.blocks().register(
        "reinforced_glass",
        BlockSpecBuilder::template("minecraft:glass")
            .hardness(GLASS_HARDNESS)
            .resistance(GLASS_RESISTANCE)
            .sound("glass")
            .texture(&blockTexture("reinforced_glass"))
            .opacity(0),
    )?;
    Ok(())
}

// the treetap's sound and resin when it opens a hole, shared with nothing else but kept beside the wood rules
pub(crate) fn tapSound(world: World, pos: BlockPos) { world.playSound(dir::center(pos), &sound("treetap"), 1.0, 1.0); }

pub(crate) fn tapChance(world: World) -> bool
{
    const ONE_IN: i32 = 5;

    random::float(world) < 1.0 / f64::from(ONE_IN)
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    registerCubes(ctx)?;
    registerRubberTree(ctx)?;
    registerSpecial(ctx)?;
    registerDoor(ctx)
}

mod door
{
    pub const NAME: &str = "reinforced_door_block";
    pub const HARDNESS: f32 = 10.0;
    pub const RESISTANCE: f32 = 100.0;
    pub const TURNS: f64 = 4.0;
    pub const HALF_TURN: f64 = 180.0;
    pub const FULL_TURN: f64 = 360.0;
    pub const ROUND: f64 = 0.5;
    pub const BUILD_LIMIT: i32 = 127;
    // BlockDoor metadata as states, listed so each state's index is the metadata the door template reads
    pub const HALF: &str = "half";
    pub const HALVES: [&str; 2] = ["lower", "upper"];
    pub const OPEN: &str = "open";
    pub const FACING: &str = "facing";
    pub const FACINGS: [&str; 4] = ["east", "south", "west", "north"];
}

fn doorFits(world: World, pos: BlockPos) -> bool
{
    pos.y < door::BUILD_LIMIT
        && opaque(world, pos.offset(0, -1, 0))
        && world.isAir(pos)
        && world.isAir(pos.offset(0, 1, 0))
}

fn normalCube(world: World, pos: BlockPos) -> i32 { i32::from(opaque(world, pos)) }

fn doorState(half: &str, open: bool, facing: usize) -> Result<StateHandle>
{
    let state: StateData = StateData::of(Identifier::parse(&own(door::NAME))?)
        .with(door::HALF, half)
        .with(door::OPEN, open)
        .with(door::FACING, door::FACINGS[facing % door::FACINGS.len()]);
    Ok(StateHandle::new(state))
}

// ItemIC2Door.onItemUse: ItemDoor's placement, hinged toward the wall or the neighbouring door
struct DoorItem;

#[item_behavior]
impl ItemBehavior for DoorItem
{
    fn onUseOnBlock(&self, world: World, player: Player, stack: ItemStack, pos: BlockPos, face: i32) -> Use
    {
        let at: BlockPos = pos.offset(0, 1, 0);
        let Some(info) = player.info()
        else
        {
            return Use::Pass;
        };

        if face != dir::UP || !doorFits(world, at)
        {
            return Use::Pass;
        }

        let turns: f64 = (f64::from(info.yaw) + door::HALF_TURN) * door::TURNS / door::FULL_TURN - door::ROUND;
        let quarter: usize = (turns.floor() as i32).rem_euclid(door::FACINGS.len() as i32) as usize;
        let (dx, dz): (i32, i32) = match quarter
        {
            0 => (0, 1),
            1 => (-1, 0),
            2 => (0, -1),
            _ => (1, 0),
        };
        let left: i32 = normalCube(world, at.offset(-dx, 0, -dz)) + normalCube(world, at.offset(-dx, 1, -dz));
        let right: i32 = normalCube(world, at.offset(dx, 0, dz)) + normalCube(world, at.offset(dx, 1, dz));
        let doorLeft: bool =
            world.is(at.offset(-dx, 0, -dz), door::NAME) || world.is(at.offset(-dx, 1, -dz), door::NAME);
        let doorRight: bool = world.is(at.offset(dx, 0, dz), door::NAME) || world.is(at.offset(dx, 1, dz), door::NAME);

        // ItemDoor hinges a mirrored door by turning it back a quarter and setting its open bit
        let mirrored: bool = doorLeft && !doorRight || right > left;
        let facing: usize = if mirrored
        {
            quarter + door::FACINGS.len() - 1
        }
        else
        {
            quarter
        };
        let halves: Result<Vec<StateHandle>> =
            door::HALVES.iter().map(|half| doorState(half, mirrored, facing)).collect();
        let Ok(halves) = halves
        else
        {
            return Use::Pass;
        };

        for (rise, half) in halves.iter().enumerate()
        {
            if let Err(error) = half.place(world, at.offset(0, rise as i32, 0), false)
            {
                warn(&format!("could not place the reinforced door: {error}"));
            }
        }

        for (rise, half) in halves.iter().enumerate()
        {
            let _ = world.notifyNeighbors(at.offset(0, rise as i32, 0), half.block());
        }

        Use::Done(stacks::less(stack, 1))
    }
}

pub(crate) fn registerDoor(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.blocks().register(
        door::NAME,
        BlockSpecBuilder::template("minecraft:door")
            .material("iron")
            .sound("metal")
            .hardness(door::HARDNESS)
            .resistance(door::RESISTANCE)
            .texture(&blockTexture("reinforced_door_bottom"))
            .tool(ToolClass::Pickaxe, 3)
            .drops(vec![drop(&own("reinforced_door"), 1.0)?]),
    )?;
    ctx.blocks().states(
        door::NAME,
        BlockStatesSpec
        {
            properties: vec![
                PropertySpec::new(door::HALF, &door::HALVES),
                PropertySpec::new(door::OPEN, &FLAG),
                PropertySpec::new(door::FACING, &door::FACINGS),
            ],
            variants: vec![StateVariantSpec
            {
                when: [(door::HALF.to_owned(), door::HALVES[1].to_owned())].into(),
                texture: Some(blockTexture("reinforced_door_top")),
                textures: None,
            }],
            ..BlockStatesSpec::default()
        },
    )?;
    ctx.items().register(
        "reinforced_door",
        ItemSpecBuilder::new().texture(&crate::core::itemTexture("reinforced_door")).behavior(DoorItem),
    )?;
    Ok(())
}
