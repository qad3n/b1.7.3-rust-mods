use super::{Kind, diamond, joins, kindAt, output};
use crate::core::stacks::Stored;
use crate::core::{dir, random};
use b173_api::prelude::*;

// Utils.pipeNormalSpeed and getPipeFloorOf: items ride a quarter block above the pipe's floor
pub(crate) const NORMAL_SPEED: f32 = 0.01;
pub(crate) const FLOOR: f64 = 0.25;
const GOLD_BOOST: f32 = 20.0;
const HALF: f64 = 0.5;

// TilePipe.EntityData around an EntityPassiveItem: an item on its way through one pipe
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub(crate) struct Traveler
{
    pub(crate) m_id: u32,
    pub(crate) m_stack: Stored,
    pub(crate) m_pos: [f64; 3],
    pub(crate) m_speed: f32,
    pub(crate) m_dir: i32,
    pub(crate) m_toCenter: bool,
    // EntityPassiveItem.deterministicRandomization, so both sides pick the same way out
    pub(crate) m_spin: i32,
    // EntityPassiveItem.synchroTracker, a server tells clients about an item at most once a second
    pub(crate) m_sync: crate::core::power::Tracker,
}

pub(crate) fn center(pos: BlockPos) -> [f64; 3] { [pos.x as f64 + HALF, pos.y as f64 + FLOOR, pos.z as f64 + HALF] }

// TileGoldenPipe.entityEntering then TilePipe.readjustSpeed: a powered golden pipe throws items, a stone pipe slows
// them half as fast
pub(crate) fn adjustSpeed(kind: Kind, world: World, pos: BlockPos, speed: f32) -> f32
{
    let speed: f32 = if kind == Kind::Golden && world.isPowered(pos)
    {
        NORMAL_SPEED * GOLD_BOOST
    }
    else
    {
        speed
    };
    let slowdown: f32 = if kind == Kind::Stone
    {
        NORMAL_SPEED / 2.0
    }
    else
    {
        NORMAL_SPEED
    };
    let slowed: f32 = if speed > NORMAL_SPEED
    {
        speed - slowdown
    }
    else
    {
        speed
    };
    slowed.max(NORMAL_SPEED)
}

pub(crate) fn middleReached(pos: BlockPos, item: &Traveler) -> bool
{
    let limit: f64 = f64::from(item.m_speed * 1.01);
    let middle: [f64; 3] = center(pos);
    (0..3).all(|axis| (middle[axis] - item.m_pos[axis]).abs() < limit)
}

fn outside(pos: BlockPos, item: &Traveler, margin: f64) -> bool
{
    let base: [f64; 3] = crate::core::render::corner(pos);
    (0..3).any(|axis| item.m_pos[axis] > base[axis] + 1.0 + margin || item.m_pos[axis] < base[axis] - margin)
}

pub(crate) fn endReached(pos: BlockPos, item: &Traveler) -> bool { outside(pos, item, 0.0) }

pub(crate) fn outOfBounds(pos: BlockPos, item: &Traveler) -> bool { outside(pos, item, 1.0) }

// TilePipe.canReceivePipeObjects: a joined pipe always, an inventory only with room for the item
fn receives(world: World, pos: BlockPos, face: i32, item: &Traveler) -> bool
{
    if !joins(world, pos, face)
    {
        return false;
    }

    let target: BlockPos = dir::step(pos, face);
    if kindAt(world, target).is_some()
    {
        return true;
    }

    let stack: ItemStack = item.m_stack.stack();
    crate::core::stacks::insert(world, target, dir::reverse(face), &stack, true) < stack.count
}

// TilePipe.getPossibleMovements and the iron and diamond overrides
pub(crate) fn exits(kind: Kind, world: World, pos: BlockPos, item: &Traveler) -> Vec<i32>
{
    match kind
    {
        Kind::Iron =>
        {
            let Some(output) = output(world, pos)
            else
            {
                return Vec::new();
            };
            let target: BlockPos = dir::step(pos, output);
            let open: bool = kindAt(world, target).is_some()
                || world.capabilities(target, dir::reverse(output)) & capability::ITEMS != 0;
            if open { vec![output] } else { Vec::new() }
        }

        Kind::Diamond =>
        {
            let open: Vec<i32> = plain(world, pos, item);
            diamond::filter(world, pos, item, open)
        }

        _ => plain(world, pos, item),
    }
}

fn plain(world: World, pos: BlockPos, item: &Traveler) -> Vec<i32>
{
    dir::all()
        .filter(|face| *face != dir::reverse(item.m_dir))
        .filter(|face| receives(world, pos, *face, item))
        .collect()
}

// TilePipe.resolveDestination in its deterministic form, the way BuildCraft's server and client agree
pub(crate) fn destination(kind: Kind, world: World, pos: BlockPos, item: &Traveler) -> i32
{
    let exits: Vec<i32> = exits(kind, world, pos, item);
    if exits.is_empty()
    {
        return dir::UNKNOWN;
    }

    // single player picks at random, a server and its clients pick the same way from the item's numbers
    if singlePlayer(world)
    {
        return exits[world.random(exits.len() as i32) as usize];
    }

    let seed: i64 =
        i64::from(item.m_id) + i64::from(pos.x) + i64::from(pos.y) + i64::from(pos.z) + i64::from(item.m_spin);
    exits[(seed.unsigned_abs() % exits.len() as u64) as usize]
}

// APIProxy.isClient and isServerSide both false: the world is owned here and drawn here
pub(crate) fn singlePlayer(world: World) -> bool { world.isAuthority() && isClient() }

// APIProxy.isServerSide, a dedicated server owning the world
pub(crate) fn dedicated(world: World) -> bool { world.isAuthority() && !isClient() }

// EntityPassiveItem.toEntityItem: thrown out along its way, the id goes to the obsidian pipe's ignore list
pub(crate) fn toEntity(world: World, item: &Traveler) -> Option<i32>
{
    const PUSH: f64 = 0.1;
    const WOBBLE: f64 = 0.02;
    const DELAY: i32 = 20;

    if !world.isAuthority()
    {
        return None;
    }

    let push: [f64; 3] = dir::forward([0.0; 3], item.m_dir, PUSH + f64::from(item.m_speed) * 2.0);
    let wobble: f64 = random::float(world) * WOBBLE * 2.0 - WOBBLE;
    let motion: [f64; 3] = std::array::from_fn(|axis| random::gaussian(world) * wobble + push[axis]);

    match world.dropItem(item.m_pos, &item.m_stack.stack(), motion, DELAY)
    {
        Ok(entity) => Some(entity.id),

        Err(error) =>
        {
            warn(&format!("a pipe could not drop its item: {error}"));
            None
        }
    }
}
