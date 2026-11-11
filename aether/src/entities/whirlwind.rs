use super::{closestPlayer, fraction, monster, onAetherGrass};
use b173_api::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

const NAME: &str = "whirlwind";

// EntityWhirlwind.getRandomDrop: a roll of 1 to 100 at or above the bound picks the item
const DROPS: [(i32, &str); 11] = [
    (100, "minecraft:diamond"),
    (96, "minecraft:iron_ingot"),
    (91, "minecraft:gold_ingot"),
    (82, "minecraft:coal"),
    (80, "minecraft:pumpkin"),
    (75, "minecraft:gravel"),
    (64, "minecraft:clay"),
    (52, "minecraft:stick"),
    (38, "minecraft:flint"),
    (21, "minecraft:log"),
    (1, "minecraft:sand"),
];

mod wind
{
    pub const LIFE_MIN: i32 = 512;
    pub const LIFE_SPREAD: i32 = 512;
    pub const EVIL_ODDS: i32 = 10;
    pub const SPEED_MIN: f64 = 0.025;
    pub const SPEED_SPREAD: f64 = 0.025;
    pub const CURVE: f64 = 0.1;
    pub const SIGHT: f64 = 16.0;
    pub const ACTION_TICKS: i32 = 128;
    pub const DROP_ODDS: i32 = 4;
    pub const REACH: f64 = 2.5;
    pub const CATCH: f64 = 1.5;
    pub const LIFT: f64 = 0.125;
    pub const HIGH_DROP: f64 = -0.45;
    pub const HIGH_LIFT: f64 = 0.35;
    pub const SPIN: f64 = 160.0;
    pub const SPIN_PULL: f64 = 0.1;
    pub const DRIFT: f64 = 0.01;
    pub const BLOCKED_COST: i32 = 50;
    pub const PARTICLES: i32 = 3;
    pub const TOP: f64 = 0.925;
    pub const RADIUS: f64 = 0.75;
    pub const FULL_TURN: f64 = 360.0;
    pub const DROP_ROLL: i32 = 100;
    pub const PULL_OFFSET: f64 = 0.25;
    // an evil whirlwind closes in on its target four times as fast as it drifts
    pub const CHASE: f64 = 4.0;
    pub const SPAWN_LIFT: f64 = 0.5;
    pub const POP_VOLUME: f32 = 0.5;
}

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct Wind
{
    m_ready: bool,
    m_evil: bool,
}

#[derive(Clone, Copy)]
struct Course
{
    m_life: i32,
    m_timer: i32,
    m_angle: f64,
    m_curve: f64,
    m_speed: f64,
}

thread_local!
{
    static COURSES: RefCell<HashMap<i32, Course>> = RefCell::new(HashMap::new());
}

fn wind(entity: Entity) -> Wind
{
    let mut state: Wind = entity.synced();
    if !state.m_ready && entity.world.isAuthority()
    {
        state = Wind { m_ready: true, m_evil: entity.world.random(wind::EVIL_ODDS) == 0 };
        let _ = entity.setSynced(&state);
    }

    state
}

fn course(entity: Entity, evil: bool) -> Course
{
    COURSES.with(|courses|
    {
        *courses.borrow_mut().entry(entity.id).or_insert_with(||
        {
            let world: World = entity.world;
            let life: i32 = wind::LIFE_MIN + world.random(wind::LIFE_SPREAD);

            Course
            {
                m_life: if evil { life / 2 } else { life },
                m_timer: 0,
                m_angle: fraction(world) * wind::FULL_TURN,
                m_curve: (fraction(world) - fraction(world)) * wind::CURVE,
                m_speed: wind::SPEED_MIN + fraction(world) * wind::SPEED_SPREAD,
            }
        })
    })
}

fn randomDrop(world: World) -> &'static str
{
    let roll: i32 = world.random(wind::DROP_ROLL) + 1;
    DROPS.iter().find(|(bound, _)| roll >= *bound).map_or("minecraft:sand", |(_, item)| item)
}

// the whirlwind pulls everything near into its funnel and spins it around
fn pull(world: World, info: &EntityInfo, entity: Entity)
{
    for other in world.entitiesNear(info.pos, wind::REACH).into_iter().filter(|other| other.id != entity.id)
    {
        let caught: Entity = Entity { world, id: other.id };
        let [dx, dz] = [info.pos[0] - other.pos[0], info.pos[2] - other.pos[2]];
        let height: f64 = other.pos[1] - info.pos[1];
        let mut distance: f64 = ((0..3).map(|axis| (other.pos[axis] - info.pos[axis]).powi(2)).sum::<f64>()).sqrt();
        let angle: f64 = dx.atan2(dz).to_degrees();

        if distance <= wind::CATCH + height
        {
            caught.setField(EntityField::FallDistance, 0.0);

            let lift: f64 = if height > wind::CATCH
            {
                distance += height * wind::CATCH;
                wind::HIGH_DROP + height * wind::HIGH_LIFT
            }
            else
            {
                wind::LIFT
            };
            let spin: f64 = (angle + wind::SPIN).to_radians();
            let pullBy: f64 = (distance + wind::PULL_OFFSET) * wind::SPIN_PULL;
            let _ = caught.setMotion([-spin.cos() * pullBy, lift, spin.sin() * pullBy]);

            if other.kind.as_ref().is_some_and(|kind| kind.path() == NAME)
            {
                caught.kill();
            }
        }
        else
        {
            let toward: f64 = angle.to_radians();
            let _ = caught.setMotion([
                other.motion[0] + toward.sin() * wind::DRIFT,
                other.motion[1],
                other.motion[2] + toward.cos() * wind::DRIFT,
            ]);
        }
    }
}

struct Whirlwind;

#[entity_behavior]
impl EntityBehavior for Whirlwind
{
    // EntityWhirlwind.onLivingUpdate
    fn onAi(&self, entity: Entity, info: &EntityInfo)
    {
        let world: World = entity.world;
        let state: Wind = wind(entity);
        let mut current: Course = course(entity, state.m_evil);
        let player: Option<EntityInfo> = closestPlayer(world, info, wind::SIGHT);
        let chasing: Option<&EntityInfo> = player.as_ref().filter(|_| state.m_evil);
        match chasing
        {
            Some(target) =>
            {
                let [dx, dz] = [target.pos[0] - info.pos[0], target.pos[2] - info.pos[2]];
                let length: f64 = (dx * dx + dz * dz).sqrt().max(1.0e-4);
                let chase: f64 = current.m_speed * wind::CHASE / length;
                let _ = entity.setMotion([dx * chase, info.motion[1], dz * chase]);
            }

            None =>
            {
                let angle: f64 = current.m_angle.to_radians();
                let _ =
                    entity.setMotion([angle.cos() * current.m_speed, info.motion[1], -angle.sin() * current.m_speed]);
                current.m_angle += current.m_curve;
            }
        }

        current.m_life -= 1;

        let below: BlockPos =
            BlockPos::new(info.pos[0].floor() as i32, info.pos[1].floor() as i32, info.pos[2].floor() as i32);

        if !world.isAir(below)
        {
            current.m_life -= wind::BLOCKED_COST;
        }

        if world.isAuthority()
        {
            if current.m_life <= 0 || entity.flag(EntityField::InWater)
            {
                entity.kill();
                return;
            }

            if player.is_some()
            {
                current.m_timer += 1;
            }

            if current.m_timer >= wind::ACTION_TICKS
            {
                if state.m_evil
                {
                    let _ = world
                        .spawnEntity("minecraft:creeper", [info.pos[0], info.pos[1] + wind::SPAWN_LIFT, info.pos[2]]);
                    world.playSound(info.pos, "random.pop", wind::POP_VOLUME, 1.0);
                    current.m_timer = 0;
                }
                else if world.random(wind::DROP_ODDS) == 0
                {
                    let _ = world.spawnItem(info.pos, ItemStack::of(randomDrop(world), 1));

                    world.playSound(info.pos, "random.pop", wind::POP_VOLUME, 1.0);
                    current.m_timer = 0;
                }
            }
        }

        pull(world, info, entity);
        COURSES.with(|courses| courses.borrow_mut().insert(entity.id, current));
    }

    // EntityWhirlwind.updateParticles: a funnel of spinning puffs, dark for an evil one
    fn onTick(&self, entity: Entity, info: &EntityInfo)
    {
        if entity.world.isAuthority() && !isClient()
        {
            return;
        }

        let state: Wind = entity.synced();
        let name: &str = if state.m_evil
        {
            crate::particles::EVIL_WHIRLY
        }
        else
        {
            crate::particles::WHIRLY
        };

        for _ in 0..wind::PARTICLES
        {
            let angle: f64 = (fraction(entity.world) * wind::FULL_TURN).to_radians();
            let height: f64 = fraction(entity.world) * wind::TOP * 2.0;
            let spot: [f64; 3] = [
                info.pos[0] - angle.sin() * wind::RADIUS * height,
                info.pos[1] + height,
                info.pos[2] + angle.cos() * wind::RADIUS * height,
            ];
            entity.world.particle(
                name,
                spot,
                [angle.cos() * wind::SPIN_PULL, wind::LIFT, angle.sin() * wind::SPIN_PULL],
            );
        }
    }

    fn canSpawn(&self, entity: Entity, info: &EntityInfo) -> bool
    {
        crate::config::spawnRoll(entity, "whirlwind") && onAetherGrass(info)
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.entities().register(
        NAME,
        EntitySpec
        {
            template: Identifier::parse("minecraft:monster").ok(),
            size: Some([0.6, 0.8]),
            health: Some(10),
            spawn: monster(10),
            drops: Some(Vec::new()),
            sounds: Some(EntitySounds
            {
                living: Some(String::new()),
                hurt: Some(String::new()),
                death: Some(String::new()),
            }),
            model: Some(EntityModelSpec { shadow: 0.0, ..EntityModelSpec::default() }),
            ..EntitySpec::default()
        },
        Some(std::rc::Rc::new(Whirlwind)),
    )?;
    Ok(())
}
