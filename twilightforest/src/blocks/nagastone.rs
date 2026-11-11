use super::{Kind, base, texture};
use crate::names::{self, block};
use b173_api::math::Face;
use b173_api::prelude::*;

// BlockTFNagastone: one block in the source whose sixteen metas are a head turned four ways, eight bends down or up,
// three straight runs and a flat bend; heads and bodies are their own blocks here because each has its own item,
// and neighbor changes refit every piece to the stones around it

mod naga
{
    pub const HARDNESS: f32 = 1.5;
    pub const RESISTANCE: f32 = 10.0;
    pub const HEADS: i32 = 4;
    pub const BODIES: i32 = 12;
    // the source's first body meta and the meta its body item carries
    pub const FIRST_BODY: i32 = 4;
    pub const BODY_ITEM: i32 = 13;
    pub const HEAD_ITEM: i32 = 1;
    pub const STRAIGHT: i32 = 12;
    pub const DOWN_BEND: i32 = 4;
    pub const UP_BEND: i32 = 8;
    pub const FLAT_BEND: i32 = 15;
}

#[derive(Clone, Copy)]
enum Sprite
{
    FaceLeft,
    FaceRight,
    Cross,
    FaceFront,
    TopTip,
    TopLong,
    BottomTip,
    BottomLong,
    LeftDown,
    RightDown,
    LeftUp,
    RightUp,
    TipLeft,
    LongSide,
    TipRight,
    TurnTop,
}

const SPRITES: [&str; 16] = [
    "nagastone_face_left",
    "nagastone_face_right",
    "nagastone_cross_section",
    "nagastone_face_front",
    "nagastone_top_tip",
    "nagastone_tip_long",
    "nagastone_bottom_tip",
    "nagastone_bottom_long",
    "nagastone_left_down",
    "nagastone_right_down",
    "nagastone_left_up",
    "nagastone_right_up",
    "nagastone_tip_left",
    "nagastone_long_side",
    "nagastone_tip_right",
    "nagastone_turn_top",
];

// BlockTFNagastone.getIcon and RenderBlockTFNagastone.setRenderRotate by source meta, faces in index order
struct Shape
{
    m_faces: [Sprite; 6],
    m_turns: [u8; 6],
}

const SHAPES: [Shape; 16] =
{
    use Sprite::*;
    [
        Shape { m_faces: [BottomTip, TopTip, FaceFront, Cross, FaceLeft, FaceRight], m_turns: [2, 1, 0, 0, 0, 0] },
        Shape { m_faces: [BottomTip, TopTip, Cross, FaceFront, FaceRight, FaceLeft], m_turns: [1, 2, 0, 0, 0, 0] },
        Shape { m_faces: [BottomTip, TopTip, FaceRight, FaceLeft, FaceFront, Cross], m_turns: [3, 0, 0, 0, 0, 0] },
        Shape { m_faces: [BottomTip, TopTip, FaceLeft, FaceRight, Cross, FaceFront], m_turns: [0, 3, 0, 0, 0, 0] },
        Shape { m_faces: [Cross, TopTip, Cross, TipRight, LeftDown, RightDown], m_turns: [1, 2, 0, 2, 0, 0] },
        Shape { m_faces: [Cross, TopTip, TipLeft, Cross, RightDown, LeftDown], m_turns: [2, 1, 2, 0, 0, 0] },
        Shape { m_faces: [Cross, TopTip, RightDown, LeftDown, Cross, TipLeft], m_turns: [0, 3, 0, 0, 0, 2] },
        Shape { m_faces: [Cross, TopTip, LeftDown, RightDown, TipRight, Cross], m_turns: [3, 0, 0, 0, 2, 0] },
        Shape { m_faces: [BottomTip, Cross, Cross, TipLeft, LeftUp, RightUp], m_turns: [1, 2, 0, 2, 0, 0] },
        Shape { m_faces: [BottomTip, Cross, TipRight, Cross, RightUp, LeftUp], m_turns: [2, 1, 2, 0, 0, 0] },
        Shape { m_faces: [BottomTip, Cross, RightUp, LeftUp, Cross, TipRight], m_turns: [0, 3, 0, 0, 0, 2] },
        Shape { m_faces: [BottomTip, Cross, LeftUp, RightUp, TipLeft, Cross], m_turns: [3, 0, 0, 0, 2, 0] },
        Shape { m_faces: [BottomLong, TopLong, LongSide, LongSide, Cross, Cross], m_turns: [0; 6] },
        Shape { m_faces: [BottomLong, TopLong, Cross, Cross, LongSide, LongSide], m_turns: [1, 1, 0, 0, 0, 0] },
        Shape { m_faces: [Cross, Cross, LongSide, LongSide, LongSide, LongSide], m_turns: [0, 0, 2, 2, 2, 2] },
        Shape { m_faces: [Cross, TurnTop, LongSide, LongSide, LongSide, LongSide], m_turns: [0; 6] },
    ]
};

// BlockTFNagastone.isNagaStoneInDirection, north south west east down up
const DIRECTIONS: [[i32; 3]; 6] = [[0, 0, -1], [0, 0, 1], [-1, 0, 0], [1, 0, 0], [0, -1, 0], [0, 1, 0]];

// a block's meta counts from its item's source meta, so the item shows and places the source's item shape
pub(crate) fn sourceHead(meta: i32) -> i32 { (meta + naga::HEAD_ITEM) % naga::HEADS }

pub(crate) fn sourceBody(meta: i32) -> i32
{
    (meta + naga::BODY_ITEM - naga::FIRST_BODY) % naga::BODIES + naga::FIRST_BODY
}

// the source's shapes by its meta as named states, heads by the way the face looks, bodies by their run or bend
const HEAD_FACINGS: [&str; 4] = ["north", "south", "west", "east"];
const BODY_SHAPES: [&str; 12] = [
    "down_north",
    "down_south",
    "down_west",
    "down_east",
    "up_north",
    "up_south",
    "up_west",
    "up_east",
    "x",
    "z",
    "y",
    "flat",
];

// the block and named state a source meta stands for, as the 1.7 structures name nagastone
pub(crate) fn sourceState(source: i32) -> (&'static str, &'static str, &'static str)
{
    match source
    {
        0..naga::FIRST_BODY => (block::NAGASTONE_HEAD, "facing", HEAD_FACINGS[source as usize]),
        _ => (block::NAGASTONE, "shape", BODY_SHAPES[(source - naga::FIRST_BODY) as usize % BODY_SHAPES.len()]),
    }
}

fn setSource(ctx: &BlockCtx, source: i32)
{
    let (_, property, value): (&str, &str, &str) = sourceState(source);
    if let Err(error) = ctx.world().setProperty(ctx.pos(), property, value)
    {
        warn(&format!("nagastone at {:?}: {}", ctx.pos(), error.0));
    }
}

fn headSource(ctx: &BlockCtx) -> usize { names::pick(ctx.stateAt(), "facing", &HEAD_FACINGS).unwrap_or(0) }

fn bodySource(ctx: &BlockCtx) -> usize
{
    names::pick(ctx.stateAt(), "shape", &BODY_SHAPES).unwrap_or(0) + naga::FIRST_BODY as usize
}

fn nagastone(world: World, pos: BlockPos) -> bool
{
    let found: i32 = world.block(pos);
    blockIs(found, block::NAGASTONE) || blockIs(found, block::NAGASTONE_HEAD)
}

// BlockTFNagastone.getPosibleConnection, the first direction past the excluded ones with a stone in it
fn connection(world: World, pos: BlockPos, skip: [Option<usize>; 2]) -> Option<usize>
{
    (0..DIRECTIONS.len()).filter(|direction| !skip.contains(&Some(*direction))).find(|direction|
    {
        let [dx, dy, dz] = DIRECTIONS[*direction];
        nagastone(world, pos.offset(dx, dy, dz))
    })
}

// BlockTFNagastone.placePerfectFitHead, a head with one level neighbor faces away from it
fn fitHead(world: World, pos: BlockPos) -> Option<i32>
{
    let first: usize = connection(world, pos, [None, None])?;
    let second: Option<usize> = connection(world, pos, [Some(first), None]);
    let level: bool = (1..4).contains(&first) && !second.is_some_and(|second| second <= 3);
    level.then(|| [1, 0, 3, 2][first])
}

// BlockTFNagastone.placePerfectFitBody, a body with exactly two neighbors runs or bends between them
fn fitBody(world: World, pos: BlockPos) -> Option<i32>
{
    let first: usize = connection(world, pos, [None, None])?;
    let second: usize = connection(world, pos, [Some(first), None])?;
    if connection(world, pos, [Some(first), Some(second)]).is_some()
    {
        return None;
    }

    let source: i32 = match (first, second)
    {
        (2, 3) | (3, 2) => naga::STRAIGHT,
        _ if first + second == 1 => naga::STRAIGHT + 1,
        _ if first + second == 9 => naga::STRAIGHT + 2,
        (4, other) | (other, 4) => naga::DOWN_BEND | other as i32,
        (5, other) | (other, 5) => naga::UP_BEND | other as i32,
        _ => naga::FLAT_BEND,
    };
    Some(source)
}

struct Head;

#[block_behavior]
impl BlockBehavior for Head
{
    // BlockTFNagastone.onBlockPlaced, a head looks out of the face it was set against
    fn onPlaced(&self, ctx: &mut BlockCtx, face: i32)
    {
        let source: i32 = match Face::fromIndex(face)
        {
            Some(Face::North) => 0,
            Some(Face::West) => 2,
            Some(Face::East) => 3,
            _ => 1,
        };
        setSource(ctx, source);
    }

    fn onNeighborChanged(&self, ctx: &mut BlockCtx, _neighbor: i32)
    {
        if !ctx.isAuthority()
        {
            return;
        }

        if let Some(source) = fitHead(ctx.world(), ctx.pos())
        {
            setSource(ctx, source);
        }
    }

    fn textureFor(&self, face: i32, state: &StateData) -> Option<String>
    {
        source(state, "facing", &HEAD_FACINGS).map(|source| faceTexture(source, face))
    }

    fn faceRotations(&self, ctx: &mut BlockCtx) -> Option<[u8; 6]> { Some(SHAPES[headSource(ctx)].m_turns) }
}

struct Body;

#[block_behavior]
impl BlockBehavior for Body
{
    // BlockTFNagastone.onBlockPlaced, a straight run lies along the axis of the face it was set against
    fn onPlaced(&self, ctx: &mut BlockCtx, face: i32)
    {
        let axis: i32 = match Face::fromIndex(face)
        {
            Some(Face::Down | Face::Up) => 2,
            Some(Face::North | Face::South) => 1,
            _ => 0,
        };
        setSource(ctx, naga::STRAIGHT + axis);
    }

    fn onNeighborChanged(&self, ctx: &mut BlockCtx, _neighbor: i32)
    {
        if !ctx.isAuthority()
        {
            return;
        }

        if let Some(source) = fitBody(ctx.world(), ctx.pos())
        {
            setSource(ctx, source);
        }
    }

    fn textureFor(&self, face: i32, state: &StateData) -> Option<String>
    {
        source(state, "shape", &BODY_SHAPES).map(|source| faceTexture(source + naga::FIRST_BODY as usize, face))
    }

    fn faceRotations(&self, ctx: &mut BlockCtx) -> Option<[u8; 6]> { Some(SHAPES[bodySource(ctx)].m_turns) }
}

// the source meta a named state stands for, its value's place in the source's own order
fn source(state: &StateData, property: &str, values: &[&str]) -> Option<usize>
{
    values.iter().position(|value| state.get(property) == Some(*value))
}

fn faceTexture(source: usize, face: i32) -> String
{
    let shape: &Shape = &SHAPES[source.min(SHAPES.len() - 1)];
    texture(SPRITES[shape.m_faces[face.clamp(0, 5) as usize] as usize])
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    // the hooks pick every sprite, asking for their slots now gives them room in the atlas
    for name in SPRITES
    {
        let _ = b173_api::world::terrainSlot(&texture(name));
    }

    super::put(
        ctx,
        block::NAGASTONE_HEAD,
        base("minecraft:block", "Nagastone Head", Kind::Rock, naga::HARDNESS)
            .resistance(naga::RESISTANCE)
            .texture(&texture("nagastone_face_front"))
            .behavior(Head),
    )?;
    super::put(
        ctx,
        block::NAGASTONE,
        base("minecraft:block", "Nagastone", Kind::Rock, naga::HARDNESS)
            .resistance(naga::RESISTANCE)
            .texture(&texture("nagastone_long_side"))
            .behavior(Body),
    )?;

    // values in stored metadata order, so saved nagastone keeps its shape
    let heads: Vec<&str> = (0..naga::HEADS).map(|meta| HEAD_FACINGS[sourceHead(meta) as usize]).collect();
    let bodies: Vec<&str> =
        (0..naga::BODIES).map(|meta| BODY_SHAPES[(sourceBody(meta) - naga::FIRST_BODY) as usize]).collect();
    ctx.blocks().states(
        block::NAGASTONE_HEAD,
        BlockStatesSpec { properties: vec![PropertySpec::new("facing", &heads)], ..BlockStatesSpec::default() },
    )?;
    ctx.blocks().states(
        block::NAGASTONE,
        BlockStatesSpec { properties: vec![PropertySpec::new("shape", &bodies)], ..BlockStatesSpec::default() },
    )
}
