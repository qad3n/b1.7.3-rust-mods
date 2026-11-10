use super::{dropAt, killer, showBar};
use crate::entities::ai::distance;
use crate::ids::{self, item};
use crate::math::look;
use crate::models::{Program, tf_hydra, tf_hydra_head, tf_hydra_mortar, tf_hydra_neck};
use crate::progress::{self, Gate};
use b173_api::prelude::*;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

// EntityTFHydra and HydraHeadContainer: a body that only turns, seven heads on necks that bite, breathe fire and
// lob mortars, each head knocked out after enough damage and grown back with a second one
pub const NAME: &str = "hydra";
const LABEL: &str = "Hydra";
const HEAD: &str = "hydra_head";
const NECK: &str = "hydra_neck";
const MORTAR: &str = "hydra_mortar";

const HEADS: usize = 7;
const NECKS: usize = 5;
const STATES: usize = 17;
// the heads that start grown, the other four wait to be regrown
const START_ACTIVE: usize = 3;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
enum State
{
    #[default]
    Idle,
    BiteBeginning,
    BiteReady,
    BiteBiting,
    BiteEnding,
    FlameBeginning,
    FlameBreathing,
    FlameEnding,
    MortarBeginning,
    MortarLaunch,
    MortarEnding,
    Dying,
    Dead,
    AttackCooldown,
    Born,
    RoarStart,
    RoarRawr,
}

// HydraHeadContainer's state table in State order: the state after it and how many ticks it lasts
const FLOW: [(State, i32); STATES] = [
    (State::Idle, 10),
    (State::BiteReady, 40),
    (State::BiteBiting, 80),
    (State::BiteEnding, 7),
    (State::AttackCooldown, 40),
    (State::FlameBreathing, 40),
    (State::FlameEnding, 100),
    (State::AttackCooldown, 30),
    (State::MortarLaunch, 40),
    (State::MortarEnding, 25),
    (State::AttackCooldown, 30),
    (State::Dead, 70),
    (State::Dead, 20),
    (State::Idle, 80),
    (State::RoarStart, 20),
    (State::RoarRawr, 10),
    (State::Idle, 50),
];

// HydraHeadContainer.setupStateRotations by head and state: pitch, yaw, neck length and mouth opening
const POSES: [[[f32; 4]; STATES]; HEADS] = [
    [
        [60.0, 0.0, 7.0, 0.0],
        [-5.0, 60.0, 5.0, 0.25],
        [-5.0, 60.0, 5.0, 1.0],
        [-5.0, -30.0, 5.0, 0.2],
        [60.0, 0.0, 7.0, 0.0],
        [50.0, 0.0, 8.0, 0.75],
        [45.0, 0.0, 8.0, 1.0],
        [60.0, 0.0, 7.0, 0.0],
        [50.0, 0.0, 8.0, 0.75],
        [45.0, 0.0, 8.0, 1.0],
        [60.0, 0.0, 7.0, 0.0],
        [-20.0, 0.0, 7.0, 0.0],
        [0.0, 179.0, 4.0, 0.0],
        [60.0, 0.0, 7.0, 0.0],
        [60.0, 0.0, 7.0, 0.0],
        [60.0, 0.0, 7.0, 0.25],
        [60.0, 0.0, 9.0, 1.0],
    ],
    [
        [10.0, 60.0, 9.0, 0.0],
        [-10.0, 60.0, 9.0, 0.25],
        [-10.0, 60.0, 9.0, 1.0],
        [-10.0, -30.0, 5.0, 0.2],
        [-10.0, 60.0, 9.0, 0.0],
        [30.0, 45.0, 9.0, 0.75],
        [30.0, 60.0, 9.0, 1.0],
        [10.0, 45.0, 9.0, 0.0],
        [30.0, 45.0, 9.0, 0.75],
        [30.0, 60.0, 9.0, 1.0],
        [10.0, 45.0, 9.0, 0.0],
        [-20.0, 60.0, 9.0, 0.0],
        [0.0, 179.0, 4.0, 0.0],
        [10.0, 60.0, 9.0, 0.0],
        [10.0, 60.0, 9.0, 0.0],
        [10.0, 60.0, 9.0, 0.25],
        [10.0, 60.0, 11.0, 1.0],
    ],
    [
        [10.0, -60.0, 9.0, 0.0],
        [-10.0, -60.0, 9.0, 0.25],
        [-10.0, -60.0, 9.0, 1.0],
        [-10.0, 30.0, 5.0, 0.2],
        [-10.0, -60.0, 9.0, 0.0],
        [30.0, -45.0, 9.0, 0.75],
        [30.0, -60.0, 9.0, 1.0],
        [10.0, -45.0, 9.0, 0.0],
        [30.0, -45.0, 9.0, 0.75],
        [30.0, -60.0, 9.0, 1.0],
        [10.0, -45.0, 9.0, 0.0],
        [-20.0, -60.0, 9.0, 0.0],
        [0.0, -180.0, 4.0, 0.0],
        [10.0, -60.0, 9.0, 0.0],
        [10.0, -60.0, 9.0, 0.0],
        [10.0, -60.0, 9.0, 0.25],
        [10.0, -60.0, 11.0, 1.0],
    ],
    [
        [50.0, 90.0, 8.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
        [50.0, 90.0, 8.0, 0.75],
        [50.0, 90.0, 8.0, 1.0],
        [50.0, 90.0, 8.0, 0.0],
        [50.0, 90.0, 8.0, 0.75],
        [50.0, 90.0, 8.0, 1.0],
        [50.0, 90.0, 8.0, 0.0],
        [-20.0, 90.0, 8.0, 0.0],
        [0.0, 179.0, 4.0, 0.0],
        [50.0, 90.0, 8.0, 0.0],
        [50.0, 90.0, 8.0, 0.0],
        [50.0, 90.0, 8.0, 0.25],
        [50.0, 90.0, 10.0, 1.0],
    ],
    [
        [50.0, -90.0, 8.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
        [50.0, -90.0, 8.0, 0.75],
        [50.0, -90.0, 8.0, 1.0],
        [50.0, -90.0, 8.0, 0.0],
        [50.0, -90.0, 8.0, 0.75],
        [50.0, -90.0, 8.0, 1.0],
        [50.0, -90.0, 8.0, 0.0],
        [-20.0, -90.0, 8.0, 0.0],
        [0.0, -180.0, 4.0, 0.0],
        [50.0, -90.0, 8.0, 0.0],
        [50.0, -90.0, 8.0, 0.0],
        [50.0, -90.0, 8.0, 0.25],
        [50.0, -90.0, 10.0, 1.0],
    ],
    [
        [-10.0, 90.0, 9.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
        [-10.0, 90.0, 9.0, 0.75],
        [-10.0, 90.0, 9.0, 1.0],
        [-10.0, 90.0, 9.0, 0.0],
        [-10.0, 90.0, 9.0, 0.75],
        [-10.0, 90.0, 9.0, 1.0],
        [-10.0, 90.0, 9.0, 0.0],
        [-10.0, 90.0, 9.0, 0.0],
        [0.0, 179.0, 4.0, 0.0],
        [-10.0, 90.0, 9.0, 0.0],
        [-10.0, 90.0, 9.0, 0.0],
        [-10.0, 90.0, 9.0, 0.25],
        [-10.0, 90.0, 11.0, 1.0],
    ],
    [
        [-10.0, -90.0, 9.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
        [-10.0, -90.0, 9.0, 0.75],
        [-10.0, -90.0, 9.0, 1.0],
        [-10.0, -90.0, 9.0, 0.0],
        [-10.0, -90.0, 9.0, 0.75],
        [-10.0, -90.0, 9.0, 1.0],
        [-10.0, -90.0, 9.0, 0.0],
        [-10.0, -90.0, 9.0, 0.0],
        [0.0, -180.0, 4.0, 0.0],
        [-10.0, -90.0, 9.0, 0.0],
        [-10.0, -90.0, 9.0, 0.0],
        [-10.0, -90.0, 9.0, 0.25],
        [-10.0, -90.0, 11.0, 1.0],
    ],
];

// HydraHeadContainer.setNeckPosition: where each neck leaves the body and the way it turns from there
const NECK_ROOTS: [([f64; 3], f32); HEADS] = [
    ([0.0, 3.0, -1.0], 0.0),
    ([-1.0, 3.0, 3.0], 90.0),
    ([1.0, 3.0, 3.0], -90.0),
    ([-1.0, 3.0, 3.0], 135.0),
    ([1.0, 3.0, 3.0], -135.0),
    ([-1.0, 3.0, 5.0], 135.0),
    ([1.0, 3.0, 5.0], -135.0),
];

// the idle sway of each head, x and y periods in ticks
const SWAY: [(f32, f32); HEADS] =
    [(20.0, 10.0), (5.0, 6.0), (7.0, 5.0), (20.0, 5.0), (5.0, 10.0), (7.0, 5.0), (7.0, 6.0)];

mod hydra
{
    pub const HEALTH: i32 = 360;
    pub const HEALING_AFTER: i32 = 1000;
    pub const HEAD_RESPAWN: i32 = 100;
    pub const HEAD_MAX_DAMAGE: i32 = 120;
    pub const ARMOR: f32 = 8.0;
    pub const HEADS_ACTIVITY: f32 = 0.3;
    pub const SECONDARY_FLAME: i32 = 10;
    pub const SECONDARY_MORTAR: i32 = 16;
    pub const BITE_CHANCE: i32 = 10;
    pub const FLAME_CHANCE: i32 = 100;
    pub const MORTAR_CHANCE: i32 = 160;
    pub const SEARCH: f64 = 48.0;
    pub const SECONDARY_RANGE: f64 = 20.0;
    // attacks from farther than 20 blocks do nothing
    pub const OUT_OF_REACH: f64 = 20.0;
    pub const FLAME_DAMAGE: i32 = 19;
    pub const FLAME_BURN: f64 = 3.0 * 20.0;
    pub const BITE_DAMAGE: i32 = 48;
    pub const FLAME_TRACKING: f64 = 0.04;
    pub const FLAME_RANGE: f64 = 30.0;
    pub const MOUTH_REACH: f64 = 3.5;
    pub const TURN: f32 = 10.0;
    pub const HEAD_TURN: f32 = 5.0;
    pub const PITCH_LIMIT: f32 = 500.0;
    pub const BODY_BACK: f64 = 3.0;
    pub const TAIL_BACK: f64 = 10.5;
    pub const BODY_SIZE: f64 = 6.0;
    pub const PUSH: f64 = 4.0;
    pub const HEAD_SIZE: f64 = 4.0;
    // the bite aims 16 blocks along the gaze, idle heads look 30 blocks ahead of the body
    pub const BITE_AIM: f64 = 16.0;
    pub const IDLE_LOOK: f64 = 30.0;
    pub const IDLE_TURN: f32 = 1.5;
    // heads ride three blocks over the hydra and hide four under it while dead
    pub const HEAD_RISE: f64 = 3.0;
    pub const HIDDEN: f64 = 4.0;
    pub const WARN_TICK: i32 = 60;
    pub const TARGET_TIME: i32 = 100;
    pub const TARGET_SPREAD: i32 = 20;
    pub const LOOK_CHANCE: f32 = 0.7;
    pub const SPIN_CHANCE: f32 = 0.05;
    pub const SPIN: f32 = 20.0;
    pub const GAZE_RADIUS: f64 = 1.5;
    pub const MORTAR_PERIOD: i32 = 10;
}

mod mortar
{
    pub const SPEED: f64 = 0.75;
    pub const LIFT: f32 = 20.0;
    pub const GRAVITY: f64 = 0.05;
    pub const DRAG: f64 = 0.99;
    pub const FUSE: i32 = 80;
    pub const DAMAGE: i32 = 18;
    pub const BURN: f64 = 5.0 * 20.0;
    pub const SMALL: f32 = 0.1;
    pub const MEGA: f32 = 4.0;
    pub const HIT_RADIUS: f64 = 0.4;
}

#[derive(Clone, Copy, Default)]
struct Head
{
    m_id: Option<i32>,
    m_necks: [Option<i32>; NECKS],
    m_prev: State,
    m_state: State,
    m_next: Option<State>,
    m_needed: i32,
    m_progress: i32,
    m_target: Option<i32>,
    m_aim: [f64; 3],
    m_secondary: bool,
    m_damage: i32,
    m_respawn: i32,
    m_yaw: f32,
    m_pitch: f32,
}

#[derive(Default)]
struct Brain
{
    m_heads: [Head; HEADS],
    m_target: Option<i32>,
    m_targetTime: i32,
    m_quiet: i32,
    m_spin: f32,
    m_ticks: i32,
    // ticks since its health ran out, zero while alive
    m_death: i32,
}

// what the clients draw of a head: its state and how wide its mouth is
#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct HeadShown
{
    m_state: State,
    m_mouth: f32,
}

thread_local!
{
    static BRAINS: RefCell<HashMap<(i32, i32), Brain>> = RefCell::new(HashMap::new());
    // the head and neck entities back to the hydra and head they belong to
    static PARTS: RefCell<HashMap<(i32, i32), (i32, Option<usize>)>> = RefCell::new(HashMap::new());
    // damage the hydra deals itself through its parts skips the body armor
    static DIRECT: Cell<bool> = const { Cell::new(false) };
}

fn wrap(angle: f32) -> f32
{
    let mut angle: f32 = angle % 360.0;
    if angle >= 180.0
    {
        angle -= 360.0;
    }
    if angle < -180.0
    {
        angle += 360.0;
    }
    angle
}

// HydraHeadContainer.updateRotation
fn approach(current: f32, intended: f32, step: f32) -> f32
{
    wrap(current + wrap(intended - current).clamp(-step, step))
}

// Vec3.rotateAroundX and rotateAroundY
fn turnX(vector: [f64; 3], angle: f64) -> [f64; 3]
{
    let (cos, sin): (f64, f64) = (angle.cos(), angle.sin());
    [vector[0], vector[1] * cos + vector[2] * sin, vector[2] * cos - vector[1] * sin]
}

fn turnY(vector: [f64; 3], angle: f64) -> [f64; 3]
{
    let (cos, sin): (f64, f64) = (angle.cos(), angle.sin());
    [vector[0] * cos + vector[2] * sin, vector[1], vector[2] * cos - vector[0] * sin]
}

fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] { [a[0] + b[0], a[1] + b[1], a[2] + b[2]] }

impl Head
{
    fn start(active: bool) -> Head
    {
        let state: State = if active { State::Idle } else { State::Dead };
        let ticks: i32 = if active { 60 } else { 20 };
        Head { m_prev: state, m_state: state, m_needed: ticks, m_progress: ticks, m_respawn: -1, ..Head::default() }
    }

    fn active(&self) -> bool { !matches!(self.m_state, State::Dying | State::Dead) }

    fn attacking(&self) -> bool
    {
        matches!(
            self.m_state,
            State::BiteBeginning
                | State::BiteReady
                | State::BiteBiting
                | State::FlameBeginning
                | State::FlameBreathing
                | State::MortarBeginning
                | State::MortarLaunch
        )
    }

    fn biting(&self) -> bool
    {
        matches!(self.m_state, State::BiteBeginning | State::BiteReady | State::BiteBiting)
            || self.m_next == Some(State::BiteBeginning)
    }

    fn end(&mut self) { self.m_progress = self.m_needed; }

    fn progress(&self) -> f32 { self.m_progress as f32 / self.m_needed.max(1) as f32 }

    // HydraHeadContainer.getCurrentNeckLength and its siblings, one pose value eased from the last state
    fn pose(&self, number: usize, index: usize) -> f32
    {
        let from: f32 = POSES[number][self.m_prev as usize][index];
        let to: f32 = POSES[number][self.m_state as usize][index];
        from + (to - from) * self.progress()
    }

    // HydraHeadContainer.advanceHeadState
    fn advance(&mut self)
    {
        self.m_progress += 1;
        if self.m_progress < self.m_needed
        {
            return;
        }

        let next: State = match self.m_next.take()
        {
            Some(next) => next,
            None =>
            {
                let flowing: State = FLOW[self.m_state as usize].0;
                if flowing != self.m_state && self.m_secondary && flowing == State::AttackCooldown
                {
                    self.m_secondary = false;
                    State::Idle
                }
                else
                {
                    flowing
                }
            }
        };
        self.m_needed = FLOW[next as usize].1;
        self.m_progress = 0;
        self.m_prev = self.m_state;
        self.m_state = next;
    }
}

fn part(world: World, id: Option<i32>) -> Option<Entity> { id.map(|id| Entity { world, id }) }

// EntityTFHydra.onLivingUpdate's first job, the seven heads and their necks spawned around the body
fn spawnParts(entity: Entity, info: &EntityInfo, brain: &mut Brain)
{
    let world: World = entity.world;
    for (number, head) in brain.m_heads.iter_mut().enumerate()
    {
        if head.m_id.is_none()
        {
            *head = Head::start(number < START_ACTIVE);
            head.m_id = spawnPart(world, HEAD, info.pos, entity.id, Some(number));
        }
        for neck in &mut head.m_necks
        {
            if neck.is_none()
            {
                *neck = spawnPart(world, NECK, info.pos, entity.id, Some(number));
            }
        }
    }
}

fn spawnPart(world: World, kind: &str, at: [f64; 3], hydra: i32, head: Option<usize>) -> Option<i32>
{
    match world.spawnEntity(&format!("twilightforest:{kind}"), at)
    {
        Ok(spawned) =>
        {
            PARTS.with(|parts| parts.borrow_mut().insert((world.dimension, spawned.id), (hydra, head)));
            Some(spawned.id)
        }
        Err(error) =>
        {
            warn(&format!("cannot spawn {kind}: {}", error.0));
            None
        }
    }
}

// HydraHeadContainer.setHeadPosition
fn placeHead(info: &EntityInfo, ticks: i32, number: usize, head: &Head) -> [f64; 3]
{
    let (periodX, periodY): (f32, f32) = SWAY[number];
    let (swingX, swingY): (f32, f32) = if head.active()
    {
        ((ticks as f32 / periodX).sin() * 3.0, (ticks as f32 / periodY).sin() * 5.0)
    }
    else
    {
        (0.0, 0.0)
    };
    let length: f32 = head.pose(number, 2);
    // the source scales only the pitch by pi before dividing the sum by 180, its sway barely tilts
    let pitch: f64 = f64::from((head.pose(number, 0) * std::f32::consts::PI + swingX) / 180.0);
    let yaw: f64 = f64::from(-(info.yaw + head.pose(number, 1) + swingY)).to_radians();
    let vector: [f64; 3] = turnY(turnX([0.0, 0.0, f64::from(length)], pitch), yaw);
    [info.pos[0] + vector[0], info.pos[1] + vector[1] + hydra::HEAD_RISE, info.pos[2] + vector[2]]
}

// HydraHeadContainer.faceVec, the head turned a few degrees a tick toward a point
fn faceToward(head: &mut Head, at: [f64; 3], to: [f64; 3], step: f32)
{
    let (dx, dz): (f64, f64) = (to[0] - at[0], to[2] - at[2]);
    let dy: f64 = at[1] + 1.0 - to[1];
    let flat: f64 = (dx * dx + dz * dz).sqrt();
    let yaw: f32 = (dz.atan2(dx).to_degrees() - 90.0) as f32;
    let pitch: f32 = -(dy.atan2(flat).to_degrees() as f32);
    head.m_pitch = -approach(head.m_pitch, pitch, hydra::PITCH_LIMIT);
    head.m_yaw = approach(head.m_yaw, yaw, step);
}

fn eyesOf(world: World, id: i32) -> Option<[f64; 3]>
{
    let info: EntityInfo = Entity { world, id }.info()?;
    let height: f64 = Entity { world, id }.field(EntityField::Height);
    Some([info.pos[0], info.pos[1] + height * 0.85, info.pos[2]])
}

// HydraHeadContainer.setHeadFacing
fn faceHead(world: World, info: &EntityInfo, at: [f64; 3], number: usize, head: &mut Head)
{
    let target: Option<[f64; 3]> = head.m_target.and_then(|id| eyesOf(world, id));
    match head.m_state
    {
        State::BiteReady =>
        {
            if let Some(target) = target
            {
                faceToward(head, at, target, hydra::HEAD_TURN);
            }
            let (high, low): (f32, f32) = if number == 2
            {
                (60.0, 90.0)
            }
            else
            {
                (-60.0, -90.0)
            };
            let offset: f32 = wrap(head.m_yaw - info.yaw);
            if offset > high
            {
                head.m_yaw = info.yaw + high;
            }
            if offset < low
            {
                head.m_yaw = info.yaw + low;
            }
            let aim: [f64; 3] = look(head.m_yaw, head.m_pitch);
            head.m_aim = [
                at[0] + aim[0] * hydra::BITE_AIM,
                at[1] + 1.5 + aim[1] * hydra::BITE_AIM,
                at[2] + aim[2] * hydra::BITE_AIM,
            ];
        }
        State::BiteBiting | State::BiteEnding =>
        {
            if let Some(target) = target
            {
                faceToward(head, at, target, hydra::HEAD_TURN);
            }
            head.m_pitch += 45.0;
        }
        State::RoarRawr => faceToward(head, at, head.m_aim, hydra::TURN),
        State::FlameBeginning | State::FlameBreathing =>
        {
            if let Some(goal) = head.m_target.and_then(|id| Entity { world, id }.info())
            {
                let delta: [f64; 3] = [0, 1, 2].map(|axis| goal.pos[axis] - head.m_aim[axis]);
                let length: f64 = delta.iter().map(|axis| axis * axis).sum::<f64>().sqrt().max(1.0e-4);
                head.m_aim = [0, 1, 2].map(|axis| head.m_aim[axis] + delta[axis] / length * hydra::FLAME_TRACKING);
            }
            let aim: [f64; 3] = head.m_aim;
            faceToward(head, at, aim, hydra::HEAD_TURN);
        }
        _ if head.active() => match (target, head.m_target.and_then(|id| Entity { world, id }.info()))
        {
            (Some(eyes), Some(goal)) =>
            {
                faceToward(head, at, eyes, hydra::HEAD_TURN);
                head.m_aim = goal.pos;
            }
            _ =>
            {
                let angle: f64 = f64::from(info.yaw).to_radians();
                let idle: [f64; 3] = [
                    info.pos[0] - angle.sin() * hydra::IDLE_LOOK,
                    info.pos[1] + hydra::HEAD_RISE,
                    info.pos[2] + angle.cos() * hydra::IDLE_LOOK,
                ];
                faceToward(head, at, idle, hydra::IDLE_TURN);
            }
        },
        _ =>
        {}
    }
}

fn hydraPart(info: &EntityInfo) -> bool
{
    info.kind
        .as_ref()
        .is_some_and(|kind| kind.namespace() == "twilightforest" && [NAME, HEAD, NECK, MORTAR].contains(&kind.path()))
}

// HydraHeadContainer.getHeadLookTarget, the first thing in a line along the head's gaze
fn lookTarget(world: World, at: [f64; 3], head: &Head) -> Option<EntityInfo>
{
    let from: [f64; 3] = [at[0], at[1] + 1.0, at[2]];
    let aim: [f64; 3] = look(head.m_yaw, head.m_pitch);
    let to: [f64; 3] = [0, 1, 2].map(|axis| from[axis] + aim[axis] * hydra::FLAME_RANGE);
    world
        .entitiesAlong(from, to, hydra::GAZE_RADIUS, head.m_id)
        .into_iter()
        .filter(|hit| hit.health.is_some())
        .min_by(|a, b| distance(a.pos, from).total_cmp(&distance(b.pos, from)))
}

// HydraHeadContainer.executeAttacks
fn attack(entity: Entity, at: [f64; 3], head: &mut Head)
{
    let world: World = entity.world;
    let mouth: [f64; 3] =
    {
        let aim: [f64; 3] = look(head.m_yaw, head.m_pitch);
        [
            at[0] + aim[0] * hydra::MOUTH_REACH,
            at[1] + 1.0 + aim[1] * hydra::MOUTH_REACH,
            at[2] + aim[2] * hydra::MOUTH_REACH,
        ]
    };
    match head.m_state
    {
        State::MortarLaunch if head.m_progress % hydra::MORTAR_PERIOD == 0 =>
        {
            if lookTarget(world, at, head).is_some_and(|hit| hydraPart(&hit))
            {
                head.end();
                return;
            }

            let blind: bool = head
                .m_target
                .and_then(|id| eyesOf(world, id))
                .is_some_and(|eyes| world.rayTrace([at[0], at[1] + 1.0, at[2]], eyes, false).is_some());
            launchMortar(world, mouth, head.m_yaw, head.m_pitch, blind, Some(entity.id));
            world.playSound(at, "mob.ghast.fireball", 1.0, 1.0);
        }
        State::BiteBiting =>
        {
            let half: f64 = hydra::HEAD_SIZE / 2.0;
            for near in world.entitiesNear([at[0], at[1] + half, at[2]], half + 1.0)
            {
                if near.health.is_some() && !hydraPart(&near) && (near.pos[1] - at[1]).abs() <= hydra::HEAD_SIZE + 1.0
                {
                    Entity { world, id: near.id }.damage(hydra::BITE_DAMAGE, Some(entity.id));
                }
            }
        }
        State::FlameBreathing =>
        {
            let Some(hit) = lookTarget(world, at, head)
            else
            {
                return;
            };

            if hydraPart(&hit)
            {
                head.end();
                return;
            }
            let victim: Entity = Entity { world, id: hit.id };
            if victim.damage(hydra::FLAME_DAMAGE, None)
            {
                victim.setField(EntityField::FireTicks, hydra::FLAME_BURN);
            }
        }
        _ =>
        {}
    }
}

// EntityThrowable(world, thrower) along the head's gaze tipped 20 degrees up
fn launchMortar(world: World, from: [f64; 3], yaw: f32, pitch: f32, mega: bool, owner: Option<i32>)
{
    let Ok(shell) = world.spawnEntity(&format!("twilightforest:{MORTAR}"), from)
    else
    {
        warn("cannot launch a hydra mortar");
        return;
    };

    let aim: [f64; 3] = look(yaw, pitch - mortar::LIFT);
    let _ = shell.setMotion(aim.map(|axis| axis * mortar::SPEED));
    let _ = shell.setData(&Shell { m_owner: owner, m_mega: mega, m_fuse: mortar::FUSE, m_landed: false });
}

// HydraHeadContainer.addMouthParticles, drawn on the clients from the shown state
fn mouthParticles(world: World, info: &EntityInfo, shown: HeadShown)
{
    let aim: [f64; 3] = look(info.yaw, info.pitch);
    let mouth: [f64; 3] = [
        info.pos[0] + aim[0] * hydra::MOUTH_REACH,
        info.pos[1] + 1.0 + aim[1] * hydra::MOUTH_REACH,
        info.pos[2] + aim[2] * hydra::MOUTH_REACH,
    ];
    let jitter = || [0, 1, 2].map(|axis| mouth[axis] + f64::from(ids::fraction(world)) - 0.5);
    match shown.m_state
    {
        State::FlameBeginning =>
        {
            world.particle("flame", jitter(), [0.0; 3]);
            world.particle("smoke", jitter(), [0.0; 3]);
        }
        State::FlameBreathing =>
        {
            for _ in 0..5
            {
                let speed: f64 = 1.0 + f64::from(ids::fraction(world));
                let spread: f64 = (5.0 + f64::from(ids::fraction(world)) * 2.5) * 0.0075;
                let motion: [f64; 3] =
                    aim.map(|axis| (axis + (f64::from(ids::fraction(world)) - 0.5) * spread * 2.0) * speed);
                world.particle("flame", mouth, motion);
            }
        }
        State::BiteBeginning | State::BiteReady => world.particle("splash", jitter(), [0.0; 3]),
        State::MortarBeginning => world.particle("largesmoke", jitter(), [0.0; 3]),
        _ =>
        {}
    }
}

// HydraHeadContainer.setNeckPositon, five necks eased from behind the head back to the body
fn placeNecks(world: World, info: &EntityInfo, number: usize, at: [f64; 3], head: &Head)
{
    let (root, turn): ([f64; 3], f32) = NECK_ROOTS[number];
    let start: [f64; 3] = add(info.pos, turnY(root, f64::from(-(info.yaw + turn)).to_radians()));
    let behind: [f64; 3] = if head.m_pitch > 0.0
    {
        turnY([0.0, 0.0, -1.0], f64::from(-head.m_yaw).to_radians())
    }
    else
    {
        look(head.m_yaw, head.m_pitch).map(|axis| -axis)
    };
    let end: [f64; 3] = add(at, behind);
    let visible: bool = head.m_state != State::Dead;
    for (index, neck) in head.m_necks.iter().enumerate()
    {
        let Some(neck) = part(world, *neck)
        else
        {
            continue;
        };

        let factor: f64 = index as f64 * 0.25;
        let spot: [f64; 3] = [0, 1, 2].map(|axis| end[axis] + (start[axis] - end[axis]) * factor);
        // a dead head's neck hides under the body
        let spot: [f64; 3] = if visible
        {
            spot
        }
        else
        {
            [info.pos[0], info.pos[1] - hydra::HIDDEN, info.pos[2]]
        };
        let _ = neck.setPosition(spot);
        let _ = neck.setMotion([0.0; 3]);
        neck.setField(EntityField::Yaw, f64::from(head.m_yaw + (info.yaw - head.m_yaw) * factor as f32));
        neck.setField(EntityField::Pitch, f64::from(head.m_pitch - head.m_pitch * factor as f32));
    }
}

// HydraHeadContainer.onUpdate on the server
fn updateHead(entity: Entity, info: &EntityInfo, ticks: i32, number: usize, head: &mut Head)
{
    let world: World = entity.world;
    let Some(headEntity) = part(world, head.m_id)
    else
    {
        return;
    };

    if head.m_state == State::Dead && head.m_respawn > -1
    {
        head.m_respawn -= 1;
        if head.m_respawn <= 0
        {
            head.m_next = Some(State::Born);
            head.m_damage = 0;
            head.end();
            head.m_respawn = -1;
        }
    }
    head.advance();
    let at: [f64; 3] = placeHead(info, ticks, number, head);
    let at: [f64; 3] = if head.m_state == State::Dead
    {
        [info.pos[0], info.pos[1] - hydra::HIDDEN, info.pos[2]]
    }
    else
    {
        at
    };
    faceHead(world, info, at, number, head);
    attack(entity, at, head);
    let _ = headEntity.setPosition(at);
    let _ = headEntity.setMotion([0.0; 3]);
    headEntity.setField(EntityField::Yaw, f64::from(head.m_yaw));
    headEntity.setField(EntityField::Pitch, f64::from(head.m_pitch));
    let shown: HeadShown = HeadShown { m_state: head.m_state, m_mouth: head.pose(number, 3) };
    let old: HeadShown = headEntity.synced();
    if old.m_state != shown.m_state || (old.m_mouth - shown.m_mouth).abs() > 0.05
    {
        let _ = headEntity.setSynced(&shown);
    }
    if head.active()
    {
        destroyAround(world, at, hydra::HEAD_SIZE / 2.0, hydra::HEAD_SIZE);
    }
    placeNecks(world, info, number, at, head);
    if head.m_state == State::RoarRawr && head.m_progress == 1
    {
        world.playSound(at, "twilightforest.mob.hydra.roar", 1.25, 0.7 + ids::fraction(world) * 0.3);
    }
    if head.m_state == State::BiteReady && head.m_progress == hydra::WARN_TICK
    {
        world.playSound(at, "twilightforest.mob.hydra.warn", 2.0, 0.7 + ids::fraction(world) * 0.3);
    }
    if head.m_state == State::FlameBreathing && ticks % 5 == 0
    {
        world.playSound(at, "mob.ghast.fireball", 0.5 + ids::fraction(world), 0.3 + ids::fraction(world) * 0.7);
    }
}

// EntityTFHydra.destroyBlocksInAABB, all but obsidian and bedrock
fn destroyAround(world: World, center: [f64; 3], half: f64, height: f64)
{
    let unbreakable: [i32; 2] = [ids::blockId("minecraft:obsidian"), ids::blockId("minecraft:bedrock")];
    let low: [i32; 3] =
        [(center[0] - half).floor() as i32, center[1].floor() as i32, (center[2] - half).floor() as i32];
    let high: [i32; 3] =
        [(center[0] + half).floor() as i32, (center[1] + height).floor() as i32, (center[2] + half).floor() as i32];
    for x in low[0]..=high[0]
    {
        for y in low[1]..=high[1]
        {
            for z in low[2]..=high[2]
            {
                let pos: BlockPos = BlockPos::new(x, y, z);
                let (block, meta): (i32, i32) = (world.block(pos), world.meta(pos));
                if block != 0 && !unbreakable.contains(&block)
                {
                    let _ = world.setBlock(pos, 0, 0);
                    let _ = world.breakEffect(pos, block, meta);
                }
            }
        }
    }
}

fn activeCount(brain: &Brain) -> usize { brain.m_heads.iter().filter(|head| head.active()).count() }

// EntityTFHydra.areTooManyHeadsAttacking
fn crowded(brain: &Brain, number: usize) -> bool
{
    let busy: usize = brain
        .m_heads
        .iter()
        .enumerate()
        .filter(|(other, head)| *other != number && head.attacking())
        .map(|(_, head)| if head.biting() { 3 } else { 1 })
        .sum();
    busy as f32 >= 1.0 + activeCount(brain) as f32 * hydra::HEADS_ACTIVITY
}

// EntityTFHydra.attackEntity, idle heads pick a bite, a flame or a mortar by range
fn choose(world: World, info: &EntityInfo, target: &EntityInfo, brain: &mut Brain)
{
    let range: f64 = distance(info.pos, target.pos);
    let above: bool = target.pos[1] > info.pos[1] + Entity { world, id: info.id }.field(EntityField::Height);
    for number in 0..HEADS
    {
        if brain.m_heads[number].m_state != State::Idle || crowded(brain, number)
        {
            continue;
        }

        let biter: bool = number < START_ACTIVE;
        let otherBiting: bool = brain.m_heads.iter().enumerate().any(|(other, head)| other != number && head.biting());
        let next: Option<State> = if biter
            && range > 4.0
            && range < 10.0
            && world.random(hydra::BITE_CHANCE) == 0
            && activeCount(brain) > 2
            && !otherBiting
        {
            Some(State::BiteBeginning)
        }
        else if range > 0.0 && range < 20.0 && world.random(hydra::FLAME_CHANCE) == 0
        {
            Some(State::FlameBeginning)
        }
        else if range > 8.0 && range < 32.0 && !above && world.random(hydra::MORTAR_CHANCE) == 0
        {
            Some(State::MortarBeginning)
        }
        else
        {
            None
        };
        if next.is_some()
        {
            brain.m_heads[number].m_next = next;
        }
    }
}

// EntityTFHydra.secondaryAttacks, side heads take on whoever else came along
fn secondary(world: World, info: &EntityInfo, brain: &mut Brain)
{
    if brain.m_heads.iter().any(|head| head.m_id.is_none())
    {
        return;
    }

    let targeted: Vec<i32> = brain.m_heads.iter().filter_map(|head| head.m_target).collect();
    let Some(other) = world
        .entitiesNear(info.pos, hydra::SECONDARY_RANGE)
        .into_iter()
        .filter(|near|
        {
            near.health.is_some() && !hydraPart(near) && Some(near.id) != brain.m_target && !targeted.contains(&near.id)
        })
        .min_by(|a, b| distance(a.pos, info.pos).total_cmp(&distance(b.pos, info.pos)))
    else
    {
        return;
    };

    let range: f64 = distance(other.pos, info.pos);
    for number in 1..HEADS
    {
        let head: &Head = &brain.m_heads[number];
        let Some(headAt) = part(world, head.m_id).and_then(Entity::info)
        else
        {
            continue;
        };

        let flat = |a: [f64; 3], b: [f64; 3]| (a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2);
        if !head.active() || head.m_state != State::Idle || flat(headAt.pos, other.pos) >= flat(info.pos, other.pos)
        {
            continue;
        }

        let next: Option<State> = if range < 20.0 && world.random(hydra::SECONDARY_FLAME) == 0
        {
            Some(State::FlameBeginning)
        }
        else if range > 8.0 && range < 32.0 && world.random(hydra::SECONDARY_MORTAR) == 0
        {
            Some(State::MortarBeginning)
        }
        else
        {
            None
        };
        if next.is_some()
        {
            let head: &mut Head = &mut brain.m_heads[number];
            head.m_target = Some(other.id);
            head.m_secondary = true;
            head.m_next = next;
        }
    }
}

// EntityTFHydra.updateEntityActionState
fn think(entity: Entity, info: &EntityInfo, brain: &mut Brain)
{
    let world: World = entity.world;
    for number in 0..HEADS
    {
        if !brain.m_heads[number].active() || brain.m_heads[number].m_damage <= hydra::HEAD_MAX_DAMAGE
        {
            continue;
        }

        let head: &mut Head = &mut brain.m_heads[number];
        head.m_next = Some(State::Dying);
        head.end();
        head.m_respawn = hydra::HEAD_RESPAWN;
        if let Some(other) = brain.m_heads.iter_mut().find(|head| head.m_state == State::Dead && head.m_respawn == -1)
        {
            other.m_respawn = hydra::HEAD_RESPAWN;
        }
    }

    if ids::fraction(world) < hydra::LOOK_CHANCE
    {
        match crate::entities::closestPlayer(world, info, hydra::SEARCH)
        {
            Some(player) =>
            {
                brain.m_target = Some(player.id);
                brain.m_targetTime = hydra::TARGET_TIME + world.random(hydra::TARGET_SPREAD);
            }
            None => brain.m_spin = (ids::fraction(world) - 0.5) * hydra::SPIN,
        }
    }

    let target: Option<EntityInfo> =
        brain.m_target.and_then(|id| Entity { world, id }.info()).filter(|target| target.alive);
    match &target
    {
        Some(target) =>
        {
            let yaw: f32 =
                ((target.pos[2] - info.pos[2]).atan2(target.pos[0] - info.pos[0]).to_degrees() - 90.0) as f32;
            entity.setField(EntityField::Yaw, f64::from(approach(info.yaw, yaw, hydra::TURN)));
            for head in brain.m_heads.iter_mut()
            {
                if !head.attacking() && !head.m_secondary
                {
                    head.m_target = Some(target.id);
                }
            }
            let seen: bool =
                world.rayTrace([info.pos[0], info.pos[1] + hydra::HEAD_RISE, info.pos[2]], target.pos, false).is_none();
            if seen
            {
                choose(world, info, target, brain);
            }
            brain.m_targetTime -= 1;
            if brain.m_targetTime <= 0 || distance(target.pos, info.pos) > hydra::SEARCH
            {
                brain.m_target = None;
            }
        }
        None =>
        {
            brain.m_target = None;
            if ids::fraction(world) < hydra::SPIN_CHANCE
            {
                brain.m_spin = (ids::fraction(world) - 0.5) * hydra::SPIN;
            }
            entity.setField(EntityField::Yaw, f64::from(wrap(info.yaw + brain.m_spin)));
            for head in brain.m_heads.iter_mut().filter(|head| head.m_state == State::Idle)
            {
                head.m_target = None;
            }
        }
    }
    secondary(world, info, brain);
}

// EntityTFHydra.collideWithEntities, the body and tail shove what stands in them
fn shove(world: World, center: [f64; 3], half: f64)
{
    for near in world.entitiesNear(center, half)
    {
        if near.health.is_none() || hydraPart(&near)
        {
            continue;
        }

        let (dx, dz): (f64, f64) = (near.pos[0] - center[0], near.pos[2] - center[2]);
        let square: f64 = (dx * dx + dz * dz).max(0.01);
        let victim: Entity = Entity { world, id: near.id };
        let _ = victim.setMotion([
            near.motion[0] + dx / square * hydra::PUSH,
            near.motion[1] + 0.2,
            near.motion[2] + dz / square * hydra::PUSH,
        ]);
    }
}

fn body(entity: Entity, info: &EntityInfo, brain: &mut Brain)
{
    let world: World = entity.world;
    brain.m_ticks += 1;
    spawnParts(entity, info, brain);
    brain.m_quiet += 1;
    if brain.m_quiet > hydra::HEALING_AFTER && brain.m_quiet % 5 == 0
    {
        let health: f64 = entity.field(EntityField::Health);
        entity.setField(EntityField::Health, (health + 1.0).min(f64::from(hydra::HEALTH)));
    }
    think(entity, info, brain);
    let _ = entity.setMotion([0.0, info.motion[1], 0.0]);
    let ticks: i32 = brain.m_ticks;
    for number in 0..HEADS
    {
        updateHead(entity, info, ticks, number, &mut brain.m_heads[number]);
    }

    let angle: f64 = f64::from(info.yaw + 180.0).to_radians();
    let back = |by: f64| [info.pos[0] - angle.sin() * by, info.pos[1] + 0.1, info.pos[2] + angle.cos() * by];
    let half: f64 = hydra::BODY_SIZE / 2.0;
    shove(world, back(hydra::BODY_BACK), half);
    shove(world, back(hydra::TAIL_BACK), half);
    destroyAround(world, back(hydra::BODY_BACK), half, hydra::BODY_SIZE);
    destroyAround(world, back(hydra::TAIL_BACK), half, 2.0);
}

// EntityTFHydra.onDeathUpdate, ten seconds of explosions while the heads drop one by one
mod death
{
    pub const LENGTH: i32 = 200;
    pub const HEAD_EVERY: i32 = 20;
    pub const LAST_HEAD: i32 = 140;
    // beta removes a dying entity when its death time reaches twenty, so it is held just under that
    pub const HOLD: f64 = 18.0;
    pub const RELEASE: f64 = 19.0;
    pub const PUFFS: i32 = 20;
    pub const SPREAD: f64 = 0.02;
}

// the heads go quiet, then one more falls every second, then the hydra and its parts are gone
fn dying(entity: Entity, info: &EntityInfo, brain: &mut Brain) -> bool
{
    brain.m_death += 1;
    let time: i32 = brain.m_death;
    if time == 1
    {
        for head in &mut brain.m_heads
        {
            head.m_respawn = -1;
            if head.active()
            {
                head.m_next = Some(State::Idle);
                head.end();
            }
        }
    }
    if time <= death::LAST_HEAD && time % death::HEAD_EVERY == 0
    {
        let head: &mut Head = &mut brain.m_heads[(time / death::HEAD_EVERY - 1) as usize];
        if head.active()
        {
            head.m_next = Some(State::Dying);
            head.end();
        }
    }
    brain.m_ticks += 1;
    let ticks: i32 = brain.m_ticks;
    for number in 0..HEADS
    {
        updateHead(entity, info, ticks, number, &mut brain.m_heads[number]);
    }

    let done: bool = time >= death::LENGTH;
    let hold: f64 = if done
    {
        death::RELEASE
    }
    else
    {
        death::HOLD.min(f64::from(time))
    };
    entity.setField(EntityField::DeathTime, hold);
    done
}

// the clients' share of onDeathUpdate, explosions all over the body
fn deathPuffs(world: World, info: &EntityInfo)
{
    let size: f64 = hydra::BODY_SIZE;
    let jitter =
        || crate::math::gaussian(f64::from(ids::fraction(world)), f64::from(ids::fraction(world))) * death::SPREAD;
    for _ in 0..death::PUFFS
    {
        let name: &str = if world.random(2) == 0
        {
            "largeexplode"
        }
        else
        {
            "explode"
        };
        let at: [f64; 3] = [
            info.pos[0] + f64::from(ids::fraction(world)) * size * 2.0 - size,
            info.pos[1] + f64::from(ids::fraction(world)) * size,
            info.pos[2] + f64::from(ids::fraction(world)) * size * 2.0 - size,
        ];
        crate::particles::spawn(world, name, at, [jitter(), jitter(), jitter()]);
    }
}

// the hydra's heads and necks go with it
fn removeParts(world: World, brain: Brain)
{
    for head in brain.m_heads
    {
        for id in std::iter::once(head.m_id).chain(head.m_necks).flatten()
        {
            if let Some(info) = (Entity { world, id }).info()
            {
                crate::particles::spawn(world, "largeexplode", info.pos, [0.0; 3]);
            }
            Entity { world, id }.kill();
            PARTS.with(|parts| parts.borrow_mut().remove(&(world.dimension, id)));
        }
    }
}

struct Hydra;

#[entity_behavior]
impl EntityBehavior for Hydra
{
    fn onAi(&self, entity: Entity, info: &EntityInfo)
    {
        if !entity.world.isAuthority()
        {
            return;
        }

        BRAINS.with(|brains|
        {
            body(entity, info, brains.borrow_mut().entry((entity.world.dimension, entity.id)).or_default())
        });
        showBar(entity, LABEL, info);
    }

    // EntityTFHydra.attackEntityFromPart for its body, armored and out of reach from afar
    fn onHurt(&self, entity: Entity, source: Option<i32>, amount: i32) -> Option<i32>
    {
        if DIRECT.with(Cell::get)
        {
            return Some(amount);
        }
        if far(entity, source)
        {
            return None;
        }

        BRAINS.with(|brains|
        {
            if let Some(brain) = brains.borrow_mut().get_mut(&(entity.world.dimension, entity.id))
            {
                brain.m_quiet = 0;
            }
        });
        Some((amount as f32 / hydra::ARMOR).round() as i32)
    }

    fn onTick(&self, entity: Entity, info: &EntityInfo)
    {
        let world: World = entity.world;
        if info.health.is_none_or(|health| health > 0)
        {
            return;
        }

        if !world.isAuthority()
        {
            deathPuffs(world, info);
            return;
        }

        let key: (i32, i32) = (world.dimension, entity.id);
        let done: bool =
            BRAINS.with(|brains| brains.borrow_mut().get_mut(&key).is_some_and(|brain| dying(entity, info, brain)));
        if done && let Some(brain) = BRAINS.with(|brains| brains.borrow_mut().remove(&key))
        {
            removeParts(world, brain);
        }
    }

    fn onDeath(&self, entity: Entity, source: Option<i32>)
    {
        let world: World = entity.world;
        let Some(info) = entity.info()
        else
        {
            return;
        };

        if world.isAuthority()
        {
            for _ in 0..world.random(3) + 5
            {
                dropAt(world, info.pos, item::HYDRA_CHOP, 5);
            }
            dropAt(world, info.pos, item::FIERY_BLOOD, world.random(4) + 7);
            dropAt(world, info.pos, item::HYDRA_TROPHY, 1);
            super::conquer(world, info.pos);
        }
        if let Some(player) = killer(world, source)
        {
            progress::grant(player, Gate::Hunter);
            progress::grant(player, Gate::KillHydra);
        }
    }

    // ModelTFHydra.render, the body, legs and tail; the heads and necks are their own entities
    fn pose(&self, _input: &PoseInput) -> Pose
    {
        let mut program: Program = Program::new();
        for part in [
            tf_hydra::BODY,
            tf_hydra::LEG1,
            tf_hydra::LEG2,
            tf_hydra::TAIL1,
            tf_hydra::TAIL2,
            tf_hydra::TAIL3,
            tf_hydra::TAIL4,
        ]
        {
            program.rest(part);
        }
        Pose { pre: Vec::new(), model: program.take() }
    }
}

// EntityTFHydra.calculateRange, attacks from more than 20 blocks away do nothing
fn far(entity: Entity, source: Option<i32>) -> bool
{
    let world: World = entity.world;
    match (source.and_then(|id| Entity { world, id }.info()), entity.info())
    {
        (Some(attacker), Some(info)) => distance(attacker.pos, info.pos) > hydra::OUT_OF_REACH,
        _ => false,
    }
}

// EntityTFHydraPart.attackEntityFrom passed to the hydra: open mouths take full damage, the rest is armored
fn hurtFromPart(entity: Entity, source: Option<i32>, amount: i32)
{
    let world: World = entity.world;
    let Some((hydraId, number)) = PARTS.with(|parts| parts.borrow().get(&(world.dimension, entity.id)).copied())
    else
    {
        return;
    };

    let hydraEntity: Entity = Entity { world, id: hydraId };
    if far(hydraEntity, source)
    {
        return;
    }

    let isHead: bool = entity.info().is_some_and(|info| info.kind.as_ref().is_some_and(|kind| kind.path() == HEAD));
    let dealt: Option<i32> = BRAINS.with(|brains|
    {
        let mut brains = brains.borrow_mut();
        let brain: &mut Brain = brains.get_mut(&(world.dimension, hydraId))?;
        let head: Option<&mut Head> = number.filter(|_| isHead).map(|number| &mut brain.m_heads[number]);
        let dealt: i32 = match head
        {
            Some(head) if !head.active() => return None,
            Some(head) if head.pose(number.unwrap_or(0), 3) > 0.5 =>
            {
                head.m_damage += amount;
                amount
            }
            Some(head) =>
            {
                let armored: i32 = (amount as f32 / hydra::ARMOR).round() as i32;
                head.m_damage += armored;
                armored
            }
            None => (amount as f32 / hydra::ARMOR).round() as i32,
        };
        brain.m_quiet = 0;
        Some(dealt)
    });
    if let Some(dealt) = dealt.filter(|dealt| *dealt > 0)
    {
        DIRECT.with(|direct| direct.set(true));
        hydraEntity.damage(dealt, source);
        DIRECT.with(|direct| direct.set(false));
    }
}

struct Part
{
    m_head: bool,
}

#[entity_behavior]
impl EntityBehavior for Part
{
    // the hydra places its heads and necks, they never move or think on their own
    fn onAi(&self, entity: Entity, _info: &EntityInfo)
    {
        if entity.world.isAuthority()
            && PARTS.with(|parts| !parts.borrow().contains_key(&(entity.world.dimension, entity.id)))
        {
            entity.kill();
        }
    }

    fn onTick(&self, entity: Entity, info: &EntityInfo)
    {
        if self.m_head && !entity.world.isAuthority()
        {
            mouthParticles(entity.world, info, entity.synced());
        }
    }

    fn onHurt(&self, entity: Entity, source: Option<i32>, amount: i32) -> Option<i32>
    {
        if entity.world.isAuthority()
        {
            hurtFromPart(entity, source, amount);
        }
        None
    }

    fn pose(&self, input: &PoseInput) -> Pose
    {
        let mut program: Program = Program::new();
        if !self.m_head
        {
            program.part(
                tf_hydra_neck::NECK,
                [input.head_pitch.to_radians(), input.head_yaw.to_radians(), 0.0],
                tf_hydra_neck::PIVOTS[0],
            );
            return Pose { pre: Vec::new(), model: program.take() };
        }

        let shown: HeadShown = b173_api::fromBytes(&input.synced).unwrap_or_default();
        if shown.m_state == State::Dead
        {
            return Pose { pre: Vec::new(), model: Vec::new() };
        }
        let quarter: f32 = std::f32::consts::PI / 12.0;
        let third: f32 = std::f32::consts::PI / 3.0;
        let pitch: f32 = input.head_pitch.to_radians() - shown.m_mouth * quarter;
        program.part(tf_hydra_head::HEAD, [pitch, input.head_yaw.to_radians(), 0.0], tf_hydra_head::PIVOTS[0]);
        program.part(tf_hydra_head::JAW, [shown.m_mouth * third, 0.0, 0.0], tf_hydra_head::PIVOTS[1]);
        program.part(tf_hydra_head::FRILL, tf_hydra_head::ROTATIONS[2], tf_hydra_head::PIVOTS[2]);
        Pose { pre: Vec::new(), model: program.take() }
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct Shell
{
    m_owner: Option<i32>,
    m_mega: bool,
    m_fuse: i32,
    m_landed: bool,
}

// EntityTFHydraMortar.detonate, a small fiery blast or, when its target hid, a big one
fn detonate(entity: Entity, at: [f64; 3], state: &Shell)
{
    let world: World = entity.world;
    let power: f32 = if state.m_mega
    {
        mortar::MEGA
    }
    else
    {
        mortar::SMALL
    };
    if let Err(error) = world.explode(at, power)
    {
        warn(&format!("hydra mortar blast: {}", error.0));
    }
    for near in world.entitiesNear(at, 1.5)
    {
        if near.health.is_some() && !hydraPart(&near)
        {
            let victim: Entity = Entity { world, id: near.id };
            if victim.damage(mortar::DAMAGE, state.m_owner)
            {
                victim.setField(EntityField::FireTicks, mortar::BURN);
            }
        }
    }
    entity.kill();
}

struct Mortar;

#[entity_behavior]
impl EntityBehavior for Mortar
{
    // EntityTFHydraMortar.onUpdate and onImpact, it rolls to a stop and its fuse burns down
    fn onTick(&self, entity: Entity, info: &EntityInfo)
    {
        let world: World = entity.world;
        if !world.isAuthority()
        {
            world.particle("smoke", info.pos, [0.0; 3]);
            return;
        }

        let mut state: Shell = entity.data();
        if state.m_landed
        {
            let _ = entity.setMotion([info.motion[0] * 0.9, 0.0, info.motion[2] * 0.9]);
            state.m_fuse -= 1;
            if state.m_fuse <= 0
            {
                detonate(entity, info.pos, &state);
                return;
            }
            let _ = entity.setData(&state);
            return;
        }

        let to: [f64; 3] = info.pos;
        let from: [f64; 3] = [0, 1, 2].map(|axis| to[axis] - info.motion[axis]);
        let struck: bool = world
            .entitiesAlong(from, to, mortar::HIT_RADIUS, state.m_owner)
            .iter()
            .any(|hit| hit.health.is_some() && !hydraPart(hit));
        if struck
        {
            detonate(entity, to, &state);
            return;
        }
        if let Some(block) = world.rayTrace(from, to, false)
        {
            if state.m_mega
            {
                detonate(entity, block.point, &state);
                return;
            }
            state.m_landed = true;
            let _ = entity.setPosition(block.point);
            let _ = entity.setMotion([info.motion[0], 0.0, info.motion[2]]);
            let _ = entity.setData(&state);
            return;
        }
        let _ = entity.setMotion([
            info.motion[0] * mortar::DRAG,
            info.motion[1] * mortar::DRAG - mortar::GRAVITY,
            info.motion[2] * mortar::DRAG,
        ]);
    }

    fn pose(&self, _input: &PoseInput) -> Pose
    {
        let mut program: Program = Program::new();
        program.rest(0);
        Pose { pre: Vec::new(), model: program.take() }
    }
}

pub fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.entities().register(
        NAME,
        EntitySpec
        {
            template: Identifier::parse("minecraft:monster").ok(),
            texture: Some("twilightforest:model/hydra4".to_owned()),
            size: Some([hydra::BODY_SIZE as f32, hydra::BODY_SIZE as f32]),
            health: Some(hydra::HEALTH),
            speed: Some(0.0),
            name: Some(LABEL.to_owned()),
            sounds: Some(EntitySounds
            {
                living: Some("twilightforest.mob.hydra.growl".to_owned()),
                hurt: Some("twilightforest.mob.hydra.hurt".to_owned()),
                death: Some("twilightforest.mob.hydra.death".to_owned()),
            }),
            model: Some(EntityModelSpec { shadow: 4.0, ..tf_hydra::model() }),
            fire_immune: true,
            persistent: true,
            immovable: true,
            tracking_range: Some(160),
            ..EntitySpec::default()
        },
        Some(Rc::new(Hydra)),
    )?;
    for (name, label, size, model, head) in [
        (HEAD, "Hydra Head", 4.0, tf_hydra_head::model as fn() -> EntityModelSpec, true),
        (NECK, "Hydra Neck", 2.0, tf_hydra_neck::model as fn() -> EntityModelSpec, false),
    ]
    {
        ctx.entities().register(
            name,
            EntitySpec
            {
                template: Identifier::parse("minecraft:monster").ok(),
                texture: Some("twilightforest:model/hydra4".to_owned()),
                size: Some([size, size]),
                health: Some(hydra::HEALTH),
                name: Some(label.to_owned()),
                model: Some(EntityModelSpec { shadow: 0.0, ..model() }),
                fire_immune: true,
                persistent: true,
                immovable: true,
                tracking_range: Some(160),
                update_frequency: Some(1),
                ..EntitySpec::default()
            },
            Some(Rc::new(Part { m_head: head })),
        )?;
    }
    ctx.entities().register(
        MORTAR,
        EntitySpec
        {
            template: Identifier::parse("minecraft:projectile").ok(),
            texture: Some("twilightforest:model/hydramortar".to_owned()),
            size: Some([0.75, 0.75]),
            tracking_range: Some(64),
            update_frequency: Some(2),
            send_velocity: Some(true),
            model: Some(EntityModelSpec { shadow: 0.5, ..tf_hydra_mortar::model() }),
            ..EntitySpec::default()
        },
        Some(Rc::new(Mortar)),
    )?;
    Ok(())
}
