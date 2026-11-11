use super::{BossBar, Home, bossName, pitch, say, unlock};
use crate::entities::fraction;
use crate::ids::blockId;
use crate::models::{Program, slider as model};
use crate::progress::{self, Award};
use b173_api::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

const NAME: &str = super::SLIDER;
const LOCKED: [&str; 2] = ["locked_carved_stone", "locked_sentry_stone"];
// BlockDungeonBase, the stone the slider never breaks; the locked copies are unbreakable anyway
const DUNGEON_STONE: [&str; 6] =
    ["carved_stone", "sentry_stone", "angelic_stone", "light_angelic_stone", "hellfire_stone", "light_hellfire_stone"];

mod texture
{
    pub const ASLEEP: &str = "aether:entity/boss/slider/slider_asleep";
    pub const ASLEEP_CRITICAL: &str = "aether:entity/boss/slider/slider_asleep_critical";
    pub const AWAKE: &str = "aether:entity/boss/slider/slider_awake";
    pub const AWAKE_CRITICAL: &str = "aether:entity/boss/slider/slider_awake_critical";
    pub const GLOW: &str = "aether:entity/boss/slider/slider_awake_glow";
    pub const GLOW_CRITICAL: &str = "aether:entity/boss/slider/slider_awake_critical_glow";
}

mod sound
{
    pub const MOVE: &str = "aether.aeboss.slider.move";
    pub const COLLIDE: &str = "aether.aeboss.slider.collide";
    pub const AWAKEN: &str = "aether.aeboss.slider.awaken";
    pub const DEATH: &str = "aether.aeboss.slider.death";
    pub const VOLUME: f32 = 2.5;
    pub const EXPLODE_VOLUME: f32 = 3.0;
}

mod slide
{
    // EntitySlider
    pub const HEALTH: i32 = 500;
    pub const CRITICAL: i32 = 80;
    pub const TOP_SPEED: f32 = 2.0;
    pub const ACCELERATION: f32 = 0.035;
    pub const CRITICAL_ACCELERATION: f32 = 0.07;
    pub const PAUSE: i32 = 12;
    pub const OVERSHOOT_PAUSE: i32 = 8;
    pub const CRITICAL_PAUSE: i32 = 4;
    pub const RISE_PAST: f64 = 0.35;
    pub const DROP_PAST: f64 = 0.25;
    pub const SIDE_PAST: f64 = 0.125;
    pub const VERTICAL_ODDS: i32 = 5;
    pub const CRUSH: i32 = 6;
    pub const BOUNCE: f64 = 0.35;
    pub const REACH: f64 = 6.0;
    pub const SLOWED: f32 = 0.75;
    pub const LOWEST_CRUSH: f64 = 4.0;
    pub const CRUSH_GRID: i32 = 5;
    pub const CRUSH_STEP: f64 = 0.75;
    pub const CRUSH_AHEAD: f64 = 1.5;
    pub const CHAT_COOLDOWN: i32 = 60;
    pub const HURT_TILT: f32 = 0.7;
    pub const HURT_SPREAD: f32 = 875.0;
    pub const HURT_FADE: f32 = 0.8;
    pub const HURT_DEGREES: f32 = -30.0;
    pub const KEY_LIFT: f64 = 0.5;
    pub const STONE_DROPS: i32 = 7;
    pub const STONE_SPREAD: i32 = 3;
    pub const SCALE: f32 = 2.0;
    pub const PIVOT: [f32; 3] = [0.0, 12.0, 0.0];
    pub const HOME: [i32; 3] = [8, 2, 8];
    pub const DOOR_SCATTER: f64 = 1.5;
    pub const DOOR_RISE: f64 = 1.75;
    pub const EXPLODE_PITCH: f32 = 0.625;
    pub const EXPLODE_LOW: f32 = 0.7;
}

#[derive(Clone, Copy, PartialEq, Default)]
enum Direction
{
    #[default]
    Up,
    Down,
    East,
    West,
    South,
    North,
}

// what clients see: the bar, whether it is awake or sliding, and its last hurt tilt
#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
struct Sync
{
    m_bar: BossBar,
    m_awake: bool,
    m_moving: bool,
    m_hurtAge: f32,
    m_hurtTilt: f32,
    m_hurtAxis: [f32; 2],
}

// the server's slide in progress, as the Java fields that never reach clients
#[derive(Default, Clone, Copy)]
struct Slide
{
    m_moveTime: i32,
    m_velocity: f32,
    m_direction: Direction,
    m_chatTime: i32,
}

thread_local! {
    static SLIDES: RefCell<HashMap<i32, Slide>> = RefCell::new(HashMap::new());
}

pub fn bar(boss: Entity) -> BossBar { boss.synced::<Sync>().m_bar }

fn slide(entity: Entity) -> Slide { SLIDES.with(|slides| slides.borrow().get(&entity.id).copied()).unwrap_or_default() }

fn setSlide(entity: Entity, state: Slide) { SLIDES.with(|slides| slides.borrow_mut().insert(entity.id, state)); }

fn sync(entity: Entity) -> Sync
{
    let mut state: Sync = entity.synced();
    if state.m_bar.m_name.is_empty() && entity.world.isAuthority()
    {
        state.m_bar = BossBar {
            m_name: bossName(entity.world, NAME),
            m_health: slide::HEALTH,
            m_maxHealth: slide::HEALTH,
            m_focus: None,
        };
        let _ = entity.setSynced(&state);
    }
    state
}

fn critical(state: &Sync) -> bool { state.m_bar.m_health <= slide::CRITICAL }

// EntitySlider.stop
fn stop(entity: Entity, state: &mut Sync, motion: &mut Slide)
{
    state.m_moving = false;
    motion.m_moveTime = slide::PAUSE;
    motion.m_direction = Direction::Up;
    motion.m_velocity = 0.0;
    let _ = entity.setMotion([0.0; 3]);
}

fn feet(entity: Entity) -> f64 { entity.field(EntityField::Bottom) }

// EntitySlider.destroyBlock: everything but dungeon stone and unbreakable blocks gives way
fn crushBlock(world: World, at: [f64; 3]) -> bool
{
    let pos: BlockPos = BlockPos::new(at[0] as i32, at[1] as i32, at[2] as i32);
    let block: i32 = world.block(pos);
    if block == 0 || World::blockInfo(block).hardness < 0.0 || DUNGEON_STONE.iter().any(|name| blockId(name) == block)
    {
        return false;
    }
    let meta: i32 = world.meta(pos);
    let _ = world.breakEffect(pos, block, meta);
    let _ = world.setBlock(pos, 0, 0);
    let wobble = || (fraction(world) - fraction(world)) * 0.375;
    world.particle("smoke", [at[0] + 0.5 + wobble(), at[1] + 0.5 + wobble(), at[2] + 0.5 + wobble()], [0.0; 3]);
    true
}

// the 5 by 5 face of the slider in the way it was going
fn crushAhead(world: World, info: &EntityInfo, bottom: f64, direction: Direction) -> bool
{
    let [x, y, z] = [info.pos[0] - 0.5, bottom + 0.75, info.pos[2] - 0.5];
    if y <= slide::LOWEST_CRUSH
    {
        return false;
    }
    let mut crushed: bool = false;
    for i in 0..slide::CRUSH_GRID * slide::CRUSH_GRID
    {
        let a: f64 = (i / slide::CRUSH_GRID - 2) as f64 * slide::CRUSH_STEP;
        let b: f64 = (i % slide::CRUSH_GRID - 2) as f64 * slide::CRUSH_STEP;
        let ahead: f64 = slide::CRUSH_AHEAD;
        let at: [f64; 3] = match direction
        {
            Direction::Up => [x + a, y + ahead, z + b],
            Direction::Down => [x + a, y - ahead, z + b],
            Direction::East => [x + ahead, y + a, z + b],
            Direction::West => [x - ahead, y + a, z + b],
            Direction::South => [x + a, y + b, z + ahead],
            Direction::North => [x + a, y + b, z - ahead],
        };
        crushed |= crushBlock(world, at);
    }
    crushed
}

// EntitySlider.reset: back to sleep on its spot, healed, with the door open
fn reset(entity: Entity, state: &mut Sync, motion: &mut Slide)
{
    stop(entity, state, motion);
    motion.m_moveTime = 0;
    openDoor(entity);
    state.m_awake = false;
    state.m_bar.m_focus = None;
    state.m_bar.m_health = slide::HEALTH;
    entity.setField(EntityField::Health, slide::HEALTH as f64);
    let home: Home = entity.data();
    let [x, y, z] = home.m_dungeon;
    let _ = entity.setPosition([(x + slide::HOME[0]) as f64, (y + slide::HOME[1]) as f64, (z + slide::HOME[2]) as f64]);
}

fn openDoor(entity: Entity)
{
    let mut home: Home = entity.data();
    let Some([start, end]) = home.m_door.take()
    else
    {
        return;
    };
    for x in start[0]..=end[0]
    {
        for y in start[1]..=end[1]
        {
            for z in start[2]..=end[2]
            {
                let _ = entity.world.setBlock(BlockPos::new(x, y, z), 0, 0);
            }
        }
    }
    let _ = entity.setData(&home);
}

// EntitySlider.closeDoor: walls up the first of the room's four doorways that is open
fn closeDoor(entity: Entity)
{
    let world: World = entity.world;
    let mut home: Home = entity.data();
    let [x, y, z] = home.m_dungeon;
    let doorways: [([i32; 3], [i32; 3], bool); 4] = [
        ([x + 15, y + 1, z + 6], [x + 15, y + 4, z + 9], true),
        ([x, y + 1, z + 6], [x, y + 4, z + 9], true),
        ([x + 6, y + 1, z + 15], [x + 9, y + 4, z + 15], false),
        ([x + 6, y + 1, z], [x + 9, y + 4, z], false),
    ];
    let open = |start: [i32; 3], end: [i32; 3]| {
        (start[0]..=end[0]).all(|bx| {
            (start[1]..=end[1]).all(|by| (start[2]..=end[2]).all(|bz| world.isAir(BlockPos::new(bx, by, bz))))
        })
    };
    let Some((start, end, alongZ)) = doorways.into_iter().find(|(start, end, _)| open(*start, *end))
    else
    {
        return;
    };
    let wall: i32 = blockId(LOCKED[0]);
    for height in y + 1..y + 8
    {
        for across in 5..11
        {
            let pos: BlockPos = if alongZ
            {
                BlockPos::new(start[0], height, z + across)
            }
            else
            {
                BlockPos::new(x + across, height, start[2])
            };
            let _ = world.setBlock(pos, wall, 0);
        }
    }
    home.m_door = Some([start, end]);
    let _ = entity.setData(&home);
}

// EntitySlider.updateEntityActionState: waits, picks the axis the player is furthest along and slides at them
fn act(entity: Entity, info: &EntityInfo, state: &mut Sync, motion: &mut Slide)
{
    let world: World = entity.world;
    let target: Option<EntityInfo> = state
        .m_bar
        .m_focus
        .and_then(|id| Entity { world, id }.info())
        .filter(|target| target.alive && target.health.unwrap_or(0) > 0);
    let Some(target) = target
    else
    {
        reset(entity, state, motion);
        return;
    };
    let targetEntity: Entity = Entity { world, id: target.id };
    let bottom: f64 = feet(entity);
    if state.m_moving
    {
        if entity.flag(EntityField::Collided)
        {
            if crushAhead(world, info, bottom, motion.m_direction)
            {
                let explode: f32 =
                    (slide::EXPLODE_PITCH + (fraction(world) - fraction(world)) as f32 * 0.2) * slide::EXPLODE_LOW;
                world.playSound(info.pos, "random.explode", sound::EXPLODE_VOLUME, explode);
                world.playSound(info.pos, sound::COLLIDE, sound::VOLUME, pitch(world));
            }
            stop(entity, state, motion);
            return;
        }
        if motion.m_velocity < slide::TOP_SPEED
        {
            motion.m_velocity += if critical(state)
            {
                slide::CRITICAL_ACCELERATION
            }
            else
            {
                slide::ACCELERATION
            };
        }
        let speed: f64 = motion.m_velocity as f64;
        let targetBottom: f64 = feet(targetEntity);
        let (velocity, past): ([f64; 3], bool) = match motion.m_direction
        {
            Direction::Up => ([0.0, speed, 0.0], bottom > targetBottom + slide::RISE_PAST),
            Direction::Down => ([0.0, -speed, 0.0], bottom < targetBottom - slide::DROP_PAST),
            Direction::East => ([speed, 0.0, 0.0], info.pos[0] > target.pos[0] + slide::SIDE_PAST),
            Direction::West => ([-speed, 0.0, 0.0], info.pos[0] < target.pos[0] - slide::SIDE_PAST),
            Direction::South => ([0.0, 0.0, speed], info.pos[2] > target.pos[2] + slide::SIDE_PAST),
            Direction::North => ([0.0, 0.0, -speed], info.pos[2] < target.pos[2] - slide::SIDE_PAST),
        };
        let _ = entity.setMotion(velocity);
        if past
        {
            stop(entity, state, motion);
            motion.m_moveTime = if critical(state)
            {
                slide::CRITICAL_PAUSE
            }
            else
            {
                slide::OVERSHOOT_PAUSE
            };
        }
        return;
    }
    if motion.m_moveTime > 0
    {
        motion.m_moveTime -= 1;
        if critical(state) && world.random(2) == 0
        {
            motion.m_moveTime -= 1;
        }
        let _ = entity.setMotion([0.0; 3]);
        return;
    }
    let a: f64 = (info.pos[0] - target.pos[0]).abs();
    let b: f64 = (bottom - feet(targetEntity)).abs();
    let c: f64 = (info.pos[2] - target.pos[2]).abs();
    motion.m_direction = match (a > c, info.pos[0] > target.pos[0], info.pos[2] > target.pos[2])
    {
        (true, true, _) => Direction::West,
        (true, false, _) => Direction::East,
        (false, _, true) => Direction::North,
        (false, _, false) => Direction::South,
    };
    if (b > a && b > c) || (b > slide::DROP_PAST && world.random(slide::VERTICAL_ODDS) == 0)
    {
        motion.m_direction = if info.pos[1] > target.pos[1]
        {
            Direction::Down
        }
        else
        {
            Direction::Up
        };
    }
    world.playSound(info.pos, sound::MOVE, sound::VOLUME, pitch(world));
    state.m_moving = true;
}

// EntitySlider.collideWithNearbyEntities and applyEntityCollision: a sliding slider crushes what it runs into
fn crushEntities(entity: Entity, info: &EntityInfo, state: &mut Sync, motion: &mut Slide)
{
    const REACH: f64 = 1.2;
    for other in entity
        .world
        .entitiesNear(info.pos, REACH)
        .into_iter()
        .filter(|other| other.id != entity.id && other.health.is_some())
    {
        let hit: Entity = Entity { world: entity.world, id: other.id };
        if !crate::damage::hit(hit, slide::CRUSH, Some(entity.id), crate::damage::kind::MOB)
        {
            continue;
        }
        // EntityLivingBase.addVelocity(motionY, 0.35, motionZ) as the source calls it
        let [x, y, z] = other.motion;
        let _ = hit.setMotion([x + y, y + slide::BOUNCE, z + z]);
        entity.world.playSound(info.pos, sound::COLLIDE, sound::VOLUME, pitch(entity.world));
        stop(entity, state, motion);
        return;
    }
}

fn chat(entity: Entity, player: Player, key: &str)
{
    let mut motion: Slide = slide(entity);
    if motion.m_chatTime <= 0
    {
        say(player, key);
        motion.m_chatTime = slide::CHAT_COOLDOWN;
        setSlide(entity, motion);
    }
}

fn isPickaxe(stack: &ItemStack) -> bool { stack.item.name().is_some_and(|name| name.path().ends_with("_pickaxe")) }

struct Slider;

#[entity_behavior]
impl EntityBehavior for Slider
{
    fn onAi(&self, entity: Entity, info: &EntityInfo)
    {
        entity.setField(EntityField::Yaw, 0.0);
        entity.setField(EntityField::Pitch, 0.0);
        if !entity.world.isAuthority()
        {
            return;
        }
        let before: Sync = sync(entity);
        let mut state: Sync = before.clone();
        let mut motion: Slide = slide(entity);
        motion.m_chatTime -= 1;
        state.m_bar.m_health = entity.field(EntityField::Health) as i32;
        if state.m_awake
        {
            if state.m_moving
            {
                crushEntities(entity, info, &mut state, &mut motion);
            }
            act(entity, info, &mut state, &mut motion);
        }
        else
        {
            let _ = entity.setMotion([0.0; 3]);
        }
        setSlide(entity, motion);
        if state != before
        {
            let _ = entity.setSynced(&state);
        }
    }

    // EntitySlider.attackEntityFrom: only a player's pickaxe in close reach hurts it, the first hit wakes it
    fn onHurt(&self, entity: Entity, source: Option<i32>, amount: i32) -> Option<i32>
    {
        let world: World = entity.world;
        // EntitySlider.attackEntityFrom only takes a player's own blow, no shot, spell, blast or fire
        let indirect: bool = crate::damage::projectile()
            || crate::damage::magic()
            || crate::damage::explosion()
            || crate::damage::burning();
        if indirect
        {
            return None;
        }

        let player: EntityInfo =
            source.and_then(|id| Entity { world, id }.info()).filter(|attacker| attacker.player.is_some())?;
        let info: EntityInfo = entity.info()?;
        let attacker: Player = Player { id: player.id };
        let mut state: Sync = sync(entity);
        let held: Option<ItemStack> = attacker.held();
        if held.as_ref().is_some_and(|stack| stack.item.is("minecraft:apple"))
        {
            chat(entity, attacker, "gui.slider.apple");
            return None;
        }
        let distance: f64 = (0..3).map(|axis| (player.pos[axis] - info.pos[axis]).powi(2)).sum::<f64>().sqrt();
        if distance > slide::REACH && !state.m_awake
        {
            chat(entity, attacker, "gui.slider.far");
            return None;
        }
        if !held.as_ref().is_some_and(isPickaxe)
        {
            chat(entity, attacker, "gui.slider.notpickaxe");
            return None;
        }
        if !world.isAuthority()
        {
            return Some(amount);
        }
        let health: i32 = entity.field(EntityField::Health) as i32 - amount;
        let mut motion: Slide = slide(entity);
        for _ in 0..if health <= 0 { 2 } else { 4 }
        {
            let spot: [f64; 3] = [
                info.pos[0] + (fraction(world) - fraction(world)) * slide::DOOR_SCATTER,
                feet(entity) + slide::DOOR_RISE,
                info.pos[2] + (fraction(world) - fraction(world)) * slide::DOOR_SCATTER,
            ];
            if health <= 0
            {
                world.particle("explode", spot, [0.0; 3]);
            }
            let _ = world.breakEffect(
                BlockPos::new(spot[0].floor() as i32, spot[1].floor() as i32, spot[2].floor() as i32),
                blockId("minecraft:stone"),
                0,
            );
        }
        if !state.m_awake
        {
            world.playSound(info.pos, sound::AWAKEN, sound::VOLUME, pitch(world));
            state.m_bar.m_focus = Some(player.id);
            closeDoor(entity);
            state.m_awake = true;
        }
        if state.m_moving
        {
            motion.m_velocity *= slide::SLOWED;
        }
        let (a, c) = ((info.pos[0] - player.pos[0]).abs(), (info.pos[2] - player.pos[2]).abs());
        state.m_hurtAxis = if a > c
        {
            [
                0.0,
                if info.pos[0] > player.pos[0]
                {
                    -1.0
                }
                else
                {
                    1.0
                },
            ]
        }
        else
        {
            [
                if info.pos[2] > player.pos[2]
                {
                    -1.0
                }
                else
                {
                    1.0
                },
                0.0,
            ]
        };
        state.m_hurtTilt = slide::HURT_TILT - health.max(0) as f32 / slide::HURT_SPREAD;
        state.m_hurtAge = entity.field(EntityField::TicksExisted) as f32;
        state.m_bar.m_health = health.max(0);
        setSlide(entity, motion);
        let _ = entity.setSynced(&state);
        Some(amount)
    }

    // the dying hit opens the door, frees the dungeon stone, lays the trapdoors over the treasure and leaves the key
    fn onDeath(&self, entity: Entity, source: Option<i32>)
    {
        let world: World = entity.world;
        let Some(info) = entity.info().filter(|_| world.isAuthority())
        else
        {
            return;
        };
        let home: Home = entity.data();
        openDoor(entity);
        unlock(world, home.m_dungeon, &LOCKED);
        // the killer and everyone in the dungeon earn it
        let dungeonReach: f64 = 10.0;
        progress::grantInside(world, home.m_dungeon, [dungeonReach; 3], Award::Bronze);
        if let Some(killer) = source.and_then(|id| world.entity(id)).filter(|killer| killer.player.is_some())
        {
            progress::grant(Player { id: killer.id }, Award::Bronze);
        }
        let [x, y, z] = home.m_dungeon;
        let trapdoor: i32 = blockId("minecraft:trapdoor");
        const TRAPDOORS: [([i32; 2], i32); 4] = [([7, 7], 3), ([8, 7], 2), ([7, 8], 3), ([8, 8], 2)];
        for ([dx, dz], meta) in TRAPDOORS
        {
            let _ = world.setBlock(BlockPos::new(x + dx, y + 1, z + dz), trapdoor, meta);
        }
        world.playSound(info.pos, sound::DEATH, sound::VOLUME, pitch(world));
        let _ =
            world.spawnItem([info.pos[0], info.pos[1] + slide::KEY_LIFT, info.pos[2]], ItemStack::of("bronze_key", 1));
        SLIDES.with(|slides| slides.borrow_mut().remove(&entity.id));
        entity.kill();
    }

    // SliderRenderer: the stone cube, red when critical, tilted by its last hit, with glowing eyes while awake
    fn pose(&self, input: &PoseInput) -> Pose
    {
        let state: Sync = b173_api::fromBytes(&input.synced).unwrap_or_default();
        let critical: bool = critical(&state);
        let skin: &str = match (state.m_awake, critical)
        {
            (true, true) => texture::AWAKE_CRITICAL,
            (true, false) => texture::AWAKE,
            (false, true) => texture::ASLEEP_CRITICAL,
            (false, false) => texture::ASLEEP,
        };
        let tilt: f32 = state.m_hurtTilt * slide::HURT_FADE.powf((input.age - state.m_hurtAge).max(0.0));
        let mut body: Program = Program::new();
        body.push().scale(slide::SCALE, slide::SCALE, slide::SCALE);
        if tilt > 0.01
        {
            body.rotate(tilt * slide::HURT_DEGREES, state.m_hurtAxis[0], 0.0, state.m_hurtAxis[1]);
        }
        body.texture(skin).part(model::HEAD, [0.0; 3], slide::PIVOT);
        if state.m_awake
        {
            let glow: &str = if critical
            {
                texture::GLOW_CRITICAL
            }
            else
            {
                texture::GLOW
            };
            body.texture(glow)
                .blend(true)
                .lighting(false)
                .part(model::HEAD, [0.0; 3], slide::PIVOT)
                .lighting(true)
                .blend(false);
        }
        body.pop();
        Pose { pre: Vec::new(), model: body.take() }
    }
}

pub fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.entities().register(
        NAME,
        EntitySpec {
            template: Identifier::parse("minecraft:ghast").ok(),
            texture: Some(texture::ASLEEP.to_owned()),
            size: Some([2.0, 2.0]),
            health: Some(slide::HEALTH),
            // EntitySlider.dropFewItems, seven to nine carved stone; the key floats up on its own in onDeath
            drops: Some(vec![crate::entities::drop(
                "carved_stone",
                [slide::STONE_DROPS, slide::STONE_DROPS + slide::STONE_SPREAD - 1],
            )]),
            persistent: true,
            immovable: true,
            sounds: crate::entities::sounds("ambient.cave.cave", "step.stone", ""),
            model: Some(EntityModelSpec { shadow: 1.5, ..model::model() }),
            ..EntitySpec::default()
        },
        Some(std::rc::Rc::new(Slider)),
    )?;
    Ok(())
}
