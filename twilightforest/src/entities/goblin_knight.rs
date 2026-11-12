use super::ai::target;
use crate::models::
{
    DEGREES_PER_RADIAN, Program, WALK_RATE, biped, bipedPose, tf_goblin_knight_lower, tf_goblin_knight_upper,
};
use b173_api::math;
use b173_api::math::{distance, flatLength};
use b173_api::prelude::*;
use std::f32::consts::{PI, TAU};

// EntityTFGoblinKnightLower carries EntityTFGoblinKnightUpper, the rider's shield guards their backs, both wear
// armor that a blow from the front breaks, and the rider stops to bring its spear down in a heavy attack
const UPPER: &str = "twilightforest:goblin_knight_upper";
const BREAK_SOUND: &str = "random.break";

mod tuning
{
    pub const SHIELD_HITS: i32 = 3;
    // takeHitOnShield only dents the shield for a hit over this
    pub const SHIELD_DENT: i32 = 10;
    pub const SHIELD_FROM: f32 = 150.0;
    pub const SHIELD_TO: f32 = 230.0;
    pub const FRONT: f32 = 60.0;
    pub const YAW_OFFSET: f32 = 90.0;
    pub const FULL_TURN: f32 = 360.0;
    // getTotalArmorValue, 1.7 armor takes four percent a point
    pub const ARMOR_SCALE: i32 = 25;
    pub const UPPER_ARMOR: i32 = 20;
    pub const LOWER_ARMOR: i32 = 17;
    pub const SPEAR_TICKS: i64 = 60;
    // heavySpearTimer counts down from 60 and lands at 25, the mount holds still between 50 and 0
    pub const SPEAR_LANDS: i64 = 35;
    pub const SPEAR_HOLDS: i64 = 10;
    pub const SPEAR_REACH: f64 = 2.5;
    pub const SPEAR_CHANCE: i32 = 40;
    pub const SPEAR_AHEAD: f64 = 1.25;
    pub const SPEAR_BELOW: f64 = 0.75;
    pub const SPEAR_RADIUS: f64 = 1.5;
    pub const SPEAR_SMOKE: i32 = 50;
    pub const SMOKE_DRIFT: f64 = 0.25;
    pub const ATTACK: f64 = 4.0;
    pub const KNOCK: f64 = 0.4;
    pub const REMOUNT_REACH: f64 = 1.5;
    pub const REMOUNT_GRACE: i32 = 20;
}

mod pose
{
    // ModelTFGoblinKnightUpper.setRotationAngles
    pub const ARM_SWING: f32 = 2.0 * 0.5;
    pub const SHIELD_SWING: f32 = 0.2;
    pub const RIDING_ARMS: f32 = std::f32::consts::PI / 5.0;
    pub const HOLDING: f32 = std::f32::consts::PI / 10.0;
    pub const SPEAR_HOLD: f32 = 2.0734512;
    pub const SWAY: f32 = 0.05;
    pub const SWAY_ROLL: f32 = 0.09;
    pub const SWAY_PITCH: f32 = 0.067;
    pub const PIXEL: f32 = 1.0 / 16.0;
}

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct Gear
{
    m_ready: bool,
    m_armor: bool,
    m_shield: bool,
    // world time the heavy spear attack began, 0 while none runs
    m_spear: i64,
}

static SHIELD_HITS: EntityState<i32> = EntityState::new("shield_hits");
static LOOSE: EntityState<i32> = EntityState::new("loose");

fn setGear(entity: Entity, gear: Gear)
{
    if let Err(error) = entity.setSynced(&gear)
    {
        warn(&format!("goblin knight gear: {}", error.0));
    }
}

fn gear(entity: Entity) -> Gear
{
    let gear: Gear = entity.synced();
    if gear.m_ready
    {
        return gear;
    }

    let fresh: Gear = Gear { m_ready: true, m_armor: true, m_shield: true, m_spear: 0 };

    if entity.world.isAuthority()
    {
        setGear(entity, fresh);
    }

    fresh
}

fn breakSound(entity: Entity, info: &EntityInfo)
{
    let pitch: f32 = 0.8 + crate::math::fraction(entity.world) * 0.4;
    entity.world.playSound(info.pos, BREAK_SOUND, 1.0, pitch);
}

// EntityLivingBase.attackEntityFrom's facing test, the attacker's angle against the knight's body
fn facing(info: &EntityInfo, attacker: &EntityInfo) -> f32
{
    let (dx, dz): (f64, f64) = (info.pos[0] - attacker.pos[0], info.pos[2] - attacker.pos[2]);
    let angle: f32 = dz.atan2(dx).to_degrees() as f32 - tuning::YAW_OFFSET;
    ((info.yaw - angle) % tuning::FULL_TURN).abs()
}

fn front(difference: f32) -> bool { difference > tuning::FULL_TURN - tuning::FRONT || difference < tuning::FRONT }

fn behind(difference: f32) -> bool { difference > tuning::SHIELD_FROM && difference < tuning::SHIELD_TO }

fn armored(amount: i32, armor: i32) -> i32 { amount * (tuning::ARMOR_SCALE - armor) / tuning::ARMOR_SCALE }

// damageShield, the third dent breaks it
fn dent(upper: Entity, info: &EntityInfo)
{
    let hits: i32 = SHIELD_HITS.get(upper) + 1;
    super::store(&SHIELD_HITS, upper, &hits);

    breakSound(upper, info);

    if hits >= tuning::SHIELD_HITS
    {
        setGear(upper, Gear { m_shield: false, ..gear(upper) });
    }
}

// takeHitOnShield, the blow is turned and the pair is pushed back from the attacker
fn shieldHit(upper: Entity, info: &EntityInfo, attacker: &EntityInfo, amount: i32)
{
    if amount > tuning::SHIELD_DENT
    {
        dent(upper, info);
    }
    else
    {
        breakSound(upper, info);
    }
    let pushed: Entity = upper.riding().map_or(upper, |mount| Entity { world: upper.world, id: mount });
    let (dx, dz): (f64, f64) = (info.pos[0] - attacker.pos[0], info.pos[2] - attacker.pos[2]);
    let flat: f64 = flatLength(dx, dz).max(1.0e-3);
    let _ = pushed.setMotion([dx / flat * tuning::KNOCK, tuning::KNOCK, dz / flat * tuning::KNOCK]);
}

fn breakArmor(entity: Entity, info: &EntityInfo, gear: Gear)
{
    breakSound(entity, info);
    setGear(entity, Gear { m_armor: false, ..gear });
}

fn attackerOf(entity: Entity, source: Option<i32>) -> Option<EntityInfo>
{
    Entity { world: entity.world, id: source? }.info()
}

// EntityTFGoblinKnightUpper.attackEntityFrom
pub(crate) fn upperHurt(entity: Entity, source: Option<i32>, amount: i32) -> Option<i32>
{
    let info: EntityInfo = entity.info()?;

    // a rider stuck in its mount's block does not suffocate
    if entity.riding().is_some() && crate::combat::hurtBy(crate::combat::kind::IN_WALL)
    {
        return None;
    }

    let Some(attacker) = attackerOf(entity, source)
    else
    {
        return Some(amount);
    };

    let mut current: Gear = gear(entity);
    let difference: f32 = facing(&info, &attacker);
    if current.m_shield && behind(difference)
    {
        shieldHit(entity, &info, &attacker, amount);
        return None;
    }

    if current.m_shield && entity.world.random(2) == 0
    {
        dent(entity, &info);
        current = gear(entity);
    }

    if current.m_armor && front(difference)
    {
        breakArmor(entity, &info, current);
        current.m_armor = false;
    }

    Some(
        if current.m_armor
        {
            armored(amount, tuning::UPPER_ARMOR)
        }
        else
        {
            amount
        },
    )
}

// EntityTFGoblinKnightLower.attackEntityFrom, the rider's shield covers the mount's back too
pub(crate) fn lowerHurt(entity: Entity, source: Option<i32>, amount: i32) -> Option<i32>
{
    let info: EntityInfo = entity.info()?;

    let Some(attacker) = attackerOf(entity, source)
    else
    {
        return Some(amount);
    };

    let difference: f32 = facing(&info, &attacker);
    let rider: Option<Entity> = entity.rider().map(|id| Entity { world: entity.world, id });
    let shielded: Option<(Entity, EntityInfo)> = rider
        .filter(|rider|
        {
            rider.info().is_some_and(|other| other.kind.as_ref().is_some_and(|kind| kind.to_string() == UPPER))
        })
        .filter(|rider| gear(*rider).m_shield)
        .and_then(|rider| Some((rider, rider.info()?)));

    if let Some((upper, upperInfo)) = shielded
        && behind(difference)
    {
        shieldHit(upper, &upperInfo, &attacker, amount);
        return None;
    }

    let current: Gear = gear(entity);
    if current.m_armor && front(difference)
    {
        breakArmor(entity, &info, current);
        return Some(amount);
    }

    Some(
        if current.m_armor
        {
            armored(amount, tuning::LOWER_ARMOR)
        }
        else
        {
            amount
        },
    )
}

fn shareTarget(entity: Entity, other: Option<i32>)
{
    if target(entity).is_some()
    {
        return;
    }

    let goal: Option<i32> = other.and_then(|id| target(Entity { world: entity.world, id })).map(|goal| goal.id);
    if let Some(goal) = goal
    {
        entity.setField(EntityField::TargetId, f64::from(goal));
    }
}

fn spearTime(gear: Gear, now: i64) -> Option<i64>
{
    (gear.m_spear > 0).then_some(now - gear.m_spear).filter(|elapsed| *elapsed < tuning::SPEAR_TICKS)
}

// landHeavySpearAttack, smoke and a blow on everything in front but its mount
fn landSpear(entity: Entity, info: &EntityInfo)
{
    let world: World = entity.world;
    let yaw: f32 = info.yaw.to_radians();
    let point: [f64; 3] = [
        info.pos[0] - f64::from(yaw.sin()) * tuning::SPEAR_AHEAD,
        info.pos[1] - tuning::SPEAR_BELOW,
        info.pos[2] + f64::from(yaw.cos()) * tuning::SPEAR_AHEAD,
    ];

    for _ in 0..tuning::SPEAR_SMOKE
    {
        let drift = || f64::from(crate::math::fraction(world) - crate::math::fraction(world)) * tuning::SMOKE_DRIFT;
        world.particle("largesmoke", point, [drift(), 0.0, drift()]);
    }

    let mount: Option<i32> = entity.riding();

    for other in world.entitiesNear(point, tuning::SPEAR_RADIUS)
    {
        if other.id != entity.id && Some(other.id) != mount && other.health.is_some()
        {
            crate::combat::hit(
                Entity { world, id: other.id },
                tuning::ATTACK as i32,
                Some(entity.id),
                crate::combat::kind::MOB,
            );
        }
    }
}

pub(crate) fn upperTick(entity: Entity, info: &EntityInfo)
{
    let world: World = entity.world;
    let mut current: Gear = gear(entity);

    shareTarget(entity, entity.riding());

    let now: i64 = world.time();
    match spearTime(current, now)
    {
        Some(tuning::SPEAR_LANDS) => landSpear(entity, info),
        Some(_) =>
        {}
        None if current.m_spear > 0 =>
        {
            current.m_spear = 0;
            setGear(entity, current);
            entity.setField(EntityField::AttackStrength, tuning::ATTACK);
        }

        None =>
        {
            let close: bool = target(entity).is_some_and(|goal| distance(goal.pos, info.pos) < tuning::SPEAR_REACH);
            if close && world.random(tuning::SPEAR_CHANCE) == 0
            {
                current.m_spear = now;
                setGear(entity, current);
                entity.setField(EntityField::AttackStrength, 0.0);
            }
        }
    }

    let loose: i32 = if entity.riding().is_some()
    {
        0
    }
    else
    {
        LOOSE.get(entity) + 1
    };
    super::store(&LOOSE, entity, &loose);
    // a pair reloaded from disk gets a moment to remount before the shield falls
    if loose > tuning::REMOUNT_GRACE && current.m_shield
    {
        breakSound(entity, info);
        setGear(entity, Gear { m_shield: false, ..current });
    }
}

// beta keeps no mount in saved chunks, so a reloaded pair finds each other again
fn remount(entity: Entity, info: &EntityInfo)
{
    let world: World = entity.world;
    let loose: Option<EntityInfo> = world.entitiesNear(info.pos, tuning::REMOUNT_REACH).into_iter().find(|other|
    {
        let upper: bool = other.kind.as_ref().is_some_and(|kind| kind.to_string() == UPPER);
        upper && Entity { world, id: other.id }.riding().is_none()
    });
    let Some(upper) = loose
    else
    {
        return;
    };

    if let Err(error) = (Entity { world, id: upper.id }).mount(Some(entity.id))
    {
        warn(&format!("goblin knight remount: {}", error.0));
    }
}

// EntityTFGoblinKnightLower.onSpawnWithEgg seats a fresh rider, EntityAITFRiderSpearAttack holds still for its blow
pub(crate) fn lowerTick(entity: Entity, info: &EntityInfo)
{
    let world: World = entity.world;
    let fresh: bool = !entity.synced::<Gear>().m_ready;
    // the first look also arms a fresh knight
    gear(entity);

    if fresh
    {
        match world.spawnEntity(UPPER, info.pos)
        {
            Ok(upper) =>
            {
                if let Err(error) = upper.mount(Some(entity.id))
                {
                    warn(&format!("goblin knight rider: {}", error.0));
                }
            }

            Err(error) => warn(&format!("goblin knight rider: {}", error.0)),
        }
    }
    else if entity.rider().is_none()
    {
        remount(entity, info);
    }
    shareTarget(entity, entity.rider());

    let holding: bool = entity.rider().is_some_and(|id|
    {
        let rider: Entity = Entity { world, id };
        spearTime(rider.synced(), world.time()).is_some_and(|elapsed| elapsed > tuning::SPEAR_HOLDS)
    });
    if holding
    {
        let _ = entity.setMotion([0.0, info.motion[1], 0.0]);
    }
}

pub(crate) fn upperModel() -> EntityModelSpec
{
    let mut model: EntityModelSpec = tf_goblin_knight_upper::model();
    // the loader always draws a child with its parent, the shield comes off the arm so it can be left out
    model.parts[tf_goblin_knight_upper::SHIELD as usize].parent = None;
    model
}

// getArmRotationDuringSwing, degrees by tick of the heavy attack
fn spearSwing(elapsed: f32) -> f32
{
    match elapsed
    {
        tick if tick <= 10.0 => tick,
        tick if tick <= 30.0 => 10.0,
        tick if tick <= 33.0 => (tick - 30.0) * -8.0 + 10.0,
        tick if tick <= 50.0 => -15.0,
        tick if tick <= 60.0 => (10.0 - (tick - 50.0)) * -1.5,
        _ => 0.0,
    }
}

// ModelTFGoblinKnightUpper.setRotationAngles and render
pub(crate) fn upperPose(input: &PoseInput) -> Pose
{
    let gear: Gear = b173_api::fromBytes(&input.synced).unwrap_or_default();
    let pivots: &[[f32; 3]] = &tf_goblin_knight_upper::PIVOTS;
    let head: [f32; 3] = [input.head_pitch / DEGREES_PER_RADIAN, input.head_yaw / DEGREES_PER_RADIAN, 0.0];
    let swing: f32 = input.walk * WALK_RATE;

    let constraint: f32 = if gear.m_shield
    {
        pose::SHIELD_SWING
    }
    else
    {
        input.walk_speed
    };
    let roll: f32 = math::cos(input.age * pose::SWAY_ROLL) * pose::SWAY + pose::SWAY;
    let sway: f32 = math::sin(input.age * pose::SWAY_PITCH) * pose::SWAY;

    let mut right: f32 = math::cos(swing + PI) * pose::ARM_SWING * input.walk_speed - pose::RIDING_ARMS;
    let mut left: f32 = math::cos(swing) * pose::ARM_SWING * constraint - pose::RIDING_ARMS;

    right = right * 0.5 - pose::HOLDING - pose::SPEAR_HOLD;

    let world: World = World { dimension: input.entity.world };
    if let Some(elapsed) = spearTime(gear, world.time())
    {
        right -= spearSwing(elapsed as f32 + input.partial) / DEGREES_PER_RADIAN;
    }

    right += sway;
    left -= sway;

    let rightArm: [f32; 3] = [right, 0.0, roll];
    let leftArm: [f32; 3] = [left, 0.0, roll];
    let legs: [f32; 3] = [0.0; 3];

    let mut body: Program = Program::new();

    for (part, rotation) in [
        (biped::HEAD, head),
        (biped::HEADWEAR, head),
        (biped::BODY, [0.0; 3]),
        (biped::RIGHT_ARM, rightArm),
        (biped::LEFT_ARM, leftArm),
        (biped::RIGHT_LEG, legs),
        (biped::LEFT_LEG, legs),
    ]
    {
        body.part(part, rotation, pivots[part as usize]);
    }

    if gear.m_shield
    {
        let [x, y, z]: [f32; 3] = pivots[biped::LEFT_ARM as usize];
        body.push()
            .translate(x * pose::PIXEL, y * pose::PIXEL, z * pose::PIXEL)
            .rotate(leftArm[2] * DEGREES_PER_RADIAN, 0.0, 0.0, 1.0)
            .rotate(leftArm[0] * DEGREES_PER_RADIAN, 1.0, 0.0, 0.0);
        body.part(
            tf_goblin_knight_upper::SHIELD,
            [TAU - left, 0.0, 0.0],
            pivots[tf_goblin_knight_upper::SHIELD as usize],
        );
        body.pop();
    }

    if gear.m_armor
    {
        body.rest(tf_goblin_knight_upper::BREASTPLATE);
    }

    Pose { pre: Vec::new(), model: body.take() }
}

// ModelTFGoblinKnightLower, a plain biped with its tunic while it keeps its armor
pub(crate) fn lowerPose(input: &PoseInput) -> Pose
{
    let gear: Gear = b173_api::fromBytes(&input.synced).unwrap_or_default();
    let mut body: Program = Program::new();
    bipedPose(
        &mut body,
        &tf_goblin_knight_lower::PIVOTS[..biped::PARTS as usize],
        &tf_goblin_knight_lower::ROTATIONS[..biped::PARTS as usize],
        input,
    );

    if gear.m_armor
    {
        body.rest(tf_goblin_knight_lower::TUNIC);
    }

    Pose { pre: Vec::new(), model: body.take() }
}
