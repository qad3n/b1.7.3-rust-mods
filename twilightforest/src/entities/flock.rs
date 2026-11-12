use super::ai::target;
use b173_api::math::{blockOf, distance, flatLength, floor};
use b173_api::prelude::*;
use std::cell::{RefCell, RefMut};

// EntityAITFFlockToSameKind, EntityAITFPanicOnFlockDeath and EntityAILeapAtTarget, the kobold's pack habits
mod tuning
{
    pub const FLOCK_CHANCE: i32 = 40;
    pub const FLOCK_REACH: f64 = 16.0;
    pub const FLOCK_NEAR: f64 = 5.0;
    pub const PANIC_REACH: f64 = 4.0;
    pub const PANIC_TICKS: i64 = 40;
    // a kobold that died this recently still frightens its pack, EntityLiving.deathTime runs twenty ticks
    pub const DEATH_TICKS: i64 = 20;
    pub const FLEE: i32 = 5;
    pub const SPEED: f32 = 1.0;
    pub const PANIC_SPEED: f32 = 2.0;
    pub const LEAP_FROM: f64 = 2.0;
    pub const LEAP_TO: f64 = 4.0;
    pub const LEAP_CHANCE: i32 = 5;
    pub const LEAP_PUSH: f64 = 0.4;
    pub const LEAP_LIFT: f64 = 0.3;
    // the leap keeps a fifth of the mob's own motion
    pub const LEAP_KEEP: f64 = 0.2;
    pub const SPLASHES: i32 = 2;
    pub const SPLASH_SPREAD: f64 = 0.5;
    pub const EYES: f64 = 0.85;
}

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct Panic
{
    m_panicked: bool,
}

thread_local!
{
    // where and when pack members died, by dimension
    static DEATHS: RefCell<Vec<(i32, [f64; 3], i64)>> = const { RefCell::new(Vec::new()) };
}

// the time a pack member stops fleeing
static FLEEING: EntityState<i64> = EntityState::new("fleeing");

fn same(world: World, info: &EntityInfo, reach: f64) -> Vec<EntityInfo>
{
    world.entitiesNear(info.pos, reach).into_iter().filter(|other| other.kind == info.kind).collect()
}

pub(crate) fn died(entity: Entity, info: &EntityInfo)
{
    let now: i64 = entity.world.time();

    DEATHS.with(|deaths|
    {
        let mut deaths: RefMut<Vec<(i32, [f64; 3], i64)>> = deaths.borrow_mut();
        deaths.retain(|(_, _, when)| now - when < tuning::DEATH_TICKS);
        deaths.push((entity.world.dimension, info.pos, now));
    });

    super::clear(&FLEEING, entity);
}

fn frightened(world: World, info: &EntityInfo) -> bool
{
    let now: i64 = world.time();

    DEATHS.with(|deaths|
    {
        deaths.borrow().iter().any(|(dimension, pos, when)|
        {
            *dimension == world.dimension
                && now - when < tuning::DEATH_TICKS
                && distance(*pos, info.pos) <= tuning::PANIC_REACH
        })
    })
}

fn setPanicked(entity: Entity, panicked: bool)
{
    let shown: Panic = entity.synced();
    if shown.m_panicked == panicked
    {
        return;
    }

    if let Err(error) = entity.setSynced(&Panic { m_panicked: panicked })
    {
        warn(&format!("kobold panic: {}", error.0));
    }
}

// EntityAITFPanicOnFlockDeath, a run to a random spot while the pack mourns
fn panic(entity: Entity, info: &EntityInfo) -> bool
{
    let world: World = entity.world;
    let now: i64 = world.time();
    let until: Option<i64> = FLEEING.find(entity).filter(|until| *until > now);
    if until.is_none() && !frightened(world, info)
    {
        setPanicked(entity, false);
        return false;
    }

    if until.is_none()
    {
        super::store(&FLEEING, entity, &(now + tuning::PANIC_TICKS));

        let spot: BlockPos = BlockPos::new(
            floor(info.pos[0]) + world.random(tuning::FLEE * 2 + 1) - tuning::FLEE,
            floor(info.pos[1]),
            floor(info.pos[2]) + world.random(tuning::FLEE * 2 + 1) - tuning::FLEE,
        );
        entity.walkTo(spot, tuning::PANIC_SPEED);
    }

    setPanicked(entity, true);
    true
}

// EntityAILeapAtTarget
fn leap(entity: Entity, info: &EntityInfo) -> bool
{
    let Some(goal) = target(entity)
    else
    {
        return false;
    };
    let gap: f64 = distance(goal.pos, info.pos);
    let ready: bool = entity.flag(EntityField::OnGround) && (tuning::LEAP_FROM..=tuning::LEAP_TO).contains(&gap);
    if !ready || entity.world.random(tuning::LEAP_CHANCE) != 0
    {
        return false;
    }

    let (dx, dz): (f64, f64) = (goal.pos[0] - info.pos[0], goal.pos[2] - info.pos[2]);
    let flat: f64 = flatLength(dx, dz).max(1.0e-3);

    let push = |delta: f64, motion: f64|
    {
        delta / flat * tuning::LEAP_PUSH * (1.0 - tuning::LEAP_KEEP) + motion * tuning::LEAP_KEEP
    };
    let _ = entity.setMotion([push(dx, info.motion[0]), tuning::LEAP_LIFT, push(dz, info.motion[2])]);

    true
}

// EntityAITFFlockToSameKind, now and then a walk toward the middle of the pack
fn gather(entity: Entity, info: &EntityInfo)
{
    let world: World = entity.world;
    if world.random(tuning::FLOCK_CHANCE) != 0 || target(entity).is_some()
    {
        return;
    }
    let pack: Vec<EntityInfo> = same(world, info, tuning::FLOCK_REACH);
    if pack.is_empty()
    {
        return;
    }
    let count: f64 = pack.len() as f64;
    let middle: [f64; 3] = [0, 1, 2].map(|axis| pack.iter().map(|other| other.pos[axis]).sum::<f64>() / count);
    if distance(middle, info.pos) < tuning::FLOCK_NEAR
    {
        return;
    }

    entity.walkTo(blockOf(middle), tuning::SPEED);
}

// EntityTFKobold.onLivingUpdate splashes sweat while it panics
fn sweat(entity: Entity, info: &EntityInfo)
{
    let panic: Panic = entity.synced();
    if !panic.m_panicked
    {
        return;
    }

    let world: World = entity.world;
    let width: f64 = entity.field(EntityField::Width) * tuning::SPLASH_SPREAD;
    let height: f64 = entity.field(EntityField::Height) * tuning::EYES;

    for _ in 0..tuning::SPLASHES
    {
        let at: [f64; 3] = [
            info.pos[0] + (f64::from(crate::math::fraction(world)) - 0.5) * width,
            info.pos[1] + height,
            info.pos[2] + (f64::from(crate::math::fraction(world)) - 0.5) * width,
        ];
        world.particle("splash", at, [0.0; 3]);
    }
}

// the sweat is the client's own, the pack habits run where the world is owned
pub(crate) fn kobold(entity: Entity, info: &EntityInfo)
{
    if !entity.world.isAuthority()
    {
        sweat(entity, info);
        return;
    }

    if !panic(entity, info) && !leap(entity, info)
    {
        gather(entity, info);
    }
}

pub(crate) fn leaper(entity: Entity, info: &EntityInfo)
{
    if entity.world.isAuthority()
    {
        leap(entity, info);
    }
}
