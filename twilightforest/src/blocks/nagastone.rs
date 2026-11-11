use super::{Kind, base, texture};
use crate::names::block;
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

pub(crate) fn headMeta(source: i32) -> i32 { (source - naga::HEAD_ITEM).rem_euclid(naga::HEADS) }

pub(crate) fn sourceBody(meta: i32) -> i32
{
    (meta + naga::BODY_ITEM - naga::FIRST_BODY) % naga::BODIES + naga::FIRST_BODY
}

pub(crate) fn bodyMeta(source: i32) -> i32 { (source - naga::BODY_ITEM).rem_euclid(naga::BODIES) }

fn sprite(sprite: Sprite) -> i32 { b173_api::world::terrainSlot(&texture(SPRITES[sprite as usize])).unwrap_or(0) }

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
        ctx.setMeta(headMeta(source) as u8);
    }

    fn onNeighborChanged(&self, ctx: &mut BlockCtx, _neighbor: i32)
    {
        if !ctx.isAuthority()
        {
            return;
        }

        if let Some(source) = fitHead(ctx.world(), ctx.pos())
            && let Err(error) = ctx.world().setMeta(ctx.pos(), headMeta(source))
        {
            warn(&format!("nagastone head refit: {}", error.0));
        }
    }

    fn texture(&self, face: i32, meta: i32) -> i32 { faceSprite(sourceHead(meta), face) }

    fn faceRotations(&self, ctx: &mut BlockCtx) -> Option<[u8; 6]>
    {
        Some(SHAPES[sourceHead(i32::from(ctx.meta())) as usize].m_turns)
    }
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
        ctx.setMeta(bodyMeta(naga::STRAIGHT + axis) as u8);
    }

    fn onNeighborChanged(&self, ctx: &mut BlockCtx, _neighbor: i32)
    {
        if !ctx.isAuthority()
        {
            return;
        }

        if let Some(source) = fitBody(ctx.world(), ctx.pos())
            && let Err(error) = ctx.world().setMeta(ctx.pos(), bodyMeta(source))
        {
            warn(&format!("nagastone body refit: {}", error.0));
        }
    }

    fn texture(&self, face: i32, meta: i32) -> i32 { faceSprite(sourceBody(meta), face) }

    fn faceRotations(&self, ctx: &mut BlockCtx) -> Option<[u8; 6]>
    {
        Some(SHAPES[sourceBody(i32::from(ctx.meta())) as usize].m_turns)
    }
}

fn faceSprite(source: i32, face: i32) -> i32
{
    let shape: &Shape = &SHAPES[source.clamp(0, SHAPES.len() as i32 - 1) as usize];
    sprite(shape.m_faces[face.clamp(0, 5) as usize])
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
    )
}
