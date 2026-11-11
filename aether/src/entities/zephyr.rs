use super::projectile::{self, ZEPHYR_SNOWBALL};
use super::{closestPlayer, monster, onAetherGrass, sounds};
use crate::models::{Program, zephyr as model};
use b173_api::math::{distanceSquared, sub};
use b173_api::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

const NAME: &str = "zephyr";
const TEXTURE: &str = "aether:entity/zephyr/zephyr_main";
const LAYER: &str = "aether:entity/zephyr/zephyr_layer";
const CALL: &str = "aether.aemob.zephyr.call";

mod flight
{
    // EntityZephyr.updateEntityActionState, the ghast's wandering with a 16 block waypoint box
    pub const WAYPOINT_SPREAD: f64 = 16.0;
    pub const NEAR: f64 = 1.0;
    pub const FAR: f64 = 3600.0;
    pub const PUSH: f64 = 0.1;
    pub const COOLDOWN_MIN: i32 = 2;
    pub const COOLDOWN_SPREAD: i32 = 5;
    pub const SIGHT: f64 = 100.0;
    pub const RANGE_SQUARED: f64 = 4096.0;
    pub const WARN: i32 = 10;
    pub const FIRE: i32 = 20;
    pub const RELOAD: i32 = -40;
    pub const SHOT_SPEED: f64 = 1.2;
    pub const SHOT_SPREAD: f64 = 1.0;
    pub const MUZZLE: f64 = 4.0;
    pub const HEIGHT: f64 = 4.0;
    pub const RANDOM_STEPS: i32 = 1000000;
}

#[derive(Default, Clone, Copy)]
struct Course
{
    m_waypoint: [f64; 3],
    m_cooldown: i32,
    m_attack: i32,
}

thread_local!
{
    static COURSES: RefCell<HashMap<i32, Course>> = RefCell::new(HashMap::new());
}

// the attack counter the client draws the squash from
#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct Charge
{
    m_counter: i32,
}

fn unit(world: World) -> f64 { world.random(flight::RANDOM_STEPS) as f64 / flight::RANDOM_STEPS as f64 * 2.0 - 1.0 }

struct Zephyr;

#[entity_behavior]
impl EntityBehavior for Zephyr
{
    // ZephyrAITravelCourse and ZephyrAIShootTarget in place of the ghast template's AI
    fn onAi(&self, entity: Entity, info: &EntityInfo)
    {
        let world: World = entity.world;
        let mut course: Course = COURSES.with(|courses| courses.borrow().get(&entity.id).copied()).unwrap_or_default();
        let mut motion: [f64; 3] = info.motion;
        let offset: [f64; 3] = sub(course.m_waypoint, info.pos);
        let squared: f64 = offset.iter().map(|value| value * value).sum();
        if squared < flight::NEAR || squared > flight::FAR
        {
            course.m_waypoint = [0, 1, 2].map(|axis| info.pos[axis] + unit(world) * flight::WAYPOINT_SPREAD);
        }

        course.m_cooldown -= 1;
        if course.m_cooldown <= 0
        {
            course.m_cooldown += world.random(flight::COOLDOWN_SPREAD) + flight::COOLDOWN_MIN;

            let distance: f64 = squared.sqrt().max(1.0e-4);

            if world.rayTrace(info.pos, course.m_waypoint, false).is_none()
            {
                for axis in 0..3
                {
                    motion[axis] += offset[axis] / distance * flight::PUSH;
                }
            }
            else
            {
                course.m_waypoint = info.pos;
            }
        }

        let center: [f64; 3] = [info.pos[0], info.pos[1] + flight::HEIGHT / 2.0, info.pos[2]];

        let target: Option<EntityInfo> = closestPlayer(world, info, flight::SIGHT).filter(|player|
        {
            let gap: f64 = distanceSquared(player.pos, info.pos);
            gap < flight::RANGE_SQUARED
                && world.rayTrace(center, [player.pos[0], player.pos[1] + 1.0, player.pos[2]], false).is_none()
        });

        let before: i32 = course.m_attack;

        match target
        {
            Some(player) =>
            {
                let [dx, dz] = [player.pos[0] - info.pos[0], player.pos[2] - info.pos[2]];
                entity.setField(EntityField::Yaw, -dx.atan2(dz).to_degrees());
                course.m_attack += 1;
                if course.m_attack == flight::WARN
                {
                    world.playSound(info.pos, CALL, 3.0, 1.0);
                }
                else if course.m_attack == flight::FIRE
                {
                    world.playSound(info.pos, CALL, 3.0, 1.0);

                    let yaw: f64 = (-dx.atan2(dz)).to_radians();
                    let muzzle: [f64; 3] = [
                        info.pos[0] - yaw.sin() * flight::MUZZLE,
                        center[1] + 0.5,
                        info.pos[2] + yaw.cos() * flight::MUZZLE,
                    ];

                    let aim: [f64; 3] = [player.pos[0], player.pos[1] + 1.0, player.pos[2]];
                    projectile::fire(
                        world,
                        ZEPHYR_SNOWBALL,
                        muzzle,
                        aim,
                        flight::SHOT_SPEED,
                        flight::SHOT_SPREAD,
                        Some(entity.id),
                    );
                    course.m_attack = flight::RELOAD;
                }
            }

            None if course.m_attack > 0 => course.m_attack -= 1,
            None =>
            {}
        }

        if course.m_attack != before
        {
            let _ = entity.setSynced(&Charge { m_counter: course.m_attack });
        }

        COURSES.with(|courses| courses.borrow_mut().insert(entity.id, course));

        let _ = entity.setMotion(motion);
    }

    fn canSpawn(&self, entity: Entity, info: &EntityInfo) -> bool
    {
        crate::config::spawnRoll(entity, "zephyr") && onAetherGrass(info)
    }

    // ZephyrRenderer.renderZephyrMovement squashes it while it charges, ZephyrModel sways its sides and tail
    fn pose(&self, input: &PoseInput) -> Pose
    {
        let charge: Charge = b173_api::fromBytes(&input.synced).unwrap_or_default();
        let counter: f32 = (charge.m_counter as f32 / flight::FIRE as f32).max(0.0);
        let squash: f32 = 1.0 / (counter.powi(5) * 2.0 + 1.0);
        let tall: f32 = (8.0 + squash) / 2.0;
        let wide: f32 = (8.0 + 1.0 / squash) / 2.0;
        let mut pre: Program = Program::new();
        pre.scale(wide, tall, wide).translate(0.0, 0.5, 0.0).color(1.0, 1.0, 1.0, 1.0);

        let wave = |rate: f32| (input.walk * rate / 57.29578).sin() * input.walk_speed;
        let motion: f32 = wave(20.0) * 0.5;
        let rotation = |part: u16| model::ROTATIONS[part as usize];
        let pivot = |part: u16| model::PIVOTS[part as usize];
        let tailYaw: f32 = wave(0.5) * 0.75;
        let tail2Yaw: f32 = tailYaw + 0.25;
        let mut body: Program = Program::new();

        let sides = |program: &mut Program|
        {
            program.part(
                model::TAIL3,
                [rotation(model::TAIL3)[0], tail2Yaw + 0.35, 0.0],
                [wave(10.0) * 0.95, -motion, pivot(model::TAIL3)[2]],
            );
            program.part(
                model::TAIL2,
                [rotation(model::TAIL2)[0], tail2Yaw, 0.0],
                [wave(15.0) * 0.85, motion * 1.25, pivot(model::TAIL2)[2]],
            );
            program.part(
                model::LEFT_FACE,
                rotation(model::LEFT_FACE),
                [motion * 0.5, motion + 8.0, pivot(model::LEFT_FACE)[2]],
            );
            program.part(
                model::BODY_RIGHT_SIDE2,
                rotation(model::BODY_RIGHT_SIDE2),
                [pivot(model::BODY_RIGHT_SIDE2)[0], 9.0 + motion * 0.5, pivot(model::BODY_RIGHT_SIDE2)[2]],
            );
            program.rest(model::MOUTH).rest(model::CLOUD_BUTT);
            program.part(
                model::RIGHT_FACE,
                rotation(model::RIGHT_FACE),
                [-motion * 0.5, 8.0 - motion, pivot(model::RIGHT_FACE)[2]],
            );
            program.part(
                model::BODY_LEFT_SIDE1,
                rotation(model::BODY_LEFT_SIDE1),
                [pivot(model::BODY_LEFT_SIDE1)[0], 8.0 - motion * 0.5, pivot(model::BODY_LEFT_SIDE1)[2]],
            );
            program.part(
                model::BODY_LEFT_SIDE2,
                rotation(model::BODY_LEFT_SIDE2),
                [pivot(model::BODY_LEFT_SIDE2)[0], 9.0 + motion * 0.5, pivot(model::BODY_LEFT_SIDE2)[2]],
            );
            program.rest(model::BODY);
            program.part(
                model::BODY_RIGHT_SIDE1,
                rotation(model::BODY_RIGHT_SIDE1),
                [pivot(model::BODY_RIGHT_SIDE1)[0], 8.0 - motion * 0.5, pivot(model::BODY_RIGHT_SIDE1)[2]],
            );
            program.part(
                model::TAIL1,
                [rotation(model::TAIL1)[0], tailYaw, 0.0],
                [wave(20.0) * 0.75, 8.0 - motion, pivot(model::TAIL1)[2]],
            );
        };
        sides(&mut body);
        body.texture(LAYER).blend(true);
        sides(&mut body);
        body.blend(false);
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
            size: Some([4.0, 4.0]),
            // 1.7 max health default, the class sets none
            health: Some(20),
            spawn: monster(50),
            // EntityZephyr.dropFewItems
            drops: Some(vec![super::drop(crate::ids::block::COLD_AERCLOUD, [1, 1])]),
            sounds: sounds(CALL, "aether.aemob.zephyr.call", "aether.aemob.zephyr.call"),
            model: Some(EntityModelSpec { shadow: 0.5, ..model::model() }),
            ..EntitySpec::default()
        },
        Some(std::rc::Rc::new(Zephyr)),
    )?;
    Ok(())
}
