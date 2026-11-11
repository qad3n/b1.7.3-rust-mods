use super::ai::target;
use crate::models::{Program, combine, tf_block_goblin, tf_goblin_chain, tf_spike_block};
use b173_api::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

// EntityTFBlockGoblin swings its spiked block on a chain around itself, wide while it hunts, and the block hurts
// whatever it sweeps through
mod tuning
{
    pub const SPIN: f64 = 16.0;
    pub const FULL_TURN: f64 = 360.0;
    pub const SWINGING: f64 = 0.9;
    pub const RESTING: f64 = 0.3;
    // getChainYOffset, the block hangs lower the longer the chain
    pub const HANG: f64 = 1.5;
    pub const HANG_DROP: f64 = 4.0;
    pub const HIT_REACH: f64 = 0.6;
    pub const DAMAGE: i32 = 8;
    pub const TOSS: f64 = 0.4;
    pub const RECOIL: i32 = 40;
    // the three links sit at these fractions from the goblin's head toward the block
    pub const LINKS: [f64; 3] = [0.4, 0.5, 0.6];
    pub const HEAD_DROP: f64 = 0.1;
    // a third of the spike block's height, where the links aim
    pub const BLOCK_THIRD: f64 = 0.75 / 3.0;
    // RenderLiving's lift from the model origin to the feet, 24 pixels and a sliver
    pub const MODEL_LIFT: f64 = 1.5078125;
    pub const YAW_FLIP: f64 = 180.0;
}

const HEIGHT: f64 = 1.4;

// the beta sound closest to mob.irongolem.throw
const HIT_SOUND: &str = "random.bowhit";

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct Chain
{
    m_swinging: bool,
}

thread_local!
{
    static RECOIL: RefCell<HashMap<(i32, i32), i32>> = RefCell::new(HashMap::new());
}

pub(crate) fn model() -> EntityModelSpec
{
    combine(vec![tf_block_goblin::model(), tf_spike_block::model(), tf_goblin_chain::model()])
}

fn angle(world: World, partial: f64) -> f64 { ((world.time() as f64 + partial) * tuning::SPIN) % tuning::FULL_TURN }

fn length(swinging: bool) -> f64
{
    if swinging
    {
        tuning::SWINGING
    }
    else
    {
        tuning::RESTING
    }
}

// getChainPosition, the block's offset from the goblin's feet
fn blockOffset(angle: f64, length: f64) -> [f64; 3]
{
    let radians: f64 = angle.to_radians();
    [radians.cos() * length, tuning::HANG - length / tuning::HANG_DROP, radians.sin() * length]
}

pub(crate) fn forget(entity: Entity)
{
    RECOIL.with(|recoil| recoil.borrow_mut().remove(&(entity.world.dimension, entity.id)));
}

pub(crate) fn tick(entity: Entity, info: &EntityInfo)
{
    let world: World = entity.world;
    let key: (i32, i32) = (world.dimension, entity.id);
    let recoil: i32 = RECOIL.with(|recoil| recoil.borrow().get(&key).copied().unwrap_or(0)).saturating_sub(1).max(0);
    let swinging: bool = target(entity).is_some() && recoil == 0;
    let shown: Chain = entity.synced();
    if shown.m_swinging != swinging
        && let Err(error) = entity.setSynced(&Chain { m_swinging: swinging })
    {
        warn(&format!("block goblin chain: {}", error.0));
    }

    let mut left: i32 = recoil;

    if swinging
    {
        let offset: [f64; 3] = blockOffset(angle(world, 0.0), length(true));
        let block: [f64; 3] = [0, 1, 2].map(|axis| info.pos[axis] + offset[axis]);
        let struck: Vec<EntityInfo> = world
            .entitiesNear(block, tuning::HIT_REACH)
            .into_iter()
            .filter(|other| other.id != entity.id && other.health.is_some())
            .collect();

        for hit in struck
        {
            let victim: Entity = Entity { world, id: hit.id };
            if crate::combat::hit(victim, tuning::DAMAGE, Some(entity.id), crate::combat::kind::MOB)
            {
                let _ = victim.setMotion([hit.motion[0], hit.motion[1] + tuning::TOSS, hit.motion[2]]);

                world.playSound(block, HIT_SOUND, 1.0, 1.0);
                left = tuning::RECOIL;
            }
        }
    }

    RECOIL.with(|recoil| recoil.borrow_mut().insert(key, left));
}

// a world offset from the goblin's feet in the space its model draws in, undoing RenderLiving's turn and flip
fn modelSpace(offset: [f64; 3], yaw: f64) -> [f32; 3]
{
    let turn: f64 = (tuning::YAW_FLIP - yaw).to_radians();
    let (sin, cos): (f64, f64) = turn.sin_cos();
    let x: f64 = offset[0] * cos - offset[2] * sin;
    let z: f64 = offset[0] * sin + offset[2] * cos;
    [-x as f32, (-offset[1] + tuning::MODEL_LIFT) as f32, z as f32]
}

fn drawAt(body: &mut Program, offset: [f64; 3], yaw: f64, turn: f64, parts: std::ops::Range<u16>)
{
    let [x, y, z]: [f32; 3] = modelSpace(offset, yaw);
    body.push().translate(x, y, z).rotate((turn - yaw) as f32, 0.0, 1.0, 0.0);

    for part in parts
    {
        body.rest(part);
    }

    body.pop();
}

// RenderTFBlockGoblin draws the spike block and the three links with their own renderers
pub(crate) fn draw(body: &mut Program, input: &PoseInput)
{
    let world: World = World { dimension: input.entity.world };
    let chain: Chain = b173_api::fromBytes(&input.synced).unwrap_or_default();
    let spin: f64 = angle(world, f64::from(input.partial));
    let offset: [f64; 3] = blockOffset(spin, length(chain.m_swinging));
    let yaw: f64 = f64::from(input.entity.yaw);
    let goblinParts: u16 = tf_block_goblin::PIVOTS.len() as u16;
    let spikeParts: u16 = tf_spike_block::PIVOTS.len() as u16;
    let linkPart: u16 = goblinParts + spikeParts + tf_goblin_chain::CHAIN;

    drawAt(body, offset, yaw, spin, goblinParts..goblinParts + spikeParts);

    let head: [f64; 3] = [0.0, HEIGHT - tuning::HEAD_DROP, 0.0];
    let reach: [f64; 3] = [head[0] - offset[0], head[1] - offset[1] - tuning::BLOCK_THIRD, head[2] - offset[2]];

    for share in tuning::LINKS
    {
        let link: [f64; 3] = [0, 1, 2].map(|axis| head[axis] - reach[axis] * share);
        drawAt(body, link, yaw, spin, linkPart..linkPart + 1);
    }
}
