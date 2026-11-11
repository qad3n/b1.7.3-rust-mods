use super::liquids::{CENTER_CAPACITY, Liquid, SIDE_CAPACITY};
use super::pipe::{self, PipeState};
use crate::core::render::{self, Scene};
use crate::core::stacks;
use b173_api::prelude::*;

// RenderPipe: liquid as boxes growing along each arm and through the core, items riding inside
const OIL_SPRITE: &str = "buildcraft:block/oil_still";
// glRotatef angles that turn the west arm's box onto each face, by face index
const ANGLE_Y: [f32; 6] = [0.0, 0.0, 270.0, 90.0, 0.0, 180.0];
const ANGLE_Z: [f32; 6] = [90.0, 270.0, 0.0, 0.0, 0.0, 0.0];
// the display list stages compileDisplayLists builds
const SIDE_STAGES: f32 = 39.0;
const CENTER_STAGES: f32 = 79.0;
const SIDE_LENGTH: f32 = 0.25;
const CENTER_LENGTH: f32 = 0.5;
const INNER: f32 = 0.26;
const OUTER: f32 = 0.74;
const CENTER_END: f32 = 0.75;
const CENTER_GAP: f32 = 0.01;
const BLOCK_LIFT: f32 = 0.25;
const ITEM_LIFT: f32 = 0.1;

// one turn about the block's middle, as the glRotatef calls around the translated center
fn turn(point: [f32; 3], angleY: f32, angleZ: f32) -> [f32; 3]
{
    let [x, y, z] = point.map(|value| value - 0.5);
    let (sinZ, cosZ): (f32, f32) = angleZ.to_radians().sin_cos();
    let (x, y): (f32, f32) = (x * cosZ - y * sinZ, x * sinZ + y * cosZ);
    let (sinY, cosY): (f32, f32) = angleY.to_radians().sin_cos();
    let (x, z): (f32, f32) = (x * cosY + z * sinY, -x * sinY + z * cosY);
    [x + 0.5, y + 0.5, z + 0.5]
}

fn turned(draw: &mut Scene, min: [f32; 3], max: [f32; 3], face: i32, sprite: [f32; 4], light: f32)
{
    let index: usize = face as usize;
    let first: [f32; 3] = turn(min, ANGLE_Y[index], ANGLE_Z[index]);
    let second: [f32; 3] = turn(max, ANGLE_Y[index], ANGLE_Z[index]);
    let low: [f32; 3] = std::array::from_fn(|axis| first[axis].min(second[axis]));
    let high: [f32; 3] = std::array::from_fn(|axis| first[axis].max(second[axis]));
    draw.cube(low, high, sprite, light);
}

// renderSide, a full arm or the part flowing in from the face and the part flowing out to it
fn side(draw: &mut Scene, face: i32, toCenter: i32, toSide: i32, sprite: [f32; 4], light: f32)
{
    let toCenter: i32 = toCenter.min(SIDE_CAPACITY);
    let toSide: i32 = toSide.min(SIDE_CAPACITY);
    if toCenter == 0 && toSide == 0
    {
        return;
    }

    let edge = |stage: f32| stage / SIDE_STAGES * SIDE_LENGTH;
    let filled = |amount: i32| (amount as f32 / SIDE_CAPACITY as f32 * SIDE_STAGES).floor();

    if toCenter + toSide >= SIDE_CAPACITY
    {
        turned(draw, [0.0, INNER, INNER], [SIDE_LENGTH, OUTER, OUTER], face, sprite, light);
        return;
    }

    if toCenter > 0
    {
        turned(draw, [0.0, INNER, INNER], [edge(filled(toCenter)), OUTER, OUTER], face, sprite, light);
    }

    if toSide > 0
    {
        let stage: f32 = SIDE_STAGES - filled(toSide);
        turned(
            draw,
            [stage / (SIDE_STAGES + 1.0) * SIDE_LENGTH, INNER, INNER],
            [SIDE_LENGTH, OUTER, OUTER],
            face,
            sprite,
            light,
        );
    }
}

// renderCenter, turned toward where the liquid last came from or last went
fn center(draw: &mut Scene, liquid: &Liquid, sprite: [f32; 4], light: f32)
{
    let incoming: i32 = liquid.m_centerIn.min(CENTER_CAPACITY);
    let outgoing: i32 = liquid.m_centerOut.min(CENTER_CAPACITY);
    if incoming == 0 && outgoing == 0
    {
        return;
    }

    let filled = |amount: i32| (amount as f32 / CENTER_CAPACITY as f32 * CENTER_STAGES).floor();
    let reach = |stage: f32| CENTER_END - (CENTER_LENGTH - stage / CENTER_STAGES * CENTER_LENGTH) - CENTER_GAP;

    if incoming + outgoing >= CENTER_CAPACITY
    {
        turned(draw, [INNER, INNER, INNER], [reach(CENTER_STAGES), OUTER, OUTER], liquid.m_lastFrom, sprite, light);
        return;
    }

    let face: i32 = if incoming > 0
    {
        liquid.m_lastFrom
    }
    else
    {
        crate::core::dir::reverse(liquid.m_lastTo)
    };

    if incoming > 0
    {
        turned(draw, [INNER, INNER, INNER], [reach(filled(incoming)), OUTER, OUTER], face, sprite, light);
    }

    if outgoing > 0
    {
        let stage: f32 = CENTER_STAGES - filled(outgoing);
        let start: f32 = INNER + (stage - 1.0) / CENTER_STAGES * CENTER_LENGTH;
        turned(draw, [start, INNER, INNER], [OUTER, OUTER, OUTER], face, sprite, light);
    }
}

pub(crate) fn pose(ctx: &mut BlockCtx) -> Pose
{
    let world: World = ctx.world();
    let pos: BlockPos = ctx.pos();
    let state: PipeState = pipe::load(world, pos);
    let liquid: &Liquid = &state.m_liquid;
    let wet: bool = liquid.m_centerIn + liquid.m_centerOut > 0
        || liquid.m_sideToCenter.iter().chain(&liquid.m_centerToSide).any(|amount| *amount > 0);
    if state.m_items.is_empty() && !wet
    {
        return Pose::default();
    }

    let light: f32 = render::lightAt(world, pos);
    let sprite: [f32; 4] = render::uv(OIL_SPRITE);
    let mut draw: Scene = Scene::new();

    for face in crate::core::dir::all()
    {
        let index: usize = face as usize;
        side(&mut draw, face, liquid.m_sideToCenter[index], liquid.m_centerToSide[index], sprite, light);
    }

    center(&mut draw, liquid, sprite, light);

    for item in &state.m_items
    {
        let stack: ItemStack = item.m_stack.stack();
        let lift: f32 = if stacks::isBlock(&stack)
        {
            BLOCK_LIFT
        }
        else
        {
            ITEM_LIFT
        };
        let at: [f64; 3] = render::relative(pos, item.m_pos);
        draw.item([at[0] as f32, at[1] as f32 + lift, at[2] as f32], &stack, light);
    }

    draw.finish()
}
