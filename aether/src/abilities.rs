use crate::accessories::Worn;
use crate::entities::fraction;
use crate::progress::{self, Award};
use b173_api::prelude::*;
use std::cell::RefCell;

const SHIELD: &str = "repulsion_shield";
const INVISIBILITY: &str = "invisibility_cape";
const SENTRY_BOOTS: &str = "sentry_boots";
const LIGHTNING_HELD: [&str; 2] = ["lightning_sword", "lightning_knife"];
const WINGS_TEXTURE: &str = "aether:entity/valkyrie/valkyrie";
const DEGREES_PER_RADIAN: f32 = 57.29578;
const WHITE: u32 = 0xFFFFFF;
const ENTERED: &str = "entered_aether";
// PlayerAetherEvents.onAchievementGet: the first trip into the Aether hands these over, each behind its option
const WELCOME_GIFTS: [(&str, &str); 2] =
    [("lore_book", crate::config::GIVE_LORE_BOOK), ("golden_parachute", crate::config::GIVE_GOLDEN_PARACHUTE)];
// PlayerAether.shouldGetPortal, cleared once a player logged in with the option off or got the frame
const PORTAL_FRAME_HANDLED: &str = "portal_frame_handled";

mod repel
{
    // AbilityRepulsion: projectiles inside the player's box grown by three blocks are turned away
    pub const REACH: f64 = 3.0;
    pub const PLAYER_HALF_WIDTH: f64 = 0.3;
    pub const PLAYER_HEIGHT: f64 = 1.8;
    pub const FLAMES: i32 = 12;
    pub const PUSH_BACK: f64 = 0.15;
    pub const JITTER: f64 = 0.05;
    pub const HALF: f64 = 0.5;
    pub const NOBODY: i32 = -1;
    pub const NOT_A_PROJECTILE: i32 = -2;
}

mod flight
{
    // AbilityFlight
    pub const MAX_COUNT: i32 = 52;
    pub const LIFT_AFTER: i32 = 2;
    pub const MAX_MOD: f64 = 15.0;
    pub const MOD_STEP: f64 = 0.25;
    pub const LIFT: f64 = 0.025;
}

mod wing
{
    // PlayerAether.onUpdate wingSinage
    pub const AIR_FLAP: f32 = 0.75;
    pub const GROUND_FLAP: f32 = 0.15;
    pub const DRIFT: f32 = 0.1;
    pub const TURN: f32 = std::f32::consts::TAU;
    // ModelAetherWings
    pub const MODEL_SCALE: f32 = 0.0625;
    pub const TEXTURE: [f32; 2] = [64.0, 32.0];
    pub const UV: [f32; 2] = [24.0, 31.0];
    pub const SIZE: [f32; 2] = [20.0, 8.0];
    pub const LEFT_ORIGIN: [f32; 3] = [0.0, -7.0, 1.0];
    pub const RIGHT_ORIGIN: [f32; 3] = [-19.0, -7.0, 1.0];
    pub const LEFT_PIVOT: [f32; 3] = [0.5, 5.0, 2.625];
    pub const RIGHT_PIVOT: [f32; 3] = [-0.5, 5.0, 2.625];
    pub const YAW: f32 = 0.4;
    pub const ROLL: f32 = 0.125;
    pub const YAW_SWING: f32 = 6.0;
    pub const GROUND_ROLL: f32 = 8.0;
    pub const AIR_ROLL: f32 = 3.0;
    pub const SNEAK_PITCH: f32 = 0.45;
    pub const SNEAK_OFFSET: [f32; 2] = [-0.17, 0.112];
}

mod double
{
    pub const SWORD: &str = "skyroot_sword";
    pub const VALKYRIE: &str = "aether:valkyrie";
    pub const ITEM: &str = "minecraft:item";
    // ItemDungeonKey and the victory medal never double
    pub const KEPT_SINGLE: [&str; 4] = ["bronze_key", "silver_key", "golden_key", "victory_medal"];
    pub const REACH: f64 = 1.5;
    pub const FRESH: f64 = 1.0;
}

thread_local!
{
    // AbilityFlight count and modifier of the local player
    static FLIGHT: RefCell<(i32, f64)> = const { RefCell::new((0, 1.0)) };
    // where a skyroot sword made a kill this tick, by dimension
    static DOUBLES: RefCell<Vec<(i32, [f64; 3])>> = const { RefCell::new(Vec::new()) };
}

// PlayerAether.wingSinage of every player this side draws
static SINAGE: EntityState<f32> = EntityState::new("wing_sinage");

pub(crate) fn entityOf(player: Player) -> Option<(Entity, EntityInfo)>
{
    let info: EntityInfo = player.info()?;
    Some((Entity { world: World { dimension: info.world }, id: player.id }, info))
}

// AbilityRepulsion.onUpdate: the projectile burns away and the shield wears, a shot with no shooter stops the sweep
pub(crate) fn repel(player: Player, worn: &mut Worn)
{
    let Some((entity, info)) = entityOf(player).filter(|_| worn.wearing(SHIELD))
    else
    {
        return;
    };

    let world: World = entity.world;
    let bottom: f64 = entity.field(EntityField::Bottom);
    let reach: f64 = repel::REACH + repel::PLAYER_HALF_WIDTH;

    let inBox = |pos: [f64; 3]|
    {
        (pos[0] - info.pos[0]).abs() <= reach
            && (pos[2] - info.pos[2]).abs() <= reach
            && pos[1] >= bottom - repel::REACH
            && pos[1] <= bottom + repel::PLAYER_HEIGHT + repel::REACH
    };

    let radius: f64 = repel::REACH + repel::PLAYER_HEIGHT;
    for near in world.entitiesNear(info.pos, radius).into_iter().filter(|near| near.id != player.id && inBox(near.pos))
    {
        let projectile: Entity = Entity { world, id: near.id };
        let shooter: i32 = projectile.field(EntityField::Shooter) as i32;
        if shooter == repel::NOT_A_PROJECTILE || shooter == player.id
        {
            continue;
        }
        if shooter == repel::NOBODY
        {
            return;
        }

        projectile.kill();

        for _ in 0..repel::FLAMES
        {
            let push: [f64; 3] =
                near.motion.map(|motion| -motion * repel::PUSH_BACK + (fraction(world) - repel::HALF) * repel::JITTER);
            world.particle("flame", near.pos, push);
        }

        world.playSound(info.pos, "note.snare", 1.0, 1.0);
        worn.wear(SHIELD, 1);
    }
}

// PlayerAetherEvents.onLivingAttack: the full phoenix set shrugs off fire, and AbilityRepulsion.onPlayerAttacked
// lets no projectile through the shield
fn shielded(event: &mut Event)
{
    let EventData::Attack { entity, .. } = event.data().clone()
    else
    {
        return;
    };
    if entity.player.is_none()
    {
        return;
    }

    let player: Player = Player { id: entity.id };
    let fireproof: bool = crate::damage::burning() && crate::player::wearingSet(player, "phoenix");
    if fireproof || (crate::damage::projectile() && Worn::of(player).wearing(SHIELD)) || cushioned(player)
    {
        event.cancel();
    }
}

// AetherEventHandler.onEntityStruckByLightning: a player holding a lightning sword or knife is never struck
fn grounded(event: &mut Event)
{
    let EventData::Entity { entity } = event.data()
    else
    {
        return;
    };
    if entity.player.is_none()
    {
        return;
    }

    let held: Option<ItemStack> = (Player { id: entity.id }).held();
    if held.is_some_and(|held| LIGHTNING_HELD.iter().any(|name| held.item.is(name)))
    {
        event.cancel();
    }
}

// AetherEventHandler.onFall: sentry boots and the full gravitite and valkyrie sets take every fall
fn cushioned(player: Player) -> bool
{
    crate::damage::fall()
        && (crate::player::wearingBoots(player, SENTRY_BOOTS)
            || crate::player::wearingSet(player, "gravitite")
            || crate::player::wearingSet(player, "valkyrie"))
}

// AetherEventHandler.onEntityDropLoot: what a skyroot sword kills drops everything twice, the loot only falls after
// the event so the second helping is laid out at the end of the tick
fn doubled(event: &mut Event)
{
    let EventData::Death { entity: dead, source: Some(killer), .. } = event.data().clone()
    else
    {
        return;
    };
    let world: World = World { dimension: dead.world };
    let spared: bool =
        dead.player.is_some() || dead.kind.as_ref().is_some_and(|kind| kind.to_string() == double::VALKYRIE);
    let byPlayer: bool = world.entity(killer).is_some_and(|info| info.player.is_some());
    if spared || !byPlayer || !world.isAuthority()
    {
        return;
    }

    if !(Player { id: killer }).held().is_some_and(|held| held.item.is(double::SWORD))
    {
        return;
    }

    DOUBLES.with(|doubles| doubles.borrow_mut().push((dead.world, dead.pos)));
}

fn layDoubles()
{
    for (dimension, pos) in DOUBLES.take()
    {
        let world: World = World { dimension };
        for info in world.entitiesNear(pos, double::REACH)
        {
            let entity: Entity = Entity { world, id: info.id };
            let fresh: bool = info.kind.as_ref().is_some_and(|kind| kind.to_string() == double::ITEM)
                && entity.field(EntityField::TicksExisted) <= double::FRESH;
            let Some(stack) = entity.itemStack().filter(|_| fresh)
            else
            {
                continue;
            };

            if !double::KEPT_SINGLE.iter().any(|name| stack.item.is(name))
            {
                let _ = world.spawnItem(pos, stack);
            }
        }
    }
}

// AbilityFlight.onUpdate: holding jump in the full valkyrie set lifts the player a little more each tick
fn fly(entity: Entity, info: &EntityInfo)
{
    let jumping: bool = entity.flag(EntityField::Jumping);
    let grounded: bool = entity.flag(EntityField::OnGround);

    FLIGHT.with(|state|
    {
        let (count, modifier) = &mut *state.borrow_mut();

        if jumping
        {
            *modifier = modifier.min(flight::MAX_MOD);

            if *count > flight::LIFT_AFTER
            {
                if *count < flight::MAX_COUNT
                {
                    *modifier += flight::MOD_STEP;
                    let _ = entity.setMotion([info.motion[0], flight::LIFT * *modifier, info.motion[2]]);

                    *count += 1;
                }
            }
            else
            {
                *count += 1;
            }
        }
        else
        {
            *modifier = 1.0;
        }

        if grounded
        {
            *count = 0;
            *modifier = 1.0;
        }
    });
}

fn flap(entity: Entity)
{
    let grounded: bool = entity.flag(EntityField::OnGround);
    let mut angle: f32 = SINAGE.get(entity);
    angle += if grounded
    {
        wing::GROUND_FLAP
    }
    else
    {
        wing::AIR_FLAP
    };

    if angle > wing::TURN
    {
        angle -= wing::TURN;
    }
    else
    {
        angle += wing::DRIFT;
    }

    crate::entities::store(&SINAGE, entity, &angle);
}

fn tick()
{
    let local: Option<i32> = localPlayer().map(|player| player.id);

    for player in allPlayers()
    {
        let Some((entity, info)) = entityOf(player)
        else
        {
            continue;
        };
        let worn: Worn = Worn::of(player);
        entity.setField(EntityField::Invisible, if worn.wearing(INVISIBILITY) { 1.0 } else { 0.0 });

        if !isClient()
        {
            continue;
        }

        flap(entity);

        if local == Some(player.id) && worn.m_valkyrieSet
        {
            fly(entity, &info);
        }
    }
}

// ModelBox faces of a flat 20 by 8 wing: the front and back quads, a mirrored box flips its x and its winding
fn wingQuads(origin: [f32; 3], mirror: bool) -> Vec<[f32; 5]>
{
    let scale: f32 = wing::MODEL_SCALE;
    let [mut x1, y1, z] = origin;
    let mut x2: f32 = x1 + wing::SIZE[0];
    let y2: f32 = y1 + wing::SIZE[1];

    if mirror
    {
        std::mem::swap(&mut x1, &mut x2);
    }

    let [u, v] = wing::UV;
    let [width, height] = wing::SIZE;
    let [textureWidth, textureHeight] = wing::TEXTURE;
    let corner =
        |x: f32, y: f32, u: f32, v: f32| [x * scale, y * scale, z * scale, u / textureWidth, v / textureHeight];

    let face = |u1: f32, corners: [[f32; 2]; 4]| -> Vec<[f32; 5]>
    {
        let u2: f32 = u1 + width;
        let uvs: [[f32; 2]; 4] = [[u2, v], [u1, v], [u1, v + height], [u2, v + height]];
        let mut quad: Vec<[f32; 5]> =
            corners.iter().zip(uvs.iter()).map(|([x, y], [u, v])| corner(*x, *y, *u, *v)).collect();

        if mirror
        {
            quad.reverse();
        }

        quad
    };
    let mut quads: Vec<[f32; 5]> = face(u, [[x2, y1], [x1, y1], [x1, y2], [x2, y2]]);
    quads.extend(face(u + width, [[x1, y1], [x2, y1], [x2, y2], [x1, y2]]));
    quads
}

// ModelAetherWings.setRotationAngles and the wingLeft and wingRight renders
fn wingOps(origin: [f32; 3], pivot: [f32; 3], mirror: bool, angles: [f32; 3], offset: [f32; 2]) -> Vec<ModelOp>
{
    let [pitch, yaw, roll] = angles.map(|angle| angle * DEGREES_PER_RADIAN);
    vec![
        ModelOp::Push,
        ModelOp::Translate([0.0, offset[0], offset[1]]),
        ModelOp::Translate(pivot.map(|axis| axis * wing::MODEL_SCALE)),
        ModelOp::Rotate(roll, [0.0, 0.0, 1.0]),
        ModelOp::Rotate(yaw, [0.0, 1.0, 0.0]),
        ModelOp::Rotate(pitch, [1.0, 0.0, 0.0]),
        ModelOp::Quads(wingQuads(origin, mirror)),
        ModelOp::Pop,
    ]
}

// PlayerAetherRenderer: the full valkyrie set grows the valkyrie's wings, flapping harder in the air
pub(crate) fn wings(player: &EntityInfo, worn: &Worn) -> Option<PlayerLayer>
{
    if !worn.m_valkyrieSet
    {
        return None;
    }

    let entity: Entity = Entity { world: World { dimension: player.world }, id: player.id };
    let sinage: f32 = SINAGE.get(entity);
    let sneaking: bool = entity.flag(EntityField::Sneaking);

    let rollDivisor: f32 = if entity.flag(EntityField::OnGround)
    {
        wing::GROUND_ROLL
    }
    else
    {
        wing::AIR_ROLL
    };
    let pitch: f32 = if sneaking { wing::SNEAK_PITCH } else { 0.0 };

    let offset: [f32; 2] = if sneaking
    {
        wing::SNEAK_OFFSET
    }
    else
    {
        [0.0, 0.0]
    };
    let swing: f32 = sinage.sin() / wing::YAW_SWING;
    let roll: f32 = sinage.cos() / rollDivisor;
    let mut ops: Vec<ModelOp> =
        wingOps(wing::LEFT_ORIGIN, wing::LEFT_PIVOT, false, [pitch, -wing::YAW + swing, -wing::ROLL + roll], offset);
    ops.extend(wingOps(
        wing::RIGHT_ORIGIN,
        wing::RIGHT_PIVOT,
        true,
        [pitch, wing::YAW - swing, wing::ROLL - roll],
        offset,
    ));

    Some(PlayerLayer
    {
        texture: WINGS_TEXTURE.to_owned(),
        color: WHITE,
        parts: Vec::new(),
        inflate: 0.0,
        glow: false,
        scale: 1.0,
        blend: false,
        ops,
    })
}

// PlayerAetherEvents.onLivingHurt: the full obsidian set halves every hit, rounded up in whole half hearts
fn hardened(event: &mut Event)
{
    let wearer: Option<Player> = event
        .entity()
        .and_then(|entity| entity.info())
        .filter(|info| info.player.is_some())
        .map(|info| Player { id: info.id });

    if !wearer.is_some_and(|player| crate::player::wearingSet(player, "obsidian"))
    {
        return;
    }

    if let EventData::Attack { amount, .. } = event.dataMut()
    {
        *amount = (*amount + 1) / 2;
    }
}

// PlayerAetherEvents.onPlayerAetherLoggedIn: with the option on a player gets one portal frame on the first login
fn starter(event: &mut Event)
{
    let (Some(player), Some(world)) = (event.player(), event.world())
    else
    {
        return;
    };
    if !world.isAuthority() || player.data::<bool>(PORTAL_FRAME_HANDLED)
    {
        return;
    }

    if crate::config::enabled(crate::config::PORTAL_FRAME_START)
        && let Err(error) = player.give(ItemStack::of("aether_portal_frame", 1))
    {
        warn(&format!("starting portal frame: {error}"));
    }

    if let Err(error) = player.setData(PORTAL_FRAME_HANDLED, &true)
    {
        warn(&format!("starting portal frame flag: {error}"));
    }
}

fn welcome(event: &mut Event)
{
    let (EventData::Dimension { to, .. }, Some(player)) = (event.data().clone(), event.player())
    else
    {
        return;
    };
    if Some(to) != crate::portal::aetherDimension() || !(World { dimension: to }).isAuthority()
    {
        return;
    }

    progress::grant(player, Award::EnterAether);

    if player.data::<bool>(ENTERED)
    {
        return;
    }

    let _ = player.setData(ENTERED, &true);

    for (gift, option) in WELCOME_GIFTS
    {
        if crate::config::enabled(option)
        {
            let _ = player.give(ItemStack::of(gift, 1));
        }
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.events().on::<TickPost>(Priority::Normal, |_|
    {
        tick();
        layDoubles();
    })?;
    ctx.events().on::<LivingAttack>(Priority::Normal, shielded)?;
    ctx.events().on::<LivingDrops>(Priority::Normal, doubled)?;
    ctx.events().on::<LightningStrike>(Priority::Normal, grounded)?;
    ctx.events().on::<LivingHurt>(Priority::Normal, hardened)?;
    ctx.events().on::<PlayerChangeDimension>(Priority::Normal, welcome)?;
    ctx.events().on::<PlayerLogin>(Priority::Normal, starter)?;
    Ok(())
}
