use super::{dropAt, killer, showBar};
use crate::entities::ai::{eyes, face};
use crate::ids::{self, item};
use crate::math::look;
use crate::models::{Program, bipedPose, tf_snow_queen};
use crate::progress::{self, Gate};
use b173_api::math::{blockOf, distance, floor, length, sub};
use b173_api::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// EntityTFSnowQueen, the aurora palace's boss: she hovers summoning ice crystals, drops on the player to smash the
// ice around them, then hovers breathing an ice beam, and seven ice shields circle her
pub(crate) const NAME: &str = "snow_queen";
const LABEL: &str = "Snow Queen";
const SHIELD: &str = "snow_queen_shield";
const CRYSTAL: &str = "twilightforest:ice_crystal";

const SHIELDS: usize = 7;

mod queen
{
    pub const HEALTH: i32 = 200;
    pub const SPEED: f32 = 0.5;
    pub const ATTACK: i32 = 7;
    pub const SEARCH: f64 = 40.0;
    pub const SUMMONS: i32 = 6;
    pub const MINIONS_AT_ONCE: usize = 4;
    pub const MINION_AGE: i32 = 600;
    pub const BEAM_DAMAGE_LIMIT: i32 = 25;
    pub const DROPS_MIN: i32 = 2;
    pub const DROPS_SPREAD: i32 = 3;
    // EntityAITFHoverSummon, HoverThenDrop and HoverBeam: how high and how far around the target she hovers
    pub const SUMMON_HEIGHT: f64 = 6.0;
    pub const SUMMON_RADIUS: f32 = 6.0;
    pub const DROP_HEIGHT: f64 = 6.0;
    pub const BEAM_HEIGHT: f64 = 3.0;
    pub const BEAM_RADIUS: f32 = 4.0;
    pub const HOVER_TICKS: i32 = 80;
    pub const DROP_TICKS: i32 = 20;
    pub const BEAM_TICKS: i32 = 100;
    pub const SEEK_TICKS: i32 = 80;
    pub const ARRIVED: f64 = 1.0;
    pub const PULL: f64 = 0.05;
    pub const PULL_UP: f64 = 0.1;
    pub const HOVER_LIFT: f64 = 0.05;
    pub const BEAM_LIFT: f64 = 0.02;
    pub const BEAM_SINK: f64 = 0.05;
    pub const SPOT_TRIES: i32 = 100;
    pub const REACH: f64 = 2.0;
    pub const COOLDOWN: i32 = 20;
    pub const SHIELD_LIFT: f64 = 0.4;
    pub const SHIELD_REACH: f64 = 0.6;
    pub const SHIELD_SPIN: f32 = 5.0;
    pub const SHIELD_STEP: f32 = 60.0;
    pub const SHIELD_Y: f64 = 0.1;
    pub const ICE_DROPS_MIN: i32 = 1;
    pub const ICE_DROPS_SPREAD: i32 = 4;
    pub const SNOW_DROPS_MIN: i32 = 5;
    pub const SNOW_DROPS_SPREAD: i32 = 5;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
enum Phase
{
    #[default]
    Summon,
    Drop,
    Beam,
}

#[derive(Default)]
struct Brain
{
    m_phase: Phase,
    m_summons: i32,
    m_drops: i32,
    m_maxDrops: i32,
    m_beamDamage: i32,
    m_spot: Option<[f64; 3]>,
    m_seek: i32,
    m_hover: i32,
    m_act: i32,
    m_floor: f64,
    m_minions: Vec<(i32, i32)>,
    m_shields: [Option<i32>; SHIELDS],
    m_cooldown: i32,
    m_ticks: i32,
    m_started: bool,
}

#[derive(Serialize, Deserialize, Default, Clone, Copy, PartialEq)]
struct Shown
{
    m_phase: Phase,
    m_breathing: bool,
}

thread_local!
{
    static BRAINS: RefCell<HashMap<(i32, i32), Brain>> = RefCell::new(HashMap::new());
}

impl Brain
{
    // EntityTFSnowQueen.setCurrentPhase
    fn enter(&mut self, world: World, phase: Phase)
    {
        self.m_phase = phase;
        self.m_spot = None;
        self.m_seek = 0;
        self.m_hover = 0;
        self.m_act = 0;

        match phase
        {
            Phase::Summon => self.m_summons = queen::SUMMONS,

            Phase::Drop =>
            {
                self.m_drops = 0;
                self.m_maxDrops = queen::DROPS_MIN + world.random(queen::DROPS_SPREAD);
            }

            Phase::Beam => self.m_beamDamage = 0,
        }
    }
}

fn sees(world: World, from: [f64; 3], to: [f64; 3]) -> bool { world.rayTrace(from, to, false).is_none() }

// the hover AIs' makeNewHoverSpot, which keeps trying until the spot is blocked and seen, as the source checks
fn hoverSpot(world: World, own: [f64; 3], target: &EntityInfo, height: f64, radius: f32) -> [f64; 3]
{
    let mut spot: [f64; 3] = target.pos;

    for _ in 0..queen::SPOT_TRIES
    {
        let spread = || f64::from((ids::fraction(world) - ids::fraction(world)) * radius);
        spot = [target.pos[0] + spread(), target.pos[1] + height, target.pos[2] + spread()];
        let blocked: bool = world.block(blockOf(spot)) != 0;

        if blocked && sees(world, own, spot) && sees(world, [target.pos[0], target.pos[1] + 1.62, target.pos[2]], spot)
        {
            break;
        }
    }

    spot
}

fn pull(entity: Entity, info: &EntityInfo, to: [f64; 3], lift: f64)
{
    let delta: [f64; 3] = sub(to, info.pos);
    let length: f64 = length(delta).max(1.0e-4);
    let motion: [f64; 3] = [
        info.motion[0] + delta[0] / length * queen::PULL,
        info.motion[1] + delta[1] / length * queen::PULL_UP + lift,
        info.motion[2] + delta[2] / length * queen::PULL,
    ];

    let _ = entity.setMotion(motion);
}

// EntityTFSnowQueen.summonMinionAt, a crystal set somewhere the target can see, gone after 30 seconds
fn summon(world: World, target: &EntityInfo, brain: &mut Brain)
{
    let mut spot: [f64; 3] = target.pos;

    for _ in 0..queen::SPOT_TRIES
    {
        let gaussian = |spread: f64| f64::from(ids::fraction(world) - ids::fraction(world)) * spread;
        let mut at: [f64; 3] =
            [target.pos[0] + gaussian(16.0), target.pos[1] + gaussian(8.0), target.pos[2] + gaussian(16.0)];

        while at[1] > 0.0 && world.block(BlockPos::new(floor(at[0]), floor(at[1]) - 1, floor(at[2]))) == 0
        {
            at[1] -= 1.0;
        }

        spot = at;

        if sees(world, [target.pos[0], target.pos[1] + 1.62, target.pos[2]], [at[0], at[1] + 1.0, at[2]])
        {
            break;
        }
    }

    if let Some(minion) = crate::entities::spawnAt(world, CRYSTAL, spot, 0.0)
    {
        minion.setField(EntityField::TargetId, f64::from(target.id));
        brain.m_minions.push((minion.id, queen::MINION_AGE));
    }

    brain.m_summons -= 1;
}

// EntityTFIceCrystal.setToDieIn30Seconds for the queen's own crystals
fn ageMinions(world: World, brain: &mut Brain)
{
    brain.m_minions.retain_mut(|(id, left)|
    {
        *left -= 1;

        let alive: bool = Entity { world, id: *id }.info().is_some_and(|info| info.alive);
        if alive && *left <= 0
        {
            Entity { world, id: *id }.kill();
        }

        alive && *left > 0
    });
}

// EntityTFSnowQueen.destroyBlocksInAABB, only ice gives way under her drop
fn smashIce(world: World, info: &EntityInfo)
{
    let ice: i32 = ids::blockId("minecraft:ice");
    let below: [i32; 3] = info.pos.map(floor);

    for x in -1..=1
    {
        for y in -1..=2
        {
            for z in -1..=1
            {
                let pos: BlockPos = BlockPos::new(below[0] + x, below[1] + y, below[2] + z);
                if world.block(pos) == ice
                {
                    let _ = world.setBlock(pos, 0, 0);
                    let _ = world.breakEffect(pos, ice, 0);
                }
            }
        }
    }
}

fn act(entity: Entity, info: &EntityInfo, target: &EntityInfo, brain: &mut Brain)
{
    let world: World = entity.world;
    let own: [f64; 3] = eyes(entity, info);

    face(entity, info.pos, target.pos);

    match brain.m_phase
    {
        // EntityAITFHoverSummon
        Phase::Summon =>
        {
            let spot: [f64; 3] = *brain
                .m_spot
                .get_or_insert_with(|| hoverSpot(world, own, target, queen::SUMMON_HEIGHT, queen::SUMMON_RADIUS));
            brain.m_seek += 1;

            if distance(info.pos, spot) <= queen::ARRIVED
            {
                if brain.m_summons > 0 && brain.m_minions.len() < queen::MINIONS_AT_ONCE
                {
                    summon(world, target, brain);
                }

                brain.m_spot = Some(hoverSpot(world, own, target, queen::SUMMON_HEIGHT, queen::SUMMON_RADIUS));
                brain.m_seek = 0;
            }

            if brain.m_seek > queen::SEEK_TICKS || !sees(world, own, spot)
            {
                brain.m_spot = None;
            }

            pull(entity, info, spot, queen::HOVER_LIFT);
        }
        // EntityAITFHoverThenDrop
        Phase::Drop =>
        {
            let spot: [f64; 3] =
                *brain.m_spot.get_or_insert_with(|| hoverSpot(world, own, target, queen::DROP_HEIGHT, 0.0));

            if brain.m_hover == 0 && brain.m_act == 0
            {
                brain.m_floor = target.pos[1] - 1.0;
            }

            if distance(info.pos, spot) <= queen::ARRIVED || brain.m_hover > 0
            {
                brain.m_hover += 1;
            }
            else
            {
                brain.m_seek += 1;
            }

            if brain.m_hover < queen::HOVER_TICKS
            {
                pull(entity, info, spot, queen::HOVER_LIFT);
            }
            else
            {
                brain.m_act += 1;

                if info.pos[1] > brain.m_floor
                {
                    smashIce(world, info);
                }

                if brain.m_act >= queen::DROP_TICKS
                {
                    brain.m_drops += 1;
                    brain.m_spot = None;
                    brain.m_hover = 0;
                    brain.m_act = 0;
                }
            }

            if brain.m_seek > queen::SEEK_TICKS
            {
                brain.m_spot = None;
                brain.m_seek = 0;
            }
        }
        // EntityAITFHoverBeam, the beam itself does nothing in 2.4.3 but show
        Phase::Beam =>
        {
            let mut spot: [f64; 3] = *brain
                .m_spot
                .get_or_insert_with(|| hoverSpot(world, own, target, queen::BEAM_HEIGHT, queen::BEAM_RADIUS));

            if brain.m_hover == 0 && brain.m_act == 0
            {
                brain.m_floor = target.pos[1];
            }

            if distance(info.pos, spot) <= queen::ARRIVED || brain.m_hover > 0
            {
                brain.m_hover += 1;
            }
            else
            {
                brain.m_seek += 1;
            }

            if brain.m_hover >= queen::HOVER_TICKS
            {
                brain.m_act += 1;
                spot[1] = (spot[1] - queen::BEAM_SINK).max(brain.m_floor);
                brain.m_spot = Some(spot);
            }

            if distance(info.pos, spot) > 0.5
            {
                pull(entity, info, spot, queen::BEAM_LIFT);
            }

            if brain.m_act >= queen::BEAM_TICKS || brain.m_seek >= queen::SEEK_TICKS
            {
                brain.m_spot = None;
                brain.m_hover = 0;
                brain.m_act = 0;
                brain.m_seek = 0;
            }
        }
    }

    brain.m_cooldown -= 1;
    if brain.m_cooldown <= 0 && distance(info.pos, target.pos) < queen::REACH
    {
        brain.m_cooldown = queen::COOLDOWN;
        crate::combat::hit(Entity { world, id: target.id }, queen::ATTACK, Some(entity.id), crate::combat::kind::MOB);
    }
}

// EntityTFSnowQueen.onUpdate, six shields circling at her waist and one under her feet that strike what they touch
fn shields(entity: Entity, info: &EntityInfo, brain: &mut Brain)
{
    let world: World = entity.world;

    for (index, shield) in brain.m_shields.iter_mut().enumerate()
    {
        let angle: f32 = queen::SHIELD_STEP * index as f32 + brain.m_ticks as f32 * queen::SHIELD_SPIN;

        let at: [f64; 3] = if index < SHIELDS - 1
        {
            let radians: f64 = f64::from(angle).to_radians();
            [info.pos[0] + radians.cos(), info.pos[1] + queen::SHIELD_Y, info.pos[2] + radians.sin()]
        }
        else
        {
            [info.pos[0], info.pos[1] - 1.0, info.pos[2]]
        };
        let live: Option<Entity> =
            shield.map(|id| Entity { world, id }).filter(|shield| shield.info().is_some_and(|info| info.alive));

        let Some(live) = live.or_else(||
        {
            let spawned: Entity = world.spawnEntity(&format!("twilightforest:{SHIELD}"), at).ok()?;
            spawned.setField(EntityField::Block, f64::from(ids::blockId("minecraft:ice")));
            Some(spawned)
        })
        else
        {
            continue;
        };

        *shield = Some(live.id);

        let _ = live.setPosition(at);
        let _ = live.setMotion([0.0; 3]);

        live.setField(EntityField::Yaw, f64::from(angle));

        for near in world.entitiesNear(at, queen::SHIELD_REACH)
        {
            if near.id == entity.id
                || near.health.is_none()
                || near.kind.as_ref().is_some_and(|kind| kind.path() == SHIELD)
            {
                continue;
            }

            let victim: Entity = Entity { world, id: near.id };
            if crate::combat::hit(victim, queen::ATTACK, Some(entity.id), crate::combat::kind::MOB)
            {
                let _ = victim.setMotion([near.motion[0], near.motion[1] + queen::SHIELD_LIFT, near.motion[2]]);

                world.playSound(info.pos, "mob.zombie.wood", 1.0, 1.0);
            }
        }
    }
}

fn think(entity: Entity, info: &EntityInfo, brain: &mut Brain)
{
    let world: World = entity.world;

    if !brain.m_started
    {
        brain.enter(world, Phase::Summon);
        brain.m_started = true;
    }

    brain.m_ticks += 1;
    ageMinions(world, brain);
    shields(entity, info, brain);

    // EntityTFSnowQueen.updateAITasks, the phase moves on
    match brain.m_phase
    {
        Phase::Summon if brain.m_summons == 0 && brain.m_minions.is_empty() => brain.enter(world, Phase::Drop),
        Phase::Drop if brain.m_drops >= brain.m_maxDrops => brain.enter(world, Phase::Beam),
        Phase::Beam if brain.m_beamDamage >= queen::BEAM_DAMAGE_LIMIT => brain.enter(world, Phase::Summon),
        _ =>
        {}
    }

    let target: Option<EntityInfo> =
        crate::entities::ai::target(entity).or_else(|| crate::entities::closestPlayer(world, info, queen::SEARCH));
    entity.setField(EntityField::TargetId, target.as_ref().map_or(-1.0, |target| f64::from(target.id)));

    if let Some(target) = target
    {
        act(entity, info, &target, brain);
    }

    let shown: Shown = Shown { m_phase: brain.m_phase, m_breathing: brain.m_phase == Phase::Beam && brain.m_act > 0 };
    if entity.synced::<Shown>() != shown
    {
        let _ = entity.setSynced(&shown);
    }
}

// EntityTFSnowQueen.onLivingUpdate's snow and warning flakes and the ice beam, drawn on the clients
fn sparkle(entity: Entity, info: &EntityInfo)
{
    // drawn by each client itself, the loader would echo server particles to them
    if entity.world.isAuthority()
    {
        return;
    }

    let world: World = entity.world;
    let shown: Shown = entity.synced();
    let height: f64 = entity.field(EntityField::Height) * 0.85;

    for _ in 0..3
    {
        let spread = |by: f32| f64::from((ids::fraction(world) - ids::fraction(world)) * by);
        world.particle(
            "snowballpoof",
            [info.pos[0] + spread(0.3), info.pos[1] + height + spread(0.5), info.pos[2] + spread(0.3)],
            [0.0; 3],
        );
    }

    if !shown.m_breathing
    {
        return;
    }

    let aim: [f64; 3] = look(info.yaw, info.pitch);
    let mouth: [f64; 3] = [info.pos[0] + aim[0] * 0.5, info.pos[1] + 1.7 + aim[1] * 0.5, info.pos[2] + aim[2] * 0.5];

    for _ in 0..10
    {
        let speed: f64 = 2.0 + f64::from(ids::fraction(world)) * 0.15;
        let spread: f64 = (2.0 + f64::from(ids::fraction(world)) * 2.5) * 0.0075;
        let motion: [f64; 3] =
            [aim[0], 0.0, aim[2]].map(|axis| (axis + (f64::from(ids::fraction(world)) - 0.5) * spread * 2.0) * speed);
        world.particle("snowshovel", mouth, motion);
    }
}

struct SnowQueen;

#[entity_behavior]
impl EntityBehavior for SnowQueen
{
    fn onAi(&self, entity: Entity, info: &EntityInfo)
    {
        if !entity.world.isAuthority()
        {
            return;
        }

        BRAINS.with(|brains|
        {
            think(entity, info, brains.borrow_mut().entry((entity.world.dimension, entity.id)).or_default())
        });

        showBar(entity, LABEL, info);
    }

    fn onTick(&self, entity: Entity, info: &EntityInfo) { sparkle(entity, info); }

    // EntityTFSnowQueen.attackEntityFrom, damage taken while beaming ends the beam phase
    fn onHurt(&self, entity: Entity, _source: Option<i32>, amount: i32) -> Option<i32>
    {
        BRAINS.with(|brains|
        {
            if let Some(brain) = brains.borrow_mut().get_mut(&(entity.world.dimension, entity.id))
                && brain.m_phase == Phase::Beam
            {
                brain.m_beamDamage += amount;
            }
        });

        Some(amount)
    }

    fn onDeath(&self, entity: Entity, source: Option<i32>)
    {
        let world: World = entity.world;
        if let Some(brain) = BRAINS.with(|brains| brains.borrow_mut().remove(&(world.dimension, entity.id)))
        {
            for id in brain.m_shields.into_iter().flatten()
            {
                Entity { world, id }.kill();
            }
        }

        if let Some(player) = killer(world, source)
        {
            progress::grant(player, Gate::Hunter);
            progress::grant(player, Gate::Glacier);
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

        let bow: &str = if world.random(2) == 0
        {
            item::TRIPLE_BOW
        }
        else
        {
            item::SEEKER_BOW
        };
        dropAt(world, info.pos, bow, 1);

        for _ in 0..queen::ICE_DROPS_MIN + world.random(queen::ICE_DROPS_SPREAD)
        {
            dropAt(world, info.pos, "minecraft:ice", 7);
        }

        for _ in 0..queen::SNOW_DROPS_MIN + world.random(queen::SNOW_DROPS_SPREAD)
        {
            dropAt(world, info.pos, "minecraft:snowball", 16);
        }

        super::conquer(world, info.pos);
    }

    fn pose(&self, input: &PoseInput) -> Pose
    {
        let mut body: Program = Program::new();
        bipedPose(&mut body, &tf_snow_queen::PIVOTS, &tf_snow_queen::ROTATIONS, input);
        Pose { pre: Vec::new(), model: body.take() }
    }
}

// EntityTFSnowQueenIceShield, placed by the queen every tick
struct IceShield;

#[entity_behavior]
impl EntityBehavior for IceShield
{
    fn onTick(&self, entity: Entity, _info: &EntityInfo)
    {
        if !entity.world.isAuthority()
        {
            return;
        }

        let owned: bool =
            BRAINS.with(|brains| brains.borrow().values().any(|brain| brain.m_shields.contains(&Some(entity.id))));

        if !owned
        {
            entity.kill();
        }
    }

    // RenderTFSnowQueenIceShield draws a small block of ice
    fn pose(&self, _input: &PoseInput) -> Pose
    {
        let mut program: Program = Program::new();
        let scale: f32 = 0.75;
        program.scale(scale, scale, scale);
        Pose { pre: program.take(), model: Vec::new() }
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.entities().register(
        NAME,
        EntitySpec
        {
            template: Identifier::parse("minecraft:monster").ok(),
            texture: Some("twilightforest:model/snowqueen".to_owned()),
            size: Some([0.7, 2.2]),
            health: Some(queen::HEALTH),
            speed: Some(queen::SPEED),
            name: Some(LABEL.to_owned()),
            sounds: Some(EntitySounds
            {
                living: Some("twilightforest.mob.ice.noise".to_owned()),
                hurt: Some("twilightforest.mob.ice.hurt".to_owned()),
                death: Some("twilightforest.mob.ice.death".to_owned()),
            }),
            drops: Some(vec![crate::entities::drop(item::SNOW_QUEEN_TROPHY, [1, 1])]),
            model: Some(EntityModelSpec { shadow: 0.6, ..tf_snow_queen::model() }),
            fire_immune: true,
            persistent: true,
            ..EntitySpec::default()
        },
        Some(Rc::new(SnowQueen)),
    )?;
    ctx.entities().register(
        SHIELD,
        EntitySpec
        {
            template: Identifier::parse("minecraft:moving_block").ok(),
            size: Some([0.75, 0.75]),
            tracking_range: Some(64),
            update_frequency: Some(1),
            ..EntitySpec::default()
        },
        Some(Rc::new(IceShield)),
    )?;
    Ok(())
}
