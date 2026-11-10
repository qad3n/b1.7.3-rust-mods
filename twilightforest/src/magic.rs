use crate::blocks::wood::Effect;
use crate::ids;
use b173_api::prelude::*;

// BlockTFMagicLogSpecial.updateTick, what each running magic tree core does every second
mod core
{
    // the core ticks every twenty ticks and the tree of time speeds that many ticks' worth of blocks
    pub const INTERVAL: i32 = 20;
    pub const TIME_TICKS: i32 = 24 * INTERVAL;
    pub const TIME_REACH: i32 = 16;
    pub const MINE_REACH: i32 = 32;
    pub const SORT_REACH: i32 = 16;
    pub const CHEST_SLOTS: i32 = 27;
    pub const QUIET: f32 = 0.1;
}

fn center(pos: BlockPos) -> [f64; 3] { ids::center(pos) }

fn spread(world: World, reach: i32) -> i32 { world.random(reach * 2) - reach }

// doTreeOfTimeEffect, random ticks for the blocks around the clock
fn time(world: World, pos: BlockPos)
{
    world.playSound(center(pos), "random.click", core::QUIET, 0.5);
    for _ in 0..core::TIME_TICKS
    {
        let at: BlockPos = pos.offset(
            spread(world, core::TIME_REACH),
            spread(world, core::TIME_REACH),
            spread(world, core::TIME_REACH),
        );
        let here: i32 = world.block(at);
        if here == 0
        {
            continue;
        }
        if let Err(error) = world.randomTick(at)
        {
            warn(&format!("tree of time at {at:?}: {}", error.0));
            return;
        }
    }
}

// doTreeOfTransformationEffect, the heart chimes and turns one column within reach to the enchanted forest
fn transformation(world: World, pos: BlockPos)
{
    use crate::world::biomes::{self, Biome};
    let reach: i32 = 16;
    let (dx, dz): (i32, i32) = (spread(world, reach), spread(world, reach));
    world.playSound(center(pos), "note.harp", core::QUIET, ids::fraction(world) * 2.0);
    let (x, z): (i32, i32) = (pos.x + dx, pos.z + dz);
    let inside: bool = f64::from(dx).hypot(f64::from(dz)) < f64::from(reach);
    if !inside || biomes::biomeAt(world, BlockPos::new(x, pos.y, z)) == Biome::EnchantedForest
    {
        return;
    }

    let name: String = format!("twilightforest:{}", biomes::rowOf(Biome::EnchantedForest).m_name);
    if let Err(error) = world.setBiome(x, z, &name)
    {
        warn(&format!("tree of transformation at {x} {z}: {}", error.0));
    }
}

// doMinersTreeEffect, the miner's core pulls ores toward itself like an ore magnet
fn mining(world: World, pos: BlockPos)
{
    let to: [i32; 3] = [pos.x, pos.y, pos.z].map(|axis| axis + spread(world, core::MINE_REACH));
    let moved: i32 = crate::items::magic::pull(world, [pos.x, pos.y, pos.z], to);
    if moved > 0
    {
        world.playSound(center(pos), "mob.endermen.portal", core::QUIET, 1.0);
    }
}

// one filled slot of a chest the sorting engine found
#[derive(Clone)]
struct Held
{
    m_chest: BlockPos,
    m_slot: i32,
    m_stack: ItemStack,
}

fn same(a: &ItemStack, b: &ItemStack) -> bool { a.item.name() == b.item.name() }

// doSortingTreeEffect, one random item moves to the chest holding the most of its kind and gathers its stacks,
// beta has no creative tabs so its kind is the item itself
fn sorting(world: World, pos: BlockPos)
{
    let chest: i32 = ids::blockId("minecraft:chest");
    let reach: i32 = core::SORT_REACH;
    let mut chests: Vec<BlockPos> = Vec::new();
    let mut items: Vec<Held> = Vec::new();
    for x in pos.x - reach..pos.x + reach
    {
        for y in pos.y - reach..pos.y + reach
        {
            for z in pos.z - reach..pos.z + reach
            {
                let at: BlockPos = BlockPos::new(x, y, z);
                if world.block(at) != chest
                {
                    continue;
                }
                let before: usize = items.len();
                for slot in 0..core::CHEST_SLOTS
                {
                    if let Some(stack) = world.tileSlot(at, slot)
                    {
                        items.push(Held { m_chest: at, m_slot: slot, m_stack: stack });
                    }
                }
                if items.len() > before
                {
                    chests.push(at);
                }
            }
        }
    }
    if items.is_empty()
    {
        return;
    }
    let sorted: Held = items[world.random(items.len() as i32) as usize].clone();
    let best: Option<BlockPos> = chests
        .iter()
        .map(|chest|
        {
            (
                chest,
                items
                    .iter()
                    .filter(|held| held.m_chest == *chest && same(&held.m_stack, &sorted.m_stack))
                    .map(|held| held.m_stack.count)
                    .sum::<i32>(),
            )
        })
        .filter(|(_, count)| *count > 0)
        .max_by_key(|(_, count)| *count)
        .map(|(chest, _)| *chest);
    let mut home: (BlockPos, i32) = (sorted.m_chest, sorted.m_slot);
    if let Some(target) = best.filter(|target| *target != sorted.m_chest)
        && let Some(free) = (0..core::CHEST_SLOTS).find(|slot| world.tileSlot(target, *slot).is_none())
    {
        let _ = world.setTileSlot(sorted.m_chest, sorted.m_slot, None);
        let _ = world.setTileSlot(target, free, Some(sorted.m_stack.clone()));
        home = (target, free);
    }
    gather(world, &items, &sorted, home);
}

// the second half of doSortingTreeEffect, smaller stacks of the same item fold into the sorted one
fn gather(world: World, items: &[Held], sorted: &Held, home: (BlockPos, i32))
{
    let most: i32 = 64;
    let mut stack: ItemStack = sorted.m_stack.clone();
    for held in items
    {
        let elsewhere: bool = (held.m_chest, held.m_slot) != (sorted.m_chest, sorted.m_slot);
        // ItemStack.isItemEqual, the same item and damage whatever the count
        let alike: bool = same(&held.m_stack, &stack) && held.m_stack.damage == stack.damage;
        if elsewhere && alike && held.m_stack.count <= most - stack.count
        {
            let _ = world.setTileSlot(held.m_chest, held.m_slot, None);
            stack.count += held.m_stack.count;
        }
    }
    if stack.count != sorted.m_stack.count
    {
        let _ = world.setTileSlot(home.0, home.1, Some(stack));
    }
}

pub fn coreEffect(world: World, pos: BlockPos, effect: Effect)
{
    match effect
    {
        Effect::Time => time(world, pos),
        Effect::Transformation => transformation(world, pos),
        Effect::Mining => mining(world, pos),
        Effect::Sorting => sorting(world, pos),
    }
}

// what grows on uberous soil, IPlantable.getPlantType Crop then Plains, anything else is a cave plant
const CROPS: [&str; 1] = ["minecraft:wheat"];
const PLAINS: [&str; 4] =
    ["minecraft:red_flower", "minecraft:yellow_flower", "minecraft:sapling", "minecraft:tallgrass"];
const CAVE: [&str; 2] = ["minecraft:brown_mushroom", "minecraft:red_mushroom"];

fn anyOf(block: i32, names: &[&str]) -> bool { names.iter().any(|name| block == ids::blockId(name)) }

// BlockTFUberousSoil.onNeighborBlockChange, a plant set on it takes root in the right soil and grows four bone meals
// at once
pub fn uberous(world: World, pos: BlockPos)
{
    let above: BlockPos = pos.offset(0, 1, 0);
    let plant: i32 = world.block(above);
    if World::blockInfo(plant).solid
    {
        let _ = world.setBlock(pos, ids::blockId("minecraft:dirt"), 0);
        return;
    }
    let twilight: bool = crate::blocks::wood::isSapling(plant);
    let soil: Option<(&str, i32)> = if anyOf(plant, &CROPS)
    {
        Some(("minecraft:farmland", 2))
    }
    else if anyOf(plant, &PLAINS) || twilight
    {
        Some(("minecraft:grass", 0))
    }
    else if anyOf(plant, &CAVE)
    {
        Some(("minecraft:dirt", 0))
    }
    else
    {
        None
    };
    let Some((ground, meta)) = soil
    else
    {
        return;
    };
    let _ = world.setBlock(pos, ids::blockId(ground), meta);
    let meals: i32 = 4;
    for _ in 0..meals
    {
        if twilight
        {
            crate::blocks::wood::fertilize(world, above);
        }
        else if let Err(error) = world.fertilize(above)
        {
            warn(&format!("uberous soil bone meal: {}", error.0));
        }
    }
}
