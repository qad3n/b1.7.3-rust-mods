use super::{Kind, base, put, vanillaTexture};
use crate::names::block;
use b173_api::math::Face;
use b173_api::prelude::*;

// BlockTFGiantBlock: one item fills a 4x4x4 cell aligned to the grid, breaking any part clears the whole cell
mod cell
{
    pub const SIZE: i32 = 4;
}

struct Giant;

fn origin(pos: BlockPos) -> BlockPos
{
    let size: i32 = cell::SIZE;
    BlockPos::new(pos.x.div_euclid(size) * size, pos.y.div_euclid(size) * size, pos.z.div_euclid(size) * size)
}

fn cells(pos: BlockPos) -> impl Iterator<Item = BlockPos>
{
    let start: BlockPos = origin(pos);
    let size: i32 = cell::SIZE;
    (0..size).flat_map(move |dx| (0..size).flat_map(move |dy| (0..size).map(move |dz| start.offset(dx, dy, dz))))
}

#[block_behavior]
impl BlockBehavior for Giant
{
    fn canPlaceAt(&self, ctx: &mut BlockCtx) -> bool
    {
        let world: World = ctx.world();
        cells(ctx.pos()).all(|at| World::blockInfo(world.block(at)).replaceable || world.isAir(at))
    }

    fn onPlacedBy(&self, ctx: &mut BlockCtx, _entity: Entity)
    {
        if !ctx.isAuthority()
        {
            return;
        }

        for at in cells(ctx.pos())
        {
            let _ = ctx.world().setBlockQuiet(at, ctx.block(), 0);
        }
    }

    fn onDestroyedByPlayer(&self, ctx: &mut BlockCtx)
    {
        if !ctx.isAuthority()
        {
            return;
        }

        let world: World = ctx.world();
        for at in cells(ctx.pos()).filter(|at| *at != ctx.pos() && world.block(*at) == ctx.block())
        {
            let _ = world.setBlock(at, 0, 0);
        }
    }

    fn onDestroyedByExplosion(&self, ctx: &mut BlockCtx) { self.onDestroyedByPlayer(ctx) }

    fn faceTexture(&self, ctx: &mut BlockCtx, face: i32) -> Option<i32>
    {
        let row: &GiantRow = rowOf(ctx.block())?;
        let (column, line): (i32, i32) = tile(ctx.pos(), face);
        b173_api::world::terrainSlot(&b173_api::world::tileName(&faceSprite(row, face), column, line))
    }
}

struct GiantRow
{
    m_name: &'static str,
    m_label: &'static str,
    m_kind: Kind,
    m_hardness: f32,
    m_resistance: Option<f32>,
    m_top: &'static str,
    m_side: &'static str,
}

// the vanilla sprites the giant blocks blow up, cobblestone log leaves and obsidian
const GIANTS: [GiantRow; 4] = [
    GiantRow
    {
        m_name: block::GIANT_COBBLE,
        m_label: "Giant Cobblestone",
        m_kind: Kind::Rock,
        m_hardness: 128.0,
        m_resistance: Some(10.0),
        m_top: "cobblestone",
        m_side: "cobblestone",
    },
    GiantRow
    {
        m_name: block::GIANT_LOG,
        m_label: "Giant Log",
        m_kind: Kind::Wood,
        m_hardness: 128.0,
        m_resistance: None,
        m_top: "oak_log_top",
        m_side: "oak_log",
    },
    GiantRow
    {
        m_name: block::GIANT_LEAVES,
        m_label: "Giant Leaves",
        m_kind: Kind::Plant,
        m_hardness: 12.8,
        m_resistance: None,
        m_top: "oak_leaves",
        m_side: "oak_leaves",
    },
    GiantRow
    {
        m_name: block::GIANT_OBSIDIAN,
        m_label: "Giant Obsidian",
        m_kind: Kind::Rock,
        m_hardness: 204800.0,
        m_resistance: Some(8192000.0),
        m_top: "obsidian",
        m_side: "obsidian",
    },
];

// GiantBlockIcon, a quarter of a quarter of the vanilla face blown up to a whole face, picked by the block's spot
fn tile(pos: BlockPos, face: i32) -> (i32, i32)
{
    let last: i32 = cell::SIZE - 1;
    let [x, y, z] = [pos.x & last, pos.y & last, pos.z & last];

    match Face::fromIndex(face)
    {
        Some(Face::North) => (last - x, last - y),
        Some(Face::South) => (x, last - y),
        Some(Face::West) => (z, last - y),
        Some(Face::East) => (last - z, last - y),
        _ => (x, z),
    }
}

fn faceSprite(row: &GiantRow, face: i32) -> String { vanillaTexture(if face < 2 { row.m_top } else { row.m_side }) }

fn rowOf(block: i32) -> Option<&'static GiantRow> { GIANTS.iter().find(|row| blockIs(block, row.m_name)) }

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for row in &GIANTS
    {
        for texture in [vanillaTexture(row.m_top), vanillaTexture(row.m_side)]
        {
            b173_api::world::terrainTiles(&texture, cell::SIZE, cell::SIZE)?;
        }

        let mut spec: BlockSpecBuilder = base("minecraft:block", row.m_label, row.m_kind, row.m_hardness)
            .faces(FaceTextures
            {
                top: Some(vanillaTexture(row.m_top)),
                bottom: Some(vanillaTexture(row.m_top)),
                side: Some(vanillaTexture(row.m_side)),
                front: None,
            })
            .behavior(Giant)
            .tag(crate::tags::GIANT);

        if let Some(resistance) = row.m_resistance
        {
            spec = spec.resistance(resistance);
        }

        if row.m_name == block::GIANT_LEAVES
        {
            spec = spec.material("leaves").opacity(1);
        }

        put(ctx, row.m_name, spec)?;
    }

    Ok(())
}
