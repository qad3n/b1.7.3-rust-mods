use super::random;
use b173_api::prelude::*;

mod toss
{
    // BlockMultiID.dropStack puts the item somewhere in the middle 70 percent of the block
    pub const SPREAD: f64 = 0.7;
    // EntityItem's constructor throws it up and a little to a random side
    pub const SIDEWAYS: f64 = 0.2;
    pub const UP: f64 = 0.2;
    pub const PICKUP_DELAY: i32 = 10;
}

// BlockMultiID.dropStack and every IC2 drop that copies it
pub(crate) fn drop(world: World, pos: BlockPos, stack: &ItemStack)
{
    let edge: f64 = (1.0 - toss::SPREAD) * 0.5;
    let spot = || random::float(world) * toss::SPREAD + edge;
    let at: [f64; 3] = [pos.x as f64 + spot(), pos.y as f64 + spot(), pos.z as f64 + spot()];
    let half: f64 = toss::SIDEWAYS * 0.5;
    let motion: [f64; 3] =
        [random::float(world) * toss::SIDEWAYS - half, toss::UP, random::float(world) * toss::SIDEWAYS - half];

    if let Err(error) = world.dropItem(at, stack, motion, toss::PICKUP_DELAY)
    {
        warn(&format!("could not drop {:?} at {pos:?}: {error}", stack.item.name()));
    }
}

// the stack less one item, None once it runs out
pub(crate) fn less(stack: ItemStack, count: i32) -> Option<ItemStack>
{
    let left: i32 = stack.count - count;
    (left > 0).then_some(ItemStack { count: left, ..stack })
}

pub(crate) fn same(a: &ItemStack, b: &ItemStack) -> bool
{
    a.item.id().ok() == b.item.id().ok() && a.damage == b.damage
}

pub(crate) fn maxStack(stack: &ItemStack) -> i32
{
    const DEFAULT: i32 = 64;

    stack.info().map_or(DEFAULT, |info| info.max_stack)
}

// whether an output slot takes the result as SlotFurnace output, isItemEqual and the max stack size
pub(crate) fn fits(slot: &Option<ItemStack>, result: &ItemStack) -> bool
{
    match slot
    {
        None => true,
        Some(held) => same(held, result) && held.count + result.count <= maxStack(held),
    }
}

// adds a result into an output slot that fits it
pub(crate) fn merged(slot: Option<ItemStack>, result: &ItemStack) -> ItemStack
{
    match slot
    {
        Some(held) => ItemStack { count: held.count + result.count, ..held },
        None => result.clone(),
    }
}

// a machine input used up, an item with a container leaves it behind as Item.getContainerItem
pub(crate) fn consume(stack: ItemStack) -> Option<ItemStack>
{
    match containerItem(&stack)
    {
        Some(container) => Some(container),
        None => less(stack, 1),
    }
}

pub(crate) fn set(world: World, pos: BlockPos, slot: i32, stack: Option<ItemStack>)
{
    if let Err(error) = world.setTileSlot(pos, slot, stack)
    {
        warn(&format!("could not change slot {slot} of the machine at {pos:?}: {error}"));
    }
}
