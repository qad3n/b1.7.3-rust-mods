use super::ai::{distance, target};
use crate::ids;
use b173_api::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

// EntityTFRedcap's own tasks: EntityAITFRedcapShy circles a player who looks at it, EntityAITFRedcapLightTNT lights
// tnt nearby, EntityAITFRedcapPlantTNT has the sapper plant its three tnt behind a player's back
mod tuning
{
    pub const SHY_FROM: f64 = 3.0;
    pub const SHY_TO: f64 = 6.0;
    pub const CIRCLE: f64 = 5.0;
    // EntityLiving.recentlyHit, a redcap a player just hit stops being shy
    pub const BOLD_TICKS: i64 = 60;
    pub const LIGHT_SEARCH: i32 = 8;
    pub const LIGHT_REACH: f64 = 2.4;
    pub const LIGHT_DELAY: i32 = 20;
    pub const PLANT_REACH: f64 = 5.0;
    pub const PLANT_CLEAR: i32 = 5;
    pub const LIT_CLEAR: f64 = 8.0;
    // isTargetLookingAtMe allows sixty degrees either side
    pub const GAZE: f32 = 60.0;
    // minecraft yaw counts from south where atan2 counts from east
    pub const YAW_OFFSET: f32 = 90.0;
    pub const FULL_TURN: f32 = 360.0;
    pub const SPEED: f32 = 1.0;
}

const SAPPER_TNT: i32 = 3;
const TNT: &str = "minecraft:tnt";
const PRIMED: &str = "minecraft:primed_tnt";

#[derive(Default, Clone, Copy)]
struct Redcap
{
    m_lefty: bool,
    m_hit: i64,
    m_delay: i32,
    m_planted: i32,
}

thread_local!
{
    static REDCAPS: RefCell<HashMap<(i32, i32), Redcap>> = RefCell::new(HashMap::new());
}

fn state(entity: Entity) -> Redcap
{
    REDCAPS.with(|redcaps|
    {
        *redcaps
            .borrow_mut()
            .entry((entity.world.dimension, entity.id))
            .or_insert_with(|| Redcap { m_lefty: entity.world.random(2) == 0, ..Redcap::default() })
    })
}

fn setState(entity: Entity, redcap: Redcap)
{
    REDCAPS.with(|redcaps| redcaps.borrow_mut().insert((entity.world.dimension, entity.id), redcap));
}

pub(crate) fn forget(entity: Entity)
{
    REDCAPS.with(|redcaps| redcaps.borrow_mut().remove(&(entity.world.dimension, entity.id)));
}

// recentlyHit for the shy check
pub(crate) fn hurt(entity: Entity)
{
    let redcap: Redcap = state(entity);
    setState(entity, Redcap { m_hit: entity.world.time(), ..redcap });
}

fn floor(pos: [f64; 3]) -> BlockPos
{
    BlockPos::new(pos[0].floor() as i32, pos[1].floor() as i32, pos[2].floor() as i32)
}

// EntityAITFRedcapBase.isTargetLookingAtMe
fn watched(info: &EntityInfo, watcher: &EntityInfo) -> bool
{
    let (dx, dz): (f64, f64) = (info.pos[0] - watcher.pos[0], info.pos[2] - watcher.pos[2]);
    let angle: f32 = dz.atan2(dx).to_degrees() as f32 - tuning::YAW_OFFSET;
    let difference: f32 = ((watcher.yaw - angle) % tuning::FULL_TURN).abs();
    difference < tuning::GAZE || difference > tuning::FULL_TURN - tuning::GAZE
}

// EntityAITFRedcapBase.findBlockTNTNearby, the first tnt in the cube around it
fn tntNear(world: World, at: BlockPos, range: i32) -> Option<BlockPos>
{
    let tnt: i32 = ids::blockId(TNT);
    (-range..=range)
        .flat_map(|x| (-range..=range).flat_map(move |y| (-range..=range).map(move |z| at.offset(x, y, z))))
        .find(|pos| world.block(*pos) == tnt)
}

fn litNear(world: World, info: &EntityInfo) -> bool
{
    world
        .entitiesNear(info.pos, tuning::LIT_CLEAR)
        .iter()
        .any(|other| other.kind.as_ref().is_some_and(|kind| kind.to_string() == PRIMED))
}

// EntityAITFRedcapShy.findCirclePoint
fn circle(info: &EntityInfo, around: &EntityInfo, lefty: bool) -> BlockPos
{
    let turn: f64 = if lefty { 1.0 } else { -1.0 };
    let angle: f64 = (info.pos[2] - around.pos[2]).atan2(info.pos[0] - around.pos[0]) + turn;
    floor([around.pos[0] + angle.cos() * tuning::CIRCLE, info.pos[1], around.pos[2] + angle.sin() * tuning::CIRCLE])
}

fn shy(entity: Entity, info: &EntityInfo, redcap: Redcap) -> bool
{
    let Some(goal) = target(entity)
    else
    {
        return false;
    };
    let gap: f64 = distance(goal.pos, info.pos);
    let bold: bool = entity.world.time() - redcap.m_hit < tuning::BOLD_TICKS;
    if bold || !(tuning::SHY_FROM..=tuning::SHY_TO).contains(&gap) || !watched(info, &goal)
    {
        return false;
    }

    entity.walkTo(circle(info, &goal, redcap.m_lefty), tuning::SPEED);
    true
}

// EntityAITFRedcapLightTNT
fn light(entity: Entity, info: &EntityInfo, redcap: &mut Redcap) -> bool
{
    if redcap.m_delay > 0
    {
        redcap.m_delay -= 1;
        return false;
    }

    let world: World = entity.world;
    let Some(tnt) = tntNear(world, floor(info.pos), tuning::LIGHT_SEARCH)
    else
    {
        return false;
    };

    let center: [f64; 3] = [f64::from(tnt.x) + 0.5, f64::from(tnt.y), f64::from(tnt.z) + 0.5];
    if distance(center, info.pos) >= tuning::LIGHT_REACH
    {
        entity.walkTo(tnt, tuning::SPEED);
        return true;
    }

    if let Err(error) = world.setBlock(tnt, 0, 0)
    {
        warn(&format!("redcap tnt: {}", error.0));
        return true;
    }

    if let Err(error) = world.spawnEntity(PRIMED, center)
    {
        warn(&format!("redcap primed tnt: {}", error.0));
    }

    redcap.m_delay = tuning::LIGHT_DELAY;
    true
}

// EntityAITFRedcapPlantTNT, the sapper sets tnt at its feet while the player looks away
fn plant(entity: Entity, info: &EntityInfo, redcap: &mut Redcap)
{
    let world: World = entity.world;

    let Some(goal) = target(entity)
    else
    {
        return;
    };

    let feet: BlockPos = floor(info.pos);
    let ready: bool = redcap.m_planted < SAPPER_TNT
        && distance(goal.pos, info.pos) < tuning::PLANT_REACH
        && !watched(info, &goal)
        && !litNear(world, info)
        && tntNear(world, feet, tuning::PLANT_CLEAR).is_none();

    if !ready || world.block(feet) != 0
    {
        return;
    }

    if let Err(error) = world.setBlock(feet, ids::blockId(TNT), 0)
    {
        warn(&format!("sapper tnt: {}", error.0));
        return;
    }

    redcap.m_planted += 1;
}

pub(crate) fn tick(entity: Entity, info: &EntityInfo, sapper: bool)
{
    let mut redcap: Redcap = state(entity);
    if !shy(entity, info, redcap) && !light(entity, info, &mut redcap) && sapper
    {
        plant(entity, info, &mut redcap);
    }

    setState(entity, redcap);
}
