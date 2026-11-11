use super::engine::{self, ENGINES, EngineRow, Stage, View};
use crate::core::dir;
use b173_api::prelude::*;
use std::f32::consts::{FRAC_PI_2, PI};

// RenderEngine: a base plate, a piston plate riding out, bellows chambers between them and a trunk colored by heat
const TEXTURE_SIZE: [i32; 2] = [64, 32];
const PIVOT: [f32; 3] = [8.0, 8.0, 8.0];
const CHAMBER: &str = "buildcraft:model/engine_chamber";
// the stroke of a full piston push in pixels, and the gap between chamber plates in blocks
const STROKE: f32 = 7.99;
const PIXEL: f32 = 1.0 / 16.0;
const CHAMBER_STEP: f32 = 0.125;
const CHAMBER_SPACING: usize = 2;
const HALF: f32 = 0.5;

mod part
{
    pub const BOX: u16 = 0;
    pub const TRUNK: u16 = 1;
    pub const MOVING: u16 = 2;
    pub const CHAMBER: u16 = 3;
}

fn modelPart(name: &str, origin: [f32; 3], size: [i32; 3]) -> ModelPart
{
    ModelPart
    {
        name: name.to_owned(),
        parent: None,
        uv: [0, 0],
        mirror: false,
        pivot: PIVOT,
        rotation: [0.0; 3],
        boxes: vec![ModelBox { origin, size, inflate: 0.0, uv: None }],
        texture_size: None,
    }
}

pub(crate) fn spec() -> EntityModelSpec
{
    EntityModelSpec
    {
        texture_size: TEXTURE_SIZE,
        parts: vec![
            modelPart("box", [-8.0, -8.0, -8.0], [16, 4, 16]),
            modelPart("trunk", [-4.0, -4.0, -4.0], [8, 12, 8]),
            modelPart("moving_box", [-8.0, -4.0, -8.0], [16, 4, 16]),
            modelPart("chamber", [-5.0, -4.0, -5.0], [10, 2, 10]),
        ],
        shadow: 0.0,
    }
}

// the part rotation and the push direction for each output face
fn facing(face: i32) -> ([f32; 3], [f32; 3])
{
    match face
    {
        dir::EAST => ([0.0, 0.0, -FRAC_PI_2], [1.0, 0.0, 0.0]),
        dir::WEST => ([0.0, 0.0, FRAC_PI_2], [-1.0, 0.0, 0.0]),
        dir::DOWN => ([0.0, 0.0, PI], [0.0, -1.0, 0.0]),
        dir::SOUTH => ([FRAC_PI_2, 0.0, 0.0], [0.0, 0.0, 1.0]),
        dir::NORTH => ([-FRAC_PI_2, 0.0, 0.0], [0.0, 0.0, -1.0]),
        _ => ([0.0; 3], [0.0, 1.0, 0.0]),
    }
}

fn scaled(direction: [f32; 3], amount: f32) -> [f32; 3] { direction.map(|value| value * amount) }

// the trunk's heat color
fn trunk(stage: Stage) -> &'static str
{
    match stage
    {
        Stage::Blue => "buildcraft:model/engine_trunk_blue",
        Stage::Green => "buildcraft:model/engine_trunk_green",
        Stage::Yellow => "buildcraft:model/engine_trunk_yellow",
        Stage::Red | Stage::Explosion => "buildcraft:model/engine_trunk_red",
    }
}

// RenderEngine.render for one engine's stage, stroke and output face
fn ops(row: &EngineRow, view: &View) -> Vec<ModelOp>
{
    let progress: f32 = view.m_progress;
    let step: f32 = if progress > HALF
    {
        STROKE - (progress - HALF) * 2.0 * STROKE
    }
    else
    {
        progress * 2.0 * STROKE
    };
    let (rotation, direction) = facing(view.m_orientation);
    let lift: [f32; 3] = scaled(direction, step * PIXEL);
    let chambers: usize = (step + CHAMBER_SPACING as f32).max(0.0) as usize / CHAMBER_SPACING + 1;
    let trunk: &str = trunk(view.m_stage);
    let mut ops: Vec<ModelOp> = vec![
        ModelOp::Lighting(false),
        ModelOp::Texture(row.m_base.to_owned()),
        ModelOp::Part { part: part::BOX, rotation, pivot: PIVOT },
        ModelOp::Translate(lift),
        ModelOp::Part { part: part::MOVING, rotation, pivot: PIVOT },
        ModelOp::Translate(scaled(lift, -1.0)),
        ModelOp::Texture(CHAMBER.to_owned()),
    ];

    for _ in 0..chambers
    {
        ops.push(ModelOp::Part { part: part::CHAMBER, rotation, pivot: PIVOT });
        ops.push(ModelOp::Translate(scaled(direction, CHAMBER_STEP)));
    }

    ops.push(ModelOp::Translate(scaled(direction, -CHAMBER_STEP * chambers as f32)));
    ops.push(ModelOp::Texture(trunk.to_owned()));
    ops.push(ModelOp::Part { part: part::TRUNK, rotation, pivot: PIVOT });
    ops.push(ModelOp::Lighting(true));
    ops
}

pub(crate) fn pose(ctx: &mut BlockCtx) -> Pose
{
    let world: World = ctx.world();
    let Some(row) = engine::rowOf(world, ctx.pos())
    else
    {
        return Pose::default();
    };
    let view: View = engine::shown(world, ctx.pos()).unwrap_or(View { m_orientation: dir::UP, ..View::default() });
    Pose { pre: Vec::new(), model: ops(row, &view) }
}

// RenderEngine.inventoryRender: the blue engine at rest, pointing up, for the item in hand and in slots
pub(crate) fn itemPose(name: &str) -> Pose
{
    let Some(row) = ENGINES.iter().find(|row| row.m_name == name)
    else
    {
        return Pose::default();
    };
    const RESTING: f32 = 0.25;

    let view: View = View { m_orientation: dir::UP, m_progress: RESTING, ..View::default() };
    Pose { pre: Vec::new(), model: ops(row, &view) }
}
