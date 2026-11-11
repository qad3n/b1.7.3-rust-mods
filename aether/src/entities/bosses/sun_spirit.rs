use super::{BossBar, Home, bossName, freeBox};
use crate::entities::crystal::{self, Kind};
use crate::entities::fire_minion::{SPIRIT_TEXTURE, spiritPose};
use crate::entities::fraction;
use crate::models::Program;
use crate::names::{AIR, Pick, holds};
use crate::progress::{self, Award};
use b173_api::math::{floor, yawTowards};
use b173_api::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

const NAME: &str = super::SUN_SPIRIT;
const FROZEN_TEXTURE: &str = "aether:entity/boss/sun_spirit/frozen_sun_spirit";
const LOCKED: &str = "locked_hellfire_stone";
const LOCKED_LIGHT: &str = "locked_light_hellfire_stone";

mod spirit
{
    // EntitySunSpirit
    pub const HEALTH: i32 = 50;
    pub const SIZE: f32 = 2.0;
    pub const BASE_SPEED: f64 = 0.5;
    pub const SPEED_LOSS: f64 = 0.2;
    pub const SPEED_HEALTH: f64 = 70.0;
    pub const EAST_EDGE: f64 = 8.5;
    pub const WEST_EDGE: f64 = 10.0;
    pub const SOUTH_EDGE: f64 = 10.0;
    pub const NORTH_EDGE: f64 = 9.0;
    pub const COURSE_TICKS: i32 = 20;
    pub const SWERVE_ODDS: i32 = 3;
    pub const SWERVE: f64 = 60.0;
    pub const FLAME_AT: i32 = 40;
    pub const BALL_AT: f32 = 55.0;
    pub const BALLS_MIN: i32 = 2;
    pub const BALLS_SPREAD: i32 = 3;
    pub const BURN_DAMAGE: i32 = 10;
    pub const BURN_TICKS: f64 = 300.0;
    pub const BURN_REACH: f64 = 4.0;
    // half the spirit's width plus a player's
    pub const BURN_SIDE: f64 = 1.3;
    pub const STEAM_RADIUS: i32 = 10;
    pub const STEAM_LAYERS: i32 = 8;
    pub const STEAM_PUFFS: i32 = 8;
    // the water search runs every few ticks instead of every tick, puddles still vanish within a quarter second
    pub const STEAM_EVERY: f64 = 5.0;
    pub const DOOR_REACH: i32 = 13;
    pub const UNLOCK_REACH: i32 = 20;
    pub const UNLOCK_LOW: i32 = -3;
    pub const UNLOCK_HIGH: i32 = 6;
    pub const DIALOGUE_LINES: u8 = 9;
    pub const DUEL_LINE: u8 = 9;
    pub const RETORT_LINE: u8 = 10;
    pub const CHAT_PAUSE: i32 = 100;
    pub const FALL: f64 = -0.075;
    pub const LIFT: f32 = 0.5;
    pub const SCALE: f32 = 2.25;
    pub const PITCH_WOBBLE: f32 = 0.2;
    pub const PITCH: f32 = 1.2;
    pub const FIZZ_VOLUME: f32 = 2.0;
    pub const FULL_TURN: f64 = 360.0;
    pub const HALF_TURN: f64 = 180.0;
    pub const FACING: [f64; 4] = [90.0, 270.0, 180.0, 0.0];
}

// gui.spirit lines in order, a line with two parts sends both
const LINES: [&[&str]; 9] = [
    &["line0"],
    &["line1"],
    &["line2"],
    &["line3"],
    &["line4"],
    &["line5.1", "line5.2"],
    &["line6.1", "line6.2"],
    &["line7.1", "line7.2"],
    &["line8"],
];
const FIREPROOF: [&str; 2] = ["crystal", "fire_minion"];
const RED: &str = "\u{a7}c";
const GOLD: &str = "\u{a7}6";
const AQUA: &str = "\u{a7}b";

#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
struct Sync
{
    m_bar: BossBar,
    m_freezing: bool,
}

#[derive(Default, Clone, Copy)]
struct Mind
{
    m_started: bool,
    m_rotary: f64,
    m_motionTimer: i32,
    m_flameCount: i32,
    m_ballCount: i32,
    m_chatLine: u8,
    m_chatCount: i32,
}

thread_local!
{
    static MINDS: RefCell<HashMap<i32, Mind>> = RefCell::new(HashMap::new());
}

pub(crate) fn bar(boss: Entity) -> BossBar { boss.synced::<Sync>().m_bar }

fn mind(entity: Entity) -> Mind { MINDS.with(|minds| minds.borrow().get(&entity.id).copied()).unwrap_or_default() }

fn setMind(entity: Entity, state: Mind) { MINDS.with(|minds| minds.borrow_mut().insert(entity.id, state)); }

fn sync(entity: Entity) -> Sync
{
    let mut state: Sync = entity.synced();
    if state.m_bar.m_name.is_empty() && entity.world.isAuthority()
    {
        state.m_bar = BossBar
        {
            m_name: bossName(entity.world, NAME),
            m_health: spirit::HEALTH,
            m_maxHealth: spirit::HEALTH,
            m_focus: None,
        };

        let _ = entity.setSynced(&state);
    }

    state
}

fn line(player: Player, color: &str, key: &str)
{
    player.chat(&format!("{color}{}", translate(&format!("aether.gui.spirit.{key}"))));
}

// EntitySunSpirit.setDoor on the entrance side, unlockTreasure opens the side across the hall
fn door(world: World, home: &Home, far: bool, block: Pick)
{
    let [x, y, z] = home.m_dungeon;

    let side: i32 = match (home.m_direction, far)
    {
        (0, false) | (1, true) => -spirit::DOOR_REACH,
        (0, true) | (1, false) => spirit::DOOR_REACH,
        (3, false) | (2, true) => spirit::DOOR_REACH,
        _ => -spirit::DOOR_REACH,
    };

    for dy in -1..2
    {
        for across in -1..2
        {
            let pos: BlockPos = if home.m_direction / 2 == 0
            {
                BlockPos::new(x + side, y + dy, z + across)
            }
            else
            {
                BlockPos::new(x + across, y + dy, z + side)
            };

            if !holds(world, pos, block)
            {
                let _ = crate::names::handle(block).place(world, pos, true);
            }
        }
    }
}

fn unlockTreasure(world: World, home: &Home)
{
    door(world, home, true, AIR);

    let [x, y, z] = home.m_dungeon;
    let reach: i32 = spirit::UNLOCK_REACH;
    freeBox(
        world,
        [x - reach, y + spirit::UNLOCK_LOW, z - reach],
        [x + reach, y + spirit::UNLOCK_HIGH, z + reach],
        &[(LOCKED, "hellfire_stone"), (LOCKED_LIGHT, "light_hellfire_stone")],
    );
}

fn pitch(world: World) -> f32 { fraction(world) as f32 - fraction(world) as f32 * spirit::PITCH_WOBBLE + spirit::PITCH }

// EntitySunSpirit.burnEntities: anything living under or around it catches fire
fn burn(entity: Entity, info: &EntityInfo)
{
    let world: World = entity.world;
    for other in world
        .entitiesNear(info.pos, spirit::BURN_REACH)
        .into_iter()
        .filter(|other| other.id != entity.id && other.health.is_some())
    {
        let horizontal: bool = (other.pos[0] - info.pos[0]).abs() < spirit::BURN_SIDE
            && (other.pos[2] - info.pos[2]).abs() < spirit::BURN_SIDE;
        // crystals and minions are immune to fire like the spirit's own aura
        let fireproof: bool =
            other.kind.as_ref().is_some_and(|kind| kind.namespace() == "aether" && FIREPROOF.contains(&kind.path()));

        if !horizontal || fireproof
        {
            continue;
        }

        let victim: Entity = Entity { world, id: other.id };
        crate::damage::hit(victim, spirit::BURN_DAMAGE, Some(entity.id), crate::damage::kind::MOB);
        victim.setField(EntityField::FireTicks, spirit::BURN_TICKS);
    }
}

// EntitySunSpirit.evapWater over the hall floor
fn evaporate(entity: Entity, info: &EntityInfo, originY: i32)
{
    let world: World = entity.world;
    let [cx, cz] = [floor(info.pos[0]), floor(info.pos[2])];
    for x in cx - spirit::STEAM_RADIUS..=cx + spirit::STEAM_RADIUS
    {
        for z in cz - spirit::STEAM_RADIUS..=cz + spirit::STEAM_RADIUS
        {
            for layer in 0..spirit::STEAM_LAYERS
            {
                let y: i32 = originY - 2 + layer;
                let pos: BlockPos = BlockPos::new(x, y, z);
                let block: i32 = world.block(pos);
                if block == 0 || !World::blockInfo(block).liquid || world.tagged(pos, "minecraft:lava")
                {
                    continue;
                }

                let _ = world.clear(pos);

                world.playSound(info.pos, "random.fizz", spirit::FIZZ_VOLUME, pitch(world));

                for _ in 0..spirit::STEAM_PUFFS
                {
                    world.particle(
                        "largesmoke",
                        [x as f64 + fraction(world), y as f64 + 0.75, z as f64 + fraction(world)],
                        [0.0; 3],
                    );
                }
            }
        }
    }
}

// EntitySunSpirit.makeFireBall: every few fire balls one is an ice ball the player can bat back at it
fn fireBall(entity: Entity, info: &EntityInfo, state: &mut Mind)
{
    let world: World = entity.world;
    world.playSound(info.pos, "mob.ghast.fireball", pitch(world), 1.0);
    state.m_ballCount += 1;
    let icy: bool = state.m_ballCount >= spirit::BALLS_MIN + world.random(spirit::BALLS_SPREAD);
    if icy
    {
        state.m_ballCount = 0;
    }

    let at: [f64; 3] = [info.pos[0] - info.motion[0] / 2.0, info.pos[1], info.pos[2] - info.motion[2] / 2.0];
    crystal::spawn(world, if icy { Kind::Ice } else { Kind::Fire }, at, None, Some(entity.id));
}

// EntitySunSpirit.updateEntityActionState: circles the hall at its origin height, bouncing off the walls
fn circle(entity: Entity, info: &EntityInfo, target: &EntityInfo, home: &Home, state: &mut Mind)
{
    let world: World = entity.world;
    let [ox, oy, oz] = home.m_dungeon.map(|axis| axis as f64);
    let health: f64 = entity.field(EntityField::Health);
    let velocity: f64 = spirit::BASE_SPEED - health / spirit::SPEED_HEALTH * spirit::SPEED_LOSS;
    let _ = entity.setPosition([info.pos[0], oy, info.pos[2]]);
    let mut changed: bool = false;

    if (info.motion[0] >= 0.0 && info.pos[0] > ox + spirit::EAST_EDGE)
        || (info.motion[0] <= 0.0 && info.pos[0] < ox - spirit::WEST_EDGE)
    {
        state.m_rotary = spirit::FULL_TURN - state.m_rotary;
        changed = true;
    }

    if (info.motion[2] >= 0.0 && info.pos[2] > oz + spirit::SOUTH_EDGE)
        || (info.motion[2] <= 0.0 && info.pos[2] < oz - spirit::NORTH_EDGE)
    {
        state.m_rotary = spirit::HALF_TURN - state.m_rotary;
        changed = true;
    }

    state.m_rotary = state.m_rotary.rem_euclid(spirit::FULL_TURN);

    entity.setField(EntityField::Yaw, f64::from(yawTowards(info.pos, target.pos)));

    let angle: f64 = state.m_rotary.to_radians();
    let _ = entity.setMotion([angle.sin() * velocity, 0.0, angle.cos() * velocity]);

    state.m_motionTimer += 1;
    if state.m_motionTimer >= spirit::COURSE_TICKS || changed
    {
        if world.random(spirit::SWERVE_ODDS) == 0
        {
            state.m_rotary += (fraction(world) - fraction(world)) * spirit::SWERVE;
        }

        state.m_motionTimer = 0;
    }

    state.m_flameCount += 1;
    if state.m_flameCount == spirit::FLAME_AT
    {
        let below: BlockPos = BlockPos::new(floor(info.pos[0]), home.m_dungeon[1] - 2, floor(info.pos[2]));

        if world.isAir(below)
        {
            let _ = world.place(below, "minecraft:fire");
        }
    }
    else if state.m_flameCount as f32 >= spirit::BALL_AT + health as f32 / 2.0
    {
        fireBall(entity, info, state);
        state.m_flameCount = 0;
    }
}

// the player it fought died: back to its spot, healed, door open
fn reset(entity: Entity, home: &Home, sync: &mut Sync, state: &mut Mind, loser: Player)
{
    let [x, y, z] = home.m_dungeon;
    let _ = entity.setPosition([x as f64 + 0.5, y as f64, z as f64 + 0.5]);
    let _ = entity.setMotion([0.0; 3]);

    state.m_chatLine = spirit::RETORT_LINE;
    line(loser, RED, "playerdied");
    state.m_chatCount = spirit::CHAT_PAUSE;
    door(entity.world, home, false, AIR);
    sync.m_freezing = false;
    sync.m_bar.m_focus = None;
    entity.setField(EntityField::Health, spirit::HEALTH as f64);
    sync.m_bar.m_health = spirit::HEALTH;
}

struct SunSpirit;

#[entity_behavior]
impl EntityBehavior for SunSpirit
{
    fn onAi(&self, entity: Entity, info: &EntityInfo)
    {
        let world: World = entity.world;
        if !world.isAuthority()
        {
            return;
        }

        let home: Home = entity.data();
        let mut state: Mind = mind(entity);
        let target: Option<EntityInfo> =
            sync(entity).m_bar.m_focus.and_then(|id| Entity { world, id }.info()).filter(|target| target.alive);

        match target
        {
            Some(target) => circle(entity, info, &target, &home, &mut state),
            None => entity.setField(EntityField::Yaw, spirit::FACING[home.m_direction.clamp(0, 3) as usize]),
        }

        setMind(entity, state);
    }

    // EntitySunSpirit.onUpdate: flames, burning, steam and keeping track of its challenger
    fn onTick(&self, entity: Entity, info: &EntityInfo)
    {
        let world: World = entity.world;
        if !world.isAuthority()
        {
            let below: f64 = entity.field(EntityField::Bottom);
            let spot: [f64; 3] = [
                info.pos[0] + (fraction(world) - 0.5) * fraction(world),
                below + fraction(world) - 0.5,
                info.pos[2] + (fraction(world) - 0.5) * fraction(world),
            ];
            world.particle("flame", spot, [0.0, spirit::FALL, 0.0]);
            return;
        }

        let home: Home = entity.data();
        let before: Sync = sync(entity);
        let mut state: Sync = before.clone();
        let mut thinking: Mind = mind(entity);
        if !thinking.m_started
        {
            thinking.m_started = true;
            thinking.m_rotary = fraction(world) * spirit::FULL_TURN;
        }

        let health: i32 = entity.field(EntityField::Health) as i32;
        state.m_bar.m_health = health.max(0);

        if let Some(id) = state.m_bar.m_focus
            && (Entity { world, id }).info().is_none_or(|target| !target.alive)
        {
            reset(entity, &home, &mut state, &mut thinking, Player { id });
        }
        // hurt frames freeze it on the client from hurt_time, only death keeps it frozen here
        state.m_freezing = health <= 0;

        if health > 0
        {
            burn(entity, info);

            if entity.field(EntityField::TicksExisted) % spirit::STEAM_EVERY == 0.0
            {
                evaporate(entity, info, home.m_dungeon[1]);
            }
        }

        thinking.m_chatCount -= 1;
        setMind(entity, thinking);

        if state != before
        {
            let _ = entity.setSynced(&state);
        }
    }

    // EntitySunSpirit.chatWithMe: nine warnings, then the duel
    fn onInteract(&self, entity: Entity, player: Player) -> bool
    {
        let world: World = entity.world;
        if !world.isAuthority()
        {
            return true;
        }
        let mut state: Mind = mind(entity);
        if state.m_chatCount > 0
        {
            return false;
        }

        let fighting: bool = sync(entity).m_bar.m_focus.is_some();

        match state.m_chatLine
        {
            spoken if spoken < spirit::DIALOGUE_LINES =>
            {
                for key in LINES[spoken as usize]
                {
                    line(player, RED, key);
                }

                state.m_chatLine += 1;
                setMind(entity, state);
                false
            }

            spirit::DUEL_LINE =>
            {
                line(player, GOLD, "line9");
                state.m_chatLine = spirit::RETORT_LINE;

                if let (Some(info), Some(seen)) = (entity.info(), player.info())
                {
                    state.m_rotary = (info.pos[0] - seen.pos[0]).atan2(info.pos[2] - seen.pos[2]).to_degrees();
                }

                setMind(entity, state);

                let mut synced: Sync = sync(entity);
                synced.m_bar.m_focus = Some(player.id);

                let _ = entity.setSynced(&synced);

                door(world, &entity.data(), false, Pick::of(LOCKED));
                true
            }

            _ =>
            {
                if !fighting
                {
                    line(player, RED, "line10");
                    state.m_chatLine = spirit::DUEL_LINE;
                    setMind(entity, state);
                }

                false
            }
        }
    }

    // EntitySunSpirit.attackEntityFrom: only an ice crystal batted back at it hurts, and each wound calls a minion
    fn onHurt(&self, entity: Entity, source: Option<i32>, amount: i32) -> Option<i32>
    {
        let world: World = entity.world;
        let crystalHit: EntityInfo = source
            .and_then(|id| Entity { world, id }.info())
            .filter(|hit| hit.kind.as_ref().is_some_and(|kind| kind.path() == "crystal"))?;
        let look: crystal::Look = Entity { world, id: crystalHit.id }.synced();
        if look.m_kind != Kind::Ice
        {
            return None;
        }

        if world.isAuthority()
            && let Some(info) = entity.info()
            && let Ok(minion) = world.spawnEntity("aether:fire_minion", info.pos)
            && let Some(focus) = sync(entity).m_bar.m_focus
        {
            minion.setField(EntityField::TargetId, focus as f64);
        }

        Some(amount)
    }

    // the cold finally reaches it: the doors open, the hellfire stone unlocks and the key falls
    fn onDeath(&self, entity: Entity, _source: Option<i32>)
    {
        let world: World = entity.world;
        if entity.info().is_none() || !world.isAuthority()
        {
            return;
        }

        let home: Home = entity.data();
        let mut state: Sync = sync(entity);
        state.m_freezing = true;

        if let Some(id) = state.m_bar.m_focus
        {
            line(Player { id }, AQUA, "dead");

            let hallReach: [f64; 3] = [20.0, 3.0, 20.0];
            progress::grantInside(world, home.m_dungeon, hallReach, Award::Gold);
            progress::grant(Player { id }, Award::Gold);
        }

        state.m_bar.m_health = 0;

        let _ = entity.setSynced(&state);

        door(world, &home, false, AIR);
        unlockTreasure(world, &home);

        crate::sun_altar::unlock();
        MINDS.with(|minds| minds.borrow_mut().remove(&entity.id));
    }

    // SunSpiritRenderer: the fire minion's body drawn larger, frozen blue while hurt
    fn pose(&self, input: &PoseInput) -> Pose
    {
        let state: Sync = b173_api::fromBytes(&input.synced).unwrap_or_default();
        let mut pre: Program = Program::new();
        pre.translate(0.0, spirit::LIFT, 0.0);

        let body: Pose = spiritPose(input, spirit::SCALE);
        let mut model: Program = Program::new();
        model.texture(
            if state.m_freezing || input.hurt_time > 0
            {
                FROZEN_TEXTURE
            }
            else
            {
                SPIRIT_TEXTURE
            },
        );
        model.m_ops.extend(body.model);
        Pose { pre: pre.take(), model: model.take() }
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.entities().register(
        NAME,
        EntitySpec
        {
            template: Identifier::parse("minecraft:ghast").ok(),
            texture: Some(SPIRIT_TEXTURE.to_owned()),
            size: Some([spirit::SIZE, spirit::SIZE]),
            health: Some(spirit::HEALTH),
            fire_immune: true,
            persistent: true,
            immovable: true,
            // EntitySunSpirit.dropFewItems
            drops: Some(vec![crate::entities::drop("golden_key", [1, 1]), crate::entities::drop("sun_altar", [1, 1])]),
            sounds: crate::entities::sounds("", "", ""),
            model: Some(EntityModelSpec { shadow: 1.0, ..crate::models::sun_spirit::model() }),
            ..EntitySpec::default()
        },
        Some(std::rc::Rc::new(SunSpirit)),
    )?;
    Ok(())
}
