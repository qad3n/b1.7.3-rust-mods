use super::{Home, home, killer};
use crate::entities::ai::face;
use crate::entities::shots::{self, THROWN_AXE, THROWN_PICK};
use crate::entities::{clear, store};
use crate::models::{Program, bipedPose, combine, tf_knight_phantom, tf_phantom_armor};
use crate::names;
use crate::progress::{self, Gate};
use crate::treasure::{self, Table};
use crate::world::generate::GenWorld;
use b173_api::math::{distance, flatLength, floor, length, sub};
use b173_api::prelude::*;
use std::rc::Rc;

// EntityTFKnightPhantom, six ghostly knights of the stronghold that fly in formations around their spawner and take
// turns charging the player
pub(crate) const NAME: &str = "knight_phantom";
const LABEL: &str = "Knight Phantom";
const KNIGHTS: i32 = 6;

// EntityTFKnightPhantom.getMaxTicksForFormation
mod length
{
    pub const LONG: i32 = 180;
    pub const SHORT: i32 = 90;
    pub const CHARGE: i32 = 50;
    pub const WAIT: i32 = 10;
}

mod knight
{
    // weapons leave from 82 percent of the knight's height
    pub const HAND: f64 = 0.82;
    pub const CHEST: f64 = 0.9;
    pub const HEALTH: i32 = 35;
    pub const ATTACK: i32 = 1;
    pub const CHARGE_BONUS: i32 = 7;
    pub const SMALL_CIRCLE: f64 = 2.5;
    pub const LARGE_CIRCLE: f64 = 8.5;
    pub const SPEED: f64 = 0.1;
    pub const DRAG: f64 = 0.91;
    pub const SEEK: f64 = 9.0;
    pub const SEEK_CHARGING: f64 = 24.0;
    pub const NEARBY: f64 = 32.0;
    pub const HOME_RANGE: f64 = 46.0;
    pub const REACH: f64 = 2.0;
    pub const COOLDOWN: i32 = 20;
    pub const THROW_PERIOD: i32 = 4;
    pub const AXE_SPEED: f64 = 0.75;
    pub const PICK_SPEED: f64 = 0.5;
    pub const PICKS: i32 = 8;
    // TileEntityTFKnightPhantomsSpawner rings the knights four blocks out
    pub const RING: f64 = 4.0;
    pub const RING_STEP: f64 = 60.0;
    pub const BOB: f32 = 7.0;
    // the charge across the hall waits 60 ticks, then sweeps 14 blocks over 120
    pub const SWEEP_WAIT: i32 = 60;
    pub const SWEEP_TICKS: f32 = 120.0;
    pub const SWEEP_SPAN: f32 = 14.0;
    pub const SWEEP_FROM: f32 = -7.0;
    pub const LINE_GAP: f32 = 3.0;
    pub const LINE_FROM: f32 = 7.5;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
enum Formation
{
    #[default]
    Hover,
    LargeClockwise,
    SmallClockwise,
    LargeAnticlockwise,
    SmallAnticlockwise,
    ChargePlusX,
    ChargeMinusX,
    ChargePlusZ,
    ChargeMinusZ,
    WaitingForLeader,
    AttackStart,
    AttackStrike,
}

// EntityTFKnightPhantom.pickRandomFormation, small anticlockwise twice and small clockwise twice
const RANDOM_FORMATIONS: [Formation; 8] = [
    Formation::SmallClockwise,
    Formation::SmallAnticlockwise,
    Formation::SmallAnticlockwise,
    Formation::ChargePlusX,
    Formation::ChargeMinusX,
    Formation::ChargePlusZ,
    Formation::ChargeMinusZ,
    Formation::SmallClockwise,
];

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
enum Weapon
{
    #[default]
    Sword,
    Axe,
    Pick,
}

const WEAPONS: [Weapon; 3] = [Weapon::Sword, Weapon::Axe, Weapon::Pick];

// what each knight keeps between ticks, saved with it
#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct Knight
{
    m_number: i32,
    m_formation: Formation,
    m_ticks: i32,
    m_weapon: Weapon,
    m_charge: [i32; 3],
    m_cooldown: i32,
}

// what the clients see: whether it charges, for the trail of helmets and smoke
#[derive(Serialize, Deserialize, Default, Clone, Copy, PartialEq)]
struct Shown
{
    m_charging: bool,
    m_weapon: Weapon,
}

// the formation a leader hands out reaches every knight this tick
static ORDERS: EntityState<Option<Formation>> = EntityState::new("order");

impl Formation
{
    fn charging(self) -> bool { matches!(self, Self::AttackStart | Self::AttackStrike) }

    // EntityTFKnightPhantom.getMaxTicksForFormation
    fn length(self) -> i32
    {
        match self
        {
            Self::LargeClockwise | Self::LargeAnticlockwise => length::LONG,
            Self::ChargePlusX | Self::ChargeMinusX | Self::ChargePlusZ | Self::ChargeMinusZ => length::LONG,
            Self::AttackStart | Self::AttackStrike => length::CHARGE,
            Self::WaitingForLeader => length::WAIT,
            _ => length::SHORT,
        }
    }
}

fn homeOf(entity: Entity, info: &EntityInfo) -> [f64; 3] { home(entity).map_or(info.pos, |at| at.map(f64::from)) }

fn bob(state: &Knight) -> f64 { f64::from((state.m_ticks as f32 / knight::BOB + state.m_number as f32).cos()) }

// EntityTFKnightPhantom.getDestination and its getCirclePosition, getMoveAcrossPosition and getHoverPosition
fn destination(state: &Knight, info: &EntityInfo, home: [f64; 3]) -> [f64; 3]
{
    let circle = |radius: f64, clockwise: bool|
    {
        let turn: f64 = f64::from(state.m_ticks) * 2.0 * if clockwise { 1.0 } else { -1.0 };
        let angle: f64 = (turn + knight::RING_STEP * f64::from(state.m_number)).to_radians();
        [home[0] + angle.cos() * radius, home[1] + bob(state), home[2] + angle.sin() * radius]
    };

    let across = |plus: bool, alongX: bool|
    {
        let line: f32 = state.m_number as f32 * knight::LINE_GAP - knight::LINE_FROM;

        let mut sweep: f32 = if state.m_ticks < knight::SWEEP_WAIT
        {
            knight::SWEEP_FROM
        }
        else
        {
            knight::SWEEP_FROM + (state.m_ticks - knight::SWEEP_WAIT) as f32 / knight::SWEEP_TICKS * knight::SWEEP_SPAN
        };

        if !plus
        {
            sweep = -sweep;
        }

        let (dx, dz): (f32, f32) = if alongX { (line, sweep) } else { (sweep, line) };
        [home[0] + f64::from(dx), home[1] + bob(state), home[2] + f64::from(dz)]
    };

    let hover = ||
    {
        let (ox, oz): (f64, f64) = (home[0] - info.pos[0], home[2] - info.pos[2]);
        let reach: f64 = flatLength(ox, oz);
        let (x, z): (f64, f64) = if reach > knight::LARGE_CIRCLE
        {
            (home[0] + ox / reach * knight::LARGE_CIRCLE, home[2] + oz / reach * knight::LARGE_CIRCLE)
        }
        else
        {
            (info.pos[0], info.pos[2])
        };
        [x, home[1] + bob(state), z]
    };

    match state.m_formation
    {
        Formation::LargeClockwise => circle(knight::LARGE_CIRCLE, true),
        Formation::SmallClockwise => circle(knight::SMALL_CIRCLE, true),
        Formation::LargeAnticlockwise => circle(knight::LARGE_CIRCLE, false),
        Formation::SmallAnticlockwise => circle(knight::SMALL_CIRCLE, false),
        Formation::ChargePlusX => across(true, true),
        Formation::ChargeMinusX => across(false, true),
        Formation::ChargePlusZ => across(true, false),
        Formation::ChargeMinusZ => across(false, false),
        Formation::AttackStart | Formation::Hover => hover(),
        Formation::AttackStrike if state.m_weapon == Weapon::Sword => state.m_charge.map(f64::from),
        Formation::AttackStrike => hover(),
        Formation::WaitingForLeader => [home[0], home[1] + bob(state), home[2]],
    }
}

fn knightsNear(world: World, at: [f64; 3]) -> Vec<(Entity, Knight)>
{
    let mut found: Vec<(Entity, Knight)> = world
        .entitiesNear(at, knight::NEARBY)
        .into_iter()
        .filter(|near|
        {
            near.alive
                && near.kind.as_ref().is_some_and(|kind| kind.namespace() == "twilightforest" && kind.path() == NAME)
        })
        .map(|near|
        {
            let other: Entity = Entity { world, id: near.id };
            (other, other.data::<Knight>())
        })
        .collect();
    found.sort_by_key(|(_, state)| state.m_number);
    found
}

fn switch(state: &mut Knight, formation: Formation)
{
    state.m_formation = formation;
    state.m_ticks = 0;
}

// EntityTFKnightPhantom.switchToNextFormation
fn nextFormation(entity: Entity, info: &EntityInfo, state: &mut Knight)
{
    let world: World = entity.world;
    let near: Vec<(Entity, Knight)> = knightsNear(world, info.pos);

    match state.m_formation
    {
        Formation::AttackStart => switch(state, Formation::AttackStrike),
        Formation::AttackStrike if near.len() > 1 => switch(state, Formation::WaitingForLeader),

        Formation::AttackStrike =>
        {
            state.m_weapon = WEAPONS[world.random(3) as usize];
            switch(state, Formation::AttackStart);
        }

        Formation::WaitingForLeader => match near.get(1)
        {
            Some((_, other)) =>
            {
                switch(state, other.m_formation);
                state.m_ticks = other.m_ticks;
            }

            None => switch(state, Formation::AttackStart),
        },

        _ if near.iter().all(|(_, other)| other.m_number >= state.m_number) =>
        {
            let formation: Formation = RANDOM_FORMATIONS[world.random(RANDOM_FORMATIONS.len() as i32) as usize];
            switch(state, formation);

            let nobodyCharging: bool = near.iter().all(|(_, other)| !other.m_formation.charging());
            let charger: Option<i32> = nobodyCharging.then(|| near[world.random(near.len() as i32) as usize].0.id);

            for (other, knightState) in &near
            {
                let knight: Entity = Entity { world, id: other.id };
                if Some(other.id) == charger
                {
                    store(&ORDERS, knight, &Some(Formation::AttackStart));
                }
                else if !knightState.m_formation.charging() && other.id != entity.id
                {
                    store(&ORDERS, knight, &Some(formation));
                }
            }

            if charger == Some(entity.id)
            {
                switch(state, Formation::AttackStart);
            }
        }

        _ =>
        {}
    }
}

// EntityTFKnightPhantom.launchAxeAt and launchPicks
fn throwWeapon(entity: Entity, info: &EntityInfo, state: &Knight, target: &EntityInfo)
{
    let world: World = entity.world;
    let height: f64 = entity.field(EntityField::Height) * knight::HAND;
    let pitch: f32 = (crate::math::fraction(world) - crate::math::fraction(world)) * 0.2 + 0.4;
    world.playSound(info.pos, "random.bow", 1.0, pitch);

    match state.m_weapon
    {
        Weapon::Axe =>
        {
            let facing: f64 = f64::from(info.yaw).to_radians();
            let from: [f64; 3] = [info.pos[0] + facing.cos(), info.pos[1] + height, info.pos[2] + facing.sin()];
            let toward: [f64; 3] = [target.pos[0], target.pos[1] + knight::CHEST, target.pos[2]];
            shots::fire(world, THROWN_AXE, from, toward, knight::AXE_SPEED, 1.0, Some(entity.id));
        }

        Weapon::Pick =>
        {
            for i in 0..knight::PICKS
            {
                let angle: f64 = f64::from(i) * std::f64::consts::FRAC_PI_4;
                let from: [f64; 3] = [info.pos[0] + angle.cos(), info.pos[1] + height, info.pos[2] + angle.sin()];
                let toward: [f64; 3] = [from[0] + angle.cos(), from[1], from[2] + angle.sin()];
                shots::fire(world, THROWN_PICK, from, toward, knight::PICK_SPEED, 1.0, Some(entity.id));
            }
        }

        Weapon::Sword =>
        {}
    }
}

// EntityTFKnightPhantom.updateEntityActionState
fn think(entity: Entity, info: &EntityInfo, state: &mut Knight)
{
    let world: World = entity.world;
    if let Some(order) = ORDERS.get(entity)
    {
        clear(&ORDERS, entity);
        switch(state, order);
    }

    state.m_ticks += 1;
    if state.m_ticks >= state.m_formation.length()
    {
        nextFormation(entity, info, state);
    }

    let home: [f64; 3] = homeOf(entity, info);

    let seek: f64 = if state.m_formation.charging()
    {
        knight::SEEK_CHARGING
    }
    else
    {
        knight::SEEK
    };
    let target: Option<EntityInfo> = crate::entities::closestPlayer(world, info, seek);
    if let Some(target) = &target
        && state.m_formation == Formation::AttackStart
    {
        let at: [i32; 3] = target.pos.map(floor);

        state.m_charge = if distance(target.pos, home) < knight::HOME_RANGE
        {
            at
        }
        else
        {
            home.map(|axis| axis as i32)
        };
    }

    let dest: [f64; 3] = destination(state, info, home);
    let delta: [f64; 3] = sub(dest, info.pos);
    let length: f64 = length(delta).max(1.0e-4);
    let motion: [f64; 3] =
        [0, 1, 2].map(|axis| (info.motion[axis] + delta[axis] / length * knight::SPEED) * knight::DRAG);
    let _ = entity.setMotion(motion);

    state.m_cooldown -= 1;

    let Some(target) = target
    else
    {
        return;
    };

    face(entity, info.pos, target.pos);

    let seen: bool = world.rayTrace(info.pos, target.pos, false).is_none();
    if seen && state.m_cooldown <= 0 && distance(info.pos, target.pos) < knight::REACH
    {
        state.m_cooldown = knight::COOLDOWN;

        let bonus: i32 = if state.m_formation.charging()
        {
            knight::CHARGE_BONUS
        }
        else
        {
            0
        };
        crate::combat::hit(
            Entity { world, id: target.id },
            knight::ATTACK + bonus,
            Some(entity.id),
            crate::combat::kind::MOB,
        );
    }

    if state.m_formation == Formation::AttackStrike && state.m_ticks % knight::THROW_PERIOD == 0
    {
        throwWeapon(entity, info, state, &target);
    }
}

// the helmets and smoke trailing a charging knight
fn trail(entity: Entity, info: &EntityInfo)
{
    // drawn by each client itself, the loader would echo server particles to them
    if entity.world.isAuthority()
    {
        return;
    }

    let world: World = entity.world;

    if !entity.synced::<Shown>().m_charging
    {
        return;
    }

    let width: f64 = entity.field(EntityField::Width);
    let height: f64 = entity.field(EntityField::Height);

    for _ in 0..4
    {
        let spot = ||
        {
            let x: f64 =
                info.pos[0] + (f64::from(crate::math::fraction(world) * crate::math::fraction(world)) - 0.5) * width;
            let y: f64 = info.pos[1] + f64::from(crate::math::fraction(world)) * (height - 0.75) + 0.5;
            let z: f64 =
                info.pos[2] + (f64::from(crate::math::fraction(world) * crate::math::fraction(world)) - 0.5) * width;
            [x, y, z]
        };
        world.particle("smoke", spot(), [0.0, 0.1, 0.0]);
        crate::particles::spawn(world, "crit", spot(), [0.0, -0.1, 0.0]);
    }
}

struct KnightPhantom;

#[entity_behavior]
impl EntityBehavior for KnightPhantom
{
    fn onAi(&self, entity: Entity, info: &EntityInfo)
    {
        if !entity.world.isAuthority()
        {
            return;
        }

        let mut state: Knight = entity.data();
        think(entity, info, &mut state);

        let _ = entity.setData(&state);
        let shown: Shown = Shown { m_charging: state.m_formation.charging(), m_weapon: state.m_weapon };
        if entity.synced::<Shown>() != shown
        {
            let _ = entity.setSynced(&shown);
        }
    }

    fn onTick(&self, entity: Entity, info: &EntityInfo) { trail(entity, info); }

    // EntityTFKnightPhantom.attackEntityFrom, a phantom inside a wall does not suffocate
    fn onHurt(&self, _entity: Entity, _source: Option<i32>, amount: i32) -> Option<i32>
    {
        (!crate::combat::hurtBy(crate::combat::kind::IN_WALL)).then_some(amount)
    }

    // EntityTFKnightPhantom.onDeath, the last one to fall leaves the stronghold's treasure at the spawner
    fn onDeath(&self, entity: Entity, source: Option<i32>)
    {
        let world: World = entity.world;
        if let Some(player) = killer(world, source)
        {
            progress::grant(player, Gate::Hunter);
            progress::grant(player, Gate::Knights);
        }

        let Some(info) = entity.info()
        else
        {
            return;
        };

        if !world.isAuthority()
            || knightsNear(world, info.pos).iter().filter(|(other, _)| other.id != entity.id).count() > 0
        {
            return;
        }

        let at: [i32; 3] = home(entity).map_or(info.pos.map(floor), |[x, y, z]| [x, y - 1, z]);
        let generation: GenWorld = GenWorld::new(world);
        treasure::place(
            &generation,
            &b173_api::terrain::Random::withSeed(0),
            Table::StrongholdBoss,
            at[0],
            at[1],
            at[2],
        );
        super::conquer(world, info.pos);
    }

    fn pose(&self, input: &PoseInput) -> Pose
    {
        let mut body: Program = Program::new();
        bipedPose(&mut body, &tf_knight_phantom::PIVOTS[..7], &tf_knight_phantom::ROTATIONS[..7], input);

        let shown: Shown = b173_api::fromBytes(&input.synced).unwrap_or_default();
        let weapon: &str = match shown.m_weapon
        {
            Weapon::Sword => names::item::KNIGHTLY_SWORD,
            Weapon::Axe => names::item::KNIGHTLY_AXE,
            Weapon::Pick => names::item::KNIGHTLY_PICK,
        };
        body.held(crate::models::biped::RIGHT_ARM, weapon);

        let armor: u16 = tf_knight_phantom::PIVOTS.len() as u16;
        body.texture("twilightforest:armor/phantom_1");

        for part in 0..tf_phantom_armor::PIVOTS.len() as u16
        {
            body.part(
                armor + part,
                tf_phantom_armor::ROTATIONS[part as usize],
                tf_phantom_armor::PIVOTS[part as usize],
            );
        }

        Pose { pre: Vec::new(), model: body.take() }
    }
}

// TileEntityTFKnightPhantomsSpawner.spawnMyBoss, six knights in a ring, each with its number and weapon
pub(crate) fn spawnGroup(world: World, pos: BlockPos)
{
    for number in 0..KNIGHTS
    {
        let angle: f64 = (knight::RING_STEP * f64::from(number)).to_radians();
        let at: [f64; 3] = [
            f64::from(pos.x) + 0.5 + angle.cos() * knight::RING,
            f64::from(pos.y) + 0.5,
            f64::from(pos.z) + 0.5 + angle.sin() * knight::RING,
        ];

        let yaw: f32 = crate::math::fraction(world) * 360.0;
        let Some(spawned) = crate::entities::spawnAt(world, &format!("twilightforest:{NAME}"), at, yaw)
        else
        {
            continue;
        };

        let state: Knight = Knight { m_number: number, m_weapon: WEAPONS[(number % 3) as usize], ..Knight::default() };
        if spawned.setData(&state).is_err() || spawned.setData(&Home { m_home: Some([pos.x, pos.y, pos.z]) }).is_err()
        {
            warn("knight phantom lost its number or home");
        }
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.entities().register(
        NAME,
        EntitySpec
        {
            template: Identifier::parse("minecraft:ghast").ok(),
            texture: Some("twilightforest:model/phantomskeleton".to_owned()),
            size: Some([1.5, 3.0]),
            health: Some(knight::HEALTH),
            name: Some(LABEL.to_owned()),
            sounds: Some(EntitySounds
            {
                living: Some("twilightforest.mob.wraith.wraith".to_owned()),
                hurt: Some("twilightforest.mob.wraith.wraith".to_owned()),
                death: Some("twilightforest.mob.wraith.wraith".to_owned()),
            }),

            model: Some(EntityModelSpec
            {
                shadow: 0.75,
                ..combine(vec![tf_knight_phantom::model(), tf_phantom_armor::model()])
            }),
            fire_immune: true,
            persistent: true,
            ..EntitySpec::default()
        },
        Some(Rc::new(KnightPhantom)),
    )?;
    Ok(())
}
