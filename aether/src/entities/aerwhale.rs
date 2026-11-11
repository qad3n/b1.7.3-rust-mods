use super::{creature, onAetherGrass, sounds};
use crate::models::{Program, aerwhale as model};
use b173_api::prelude::*;
use b173_api::terrain::math;
use std::cell::RefCell;
use std::collections::HashMap;

const NAME: &str = "aerwhale";
const TEXTURE: &str = "aether:entity/aerwhale/aerwhale";

mod course
{
    // AerwhaleAITravelCourse looks this far ahead along each heading
    pub const SIGHT: f64 = 50.0;
    pub const TURN: f64 = 5.0;
    pub const BOUNCE_TURN: f64 = 10.0;
    pub const DAMPING: f64 = 0.9;
    pub const CEILING: f64 = 100.0;
    pub const FLOOR: f64 = 20.0;
    pub const CLIMB: f64 = 2.0;
    pub const STEER: f64 = 0.1;
    pub const MAX_PITCH: f32 = 60.0;
    pub const PITCH_EASE: f32 = 0.99;
    pub const THRUST: f64 = 0.005;
    pub const DRAG: f64 = 0.98;
    pub const SIDE: f32 = 45.0;
    // AerwhaleAIUnstuck nudges the whale off blocks around it
    pub const NUDGE: f64 = 0.002;
    pub const DESPAWN_Y: f64 = -64.0;
    // the Java's float degrees to radians factor, kept for the same headings
    pub const RADIANS: f32 = 0.01745329;
    pub const JITTER_STEPS: i32 = 1000000;
}

#[derive(Default, Clone, Copy)]
struct Heading
{
    m_yaw: f64,
    m_pitch: f64,
}

thread_local!
{
    static HEADINGS: RefCell<HashMap<i32, Heading>> = RefCell::new(HashMap::new());
}

// AerwhaleAITravelCourse.checkForTravelableCourse: distance to the first block along a heading, or the full sight
fn openDistance(world: World, info: &EntityInfo, yawOffset: f32, pitchOffset: f32) -> f64
{
    let yaw: f32 = info.yaw + yawOffset;
    let pitch: f32 = info.pitch + pitchOffset;
    let cosYaw: f32 = math::cos(-yaw * course::RADIANS - std::f32::consts::PI);
    let sinYaw: f32 = math::sin(-yaw * course::RADIANS - std::f32::consts::PI);
    let cosPitch: f32 = math::cos(-pitch * course::RADIANS);
    let sinPitch: f32 = math::sin(-pitch * course::RADIANS);
    let from: [f64; 3] = info.pos;
    let to: [f64; 3] = [
        from[0] + (sinYaw * cosPitch) as f64 * course::SIGHT,
        from[1] + sinPitch as f64 * course::SIGHT,
        from[2] + (cosYaw * cosPitch) as f64 * course::SIGHT,
    ];

    match world.rayTrace(from, to, true)
    {
        Some(hit) =>
        {
            let [dx, dy, dz] = [hit.pos.x as f64 - from[0], hit.pos.y as f64 - from[1], hit.pos.z as f64 - from[2]];
            (dx * dx + dy * dy + dz * dz).sqrt()
        }

        None => course::SIGHT,
    }
}

fn anyBlock(world: World, from: [f64; 3], to: [f64; 3]) -> bool
{
    let floor = |value: f64| value.floor() as i32;
    (floor(from[0])..=floor(to[0])).any(|x|
    {
        (floor(from[1])..=floor(to[1]))
            .any(|y| (floor(from[2])..=floor(to[2])).any(|z| !world.isAir(BlockPos::new(x, y, z))))
    })
}

// AerwhaleAIUnstuck: a push away from every side that touches blocks
fn unstuck(world: World, info: &EntityInfo, motion: &mut [f64; 3])
{
    let [x, y, z] = info.pos;
    let nudges: [([f64; 3], [f64; 3], [f64; 3]); 6] = [
        ([x - 2.0, y + 3.0, z - 2.0], [x, y + 3.0, z], [0.0, -1.0, 0.0]),
        ([x - 2.0, y - 1.0, z - 2.0], [x, y - 1.0, z], [0.0, 1.0, 0.0]),
        ([x + 1.0, y, z - 2.0], [x + 1.0, y + 2.0, z], [-1.0, 1.0, 0.0]),
        ([x - 3.0, y, z - 2.0], [x - 3.0, y + 2.0, z], [1.0, 1.0, 0.0]),
        ([x - 2.0, y, z + 1.0], [x, y + 2.0, z + 1.0], [0.0, 1.0, -1.0]),
        ([x - 2.0, y, z - 3.0], [x, y + 2.0, z - 3.0], [0.0, 1.0, 1.0]),
    ];

    for (from, to, push) in nudges
    {
        if anyBlock(world, from, to)
        {
            for axis in 0..3
            {
                motion[axis] += push[axis] * course::NUDGE;
            }
        }
    }
}

fn jitter(world: World) -> f64 { 2.0 * (world.random(course::JITTER_STEPS) as f64 / course::JITTER_STEPS as f64) - 1.0 }

struct Aerwhale;

#[entity_behavior]
impl EntityBehavior for Aerwhale
{
    fn onTick(&self, entity: Entity, info: &EntityInfo)
    {
        entity.setField(EntityField::FireTicks, 0.0);

        if info.pos[1] < course::DESPAWN_Y
        {
            entity.kill();
        }
    }

    // AerwhaleAIUnstuck and AerwhaleAITravelCourse, the ghast template's own AI never runs
    fn onAi(&self, entity: Entity, info: &EntityInfo)
    {
        let world: World = entity.world;
        let mut motion: [f64; 3] = info.motion;
        unstuck(world, info, &mut motion);

        let distances: [f64; 5] = [
            openDistance(world, info, 0.0, 0.0),
            openDistance(world, info, course::SIDE, 0.0),
            openDistance(world, info, 0.0, course::SIDE),
            openDistance(world, info, -course::SIDE, 0.0),
            openDistance(world, info, 0.0, -course::SIDE),
        ];

        let best: usize = (1..distances.len()).fold(0, |best, index|
        {
            if distances[index] > distances[best]
            {
                index
            }
            else
            {
                best
            }
        });

        let mut heading: Heading =
            HEADINGS.with(|headings| headings.borrow().get(&entity.id).copied()).unwrap_or_default();
        let mut yaw: f32 = info.yaw;
        let mut pitch: f32 = info.pitch;

        match best
        {
            0 if distances[0] == course::SIGHT =>
            {
                heading.m_yaw *= course::DAMPING;
                heading.m_pitch *= course::DAMPING;

                if info.pos[1] > course::CEILING
                {
                    heading.m_pitch -= course::CLIMB;
                }

                if info.pos[1] < course::FLOOR
                {
                    heading.m_pitch += course::CLIMB;
                }
            }

            0 =>
            {
                pitch = -pitch;
                yaw = -yaw;
            }

            1 => heading.m_yaw += course::TURN,
            2 => heading.m_pitch -= course::TURN,
            3 => heading.m_yaw -= course::TURN,
            _ => heading.m_pitch += course::TURN,
        }

        heading.m_yaw += jitter(world);
        heading.m_pitch += jitter(world);
        pitch = ((pitch as f64 + course::STEER * heading.m_pitch) as f32).clamp(-course::MAX_PITCH, course::MAX_PITCH)
            * course::PITCH_EASE;
        yaw = (yaw as f64 + course::STEER * heading.m_yaw) as f32;
        let (yawRadians, pitchRadians): (f64, f64) = ((yaw as f64).to_radians(), (pitch as f64).to_radians());
        motion[0] = (motion[0] + course::THRUST * yawRadians.cos() * pitchRadians.cos()) * course::DRAG;
        motion[1] = (motion[1] + course::THRUST * pitchRadians.sin()) * course::DRAG;
        motion[2] = (motion[2] + course::THRUST * yawRadians.sin() * pitchRadians.cos()) * course::DRAG;

        let at: BlockPos =
            BlockPos::new(info.pos[0].floor() as i32, info.pos[1].floor() as i32, info.pos[2].floor() as i32);
        let bounces: [(usize, [i32; 3], f64, bool); 3] = [
            (0, [1, 0, 0], course::BOUNCE_TURN, true),
            (1, [0, 1, 0], course::BOUNCE_TURN, false),
            (2, [0, 0, 1], course::BOUNCE_TURN, true),
        ];

        for (axis, step, turn, yawTurn) in bounces
        {
            let sign: i32 = if motion[axis] > 0.0
            {
                1
            }
            else if motion[axis] < 0.0
            {
                -1
            }
            else
            {
                0
            };

            if sign != 0 && !world.isAir(at.offset(step[0] * sign, step[1] * sign, step[2] * sign))
            {
                motion[axis] = -motion[axis];

                let change: f64 = -turn * sign as f64;

                if yawTurn
                {
                    heading.m_yaw += change
                }
                else
                {
                    heading.m_pitch += change
                }
            }
        }

        HEADINGS.with(|headings| headings.borrow_mut().insert(entity.id, heading));
        entity.setField(EntityField::Yaw, yaw as f64);
        entity.setField(EntityField::Pitch, pitch as f64);

        let _ = entity.setMotion(motion);
    }

    fn canSpawn(&self, entity: Entity, info: &EntityInfo) -> bool
    {
        crate::config::spawnRoll(entity, "aerwhale") && onAetherGrass(info)
    }

    // AerwhaleRenderer.preRenderCallback lifts, turns and doubles the model, which draws at rest
    fn pose(&self, _input: &PoseInput) -> Pose
    {
        let mut pre: Program = Program::new();
        pre.translate(0.0, 1.2, 0.0).rotate(-90.0, 0.0, 1.0, 0.0).scale(2.0, 2.0, 2.0);

        let mut body: Program = Program::new();

        for part in [
            model::FRONT_BODY,
            model::RIGHT_FIN,
            model::BOTTOM_PART_HEAD,
            model::LEFT_FIN,
            model::BOTTOM_PART_MIDDLEBODY,
            model::HEAD,
            model::MIDDLE_FIN,
            model::BACKFIN_RIGHT,
            model::BACK_BODY,
            model::BACKFIN_LEFT,
            model::MIDDLEBODY,
        ]
        {
            body.rest(part);
        }
        Pose { pre: pre.take(), model: body.take() }
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.entities().register(
        NAME,
        EntitySpec
        {
            template: Identifier::parse("minecraft:ghast").ok(),
            texture: Some(TEXTURE.to_owned()),
            size: Some([3.0, 3.0]),
            health: Some(20),
            spawn: creature(12),
            drops: Some(Vec::new()),
            sounds: sounds("aether.aemob.aerwhale.call", "aether.aemob.aerwhale.death", "aether.aemob.aerwhale.death"),
            model: Some(EntityModelSpec { shadow: 0.5, ..model::model() }),
            ..EntitySpec::default()
        },
        Some(std::rc::Rc::new(Aerwhale)),
    )?;
    Ok(())
}
