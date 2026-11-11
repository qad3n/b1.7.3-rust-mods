use crate::core::{AIR, dir, random};
use b173_api::prelude::*;

// a stack kept in tile data by name, numeric ids move when a world remaps them
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub(crate) struct Stored
{
    pub(crate) m_item: Identifier,
    pub(crate) m_count: i32,
    pub(crate) m_damage: i32,
}

// the SDK caches item info per content epoch, pipes ask every frame
pub(crate) fn isBlock(stack: &ItemStack) -> bool { stack.info().is_some_and(|info| info.block) }

impl Stored
{
    pub(crate) fn of(stack: &ItemStack) -> Option<Stored>
    {
        Some(Stored { m_item: stack.item.name()?, m_count: stack.count, m_damage: stack.damage })
    }

    pub(crate) fn stack(&self) -> ItemStack
    {
        let item: ItemRef = ItemRef::Named(self.m_item.clone());
        ItemStack { item, count: self.m_count, damage: self.m_damage }
    }
}

// a dropped stack the way Utils.dropItems throws it, a random spot inside the block
pub(crate) fn drop(world: World, pos: BlockPos, stack: ItemStack)
{
    const SPREAD: f64 = 0.7;
    const DELAY: i32 = 10;

    let offset = || random::float(world) * SPREAD + (1.0 - SPREAD) * 0.5;
    let spot: [f64; 3] = [pos.x as f64 + offset(), pos.y as f64 + offset(), pos.z as f64 + offset()];

    if let Err(error) = world.dropItem(spot, &stack, [0.0; 3], DELAY)
    {
        warn(&format!("could not drop items at {pos:?}: {error}"));
    }
}

// a machine's output thrown up out of its top when nothing next to it takes it, as the quarry and mining well do
pub(crate) fn eject(world: World, pos: BlockPos, stack: ItemStack)
{
    const SPREAD: f64 = 0.8;
    const EDGE: f64 = 0.1;
    const LIFT: f64 = 0.5;
    const WOBBLE: f64 = 0.05;
    const THROW: f64 = 1.0;
    const DELAY: i32 = 10;

    let offset = || random::float(world) * SPREAD + EDGE;
    let spot: [f64; 3] = [pos.x as f64 + offset(), pos.y as f64 + offset() + LIFT, pos.z as f64 + offset()];
    let motion: [f64; 3] =
        [random::gaussian(world) * WOBBLE, random::gaussian(world) * WOBBLE + THROW, random::gaussian(world) * WOBBLE];

    if let Err(error) = world.dropItem(spot, &stack, motion, DELAY)
    {
        warn(&format!("could not eject items at {pos:?}: {error}"));
    }
}

// StackUtil.addToRandomInventory: a random neighbour inventory with room takes what it can, and what is left goes on
// from that inventory to its own neighbours, as BuildCraft recurses from the inventory it filled
pub(crate) fn toInventories(world: World, pos: BlockPos, mut stack: ItemStack, from: i32) -> ItemStack
{
    let mut at: BlockPos = pos;

    loop
    {
        let candidates: Vec<i32> = dir::all()
            .filter(|face| *face != dir::reverse(from))
            .flat_map(|face| takers(world, at, face, &stack).into_iter().filter(|takes| *takes).map(move |_| face))
            .collect();
        if candidates.is_empty()
        {
            return stack;
        }

        let face: i32 = candidates[world.random(candidates.len() as i32) as usize];
        let target: BlockPos = dir::step(at, face);
        let left: i32 = insert(world, target, dir::reverse(face), &stack, false);
        if left == stack.count || left <= 0
        {
            stack.count = left;
            return stack;
        }

        stack.count = left;
        at = target;
    }
}

// the two ways addToRandomInventory lists a neighbour, so a special inventory that also joins counts twice: as a
// special inventory asked from the unknown side, and as any joined inventory with room
fn takers(world: World, at: BlockPos, face: i32, stack: &ItemStack) -> [bool; 2]
{
    let target: BlockPos = dir::step(at, face);
    let side: i32 = dir::reverse(face);
    if crate::transport::isPipe(world, target)
    {
        return [false; 2];
    }

    let special: bool = !plain(world, target, side)
        && world.insertItem(target, dir::UNKNOWN, stack.clone(), true).is_ok_and(|left| left < stack.count);
    let joined: bool = world.capabilities(target, side) & capability::ITEMS != 0
        && crate::machines::connected(world, at, target)
        && insert(world, target, side, stack, true) < stack.count;
    [special, joined]
}

// a machine's product: inventories next to it first, then a pipe, then thrown out of the top
pub(crate) fn output(world: World, pos: BlockPos, stack: ItemStack)
{
    let left: ItemStack = toInventories(world, pos, stack, dir::UNKNOWN);
    if left.count <= 0
    {
        return;
    }

    if crate::transport::toRandomPipe(world, pos, dir::UNKNOWN, &left)
    {
        return;
    }

    eject(world, pos, left);
}

// Utils.breakBlock: the block drops what it would when mined, then turns to air
pub(crate) fn breakBlock(world: World, pos: BlockPos)
{
    if !world.is(pos, AIR)
        && let Err(error) = world.dropBlockItems(pos, world.block(pos), world.meta(pos))
    {
        warn(&format!("could not drop the block at {pos:?}: {error}"));
    }

    if let Err(error) = world.clear(pos)
    {
        warn(&format!("could not clear {pos:?}: {error}"));
    }
}

// the slots BuildCraft reaches in a two or three slot inventory from a face, a furnace's layout; None for any other
fn smallSlots(size: i32, face: i32, inserting: bool) -> Option<Vec<i32>>
{
    let vertical: bool = face == dir::UP || face == dir::DOWN;

    match size
    {
        2 => Some(vec![if vertical { 0 } else { 1 }]),
        3 if inserting => Some(match face
        {
            dir::UP => vec![0],
            dir::DOWN => vec![1],
            _ => Vec::new(),
        }),
        3 => Some(vec![match face
        {
            dir::UP => 0,
            dir::DOWN => 1,
            _ => 2,
        }]),
        _ => None,
    }
}

pub(crate) fn plain(world: World, pos: BlockPos, face: i32) -> bool
{
    world.capabilities(pos, face) & capability::HANDLER == 0
}

// Utils.getInventory: a chest joined with the last neighbouring chest found west, east, north then south, its own
// slots first whichever side the other half is on
pub(crate) fn slotsOf(world: World, pos: BlockPos) -> Vec<(BlockPos, i32)>
{
    const CHEST: &str = "minecraft:chest";
    const SIDES: [i32; 4] = [dir::WEST, dir::EAST, dir::NORTH, dir::SOUTH];

    let own = |at: BlockPos| (0..world.inventorySize(at)).map(move |slot| (at, slot));
    let mut slots: Vec<(BlockPos, i32)> = own(pos).collect();

    if world.is(pos, CHEST)
        && let Some(other) = SIDES.iter().map(|face| dir::step(pos, *face)).rfind(|at| world.is(*at, CHEST))
    {
        slots.extend(own(other));
    }

    slots
}

// the slots BuildCraft reaches through a face of a plain inventory
fn reach(world: World, pos: BlockPos, face: i32, inserting: bool) -> Vec<(BlockPos, i32)>
{
    match smallSlots(world.inventorySize(pos), face, inserting)
    {
        Some(slots) => slots.into_iter().map(|slot| (pos, slot)).collect(),
        None => slotsOf(world, pos),
    }
}

// StackUtil.checkAvailableSlot: a special inventory answers itself, a plain one takes the stack an item at a time,
// joining the first equal stack with room before the first empty slot
pub(crate) fn insert(world: World, pos: BlockPos, face: i32, stack: &ItemStack, simulate: bool) -> i32
{
    if !plain(world, pos, face)
    {
        return world.insertItem(pos, face, stack.clone(), simulate).unwrap_or(stack.count);
    }

    let limit: i32 = stack.info().map_or(stack.count, |info| info.max_stack);
    let mut left: i32 = stack.count;

    for joining in [true, false]
    {
        for (at, slot) in reach(world, pos, face, true)
        {
            if left <= 0
            {
                break;
            }

            let held: Option<ItemStack> = world.tileSlot(at, slot);
            let room: i32 = match &held
            {
                Some(held) if joining && same(held, stack) => limit - held.count,
                None if !joining => limit,
                _ => 0,
            };
            let moved: i32 = room.clamp(0, left);
            if moved == 0
            {
                continue;
            }

            left -= moved;

            if !simulate
            {
                let count: i32 = held.map_or(0, |held| held.count) + moved;
                let _ = world.setTileSlot(at, slot, Some(ItemStack { count, ..stack.clone() }));
            }
        }
    }

    left
}

fn same(first: &ItemStack, second: &ItemStack) -> bool
{
    first.item.id().ok() == second.item.id().ok() && first.damage == second.damage
}

// TileWoodenPipe.checkExtract: a special inventory gives what it gives, a plain one its first filled slot BuildCraft
// reaches; pay sizes the take from what the slot holds
pub(crate) fn extract(world: World, pos: BlockPos, face: i32, mut pay: impl FnMut(i32) -> i32) -> Option<ItemStack>
{
    if !plain(world, pos, face)
    {
        return world.extractItem(pos, face, i32::MAX, false);
    }

    let small: bool = smallSlots(world.inventorySize(pos), face, false).is_some();
    let slots: Vec<(BlockPos, i32)> = reach(world, pos, face, false);
    // a small inventory looks only at its one slot, a large one at its first filled slot
    let (at, slot, held): (BlockPos, i32, ItemStack) = if small
    {
        let (at, slot): (BlockPos, i32) = *slots.first()?;
        (at, slot, world.tileSlot(at, slot).filter(|held| held.count > 0)?)
    }
    else
    {
        slots.into_iter().find_map(|(at, slot)|
        {
            world.tileSlot(at, slot).filter(|held| held.count > 0).map(|held| (at, slot, held))
        })?
    };

    let taken: i32 = pay(held.count).clamp(0, held.count);
    let rest: Option<ItemStack> = (held.count > taken).then(|| ItemStack { count: held.count - taken, ..held.clone() });
    let _ = world.setTileSlot(at, slot, rest);
    Some(ItemStack { count: taken, ..held })
}
