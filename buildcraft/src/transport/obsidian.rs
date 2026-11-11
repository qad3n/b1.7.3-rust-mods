use super::joins;
use super::pipe::{self, PipeState};
use crate::core::power::Provider;
use crate::core::{dir, random};
use b173_api::prelude::*;

// TileObsidianPipe: an obsidian pipe with one open end sucks items and arrows up to four blocks out of it
const ITEM: &str = "minecraft:item";
const ARROW: &str = "minecraft:arrow";
const MINECART: &str = "minecraft:minecart";
const REACH: i32 = 4;
const POP: &str = "random.pop";
const POP_VOLUME: f32 = 0.2;
const POP_SPREAD: f32 = 0.7;
const POP_PITCH: f32 = 2.0;

// getSuckingOrientation: the face opposite the pipe's only connection
fn suckingFace(world: World, pos: BlockPos) -> i32
{
    let open: Vec<i32> = dir::all().filter(|face| joins(world, pos, *face)).collect();
    match open.as_slice()
    {
        [face] => dir::reverse(*face),
        _ => dir::UNKNOWN,
    }
}

// getSuckingBox, a slab one block deep at the distance, widening as it goes
fn suckingBox(pos: BlockPos, face: i32, distance: i32) -> Option<([f64; 3], [f64; 3])>
{
    if face == dir::UNKNOWN
    {
        return None;
    }

    let base: [f64; 3] = crate::core::render::corner(pos);
    let reach: f64 = f64::from(distance);
    let (mut first, mut second): ([f64; 3], [f64; 3]) = (base, base);

    // the along axis steps out by the distance, the two across axes widen by it on both sides
    let along: usize = match face
    {
        dir::WEST | dir::EAST => 0,
        dir::DOWN | dir::UP => 1,
        _ => 2,
    };

    if face % 2 == 1
    {
        first[along] += reach;
        second[along] += reach + 1.0;
    }
    else
    {
        first[along] -= reach - 1.0;
        second[along] -= reach;
    }

    for axis in (0..3).filter(|axis| *axis != along)
    {
        first[axis] += reach + 1.0;
        second[axis] -= reach;
    }

    let min: [f64; 3] = std::array::from_fn(|axis| first[axis].min(second[axis]));
    let max: [f64; 3] = std::array::from_fn(|axis| first[axis].max(second[axis]));
    Some((min, max))
}

fn kind(info: &EntityInfo) -> Option<String> { info.kind.as_ref().map(Identifier::to_string) }

// canSuck: alive, not just dropped by this pipe, and paid for at a unit of energy per block of distance
fn canSuck(state: &PipeState, power: &mut Provider, info: &EntityInfo, distance: i32) -> bool
{
    if !info.alive
    {
        return false;
    }

    match kind(info).as_deref()
    {
        Some(ITEM) => !pipe::droppedIds(state).contains(&info.id) && power.useEnergy(1, distance, false) >= distance,

        Some(ARROW) => power.useEnergy(1, distance, false) >= distance,
        _ => false,
    }
}

// pullItemIntoPipe: as much of the stack as the energy pays for goes into the pipe
fn popSound(world: World, at: [f64; 3])
{
    let spread: f32 = (random::float(world) - random::float(world)) as f32 * POP_SPREAD;
    world.playSound(at, POP, POP_VOLUME, (spread + 1.0) * POP_PITCH);
}

fn pull(world: World, pos: BlockPos, state: &mut PipeState, power: &mut Provider, info: &EntityInfo, distance: i32)
{
    let face: i32 = suckingFace(world, pos);
    if face == dir::UNKNOWN
    {
        return;
    }

    popSound(world, info.pos);

    let entity: Entity = Entity { world, id: info.id };
    let stack: Option<ItemStack> = match kind(info).as_deref()
    {
        Some(ITEM) =>
        {
            let Some(held) = entity.itemStack()
            else
            {
                return;
            };

            let used: i32 = power.useEnergy(distance, held.count * distance, true);
            if distance != 0 && used / distance != held.count
            {
                let taken: i32 = used / distance;
                let rest: ItemStack = ItemStack { count: held.count - taken, ..held.clone() };
                if let Err(error) = entity.setItemStack(&rest)
                {
                    warn(&format!("an obsidian pipe could not split a stack: {error}"));
                }

                Some(ItemStack { count: taken, ..held })
            }
            else
            {
                entity.kill();
                Some(held)
            }
        }

        Some(ARROW) =>
        {
            power.useEnergy(distance, distance, true);
            entity.kill();
            Some(ItemStack::of(ARROW, 1))
        }

        _ => None,
    };

    if let Some(stack) = stack.filter(|stack| stack.count > 0)
    {
        super::pickup::show(world, info.pos, pos, &stack);
        pipe::enterFromSuction(world, pos, state, &stack, dir::reverse(face));
    }
}

// a chest minecart one block out gives one item a try, which flies into the pipe as a dropped item would
fn fromCart(world: World, pos: BlockPos, state: &mut PipeState, power: &mut Provider, info: &EntityInfo) -> bool
{
    const CHEST_CART: f64 = 1.0;
    const LIFT: f64 = 0.3;

    let cart: Entity = Entity { world, id: info.id };
    let chest: bool =
        kind(info).as_deref() == Some(MINECART) && info.alive && cart.field(EntityField::MinecartType) == CHEST_CART;
    if !chest || power.useEnergy(1, 1, false) != 1
    {
        return false;
    }

    let Some((slot, held)) = (0..cart.inventorySize())
        .find_map(|slot| cart.slot(slot).filter(|held| held.count > 0).map(|held| (slot, held)))
    else
    {
        return false;
    };

    let rest: Option<ItemStack> = (held.count > 1).then(|| ItemStack { count: held.count - 1, ..held.clone() });
    if let Err(error) = cart.setSlot(slot, rest.as_ref())
    {
        warn(&format!("an obsidian pipe could not take from a minecart: {error}"));
        return false;
    }

    power.useEnergy(1, 1, true);

    let face: i32 = suckingFace(world, pos);
    let from: [f64; 3] = [info.pos[0], info.pos[1] + LIFT, info.pos[2]];
    let stack: ItemStack = ItemStack { count: 1, ..held };
    if face == dir::UNKNOWN
    {
        spill(world, from, &stack);
        return true;
    }

    popSound(world, from);
    super::pickup::show(world, from, pos, &stack);

    // pullItemIntoPipe takes the item only when one more unit pays for it, otherwise it stays on the ground
    if power.useEnergy(1, 1, true) < 1
    {
        spill(world, from, &stack);
        return true;
    }

    pipe::enterFromSuction(world, pos, state, &stack, dir::reverse(face));
    true
}

// the EntityItem trySucc makes beside the cart, with the toss a new EntityItem gets
fn spill(world: World, at: [f64; 3], stack: &ItemStack)
{
    const TOSS: f64 = 0.1;
    const HOP: f64 = 0.2;
    const DELAY: i32 = 10;

    let toss = || random::float(world) * TOSS * 2.0 - TOSS;
    if let Err(error) = world.dropItem(at, stack, [toss(), HOP, toss()], DELAY)
    {
        warn(&format!("an obsidian pipe could not drop a minecart item: {error}"));
    }
}

// trySucc, entities in the slab in the order the world lists them
fn trySuck(world: World, pos: BlockPos, state: &mut PipeState, power: &mut Provider, distance: i32) -> bool
{
    let Some((min, max)) = suckingBox(pos, suckingFace(world, pos), distance)
    else
    {
        return false;
    };

    for info in world.entitiesIn(min, max)
    {
        if canSuck(state, power, &info, distance)
        {
            pull(world, pos, state, power, &info, distance);
            return true;
        }

        if distance == 1 && fromCart(world, pos, state, power, &info)
        {
            return true;
        }
    }

    false
}

// TileObsidianPipe.doWork, the nearest slab with something to take wins
pub(crate) fn work(world: World, pos: BlockPos, state: &mut PipeState, power: &mut Provider)
{
    for distance in 1..=REACH
    {
        if trySuck(world, pos, state, power, distance)
        {
            return;
        }
    }

    power.useEnergy(1, 1, true);
}

// BlockObsidianPipe.onEntityCollidedWithBlock, an item touching the pipe goes in for free
pub(crate) fn touched(world: World, pos: BlockPos, state: &mut PipeState, entity: Entity)
{
    let Some(info) = entity.info()
    else
    {
        return;
    };
    let Some(mut power) = state.m_power
    else
    {
        return;
    };

    if canSuck(state, &mut power, &info, 0)
    {
        pull(world, pos, state, &mut power, &info, 0);
    }

    state.m_power = Some(power);
}
