use super::{DEGREES_PER_RADIAN, HALF_PI, Program, WALK_RATE, cuboid, part, spider, tf_ghast, wolf};
use b173_api::terrain::math;
use b173_api::{EntityModelSpec, Pose, PoseInput};
use std::f32::consts::{FRAC_PI_4, FRAC_PI_8, PI};

// setRotationAngles of the vanilla spider and wolf models and of ModelTFGhast, for the mobs drawn with them

fn head(input: &PoseInput) -> [f32; 3]
{
    [input.head_pitch / DEGREES_PER_RADIAN, input.head_yaw / DEGREES_PER_RADIAN, 0.0]
}

// ModelSpider, legs splayed by pairs and scuttling in four phases
pub fn spiderPose(input: &PoseInput) -> Pose
{
    const LEG_SPLAY: f32 = 0.74;
    const SCUTTLE: f32 = 0.4;
    let legs: [u16; 8] = [
        spider::SPIDER_LEG1,
        spider::SPIDER_LEG2,
        spider::SPIDER_LEG3,
        spider::SPIDER_LEG4,
        spider::SPIDER_LEG5,
        spider::SPIDER_LEG6,
        spider::SPIDER_LEG7,
        spider::SPIDER_LEG8,
    ];
    let roll: [f32; 8] = [
        -FRAC_PI_4,
        FRAC_PI_4,
        -FRAC_PI_4 * LEG_SPLAY,
        FRAC_PI_4 * LEG_SPLAY,
        -FRAC_PI_4 * LEG_SPLAY,
        FRAC_PI_4 * LEG_SPLAY,
        -FRAC_PI_4,
        FRAC_PI_4,
    ];
    let yaw: [f32; 8] = [
        FRAC_PI_8 * 2.0,
        -FRAC_PI_8 * 2.0,
        FRAC_PI_8,
        -FRAC_PI_8,
        -FRAC_PI_8,
        FRAC_PI_8,
        -FRAC_PI_8 * 2.0,
        FRAC_PI_8 * 2.0,
    ];
    let phases: [f32; 4] = [0.0, PI, HALF_PI, PI * 3.0 / 2.0];
    let walk: f32 = input.walk * WALK_RATE;
    let mut body: Program = Program::new();
    body.part(spider::SPIDER_HEAD, head(input), spider::PIVOTS[spider::SPIDER_HEAD as usize]);
    body.rest(spider::SPIDER_NECK).rest(spider::SPIDER_BODY);
    for (index, leg) in legs.iter().enumerate()
    {
        let phase: f32 = phases[index / 2];
        let side: f32 = if index % 2 == 0 { 1.0 } else { -1.0 };
        let swing: f32 = -(math::cos(walk * 2.0 + phase) * SCUTTLE) * input.walk_speed;
        let lift: f32 = (math::sin(walk + phase) * SCUTTLE).abs() * input.walk_speed;
        let rotation: [f32; 3] = [0.0, yaw[index] + swing * side, roll[index] + lift * side];
        body.part(*leg, rotation, spider::PIVOTS[*leg as usize]);
    }
    Pose { pre: Vec::new(), model: body.take() }
}

// ModelWolf standing, an angry wolf holds its tail up
pub fn wolfPose(input: &PoseInput) -> Pose
{
    const ANGRY_TAIL: f32 = 1.5393804;
    const LEG_SWING: f32 = 1.4;
    const MANE: [f32; 3] = [-1.0, 14.0, -3.0];
    let walk: f32 = input.walk * WALK_RATE;
    let swing = |phase: f32| [math::cos(walk + phase) * LEG_SWING * input.walk_speed, 0.0, 0.0];
    let head: [f32; 3] = head(input);
    let mut body: Program = Program::new();
    for part in [wolf::WOLF_HEAD_MAIN, wolf::WOLF_RIGHT_EAR, wolf::WOLF_LEFT_EAR, wolf::WOLF_SNOUT]
    {
        body.part(part, head, wolf::PIVOTS[part as usize]);
    }
    body.part(wolf::WOLF_BODY, [HALF_PI, 0.0, 0.0], wolf::PIVOTS[wolf::WOLF_BODY as usize]);
    body.part(wolf::WOLF_MANE, [HALF_PI, 0.0, 0.0], MANE);
    for (leg, phase) in [(wolf::WOLF_LEG1, 0.0), (wolf::WOLF_LEG2, PI), (wolf::WOLF_LEG3, PI), (wolf::WOLF_LEG4, 0.0)]
    {
        body.part(leg, swing(phase), wolf::PIVOTS[leg as usize]);
    }
    body.part(wolf::WOLF_TAIL, [ANGRY_TAIL, 0.0, 0.0], wolf::PIVOTS[wolf::WOLF_TAIL as usize]);
    Pose { pre: Vec::new(), model: body.take() }
}

// ModelTFGhast.makeTentacle builds nine tentacles in a loop the converter does not follow, offsets and lengths as
// the Random(1660) it seeds draws them
const TENTACLES: [([f32; 2], i32); 9] = [
    ([-3.75, -5.0], 8),
    ([1.25, -5.0], 13),
    ([6.25, -5.0], 9),
    ([-6.25, 0.0], 11),
    ([-1.25, 0.0], 11),
    ([3.75, 0.0], 10),
    ([-3.75, 5.0], 12),
    ([1.25, 5.0], 9),
    ([6.25, 5.0], 12),
];
// 23 plus the model's yOffset of -16, below the body's pivot
const TENTACLE_DROP: f32 = 7.0;

pub fn ghastModel() -> EntityModelSpec
{
    let mut model: EntityModelSpec = tf_ghast::model();
    for (number, ([x, z], length)) in TENTACLES.iter().enumerate()
    {
        let boxes = vec![cuboid([-1.0, 0.0, -1.0], [2, *length, 2], 0.0, None)];
        model.parts.push(part(
            &format!("tentacle{number}"),
            Some("body"),
            [0, 0],
            false,
            [*x, TENTACLE_DROP, *z],
            [0.0; 3],
            boxes,
            None,
        ));
    }
    model
}

// ModelTFGhast.setRotationAngles, the tentacles wave out of step
pub fn ghastPose(input: &PoseInput) -> Pose
{
    const WAVE: f32 = 0.2;
    const WAVE_RATE: f32 = 0.3;
    const HANG: f32 = 0.4;
    let mut body: Program = Program::new();
    let first: u16 = tf_ghast::PIVOTS.len() as u16;
    for (number, ([x, z], _)) in TENTACLES.iter().enumerate()
    {
        let sway: f32 = WAVE * math::sin(input.age * WAVE_RATE + number as f32) + HANG;
        body.part(first + number as u16, [sway, 0.0, 0.0], [*x, TENTACLE_DROP, *z]);
    }
    body.part(tf_ghast::BODY, head(input), tf_ghast::PIVOTS[tf_ghast::BODY as usize]);
    Pose { pre: Vec::new(), model: body.take() }
}
