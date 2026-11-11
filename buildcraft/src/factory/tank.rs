use crate::core::render::{self, Scene};
use b173_api::prelude::*;
use std::rc::Rc;

// TileTank: sixteen buckets each, a column of tanks fills from the bottom and empties from the top
const NAME: &str = "tank";
pub(crate) const CAPACITY: i32 = 16000;
const HARDNESS: f32 = 5.0;
const TOP: &str = "buildcraft:block/tank_top";
const SIDE: &str = "buildcraft:block/tank_side";
const JOINED: &str = "buildcraft:block/tank_side_joined";
const OIL_SPRITE: &str = "buildcraft:block/oil_still";

mod shape
{
    // BlockTank's bounds and RenderTank's liquid inside them
    pub const FROM: [f32; 3] = [2.0, 0.0, 2.0];
    pub const TO: [f32; 3] = [14.0, 16.0, 14.0];
    pub const LIQUID_MIN: f32 = 0.135;
    pub const LIQUID_MAX: f32 = 0.865;
    pub const STAGES: f32 = 100.0;
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub(crate) struct TankState
{
    pub(crate) m_fluid: Option<Identifier>,
    pub(crate) m_stored: i32,
}

fn isTank(world: World, pos: BlockPos) -> bool { world.is(pos, NAME) }

fn state(world: World, pos: BlockPos) -> TankState { world.tileData(pos) }

fn save(world: World, pos: BlockPos, tank: &TankState)
{
    if let Err(error) = world.setTileData(pos, tank)
    {
        warn(&format!("the tank at {pos:?} could not save: {error}"));
    }
}

// TileTank.actualFill: this tank first, the rest overflowing into the tank above
fn fillUp(world: World, pos: BlockPos, stack: &FluidStack, simulate: bool) -> i32
{
    let mut tank: TankState = state(world, pos);
    let foreign: bool = tank.m_stored > 0 && tank.m_fluid.as_ref().is_some_and(|fluid| *fluid != stack.fluid);
    let used: i32 = if foreign
    {
        0
    }
    else
    {
        stack.amount.min(CAPACITY - tank.m_stored).max(0)
    };

    if !simulate && used > 0
    {
        tank.m_stored += used;
        tank.m_fluid = Some(stack.fluid.clone());
        save(world, pos, &tank);
    }

    let above: BlockPos = pos.offset(0, 1, 0);
    if used < stack.amount && isTank(world, above)
    {
        let rest: FluidStack = FluidStack { fluid: stack.fluid.clone(), amount: stack.amount - used };
        return used + fillUp(world, above, &rest, simulate);
    }

    used
}

// TileTank.actualEmtpy: this tank first, the rest from the tank below
// an empty tank on top passes the drain down, as BuildCraft's empty tanks hold zero
fn drainDown(world: World, pos: BlockPos, max: i32, simulate: bool) -> Option<FluidStack>
{
    let mut tank: TankState = state(world, pos);
    let taken: i32 = if tank.m_fluid.is_some()
    {
        tank.m_stored.min(max)
    }
    else
    {
        0
    };

    if !simulate && taken > 0
    {
        tank.m_stored -= taken;
        save(world, pos, &tank);
    }

    let below: BlockPos = pos.offset(0, -1, 0);
    // a tank of another fluid below stops the drain instead of being emptied into this one's stack
    let joined: bool = isTank(world, below)
        && match (&tank.m_fluid, state(world, below).m_fluid)
        {
            (Some(fluid), Some(other)) => taken == 0 || *fluid == other,
            _ => true,
        };
    let rest: Option<FluidStack> = if taken < max && joined
    {
        drainDown(world, below, max - taken, simulate)
    }
    else
    {
        None
    };

    let fluid: Identifier = match (&tank.m_fluid, &rest)
    {
        (Some(fluid), _) if taken > 0 => fluid.clone(),
        (_, Some(rest)) => rest.fluid.clone(),
        (fluid, None) => fluid.clone()?,
    };
    Some(FluidStack { fluid, amount: taken + rest.map_or(0, |rest| rest.amount) })
}

struct TankTile;

#[tile_behavior]
impl TileBehavior for TankTile
{
    fn fillFluid(&self, ctx: &mut BlockCtx, _face: i32, stack: &FluidStack, simulate: bool) -> i32
    {
        let world: World = ctx.world();
        let mut bottom: BlockPos = ctx.pos();

        while bottom.y - 1 > 1 && isTank(world, bottom.offset(0, -1, 0))
        {
            bottom = bottom.offset(0, -1, 0);
        }

        fillUp(world, bottom, stack, simulate)
    }

    fn drainFluid(&self, ctx: &mut BlockCtx, _face: i32, max: i32, simulate: bool) -> Option<FluidStack>
    {
        let world: World = ctx.world();
        let mut top: BlockPos = ctx.pos();

        while isTank(world, top.offset(0, 1, 0))
        {
            top = top.offset(0, 1, 0);
        }

        drainDown(world, top, max, simulate).filter(|stack| stack.amount > 0)
    }

    // RenderTank, the liquid rising inside the glass
    fn pose(&self, ctx: &mut BlockCtx, _partial: f32, _ticks: i32) -> Pose
    {
        let world: World = ctx.world();
        let tank: TankState = state(world, ctx.pos());
        if tank.m_stored <= 0
        {
            return Pose::default();
        }

        let stage: f32 = (tank.m_stored as f32 / CAPACITY as f32 * (shape::STAGES - 1.0)).floor() / shape::STAGES;
        let mut draw: Scene = Scene::new();
        draw.cube(
            [shape::LIQUID_MIN, 0.0, shape::LIQUID_MIN],
            [shape::LIQUID_MAX, stage, shape::LIQUID_MAX],
            render::uv(OIL_SPRITE),
            render::lightAt(world, ctx.pos()),
        );
        draw.finish()
    }
}

// BlockTank.getBlockTexture, a side that sits on another tank shows the joined glass
struct TankBlock;

#[block_behavior]
impl BlockBehavior for TankBlock
{
    fn faceTexture(&self, ctx: &mut BlockCtx, face: i32) -> Option<i32>
    {
        let sprite: &'static str = match face
        {
            0 | 1 => TOP,
            _ if isTank(ctx.world(), ctx.pos().offset(0, -1, 0)) => JOINED,
            _ => SIDE,
        };
        Some(render::slot(sprite))
    }

    fn texture(&self, face: i32, _meta: i32) -> i32 { render::slot(if face <= 1 { TOP } else { SIDE }) }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for sprite in [TOP, SIDE, JOINED]
    {
        render::slot(sprite);
    }

    ctx.tileEntities().register(
        NAME,
        TileEntitySpec { inventory: 0, ticks: false, sync: true, behavior: None },
        Some(Rc::new(TankTile)),
    )?;
    ctx.tileEntities().model(NAME, EntityModelSpec::default())?;
    ctx.blocks().register(
        NAME,
        BlockSpecBuilder::template("minecraft:block")
            .material("iron")
            .sound("glass")
            .hardness(HARDNESS)
            .texture(SIDE)
            .opacity(0)
            .model(BlockModelSpec
            {
                cuboids: vec![Cuboid
                {
                    from: shape::FROM,
                    to: shape::TO,
                    texture: None,
                    metas: Vec::new(),
                    connect: None,
                }],
                ..BlockModelSpec::default()
            })
            .tileEntity(NAME)
            .behavior(TankBlock),
    )?;

    let cuboid: Cuboid = Cuboid { from: shape::FROM, to: shape::TO, texture: None, metas: Vec::new(), connect: None };
    ctx.blocks().itemModel(NAME, ItemModelSpec { cuboids: vec![cuboid], ..ItemModelSpec::default() })?;

    ctx.recipes()
        .shaped(NAME, ItemStack::of("buildcraft:tank", 1))
        .pattern(["ggg", "g g", "ggg"])
        .key('g', "minecraft:glass")
        .build()
}
