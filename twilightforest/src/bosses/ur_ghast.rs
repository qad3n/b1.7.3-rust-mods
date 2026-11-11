use super::{killer, showBar};
use crate::entities::shots::{self, UR_FIREBALL};
use crate::math::look;
use crate::models::{Program, cuboid, part};
use crate::names;
use crate::progress::{self, Gate};
use crate::treasure::{self, Table};
use crate::world::generate::GenWorld;
use b173_api::math::{blockOf, distance, flatLength, floor, length, sub};
use b173_api::prelude::*;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;

// EntityTFUrGhast, the dark tower's boss: it circles the tower's ghast traps, spits triple fireballs, and every 48
// damage throws a crying tantrum that calls mini ghasts from the traps and hurts whoever stands under the open sky
pub(crate) const NAME: &str = "ur_ghast";
const LABEL: &str = "Ur-Ghast";

const TENTACLES: usize = 9;
const JOINTS: usize = 3;

mod boss
{
    pub const HEALTH: i32 = 250;
    pub const AGGRO_RANGE: f64 = 128.0;
    pub const STARE_RANGE: f64 = 32.0;
    pub const WANDER: f64 = 32.0;
    pub const HOVER: i32 = 20;
    pub const PHASE_DAMAGE: i32 = 48;
    pub const FIRST_PHASE_DAMAGE: i32 = 45;
    pub const TANTRUM_SHIELD: i32 = 4;
    pub const SCAN_XZ: i32 = 48;
    pub const SCAN_Y: i32 = 32;
    pub const COURSE_NEAR: f64 = 100.0;
    pub const WAYPOINT_NEAR: f64 = 1.0;
    pub const WAYPOINT_FAR: f64 = 3600.0;
    pub const WAYPOINT_TRIES: i32 = 50;
    pub const SPEED: f64 = 0.05;
    pub const DRAG: f64 = 0.91;
    pub const SPIT_CHARGE: i32 = 10;
    pub const SPIT: i32 = 20;
    pub const SPIT_REST: i32 = -40;
    pub const AGGRO_TICKS: i32 = 20;
    pub const SHOT_REACH: f64 = 8.5;
    pub const SHOT_SPREAD: f32 = 8.0;
    pub const SHOT_SPEED: f64 = 1.0;
    pub const TRAP_HOLD: i32 = 10;
    pub const MINIONS: usize = 2;
    pub const MINION_TRIES: i32 = 24;
    pub const MINION_MAX: i32 = 6;
    pub const MINION_XZ: f64 = 4.0;
    pub const MINION_Y: f64 = 8.0;
    pub const TRAP_GHASTS: usize = 4;
    pub const RAIN_DAMAGE: i32 = 3;
    pub const RAIN_PERIOD: i32 = 10;
    pub const RAIN_DEPTH: f64 = 16.0;
    pub const CRY_MIN: i32 = 20;
    pub const CRY_SPREAD: i32 = 30;
    pub const PATH_REACH: i32 = 20;
    // the renderer draws the tower boss model 24 times over, swelling over the 20 ticks before a shot
    pub const SCALE: f32 = 24.0;
    pub const SWELL_TICKS: f32 = 20.0;
    pub const SWELL_POWER: i32 = 5;
    pub const SWELL: f32 = 2.0;
    pub const TWIST: f32 = 0.4;
    pub const HEAL: f64 = 2.0;
    pub const FLOCK_REACH: f64 = 16.0;
}

// ModelTFTowerBoss.makeTentacle: where each tentacle hangs from the body and how far it leans out
const TENTACLE_ROOTS: [([f32; 3], f32); TENTACLES] = [
    ([4.5, 7.0, 4.5], 0.0),
    ([-4.5, 7.0, 4.5], 0.0),
    ([0.0, 7.0, 0.0], 0.0),
    ([5.5, 7.0, -4.5], 0.0),
    ([-5.5, 7.0, -4.5], 0.0),
    ([-7.5, 3.5, -1.0], std::f32::consts::FRAC_PI_4),
    ([-7.5, -1.5, 3.5], std::f32::consts::FRAC_PI_3),
    ([7.5, 3.5, -1.0], -std::f32::consts::FRAC_PI_4),
    ([7.5, -1.5, 3.5], -std::f32::consts::FRAC_PI_3),
];

// ModelTFTowerBoss, a ghast body with nine three jointed tentacles
pub(crate) fn model() -> EntityModelSpec
{
    let mut parts: Vec<ModelPart> = vec![part(
        "body",
        None,
        [0, 0],
        false,
        [0.0, 8.0, 0.0],
        [0.0; 3],
        vec![cuboid([-8.0, -8.0, -8.0], [16, 16, 16], 0.0, None)],
        None,
    )];

    for (number, (pivot, lean)) in TENTACLE_ROOTS.iter().enumerate()
    {
        let tentacle: String = format!("tentacle{number}");
        let uv: [i32; 2] = [(number % 3) as i32, 0];
        parts.push(part(
            &tentacle,
            Some("body"),
            uv,
            false,
            *pivot,
            [0.0, 0.0, *lean],
            vec![cuboid([-1.5, 0.0, -1.5], [3, 5, 3], 0.0, None)],
            None,
        ));

        let mut parent: String = tentacle;

        for joint in 0..JOINTS
        {
            let name: String = format!("tentacle{number}_{joint}");
            let uv: [i32; 2] = [(number % 4) as i32, joint as i32 * 5 - 1];
            parts.push(part(
                &name,
                Some(&parent),
                uv,
                false,
                [0.0, 4.0, 0.0],
                [0.0; 3],
                vec![cuboid([-1.5, -0.5, -1.5], [3, 4, 3], 0.0, None)],
                None,
            ));
            parent = name;
        }
    }

    EntityModelSpec { texture_size: [64, 32], parts, ..EntityModelSpec::default() }
}

fn tentaclePart(number: usize) -> u16 { (1 + number * (JOINTS + 1)) as u16 }

// ModelTFTowerBoss.setRotationAngles and render for a model whose parts start at first, the tentacles wave with age
pub(crate) fn drawBody(body: &mut Program, first: u16, age: f32, look: [f32; 2])
{
    body.part(first, [look[0], look[1], 0.0], [0.0, 8.0, 0.0]);

    for (number, (pivot, lean)) in TENTACLE_ROOTS.iter().enumerate()
    {
        let time: f32 = (age + number as f32 * 9.0) / 2.0;
        let base: u16 = first + tentaclePart(number);
        body.part(base, [0.0, boss::TWIST * (time * 0.3).sin(), *lean], *pivot);

        let swing: [f32; JOINTS] =
            [0.2 + (time * 0.3335).cos() * 0.15, 0.1 + (time * 0.4445).cos() * 0.2, 0.1 + (time * 0.5555).cos() * 0.25];

        for (joint, angle) in swing.iter().enumerate()
        {
            body.part(base + 1 + joint as u16, [*angle, 0.0, 0.0], [0.0, 4.0, 0.0]);
        }
    }
}

#[derive(Default)]
struct Brain
{
    m_course: Option<[f64; 3]>,
    m_waypoint: [f64; 3],
    m_traps: Vec<[i32; 3]>,
    m_path: Vec<[i32; 3]>,
    m_pathIndex: usize,
    m_noTraps: bool,
    m_tantrum: bool,
    m_cry: i32,
    m_untilPhase: i32,
    m_trapped: i32,
    m_target: Option<i32>,
    m_aggressive: bool,
    m_aggro: i32,
    m_attack: i32,
    m_steer: i32,
    m_ticks: i32,
    m_started: bool,
}

// what the clients draw: the eyes, the swelling before a shot and the tantrum's tears
#[derive(Serialize, Deserialize, Default, Clone, Copy, PartialEq)]
struct Shown
{
    m_status: u8,
    m_attack: i32,
    m_tantrum: bool,
}

thread_local!
{
    static BRAINS: RefCell<HashMap<(i32, i32), Brain>> = RefCell::new(HashMap::new());
}

fn trapAt(world: World, pos: BlockPos) -> bool
{
    let block: i32 = world.block(pos);
    blockIs(block, names::block::GHAST_TRAP) || blockIs(block, "ghast_trap_active")
}

// EntityTFUrGhast.scanForTraps
fn scan(world: World, center: [i32; 3], traps: &mut Vec<[i32; 3]>)
{
    for dx in -boss::SCAN_XZ..=boss::SCAN_XZ
    {
        for dz in -boss::SCAN_XZ..=boss::SCAN_XZ
        {
            for dy in -boss::SCAN_Y..=boss::SCAN_Y
            {
                let at: [i32; 3] = [center[0] + dx, center[1] + dy, center[2] + dz];
                if !traps.contains(&at) && trapAt(world, BlockPos::new(at[0], at[1], at[2]))
                {
                    traps.push(at);
                }
            }
        }
    }
}

fn average(points: &[[i32; 3]]) -> [i32; 3]
{
    let count: i32 = points.len().max(1) as i32;
    let sum: [i32; 3] =
        points.iter().fold([0; 3], |sum, point| [sum[0] + point[0], sum[1] + point[1], sum[2] + point[2]]);
    sum.map(|axis| axis / count)
}

// EntityTFUrGhast.scanForTrapsTwice, around itself and then around the traps it found
fn scanTwice(world: World, info: &EntityInfo, brain: &mut Brain)
{
    let at: [i32; 3] = info.pos.map(floor);
    scan(world, at, &mut brain.m_traps);

    if !brain.m_traps.is_empty()
    {
        let center: [i32; 3] = average(&brain.m_traps);
        scan(world, center, &mut brain.m_traps);
    }
}

// EntityTFUrGhast.makeTravelPath, the traps in a random order
fn makePath(world: World, info: &EntityInfo, brain: &mut Brain)
{
    let at: [i32; 3] = info.pos.map(floor);
    let reach: i32 = boss::PATH_REACH;

    let mut points: Vec<[i32; 3]> = if brain.m_noTraps
    {
        vec![
            [at[0] + reach, at[1] - reach, at[2]],
            [at[0], at[1] - reach, at[2] - reach],
            [at[0] - reach, at[1] - reach, at[2]],
            [at[0], at[1] - reach, at[2] + reach],
        ]
    }
    else
    {
        brain.m_traps.clone()
    };
    brain.m_path.clear();

    while !points.is_empty()
    {
        brain.m_path.push(points.remove(world.random(points.len() as i32) as usize));
    }

    if brain.m_noTraps
    {
        brain.m_path.push([at[0], at[1] - reach, at[2]]);
    }
}

fn miniGhastsNear(world: World, at: [f64; 3], reach: f64) -> Vec<EntityInfo>
{
    world
        .entitiesNear(at, reach)
        .into_iter()
        .filter(|near| near.kind.as_ref().is_some_and(|kind| kind.to_string() == crate::entities::MINI_GHAST))
        .collect()
}

// EntityTFUrGhast.spawnMinionGhastsAt, lightning over a trap and a flock of mini ghasts around it
fn minionsAt(world: World, trap: [i32; 3])
{
    let [x, y, z] = trap.map(f64::from);
    if let Err(error) = world.lightning([x, y + 4.0, z])
    {
        warn(&format!("ur-ghast trap lightning: {}", error.0));
    }

    let mut spawned: i32 = 0;

    for _ in 0..boss::MINION_TRIES
    {
        let spread = || f64::from(crate::math::fraction(world) - crate::math::fraction(world)) * boss::MINION_XZ;
        let at: [f64; 3] = [x + spread(), y + f64::from(crate::math::fraction(world)) * boss::MINION_Y, z + spread()];
        if world.block(blockOf(at)) != 0
        {
            continue;
        }

        if crate::entities::spawnAt(world, crate::entities::MINI_GHAST, at, crate::math::fraction(world) * 360.0)
            .is_some()
        {
            spawned += 1;
        }

        if spawned >= boss::MINION_MAX
        {
            break;
        }
    }
}

// EntityTFUrGhast.spawnGhastsAtTraps, two random traps
fn minions(world: World, brain: &Brain)
{
    let mut traps: Vec<[i32; 3]> = brain.m_traps.clone();
    for _ in 0..boss::MINIONS.min(traps.len())
    {
        let trap: [i32; 3] = traps.remove(world.random(traps.len() as i32) as usize);
        minionsAt(world, trap);
    }
}

// EntityTFUrGhast.changeCourse, the next trap on the path, hovering 20 over it
fn changeCourse(world: World, info: &EntityInfo, brain: &mut Brain)
{
    if brain.m_path.is_empty()
    {
        makePath(world, info, brain);
    }

    if brain.m_path.is_empty()
    {
        return;
    }

    if brain.m_pathIndex >= brain.m_path.len()
    {
        brain.m_pathIndex = 0;

        let flocked: bool = brain
            .m_traps
            .iter()
            .any(|trap| miniGhastsNear(world, trap.map(f64::from), boss::FLOCK_REACH).len() >= boss::TRAP_GHASTS);

        if !flocked
        {
            minions(world, brain);
        }
    }

    let [x, y, z] = brain.m_path[brain.m_pathIndex];
    brain.m_course = Some([f64::from(x), f64::from(y + boss::HOVER), f64::from(z)]);
    brain.m_pathIndex += 1;
}

// EntityTFUrGhast.makeNewWaypoint, the wander spot closest to its course
fn makeWaypoint(world: World, info: &EntityInfo, brain: &mut Brain)
{
    let course: [f64; 3] = brain.m_course.unwrap_or(info.pos);
    let mut best: f64 = distance(info.pos, course).powi(2);

    for _ in 0..boss::WAYPOINT_TRIES
    {
        let wander = || f64::from(crate::math::fraction(world) * 2.0 - 1.0) * boss::WANDER;
        let spot: [f64; 3] = [
            info.pos[0] + wander(),
            course[1] + f64::from(crate::math::fraction(world)) * 8.0 - 4.0,
            info.pos[2] + wander(),
        ];
        let away: f64 = distance(spot, course).powi(2);
        if away < best
        {
            brain.m_waypoint = spot;
            best = away;
        }
    }
}

// EntityTowerGhast.shouldAttackPlayer, a player out under the sky that can see it
fn exposed(world: World, entity: Entity, player: &EntityInfo) -> bool
{
    let [x, y, z] = player.pos.map(floor);
    let underSky: bool = world.height(x, z) <= y;
    let eyes: [f64; 3] = [player.pos[0], player.pos[1] + 1.62, player.pos[2]];
    let height: f64 = entity.field(EntityField::Height);

    let seen: bool = entity.info().is_some_and(|info|
    {
        world.rayTrace(eyes, [info.pos[0], info.pos[1] + height / 2.0, info.pos[2]], false).is_none()
    });
    underSky && seen
}

// EntityTFUrGhast.spitFireball, three fireballs, two scattered
fn spit(entity: Entity, info: &EntityInfo, target: &EntityInfo)
{
    let world: World = entity.world;
    let height: f64 = entity.field(EntityField::Height);
    let aim: [f64; 3] = look(info.yaw, info.pitch);
    let from: [f64; 3] = [
        info.pos[0] + aim[0] * boss::SHOT_REACH,
        info.pos[1] + height / 2.0 + aim[1] * boss::SHOT_REACH,
        info.pos[2] + aim[2] * boss::SHOT_REACH,
    ];

    let goal: [f64; 3] = [target.pos[0], target.pos[1] + 0.9, target.pos[2]];

    world.playSound(info.pos, "mob.ghast.fireball", 10.0, 1.0);

    for shot in 0..3
    {
        let scatter = ||
        {
            if shot == 0
            {
                0.0
            }
            else
            {
                f64::from((crate::math::fraction(world) - crate::math::fraction(world)) * boss::SHOT_SPREAD)
            }
        };
        let toward: [f64; 3] = [goal[0] + scatter(), goal[1], goal[2] + scatter()];
        shots::fire(world, UR_FIREBALL, from, toward, boss::SHOT_SPEED, 0.0, Some(entity.id));
    }
}

// EntityTFUrGhast.doTantrumDamageEffects, rain of tears on everyone under it
fn tantrumRain(world: World, info: &EntityInfo, entity: Entity)
{
    let width: f64 = entity.field(EntityField::Width);

    for player in world.players()
    {
        let Some(found) = Entity { world, id: player.id }.info()
        else
        {
            continue;
        };

        let flat: f64 = flatLength(found.pos[0] - info.pos[0], found.pos[2] - info.pos[2]);
        let below: bool = found.pos[1] <= info.pos[1] && found.pos[1] >= info.pos[1] - boss::RAIN_DEPTH * 2.0;
        let [x, y, z] = found.pos.map(floor);
        if flat <= width / 2.0 && below && world.height(x, z) <= y
        {
            Entity { world, id: found.id }.damage(boss::RAIN_DAMAGE, None);
        }
    }

    for ghast in miniGhastsNear(world, info.pos, boss::RAIN_DEPTH + width / 2.0)
    {
        if ghast.pos[1] < info.pos[1]
        {
            let _ = Entity { world, id: ghast.id }.setMotion([ghast.motion[0], ghast.motion[1] + 1.0, ghast.motion[2]]);
        }
    }
}

// EntityTFUrGhast.updateEntityActionState
fn think(entity: Entity, info: &EntityInfo, brain: &mut Brain)
{
    let world: World = entity.world;

    brain.m_ticks += 1;

    if !brain.m_started
    {
        brain.m_untilPhase = boss::FIRST_PHASE_DAMAGE;
        brain.m_started = true;
    }

    let width: f64 = entity.field(EntityField::Width);
    for ghast in miniGhastsNear(world, info.pos, width / 2.0 + 1.0)
    {
        Entity { world, id: ghast.id }.kill();

        let health: f64 = entity.field(EntityField::Health);
        entity.setField(EntityField::Health, (health + boss::HEAL).min(f64::from(boss::HEALTH)));
    }

    if brain.m_traps.is_empty() && !brain.m_noTraps
    {
        scanTwice(world, info, brain);
        brain.m_noTraps = brain.m_traps.is_empty();
    }

    if brain.m_trapped > 0
    {
        brain.m_trapped -= 1;
        brain.m_target = None;
        return;
    }

    let mut target: Option<EntityInfo> =
        brain.m_target.and_then(|id| Entity { world, id }.info()).filter(|target| target.alive);

    match &target
    {
        None =>
        {
            target = crate::entities::closestPlayer(world, info, boss::AGGRO_RANGE)
                .filter(|player| distance(player.pos, info.pos) < boss::STARE_RANGE || exposed(world, entity, player));
        }

        Some(player) if !brain.m_aggressive =>
        {
            if exposed(world, entity, player)
            {
                if brain.m_aggro == 0
                {
                    world.playSound(info.pos, "mob.ghast.moan", 1.0, 1.0);
                }

                brain.m_aggro += 1;
                if brain.m_aggro > boss::AGGRO_TICKS
                {
                    brain.m_aggro = 0;
                    brain.m_aggressive = true;
                }
            }
            else
            {
                brain.m_aggro = 0;
            }
        }

        Some(_) =>
        {}
    }

    if brain.m_tantrum
    {
        target = None;
        brain.m_cry -= 1;
        if brain.m_cry <= 0
        {
            world.playSound(info.pos, "twilightforest.mob.urghast.hurt", 16.0, 0.5);
            brain.m_cry = boss::CRY_MIN + world.random(boss::CRY_SPREAD);
        }

        if brain.m_ticks % boss::RAIN_PERIOD == 0
        {
            tantrumRain(world, info, entity);
        }
    }

    brain.m_target = target.as_ref().map(|target| target.id);

    // EntityTFUrGhast.checkAndChangeCourse and the waypoint flight
    if brain.m_course.is_none_or(|course| distance(course, info.pos).powi(2) < boss::COURSE_NEAR)
    {
        changeCourse(world, info, brain);
    }

    let away: f64 = distance(brain.m_waypoint, info.pos).powi(2);
    if !(boss::WAYPOINT_NEAR..=boss::WAYPOINT_FAR).contains(&away)
    {
        makeWaypoint(world, info, brain);
    }

    brain.m_steer -= 1;

    let mut motion: [f64; 3] = info.motion.map(|axis| axis * boss::DRAG);

    if brain.m_steer <= 0
    {
        brain.m_steer += world.random(5);

        let delta: [f64; 3] = sub(brain.m_waypoint, info.pos);
        let length: f64 = length(delta).max(1.0e-4);
        motion = [0, 1, 2].map(|axis| motion[axis] + delta[axis] / length * boss::SPEED);
    }

    let _ = entity.setMotion(motion);

    let range: f64 = if brain.m_aggro <= 0 && !brain.m_aggressive
    {
        boss::STARE_RANGE
    }
    else
    {
        boss::AGGRO_RANGE
    };
    let height: f64 = entity.field(EntityField::Height);
    let center: [f64; 3] = [info.pos[0], info.pos[1] + height / 2.0, info.pos[2]];
    let inSight: Option<EntityInfo> = target
        .filter(|target| distance(target.pos, info.pos) < range && world.rayTrace(center, target.pos, false).is_none());

    match inSight
    {
        Some(target) =>
        {
            crate::entities::ai::face(entity, info.pos, target.pos);

            if brain.m_aggressive
            {
                if brain.m_attack == boss::SPIT_CHARGE
                {
                    world.playSound(info.pos, "mob.ghast.charge", 16.0, 0.5);
                }

                brain.m_attack += 1;
                if brain.m_attack == boss::SPIT
                {
                    spit(entity, info, &target);
                    brain.m_attack = boss::SPIT_REST;
                }
            }
        }

        None =>
        {
            brain.m_aggressive = false;
            brain.m_target = None;

            let yaw: f64 = -motion[0].atan2(motion[2]).to_degrees();
            entity.setField(EntityField::Yaw, yaw);
            entity.setField(EntityField::Pitch, 0.0);
        }
    }

    if brain.m_attack > 0 && !brain.m_aggressive
    {
        brain.m_attack -= 1;
    }
}

// EntityTFUrGhast.switchPhase, a tantrum calls ghasts down from the traps and storms over the overworld
fn switchPhase(world: World, brain: &mut Brain)
{
    brain.m_tantrum = !brain.m_tantrum;
    if brain.m_tantrum
    {
        storm();
        minions(world, brain);
    }

    brain.m_untilPhase = boss::PHASE_DAMAGE;
}

// EntityTFUrGhast.startTantrum
fn storm()
{
    let ticks: i32 = 6000;
    let overworld: World = World { dimension: crate::portal::travel::OVERWORLD };
    if let Err(error) = overworld.setWeather(true, true, ticks)
    {
        warn(&format!("ur-ghast storm: {}", error.0));
    }
}

pub(crate) fn trapped(entity: Entity)
{
    BRAINS.with(|brains|
    {
        if let Some(brain) = brains.borrow_mut().get_mut(&(entity.world.dimension, entity.id))
        {
            brain.m_trapped = boss::TRAP_HOLD;
        }
    });
}

struct UrGhast;

#[entity_behavior]
impl EntityBehavior for UrGhast
{
    fn onAi(&self, entity: Entity, info: &EntityInfo)
    {
        if !entity.world.isAuthority()
        {
            return;
        }

        let shown: Shown = BRAINS.with(|brains|
        {
            let mut brains: RefMut<HashMap<(i32, i32), Brain>> = brains.borrow_mut();
            let brain: &mut Brain = brains.entry((entity.world.dimension, entity.id)).or_default();
            think(entity, info, brain);

            let status: u8 = if brain.m_attack > boss::SPIT_CHARGE
            {
                2
            }
            else if brain.m_aggro <= 0 && !brain.m_aggressive
            {
                0
            }
            else
            {
                1
            };
            Shown { m_status: status, m_attack: brain.m_attack, m_tantrum: brain.m_tantrum }
        });
        if entity.synced::<Shown>() != shown
        {
            let _ = entity.setSynced(&shown);
        }

        showBar(entity, LABEL, info);
    }

    // the tears a tantrum sheds
    fn onTick(&self, entity: Entity, info: &EntityInfo)
    {
        let world: World = entity.world;
        // the tantrum drips are each client's own
        if world.isAuthority() || !entity.synced::<Shown>().m_tantrum
        {
            return;
        }

        let width: f64 = entity.field(EntityField::Width);
        let height: f64 = entity.field(EntityField::Height);
        let at: [f64; 3] = [
            info.pos[0] + (f64::from(crate::math::fraction(world)) - 0.5) * width,
            info.pos[1] + f64::from(crate::math::fraction(world)) * height - 0.25,
            info.pos[2] + (f64::from(crate::math::fraction(world)) - 0.5) * width,
        ];
        crate::particles::spawn(world, "drip_water", at, [0.0; 3]);
    }

    // EntityTFUrGhast.attackEntityFrom, tantrums soften blows and every 48 damage flips the phase
    fn onHurt(&self, entity: Entity, _source: Option<i32>, amount: i32) -> Option<i32>
    {
        if crate::combat::hurtBy(crate::combat::kind::IN_WALL)
        {
            return None;
        }

        let world: World = entity.world;

        BRAINS.with(|brains|
        {
            let mut brains: RefMut<HashMap<(i32, i32), Brain>> = brains.borrow_mut();
            let brain: &mut Brain = brains.entry((world.dimension, entity.id)).or_default();
            let dealt: i32 = if brain.m_tantrum
            {
                amount / boss::TANTRUM_SHIELD
            }
            else
            {
                amount
            };
            brain.m_untilPhase -= dealt;
            if brain.m_untilPhase <= 0
            {
                switchPhase(world, brain);
            }

            Some(dealt)
        })
    }

    // EntityTFUrGhast.onDeathUpdate and onDeath, the boss treasure over the middle of the traps
    fn onDeath(&self, entity: Entity, source: Option<i32>)
    {
        let world: World = entity.world;
        let brain: Option<Brain> = BRAINS.with(|brains| brains.borrow_mut().remove(&(world.dimension, entity.id)));

        if let Some(player) = killer(world, source)
        {
            progress::grant(player, Gate::Hunter);
            progress::grant(player, Gate::UrGhast);
        }

        let Some(info) = entity.info()
        else
        {
            return;
        };

        if !world.isAuthority()
        {
            return;
        }

        let traps: Vec<[i32; 3]> = brain.map(|brain| brain.m_traps).unwrap_or_default();
        let chest: [i32; 3] = if traps.is_empty()
        {
            info.pos.map(floor)
        }
        else
        {
            let [x, y, z] = average(&traps);
            [x, y + 2, z]
        };
        let generation: GenWorld = GenWorld::new(world);
        treasure::place(
            &generation,
            &b173_api::terrain::Random::withSeed(0),
            Table::DarkTowerBoss,
            chest[0],
            chest[1],
            chest[2],
        );
        super::conquer(world, info.pos);
    }

    // RenderTFUrGhast and ModelTFTowerBoss.setRotationAngles: eyes by attack status, swelling before a shot
    fn pose(&self, input: &PoseInput) -> Pose
    {
        let shown: Shown = b173_api::fromBytes(&input.synced).unwrap_or_default();
        let swell: f32 = (shown.m_attack.max(0) as f32 / boss::SWELL_TICKS).powi(boss::SWELL_POWER) * boss::SWELL + 1.0;
        let swell: f32 = 1.0 / swell;
        let (across, up): (f32, f32) = ((boss::SCALE + 1.0 / swell) / 2.0, (boss::SCALE + swell) / 2.0);
        let mut pre: Program = Program::new();
        pre.scale(across, up, across);

        let texture: &str = match shown.m_status
        {
            1 => "twilightforest:model/towerboss_openeyes",
            2 => "twilightforest:model/towerboss_fire",
            _ => "twilightforest:model/towerboss",
        };

        let mut body: Program = Program::new();
        body.texture(texture);
        drawBody(&mut body, 0, input.age, [input.head_pitch.to_radians(), input.head_yaw.to_radians()]);
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
            texture: Some("twilightforest:model/towerboss".to_owned()),
            size: Some([14.0, 18.0]),
            health: Some(boss::HEALTH),
            name: Some(LABEL.to_owned()),
            sounds: Some(EntitySounds
            {
                living: Some("twilightforest.mob.urghast.moan".to_owned()),
                hurt: Some("twilightforest.mob.urghast.hurt".to_owned()),
                death: Some("twilightforest.mob.urghast.death".to_owned()),
            }),
            model: Some(EntityModelSpec { shadow: 0.625, ..model() }),
            fire_immune: true,
            persistent: true,
            tracking_range: Some(160),
            ..EntitySpec::default()
        },
        Some(Rc::new(UrGhast)),
    )?;
    Ok(())
}
