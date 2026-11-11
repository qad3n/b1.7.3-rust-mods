use crate::models::{DEGREES_PER_RADIAN, HALF_PI, Program, WALK_RATE, moa as layout, quadruped};
use b173_api::PoseInput;
use b173_api::terrain::math;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;

// MoaModel and CockatriceModel share their eleven parts and their animation
mod wing
{
    // EntityMoa.updateWingRotation: 1.233 radians a tick, opening over five ticks off the ground
    pub const FLAP: f32 = 1.233;
    pub const OPEN: f32 = 0.2;
    pub const MIN_OPEN: f32 = 0.01;
    pub const FLYING: f32 = 0.001;
    pub const LEGS_TUCKED: f32 = 0.6;
    pub const SITTING_JAW: f32 = -0.3;
}

#[derive(Clone, Copy)]
struct Wing
{
    m_rotation: f32,
    m_open: f32,
    m_lastTick: i32,
}

thread_local!
{
    static WINGS: RefCell<HashMap<i32, Wing>> = RefCell::new(HashMap::new());
}

// EntityMoa.updateWingRotation and MoaRenderer.getWingRotation, stepped on the client per rendered tick
pub(crate) fn wingSwing(input: &PoseInput) -> f32
{
    WINGS.with(|wings|
    {
        let mut wings: RefMut<HashMap<i32, Wing>> = wings.borrow_mut();
        let tick: i32 = input.age as i32;
        let state: &mut Wing =
            wings.entry(input.entity.id).or_insert(Wing { m_rotation: 0.0, m_open: 0.0, m_lastTick: tick });

        while state.m_lastTick < tick
        {
            state.m_rotation += wing::FLAP;

            state.m_open = if input.on_ground
            {
                0.0
            }
            else
            {
                (state.m_open + wing::OPEN).clamp(wing::MIN_OPEN, 1.0)
            };
            state.m_lastTick += 1;
        }

        (math::sin(state.m_rotation) + 1.0) * state.m_open
    })
}

// MoaModel.setRotationAngles and render for the eleven parts starting at base
pub(crate) fn pose(
    program: &mut Program,
    base: u16,
    pivots: &[[f32; 3]],
    rotations: &[[f32; 3]],
    input: &PoseInput,
    sitting: bool,
    swing: f32,
)
{
    let mut pivots: Vec<[f32; 3]> = pivots.to_vec();

    if sitting
    {
        let lowered: [(u16, [f32; 3]); 9] = [
            (layout::HEAD, [0.0, 16.0, -4.0]),
            (layout::JAW, [0.0, 16.0, -4.0]),
            (layout::BODY, [0.0, 24.0, 0.0]),
            (layout::LEGS, [-2.0, 24.0, 1.0]),
            (layout::LEGS2, [2.0, 24.0, 1.0]),
            (layout::NECK, [0.0, 22.0, -4.0]),
            (layout::FEATHER1, [0.0, 25.5, 1.0]),
            (layout::FEATHER2, [0.0, 25.5, 1.0]),
            (layout::FEATHER3, [0.0, 25.5, 1.0]),
        ];

        for (part, pivot) in lowered
        {
            pivots[part as usize] = pivot;
        }
    }

    let head: [f32; 3] = if sitting
    {
        [0.0; 3]
    }
    else
    {
        [input.head_pitch / DEGREES_PER_RADIAN, input.head_yaw / DEGREES_PER_RADIAN, 0.0]
    };

    let jaw: [f32; 3] = if sitting
    {
        [wing::SITTING_JAW, 0.0, 0.0]
    }
    else
    {
        head
    };
    let flying: bool = swing > wing::FLYING;
    let walk: f32 = math::cos(input.walk * WALK_RATE) * quadruped::LEG_SWING * input.walk_speed;
    let walkBack: f32 =
        math::cos(input.walk * WALK_RATE + std::f32::consts::PI) * quadruped::LEG_SWING * input.walk_speed;

    let (legs, legs2): (f32, f32) = if flying
    {
        (wing::LEGS_TUCKED, wing::LEGS_TUCKED)
    }
    else
    {
        (walk, walkBack)
    };

    let wingY: f32 = if sitting
    {
        22.0
    }
    else if flying
    {
        12.0
    }
    else
    {
        14.0
    };
    let wingZ: f32 = if flying { -1.0 } else { -3.0 };
    let wingX: f32 = if flying { 0.0 } else { HALF_PI };
    let wingRoll: f32 = if flying { swing } else { 0.0 };
    let at = |part: u16| pivots[part as usize];

    if !sitting || !input.on_ground
    {
        program.part(base + layout::LEGS, [legs, 0.0, 0.0], at(layout::LEGS));
        program.part(base + layout::LEGS2, [legs2, 0.0, 0.0], at(layout::LEGS2));
    }

    program.part(base + layout::HEAD, head, at(layout::HEAD));
    program.part(base + layout::JAW, jaw, at(layout::JAW));
    program.part(base + layout::BODY, [HALF_PI, 0.0, 0.0], at(layout::BODY));
    program.part(base + layout::WINGS, [wingX, 0.0, wingRoll], [at(layout::WINGS)[0], wingY, wingZ]);
    program.part(base + layout::WINGS2, [wingX, 0.0, -wingRoll], [at(layout::WINGS2)[0], wingY, wingZ]);

    for part in [layout::NECK, layout::FEATHER1, layout::FEATHER2, layout::FEATHER3]
    {
        program.part(base + part, rotations[part as usize], at(part));
    }
}
