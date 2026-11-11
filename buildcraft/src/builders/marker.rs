use crate::core::area::Area;
use crate::core::render::{self, Laser, Scene};
use crate::core::{dir, stacks, state};
use b173_api::prelude::*;
use std::rc::Rc;

// TileMarker: landmarks a block apart on the three axes link into one box, drawn with red lasers
pub(crate) const NAME: &str = "marker";
const BLOCK: &str = "buildcraft:marker";
const MAX_SIZE: i32 = 64;
const LIGHT: f32 = 0.5;

// TileMarker.Origin, kept by every linked marker so any of them answers for the box
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub(crate) struct Origin
{
    m_origin: Option<BlockPos>,
    m_axes: [Option<BlockPos>; 3],
}

impl Origin
{
    fn set(&self) -> bool { self.m_origin.is_some() }

    fn members(&self) -> Vec<BlockPos>
    {
        self.m_origin.into_iter().chain(self.m_axes.iter().flatten().copied()).collect()
    }

    // TileMarker.createLasers' box, each linked axis stretching it toward its marker
    pub(crate) fn area(&self) -> Option<Area>
    {
        let origin: BlockPos = self.m_origin?;
        let corner: [i32; 3] = [origin.x, origin.y, origin.z];
        let mut min: [i32; 3] = corner;
        let mut max: [i32; 3] = corner;

        for (axis, linked) in self.m_axes.iter().enumerate()
        {
            if let Some(linked) = linked
            {
                let at: i32 = [linked.x, linked.y, linked.z][axis];
                min[axis] = min[axis].min(at);
                max[axis] = max[axis].max(at);
            }
        }

        Some(Area::new(min, max))
    }
}

fn isMarker(world: World, pos: BlockPos) -> bool { world.is(pos, BLOCK) }

fn origin(world: World, pos: BlockPos) -> Origin { world.tileData(pos) }

fn store(world: World, pos: BlockPos, origin: &Origin)
{
    if let Err(error) = world.setTileData(pos, origin)
    {
        warn(&format!("the landmark at {pos:?} could not save: {error}"));
    }
}

// every marker of the box keeps the same origin
fn share(world: World, origin: &Origin)
{
    for member in origin.members()
    {
        store(world, member, origin);
    }
}

// TileMarker.linkTo
fn link(world: World, pos: BlockPos, other: BlockPos, axis: usize) -> bool
{
    let mine: Origin = origin(world, pos);
    let theirs: Origin = origin(world, other);
    if mine.set() && theirs.set()
    {
        return false;
    }

    let joined: Origin = match (mine.set(), theirs.set())
    {
        (false, false) =>
        {
            let mut fresh: Origin = Origin { m_origin: Some(pos), ..Origin::default() };
            fresh.m_axes[axis] = Some(other);
            fresh
        }

        (false, true) =>
        {
            let mut joined: Origin = theirs;
            joined.m_axes[axis] = Some(pos);
            joined
        }

        _ =>
        {
            let mut joined: Origin = mine;
            joined.m_axes[axis] = Some(other);
            joined
        }
    };

    share(world, &joined);
    true
}

// TileMarker.setVect, the nearest landmark along an axis in either direction
fn search(world: World, pos: BlockPos, axis: usize)
{
    for distance in 1..MAX_SIZE
    {
        for sign in [1, -1]
        {
            let mut step: [i32; 3] = [0; 3];
            step[axis] = distance * sign;
            let other: BlockPos = pos.offset(step[0], step[1], step[2]);

            if isMarker(world, other) && link(world, pos, other, axis)
            {
                return;
            }
        }
    }
}

// TileMarker.tryConnection on a click
fn connect(world: World, pos: BlockPos)
{
    for axis in 0..3
    {
        let current: Origin = origin(world, pos);
        if !current.set() || current.m_axes[axis].is_none()
        {
            search(world, pos, axis);
        }
    }
}

// TileMarker.destroy: the other markers of a broken landmark's box go back to standing alone
fn unlink(world: World, pos: BlockPos)
{
    let current: Origin = origin(world, pos);

    for member in current.members().into_iter().filter(|member| *member != pos)
    {
        store(world, member, &Origin::default());
    }
}

// IAreaProvider for the machines: a landmark beside them gives its box, or itself alone
pub(crate) fn areaNear(world: World, pos: BlockPos) -> Option<(BlockPos, Area)>
{
    [dir::EAST, dir::WEST, dir::SOUTH, dir::NORTH, dir::UP, dir::DOWN]
        .into_iter()
        .map(|face| dir::step(pos, face))
        .find(|near| isMarker(world, *near))
        .map(|near|
        {
            let area: Area =
                origin(world, near).area().unwrap_or(Area::new([near.x, near.y, near.z], [near.x, near.y, near.z]));
            (near, area)
        })
}

// TileMarker.removeFromWorld, the machine took the box so its landmarks pop off as items
pub(crate) fn remove(world: World, pos: BlockPos)
{
    let current: Origin = origin(world, pos);
    if !current.set()
    {
        return;
    }

    for member in current.members()
    {
        stacks::breakBlock(world, member);
    }
}

// TileMarker.switchSignals: a powered landmark shows blue lasers along the axes it can still link on
fn signals(draw: &mut Scene, world: World, pos: BlockPos, current: &Origin, light: f32)
{
    if !world.isPowered(pos)
    {
        return;
    }

    let reach: f64 = f64::from(MAX_SIZE - 1);

    for axis in 0..3
    {
        if current.set() && current.m_axes[axis].is_some()
        {
            continue;
        }

        let mut far: [f64; 3] = [0.0; 3];
        far[axis] = reach;
        let near: [f64; 3] = far.map(|value| -value);
        draw.laser([0.0; 3], far, Laser::Blue, light);
        draw.laser(near, [0.0; 3], Laser::Blue, light);
    }
}

struct MarkerTile;

#[tile_behavior]
impl TileBehavior for MarkerTile
{
    fn onRemoved(&self, ctx: &mut BlockCtx)
    {
        if ctx.isAuthority()
        {
            unlink(ctx.world(), ctx.pos());
        }
    }

    fn pose(&self, ctx: &mut BlockCtx, _partial: f32, _ticks: i32) -> Pose
    {
        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();
        let current: Origin = origin(world, pos);
        let light: f32 = render::lightAt(world, pos);
        let mut draw: Scene = Scene::new();

        // a lit marker draws at full brightness, as the block renderer does for any light giving block
        let support: i32 = state::face(ctx.stateAt(), super::landmark::SUPPORT).unwrap_or(dir::UP);
        draw.quads(super::landmark::quads(support), 1.0);
        signals(&mut draw, world, pos, &current, light);

        // the origin marker draws the box once for the whole group
        if current.m_origin == Some(pos)
            && let Some(area) = current.area()
        {
            draw.laserBox(render::relative(pos, area.min()), render::relative(pos, area.max()), Laser::Red, light);
        }

        draw.finish()
    }
}

// BlockMarker keeps the torch's ray trace but none of its placement, flames or ticks
struct MarkerBlock;

#[block_behavior]
impl BlockBehavior for MarkerBlock
{
    fn onActivated(&self, ctx: &mut BlockCtx, _player: Player) -> bool
    {
        if ctx.isAuthority()
        {
            connect(ctx.world(), ctx.pos());
        }

        true
    }

    fn canPlaceAt(&self, ctx: &mut BlockCtx) -> bool { super::landmark::canPlace(ctx.world(), ctx.pos()) }

    fn canStay(&self, ctx: &mut BlockCtx) -> bool { super::landmark::canPlace(ctx.world(), ctx.pos()) }

    fn onAdded(&self, ctx: &mut BlockCtx) { super::landmark::added(ctx); }

    fn onPlaced(&self, ctx: &mut BlockCtx, face: i32) { super::landmark::placed(ctx, face); }

    fn onNeighborChanged(&self, ctx: &mut BlockCtx, _neighbor: i32) { super::landmark::neighborChanged(ctx); }

    fn onRandomDisplayTick(&self, _ctx: &mut BlockCtx) {}

    fn onUpdateTick(&self, _ctx: &mut BlockCtx) {}
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.tileEntities().register(
        NAME,
        TileEntitySpec { inventory: 0, ticks: false, sync: true, behavior: None },
        Some(Rc::new(MarkerTile)),
    )?;
    ctx.tileEntities().model(NAME, EntityModelSpec::default())?;
    ctx.blocks().register(
        NAME,
        BlockSpecBuilder::template("minecraft:torch")
            .material("circuits")
            .texture(super::landmark::SPRITE)
            .light(LIGHT)
            .collision(Vec::new())
            .tileEntity(NAME)
            .behavior(MarkerBlock),
    )?;
    super::landmark::register(ctx, NAME)?;

    ctx.recipes()
        .shaped(NAME, ItemStack::of(BLOCK, 1))
        .pattern(["l ", "r "])
        .keyStack('l', ItemStack::of("minecraft:dye", 1).withDamage(super::dye::LAPIS))
        .key('r', "minecraft:redstone_torch")
        .build()
}
