use crate::core::render;
use b173_api::prelude::*;
use std::cell::RefCell;

// TileEntityPickupFX: a sucked up item flies into the obsidian pipe over three ticks on every client that sees it
const CHANNEL: &str = "pipe_pickup";
const TICKS: f32 = 3.0;
const FLOOR: f64 = 0.25;
const HALF: f64 = 0.5;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Pickup
{
    m_world: i32,
    m_from: [f64; 3],
    m_pipe: BlockPos,
    m_stack: crate::core::stacks::Stored,
}

thread_local!
{
    // effects in flight and the ticks each has lived
    static FLYING: RefCell<Vec<(Pickup, f32)>> = const { RefCell::new(Vec::new()) };
}

// TransportProxy.obsidianPipePickup
pub(crate) fn show(world: World, from: [f64; 3], pipe: BlockPos, stack: &ItemStack)
{
    let Some(stored) = crate::core::stacks::Stored::of(stack)
    else
    {
        return;
    };
    let pickup: Pickup = Pickup { m_world: world.dimension, m_from: from, m_pipe: pipe, m_stack: stored };

    if let Err(error) = send(&format!("buildcraft:{CHANNEL}"), Target::Tracking(world.dimension, pipe), &pickup)
    {
        warn(&format!("could not send a pipe pickup: {error}"));
    }
}

// renderParticle: the item eased toward the pipe's floor, lit where it is
fn ops(partial: f32, camera: [f64; 3]) -> Vec<ModelOp>
{
    let Some(local) = localPlayer().and_then(|player| player.info())
    else
    {
        return Vec::new();
    };
    let world: World = World { dimension: local.world };

    FLYING.with(|flying|
    {
        let mut scene: render::Scene = render::Scene::new();

        for (pickup, age) in flying.borrow().iter().filter(|(pickup, _)| pickup.m_world == world.dimension)
        {
            let progress: f64 = f64::from(((age + partial) / TICKS).powi(2));
            let target: [f64; 3] =
                [pickup.m_pipe.x as f64 + HALF, pickup.m_pipe.y as f64 + FLOOR, pickup.m_pipe.z as f64 + HALF];
            let at: [f64; 3] =
                std::array::from_fn(|axis| pickup.m_from[axis] + (target[axis] - pickup.m_from[axis]) * progress);
            let block: BlockPos = BlockPos::new(at[0].floor() as i32, at[1].floor() as i32, at[2].floor() as i32);
            let relative: [f32; 3] = std::array::from_fn(|axis| (at[axis] - camera[axis]) as f32);
            scene.item(relative, &pickup.m_stack.stack(), render::lightAt(world, block));
        }

        scene.finish().pre
    })
}

fn tick()
{
    FLYING.with(|flying|
    {
        let mut flying = flying.borrow_mut();
        flying.iter_mut().for_each(|(_, age)| *age += 1.0);
        flying.retain(|(_, age)| *age < TICKS);
    });
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.channels().register(CHANNEL, Direction::ToClient, |pickup: Pickup, _player|
    {
        FLYING.with(|flying| flying.borrow_mut().push((pickup, 0.0)));
    })?;
    ctx.screens().worldRender(ops)?;
    ctx.events().on::<ClientTick>(Priority::Normal, |_| tick())
}
