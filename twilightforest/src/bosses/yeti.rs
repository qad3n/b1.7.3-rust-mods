use super::{home, killer};
use crate::entities::ai::{eyes, face};
use crate::entities::{clear, store};
use crate::items::{put, texture};
use crate::math::look;
use crate::models::{Program, bipedPose, tf_yeti_alpha};
use crate::names::item;
use crate::progress::{self, Gate};
use b173_api::math::{blockOf, distance, flatLength, floor, length, sub};
use b173_api::prelude::*;
use std::rc::Rc;

// EntityTFYetiAlpha, the yeti cave's boss: ice bombs until hurt, then rampages that bring the ceiling down
pub(crate) const NAME: &str = "yeti_alpha";
const LABEL: &str = "Alpha Yeti";
pub(crate) const ICE_BOMB: &str = "ice_bomb";
const FALLING_ICE: &str = "falling_ice";

mod yeti
{
    pub const HEALTH: i32 = 200;
    // the 0.38 movement attribute on beta's monster scale
    pub const SPEED: f32 = 0.83;
    pub const SEARCH: f64 = 40.0;
    // EntityAIArrowAttack fires every 40 ticks from up to 40 blocks
    pub const SHOT_PERIOD: i32 = 40;
    pub const SHOT_RANGE: f64 = 40.0;
    // EntityAIStayNearHome keeps it within 30 of its spawn
    pub const HOME_RANGE: f64 = 30.0;
    // EntityAITFYetiRampage(10, 180) and EntityAITFYetiTired(100)
    pub const RAMPAGE_WAIT: i32 = 10;
    pub const RAMPAGE_TICKS: i32 = 180;
    pub const TIRED_TICKS: i32 = 100;
    pub const RAMPAGE_JUMP: f64 = 0.4;
    pub const COLLISIONS: i32 = 15;
    // EntityAITFThrowRider waits 40 ticks before it may carry someone again
    pub const THROW_REST: i32 = 40;
    pub const THROW_ODDS: i32 = 5;
    pub const THROW_SPEED: f64 = 2.0;
    pub const THROW_LIFT: f64 = 0.9;
    // hitNearbyEntities when a rampage jump lands
    pub const STOMP: i32 = 5;
    pub const STOMP_RANGE: f64 = 5.0;
    pub const STOMP_LIFT: f64 = 0.4;
    pub const DROPS_MIN: i32 = 6;
    pub const DROPS_SPREAD: i32 = 6;
    pub const WALK: f64 = 0.2;
}

mod bomb
{
    pub const SPEED: f64 = 0.75;
    pub const GRAVITY: f64 = 0.025;
    pub const DRAG: f64 = 0.99;
    pub const ZONE_TICKS: i32 = 80;
    pub const ZONE_PERIOD: i32 = 10;
    pub const ZONE_RANGE: f64 = 3.0;
    pub const TERRAIN: i32 = 3;
    // the yeti's own bombs vanish when they land within 10 blocks of it
    pub const OWN_RANGE: f64 = 10.0;
    pub const HIT_RADIUS: f64 = 0.3;
    pub const LIFETIME: i32 = 1200;
}

mod ice
{
    pub const HANG: i32 = 100;
    pub const FALL: f64 = 0.04;
    pub const DRAG: f64 = 0.98;
    pub const HURT: f64 = 10.0;
    pub const HURT_MAX: i32 = 30;
    pub const RANGE: f64 = 2.0;
    pub const SIZE: f32 = 2.98;
    pub const DROP_FROM: i32 = 3;
    pub const SEARCH_UP: i32 = 30;
    pub const RANDOM_RANGE: i32 = 30;
    pub const NEAR_RANGE: i32 = 15;
    pub const CHILL: i32 = 2;
    pub const CHILL_TICKS: i32 = 100;
    // EntityTFFallingIce.fall shatters into two hundred bits of ice around a spot two blocks up
    pub const SHARDS: i32 = 200;
    pub const SHARD_SPREAD: f64 = 3.0;
    pub const SHARD_RISE: f64 = 2.0;
}

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct Shown
{
    m_rampage: bool,
    m_tired: bool,
}

#[derive(Serialize, Deserialize, Default)]
struct Brain
{
    m_canRampage: bool,
    m_wait: i32,
    m_rampage: i32,
    m_tired: i32,
    m_shot: i32,
    m_collisions: i32,
    m_rest: i32,
    m_grounded: bool,
}

static BRAINS: EntityState<Brain> = EntityState::new("brain");

// EntityTFYetiAlpha.destroyBlocksInAABB, all but obsidian, end stone and bedrock
fn destroyBox(world: World, low: [f64; 3], high: [f64; 3])
{
    let (low, high): (BlockPos, BlockPos) = (blockOf(low), blockOf(high));
    for x in low.x..=high.x
    {
        for y in low.y..=high.y
        {
            for z in low.z..=high.z
            {
                let pos: BlockPos = BlockPos::new(x, y, z);
                let (block, meta): (i32, i32) = (world.block(pos), world.meta(pos));
                if block == 0 || blockTagged(block, "c:obsidians") || blockIs(block, "minecraft:bedrock")
                {
                    continue;
                }

                let _ = world.clear(pos);
                let _ = world.breakEffect(pos, block, meta);
            }
        }
    }
}

fn bounds(entity: Entity, info: &EntityInfo, grow: [f64; 3], lift: f64) -> ([f64; 3], [f64; 3])
{
    let half: f64 = entity.field(EntityField::Width) / 2.0;
    let height: f64 = entity.field(EntityField::Height);
    let low: [f64; 3] = [info.pos[0] - half - grow[0], info.pos[1] - grow[1] + lift, info.pos[2] - half - grow[2]];
    let high: [f64; 3] =
        [info.pos[0] + half + grow[0], info.pos[1] + height + grow[1] + lift, info.pos[2] + half + grow[2]];
    (low, high)
}

// EntityTFYetiAlpha.makeBlockFallAbove, the first block above the spot turns to ice that drops
fn dropAbove(world: World, x: i32, y: i32, z: i32)
{
    if world.block(BlockPos::new(x, y, z)) != 0
    {
        return;
    }

    let Some(up) = (1..ice::SEARCH_UP).find(|up| world.block(BlockPos::new(x, y + up, z)) != 0)
    else
    {
        return;
    };

    let pos: BlockPos = BlockPos::new(x, y + up, z);
    let (block, meta): (i32, i32) = (world.block(pos), world.meta(pos));
    let _ = world.place(pos, "minecraft:ice");
    let _ = world.breakEffect(pos, block, meta);
    let at: [f64; 3] = [f64::from(x) + 0.5, f64::from(y + up - ice::DROP_FROM), f64::from(z) + 0.5];
    match world.spawnEntity(&format!("twilightforest:{FALLING_ICE}"), at)
    {
        Ok(spawned) => crate::world::palette::withPlain("minecraft:ice", |state| spawned.setCarried(state)),
        Err(error) => warn(&format!("cannot drop ice: {}", error.0)),
    }
}

// EntityTFYetiAlpha.makeRandomBlockFall
fn dropRandom(world: World, info: &EntityInfo, eyes: f64, range: i32)
{
    let x: i32 = floor(info.pos[0]) + world.random(range) - world.random(range);
    let z: i32 = floor(info.pos[2]) + world.random(range) - world.random(range);
    dropAbove(world, x, floor(info.pos[1] + eyes), z);
}

fn target(entity: Entity, info: &EntityInfo) -> Option<EntityInfo>
{
    let current: Option<EntityInfo> = crate::entities::ai::target(entity);
    let found: Option<EntityInfo> =
        current.or_else(|| crate::entities::closestPlayer(entity.world, info, yeti::SEARCH));
    let id: f64 = found.as_ref().map_or(-1.0, |found| f64::from(found.id));
    entity.setField(EntityField::TargetId, id);
    found
}

// EntityTFYetiAlpha.attackEntityWithRangedAttack, an ice bomb lobbed at the target
fn throwBomb(world: World, from: [f64; 3], to: [f64; 3], owner: Option<i32>)
{
    let flat: f64 = flatLength(to[0] - from[0], to[2] - from[2]);
    let aim: [f64; 3] = [to[0], to[1] - 1.1 + flat * 0.2, to[2]];

    let Ok(bomb) = world.spawnEntity(&format!("twilightforest:{ICE_BOMB}"), from)
    else
    {
        warn("cannot throw an ice bomb");
        return;
    };

    let delta: [f64; 3] = sub(aim, from);
    let length: f64 = length(delta).max(1.0e-4);
    // RenderTFThrownIce draws the bomb as a block of ice
    crate::world::palette::withPlain("minecraft:ice", |state| bomb.setCarried(state));

    let _ = bomb.setMotion(delta.map(|axis| axis / length * bomb::SPEED));
    let _ = bomb.setData(&Bomb { m_owner: owner, ..Bomb::default() });
    let pitch: f32 = 1.0 / (crate::math::fraction(world) * 0.4 + 0.8);
    world.playSound(from, "random.bow", 1.0, pitch);
}

// EntityAITFThrowRider, whoever climbed on is flung forward
fn throwRider(entity: Entity, info: &EntityInfo, brain: &mut Brain)
{
    let Some(rider) = entity.rider()
    else
    {
        brain.m_rest += 1;
        return;
    };

    if brain.m_rest <= yeti::THROW_REST || entity.world.random(yeti::THROW_ODDS) > 0
    {
        return;
    }

    let rider: Entity = Entity { world: entity.world, id: rider };
    let _ = rider.mount(None);
    let aim: [f64; 3] = look(info.yaw, info.pitch);
    let _ = rider.setMotion([aim[0] * yeti::THROW_SPEED, yeti::THROW_LIFT, aim[2] * yeti::THROW_SPEED]);

    brain.m_rest = 0;
}

// EntityTFYetiAlpha.fall while rampaging, the landing knocks everyone around into the air
fn stomp(entity: Entity, info: &EntityInfo)
{
    let world: World = entity.world;
    world.playSound(info.pos, "random.bow", 1.0, 1.0 / (crate::math::fraction(world) * 0.4 + 0.8));

    for near in world.entitiesNear(info.pos, yeti::STOMP_RANGE)
    {
        if near.id == entity.id || near.health.is_none()
        {
            continue;
        }

        let victim: Entity = Entity { world, id: near.id };
        if crate::combat::hit(victim, yeti::STOMP, Some(entity.id), crate::combat::kind::MOB)
        {
            let _ = victim.setMotion([near.motion[0], near.motion[1] + yeti::STOMP_LIFT, near.motion[2]]);
        }
    }
}

// EntityAITFYetiRampage.updateTask
fn rampage(entity: Entity, info: &EntityInfo, goal: Option<&EntityInfo>, brain: &mut Brain)
{
    let world: World = entity.world;

    brain.m_rampage -= 1;

    if let Some(goal) = goal
    {
        face(entity, info.pos, goal.pos);
    }

    if entity.flag(EntityField::OnGround)
    {
        let _ = entity.setMotion([0.0, yeti::RAMPAGE_JUMP, 0.0]);
    }

    let (low, high): ([f64; 3], [f64; 3]) = bounds(entity, info, [1.0, 2.0, 1.0], 2.0);
    destroyBox(world, low, high);

    let eyes: f64 = entity.field(EntityField::Height) * 0.85;

    if brain.m_rampage % 20 == 0
    {
        dropRandom(world, info, eyes, ice::RANDOM_RANGE);
    }

    if brain.m_rampage % 40 == 0
        && let Some(goal) = goal
    {
        dropAbove(world, floor(goal.pos[0]), floor(goal.pos[1] + 1.62), floor(goal.pos[2]));
    }

    if brain.m_rampage < 40 && brain.m_rampage % 10 == 0
    {
        dropRandom(world, info, eyes, ice::NEAR_RANGE);
    }

    if brain.m_rampage <= 0
    {
        brain.m_tired = yeti::TIRED_TICKS;
        brain.m_canRampage = false;
        brain.m_wait = yeti::RAMPAGE_WAIT;
    }
}

// EntityAIArrowAttack with EntityAIStayNearHome and EntityAIWander
fn hunt(entity: Entity, info: &EntityInfo, goal: Option<&EntityInfo>, brain: &mut Brain)
{
    let world: World = entity.world;
    let homeAt: Option<[f64; 3]> = home(entity).map(|at| at.map(f64::from));
    let away: bool = homeAt.is_some_and(|at| distance(at, info.pos) > yeti::HOME_RANGE);
    let seen: bool = goal.is_some_and(|goal| world.rayTrace(eyes(entity, info), goal.pos, false).is_none());

    let walkTo: Option<[f64; 3]> = if away
    {
        homeAt
    }
    else
    {
        goal.filter(|goal| !seen || distance(info.pos, goal.pos) > yeti::SHOT_RANGE).map(|goal| goal.pos)
    };

    if let Some(to) = walkTo
    {
        let (dx, dz): (f64, f64) = (to[0] - info.pos[0], to[2] - info.pos[2]);
        let flat: f64 = flatLength(dx, dz).max(1.0e-4);
        let _ = entity.setMotion([dx / flat * yeti::WALK, info.motion[1], dz / flat * yeti::WALK]);

        face(entity, info.pos, to);
    }

    let Some(goal) = goal
    else
    {
        return;
    };

    face(entity, info.pos, goal.pos);
    brain.m_shot -= 1;
    if seen && brain.m_shot <= 0 && distance(info.pos, goal.pos) <= yeti::SHOT_RANGE
    {
        if !brain.m_canRampage
        {
            throwBomb(world, eyes(entity, info), eyes(Entity { world, id: goal.id }, goal), Some(entity.id));
        }

        brain.m_shot = yeti::SHOT_PERIOD;
    }
}

fn think(entity: Entity, info: &EntityInfo, brain: &mut Brain)
{
    let world: World = entity.world;
    let goal: Option<EntityInfo> = target(entity, info);

    if entity.flag(EntityField::Collided)
    {
        brain.m_collisions += 1;
    }

    if brain.m_collisions >= yeti::COLLISIONS
    {
        let (low, high): ([f64; 3], [f64; 3]) = bounds(entity, info, [0.0; 3], 0.0);
        destroyBox(world, low, high);
        brain.m_collisions = 0;
    }

    throwRider(entity, info, brain);

    let grounded: bool = entity.flag(EntityField::OnGround);
    if brain.m_rampage > 0 && grounded && !brain.m_grounded
    {
        stomp(entity, info);
    }

    brain.m_grounded = grounded;

    if brain.m_tired > 0
    {
        brain.m_tired -= 1;

        let _ = entity.setMotion([0.0, info.motion[1], 0.0]);
    }
    else if brain.m_rampage > 0
    {
        rampage(entity, info, goal.as_ref(), brain);
    }
    else
    {
        if goal.is_some() && brain.m_canRampage
        {
            brain.m_wait -= 1;
        }

        if brain.m_wait <= 0 && brain.m_canRampage
        {
            brain.m_rampage = yeti::RAMPAGE_TICKS;
        }
        else
        {
            hunt(entity, info, goal.as_ref(), brain);
        }
    }

    let shown: Shown = Shown { m_rampage: brain.m_rampage > 0, m_tired: brain.m_tired > 0 };
    if entity.synced::<Shown>().m_rampage != shown.m_rampage || entity.synced::<Shown>().m_tired != shown.m_tired
    {
        let _ = entity.setSynced(&shown);
    }
}

// the snow whirling round a rampaging yeti and the sweat of a tired one, drawn on each client
fn effects(entity: Entity, info: &EntityInfo)
{
    // drawn by each client itself, the loader would echo server particles to them
    if entity.world.isAuthority()
    {
        return;
    }

    let shown: Shown = entity.synced();
    let world: World = entity.world;
    let ticks: f64 = entity.field(EntityField::TicksExisted);

    if shown.m_rampage
    {
        let turn: f64 = ticks / 10.0;

        for i in 0..20
        {
            let angle: f64 = turn + f64::from(i) * 50.0;
            let at: [f64; 3] = [
                info.pos[0] + 3.0 * angle.cos(),
                info.pos[1] + (f64::from(i) + turn) % 5.0,
                info.pos[2] + 3.0 * angle.sin(),
            ];
            world.particle("snowballpoof", at, [0.0; 3]);
        }
    }

    if shown.m_tired
    {
        let width: f64 = entity.field(EntityField::Width);
        let eyes: f64 = entity.field(EntityField::Height) * 0.85;

        for _ in 0..20
        {
            let spread = || f64::from(crate::math::fraction(world)) - 0.5;
            let at: [f64; 3] =
                [info.pos[0] + spread() * width * 0.5, info.pos[1] + eyes, info.pos[2] + spread() * width * 0.5];
            world.particle("splash", at, [spread() * 0.75, 0.0, spread() * 0.75]);
        }
    }
}

struct YetiAlpha;

#[entity_behavior]
impl EntityBehavior for YetiAlpha
{
    fn onAi(&self, entity: Entity, info: &EntityInfo)
    {
        if !entity.world.isAuthority()
        {
            return;
        }

        let mut brain: Brain = BRAINS.get(entity);
        think(entity, info, &mut brain);
        store(&BRAINS, entity, &brain);
    }

    fn onTick(&self, entity: Entity, info: &EntityInfo) { effects(entity, info); }

    // EntityTFYetiAlpha.interact, a player climbs on its back
    fn onInteract(&self, entity: Entity, player: Player) -> bool
    {
        if !entity.world.isAuthority() || entity.rider().is_some_and(|rider| rider != player.id)
        {
            return false;
        }

        Entity { world: entity.world, id: player.id }.mount(Some(entity.id)).is_ok()
    }

    // EntityTFYetiAlpha.attackEntityFrom, shots bounce off until it has been hurt up close or while tired
    // EntityTFYetiAlpha.attackEntityFrom, shots bounce off until it has rampaged or tired
    fn onHurt(&self, entity: Entity, _source: Option<i32>, amount: i32) -> Option<i32>
    {
        let kind: Option<Identifier> = b173_api::world::damageType();
        let ranged: bool = kind.is_some_and(|kind| matches!(kind.path(), "arrow" | "thrown" | "fireball"));

        let mut brain: Brain = BRAINS.get(entity);
        if !brain.m_canRampage && brain.m_tired <= 0 && ranged
        {
            return None;
        }

        brain.m_canRampage = true;
        store(&BRAINS, entity, &brain);
        Some(amount)
    }

    fn onDeath(&self, entity: Entity, source: Option<i32>)
    {
        let world: World = entity.world;

        clear(&BRAINS, entity);

        let Some(info) = entity.info()
        else
        {
            return;
        };

        if world.isAuthority()
        {
            super::conquer(world, info.pos);
        }

        if let Some(player) = killer(world, source)
        {
            progress::grant(player, Gate::Hunter);
            progress::grant(player, Gate::Yeti);
        }
    }

    fn pose(&self, input: &PoseInput) -> Pose
    {
        let mut body: Program = Program::new();
        bipedPose(&mut body, &tf_yeti_alpha::PIVOTS, &tf_yeti_alpha::ROTATIONS, input);
        Pose { pre: Vec::new(), model: body.take() }
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct Bomb
{
    m_owner: Option<i32>,
    m_age: i32,
    m_zone: Option<i32>,
}

// EntityTFIceBomb.doTerrainEffect, water freezes, lava sets and snow settles around where it lands
fn chill(world: World, at: [f64; 3])
{
    let center: BlockPos = blockOf(at);

    for x in -bomb::TERRAIN..=bomb::TERRAIN
    {
        for y in -bomb::TERRAIN..=bomb::TERRAIN
        {
            for z in -bomb::TERRAIN..=bomb::TERRAIN
            {
                let pos: BlockPos = center.offset(x, y, z);
                let block: i32 = world.block(pos);
                let replaced: Option<&str> = match block
                {
                    _ if blockTagged(block, "minecraft:water") => Some("minecraft:ice"),
                    _ if blockTagged(block, "minecraft:lava") => Some("minecraft:obsidian"),
                    0 if World::blockInfo(world.block(pos.offset(0, -1, 0))).opaque => Some("minecraft:snow_layer"),
                    _ => None,
                };
                if let Some(replaced) = replaced
                {
                    let _ = world.place(pos, replaced);
                }
            }
        }
    }
}

// EntityTFIceBomb.hitNearbyEntities, yetis freeze solid and the rest take a hit
fn zone(entity: Entity, at: [f64; 3], owner: Option<i32>)
{
    let world: World = entity.world;
    for near in world.entitiesNear(at, bomb::ZONE_RANGE)
    {
        if near.health.is_none() || Some(near.id) == owner || near.id == entity.id
        {
            continue;
        }

        let victim: Entity = Entity { world, id: near.id };

        if near.kind.as_ref().is_some_and(|kind| kind.namespace() == "twilightforest" && kind.path() == "yeti")
        {
            victim.kill();

            let pos: BlockPos = blockOf(near.pos);
            let _ = world.place(pos, "minecraft:ice");
            let _ = world.place(pos.offset(0, 1, 0), "minecraft:ice");
        }
        else
        {
            // EntityTFIceBomb, a chill of five seconds at slowness three
            crate::combat::hit(victim, 1, owner, crate::combat::kind::MAGIC);
            crate::effects::give(victim, crate::effects::Effect::Slowness, ice::CHILL, ice::CHILL_TICKS);
        }
    }
}

struct IceBomb;

#[entity_behavior]
impl EntityBehavior for IceBomb
{
    fn onTick(&self, entity: Entity, info: &EntityInfo)
    {
        let world: World = entity.world;
        let mut state: Bomb = entity.data();
        if let Some(left) = state.m_zone
        {
            for _ in 0..20
            {
                let spread = || f64::from(crate::math::fraction(world) - crate::math::fraction(world)) * 3.0;
                world.particle(
                    "snowballpoof",
                    [info.pos[0] + spread(), info.pos[1] + spread(), info.pos[2] + spread()],
                    [0.0; 3],
                );
            }

            if !world.isAuthority()
            {
                return;
            }

            let _ = entity.setMotion([0.0; 3]);

            if left % bomb::ZONE_PERIOD == 0
            {
                zone(entity, info.pos, state.m_owner);
            }

            if left <= 1
            {
                entity.kill();
                return;
            }

            state.m_zone = Some(left - 1);

            let _ = entity.setData(&state);

            return;
        }

        world.particle("snowballpoof", info.pos, [0.0; 3]);

        if !world.isAuthority()
        {
            return;
        }

        state.m_age += 1;
        if state.m_age > bomb::LIFETIME
        {
            entity.kill();
            return;
        }

        let to: [f64; 3] = info.pos;
        let from: [f64; 3] = sub(to, info.motion);
        let struck: bool = world
            .entitiesAlong(from, to, bomb::HIT_RADIUS, state.m_owner)
            .iter()
            .any(|hit| hit.id != entity.id && hit.health.is_some())
            || world.rayTrace(from, to, false).is_some();

        if struck
        {
            let ownNear: bool =
                state.m_owner.and_then(|owner| Entity { world, id: owner }.info()).is_some_and(|owner|
                {
                    owner.kind.as_ref().is_some_and(|kind| kind.path() == NAME)
                        && distance(owner.pos, to) <= bomb::OWN_RANGE
                });

            if ownNear
            {
                entity.kill();
                return;
            }

            chill(world, to);
            state.m_zone = Some(bomb::ZONE_TICKS);

            let _ = entity.setData(&state);
            let _ = entity.setMotion([0.0; 3]);

            return;
        }

        let _ = entity.setMotion([
            info.motion[0] * bomb::DRAG,
            info.motion[1] * bomb::DRAG - bomb::GRAVITY,
            info.motion[2] * bomb::DRAG,
        ]);

        let _ = entity.setData(&state);
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct Fall
{
    m_ticks: i32,
    m_from: f64,
}

struct FallingIce;

#[entity_behavior]
impl EntityBehavior for FallingIce
{
    // EntityTFFallingIce.onUpdate, it hangs a moment, then drops through ice and stone until it lands
    fn onTick(&self, entity: Entity, info: &EntityInfo)
    {
        let world: World = entity.world;
        if !world.isAuthority()
        {
            for _ in 0..2
            {
                let spread = || f64::from(crate::math::fraction(world) - crate::math::fraction(world));
                crate::particles::spawn(
                    world,
                    "snowwarning",
                    [info.pos[0] + 2.0 * spread(), info.pos[1] - 3.0 + 3.0 * spread(), info.pos[2] + 2.0 * spread()],
                    [0.0, -1.0, 0.0],
                );
            }

            return;
        }

        let mut state: Fall = entity.data();
        if state.m_ticks == 0
        {
            state.m_from = info.pos[1];
        }

        state.m_ticks += 1;
        let fall: f64 = if state.m_ticks > ice::HANG
        {
            ice::FALL
        }
        else
        {
            0.0
        };
        let motion: [f64; 3] =
            [info.motion[0] * ice::DRAG, (info.motion[1] - fall) * ice::DRAG, info.motion[2] * ice::DRAG];
        let _ = entity.setMotion(motion);
        let below: BlockPos = blockOf([info.pos[0], info.pos[1] - 0.1, info.pos[2]]);
        let breakable = |block: i32|
        {
            ["minecraft:ice", "minecraft:base_stone_overworld", "c:glass_blocks"]
                .iter()
                .any(|tag| blockTagged(block, tag))
        };
        let half: i32 = 2;
        for x in -half..=half
        {
            for z in -half..=half
            {
                let pos: BlockPos = below.offset(x, 0, z);
                if breakable(world.block(pos))
                {
                    let _ = world.clear(pos);
                }
            }
        }

        let landed: bool = state.m_ticks > ice::HANG && world.block(below) != 0 && !breakable(world.block(below));

        if landed
        {
            let dropped: f64 = state.m_from - info.pos[1];
            let hurt: i32 = (dropped.ceil() as i32 - 1).max(0);
            if hurt > 0
            {
                for near in world.entitiesNear(info.pos, ice::RANGE + f64::from(ice::SIZE) / 2.0)
                {
                    let alpha: bool = near.kind.as_ref().is_some_and(|kind| kind.path() == NAME);
                    if near.health.is_some() && !alpha
                    {
                        let damage: i32 = ((f64::from(hurt) * ice::HURT).floor() as i32).min(ice::HURT_MAX);
                        Entity { world, id: near.id }.damage(damage, None);
                    }
                }
            }

            for _ in 0..ice::SHARDS
            {
                let spread =
                    || f64::from(crate::math::fraction(world) - crate::math::fraction(world)) * ice::SHARD_SPREAD;
                let at: [f64; 3] =
                    [info.pos[0] + spread(), info.pos[1] + ice::SHARD_RISE + spread(), info.pos[2] + spread()];
                crate::particles::crack(world, "minecraft:ice", at, [0.0; 3]);
            }

            // the anvil's step sound breaks as stone does in beta
            world.playSound(info.pos, "step.stone", 3.0, 0.5);
            world.playSound(info.pos, "random.glass", 3.0, 0.5);
            entity.kill();
            return;
        }

        let _ = entity.setData(&state);
    }

    fn pose(&self, _input: &PoseInput) -> Pose
    {
        let mut program: Program = Program::new();
        let scale: f32 = ice::SIZE;
        program.scale(scale, scale, scale);
        Pose { pre: program.take(), model: Vec::new() }
    }
}

// ItemTFIceBomb, thrown like a snowball
struct IceBombItem;

#[item_behavior]
impl ItemBehavior for IceBombItem
{
    fn onRightClick(&self, world: World, player: Player, stack: ItemStack) -> Use
    {
        let Some(info) = player.info()
        else
        {
            return Use::Pass;
        };

        if world.isAuthority()
        {
            let eye: [f64; 3] = [info.pos[0], info.pos[1], info.pos[2]];
            let aim: [f64; 3] = look(info.yaw, info.pitch);
            throwBomb(
                world,
                eye,
                [eye[0] + aim[0] * 10.0, eye[1] + aim[1] * 10.0, eye[2] + aim[2] * 10.0],
                Some(player.id),
            );
        }

        let left: i32 = stack.count - 1;
        Use::Done((left > 0).then_some(ItemStack { count: left, ..stack }))
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.entities().register(
        NAME,
        EntitySpec
        {
            template: Identifier::parse("minecraft:monster").ok(),
            texture: Some("twilightforest:model/yetialpha".to_owned()),
            size: Some([3.8, 5.0]),
            health: Some(yeti::HEALTH),
            speed: Some(yeti::SPEED),
            name: Some(LABEL.to_owned()),
            sounds: Some(EntitySounds
            {
                living: Some("twilightforest.mob.yeti.growl".to_owned()),
                hurt: Some("twilightforest.mob.yeti.hurt".to_owned()),
                death: Some("twilightforest.mob.yeti.death".to_owned()),
            }),
            drops: Some(vec![
                crate::entities::drop(item::ALPHA_FUR, [yeti::DROPS_MIN, yeti::DROPS_MIN + yeti::DROPS_SPREAD - 1]),
                crate::entities::drop(ICE_BOMB, [yeti::DROPS_MIN, yeti::DROPS_MIN + yeti::DROPS_SPREAD - 1]),
            ]),
            model: Some(EntityModelSpec { shadow: 1.75, ..tf_yeti_alpha::model() }),
            persistent: true,
            ..EntitySpec::default()
        },
        Some(Rc::new(YetiAlpha)),
    )?;
    ctx.entities().register(
        ICE_BOMB,
        EntitySpec
        {
            template: Identifier::parse("minecraft:moving_block").ok(),
            size: Some([1.0, 1.0]),
            tracking_range: Some(64),
            update_frequency: Some(2),
            send_velocity: Some(true),
            ..EntitySpec::default()
        },
        Some(Rc::new(IceBomb)),
    )?;
    ctx.entities().register(
        FALLING_ICE,
        EntitySpec
        {
            template: Identifier::parse("minecraft:moving_block").ok(),
            size: Some([ice::SIZE, ice::SIZE]),
            tracking_range: Some(64),
            update_frequency: Some(2),
            send_velocity: Some(true),
            ..EntitySpec::default()
        },
        Some(Rc::new(FallingIce)),
    )?;
    put(
        ctx,
        ICE_BOMB,
        ItemSpecBuilder::new().name("Ice Bomb").texture(&texture("icebomb")).maxStack(16).behavior(IceBombItem),
    )
}
