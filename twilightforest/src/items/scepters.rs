use super::magic::wearHeld;
use super::{put, texture};
use crate::entities::ai::{distance, face};
use crate::entities::shots::{self, WAND_BOLT};
use crate::ids::{self, item};
use crate::math::look;
use b173_api::prelude::*;
use std::rc::Rc;

// ItemTFTwilightWand, ItemTFScepterLifeDrain and ItemTFZombieWand, the lich's three scepters; beta has no held use so
// each click does one of the source's use ticks
const LOYAL_ZOMBIE: &str = "loyal_zombie";

mod scepter
{
    pub const CHARGES: i32 = 99;
    pub const ZOMBIES: i32 = 9;
    pub const REACH: f64 = 20.0;
    pub const BOLT_SPEED: f64 = 1.5;
    // ItemTFScepterLifeDrain: a weak first hit marks the target, a held target is drained, a shatter at 3 health
    pub const DRAIN: i32 = 1;
    pub const SIPHON: i32 = 3;
    pub const SHATTER: i32 = 3;
    // the source marks the target with twenty ticks of slowness three
    pub const HOLD_TICKS: i32 = 20;
    pub const HOLD_LEVEL: i32 = 2;
    pub const TRAIL: i32 = 32;
    pub const HEAL: f64 = 1.0;
}

mod zombie
{
    pub const HEALTH: i32 = 40;
    pub const SPEED: f32 = 0.6;
    pub const ATTACK: i32 = 7;
    // the strength the scepter grants, the zombie burns once it runs out
    pub const LIFETIME: i32 = 1200;
    pub const BURN: f64 = 100.0;
    pub const SEARCH: f64 = 16.0;
    pub const FOLLOW: f64 = 10.0;
    pub const NEAR: f64 = 2.0;
    pub const REACH: f64 = 2.0;
    pub const COOLDOWN: i32 = 20;
    // ModelTFLoyalZombie's glColor3f(0.25, 2, 0.25), clamped green
    pub const TINT: [f32; 3] = [0.25, 1.0, 0.25];
    pub const LEAP: f64 = 0.2;
    pub const WALK: f64 = 0.25;
    pub const STRENGTH: i32 = 1;
    pub const STRENGTH_TICKS: i32 = 1200;
}

fn eyes(info: &EntityInfo) -> [f64; 3] { [info.pos[0], info.pos[1], info.pos[2]] }

fn charged(stack: &ItemStack, charges: i32) -> bool { stack.damage < charges }

// ItemTFScepterLifeDrain.makeRedMagicTrail
fn trail(world: World, from: [f64; 3], to: [f64; 3])
{
    for i in 0..scepter::TRAIL
    {
        let step: f64 = f64::from(i) / f64::from(scepter::TRAIL - 1);
        let at: [f64; 3] = [0, 1, 2].map(|axis| from[axis] + (to[axis] - from[axis]) * step);
        world.particle("reddust", at, [0.0; 3]);
    }
}

// the living thing first in the player's line of sight
fn lookTarget(world: World, player: Player, info: &EntityInfo) -> Option<EntityInfo>
{
    let from: [f64; 3] = eyes(info);
    let aim: [f64; 3] = look(info.yaw, info.pitch);
    let to: [f64; 3] = [0, 1, 2].map(|axis| from[axis] + aim[axis] * scepter::REACH);
    world
        .entitiesAlong(from, to, 1.0, Some(player.id))
        .into_iter()
        .filter(|hit| hit.health.is_some())
        .min_by(|a, b| distance(a.pos, from).total_cmp(&distance(b.pos, from)))
}

struct TwilightWand;

#[item_behavior]
impl ItemBehavior for TwilightWand
{
    fn onRightClick(&self, world: World, player: Player, stack: ItemStack) -> Use
    {
        let Some(info) = player.info()
        else
        {
            return Use::Pass;
        };

        if !charged(&stack, scepter::CHARGES)
        {
            return Use::Pass;
        }
        let pitch: f32 = (ids::fraction(world) - ids::fraction(world)) * 0.2 + 1.0;
        world.playSound(info.pos, "mob.ghast.fireball", 1.0, pitch);
        if world.isAuthority()
        {
            let from: [f64; 3] = eyes(&info);
            let aim: [f64; 3] = look(info.yaw, info.pitch);
            let toward: [f64; 3] = [0, 1, 2].map(|axis| from[axis] + aim[axis]);
            shots::fire(world, WAND_BOLT, from, toward, scepter::BOLT_SPEED, 1.0, Some(player.id));
            wearHeld(player, 1);
        }
        Use::Done(player.held())
    }
}

// ItemTFScepterLifeDrain marks a target with a second of slowness three and drains one that is still slowed
fn held(victim: Entity) -> bool { victim.effectLevel("twilightforest:slowness").is_some() }

fn hold(victim: Entity)
{
    crate::effects::give(victim, crate::effects::Effect::Slowness, scepter::HOLD_LEVEL, scepter::HOLD_TICKS);
}

struct LifeDrain;

#[item_behavior]
impl ItemBehavior for LifeDrain
{
    fn onRightClick(&self, world: World, player: Player, stack: ItemStack) -> Use
    {
        let Some(info) = player.info()
        else
        {
            return Use::Pass;
        };

        if !charged(&stack, scepter::CHARGES)
        {
            return Use::Pass;
        }
        let Some(target) = lookTarget(world, player, &info)
        else
        {
            return Use::Pass;
        };

        let victim: Entity = Entity { world, id: target.id };
        let health: i32 = target.health.unwrap_or(0);
        trail(world, eyes(&info), target.pos);
        if world.isAuthority()
        {
            if !held(victim) && health >= 1
            {
                world.playSound(info.pos, "fire.ignite", 1.0, 1.0);
                crate::combat::hit(victim, scepter::DRAIN, Some(player.id), crate::combat::kind::MAGIC);
                let _ = victim.setMotion([0.0, target.motion[1], 0.0]);
                hold(victim);
            }
            else if health <= scepter::SHATTER
            {
                world.playSound(target.pos, "random.hurt", 1.0, 2.0);
                for _ in 0..50
                {
                    world.particle("explode", target.pos, [0.0; 3]);
                }
                crate::combat::hit(victim, health.max(1) * 10, Some(player.id), crate::combat::kind::MAGIC);
            }
            else
            {
                crate::combat::hit(victim, scepter::SIPHON, Some(player.id), crate::combat::kind::MAGIC);
                let _ = victim.setMotion([0.0, 0.2, 0.0]);
                hold(victim);
                let own: Entity = Entity { world, id: player.id };
                let max: f64 = own.field(EntityField::MaxHealth);
                own.setField(EntityField::Health, (own.field(EntityField::Health) + scepter::HEAL).min(max));
            }
            wearHeld(player, 1);
        }
        Use::Done(player.held())
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct Loyal
{
    m_owner: Option<i32>,
    m_age: i32,
    m_cooldown: i32,
}

struct ZombieWand;

#[item_behavior]
impl ItemBehavior for ZombieWand
{
    // ItemTFZombieWand.onItemRightClick, a loyal zombie where the player looks
    fn onRightClick(&self, world: World, player: Player, stack: ItemStack) -> Use
    {
        let Some(info) = player.info()
        else
        {
            return Use::Pass;
        };

        if !charged(&stack, scepter::ZOMBIES) || !world.isAuthority()
        {
            return Use::Pass;
        }
        let from: [f64; 3] = eyes(&info);
        let aim: [f64; 3] = look(info.yaw, info.pitch);
        let to: [f64; 3] = [0, 1, 2].map(|axis| from[axis] + aim[axis] * scepter::REACH);
        let Some(hit) = world.rayTrace(from, to, false)
        else
        {
            return Use::Pass;
        };

        if let Some(spawned) =
            crate::entities::spawnAt(world, &format!("twilightforest:{LOYAL_ZOMBIE}"), hit.point, info.yaw)
            && spawned.setData(&Loyal { m_owner: Some(player.id), ..Loyal::default() }).is_ok()
        {
            // ItemTFZombieWand, a minute of strength two
            crate::effects::give(spawned, crate::effects::Effect::Strength, zombie::STRENGTH, zombie::STRENGTH_TICKS);
            wearHeld(player, 1);
        }
        Use::Done(player.held())
    }
}

// EntityTFLoyalZombie: follows its summoner, fights the monsters near it and burns when its strength runs out
struct LoyalZombie;

#[entity_behavior]
impl EntityBehavior for LoyalZombie
{
    fn onAi(&self, entity: Entity, info: &EntityInfo)
    {
        let world: World = entity.world;
        if !world.isAuthority()
        {
            return;
        }

        let mut state: Loyal = entity.data();
        state.m_age += 1;
        state.m_cooldown -= 1;
        if state.m_age > zombie::LIFETIME
        {
            entity.setField(EntityField::FireTicks, zombie::BURN);
        }
        let owner: Option<EntityInfo> = state.m_owner.and_then(|id| Entity { world, id }.info());
        let foe: Option<EntityInfo> = world
            .entitiesNear(info.pos, zombie::SEARCH)
            .into_iter()
            .filter(|near|
            {
                near.id != entity.id && Some(near.id) != state.m_owner && near.player.is_none() && near.health.is_some()
            })
            .filter(|near| Entity { world, id: near.id }.field(EntityField::AttackStrength) > 0.0)
            .min_by(|a, b| distance(a.pos, info.pos).total_cmp(&distance(b.pos, info.pos)));
        let goal: Option<[f64; 3]> = match (&foe, &owner)
        {
            (Some(foe), _) => Some(foe.pos),
            (None, Some(owner)) if distance(owner.pos, info.pos) > zombie::FOLLOW / 2.0 => Some(owner.pos),
            _ => None,
        };
        if let Some(goal) = goal.filter(|goal| distance(*goal, info.pos) > zombie::NEAR)
        {
            let (dx, dz): (f64, f64) = (goal[0] - info.pos[0], goal[2] - info.pos[2]);
            let flat: f64 = (dx * dx + dz * dz).sqrt().max(1.0e-4);
            let up: f64 = if entity.flag(EntityField::Collided) && entity.flag(EntityField::OnGround)
            {
                0.42
            }
            else
            {
                info.motion[1]
            };
            let _ = entity.setMotion([dx / flat * zombie::WALK, up, dz / flat * zombie::WALK]);
            face(entity, info.pos, goal);
        }
        if let Some(foe) = foe
            && state.m_cooldown <= 0
            && distance(foe.pos, info.pos) < zombie::REACH
        {
            let victim: Entity = Entity { world, id: foe.id };
            if crate::combat::hit(victim, zombie::ATTACK, Some(entity.id), crate::combat::kind::MOB)
            {
                let _ = victim.setMotion([foe.motion[0], foe.motion[1] + zombie::LEAP, foe.motion[2]]);
            }
            state.m_cooldown = zombie::COOLDOWN;
        }
        let _ = entity.setData(&state);
    }

    fn pose(&self, input: &PoseInput) -> Pose { crate::models::minionPose(input, zombie::TINT) }
}

pub fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    let wand = |label: &str, look: &str, charges: i32|
    {
        ItemSpecBuilder::new().name(label).texture(&texture(look)).maxStack(1).maxDamage(charges)
    };
    put(
        ctx,
        item::SCEPTER_TWILIGHT,
        wand("Scepter of Twilight", "sceptertwilight", scepter::CHARGES).behavior(TwilightWand),
    )?;
    put(
        ctx,
        item::SCEPTER_LIFE_DRAIN,
        wand("Scepter of Life Draining", "scepterlifedrain", scepter::CHARGES).behavior(LifeDrain),
    )?;
    put(ctx, item::SCEPTER_ZOMBIE, wand("Zombie Scepter", "scepterzombie", scepter::ZOMBIES).behavior(ZombieWand))?;
    ctx.entities().register(
        LOYAL_ZOMBIE,
        EntitySpec
        {
            template: Identifier::parse("minecraft:neutral_monster").ok(),
            texture: Some("minecraft:mob/zombie".to_owned()),
            size: Some([0.6, 1.8]),
            health: Some(zombie::HEALTH),
            speed: Some(zombie::SPEED),
            name: Some("Loyal Zombie".to_owned()),
            sounds: Some(EntitySounds
            {
                living: Some("mob.zombie".to_owned()),
                hurt: Some("mob.zombiehurt".to_owned()),
                death: Some("mob.zombiedeath".to_owned()),
            }),
            model: Some(crate::models::tf_lich_minion::model()),
            ..EntitySpec::default()
        },
        Some(Rc::new(LoyalZombie)),
    )?;
    Ok(())
}
