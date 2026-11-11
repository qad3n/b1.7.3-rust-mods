use crate::models::{HALF_PI, Program};
use b173_api::prelude::*;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;

pub(crate) const SADDLE: &str = "minecraft:saddle";

// EntityMountable values every Aether mount shares
pub(crate) mod ride
{
    // getMountJumpStrength times the 0.4 jump power onMountedJump gives
    pub(crate) const FLAP: f32 = 5.0 * 0.4;
    pub(crate) const SPEED: f32 = 0.3;
    // EntityPhyg.fall and EntityFlyingCow.fall slow every falling tick
    pub(crate) const GLIDE: f32 = 0.6;
    // getMountedYOffset of the phyg, the flying cow and the moa
    pub(crate) const PHYG_SEAT: f64 = 0.65;
    pub(crate) const COW_SEAT: f64 = 1.15;
    pub(crate) const MOA_SEAT: f64 = 1.25;
}

// the loader keeps a seat height per side, so each side sets it as the mount ticks
pub(crate) fn seat(entity: Entity, height: f64) { entity.setField(EntityField::Seat, height); }

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
pub(crate) struct Saddle
{
    pub m_saddled: bool,
}

// EntitySaddleMount.interact: a saddle goes on, a saddled mount takes the player
pub(crate) fn saddleInteract(entity: Entity, player: Player) -> bool
{
    let mut saddle: Saddle = entity.synced();
    let held: Option<ItemStack> = player.held();

    if !saddle.m_saddled
    {
        let Some(held) = held.filter(|held| held.item.is(SADDLE))
        else
        {
            return false;
        };

        if entity.world.isAuthority()
        {
            saddle.m_saddled = true;

            let _ = entity.setSynced(&saddle);
            let left: i32 = held.count - 1;
            let _ = player.setHeld((left > 0).then_some(ItemStack { count: left, ..held }));
        }

        return true;
    }

    if entity.rider().is_some()
    {
        return false;
    }

    if entity.world.isAuthority()
    {
        let _ = Entity { world: entity.world, id: player.id }.mount(Some(entity.id));
    }

    true
}

pub(crate) fn dropSaddle(entity: Entity)
{
    let saddle: Saddle = entity.synced();
    if let Some(info) = entity.info()
        && saddle.m_saddled
        && entity.world.isAuthority()
    {
        let _ = entity.world.spawnItem(info.pos, ItemStack::of(SADDLE, 1));
    }
}

// EntityLiving.moveEntityWithHeading and EntityMountable values the flight model reuses
mod physics
{
    pub const GROUND_ACCELERATION: f32 = 0.16277136;
    pub const GRAVITY: f64 = 0.08;
    pub const AIR_DRAG: f64 = 0.98;
    // a mount turns its speed into air control at six tenths
    pub const AIR_CONTROL: f32 = 0.6;
    // backing up is a quarter as fast as going forward
    pub const REVERSE: f32 = 0.25;
    pub const STEP_HEIGHT: f64 = 1.0;
}

// how a player ridden mount walks, jumps and glides
#[derive(Clone, Copy)]
pub(crate) struct Steering
{
    pub m_speed: f32,
    pub m_jump: f32,
    // jumps allowed before landing again and the upward speed each gives
    pub m_airJumps: i32,
    pub m_airJump: f32,
    // air jumps only start while the mount already falls
    pub m_airJumpFalling: bool,
    // falling speed is multiplied by this every tick while the rider is not sneaking
    pub m_glide: f32,
}

// the phyg and flying cow flap once in the air and glide down
pub(crate) const WINGED: Steering = Steering
{
    m_speed: ride::SPEED,
    m_jump: ride::FLAP,
    m_airJumps: 1,
    m_airJump: ride::FLAP,
    m_airJumpFalling: false,
    m_glide: ride::GLIDE,
};

#[derive(Default, Clone, Copy)]
struct Flight
{
    m_jumpsLeft: i32,
    m_wasJumping: bool,
}

thread_local!
{
    static FLIGHTS: RefCell<HashMap<i32, Flight>> = RefCell::new(HashMap::new());
}

pub(crate) fn forget(entity: Entity) { FLIGHTS.with(|flights| flights.borrow_mut().remove(&entity.id)); }

// EntityMountable.moveEntityWithHeading for a player ridden mount, false leaves the move to the template
pub(crate) fn travel(mount: Entity, steering: Steering) -> bool
{
    let Some(rider) = mount.rider().map(|id| Entity { world: mount.world, id })
    else
    {
        return false;
    };
    let Some(info) = mount.info()
    else
    {
        return false;
    };

    let ridden: bool = rider.info().is_some_and(|rider| rider.player.is_some());
    if !ridden || mount.flag(EntityField::InWater) || mount.flag(EntityField::InLava)
    {
        return false;
    }

    let onGround: bool = mount.flag(EntityField::OnGround);
    let sneaking: bool = rider.flag(EntityField::Sneaking);
    if onGround && sneaking
    {
        let _ = rider.mount(None);
        return false;
    }

    mount.setField(EntityField::Yaw, rider.field(EntityField::Yaw));
    mount.setField(EntityField::Pitch, rider.field(EntityField::Pitch));
    mount.setField(EntityField::StepHeight, physics::STEP_HEIGHT);

    let strafe: f32 = rider.field(EntityField::MoveStrafe) as f32;
    let input: f32 = rider.field(EntityField::MoveForward) as f32;
    let forward: f32 = if input <= 0.0
    {
        input * physics::REVERSE
    }
    else
    {
        input
    };

    let [x, y, z] = info.motion;
    let motionY: f64 = jump(mount, steering, onGround, rider.flag(EntityField::Jumping), y);
    let falling: bool = motionY < 0.0 && !sneaking;
    let motionY: f64 = if falling
    {
        motionY * steering.m_glide as f64
    }
    else
    {
        motionY
    };
    let _ = mount.setMotion([x, motionY, z]);

    let slipperiness: f32 = mount.field(EntityField::Slipperiness) as f32;
    let factor: f32 = if onGround
    {
        steering.m_speed * physics::GROUND_ACCELERATION / (slipperiness * slipperiness * slipperiness)
    }
    else
    {
        steering.m_speed * physics::AIR_CONTROL
    };
    let _ = mount.accelerate(strafe, forward, factor);

    let Some(moving) = mount.info()
    else
    {
        return true;
    };

    let _ = mount.step(moving.motion);

    let Some(moved) = mount.info()
    else
    {
        return true;
    };

    let [x, y, z] = moved.motion;
    let drag: f64 = slipperiness as f64;
    let _ = mount.setMotion([x * drag, (y - physics::GRAVITY) * physics::AIR_DRAG, z * drag]);
    let _ = mount.animateWalk();
    true
}

// a fresh jump press lifts the mount from the ground or spends an air jump, the motion it leaves
fn jump(mount: Entity, steering: Steering, onGround: bool, jumping: bool, motionY: f64) -> f64
{
    FLIGHTS.with(|flights|
    {
        let mut flights: RefMut<HashMap<i32, Flight>> = flights.borrow_mut();
        let flight: &mut Flight = flights.entry(mount.id).or_default();

        if onGround
        {
            flight.m_jumpsLeft = steering.m_airJumps;
        }

        let pressed: bool = jumping && !flight.m_wasJumping;
        flight.m_wasJumping = jumping;

        let airReady: bool = flight.m_jumpsLeft > 0 && (!steering.m_airJumpFalling || motionY < 0.0);
        if !pressed || !(onGround || airReady)
        {
            return motionY;
        }

        if onGround
        {
            return steering.m_jump as f64;
        }

        flight.m_jumpsLeft -= 1;
        steering.m_airJump as f64
    })
}

mod flap
{
    // the wing angle runs a sine with this period in ticks
    pub const PERIOD: f32 = 31.830988;
    pub const FOLDED: f32 = 0.1;
    pub const SPREAD: f32 = 1.0;
    pub const EASE: f32 = 5.0;
    // wing span in model units and the outer wing's reach past the inner one
    pub const SPAN: f32 = 32.0 / 4.0;
    pub const OUTER: f32 = 3.0;
    pub const SHOULDER: f32 = 4.0;
    pub const HEIGHT: f32 = 12.0;
}

#[derive(Clone, Copy)]
struct Wings
{
    m_fold: f32,
    m_angle: f32,
    m_ticks: f32,
    m_lastTick: i32,
}

thread_local!
{
    static WINGS: RefCell<HashMap<i32, Wings>> = RefCell::new(HashMap::new());
}

// EntityPhyg.onUpdate wing state, stepped on the client once per tick it renders
fn wings(input: &PoseInput) -> (f32, f32)
{
    WINGS.with(|wings|
    {
        let mut wings: RefMut<HashMap<i32, Wings>> = wings.borrow_mut();
        let tick: i32 = input.age as i32;

        let state: &mut Wings = wings.entry(input.entity.id).or_insert(Wings
        {
            m_fold: flap::FOLDED,
            m_angle: 0.0,
            m_ticks: 0.0,
            m_lastTick: tick,
        });
        while state.m_lastTick < tick
        {
            let aim: f32 = if input.on_ground
            {
                flap::FOLDED
            }
            else
            {
                flap::SPREAD
            };
            state.m_ticks += 1.0;
            state.m_angle = state.m_fold * (state.m_ticks / flap::PERIOD).sin();
            state.m_fold += (aim - state.m_fold) / flap::EASE;
            state.m_lastTick += 1;
        }

        (state.m_fold, state.m_angle)
    })
}

// PhygWingModel.render and FlyingCowWingModel.render for an adult, parts are left inner, left outer, right inner, right outer
pub(crate) fn wingPose(program: &mut Program, base: u16, rotations: &[[f32; 3]], input: &PoseInput)
{
    let (fold, angle) = wings(input);
    let bend: f32 = -fold.clamp(-1.0, 1.0).acos();
    let x: f32 = flap::SPAN * fold;
    let y: f32 = -flap::SPAN * (1.0 - fold * fold).max(0.0).sqrt();
    let reach = |x: f32| (x * angle.cos() - y * angle.sin(), x * angle.sin() + y * angle.cos());
    let (innerX, innerY) = reach(x);
    let (outerX, outerY) = reach(x * flap::OUTER);
    let rotation = |part: u16, z: f32| [rotations[part as usize][0], rotations[part as usize][1], z];
    program.part(base + 1, rotation(1, angle - bend + HALF_PI), [flap::SHOULDER + outerX, outerY + flap::HEIGHT, 0.0]);
    program.part(base, rotation(0, angle + bend + HALF_PI), [flap::SHOULDER + innerX, innerY + flap::HEIGHT, 0.0]);
    program.part(
        base + 3,
        rotation(3, -(angle - bend + HALF_PI)),
        [-flap::SHOULDER - outerX, outerY + flap::HEIGHT, 0.0],
    );
    program.part(
        base + 2,
        rotation(2, -(angle + bend - HALF_PI)),
        [-flap::SHOULDER - innerX, innerY + flap::HEIGHT, 0.0],
    );
}
