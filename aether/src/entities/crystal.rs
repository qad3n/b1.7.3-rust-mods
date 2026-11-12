use super::{fraction, lookOf, store};
use crate::effects::Effect;
use crate::models::{DEGREES_PER_RADIAN, Program, crystal as model};
use b173_api::math::{floor, yawTowards};
use b173_api::prelude::*;
use std::f32::consts::TAU;

const NAME: &str = "crystal";

// EnumCrystalType: what a crystal sounds and looks like when it bursts or fades
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Kind
{
    #[default]
    Fire,
    Ice,
    Thunder,
    Cloud,
}

struct KindRow
{
    m_kind: Kind,
    m_texture: &'static str,
    m_fade: (&'static str, &'static str),
    m_burst: (&'static str, &'static str),
}

const KINDS: [KindRow; 4] = [
    KindRow
    {
        m_kind: Kind::Fire,
        m_texture: "aether:entity/crystals/fire",
        m_fade: ("random.fizz", "largesmoke"),
        m_burst: ("random.explode", "flame"),
    },
    KindRow
    {
        m_kind: Kind::Ice,
        m_texture: "aether:entity/crystals/ice",
        m_fade: ("random.glass", ""),
        m_burst: ("random.glass", "snowshovel"),
    },
    KindRow { m_kind: Kind::Thunder, m_texture: "aether:entity/crystals/thunder", m_fade: ("", ""), m_burst: ("", "") },
    KindRow
    {
        m_kind: Kind::Cloud,
        m_texture: "aether:entity/crystals/cloud",
        m_fade: ("random.glass", ""),
        m_burst: ("random.glass", "snowshovel"),
    },
];

fn row(kind: Kind) -> &'static KindRow { KINDS.iter().find(|row| row.m_kind == kind).unwrap_or(&KINDS[0]) }

mod crystal
{
    // EntityCrystal, ice and cloud crystals leave ten ticks of weakness on what they hit
    pub const WEAK_TICKS: i32 = 10;
    pub const BASE_SPEED: f64 = 0.2;
    pub const SPEED_SPREAD: f64 = 0.15;
    pub const SPIN_SPREAD: f32 = 6.0;
    pub const SPIN_STEP: f32 = 0.3;
    pub const SPIN_STAGGER: f32 = 0.13;
    pub const ICE_SLOW: f64 = 3.0;
    pub const DAMAGE: i32 = 5;
    pub const BURN_TICKS: f64 = 100.0;
    pub const HOMING: f64 = 0.02;
    pub const RECOIL: f64 = -0.3;
    pub const PUSHED: f64 = -0.15;
    pub const PUSH_PER_DAMAGE: f64 = 8.0;
    pub const TURN: f32 = 10.0;
    pub const THUNDER_LIFE: f64 = 200.0;
    pub const LIFE: f64 = 300.0;
    pub const TARGET_EYES: f64 = 0.75;
    pub const LEVEL_BAND: f64 = 0.5;
    pub const HOP: f64 = 0.1;
    pub const REACH: f64 = 0.9;
    pub const VOLUME: f32 = 2.0;
    pub const FADE_PUFFS: i32 = 16;
    pub const BURST_PUFFS: i32 = 40;
    pub const PUFF_SPREAD: f64 = 0.25;
    pub const BURST_SPEED: f64 = 0.5;
    pub const LIFT: f32 = 0.3;
    pub const RAISE: f32 = 0.75;
    pub const HEALTH: i32 = 20;
    // World.playSoundAtEntity pitch, nextFloat - nextFloat * 0.2 + 1.2
    pub const PITCH_WOBBLE: f64 = 0.2;
    pub const PITCH: f32 = 1.2;
}

mod angle
{
    pub const HALF: f32 = 180.0;
    pub const FULL: f32 = 360.0;
}

// what clients draw: the kind and where each of the three shells starts spinning
#[derive(Serialize, Deserialize, Default, Clone, Copy)]
pub(crate) struct Look
{
    pub m_kind: Kind,
    m_spin: [f32; 3],
}

// the server's flight, the Java smotion, deflection and homing fields
#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct Flight
{
    m_motion: [f64; 3],
    m_wasHit: bool,
    m_shooter: Option<i32>,
    m_target: Option<i32>,
}

static FLIGHTS: EntityState<Flight> = EntityState::new("flight");

fn flight(entity: Entity) -> Flight { FLIGHTS.get(entity) }

fn setFlight(entity: Entity, state: Flight) { store(&FLIGHTS, entity, &state); }

fn drift(world: World) -> f64
{
    let speed: f64 = crystal::BASE_SPEED + fraction(world) * crystal::SPEED_SPREAD;
    if world.random(2) == 0 { speed } else { -speed }
}

// EntityCrystal constructors: a random drift on each axis, ice keeps level and slower, thunder hunts its target
pub(crate) fn spawn(world: World, kind: Kind, pos: [f64; 3], target: Option<i32>, shooter: Option<i32>)
{
    spawnMoving(world, kind, pos, None, target, shooter);
}

// a crystal sent off along a set motion, as mini clouds fire them, instead of a random drift
pub(crate) fn spawnMoving(
    world: World,
    kind: Kind,
    pos: [f64; 3],
    heading: Option<[f64; 3]>,
    target: Option<i32>,
    shooter: Option<i32>,
)
{
    let mut motion: [f64; 3] = [drift(world), drift(world), drift(world)];
    let spin: [f32; 3] = [0.0; 3].map(|_| fraction(world) as f32 * crystal::SPIN_SPREAD);

    if kind == Kind::Ice
    {
        motion = [motion[0] / crystal::ICE_SLOW, 0.0, motion[2] / crystal::ICE_SLOW];
    }

    let Ok(entity) = world.spawnEntity(&format!("aether:{NAME}"), pos)
    else
    {
        return;
    };

    let _ = entity.setSynced(&Look { m_kind: kind, m_spin: spin });

    setFlight(
        entity,
        Flight
        {
            m_motion: heading.unwrap_or(motion),
            m_wasHit: kind == Kind::Cloud,
            m_shooter: shooter,
            m_target: target,
        },
    );
}

fn alive(world: World, id: Option<i32>) -> Option<EntityInfo>
{
    id.and_then(|id| Entity { world, id }.info()).filter(|info| info.alive)
}

// EntityCrystal.moveTowardsTarget along the crystal's own heading, rising or sinking toward the target's chest
fn moveTowards(entity: Entity, info: &EntityInfo, target: &EntityInfo, speed: f64)
{
    let heading: f64 = (entity.field(EntityField::Yaw)).to_radians();
    let bottom: f64 = entity.field(EntityField::Bottom);
    let mut motion: [f64; 3] = info.motion;
    motion[0] -= heading.sin() * speed;
    motion[2] += heading.cos() * speed;

    let chest: f64 = target.pos[1] - crystal::TARGET_EYES;
    if chest < bottom - crystal::LEVEL_BAND
    {
        motion[1] -= speed / 2.0;
    }
    else if chest > bottom + crystal::LEVEL_BAND
    {
        motion[1] += speed / 2.0;
    }
    else
    {
        motion[1] += (chest - bottom) * (speed / 2.0);
    }

    if entity.flag(EntityField::OnGround)
    {
        entity.setField(EntityField::OnGround, 0.0);
        motion[1] = crystal::HOP;
    }

    let _ = entity.setMotion(motion);
}

// EntityLiving.faceEntity limited to ten degrees a tick
fn face(entity: Entity, info: &EntityInfo, target: &EntityInfo)
{
    let wanted: f32 = yawTowards(info.pos, target.pos);
    let current: f32 = info.yaw;
    let mut turn: f32 = (wanted - current) % angle::FULL;
    if turn >= angle::HALF
    {
        turn -= angle::FULL;
    }

    if turn < -angle::HALF
    {
        turn += angle::FULL;
    }

    entity.setField(EntityField::Yaw, (current + turn.clamp(-crystal::TURN, crystal::TURN)) as f64);
}

fn puffs(world: World, at: [f64; 3], particle: &str, count: i32, spread: f64, speed: f64)
{
    if particle.is_empty()
    {
        return;
    }

    for _ in 0..count
    {
        let signed = || fraction(world) * 2.0 - 1.0;
        let offset = || signed() * spread;
        let velocity = || signed() * speed;
        world.particle(
            particle,
            [at[0] + offset(), at[1] + offset(), at[2] + offset()],
            [velocity(), velocity(), velocity()],
        );
    }
}

fn soundPitch(world: World) -> f32
{
    (fraction(world) - fraction(world) * crystal::PITCH_WOBBLE) as f32 + crystal::PITCH
}

// EntityCrystal.explode and expire, then gone
fn burst(entity: Entity, info: &EntityInfo, kind: Kind)
{
    let world: World = entity.world;
    let row: &KindRow = row(kind);
    if !row.m_burst.0.is_empty()
    {
        world.playSound(info.pos, row.m_burst.0, crystal::VOLUME, soundPitch(world));
    }

    let speed: f64 = if matches!(kind, Kind::Ice | Kind::Cloud)
    {
        crystal::BURST_SPEED / 2.0
    }
    else
    {
        crystal::BURST_SPEED
    };
    puffs(world, info.pos, row.m_burst.1, crystal::BURST_PUFFS, 0.0, speed);
    fade(entity, info, kind);
}

fn fade(entity: Entity, info: &EntityInfo, kind: Kind)
{
    let world: World = entity.world;
    let row: &KindRow = row(kind);
    if !row.m_fade.0.is_empty()
    {
        world.playSound(info.pos, row.m_fade.0, crystal::VOLUME, soundPitch(world));
    }

    puffs(world, info.pos, row.m_fade.1, crystal::FADE_PUFFS, crystal::PUFF_SPREAD, 0.0);
    entity.kill();
}

fn isBoss(info: &EntityInfo) -> bool
{
    info.kind.as_ref().is_some_and(|kind|
    {
        kind.namespace() == "aether"
            && [super::bosses::SLIDER, super::bosses::VALKYRIE_QUEEN, super::bosses::SUN_SPIRIT].contains(&kind.path())
    })
}

fn isMinion(info: &EntityInfo) -> bool
{
    info.kind.as_ref().is_some_and(|kind| kind.namespace() == "aether" && kind.path() == "fire_minion")
}

// EntityCrystal.applyEntityCollision against everything living it touches
fn touch(entity: Entity, info: &EntityInfo, look: Look, state: &mut Flight) -> bool
{
    let world: World = entity.world;
    for other in world
        .entitiesNear(info.pos, crystal::REACH)
        .into_iter()
        .filter(|other| other.id != entity.id && other.health.is_some())
    {
        let victim: Entity = Entity { world, id: other.id };

        let hits: bool = match look.m_kind
        {
            Kind::Fire => !isBoss(&other) && !isMinion(&other),
            Kind::Ice => state.m_wasHit,
            Kind::Cloud => !isBoss(&other),
            Kind::Thunder => state.m_target == Some(other.id),
        };
        if !hits || other.kind.as_ref().is_some_and(|kind| kind.path() == NAME)
        {
            continue;
        }
        if !crate::damage::hit(victim, crystal::DAMAGE, Some(entity.id), crate::damage::kind::THROWN)
        {
            continue;
        }

        match look.m_kind
        {
            Kind::Thunder =>
            {
                moveTowards(entity, info, &other, crystal::RECOIL);
                continue;
            }

            Kind::Fire => victim.setField(EntityField::FireTicks, crystal::BURN_TICKS),
            Kind::Ice | Kind::Cloud => crate::effects::give(victim, Effect::Weakness, crystal::WEAK_TICKS),
        }

        burst(entity, info, look.m_kind);
        return true;
    }

    false
}

// bouncing off the block in the way on each axis it moves along
fn bounce(world: World, info: &EntityInfo, bottom: f64, motion: &mut [f64; 3])
{
    let [x, y, z] = [floor(info.pos[0]), floor(bottom), floor(info.pos[2])];
    let solid = |dx: i32, dy: i32, dz: i32| !world.isAir(BlockPos::new(x + dx, y + dy, z + dz));
    let checks: [[i32; 3]; 3] = [[1, 0, 0], [0, 1, 0], [0, 0, 1]];
    for (axis, step) in checks.iter().enumerate()
    {
        let sign: i32 = if motion[axis] > 0.0
        {
            1
        }
        else if motion[axis] < 0.0
        {
            -1
        }
        else
        {
            0
        };

        if sign != 0 && solid(step[0] * sign, step[1] * sign, step[2] * sign)
        {
            motion[axis] = -motion[axis];
        }
    }
}

struct Crystal;

#[entity_behavior]
impl EntityBehavior for Crystal
{
    // no ghast AI, a crystal only drifts or hunts
    fn onAi(&self, _entity: Entity, _info: &EntityInfo) {}

    // EntityCrystal.onUpdate
    fn onTick(&self, entity: Entity, info: &EntityInfo)
    {
        let world: World = entity.world;
        if !world.isAuthority()
        {
            return;
        }

        let look: Look = entity.synced();
        let mut state: Flight = flight(entity);

        if look.m_kind == Kind::Thunder
        {
            let Some(target) = alive(world, state.m_target)
            else
            {
                fade(entity, info, look.m_kind);
                return;
            };
            face(entity, info, &target);
            moveTowards(entity, info, &target, crystal::HOMING);
        }
        else
        {
            if entity.flag(EntityField::Collided)
            {
                if state.m_wasHit && look.m_kind != Kind::Cloud
                {
                    burst(entity, info, look.m_kind);
                    return;
                }

                bounce(world, info, entity.field(EntityField::Bottom), &mut state.m_motion);
            }

            let _ = entity.setMotion(state.m_motion);
        }

        let life: f64 = if look.m_kind == Kind::Thunder
        {
            crystal::THUNDER_LIFE
        }
        else
        {
            crystal::LIFE
        };

        if entity.field(EntityField::TicksExisted) >= life
        {
            if look.m_kind == Kind::Thunder
            {
                let _ = world.lightning(info.pos);
            }

            fade(entity, info, look.m_kind);
            return;
        }

        if touch(entity, info, look, &mut state)
        {
            return;
        }

        setFlight(entity, state);
    }

    // EntityCrystal.attackEntityFrom: a hit knocks a thunder ball back and turns any other crystal the way the hitter faces
    fn onHurt(&self, entity: Entity, source: Option<i32>, amount: i32) -> Option<i32>
    {
        let world: World = entity.world;
        let attacker: EntityInfo = source.and_then(|id| Entity { world, id }.info())?;

        if !world.isAuthority()
        {
            return None;
        }

        let info: EntityInfo = entity.info()?;
        let look: Look = entity.synced();
        let mut state: Flight = flight(entity);

        if look.m_kind == Kind::Thunder
        {
            moveTowards(entity, &info, &attacker, crystal::PUSHED - amount as f64 / crystal::PUSH_PER_DAMAGE);
            return Some(amount);
        }

        let aim: [f64; 3] = lookOf(&attacker);
        state.m_motion = [
            aim[0],
            if look.m_kind == Kind::Ice
            {
                state.m_motion[1]
            }
            else
            {
                aim[1]
            },
            aim[2],
        ];
        state.m_shooter = Some(attacker.id);
        state.m_wasHit = true;
        setFlight(entity, state);
        None
    }

    // CrystalRenderer and CrystalModel: three translucent shells each spinning on its own axis
    fn pose(&self, input: &PoseInput) -> Pose
    {
        let look: Look = b173_api::fromBytes(&input.synced).unwrap_or_default();
        let head: [f32; 3] = [input.head_pitch / DEGREES_PER_RADIAN, input.head_yaw / DEGREES_PER_RADIAN, 0.0];
        let axes: [[f32; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let mut pre: Program = Program::new();
        pre.translate(0.0, crystal::LIFT, 0.0);

        let mut body: Program = Program::new();
        body.texture(row(look.m_kind).m_texture).translate(0.0, crystal::RAISE, 0.0).blend(true);

        for (index, axis) in axes.iter().enumerate()
        {
            let spin: f32 =
                (look.m_spin[index] + input.age * (crystal::SPIN_STEP + index as f32 * crystal::SPIN_STAGGER)) % TAU;
            body.push().rotate(spin * DEGREES_PER_RADIAN, axis[0], axis[1], axis[2]);
            body.part(model::HEAD_0 + index as u16, head, model::PIVOTS[index]).pop();
        }

        body.blend(false);
        Pose { pre: pre.take(), model: body.take() }
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.entities().register(
        NAME,
        EntitySpec
        {
            template: Identifier::parse("minecraft:ghast").ok(),
            texture: Some(KINDS[0].m_texture.to_owned()),
            size: Some([0.9, 0.9]),
            health: Some(crystal::HEALTH),
            fire_immune: true,
            drops: Some(Vec::new()),
            sounds: super::sounds("", "", ""),
            model: Some(EntityModelSpec { shadow: 0.25, ..model::model() }),
            ..EntitySpec::default()
        },
        Some(std::rc::Rc::new(Crystal)),
    )?;
    Ok(())
}
