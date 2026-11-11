use crate::combat::kind;
use crate::ids;
use crate::models::Program;
use b173_api::math::{blockOf, distance, length, sub};
use b173_api::prelude::*;

// the twilight forest's EntityThrowable shots: how they fall, what they do on a hit and the icon they show
pub(crate) struct Shot
{
    pub m_name: &'static str,
    // EntityThrowable.getGravityVelocity
    pub m_gravity: f64,
    pub m_damage: i32,
    // the vanilla damage type of the source's DamageSource for this shot
    pub m_kind: &'static str,
    pub m_impact: Impact,
    pub m_icon: Icon,
    pub m_puff: Puff,
    // an entity kind the shot flies through, the knight phantoms' own axes and picks pass them
    pub m_passes: Option<&'static str>,
    // makeTrail, what the shot leaves behind each tick on the client
    pub m_trail: Trail,
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Trail
{
    None,
    // a few specks of one particle within half a block
    Specks(&'static str, i32),
    // EntityTFLichBolt and EntityTFTwilightWandBolt, five pale green spell specks
    Spell,
    // EntityTFLichBomb, a flame drawn back toward the bomb
    Ember,
    // EntityFireball, a puff of smoke above it
    Smoke,
}

// what bursts where a shot lands, a named particle or bits of a block
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Puff
{
    Named(&'static str),
    Crack(&'static str),
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Impact
{
    Plain,
    // EntityTFNatureBolt, poisons what it hits, leaves where it lands
    Nature,
    // EntityTFTomeBolt, slows what it hits
    Tome,
    // EntityTFLichBomb, a small blast
    Bomb,
    // EntityTFUrGhastFireball, a ghast's block breaking explosion
    Fireball,
    // EntityTFMoonwormShot, a moonworm that clings where it lands
    Moonworm,
}

// RenderSnowball's icon, a cell of the beta item sheet or a twilight item texture
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Icon
{
    Sheet(i32),
    Texture(&'static str),
}

mod sheet
{
    pub const SEEDS: i32 = 9;
    pub const SNOWBALL: i32 = 14;
    pub const SLIME_BALL: i32 = 30;
    pub const PAPER: i32 = 58;
    pub const FIRE: i32 = 26;
    pub const CELLS: f32 = 16.0;
}

mod trail
{
    pub const SPREAD: f64 = 0.5;
    pub const SPELL_SPECKS: i32 = 5;
    // the spell's red green and blue, each scaled by a random half to whole
    pub const SPELL: [f32; 3] = [0.17, 0.8, 0.69];
    pub const EMBER_PULL: f64 = -0.25;
    pub const SMOKE_RISE: f64 = 0.5;
}

mod flight
{
    // EntityThrowable drag in the air and its life span before it gives up
    pub const DRAG: f64 = 0.99;
    pub const LIFETIME: i32 = 1200;
    pub const HIT_RADIUS: f64 = 0.3;
    pub const PUFFS: i32 = 8;
}

mod effect
{
    pub const BOMB_POWER: f32 = 2.0;
    pub const FIREBALL_POWER: f32 = 1.0;
}

const TWILIGHT_ORB: &str = "twilightforest:item/twilightorb";

pub(crate) const NATURE_BOLT: &str = "nature_bolt";
pub(crate) const TOME_BOLT: &str = "tome_bolt";
pub(crate) const WAND_BOLT: &str = "twilight_wand_bolt";
pub(crate) const SLIME_BLOB: &str = "slime_blob";
pub(crate) const ICE_SNOWBALL: &str = "ice_snowball";
pub(crate) const LICH_BOLT: &str = "lich_bolt";
pub(crate) const LICH_BOMB: &str = "lich_bomb";
pub(crate) const THROWN_AXE: &str = "thrown_axe";
pub(crate) const THROWN_PICK: &str = "thrown_pick";
pub(crate) const UR_FIREBALL: &str = "ur_ghast_fireball";
pub(crate) const MOONWORM_SHOT: &str = "moonworm_shot";
const KNIGHT_PHANTOM: &str = "knight_phantom";

pub(crate) const SHOTS: [Shot; 11] = [
    Shot
    {
        m_name: NATURE_BOLT,
        m_gravity: 0.003,
        m_damage: 2,
        m_kind: kind::MAGIC,
        m_impact: Impact::Nature,
        m_icon: Icon::Sheet(sheet::SEEDS),
        m_puff: Puff::Crack("minecraft:leaves"),
        m_passes: None,
        m_trail: Trail::Specks("happyVillager", 5),
    },
    Shot
    {
        m_name: TOME_BOLT,
        m_gravity: 0.003,
        m_damage: 6,
        m_kind: kind::THROWN,
        m_impact: Impact::Tome,
        m_icon: Icon::Sheet(sheet::PAPER),
        m_puff: Puff::Named("snowballpoof"),
        m_passes: None,
        m_trail: Trail::Specks("crit", 5),
    },
    Shot
    {
        m_name: WAND_BOLT,
        m_gravity: 0.003,
        m_damage: 6,
        m_kind: kind::MAGIC,
        m_impact: Impact::Plain,
        m_icon: Icon::Texture(TWILIGHT_ORB),
        m_puff: Puff::Named("portal"),
        m_passes: None,
        m_trail: Trail::Spell,
    },
    Shot
    {
        m_name: SLIME_BLOB,
        m_gravity: 0.006,
        m_damage: 0,
        m_kind: kind::THROWN,
        m_impact: Impact::Plain,
        m_icon: Icon::Sheet(sheet::SLIME_BALL),
        m_puff: Puff::Named("slime"),
        m_passes: None,
        m_trail: Trail::Specks("slime", 2),
    },
    Shot
    {
        m_name: ICE_SNOWBALL,
        m_gravity: 0.006,
        m_damage: 0,
        m_kind: kind::THROWN,
        m_impact: Impact::Plain,
        m_icon: Icon::Sheet(sheet::SNOWBALL),
        m_puff: Puff::Named("snowballpoof"),
        m_passes: None,
        m_trail: Trail::Specks("snowballpoof", 2),
    },
    Shot
    {
        m_name: LICH_BOLT,
        m_gravity: 0.001,
        m_damage: 6,
        m_kind: kind::MAGIC,
        m_impact: Impact::Plain,
        m_icon: Icon::Texture(TWILIGHT_ORB),
        m_puff: Puff::Named("portal"),
        m_passes: None,
        m_trail: Trail::Spell,
    },
    // beta has no magma cream, the bomb shows the fire charge cell
    Shot
    {
        m_name: LICH_BOMB,
        m_gravity: 0.001,
        m_damage: 0,
        m_kind: kind::THROWN,
        m_impact: Impact::Bomb,
        m_icon: Icon::Sheet(sheet::FIRE),
        m_puff: Puff::Named("smoke"),
        m_passes: None,
        m_trail: Trail::Ember,
    },
    Shot
    {
        m_name: THROWN_AXE,
        m_gravity: 0.001,
        m_damage: 6,
        m_kind: kind::THROWN,
        m_impact: Impact::Plain,
        m_icon: Icon::Texture("twilightforest:item/knightlyaxe"),
        m_puff: Puff::Named("crit"),
        m_passes: Some(KNIGHT_PHANTOM),
        m_trail: Trail::None,
    },
    Shot
    {
        m_name: THROWN_PICK,
        m_gravity: 0.015,
        m_damage: 3,
        m_kind: kind::THROWN,
        m_impact: Impact::Plain,
        m_icon: Icon::Texture("twilightforest:item/knightlypick"),
        m_puff: Puff::Named("crit"),
        m_passes: Some(KNIGHT_PHANTOM),
        m_trail: Trail::None,
    },
    Shot
    {
        m_name: UR_FIREBALL,
        m_gravity: 0.0,
        m_damage: 16,
        m_kind: kind::FIREBALL,
        m_impact: Impact::Fireball,
        m_icon: Icon::Sheet(sheet::FIRE),
        m_puff: Puff::Named("smoke"),
        m_passes: None,
        m_trail: Trail::Smoke,
    },
    Shot
    {
        m_name: MOONWORM_SHOT,
        m_gravity: 0.03,
        m_damage: 0,
        m_kind: kind::THROWN,
        m_impact: Impact::Moonworm,
        m_icon: Icon::Texture("twilightforest:block/tfmoonworm"),
        m_puff: Puff::Crack(crate::ids::block::MOONWORM),
        m_passes: None,
        m_trail: Trail::None,
    },
];

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct Flight
{
    m_owner: Option<i32>,
    m_age: i32,
}

// EntityThrowable.setThrowableHeading toward a point at this speed with its inaccuracy
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
    let entity: Entity = match world.spawnEntity(&format!("twilightforest:{name}"), from)
    {
        Ok(entity) => entity,

        Err(error) =>
        {
            warn(&format!("cannot fire {name}: {}", error.0));
            return None;
        }
    };

    let delta: [f64; 3] = sub(toward, from);
    let length: f64 = length(delta).max(1.0e-4);
    let motion: [f64; 3] = delta.map(|axis| (axis / length + wobble(world, spread)) * speed);
    let _ = entity.setMotion(motion);
    let _ = entity.setData(&Flight { m_owner: owner, m_age: 0 });

    if let Some(owner) = owner
    {
        entity.setField(EntityField::Shooter, f64::from(owner));
    }

    Some(entity)
}

// EntityThrowable.setThrowableHeading's inaccuracy on one axis, uniform where upstream draws a gaussian
pub(crate) fn wobble(world: World, spread: f64) -> f64
{
    let (steps, scale, inaccuracy): (i32, f64, f64) = (2001, 1000.0, 0.0075);
    (f64::from(world.random(steps)) / scale - 1.0) * inaccuracy * spread
}

fn jitter(world: World) -> f64 { trail::SPREAD * f64::from(ids::fraction(world) - ids::fraction(world)) }

fn leaveTrail(kind: Trail, world: World, info: &EntityInfo)
{
    let near = || [0, 1, 2].map(|axis| info.pos[axis] + jitter(world));

    match kind
    {
        Trail::None =>
        {}

        Trail::Specks(name, count) =>
        {
            for _ in 0..count
            {
                crate::particles::spawn(world, name, near(), [0.0; 3]);
            }
        }

        Trail::Spell =>
        {
            for _ in 0..trail::SPELL_SPECKS
            {
                let color: [f64; 3] =
                    trail::SPELL.map(|channel| f64::from((ids::fraction(world) * 0.5 + 0.5) * channel));
                crate::particles::spawn(world, "mobSpell", near(), color);
            }
        }

        Trail::Ember =>
        {
            let step: [f64; 3] = [0, 1, 2].map(|axis| jitter(world) + info.motion[axis]);
            let at: [f64; 3] = [0, 1, 2].map(|axis| info.pos[axis] + step[axis]);
            world.particle("flame", at, step.map(|axis| axis * trail::EMBER_PULL));
        }

        Trail::Smoke => world.particle("smoke", [info.pos[0], info.pos[1] + trail::SMOKE_RISE, info.pos[2]], [0.0; 3]),
    }
}

struct Thrown
{
    m_shot: &'static Shot,
}

mod bolt
{
    // EntityTFNatureBolt's and EntityTFTomeBolt's effect seconds on normal difficulty
    pub const POISON_TICKS: i32 = 3 * 20;
    pub const SLOW_TICKS: i32 = 7 * 20;
}

// what each shot does where it lands, the target first and the block after
fn land(shot: &Shot, entity: Entity, at: [f64; 3], target: Option<&EntityInfo>, owner: Option<i32>)
{
    let world: World = entity.world;

    if let Some(target) = target
        && shot.m_damage > 0
    {
        let victim: Entity = Entity { world, id: target.id };
        crate::combat::hit(victim, shot.m_damage, owner, shot.m_kind);

        // the source's normal difficulty lengths, beta keeps no difficulty a mod can read
        match shot.m_impact
        {
            Impact::Nature => crate::effects::give(victim, crate::effects::Effect::Poison, 0, bolt::POISON_TICKS),
            Impact::Tome => crate::effects::give(victim, crate::effects::Effect::Slowness, 1, bolt::SLOW_TICKS),
            _ =>
            {}
        }
    }

    if shot.m_impact == Impact::Nature && target.is_none()
    {
        let pos: BlockPos = blockOf(at);
        let here: i32 = world.block(pos);
        let info: BlockInfo = World::blockInfo(here);
        // birch leaves grow where the bolt hits stone or wood, grass gets bone meal
        let birch: i32 = 2;

        if here == crate::ids::blockId("minecraft:grass")
        {
            let _ = world.fertilize(pos);
        }
        else if info.solid && info.hardness >= 0.0
        {
            let _ = world.setBlock(pos, crate::ids::blockId("minecraft:leaves"), birch);
        }
    }

    if shot.m_impact == Impact::Bomb
    {
        blast(world, at, effect::BOMB_POWER, owner);
    }

    if shot.m_impact == Impact::Fireball
        && let Err(error) = world.explode(at, effect::FIREBALL_POWER)
    {
        warn(&format!("ur-ghast fireball: {}", error.0));
    }

    for _ in 0..flight::PUFFS
    {
        match shot.m_puff
        {
            Puff::Named(name) => crate::particles::spawn(world, name, at, [0.0; 3]),
            Puff::Crack(block) => crate::particles::crack(world, block, 0, at, [0.0; 3]),
        }
    }

    entity.kill();
}

#[entity_behavior]
impl EntityBehavior for Thrown
{
    // EntityThrowable.onUpdate after the loader moved it, the hit check runs back over this tick's path
    fn onTick(&self, entity: Entity, info: &EntityInfo)
    {
        if !entity.world.isAuthority()
        {
            leaveTrail(self.m_shot.m_trail, entity.world, info);
            return;
        }

        let mut state: Flight = entity.data();
        state.m_age += 1;
        if state.m_age > flight::LIFETIME
        {
            entity.kill();
            return;
        }

        let to: [f64; 3] = info.pos;
        let from: [f64; 3] = sub(to, info.motion);
        let hit: Option<EntityInfo> =
            entity.world.entitiesAlong(from, to, flight::HIT_RADIUS, state.m_owner).into_iter().find(|hit|
            {
                let passed: bool =
                    self.m_shot.m_passes.is_some_and(|kind| hit.kind.as_ref().is_some_and(|hit| hit.path() == kind));
                hit.id != entity.id && hit.health.is_some() && !passed
            });

        if let Some(target) = hit
        {
            land(self.m_shot, entity, to, Some(&target), state.m_owner);
            return;
        }

        if let Some(block) = entity.world.rayTrace(from, to, false)
        {
            if self.m_shot.m_impact == Impact::Moonworm
            {
                crate::items::moonworm::attach(entity.world, block.pos, block.face);
            }

            land(self.m_shot, entity, block.point, None, state.m_owner);
            return;
        }

        let drag: f64 = flight::DRAG;
        let _ = entity.setMotion([
            info.motion[0] * drag,
            info.motion[1] * drag - self.m_shot.m_gravity,
            info.motion[2] * drag,
        ]);

        let _ = entity.setData(&state);
    }

    // RenderSnowball, the icon facing the camera
    fn pose(&self, _input: &PoseInput) -> Pose
    {
        let mut program: Program = Program::new();

        let [u0, v0, u1, v1]: [f32; 4] = match self.m_shot.m_icon
        {
            Icon::Sheet(cell) =>
            {
                program.texture("minecraft:gui/items");

                let (column, row): (f32, f32) = ((cell % 16) as f32, (cell / 16) as f32);
                [column / sheet::CELLS, row / sheet::CELLS, (column + 1.0) / sheet::CELLS, (row + 1.0) / sheet::CELLS]
            }

            Icon::Texture(texture) =>
            {
                program.texture(texture);
                [0.0, 0.0, 1.0, 1.0]
            }
        };
        program.faceCamera().scale(0.5, 0.5, 0.5).normal(0.0, 1.0, 0.0).quad([
            [-0.5, -0.25, 0.0, u0, v1],
            [0.5, -0.25, 0.0, u1, v1],
            [0.5, 0.75, 0.0, u1, v0],
            [-0.5, 0.75, 0.0, u0, v0],
        ]);
        Pose { pre: Vec::new(), model: program.take() }
    }
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
                texture: Some("minecraft:gui/items".to_owned()),
                size: Some([0.25, 0.25]),
                tracking_range: Some(64),
                update_frequency: Some(5),
                send_velocity: Some(true),
                model: Some(EntityModelSpec { shadow: 0.0, ..EntityModelSpec::default() }),
                ..EntitySpec::default()
            },
            Some(std::rc::Rc::new(Thrown { m_shot: shot })),
        )?;
    }

    Ok(())
}

// World.newExplosion with neither fire nor broken blocks, Explosion.doExplosionA's damage to what stands near
pub(crate) fn blast(world: World, at: [f64; 3], power: f32, source: Option<i32>)
{
    let reach: f64 = f64::from(power) * 2.0;

    world.playSound(at, "random.explode", 4.0, 0.7 + crate::ids::fraction(world) * 0.2);
    crate::particles::spawn(world, "hugeexplosion", at, [0.0; 3]);

    for other in world.entitiesNear(at, reach)
    {
        let gap: f64 = distance(at, other.pos) / reach;
        if gap > 1.0 || other.health.is_none()
        {
            continue;
        }

        let impact: f64 = 1.0 - gap;
        let damage: i32 = ((impact * impact + impact) / 2.0 * 8.0 * reach + 1.0) as i32;
        crate::combat::hit(Entity { world, id: other.id }, damage, source, kind::EXPLOSION);
    }
}
