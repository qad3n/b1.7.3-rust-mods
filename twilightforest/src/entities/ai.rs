use super::shots;
use b173_api::math::{distance, flatLength, floor, yawTowards};
use b173_api::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

// the parts of the twilight mobs' EntityAI tasks the beta templates do not have
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Ai
{
    Plain,
    // EntityAIArrowAttack and EntityAITFMagicAttack, a shot every so many ticks at a target in sight
    Shooter
    {
        shot: &'static str,
        period: i32,
        sound: &'static str,
    },

    // EntityFlying wraiths and the hovering death tome, gliding at the target or drifting
    Flyer
    {
        speed: f64,
    },
    // EntityAIPanic and EntityAIAvoidEntity of the small animals, away from a player close by
    Skittish,
    // EntityTFBird, a hop into the air and a slow flutter down
    Bird,
    // EntityTFIceExploder, it bursts once a player stands close
    Exploder,
    // EntityAITFChargeAttack, a run at the target from a distance
    Charger,
    // EntityTFRedcap and EntityTFRedcapSapper, see redcap.rs
    Redcap
    {
        sapper: bool,
    },
    // EntityTFKobold's pack habits, see flock.rs
    Kobold,
    // EntityAILeapAtTarget alone
    Leaper,
    // EntityTFBlockGoblin's swinging chain, see goblin.rs
    Goblin,
    // the goblin knight pair, see goblin_knight.rs
    KnightUpper,
    KnightLower,
    // EntityTFYeti.attackEntityAsMob picks its target up and EntityAITFThrowRider flings it
    Grabber,
    // EntityTFTroll, its death ripens the trollber around it
    Troll,
    // EntityTFTowerTermite calls out the termites in infested wood when hurt and burrows into towerwood when idle
    Termite,
    // EntityTFPinchBeetle, a charge, a pinch and a run off with the catch
    Pincher,
    // EntityAITFBreathAttack, see breath.rs
    Breather(&'static super::breath::Breath),
}

mod tuning
{
    // EntityAITFMagicAttack only shoots inside ten blocks
    pub const RANGE: f64 = 10.0;
    // EntityAIAvoidEntity and the birds keep this far from a player
    pub const SHY: f64 = 6.0;
    pub const FLEE: f64 = 0.3;
    pub const HOP: f64 = 0.45;
    pub const FLUTTER: f64 = 0.6;
    pub const EXPLODE_REACH: f64 = 2.0;
    pub const EXPLODE_POWER: f32 = 1.0;
    pub const CHARGE_FROM: f64 = 4.0;
    pub const CHARGE_TO: f64 = 16.0;
    pub const CHARGE_SPEED: f64 = 0.45;
    pub const CHARGE_COOLDOWN: i32 = 60;
    pub const DRIFT: f64 = 0.5;
    pub const HOVER_ABOVE: f64 = 1.0;
    // a mob's eyes sit at eight tenths of its height, a thrower lets go a little under them
    pub const EYES: f64 = 0.85;
    pub const LET_GO: f64 = 0.1;
    // EntityAITFMagicAttack arcs the shot up a fifth of the flat distance and aims below the target's eyes
    pub const ARC: f64 = 0.2;
    pub const AIM_DROP: f64 = 1.1;
    pub const SHOT_SPEED: f64 = 0.6;
    pub const SHOT_SPREAD: f64 = 6.0;
    // the yeti grabs at melee reach, throws with a one in five chance a tick and rests forty ticks after
    pub const GRAB_REACH: f64 = 2.0;
    pub const THROW_ODDS: i32 = 5;
    pub const THROW_SPEED: f64 = 2.0;
    pub const THROW_LIFT: f64 = 0.9;
    pub const THROW_REST: i32 = 40;
    pub const KIDNAP_RANGE: i32 = 5;
    pub const KIDNAP_SPEED: f32 = 2.0;
    pub const RIPEN_RANGE: i32 = 12;
    pub const SUMMON_DELAY: i32 = 20;
    pub const SUMMON_RISE: i32 = 5;
    pub const SUMMON_REACH: i32 = 10;
    pub const IDLE: f64 = 0.01;
}

thread_local!
{
    // per mob countdowns, by dimension and id
    static TIMERS: RefCell<HashMap<(i32, i32), i32>> = RefCell::new(HashMap::new());
}

fn timer(entity: Entity) -> i32
{
    TIMERS.with(|timers| timers.borrow().get(&(entity.world.dimension, entity.id)).copied().unwrap_or(0))
}

fn setTimer(entity: Entity, value: i32)
{
    TIMERS.with(|timers| timers.borrow_mut().insert((entity.world.dimension, entity.id), value));
}

pub(crate) fn target(entity: Entity) -> Option<EntityInfo>
{
    let id: i32 = entity.field(EntityField::TargetId) as i32;
    (id >= 0).then(|| Entity { world: entity.world, id }.info()).flatten().filter(|info| info.alive)
}

pub(crate) fn eyes(entity: Entity, info: &EntityInfo) -> [f64; 3]
{
    let height: f64 = entity.field(EntityField::Height);
    [info.pos[0], info.pos[1] + height * tuning::EYES, info.pos[2]]
}

pub(crate) fn face(entity: Entity, from: [f64; 3], to: [f64; 3])
{
    entity.setField(EntityField::Yaw, f64::from(yawTowards(from, to)));
}

// EntityAITFMagicAttack.doRangedAttack
fn shoot(entity: Entity, info: &EntityInfo, shot: &str, period: i32, sound: &str)
{
    let Some(goal) = target(entity)
    else
    {
        return;
    };

    let from: [f64; 3] = eyes(entity, info);

    face(entity, info.pos, goal.pos);

    let left: i32 = (timer(entity) - 1).max(0);
    let seen: bool =
        entity.world.rayTrace(from, eyes(Entity { world: entity.world, id: goal.id }, &goal), false).is_none();

    if left > 0 || !seen || distance(info.pos, goal.pos) > tuning::RANGE
    {
        setTimer(entity, left);
        return;
    }

    let start: [f64; 3] = [from[0], from[1] - tuning::LET_GO, from[2]];
    let flat: f64 = flatLength(goal.pos[0] - info.pos[0], goal.pos[2] - info.pos[2]);
    let aim: [f64; 3] = [
        goal.pos[0],
        eyes(Entity { world: entity.world, id: goal.id }, &goal)[1] - tuning::AIM_DROP + flat * tuning::ARC,
        goal.pos[2],
    ];

    if shots::fire(entity.world, shot, start, aim, tuning::SHOT_SPEED, tuning::SHOT_SPREAD, Some(entity.id)).is_some()
    {
        let pitch: f32 = 1.0 / (crate::ids::fraction(entity.world) * 0.4 + 0.8);
        entity.world.playSound(info.pos, sound, 1.0, pitch);
    }

    setTimer(entity, period);
}

// EntityTFWraith and EntityTFDeathTome glide at what they hunt and hold themselves up otherwise
fn fly(entity: Entity, info: &EntityInfo, speed: f64)
{
    let motion: [f64; 3] = match target(entity)
    {
        Some(goal) =>
        {
            let aim: [f64; 3] = [goal.pos[0], goal.pos[1] + tuning::HOVER_ABOVE, goal.pos[2]];
            let length: f64 = distance(info.pos, aim).max(1.0e-3);

            face(entity, info.pos, goal.pos);
            [0, 1, 2].map(|axis| (aim[axis] - info.pos[axis]) / length * speed)
        }
        // with nothing to hunt it drifts and never sinks
        None => [info.motion[0], info.motion[1].max(0.0) * tuning::DRIFT, info.motion[2]],
    };
    let _ = entity.setMotion(motion);

    entity.setField(EntityField::FallDistance, 0.0);
}

// the shy animals run straight away from a close player
fn flee(entity: Entity, info: &EntityInfo, hop: bool)
{
    let Some(player) = super::closestPlayer(entity.world, info, tuning::SHY)
    else
    {
        if hop && !entity.flag(EntityField::OnGround) && info.motion[1] < 0.0
        {
            let _ = entity.setMotion([info.motion[0], info.motion[1] * tuning::FLUTTER, info.motion[2]]);
        }

        return;
    };
    let (dx, dz): (f64, f64) = (info.pos[0] - player.pos[0], info.pos[2] - player.pos[2]);
    let length: f64 = flatLength(dx, dz).max(1.0e-3);

    let up: f64 = if hop && entity.flag(EntityField::OnGround)
    {
        tuning::HOP
    }
    else
    {
        info.motion[1]
    };
    let _ = entity.setMotion([dx / length * tuning::FLEE, up, dz / length * tuning::FLEE]);

    face(entity, player.pos, info.pos);
}

// EntityTFIceExploder bursts into a small ice blast beside its target
fn burst(entity: Entity, info: &EntityInfo)
{
    if target(entity).is_none_or(|goal| distance(info.pos, goal.pos) >= tuning::EXPLODE_REACH)
    {
        return;
    }

    if let Err(error) = entity.world.explode(info.pos, tuning::EXPLODE_POWER)
    {
        warn(&format!("ice core burst: {}", error.0));
    }

    entity.kill();
}

// EntityAITFChargeAttack, a head down run once the target is a few blocks off
fn charge(entity: Entity, info: &EntityInfo)
{
    let left: i32 = (timer(entity) - 1).max(0);
    setTimer(entity, left);

    let Some(goal) = target(entity)
    else
    {
        return;
    };
    let gap: f64 = distance(info.pos, goal.pos);
    if left > 0 || !(tuning::CHARGE_FROM..tuning::CHARGE_TO).contains(&gap) || !entity.flag(EntityField::OnGround)
    {
        return;
    }

    let (dx, dz): (f64, f64) = (goal.pos[0] - info.pos[0], goal.pos[2] - info.pos[2]);
    let _ = entity.setMotion([
        dx / gap * tuning::CHARGE_SPEED * 2.0,
        info.motion[1],
        dz / gap * tuning::CHARGE_SPEED * 2.0,
    ]);
    setTimer(entity, tuning::CHARGE_COOLDOWN);
}

// EntityTFYeti and EntityTFPinchBeetle.attackEntityAsMob, a target in reach and riding nothing is picked up
fn seize(entity: Entity, info: &EntityInfo)
{
    let Some(goal) = target(entity)
    else
    {
        return;
    };
    let victim: Entity = Entity { world: entity.world, id: goal.id };
    if distance(goal.pos, info.pos) > tuning::GRAB_REACH || victim.riding().is_some()
    {
        return;
    }

    if let Err(error) = victim.mount(Some(entity.id))
    {
        warn(&format!("grab: {}", error.0));
    }
}

fn grab(entity: Entity, info: &EntityInfo)
{
    let world: World = entity.world;

    if let Some(rider) = entity.rider()
    {
        if world.random(tuning::THROW_ODDS) != 0
        {
            return;
        }

        let rider: Entity = Entity { world, id: rider };
        let _ = rider.mount(None);
        let aim: [f64; 3] = crate::math::look(info.yaw, info.pitch);
        let _ = rider.setMotion([aim[0] * tuning::THROW_SPEED, tuning::THROW_LIFT, aim[2] * tuning::THROW_SPEED]);

        setTimer(entity, tuning::THROW_REST);
        return;
    }

    let left: i32 = (timer(entity) - 1).max(0);
    setTimer(entity, left);

    if left == 0
    {
        seize(entity, info);
    }
}

// EntityTFPinchBeetle charges, pinches a player up and EntityAITFKidnapRider runs off with them
fn pinch(entity: Entity, info: &EntityInfo)
{
    let world: World = entity.world;

    if entity.rider().is_some()
    {
        if world.random(tuning::THROW_ODDS) == 0
        {
            let away = || world.random(tuning::KIDNAP_RANGE * 2 + 1) - tuning::KIDNAP_RANGE;
            let spot: BlockPos =
                BlockPos::new(floor(info.pos[0]) + away(), floor(info.pos[1]), floor(info.pos[2]) + away());
            entity.walkTo(spot, tuning::KIDNAP_SPEED);
        }

        return;
    }

    charge(entity, info);
    seize(entity, info);
}

// EntityTFTroll.onDeathUpdate, half the unripe trollber in range ripens as it dies
pub(crate) fn ripen(entity: Entity, info: &EntityInfo)
{
    let world: World = entity.world;
    if !world.isAuthority()
    {
        return;
    }

    let (unripe, ripe): (i32, i32) =
        (crate::ids::blockId(crate::ids::block::UNRIPE_TROLLBER), crate::ids::blockId(crate::ids::block::TROLLBER));
    let [x, y, z]: [i32; 3] = info.pos.map(floor);
    let range: i32 = tuning::RIPEN_RANGE;
    for dx in -range..range
    {
        for dy in -range..range
        {
            for dz in -range..range
            {
                let pos: BlockPos = BlockPos::new(x + dx, y + dy, z + dz);
                if world.block(pos) != unripe || world.random(2) != 0
                {
                    continue;
                }

                if let Err(error) = world.setBlock(pos, ripe, 0)
                {
                    warn(&format!("trollber ripen: {}", error.0));
                    continue;
                }

                world.particle(
                    "flame",
                    [f64::from(pos.x) + 0.5, f64::from(pos.y) + 0.5, f64::from(pos.z) + 0.5],
                    [0.0; 3],
                );
            }
        }
    }
}

// the order EntityTFTowerTermite.tryToSummonAllies walks offsets in, 0 then 1, -1, 2, -2 out to the reach
fn outward(reach: i32) -> impl Iterator<Item = i32>
{
    (0..=reach * 2).map(|step|
    {
        if step % 2 == 1
        {
            step / 2 + 1
        }
        else
        {
            -(step / 2)
        }
    })
}

pub(crate) fn termiteHurt(entity: Entity, source: Option<i32>)
{
    let called: bool = source.is_some() || crate::combat::hurtBy(crate::combat::kind::MAGIC);
    if called && timer(entity) <= 0
    {
        setTimer(entity, tuning::SUMMON_DELAY);
    }
}

fn summon(entity: Entity, info: &EntityInfo)
{
    let world: World = entity.world;
    let infested: i32 = crate::ids::blockId(crate::ids::block::INFESTED_TOWERWOOD);
    let [x, y, z]: [i32; 3] = info.pos.map(floor);

    for dy in outward(tuning::SUMMON_RISE)
    {
        for dx in outward(tuning::SUMMON_REACH)
        {
            for dz in outward(tuning::SUMMON_REACH)
            {
                let pos: BlockPos = BlockPos::new(x + dx, y + dy, z + dz);
                if world.block(pos) != infested
                {
                    continue;
                }

                let _ = world.breakEffect(pos, infested, 0);

                if let Err(error) = world.setBlock(pos, 0, 0)
                {
                    warn(&format!("termite ally: {}", error.0));
                    continue;
                }

                let center: [f64; 3] = [f64::from(pos.x) + 0.5, f64::from(pos.y), f64::from(pos.z) + 0.5];
                super::spawnAt(world, "twilightforest:tower_termite", center, 0.0);

                if world.random(2) == 0
                {
                    return;
                }
            }
        }
    }
}

// tryToBurrow, an idle termite crawls into plain towerwood beside it and leaves it infested
fn burrow(entity: Entity, info: &EntityInfo)
{
    let left: i32 = timer(entity);
    if left > 0
    {
        setTimer(entity, left - 1);

        if left == 1
        {
            summon(entity, info);
        }
    }

    let idle: bool = info.motion[0].abs() < tuning::IDLE && info.motion[2].abs() < tuning::IDLE;
    if target(entity).is_some() || !idle
    {
        return;
    }

    let world: World = entity.world;
    let [dx, dy, dz]: [i32; 3] = crate::ids::NEIGHBORS[world.random(6) as usize];
    let pos: BlockPos = BlockPos::new(floor(info.pos[0]) + dx, floor(info.pos[1] + 0.5) + dy, floor(info.pos[2]) + dz);

    if world.block(pos) != crate::ids::blockId(crate::ids::block::TOWERWOOD)
    {
        return;
    }

    if let Err(error) = world.setBlock(pos, crate::ids::blockId(crate::ids::block::INFESTED_TOWERWOOD), 0)
    {
        warn(&format!("termite burrow: {}", error.0));
        return;
    }

    world.particle("explode", info.pos, [0.0; 3]);
    entity.kill();
}

pub(crate) fn tick(ai: Ai, entity: Entity, info: &EntityInfo)
{
    if ai == Ai::Kobold
    {
        super::flock::kobold(entity, info);
        return;
    }

    // the breath's puffs are each client's own
    if let Ai::Breather(breath) = ai
        && !entity.world.isAuthority()
    {
        super::breath::puff(entity, info, breath);
        return;
    }

    if !entity.world.isAuthority()
    {
        return;
    }

    match ai
    {
        Ai::Plain =>
        {}
        Ai::Shooter { shot, period, sound } => shoot(entity, info, shot, period, sound),
        Ai::Flyer { speed } => fly(entity, info, speed),
        Ai::Skittish => flee(entity, info, false),
        Ai::Bird => flee(entity, info, true),
        Ai::Exploder => burst(entity, info),
        Ai::Charger => charge(entity, info),
        Ai::Redcap { sapper } => super::redcap::tick(entity, info, sapper),
        Ai::Kobold =>
        {}
        Ai::Leaper => super::flock::leaper(entity, info),
        Ai::Goblin => super::goblin::tick(entity, info),
        Ai::KnightUpper => super::goblin_knight::upperTick(entity, info),
        Ai::KnightLower => super::goblin_knight::lowerTick(entity, info),
        Ai::Grabber => grab(entity, info),
        Ai::Pincher => pinch(entity, info),
        Ai::Troll =>
        {}
        Ai::Termite => burrow(entity, info),
        Ai::Breather(breath) => super::breath::tick(entity, info, breath),
    }
}

pub(crate) fn forget(entity: Entity)
{
    super::redcap::forget(entity);
    super::goblin::forget(entity);
    super::goblin_knight::forget(entity);
    super::breath::forget(entity);
    TIMERS.with(|timers| timers.borrow_mut().remove(&(entity.world.dimension, entity.id)));
}
