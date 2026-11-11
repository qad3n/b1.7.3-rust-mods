use crate::damage::kind;
use crate::models::Program;
use b173_api::math::{length, sub};
use b173_api::prelude::*;
use b173_api::terrain::Random;

// a shot the Aether fires: how it flies, what it does on a hit and how it looks
pub(crate) struct Shot
{
    pub m_name: &'static str,
    pub m_texture: &'static str,
    pub m_gravity: f64,
    pub m_drag: f64,
    // EntityArrow damage scales with speed, a zero damage shot only pushes
    pub m_damage: f64,
    // the vanilla damage type closest to the source's DamageSource for this shot
    pub m_kind: &'static str,
    pub m_poison: bool,
    pub m_knockback: Option<(f64, f64)>,
    pub m_lifetime: i32,
    pub m_look: Look,
    pub m_impact: Impact,
    // ticks of fire set on what it hits
    pub m_burn: f64,
    // EntityDartBase.getStack and EntityArrow, what a player who shot it picks up once it is stuck
    pub m_pickup: Option<&'static str>,
}

// what happens where a shot lands besides its damage
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Impact
{
    Stick,
    Vanish,
    // EntityLightningKnife.summonLightning on an entity or a solid block
    Lightning,
    // EntityHammerProjectile.onImpact: a burst of smoke and flame
    Blast,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Look
{
    Dart,
    Sprite,
    // a whole item icon facing the camera
    Icon,
}

mod arrow
{
    // EntityArrow flight in b1.7.3 and 1.7.10
    pub const GRAVITY: f64 = 0.05;
    pub const DRAG: f64 = 0.99;
    // darts stay this long in a block before vanishing
    pub const STUCK_TICKS: i32 = 1200;
}

pub(crate) const GOLDEN_DART: &str = "golden_dart";
pub(crate) const POISON_DART: &str = "poison_dart";
pub(crate) const ENCHANTED_DART: &str = "enchanted_dart";
pub(crate) const POISON_NEEDLE: &str = "poison_needle";
pub(crate) const ZEPHYR_SNOWBALL: &str = "zephyr_snowball";
pub(crate) const HAMMER: &str = "hammer_projectile";
pub(crate) const LIGHTNING_KNIFE: &str = "lightning_knife";
pub(crate) const FLAMING_ARROW: &str = "flaming_arrow";

pub(crate) const SHOTS: [Shot; 8] = [
    Shot
    {
        m_name: GOLDEN_DART,
        m_texture: "aether:entity/projectile/dart/golden_dart",
        m_gravity: arrow::GRAVITY,
        m_drag: arrow::DRAG,
        m_damage: 4.0,
        m_kind: kind::ARROW,
        m_poison: false,
        m_knockback: None,
        m_lifetime: arrow::STUCK_TICKS,
        m_look: Look::Dart,
        m_impact: Impact::Stick,
        m_burn: 0.0,
        m_pickup: Some("golden_dart"),
    },
    Shot
    {
        m_name: POISON_DART,
        m_texture: "aether:entity/projectile/dart/poison_dart",
        m_gravity: arrow::GRAVITY,
        m_drag: arrow::DRAG,
        m_damage: 0.0,
        m_kind: kind::ARROW,
        m_poison: true,
        m_knockback: None,
        m_lifetime: arrow::STUCK_TICKS,
        m_look: Look::Dart,
        m_impact: Impact::Stick,
        m_burn: 0.0,
        m_pickup: Some("poison_dart"),
    },
    Shot
    {
        m_name: ENCHANTED_DART,
        m_texture: "aether:entity/projectile/dart/enchanted_dart",
        m_gravity: arrow::GRAVITY,
        m_drag: arrow::DRAG,
        m_damage: 6.0,
        m_kind: kind::ARROW,
        m_poison: false,
        m_knockback: None,
        m_lifetime: arrow::STUCK_TICKS,
        m_look: Look::Dart,
        m_impact: Impact::Stick,
        m_burn: 0.0,
        m_pickup: Some("enchanted_dart"),
    },
    Shot
    {
        m_name: POISON_NEEDLE,
        m_texture: "aether:entity/projectile/dart/poison_needle",
        m_gravity: arrow::GRAVITY,
        m_drag: arrow::DRAG,
        m_damage: 0.0,
        m_kind: kind::ARROW,
        m_poison: true,
        m_knockback: None,
        m_lifetime: arrow::STUCK_TICKS,
        m_look: Look::Dart,
        m_impact: Impact::Stick,
        m_burn: 0.0,
        m_pickup: Some("poison_dart"),
    },
    // EntityZephyrSnowball: no gravity, flings what it hits, gone after 150 ticks
    Shot
    {
        m_name: ZEPHYR_SNOWBALL,
        m_texture: "minecraft:gui/items",
        m_gravity: 0.0,
        m_drag: 1.0,
        m_damage: 0.0,
        m_kind: kind::FIREBALL,
        m_poison: false,
        m_knockback: Some((1.5, 0.5)),
        m_lifetime: 150,
        m_look: Look::Sprite,
        m_impact: Impact::Vanish,
        m_burn: 0.0,
        m_pickup: None,
    },
    // EntityHammerProjectile: straight and fast, hurts what it hits and knocks it up
    Shot
    {
        m_name: HAMMER,
        m_texture: "aether:item/misc/hammer_projectile",
        m_gravity: 0.0,
        m_drag: 1.0,
        m_damage: 0.0,
        m_kind: kind::MOB,
        m_poison: false,
        m_knockback: Some((1.0, 0.6)),
        m_lifetime: 100,
        m_look: Look::Icon,
        m_impact: Impact::Blast,
        m_burn: 0.0,
        m_pickup: None,
    },
    // EntityLightningKnife: thrown, calls lightning where it lands
    Shot
    {
        m_name: LIGHTNING_KNIFE,
        m_texture: "aether:item/weapons/lightning_knife",
        m_gravity: throwable::GRAVITY,
        m_drag: arrow::DRAG,
        m_damage: 0.0,
        m_kind: kind::THROWN,
        m_poison: false,
        m_knockback: None,
        m_lifetime: arrow::STUCK_TICKS,
        m_look: Look::Icon,
        m_impact: Impact::Lightning,
        m_burn: 0.0,
        m_pickup: None,
    },
    // EntityPhoenixArrow: an arrow that sets what it hits alight
    Shot
    {
        m_name: FLAMING_ARROW,
        m_texture: "aether:entity/projectile/flaming_arrow",
        m_gravity: arrow::GRAVITY,
        m_drag: arrow::DRAG,
        m_damage: 2.0,
        m_kind: kind::ARROW,
        m_poison: false,
        m_knockback: None,
        m_lifetime: arrow::STUCK_TICKS,
        m_look: Look::Dart,
        m_impact: Impact::Stick,
        m_burn: 100.0,
        m_pickup: Some("minecraft:arrow"),
    },
];

mod throwable
{
    // EntityThrowable gravity
    pub const GRAVITY: f64 = 0.03;
    pub const HAMMER_DAMAGE: i32 = 5;
    pub const BLAST_PUFFS: i32 = 8;
}

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct Flight
{
    m_owner: Option<i32>,
    m_stuck: bool,
    m_age: i32,
    // EntityArrow.canBePickedUp, a player's shot only
    m_pickup: bool,
    // EntityArrow.arrowShake, a stuck shot quivers before it can be picked up
    m_shake: i32,
}

// fires a shot from a point toward a target point with a speed and EntityArrow.setThrowableHeading's spread
pub(crate) fn fire(
    world: World,
    name: &str,
    from: [f64; 3],
    toward: [f64; 3],
    speed: f64,
    spread: f64,
    owner: Option<i32>,
) -> Option<Entity>
{
    let entity: Entity = world.spawnEntity(&format!("aether:{name}"), from).ok()?;
    let [dx, dy, dz] = sub(toward, from);
    let length: f64 = length([dx, dy, dz]).max(1.0e-4);
    let jitter = || (world.random(2001) as f64 / 1000.0 - 1.0) * 0.0075 * spread;
    let motion: [f64; 3] =
        [(dx / length + jitter()) * speed, (dy / length + jitter()) * speed, (dz / length + jitter()) * speed];
    let _ = entity.setMotion(motion);
    let pickup: bool = owner.and_then(|owner| world.entity(owner)).is_some_and(|shooter| shooter.player.is_some());
    let _ = entity.setData(&Flight { m_owner: owner, m_pickup: pickup, ..Flight::default() });

    if let Some(owner) = owner
    {
        entity.setField(EntityField::Shooter, owner as f64);
    }

    Some(entity)
}

struct Projectile
{
    m_shot: &'static Shot,
}

mod hit
{
    // EntityArrow.onUpdate plays bowhit where a shot strikes and quivers seven ticks once stuck
    pub const SHAKE: i32 = 7;
    pub const PITCH_LOW: f32 = 0.9;
    pub const PITCH_SPREAD: f32 = 0.2;
    pub const PITCH: f32 = 1.2;
    // EntityArrow.onUpdate in water: a stronger drag and four bubbles behind
    pub const WATER_DRAG: f64 = 0.8;
    pub const BUBBLES: i32 = 4;
    pub const BUBBLE_BACK: f64 = 0.25;
    // EntityArrow.onCollideWithPlayer, the player's box against the shot's half block box
    pub const REACH: f64 = 0.55;
    pub const EYES: f64 = 1.62;
    pub const HEIGHT: f64 = 1.8;
    pub const HALF: f64 = 0.25;
    pub const POP_VOLUME: f32 = 0.2;
    pub const POP_SPREAD: f32 = 0.7;
    pub const STACK: i32 = 64;
    pub const MAIN_SLOTS: i32 = 36;
}

fn bowhit(world: World, at: [f64; 3])
{
    let pitch: f32 = hit::PITCH / (world.random(1000) as f32 / 1000.0 * hit::PITCH_SPREAD + hit::PITCH_LOW);
    world.playSound(at, "random.bowhit", 1.0, pitch);
}

// InventoryPlayer.addItemStackToInventory fits the stack somewhere in the main inventory
fn room(player: Player, item: &str) -> bool
{
    (0..hit::MAIN_SLOTS).any(|slot| player.slot(slot).is_none_or(|held| held.item.is(item) && held.count < hit::STACK))
}

// EntityArrow.onCollideWithPlayer: a stuck shot goes to the shooter's kind of player who touches it and has room
fn pickUp(shot: &Shot, entity: Entity, info: &EntityInfo, flight: &Flight) -> bool
{
    let Some(item) = shot.m_pickup.filter(|_| flight.m_pickup && flight.m_shake <= 0)
    else
    {
        return false;
    };
    let world: World = entity.world;

    let toucher: Option<Player> = world.players().into_iter().find(|player|
    {
        player.info().is_some_and(|other|
        {
            let feet: f64 = other.pos[1] - hit::EYES;
            (other.pos[0] - info.pos[0]).abs() < hit::REACH
                && (other.pos[2] - info.pos[2]).abs() < hit::REACH
                && feet < info.pos[1] + hit::HALF
                && feet + hit::HEIGHT > info.pos[1] - hit::HALF
        })
    });
    let Some(player) = toucher.filter(|player| room(*player, item))
    else
    {
        return false;
    };

    if let Err(error) = player.give(ItemStack::of(item, 1))
    {
        warn(&format!("could not hand back a {item}: {}", error.0));
        return false;
    }

    let spread: f32 = (world.random(1000) as f32 - world.random(1000) as f32) / 1000.0;
    world.playSound(info.pos, "random.pop", hit::POP_VOLUME, (spread * hit::POP_SPREAD + 1.0) * 2.0);
    entity.kill();
    true
}

mod flame
{
    // EntityPhoenixArrow.onUpdate trails two flames a tick in flight and one every fifth tick once stuck
    pub const FLYING: i32 = 2;
    pub const STUCK_EVERY: f64 = 5.0;
    pub const STILL: f64 = 1.0e-3;
    pub const SPREAD: f64 = 5.0;
    pub const SPREAD_Z: f64 = 3.0;
}

thread_local!
{
    static GAUSSIAN: Random = Random::withSeed(0);
}

// burning shots trail flames on the client, as the source's particles are drawn only there
fn trail(shot: &Shot, entity: Entity, info: &EntityInfo)
{
    if shot.m_burn <= 0.0
    {
        return;
    }

    let still: bool = length(info.motion) < flame::STILL;
    let count: i32 = if still
    {
        i32::from(entity.field(EntityField::TicksExisted) % flame::STUCK_EVERY == 0.0)
    }
    else
    {
        flame::FLYING
    };

    for _ in 0..count
    {
        let jitter: [f64; 3] = GAUSSIAN.with(|random|
        {
            [
                random.nextGaussian() / flame::SPREAD,
                random.nextGaussian() / flame::SPREAD,
                random.nextGaussian() / flame::SPREAD_Z,
            ]
        });
        let at: [f64; 3] = [info.pos[0] + jitter[0], info.pos[1] + jitter[1], info.pos[2] + jitter[2]];
        entity.world.particle("flame", at, [0.0; 3]);
    }
}

// the shot is done: lightning or a blast where it landed, then gone
fn land(shot: &Shot, entity: Entity, at: [f64; 3])
{
    let world: World = entity.world;

    match shot.m_impact
    {
        Impact::Lightning =>
        {
            let _ = world.lightning(at);
        }

        Impact::Blast =>
        {
            for _ in 0..throwable::BLAST_PUFFS
            {
                for particle in ["explode", "explode", "smoke", "largesmoke", "flame"]
                {
                    world.particle(particle, at, [0.0; 3]);
                }
            }
        }

        Impact::Stick | Impact::Vanish =>
        {}
    }

    entity.kill();
}

// what the shot does to the entity it hit
fn strike(shot: &Shot, world: World, target: &EntityInfo, motion: [f64; 3], owner: Option<i32>)
{
    let victim: Entity = Entity { world, id: target.id };
    let speed: f64 = (motion[0] * motion[0] + motion[1] * motion[1] + motion[2] * motion[2]).sqrt();

    if shot.m_damage > 0.0
    {
        crate::damage::hit(victim, (speed * shot.m_damage).ceil() as i32, owner, shot.m_kind);
    }

    if shot.m_poison
    {
        crate::player::poison(victim);
    }

    if shot.m_burn > 0.0
    {
        victim.setField(EntityField::FireTicks, shot.m_burn);
    }

    if let Some((push, lift)) = shot.m_knockback
        && let Some(info) = victim.info()
    {
        let _ = victim.setMotion([
            info.motion[0] + motion[0] * push,
            info.motion[1] + lift,
            info.motion[2] + motion[2] * push,
        ]);
    }
}

#[entity_behavior]
impl EntityBehavior for Projectile
{
    // EntityArrow.onUpdate: the loader already moved it, so the hit check runs back over this tick's path
    fn onTick(&self, entity: Entity, info: &EntityInfo)
    {
        if !entity.world.isAuthority()
        {
            trail(self.m_shot, entity, info);
            return;
        }

        let mut flight: Flight = entity.data();
        flight.m_age += 1;
        if flight.m_age > self.m_shot.m_lifetime
        {
            entity.kill();
            return;
        }

        if flight.m_stuck
        {
            let _ = entity.setMotion([0.0; 3]);

            flight.m_shake = (flight.m_shake - 1).max(0);
            if pickUp(self.m_shot, entity, info, &flight)
            {
                return;
            }

            let _ = entity.setData(&flight);
            return;
        }

        let motion: [f64; 3] = info.motion;
        let to: [f64; 3] = info.pos;
        let from: [f64; 3] = [to[0] - motion[0], to[1] - motion[1], to[2] - motion[2]];

        if self.m_shot.m_impact == Impact::Blast
        {
            entity.world.particle("reddust", [to[0], to[1] + 0.2, to[2]], [1.0, 1.0, 1.0]);
        }

        if let Some(target) =
            entity.world.entitiesAlong(from, to, 0.3, flight.m_owner).into_iter().find(|hit| hit.id != entity.id)
        {
            if self.m_shot.m_impact == Impact::Blast
            {
                crate::damage::hit(
                    Entity { world: entity.world, id: target.id },
                    throwable::HAMMER_DAMAGE,
                    flight.m_owner,
                    kind::MOB,
                );
            }

            if self.m_shot.m_impact == Impact::Stick
            {
                bowhit(entity.world, to);
            }

            strike(self.m_shot, entity.world, &target, motion, flight.m_owner);
            land(self.m_shot, entity, to);
            return;
        }

        if let Some(hit) = entity.world.rayTrace(from, to, false)
        {
            if self.m_shot.m_impact != Impact::Stick
            {
                land(self.m_shot, entity, hit.point);
                return;
            }

            let _ = entity.setPosition(hit.point);
            bowhit(entity.world, hit.point);

            flight.m_stuck = true;
            flight.m_shake = hit::SHAKE;

            let _ = entity.setMotion([0.0; 3]);
            let _ = entity.setData(&flight);

            return;
        }

        let wet: bool = self.m_shot.m_impact == Impact::Stick && entity.flag(EntityField::InWater);
        let drag: f64 = if wet
        {
            hit::WATER_DRAG
        }
        else
        {
            self.m_shot.m_drag
        };
        if wet
        {
            let behind: [f64; 3] = [0, 1, 2].map(|axis| to[axis] - motion[axis] * hit::BUBBLE_BACK);
            for _ in 0..hit::BUBBLES
            {
                entity.world.particle("bubble", behind, motion);
            }

            entity.setField(EntityField::FireTicks, 0.0);
        }

        let _ = entity.setMotion([motion[0] * drag, motion[1] * drag - self.m_shot.m_gravity, motion[2] * drag]);

        let _ = entity.setData(&flight);
    }

    fn pose(&self, input: &PoseInput) -> Pose
    {
        let mut program: Program = Program::new();

        match self.m_shot.m_look
        {
            Look::Dart => dart(&mut program, input),
            Look::Sprite => snowball(&mut program),

            Look::Icon =>
            {
                program.texture(self.m_shot.m_texture);
                icon(&mut program);
            }
        }

        Pose { pre: Vec::new(), model: program.take() }
    }
}

// DartBaseRenderer.renderDart, an arrow's crossed fins and shaft on the dart sheet
fn dart(program: &mut Program, input: &PoseInput)
{
    const SIZE: f32 = 0.05625;

    let [tail0, tail1, tail2, tail3]: [f32; 4] = [0.0, 0.15625, 15.0 / 32.0, 20.0 / 32.0];
    let [shaft0, shaft1, shaft2, shaft3]: [f32; 4] = [0.0, 0.5, 10.0 / 32.0, 15.0 / 32.0];

    program.rotate(input.head_yaw - 90.0, 0.0, 1.0, 0.0).rotate(input.head_pitch, 0.0, 0.0, 1.0);
    program.rotate(45.0, 1.0, 0.0, 0.0).scale(SIZE, SIZE, SIZE).translate(-4.0, 0.0, 0.0);
    program.normal(SIZE, 0.0, 0.0).quad([
        [-7.0, -2.0, -2.0, tail0, tail2],
        [-7.0, -2.0, 2.0, tail1, tail2],
        [-7.0, 2.0, 2.0, tail1, tail3],
        [-7.0, 2.0, -2.0, tail0, tail3],
    ]);
    program.normal(-SIZE, 0.0, 0.0).quad([
        [-7.0, 2.0, -2.0, tail0, tail2],
        [-7.0, 2.0, 2.0, tail1, tail2],
        [-7.0, -2.0, 2.0, tail1, tail3],
        [-7.0, -2.0, -2.0, tail0, tail3],
    ]);

    for _ in 0..5
    {
        program.rotate(72.0, 1.0, 0.0, 0.0).normal(0.0, 0.0, SIZE).quad([
            [-8.0, -2.0, 0.0, shaft0, shaft2],
            [8.0, -2.0, 0.0, shaft1, shaft2],
            [8.0, 2.0, 0.0, shaft1, shaft3],
            [-8.0, 2.0, 0.0, shaft0, shaft3],
        ]);
    }
}

// a whole icon texture facing the camera, as RenderSnowball draws thrown items
fn icon(program: &mut Program)
{
    program.faceCamera().scale(0.5, 0.5, 0.5).normal(0.0, 1.0, 0.0).quad([
        [-0.5, -0.25, 0.0, 0.0, 1.0],
        [0.5, -0.25, 0.0, 1.0, 1.0],
        [0.5, 0.75, 0.0, 1.0, 0.0],
        [-0.5, 0.75, 0.0, 0.0, 0.0],
    ]);
}

// RenderSnowball for the zephyr snowball, the vanilla snowball icon from the item sheet
fn snowball(program: &mut Program)
{
    const ICON: [f32; 4] = [14.0 / 16.0, 0.0, 15.0 / 16.0, 1.0 / 16.0];

    let [u0, v0, u1, v1] = ICON;
    program.faceCamera().scale(0.5, 0.5, 0.5).normal(0.0, 1.0, 0.0).quad([
        [-0.5, -0.25, 0.0, u0, v1],
        [0.5, -0.25, 0.0, u1, v1],
        [0.5, 0.75, 0.0, u1, v0],
        [-0.5, 0.75, 0.0, u0, v0],
    ]);
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for shot in &SHOTS
    {
        ctx.entities().register(
            shot.m_name,
            EntitySpec
            {
                template: Identifier::parse("minecraft:projectile").ok(),
                texture: Some(shot.m_texture.to_owned()),
                size: Some([0.5, 0.5]),
                tracking_range: Some(64),
                update_frequency: Some(5),
                send_velocity: Some(true),
                model: Some(EntityModelSpec { shadow: 0.0, ..EntityModelSpec::default() }),
                ..EntitySpec::default()
            },
            Some(std::rc::Rc::new(Projectile { m_shot: shot })),
        )?;
    }

    Ok(())
}
