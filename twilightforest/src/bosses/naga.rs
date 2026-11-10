use super::{dropAt, home, killer, showBar};
use crate::entities::ai::{distance, face};
use crate::ids::{self, item};
use crate::models::{Program, tf_naga};
use crate::progress::{self, Gate};
use b173_api::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::f64::consts::PI;
use std::rc::Rc;

// EntityTFNaga, a serpent whose body shortens as it is hurt, circling, charging and crushing the ground under a
// player who climbs out of reach, kept to its courtyard
pub const NAME: &str = "naga";
const SEGMENT: &str = "naga_segment";
const LABEL: &str = "Naga";

mod naga
{
    pub const MAX_SEGMENTS: usize = 12;
    // the naga's health is split over ten segments, two more stay while it lives
    pub const HEALTH_PARTS: f64 = 10.0;
    pub const EXTRA_SEGMENTS: i32 = 2;
    pub const HEAL_AFTER: i32 = 600;
    pub const HEAL_EVERY: i32 = 20;
    pub const LEASH: [i32; 3] = [46, 7, 46];
    pub const SEARCH: f64 = 32.0;
    pub const LOSE: f64 = 80.0;
    pub const REACH: f64 = 4.0;
    pub const ATTACK_COOLDOWN: i32 = 20;
    pub const BITE: i32 = 6;
    pub const SEGMENT_BITE: i32 = 2;
    pub const ANIMAL_BITE: i32 = 3;
    pub const SEGMENT_COOLDOWN: i32 = 10;
    pub const SPACING: f64 = 2.0;
    pub const NORMAL: f64 = 0.6;
    pub const FAST: f64 = 1.0;
    pub const SLOW: f64 = 0.1;
    // the walking speed a land movement factor reaches against ground friction, per unit of speed
    pub const STRIDE: f64 = 0.6;
    pub const JUMP: f64 = 0.42;
    pub const ARRIVE: f64 = 7.0;
    pub const SCALES: [i32; 2] = [6, 6];
    pub const DIE_PUFFS: i32 = 20;
    pub const KNOCKBACK: f64 = 1.0;
    pub const KNOCK_UP: f64 = 0.1;
}

#[derive(Default)]
struct Brain
{
    m_segments: Vec<i32>,
    m_quiet: i32,
    m_circle: i32,
    m_intimidate: i32,
    m_crumble: i32,
    m_charge: i32,
    m_clockwise: bool,
    m_target: Option<i32>,
    m_goal: Option<[f64; 3]>,
    m_speed: f64,
    m_cooldown: i32,
}

thread_local!
{
    static BRAINS: RefCell<HashMap<(i32, i32), Brain>> = RefCell::new(HashMap::new());
}

fn inHome(home: Option<[i32; 3]>, at: [f64; 3]) -> bool
{
    let Some(center) = home
    else
    {
        return true;
    };
    (0..3).all(|axis| (center[axis] - at[axis].floor() as i32).abs() <= naga::LEASH[axis])
}

// EntityTFNaga.findCirclePoint
fn circlePoint(info: &EntityInfo, target: &EntityInfo, radius: f64, turn: f64, clockwise: bool) -> [f64; 3]
{
    let angle: f64 =
        (info.pos[2] - target.pos[2]).atan2(info.pos[0] - target.pos[0]) + if clockwise { turn } else { -turn };
    [target.pos[0] + angle.cos() * radius, info.pos[1].min(target.pos[1]), target.pos[2] + angle.sin() * radius]
}

// EntityTFNaga.breakBlock, dropping what was there
fn breakBlock(world: World, pos: BlockPos)
{
    let (block, meta): (i32, i32) = (world.block(pos), world.meta(pos));
    if block == 0 || World::blockInfo(block).hardness < 0.0
    {
        return;
    }
    let _ = world.dropBlockItems(pos, block, meta);
    let _ = world.setBlock(pos, 0, 0);
    let _ = world.breakEffect(pos, block, meta);
}

impl Brain
{
    // EntityTFNaga.acquireNewPath, the next step of the circle, charge, intimidate and crumble cycle
    fn plan(&mut self, entity: Entity, info: &EntityInfo, target: Option<&EntityInfo>)
    {
        let world: World = entity.world;
        let Some(target) = target
        else
        {
            self.m_speed = naga::NORMAL;
            let center: [f64; 3] = home(entity).map_or(info.pos, |home| home.map(f64::from));
            let spread = |reach: i32| f64::from(world.random(reach) - world.random(reach));
            self.m_goal = Some([center[0] + spread(21), center[1] + spread(7), center[2] + spread(21)]);
            return;
        };
        if self.m_intimidate > 0
        {
            self.m_goal = None;
            self.m_intimidate -= 1;
            if self.m_intimidate == 0
            {
                self.m_clockwise = !self.m_clockwise;
                let height: f64 = entity.field(EntityField::Height);
                if target.pos[1] > info.pos[1] + height
                {
                    self.m_crumble = 20 + world.random(20);
                    self.m_speed = naga::SLOW;
                }
                else
                {
                    self.charge();
                }
            }
            return;
        }
        if self.m_crumble > 0
        {
            self.m_goal = None;
            self.m_crumble -= 1;
            crumbleBelow(world, info, target, 2);
            crumbleBelow(world, info, target, 3);
            if self.m_crumble == 0
            {
                self.charge();
            }
        }
        if self.m_charge > 0
        {
            self.m_charge -= 1;
            self.m_goal = Some(circlePoint(info, target, 14.0, PI, self.m_clockwise));
            if self.m_charge == 0
            {
                self.m_circle += 10 + world.random(10);
                self.m_speed = naga::NORMAL;
            }
        }
        if self.m_circle > 0
        {
            self.m_circle -= 1;
            let mut radius: f64 = if self.m_circle % 2 == 0 { 12.0 } else { 14.0 };
            let mut turn: f64 = 1.0;
            if self.m_circle > 1 && self.m_circle < 3
            {
                radius = 16.0;
            }
            if self.m_circle == 1
            {
                turn = 0.1;
            }
            self.m_goal = Some(circlePoint(info, target, radius, turn, self.m_clockwise));
            if self.m_circle == 0
            {
                self.m_intimidate += 15 + world.random(10);
                self.m_speed = naga::SLOW;
                self.m_goal = None;
            }
        }
    }

    fn charge(&mut self)
    {
        self.m_charge = 4;
        self.m_speed = naga::FAST;
    }
}

// EntityTFNaga.crumbleBelowTarget, the blocks under a player above the naga give way
fn crumbleBelow(world: World, info: &EntityInfo, target: &EntityInfo, range: i32)
{
    let floor: i32 = info.pos[1] as i32;
    let top: i32 = Entity { world, id: target.id }.field(EntityField::Bottom) as i32;
    if top <= floor
    {
        return;
    }
    let x: i32 = target.pos[0] as i32 + world.random(range) - world.random(range);
    let z: i32 = target.pos[2] as i32 + world.random(range) - world.random(range);
    let mut y: i32 = top - world.random(range) + world.random(if range > 1 { range - 1 } else { range });
    if y <= floor
    {
        y = top;
    }
    let pos: BlockPos = BlockPos::new(x, y, z);
    if world.block(pos) != 0
    {
        breakBlock(world, pos);
    }
}

fn findTarget(entity: Entity, info: &EntityInfo) -> Option<EntityInfo>
{
    let player: EntityInfo = crate::entities::closestPlayer(entity.world, info, naga::SEARCH)?;
    let seen: bool = entity.world.rayTrace(crate::entities::ai::eyes(entity, info), player.pos, false).is_none();
    (seen && inHome(home(entity), player.pos)).then_some(player)
}

// EntityTFNaga.setSegmentsPerHealth and spawnBodySegments
fn keepSegments(entity: Entity, info: &EntityInfo, brain: &mut Brain)
{
    let world: World = entity.world;
    let health: f64 = f64::from(info.health.unwrap_or(0));
    let part: f64 = entity.field(EntityField::MaxHealth) / naga::HEALTH_PARTS;
    let wanted: usize = ((health / part) as i32
        + if health > 0.0
        {
            naga::EXTRA_SEGMENTS
        }
        else
        {
            0
        })
    .clamp(0, naga::MAX_SEGMENTS as i32) as usize;
    brain.m_segments.retain(|id| Entity { world, id: *id }.info().is_some_and(|segment| segment.alive));
    while brain.m_segments.len() > wanted
    {
        if let Some(id) = brain.m_segments.pop()
        {
            let segment: Entity = Entity { world, id };
            if let Some(at) = segment.info().map(|segment| segment.pos)
            {
                for _ in 0..naga::DIE_PUFFS
                {
                    world.particle("explode", at, [0.0; 3]);
                }
            }
            segment.kill();
        }
    }
    while brain.m_segments.len() < wanted
    {
        let offset: f64 = 0.1 * brain.m_segments.len() as f64;
        let at: [f64; 3] = [info.pos[0] + offset, info.pos[1] + 0.5, info.pos[2] + offset];
        let Ok(segment) = world.spawnEntity(&format!("twilightforest:{SEGMENT}"), at)
        else
        {
            return;
        };
        let _ = segment.setData(&Owner { m_naga: entity.id });
        brain.m_segments.push(segment.id);
    }
}

// EntityTFNaga.moveSegments, each segment trails the one before it two blocks back
fn moveSegments(entity: Entity, info: &EntityInfo, brain: &Brain)
{
    let world: World = entity.world;
    let mut leader: EntityInfo = info.clone();
    for (index, id) in brain.m_segments.iter().enumerate()
    {
        let segment: Entity = Entity { world, id: *id };
        let Some(mine) = segment.info()
        else
        {
            continue;
        };
        let angle: f64 = f64::from(leader.yaw + 180.0).to_radians();
        let straighten: f64 = 0.05 + 1.0 / (index as f64 + 1.0) * 0.5;
        let mut diff: [f64; 3] = [0, 1, 2].map(|axis| mine.pos[axis] - leader.pos[axis]);
        normalize(&mut diff);
        diff[0] += -angle.sin() * straighten;
        diff[2] += angle.cos() * straighten;
        normalize(&mut diff);
        let dest: [f64; 3] = [0, 1, 2].map(|axis| leader.pos[axis] + naga::SPACING * diff[axis]);
        let _ = segment.setPosition(dest);
        let _ = segment.setMotion(diff.map(|axis| axis * naga::SPACING));
        let flat: f64 = (diff[0] * diff[0] + diff[2] * diff[2]).sqrt();
        if index == 0
        {
            diff[1] -= 0.15;
        }
        segment.setField(EntityField::Yaw, diff[2].atan2(diff[0]).to_degrees() + 90.0);
        segment.setField(EntityField::Pitch, -diff[1].atan2(flat).to_degrees());
        leader = segment.info().unwrap_or(mine);
    }
}

fn normalize(vector: &mut [f64; 3])
{
    let length: f64 = vector.iter().map(|axis| axis * axis).sum::<f64>().sqrt();
    if length > 1.0e-4
    {
        for axis in vector.iter_mut()
        {
            *axis /= length;
        }
    }
}

// EntityTFNagaSegment.collideWithEntity, the body bites what it rolls over
fn segmentBites(entity: Entity, brain: &mut Brain)
{
    let world: World = entity.world;
    for id in &brain.m_segments
    {
        let Some(segment) = Entity { world, id: *id }.info()
        else
        {
            continue;
        };
        for other in world.entitiesNear(segment.pos, 1.1)
        {
            let ours: bool = other.id == entity.id || brain.m_segments.contains(&other.id);
            if ours || other.health.is_none()
            {
                continue;
            }
            brain.m_cooldown = naga::SEGMENT_COOLDOWN;
            let animal: bool = other
                .kind
                .as_ref()
                .is_some_and(|kind| ["pig", "cow", "sheep", "chicken", "wolf"].contains(&kind.path()));
            let bite: i32 = if animal
            {
                naga::SEGMENT_BITE * naga::ANIMAL_BITE
            }
            else
            {
                naga::SEGMENT_BITE
            };
            Entity { world, id: other.id }.damage(bite, Some(entity.id));
        }
    }
}

// steering in place of the path the naga follows, it runs at its goal and hops over steps
fn steer(entity: Entity, info: &EntityInfo, brain: &mut Brain)
{
    let Some(goal) = brain.m_goal
    else
    {
        let _ = entity.setMotion([info.motion[0] * 0.5, info.motion[1], info.motion[2] * 0.5]);
        return;
    };
    let (dx, dz): (f64, f64) = (goal[0] - info.pos[0], goal[2] - info.pos[2]);
    let flat: f64 = (dx * dx + dz * dz).sqrt();
    if flat < naga::ARRIVE
    {
        brain.m_goal = None;
        return;
    }
    let pace: f64 = brain.m_speed * naga::STRIDE;
    let climb: bool = entity.flag(EntityField::Collided) && entity.flag(EntityField::OnGround);
    let up: f64 = if climb { naga::JUMP } else { info.motion[1] };
    let _ = entity.setMotion([dx / flat * pace, up, dz / flat * pace]);
    face(entity, info.pos, goal);
}

// EntityTFNaga.breakNearbyBlocks when it runs into a wall while hunting
fn smash(entity: Entity, info: &EntityInfo)
{
    let world: World = entity.world;
    let (width, height): (f64, f64) = (entity.field(EntityField::Width), entity.field(EntityField::Height));
    let half: f64 = width / 2.0 + 0.5;
    let floor: i32 = (info.pos[1] + 1.01).floor() as i32;
    for x in (info.pos[0] - half).floor() as i32..=(info.pos[0] + half).floor() as i32
    {
        for y in floor..=(info.pos[1] + height).floor() as i32
        {
            for z in (info.pos[2] - half).floor() as i32..=(info.pos[2] + half).floor() as i32
            {
                breakBlock(world, BlockPos::new(x, y, z));
            }
        }
    }
}

struct Naga;

#[entity_behavior]
impl EntityBehavior for Naga
{
    fn onAi(&self, entity: Entity, info: &EntityInfo)
    {
        if !entity.world.isAuthority()
        {
            return;
        }
        voice(entity.world, info);
        let key: (i32, i32) = (entity.world.dimension, entity.id);
        BRAINS.with(|brains|
        {
            let mut brains = brains.borrow_mut();
            let brain: &mut Brain =
                brains.entry(key).or_insert_with(|| Brain { m_circle: 15, m_speed: naga::NORMAL, ..Brain::default() });
            think(entity, info, brain);
        });
        showBar(entity, LABEL, info);
    }

    // EntityTFNaga.attackEntityFrom, nothing from outside the courtyard hurts it
    fn onHurt(&self, entity: Entity, source: Option<i32>, amount: i32) -> Option<i32>
    {
        let attacker: Option<EntityInfo> = source.and_then(|id| Entity { world: entity.world, id }.info());
        if attacker.as_ref().is_some_and(|attacker| !inHome(home(entity), attacker.pos))
        {
            return None;
        }
        BRAINS.with(|brains|
        {
            if let Some(brain) = brains.borrow_mut().get_mut(&(entity.world.dimension, entity.id))
            {
                brain.m_quiet = 0;
                if let Some(attacker) = source.filter(|id| *id != entity.id)
                {
                    brain.m_target = Some(attacker);
                }
            }
        });
        Some(amount)
    }

    // EntityTFNaga.dropFewItems and onDeath
    fn onDeath(&self, entity: Entity, source: Option<i32>)
    {
        let world: World = entity.world;
        let segments: Vec<i32> = BRAINS
            .with(|brains| brains.borrow_mut().remove(&(world.dimension, entity.id)))
            .map(|brain| brain.m_segments)
            .unwrap_or_default();
        for id in segments
        {
            Entity { world, id }.kill();
        }
        if !world.isAuthority()
        {
            return;
        }
        if let Some(info) = entity.info()
        {
            let scales: i32 = naga::SCALES[0] + world.random(naga::SCALES[1]);
            for _ in 0..scales
            {
                dropAt(world, info.pos, item::NAGA_SCALE, 1);
            }
            dropAt(world, info.pos, item::NAGA_TROPHY, 1);
            super::conquer(world, info.pos);
        }
        if let Some(player) = killer(world, source)
        {
            progress::grant(player, Gate::Hunter);
            progress::grant(player, Gate::KillNaga);
        }
    }

    // RenderTFNaga, the head at twice the model scale
    fn pose(&self, _input: &PoseInput) -> Pose
    {
        let mut body: Program = Program::new();
        body.scale(2.0, 2.0, 2.0).rest(tf_naga::HEAD);
        Pose { pre: Vec::new(), model: body.take() }
    }
}

fn think(entity: Entity, info: &EntityInfo, brain: &mut Brain)
{
    let world: World = entity.world;
    brain.m_quiet += 1;
    if brain.m_quiet > naga::HEAL_AFTER && brain.m_quiet % naga::HEAL_EVERY == 0
    {
        let health: f64 = entity.field(EntityField::Health);
        entity.setField(EntityField::Health, (health + 1.0).min(entity.field(EntityField::MaxHealth)));
    }
    keepSegments(entity, info, brain);
    let mut target: Option<EntityInfo> =
        brain.m_target.and_then(|id| Entity { world, id }.info()).filter(|target| target.alive);
    if target
        .as_ref()
        .is_some_and(|target| !inHome(home(entity), target.pos) || distance(info.pos, target.pos) > naga::LOSE)
    {
        target = None;
    }
    if target.is_none()
    {
        target = findTarget(entity, info);
        if target.is_some()
        {
            brain.m_goal = None;
        }
    }
    brain.m_target = target.as_ref().map(|target| target.id);
    if target.is_some() && entity.flag(EntityField::Collided)
    {
        smash(entity, info);
    }
    brain.m_cooldown = (brain.m_cooldown - 1).max(0);
    if let Some(goal) = target.as_ref()
    {
        bite(entity, info, goal, brain);
    }
    if brain.m_goal.is_none()
    {
        brain.plan(entity, info, target.as_ref());
    }
    if brain.m_intimidate > 0
        && let Some(goal) = target.as_ref()
    {
        face(entity, info.pos, goal.pos);
    }
    steer(entity, info, brain);
    moveSegments(entity, info, brain);
    segmentBites(entity, brain);
}

// EntityTFNaga.attackEntity, a bite in reach that throws the target when the naga charges
fn bite(entity: Entity, info: &EntityInfo, target: &EntityInfo, brain: &mut Brain)
{
    let height: f64 = entity.field(EntityField::Height);
    let level: bool = target.pos[1] > info.pos[1] - 2.5 && target.pos[1] < info.pos[1] + height + 2.5;
    if brain.m_cooldown > 0 || distance(info.pos, target.pos) >= naga::REACH || !level
    {
        return;
    }
    brain.m_cooldown = naga::ATTACK_COOLDOWN;
    let victim: Entity = Entity { world: entity.world, id: target.id };
    victim.damage(naga::BITE, Some(entity.id));
    if brain.m_speed > 0.8
    {
        let yaw: f64 = f64::from(info.yaw).to_radians();
        let _ = victim.setMotion([
            target.motion[0] - yaw.sin() * naga::KNOCKBACK,
            target.motion[1] + naga::KNOCK_UP,
            target.motion[2] + yaw.cos() * naga::KNOCKBACK,
        ]);
    }
}

// which naga a segment belongs to
#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct Owner
{
    m_naga: i32,
}

struct Segment;

#[entity_behavior]
impl EntityBehavior for Segment
{
    // the naga moves its segments, they have no mind of their own and vanish without it
    fn onAi(&self, entity: Entity, _info: &EntityInfo)
    {
        if !entity.world.isAuthority()
        {
            return;
        }
        let owner: Owner = entity.data();
        let alive: bool = Entity { world: entity.world, id: owner.m_naga }.info().is_some_and(|naga| naga.alive);
        if !alive
        {
            entity.kill();
        }
    }

    // EntityTFNagaSegment.attackEntityFrom, two thirds of the hit goes to the naga
    fn onHurt(&self, entity: Entity, source: Option<i32>, amount: i32) -> Option<i32>
    {
        let owner: Owner = entity.data();
        if source.is_some() && entity.world.isAuthority()
        {
            let passed: i32 = (f64::from(amount) * 2.0 / 3.0).round() as i32;
            Entity { world: entity.world, id: owner.m_naga }.damage(passed, source);
        }
        None
    }

    // RenderTFNagaSegment, the body piece turned to the segment's pitch at twice the model scale
    fn pose(&self, input: &PoseInput) -> Pose
    {
        let mut body: Program = Program::new();
        body.rotate(input.entity.pitch, 1.0, 0.0, 0.0).scale(2.0, 2.0, 2.0).rest(tf_naga::BODY);
        Pose { pre: Vec::new(), model: body.take() }
    }
}

// EntityTFNaga.getLivingSound, a hiss two times in three and a rattle otherwise, at about the living sound's pace
fn voice(world: World, info: &EntityInfo)
{
    const TALK_ODDS: i32 = 100;
    const RATTLE_ODDS: i32 = 3;
    if world.random(TALK_ODDS) != 0
    {
        return;
    }
    let sound: &str = if world.random(RATTLE_ODDS) != 0
    {
        "twilightforest.mob.naga.hiss"
    }
    else
    {
        "twilightforest.mob.naga.rattle"
    };
    let pitch: f32 = (ids::fraction(world) - ids::fraction(world)) * 0.2 + 1.0;
    world.playSound(info.pos, sound, 1.0, pitch);
}

pub fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    let model: EntityModelSpec = EntityModelSpec { shadow: 1.0, ..tf_naga::model() };
    let hard: i32 = 200;
    ctx.entities().register(
        NAME,
        EntitySpec
        {
            template: Identifier::parse("minecraft:monster").ok(),
            texture: Some("twilightforest:model/nagahead".to_owned()),
            size: Some([1.75, 3.0]),
            health: Some(hard),
            name: Some(LABEL.to_owned()),
            sounds: Some(EntitySounds
            {
                // EntityTFNaga.getLivingSound mixes in a rattle, voiced from onAi
                living: None,
                hurt: Some("twilightforest.mob.naga.hurt".to_owned()),
                death: Some("twilightforest.mob.naga.hurt".to_owned()),
            }),
            model: Some(model.clone()),
            persistent: true,
            ..EntitySpec::default()
        },
        Some(Rc::new(Naga)),
    )?;
    let segmentHealth: i32 = 1000;
    ctx.entities().register(
        SEGMENT,
        EntitySpec
        {
            template: Identifier::parse("minecraft:monster").ok(),
            texture: Some("twilightforest:model/nagasegment".to_owned()),
            size: Some([1.8, 1.8]),
            health: Some(segmentHealth),
            name: Some(LABEL.to_owned()),
            model: Some(model),
            persistent: true,
            ..EntitySpec::default()
        },
        Some(Rc::new(Segment)),
    )?;
    Ok(())
}
