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
}

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

pub(crate) fn steering(airJumps: i32) -> Option<SteeringSpec>
{
    Some(SteeringSpec
    {
        speed: ride::SPEED,
        jump: ride::FLAP,
        air_jumps: airJumps,
        glide: ride::GLIDE,
        ..SteeringSpec::default()
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
