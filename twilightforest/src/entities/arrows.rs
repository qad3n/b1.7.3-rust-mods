use crate::ids::item;
use crate::math::{heading, turn};
use b173_api::math::{add, distance, dot, length, normalize, sub};
use b173_api::prelude::*;
use std::rc::Rc;

// the twilight bows fire EntityArrow and its EntityIceArrow and EntitySeekerArrow subclasses, all drawn by RenderArrow
pub(crate) const PLAIN: &str = "minecraft:arrow";
pub(crate) const ICE: &str = "twilightforest:ice_arrow";
pub(crate) const SEEKER: &str = "twilightforest:seeker_arrow";
pub(crate) const SIDE: &str = "twilightforest:side_arrow";
const KINDS: [&str; 4] = [PLAIN, ICE, SEEKER, SIDE];

mod launch
{
    // EntityArrow's player constructor starts it this far left of the eye and this far below it
    pub const SIDE: f64 = 0.16;
    pub const DROP: f64 = 0.1;
    pub const EYE: f64 = 0.12;
}

mod seek
{
    // EntitySeekerArrow.seekDistance, it looks this many ticks of flight ahead within 30 degrees either side
    pub const AHEAD: f64 = 5.0;
    pub const CONE: f64 = std::f64::consts::PI / 6.0;
    pub const HEIGHT: f64 = 3.0;
    // it homes only while faster than this
    pub const FLYING: f64 = 1.0;
    pub const PULL: [f64; 3] = [2.0, 1.0, 2.0];
    // beta arrows fall 0.03 a tick, the seeker cancels nine tenths of that as it cancels 0.045 of 0.05 upstream
    pub const LIFT: f64 = 0.027;
    pub const EYE: f64 = 0.85;
    pub const TRACKING: i32 = 150;
}

// EntityTracker's range and update period for arrows
mod tracking
{
    pub const RANGE: i32 = 64;
    pub const PERIOD: i32 = 20;
}

// the ender bow's arrow reached its target within this many blocks of where it stood when the hit landed
const HIT_REACH: f64 = 6.0;

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct Homing
{
    m_target: Option<i32>,
}

// the vector from a creature's eyes to the arrow, normalized
fn fromEyes(world: World, arrow: [f64; 3], target: &EntityInfo) -> [f64; 3]
{
    let height: f64 = Entity { world, id: target.id }.field(EntityField::Height);
    normalize([arrow[0] - target.pos[0], arrow[1] - (target.pos[1] + height * seek::EYE), arrow[2] - target.pos[2]])
}

// EntityArrow(world, player, velocity) with setArrowHeading, the triple bow's side arrows lift and rise a little
pub(crate) fn shoot(world: World, kind: &str, shooter: &EntityInfo, speed: f64, lift: f64, rise: f64)
-> Option<Entity>
{
    let yaw: f64 = f64::from(shooter.yaw).to_radians();
    let from: [f64; 3] = [
        shooter.pos[0] - yaw.cos() * launch::SIDE,
        shooter.pos[1] + launch::EYE - launch::DROP + rise,
        shooter.pos[2] - yaw.sin() * launch::SIDE,
    ];

    let entity: Entity = match world.spawnEntity(kind, from)
    {
        Ok(entity) => entity,

        Err(error) =>
        {
            warn(&format!("cannot shoot {kind}: {}", error.0));
            return None;
        }
    };

    let aim: [f64; 3] = normalize(crate::math::look(shooter.yaw, shooter.pitch));
    let mut motion: [f64; 3] = aim.map(|axis| (axis + crate::entities::shots::wobble(world, 1.0)) * speed);
    motion[1] += lift;
    if let Err(error) = entity.setMotion(motion)
    {
        warn(&format!("arrow motion: {}", error.0));
    }

    // a new arrow faces its flight since RenderArrow only eases toward it from the random spawn yaw
    let (yaw, pitch): (f64, f64) = heading(motion);
    entity.setField(EntityField::Yaw, yaw);
    entity.setField(EntityField::Pitch, pitch);
    entity.setField(EntityField::Shooter, f64::from(shooter.id));
    Some(entity)
}

// where a side arrow stood last tick, it stays put once stuck
#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct Resting
{
    m_last: Option<[f64; 3]>,
    m_stuck: bool,
}

// the triple bow's side arrows cannot be picked up, a stuck one forgets its shooter so EntityArrow keeps it
struct Side;

#[entity_behavior]
impl EntityBehavior for Side
{
    fn onTick(&self, entity: Entity, info: &EntityInfo)
    {
        let mut state: Resting = entity.data();
        if !entity.world.isAuthority() || state.m_stuck
        {
            return;
        }

        state.m_stuck = state.m_last == Some(info.pos);
        state.m_last = Some(info.pos);

        if state.m_stuck
        {
            entity.setField(EntityField::Shooter, -1.0);
        }

        if let Err(error) = entity.setData(&state)
        {
            warn(&format!("side arrow rest: {}", error.0));
        }
    }
}

struct Seeker;

impl Seeker
{
    // the living creature most nearly ahead in a cone in front of the arrow, players left alone
    fn find(entity: Entity, info: &EntityInfo) -> Option<i32>
    {
        let world: World = entity.world;
        let reach: [f64; 3] = info.motion.map(|axis| axis * seek::AHEAD);
        let corners: [[f64; 3]; 3] =
            [[0.0; 3], turn(reach, seek::CONE), turn(reach, -seek::CONE)].map(|corner| add(info.pos, corner));
        let low: [f64; 3] = [0, 1, 2].map(|axis| corners.iter().map(|corner| corner[axis]).fold(f64::MAX, f64::min));
        let high: [f64; 3] = [0, 1, 2].map(|axis| corners.iter().map(|corner| corner[axis]).fold(f64::MIN, f64::max));
        let low: [f64; 3] = [low[0], low[1] - seek::HEIGHT, low[2]];
        let high: [f64; 3] = [high[0], high[1] + seek::HEIGHT, high[2]];
        let center: [f64; 3] = [0, 1, 2].map(|axis| (low[axis] + high[axis]) / 2.0);
        let radius: f64 = distance(high, center);
        let course: [f64; 3] = normalize(info.motion);
        world
            .entitiesNear(center, radius)
            .into_iter()
            .filter(|near| near.alive && near.health.is_some() && near.player.is_none() && near.id != entity.id)
            .filter(|near| (0..3).all(|axis| (low[axis]..=high[axis]).contains(&near.pos[axis])))
            .map(|near| (dot(course, fromEyes(world, info.pos, &near)), near.id))
            .filter(|(closeness, _)| *closeness < 1.0)
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, id)| id)
    }

    // the pull toward a target still ahead, None once it falls behind
    fn pull(world: World, info: &EntityInfo, target: i32) -> Option<[f64; 3]>
    {
        let target: EntityInfo = world.entity(target).filter(|target| target.alive)?;
        let toward: [f64; 3] = fromEyes(world, info.pos, &target);
        if dot(normalize(info.motion), toward) >= 0.0
        {
            return None;
        }

        let speed: f64 = length(info.motion);
        Some([0, 1, 2].map(|axis| (toward[axis] * speed).clamp(-seek::PULL[axis], seek::PULL[axis])))
    }
}

#[entity_behavior]
impl EntityBehavior for Seeker
{
    // EntitySeekerArrow.onUpdate, after EntityArrow moved it here rather than before
    fn onTick(&self, entity: Entity, info: &EntityInfo)
    {
        if !entity.world.isAuthority() || length(info.motion) <= seek::FLYING
        {
            return;
        }

        let mut state: Homing = entity.data();
        let mut motion: [f64; 3] = info.motion;

        match state.m_target
        {
            None => state.m_target = Self::find(entity, info),
            Some(target) => match Self::pull(entity.world, info, target)
            {
                Some(pull) => motion = sub(motion, pull),
                None => state.m_target = None,
            },
        }

        motion[1] += seek::LIFT;

        if let Err(error) = entity.setMotion(motion)
        {
            warn(&format!("seeker arrow motion: {}", error.0));
        }

        if let Err(error) = entity.setData(&state)
        {
            warn(&format!("seeker arrow target: {}", error.0));
        }
    }
}

// one of the shooter's arrows close enough to the target to be what hit it
fn arrowHit(world: World, target: &EntityInfo, shooter: i32) -> bool
{
    world.entitiesNear(target.pos, HIT_REACH).into_iter().any(|near|
    {
        let arrow: bool = near.kind.as_ref().is_some_and(|kind| KINDS.contains(&kind.to_string().as_str()));
        arrow && Entity { world, id: near.id }.field(EntityField::Shooter) as i32 == shooter
    })
}

// TFEventListener.entityHurts, an arrow from a player holding the ender bow swaps the two places
fn hurt(event: &mut Event)
{
    let EventData::Attack { entity: target, source: Some(source), .. } = event.data().clone()
    else
    {
        return;
    };
    let world: World = World { dimension: target.world };
    if !world.isAuthority()
    {
        return;
    }

    let player: Player = Player { id: source };
    // TFEventListener.entityHurts, an arrow from a player holding the ice bow chills for ten seconds at three
    let icy: bool = player.held().is_some_and(|stack| stack.item.is(item::ICE_BOW));
    if icy && world.entity(source).is_some_and(|shooter| shooter.player.is_some()) && arrowHit(world, &target, source)
    {
        let (level, ticks): (i32, i32) = (2, 200);
        crate::effects::give(Entity { world, id: target.id }, crate::effects::Effect::Slowness, level, ticks);
    }

    let holding: bool = player.held().is_some_and(|stack| stack.item.is(item::ENDER_BOW));
    let Some(shooter) = world.entity(source).filter(|shooter| shooter.player.is_some() && holding)
    else
    {
        return;
    };

    if !arrowHit(world, &target, shooter.id)
    {
        return;
    }

    let player: Entity = Entity { world, id: shooter.id };
    let victim: Entity = Entity { world, id: target.id };
    for (entity, to, yaw, pitch) in
        [(player, &target, target.yaw, target.pitch), (victim, &shooter, shooter.yaw, shooter.pitch)]
    {
        if let Err(error) = entity.setPosition(to.pos)
        {
            warn(&format!("ender bow swap: {}", error.0));
        }

        entity.setField(EntityField::Yaw, f64::from(yaw));
        entity.setField(EntityField::Pitch, f64::from(pitch));
        world.playSound(to.pos, "mob.endermen.portal", 1.0, 1.0);
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    let rows: [(&str, Option<Rc<dyn EntityBehavior>>, i32, i32, bool); 3] = [
        // EntityIceArrow has no entity registration upstream, it tracks as a vanilla arrow
        ("ice_arrow", None, tracking::RANGE, tracking::PERIOD, false),
        ("side_arrow", Some(Rc::new(Side)), tracking::RANGE, tracking::PERIOD, false),
        ("seeker_arrow", Some(Rc::new(Seeker)), seek::TRACKING, 1, true),
    ];

    for (name, behavior, range, frequency, velocity) in rows
    {
        ctx.entities().register(
            name,
            EntitySpec
            {
                template: Identifier::parse("minecraft:arrow").ok(),
                tracking_range: Some(range),
                update_frequency: Some(frequency),
                send_velocity: Some(velocity),
                ..EntitySpec::default()
            },
            behavior,
        )?;
    }

    ctx.events().on::<LivingHurt>(Priority::Normal, hurt)?;
    Ok(())
}
