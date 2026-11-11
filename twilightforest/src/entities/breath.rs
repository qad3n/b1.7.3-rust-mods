use super::ai::{eyes, face, target};
use crate::ids;
use crate::math::{gaussian, look};
use b173_api::math::distance;
use b173_api::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

// EntityAITFBreathAttack with IBreathAttacker: a winter wolf breathes frost and a fire beetle breathes fire at what
// it hunts within five blocks, the beetle's flames burn what they reach
#[derive(PartialEq)]
pub(crate) struct Breath
{
    pub m_particle: &'static str,
    pub m_puffs: i32,
    // how far ahead of the head and how high above the feet the breath leaves
    pub m_mouth: [f64; 2],
    // the puffs' base speed, each adds up to another fifteen hundredths
    pub m_speed: f64,
    pub m_burns: bool,
}

pub(crate) const FROST: Breath =
    Breath { m_particle: "snowstuff", m_puffs: 10, m_mouth: [0.5, 1.25], m_speed: 3.0, m_burns: false };
pub(crate) const FIRE: Breath =
    Breath { m_particle: "flame", m_puffs: 2, m_mouth: [0.9, 0.25], m_speed: 0.15, m_burns: true };

mod tuning
{
    pub const RANGE: f64 = 5.0;
    pub const TICKS: i32 = 30;
    pub const CHANCE: f32 = 0.1;
    // the breath only bites once it has run five ticks
    pub const WARMUP: i32 = 5;
    pub const REACH: f64 = 30.0;
    pub const HIT_RADIUS: f64 = 1.0;
    pub const BURN_DAMAGE: i32 = 2;
    pub const BURN_TICKS: f64 = 200.0;
    pub const SPEED_SPREAD: f64 = 0.15;
    pub const SPREAD: f64 = 5.0;
    pub const SPREAD_RANGE: f64 = 2.5;
    pub const SCATTER: f64 = 0.0075;
    pub const SOUND_SPREAD: f32 = 0.5;
}

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct Breathing
{
    m_on: bool,
}

// whether a breather's synced state says it breathes, for drawing on the client
pub(crate) fn breathing(synced: &[u8]) -> bool
{
    b173_api::fromBytes::<Breathing>(synced).is_ok_and(|state| state.m_on)
}

thread_local!
{
    static LEFT: RefCell<HashMap<(i32, i32), i32>> = RefCell::new(HashMap::new());
}

pub(crate) fn forget(entity: Entity)
{
    LEFT.with(|left| left.borrow_mut().remove(&(entity.world.dimension, entity.id)));
}

fn setOn(entity: Entity, on: bool)
{
    if entity.synced::<Breathing>().m_on == on
    {
        return;
    }

    if let Err(error) = entity.setSynced(&Breathing { m_on: on })
    {
        warn(&format!("breath: {}", error.0));
    }
}

fn inReach(entity: Entity, info: &EntityInfo, goal: &EntityInfo) -> bool
{
    let seen: bool = entity
        .world
        .rayTrace(eyes(entity, info), eyes(Entity { world: entity.world, id: goal.id }, goal), false)
        .is_none();
    seen && distance(goal.pos, info.pos) <= tuning::RANGE
}

// the IBreathAttacker.doBreathAttack of the fire beetle, whatever its head points at catches fire
fn scorch(entity: Entity, info: &EntityInfo)
{
    let world: World = entity.world;
    let aim: [f64; 3] = look(info.yaw, info.pitch);
    let from: [f64; 3] = info.pos;
    let to: [f64; 3] = [0, 1, 2].map(|axis| from[axis] + aim[axis] * tuning::REACH);
    let hit: Option<EntityInfo> = world
        .entitiesAlong(from, to, tuning::HIT_RADIUS, Some(entity.id))
        .into_iter()
        .find(|other| other.health.is_some());

    let Some(hit) = hit
    else
    {
        return;
    };

    let victim: Entity = Entity { world, id: hit.id };
    let burning: bool = victim.field(EntityField::FireTicks) > 0.0;
    if !burning && crate::combat::hit(victim, tuning::BURN_DAMAGE, Some(entity.id), crate::combat::kind::IN_FIRE)
    {
        victim.setField(EntityField::FireTicks, tuning::BURN_TICKS);
    }
}

// the server side, EntityAITFBreathAttack's start, continue and update
pub(crate) fn tick(entity: Entity, info: &EntityInfo, breath: &Breath)
{
    let key: (i32, i32) = (entity.world.dimension, entity.id);
    let left: i32 = LEFT.with(|left| left.borrow().get(&key).copied().unwrap_or(0));
    let goal: Option<EntityInfo> = target(entity).filter(|goal| inReach(entity, info, goal));
    let next: i32 = match goal
    {
        Some(_) if left > 0 => left - 1,
        Some(_) if ids::fraction(entity.world) < tuning::CHANCE => tuning::TICKS,
        _ => 0,
    };
    LEFT.with(|left| left.borrow_mut().insert(key, next));
    setOn(entity, next > 0);

    let Some(goal) = goal.filter(|_| next > 0)
    else
    {
        return;
    };

    face(entity, info.pos, goal.pos);

    if breath.m_burns && tuning::TICKS - next > tuning::WARMUP
    {
        scorch(entity, info);
    }
}

// onLivingUpdate on the client, puffs from the mouth and the hiss of the breath
pub(crate) fn puff(entity: Entity, info: &EntityInfo, breath: &Breath)
{
    let world: World = entity.world;

    if !entity.synced::<Breathing>().m_on
    {
        return;
    }

    let aim: [f64; 3] = look(info.yaw, info.pitch);
    let [ahead, height] = breath.m_mouth;
    let mouth: [f64; 3] =
        [info.pos[0] + aim[0] * ahead, info.pos[1] + height + aim[1] * ahead, info.pos[2] + aim[2] * ahead];
    let draw = || f64::from(ids::fraction(world));

    for _ in 0..breath.m_puffs
    {
        let spread: f64 = tuning::SPREAD + draw() * tuning::SPREAD_RANGE;
        let speed: f64 = breath.m_speed + draw() * tuning::SPEED_SPREAD;
        let motion: [f64; 3] = aim.map(|axis| (axis + gaussian(draw(), draw()) * tuning::SCATTER * spread) * speed);
        crate::particles::spawn(world, breath.m_particle, mouth, motion);
    }

    let (volume, pitch): (f32, f32) =
        (ids::fraction(world) * tuning::SOUND_SPREAD, ids::fraction(world) * tuning::SOUND_SPREAD);
    world.playSound(info.pos, "mob.ghast.fireball", volume, pitch);
}
