use super::{dropAt, killer, showBar};
use crate::entities::ai::{eyes, face, target};
use crate::entities::shots::{self, LICH_BOLT, LICH_BOMB};
use crate::models::{Program, bipedPose, tf_lich};
use crate::names::item;
use crate::progress::{self, Gate};
use b173_api::math::{distance, floor};
use b173_api::prelude::*;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;

// EntityTFLich: a shielded caster that splits into shadow clones, then calls up zombie minions, then fights hand to
// hand once both run out
pub(crate) const NAME: &str = "twilight_lich";
pub(crate) const MINION: &str = "lich_minion";
const LABEL: &str = "Twilight Lich";

mod lich
{
    pub const HEALTH: i32 = 100;
    pub const SHIELD: i32 = 5;
    pub const MINIONS: i32 = 9;
    pub const MAX_CLONES: usize = 2;
    pub const MAX_MINIONS: usize = 3;
    pub const SEARCH: f64 = 32.0;
    pub const RANGE: f64 = 20.0;
    pub const MELEE: f64 = 2.0;
    pub const BITE: i32 = 6;
    // the attack timer the phases run on, EntityLiving.attackTime
    pub const CAST_PHASE_ONE: i32 = 100;
    pub const CAST_PHASE_TWO: i32 = 60;
    pub const MELEE_COOLDOWN: i32 = 20;
    pub const CLONE_AT: i32 = 60;
    pub const POP_EVERY: i32 = 15;
    // a hit has to hurt more than this to crack the shield
    pub const SHIELD_CRACK: i32 = 2;
    pub const BOLT_SPEED: f64 = 0.5;
    pub const BOMB_SPEED: f64 = 0.35;
    pub const LOS_TRIES: i32 = 100;
    pub const LOS_SPREAD: [f64; 3] = [16.0, 8.0, 16.0];
    pub const HAND_AHEAD: f64 = 0.65;
    pub const HAND_HEIGHT: f64 = 0.82;
    pub const SPARKLE_HEIGHT: f64 = 0.94;
    pub const CLONE_SHADE: f32 = 0.33;
    // onLivingUpdate's sparkles, up to eight a tick on a client whose attack timer stays at zero
    pub const SPARKLES: i32 = 8;
    pub const SPARKLE_GLOW: f32 = 0.967;
    pub const SPARKLE_JITTER: f64 = 0.025;
    pub const BOLT_TINT: [f32; 3] = [0.37, 0.99, 0.89];
    pub const BOMB_TINT: [f32; 3] = [0.99, 0.47, 0.0];
    pub const CLONE_ALPHA: f32 = 0.8;
}

// the lich's synced state, its shield shows on the client
#[derive(Serialize, Deserialize, Default, Clone, Copy, PartialEq)]
struct Lich
{
    m_clone: bool,
    m_master: i32,
    m_shield: i32,
    m_minions: i32,
    // a fresh lich starts with its whole shield and all its minions to call
    m_started: bool,
    // EntityTFLich.getNextAttackType, a bomb next tints the sparkles in its hand orange
    m_bomb: bool,
}

#[derive(Default)]
struct Brain
{
    m_attack: i32,
    m_bomb: bool,
}

thread_local!
{
    static BRAINS: RefCell<HashMap<(i32, i32), Brain>> = RefCell::new(HashMap::new());
}

fn state(entity: Entity) -> Lich
{
    let lich: Lich = entity.data();
    if lich.m_started || lich.m_clone
    {
        return lich;
    }

    Lich { m_shield: lich::SHIELD, m_minions: lich::MINIONS, m_started: true, ..lich }
}

fn save(entity: Entity, lich: Lich)
{
    let _ = entity.setData(&lich);

    if entity.synced::<Lich>() != lich
    {
        let _ = entity.setSynced(&lich);
    }
}

// EntityTFLich.getPhase
fn phase(lich: &Lich) -> i32
{
    if lich.m_clone || lich.m_shield > 0
    {
        1
    }
    else if lich.m_minions > 0
    {
        2
    }
    else
    {
        3
    }
}

fn mine(world: World, info: &EntityInfo, kind: &str, master: i32) -> Vec<EntityInfo>
{
    world
        .entitiesNear(info.pos, lich::SEARCH)
        .into_iter()
        .filter(|other|
        {
            other.kind.as_ref().is_some_and(|found| found.namespace() == "twilightforest" && found.path() == kind)
        })
        .filter(|other| Entity { world, id: other.id }.data::<Lich>().m_master == master)
        .collect()
}

// EntityTFLich.findVecInLOSOf, a standing spot the target can see
fn spotSeenBy(world: World, target: &EntityInfo) -> Option<[f64; 3]>
{
    let sight: [f64; 3] = eyes(Entity { world, id: target.id }, target);

    for _ in 0..lich::LOS_TRIES
    {
        let spread =
            |axis: usize| (crate::math::fraction(world) - crate::math::fraction(world)) as f64 * lich::LOS_SPREAD[axis];
        let [x, mut y, z] = [target.pos[0] + spread(0), target.pos[1] + spread(1), target.pos[2] + spread(2)];
        while y > 0.0 && !World::blockInfo(world.block(BlockPos::new(floor(x), floor(y) - 1, floor(z)))).solid
        {
            y -= 1.0;
        }

        let feet: BlockPos = BlockPos::new(floor(x), floor(y), floor(z));
        let open: bool = world.isAir(feet) && world.isAir(feet.offset(0, 1, 0)) && world.isAir(feet.offset(0, 2, 0));
        if y > 0.0 && open && world.rayTrace(sight, [x, y, z], false).is_none()
        {
            return Some([x, y, z]);
        }
    }

    None
}

fn trail(world: World, from: [f64; 3], to: [f64; 3], particle: &str)
{
    let steps: i32 = 32;
    for step in 0..steps
    {
        let part: f64 = f64::from(step) / f64::from(steps - 1);
        world.particle(particle, [0, 1, 2].map(|axis| from[axis] + (to[axis] - from[axis]) * part), [0.0; 3]);
    }
}

// EntityTFLich.teleportToSightOfEntity
fn teleport(entity: Entity, info: &EntityInfo, goal: &EntityInfo)
{
    let world: World = entity.world;
    let Some(dest) = spotSeenBy(world, goal)
    else
    {
        return;
    };

    let _ = entity.setPosition(dest);

    trail(world, info.pos, dest, "portal");
    world.playSound(info.pos, "mob.endermen.portal", 1.0, 1.0);
    face(entity, dest, goal.pos);
}

// EntityTFLich.onLivingUpdate, spell sparkles in the casting hand tinted by the next attack
fn sparkle(entity: Entity, info: &EntityInfo)
{
    let world: World = entity.world;
    let lich: Lich = entity.synced();
    let yaw: f64 = entity.field(EntityField::BodyYaw).to_radians();
    let height: f64 = entity.field(EntityField::Height);
    let hand: [f64; 3] = [
        info.pos[0] + yaw.cos() * lich::HAND_AHEAD,
        info.pos[1] + height * lich::SPARKLE_HEIGHT,
        info.pos[2] + yaw.sin() * lich::HAND_AHEAD,
    ];
    let tint: [f32; 3] = if lich.m_bomb
    {
        lich::BOMB_TINT
    }
    else
    {
        lich::BOLT_TINT
    };
    let color: [f64; 3] = tint.map(|channel| f64::from(channel * lich::SPARKLE_GLOW));

    for _ in 0..world.random(lich::SPARKLES)
    {
        let jitter = || f64::from(crate::math::fraction(world) - crate::math::fraction(world)) * lich::SPARKLE_JITTER;
        let at: [f64; 3] = [hand[0] + jitter(), hand[1] + jitter(), hand[2] + jitter()];
        crate::particles::spawn(world, "mobSpell", at, color);
    }
}

fn launch(entity: Entity, info: &EntityInfo, goal: &EntityInfo, bomb: bool)
{
    let world: World = entity.world;
    let yaw: f64 = f64::from(info.yaw + 90.0).to_radians();
    let height: f64 = entity.field(EntityField::Height);
    let from: [f64; 3] = [
        info.pos[0] + yaw.cos() * lich::HAND_AHEAD,
        info.pos[1] + height * lich::HAND_HEIGHT,
        info.pos[2] + yaw.sin() * lich::HAND_AHEAD,
    ];

    let aim: [f64; 3] =
        [goal.pos[0], goal.pos[1] + Entity { world, id: goal.id }.field(EntityField::Height) / 2.0, goal.pos[2]];

    let (shot, speed): (&str, f64) = if bomb
    {
        (LICH_BOMB, lich::BOMB_SPEED)
    }
    else
    {
        (LICH_BOLT, lich::BOLT_SPEED)
    };
    let pitch: f32 = (crate::math::fraction(world) - crate::math::fraction(world)) * 0.2 + 1.0;
    world.playSound(info.pos, "mob.ghast.fireball", 1.0, pitch);

    let _ = shots::fire(world, shot, from, aim, speed, 1.0, Some(entity.id));
}

// EntityTFLich.popNearbyMob, a plain monster in sight is torn apart
fn popNearby(entity: Entity, info: &EntityInfo)
{
    let world: World = entity.world;
    let poppable: [&str; 5] = ["skeleton", "zombie", "spider", "creeper", "swarm_spider"];

    let victim: Option<EntityInfo> = world.entitiesNear(info.pos, lich::SEARCH).into_iter().find(|other|
    {
        other.kind.as_ref().is_some_and(|kind| poppable.contains(&kind.path()))
            && world.rayTrace(eyes(entity, info), other.pos, false).is_none()
    });
    if let Some(victim) = victim
    {
        trail(world, victim.pos, info.pos, "reddust");
        Entity { world, id: victim.id }.kill();
    }
}

fn caster(entity: Entity, info: &EntityInfo, lich: &mut Lich, brain: &mut Brain, goal: &EntityInfo)
{
    let world: World = entity.world;
    let gap: f64 = distance(info.pos, goal.pos);
    let seen: bool = world.rayTrace(eyes(entity, info), eyes(Entity { world, id: goal.id }, goal), false).is_none();

    if !lich.m_clone && brain.m_attack % lich::POP_EVERY == 0
    {
        popNearby(entity, info);
    }

    match phase(lich)
    {
        1 =>
        {
            if brain.m_attack == lich::CLONE_AT
            {
                teleport(entity, info, goal);

                if !lich.m_clone
                    && mine(world, info, NAME, entity.id).len() < lich::MAX_CLONES
                    && let Some(spot) = spotSeenBy(world, goal)
                    && let Some(clone) =
                        crate::entities::spawnAt(world, &format!("twilightforest:{NAME}"), spot, info.yaw)
                {
                    save(clone, Lich { m_clone: true, m_master: entity.id, ..Lich::default() });
                    clone.setField(EntityField::TargetId, f64::from(goal.id));
                    trail(world, info.pos, spot, "portal");
                }
            }

            if seen && brain.m_attack == 0 && gap < lich::RANGE
            {
                launch(entity, info, goal, brain.m_bomb);
                brain.m_bomb = world.random(3) == 0;
                brain.m_attack = lich::CAST_PHASE_ONE;
            }
        }

        2 =>
        {
            for clone in mine(world, info, NAME, entity.id)
            {
                Entity { world, id: clone.id }.kill();
            }

            if brain.m_attack % lich::POP_EVERY == 0
                && lich.m_minions > 0
                && mine(world, info, MINION, entity.id).len() < lich::MAX_MINIONS
                && let Some(spot) = spotSeenBy(world, goal)
                && let Some(minion) =
                    crate::entities::spawnAt(world, &format!("twilightforest:{MINION}"), spot, info.yaw)
            {
                let _ = minion.setData(&Lich { m_master: entity.id, ..Lich::default() });

                minion.setField(EntityField::TargetId, f64::from(goal.id));
                world.playSound(spot, "random.pop", 1.0, 2.0);
                trail(world, eyes(entity, info), spot, "smoke");
                lich.m_minions -= 1;
            }

            if brain.m_attack == 0
            {
                if gap < lich::MELEE
                {
                    crate::combat::hit(
                        Entity { world, id: goal.id },
                        lich::BITE,
                        Some(entity.id),
                        crate::combat::kind::MOB,
                    );
                    brain.m_attack = lich::MELEE_COOLDOWN;
                }
                else if gap < lich::RANGE && seen
                {
                    launch(entity, info, goal, brain.m_bomb);
                    brain.m_bomb = world.random(2) == 0;
                    brain.m_attack = lich::CAST_PHASE_TWO;
                }
                else
                {
                    teleport(entity, info, goal);
                    brain.m_attack = lich::MELEE_COOLDOWN;
                }
            }
        }

        _ =>
        {
            let height: f64 = entity.field(EntityField::Height);
            if brain.m_attack <= 0 && gap < lich::MELEE && goal.pos[1] < info.pos[1] + height
            {
                crate::combat::hit(
                    Entity { world, id: goal.id },
                    lich::BITE,
                    Some(entity.id),
                    crate::combat::kind::MOB,
                );
                brain.m_attack = lich::MELEE_COOLDOWN;
            }
        }
    }
}

struct TwilightLich;

#[entity_behavior]
impl EntityBehavior for TwilightLich
{
    fn onTick(&self, entity: Entity, info: &EntityInfo)
    {
        let world: World = entity.world;
        if !world.isAuthority()
        {
            sparkle(entity, info);
            return;
        }

        let mut lich: Lich = state(entity);
        if lich.m_clone && (Entity { world, id: lich.m_master }).info().is_none_or(|master| !master.alive)
        {
            entity.kill();
            return;
        }

        BRAINS.with(|brains|
        {
            let mut brains: RefMut<HashMap<(i32, i32), Brain>> = brains.borrow_mut();
            let brain: &mut Brain = brains.entry((world.dimension, entity.id)).or_default();
            brain.m_attack = (brain.m_attack - 1).max(0);

            if let Some(goal) = target(entity)
            {
                caster(entity, info, &mut lich, brain, &goal);
            }

            lich.m_bomb = brain.m_bomb;
        });

        save(entity, lich);

        if !lich.m_clone
        {
            showBar(entity, LABEL, info);
        }
    }

    // EntityTFLich.attackEntityFrom, clones only fizz, the shield takes the hard hits one by one
    fn onHurt(&self, entity: Entity, source: Option<i32>, amount: i32) -> Option<i32>
    {
        let world: World = entity.world;
        let mut lich: Lich = state(entity);
        let pitch: f32 = ((crate::math::fraction(world) - crate::math::fraction(world)) * 0.7 + 1.0) * 2.0;
        let info: Option<EntityInfo> = entity.info();
        let at: [f64; 3] = info.as_ref().map_or([0.0; 3], |info| info.pos);

        if lich.m_clone
        {
            world.playSound(at, "random.fizz", 1.0, pitch);
            return None;
        }

        let byLich: bool = source
            .and_then(|id| Entity { world, id }.info())
            .is_some_and(|attacker| attacker.kind.as_ref().is_some_and(|kind| kind.path() == NAME));

        if byLich
        {
            return None;
        }

        if lich.m_shield > 0
        {
            world.playSound(at, "random.break", 1.0, pitch);

            if source.is_some() && amount > lich::SHIELD_CRACK
            {
                lich.m_shield -= 1;
                save(entity, lich);
            }

            if let Some(attacker) =
                source.filter(|id| Entity { world, id: *id }.info().is_some_and(|attacker| attacker.player.is_some()))
            {
                entity.setField(EntityField::TargetId, f64::from(attacker));
            }

            return None;
        }

        if (phase(&lich) < 3 || world.random(4) == 0)
            && let (Some(info), Some(goal)) = (info, target(entity))
        {
            teleport(entity, &info, &goal);
        }

        Some(amount)
    }

    // EntityTFLich.dropFewItems and onDeath, beta has no ender pearls or enchanted gold
    fn onDeath(&self, entity: Entity, source: Option<i32>)
    {
        let world: World = entity.world;
        BRAINS.with(|brains| brains.borrow_mut().remove(&(world.dimension, entity.id)));

        let lich: Lich = state(entity);
        if lich.m_clone || !world.isAuthority()
        {
            return;
        }
        let Some(info) = entity.info()
        else
        {
            return;
        };

        let scepters: [&str; 3] = [item::SCEPTER_ZOMBIE, item::SCEPTER_LIFE_DRAIN, item::SCEPTER_TWILIGHT];
        dropAt(world, info.pos, scepters[world.random(3) as usize], 1);

        let gold: [&str; 5] = [
            "minecraft:golden_sword",
            "minecraft:golden_helmet",
            "minecraft:golden_chestplate",
            "minecraft:golden_leggings",
            "minecraft:golden_boots",
        ];

        for _ in 0..world.random(3) + 2
        {
            dropAt(world, info.pos, gold[world.random(5) as usize], 1);
        }

        dropAt(world, info.pos, "minecraft:bone", world.random(5) + 5);
        dropAt(world, info.pos, item::LICH_TROPHY, 1);
        super::conquer(world, info.pos);

        if let Some(player) = killer(world, source)
        {
            progress::grant(player, Gate::Hunter);
            progress::grant(player, Gate::KillLich);
        }
    }

    // RenderTFLich, a clone drawn dark and see through
    fn pose(&self, input: &PoseInput) -> Pose
    {
        let lich: Lich = b173_api::fromBytes(&input.synced).unwrap_or_default();
        let mut body: Program = Program::new();

        if lich.m_clone
        {
            body.blend(true).color(lich::CLONE_SHADE, lich::CLONE_SHADE, lich::CLONE_SHADE, lich::CLONE_ALPHA);
        }

        bipedPose(&mut body, &tf_lich::PIVOTS, &tf_lich::ROTATIONS, input);
        Pose { pre: Vec::new(), model: body.take() }
    }
}

struct Minion;

#[entity_behavior]
impl EntityBehavior for Minion
{
    // EntityTFLichMinion.onLivingUpdate, a minion falls when its lich is gone
    fn onTick(&self, entity: Entity, _info: &EntityInfo)
    {
        if !entity.world.isAuthority()
        {
            return;
        }

        let master: i32 = entity.data::<Lich>().m_master;
        if (Entity { world: entity.world, id: master }).info().is_none_or(|lich| !lich.alive)
        {
            entity.setField(EntityField::Health, 0.0);
            entity.kill();
        }
    }

    // EntityTFLichMinion.attackEntityFrom, a blow from its own lich spurs it on with speed five and strength two
    fn onHurt(&self, entity: Entity, source: Option<i32>, amount: i32) -> Option<i32>
    {
        let master: i32 = entity.data::<Lich>().m_master;
        if entity.world.isAuthority() && source == Some(master)
        {
            let (speed, strength, ticks): (i32, i32, i32) = (4, 1, 200);
            crate::effects::give(entity, crate::effects::Effect::Speed, speed, ticks);
            crate::effects::give(entity, crate::effects::Effect::Strength, strength, ticks);
        }

        Some(amount)
    }

    fn onDeath(&self, entity: Entity, source: Option<i32>)
    {
        if let Some(player) = killer(entity.world, source)
        {
            progress::grant(player, Gate::Hunter);
        }
    }

    // ModelTFLichMinion tints it pale green without the strength potion beta lacks
    fn pose(&self, input: &PoseInput) -> Pose { crate::models::minionPose(input, MINION_TINT) }
}

const MINION_TINT: [f32; 3] = [0.5, 1.0, 0.5];

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.entities().register(
        NAME,
        EntitySpec
        {
            template: Identifier::parse("minecraft:monster").ok(),
            texture: Some("twilightforest:model/twilightlich64".to_owned()),
            size: Some([1.1, 2.5]),
            health: Some(lich::HEALTH),
            speed: Some(0.5),
            name: Some(LABEL.to_owned()),
            sounds: Some(EntitySounds
            {
                living: Some("mob.ghast.moan".to_owned()),
                hurt: Some("mob.ghast.scream".to_owned()),
                death: Some("mob.ghast.death".to_owned()),
            }),
            model: Some(EntityModelSpec { shadow: 1.0, ..tf_lich::model() }),
            fire_immune: true,
            persistent: true,
            ..EntitySpec::default()
        },
        Some(Rc::new(TwilightLich)),
    )?;
    ctx.entities().register(
        MINION,
        EntitySpec
        {
            template: Identifier::parse("minecraft:zombie").ok(),
            texture: Some("minecraft:mob/zombie".to_owned()),
            size: Some([0.6, 1.8]),
            name: Some("Lich Minion".to_owned()),
            model: Some(crate::models::tf_lich_minion::model()),
            ..EntitySpec::default()
        },
        Some(Rc::new(Minion)),
    )?;
    Ok(())
}
