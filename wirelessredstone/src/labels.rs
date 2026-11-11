use b173_api::prelude::*;

// TileEntityRedstoneWirelessRenderer, the frequency in black on all four sides just under the top edge
mod label
{
    // font pixels to blocks, and the gap that keeps the text off the face
    pub const SCALE: f32 = 0.01;
    pub const HEIGHT: f32 = 1.5;
    pub const LINE_Y: i32 = 50;
    pub const COLOR: u32 = 0;
    pub const CENTER: f32 = 0.5;
    pub const QUARTER: f32 = 90.0;
}

// where each side's text stands, its turn around the vertical axis and its normal's z
struct Side
{
    m_at: [f32; 3],
    m_turn: f32,
    m_normal: f32,
}

const SIDES: [Side; 4] = [
    Side { m_at: [label::CENTER, label::HEIGHT, 1.0 + label::SCALE], m_turn: 0.0, m_normal: -1.0 },
    Side { m_at: [1.0 + label::SCALE, label::HEIGHT, label::CENTER], m_turn: label::QUARTER, m_normal: -1.0 },
    Side { m_at: [label::CENTER, label::HEIGHT, -label::SCALE], m_turn: label::QUARTER * 2.0, m_normal: 1.0 },
    Side { m_at: [-label::SCALE, label::HEIGHT, label::CENTER], m_turn: label::QUARTER * 3.0, m_normal: -1.0 },
];

pub(crate) fn pose(freq: i32) -> Pose
{
    let text: String = freq.to_string();
    let x: i32 = -textWidth(&text) / 2;
    let mut ops: Vec<ModelOp> = Vec::new();

    for side in &SIDES
    {
        ops.push(ModelOp::Push);
        ops.push(ModelOp::Translate(side.m_at));
        ops.push(ModelOp::Rotate(side.m_turn, [0.0, 1.0, 0.0]));
        ops.push(ModelOp::Scale([label::SCALE, -label::SCALE, label::SCALE]));
        ops.push(ModelOp::Normal([0.0, 0.0, side.m_normal * label::SCALE]));
        ops.push(ModelOp::DepthMask(false));
        ops.push(ModelOp::Text { text: text.clone(), x, y: label::LINE_Y, color: label::COLOR });
        ops.push(ModelOp::DepthMask(true));
        ops.push(ModelOp::Pop);
    }

    Pose { pre: ops, model: Vec::new() }
}
