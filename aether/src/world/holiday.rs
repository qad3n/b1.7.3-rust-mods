use super::gen_world::GenWorld;
use crate::names::block;
use b173_api::terrain::Random;

const HOLIDAY_LEAVES: &str = "holiday_leaves";
const DECORATED_LEAVES: &str = "decorated_holiday_leaves";
const PRESENT: &str = "present";
const SNOW: &str = "minecraft:snow_layer";
const ICE: &str = "minecraft:ice";
const WATER: &str = "minecraft:water";
const LOG: bool = true;
const LEAF: bool = false;

mod tree
{
    pub const TRUNK: i32 = 8;
    pub const DECORATED_CHANCE: i32 = 4;
    // the snow ring reaches seven blocks out and thins to a circle
    pub const RING: i32 = 7;
    pub const RING_SQUARED: i32 = 29;
    // one present in this many snowed spots before the tree stands, and again after
    pub const PRESENTS_BEFORE: i32 = 80;
    pub const PRESENTS_AFTER: i32 = 40;
}

// AetherGenHolidayTree.generate's canopy in its order: x, y and z from the trunk's foot, and whether it is wood
const CANOPY: [(i32, i32, i32, bool); 216] = [
    (0, 2, 1, LEAF),
    (0, 2, 2, LOG),
    (0, 2, 3, LEAF),
    (0, 2, 4, LEAF),
    (0, 2, -1, LEAF),
    (0, 2, -2, LEAF),
    (0, 2, -3, LEAF),
    (0, 2, -4, LEAF),
    (1, 2, 0, LEAF),
    (2, 2, 0, LEAF),
    (3, 2, 0, LEAF),
    (4, 2, 0, LEAF),
    (-1, 2, 0, LEAF),
    (-2, 2, 0, LEAF),
    (-3, 2, 0, LOG),
    (-4, 2, 0, LEAF),
    (1, 2, 1, LEAF),
    (1, 2, 2, LEAF),
    (1, 2, 3, LEAF),
    (1, 2, 4, LEAF),
    (-1, 2, 1, LEAF),
    (-1, 2, 2, LOG),
    (-1, 2, 3, LEAF),
    (-1, 2, 4, LEAF),
    (1, 2, -1, LEAF),
    (1, 2, -2, LEAF),
    (1, 2, -3, LEAF),
    (1, 2, -4, LEAF),
    (-1, 2, -1, LEAF),
    (-1, 2, -2, LOG),
    (-1, 2, -3, LEAF),
    (-1, 2, -4, LEAF),
    (1, 2, 1, LEAF),
    (2, 2, 1, LEAF),
    (3, 2, 1, LEAF),
    (4, 2, 1, LEAF),
    (1, 2, -1, LEAF),
    (2, 2, -1, LEAF),
    (3, 2, -1, LEAF),
    (4, 2, -1, LEAF),
    (-1, 2, 1, LEAF),
    (-2, 2, 1, LEAF),
    (-3, 2, 1, LEAF),
    (-4, 2, 1, LEAF),
    (-1, 2, -1, LEAF),
    (-2, 2, -1, LEAF),
    (-3, 2, -1, LEAF),
    (-4, 2, -1, LEAF),
    (2, 2, 1, LEAF),
    (3, 2, 1, LEAF),
    (2, 2, 1, LEAF),
    (-2, 2, 2, LEAF),
    (-3, 2, 2, LEAF),
    (-2, 2, 3, LEAF),
    (2, 2, -2, LEAF),
    (3, 2, -2, LEAF),
    (2, 2, -3, LEAF),
    (-2, 2, -2, LEAF),
    (-3, 2, -2, LEAF),
    (-2, 2, -3, LEAF),
    (0, 3, 1, LEAF),
    (0, 3, 2, LEAF),
    (0, 3, 3, LEAF),
    (0, 3, 4, LEAF),
    (0, 3, -1, LEAF),
    (0, 3, -2, LEAF),
    (0, 3, -3, LEAF),
    (0, 3, -4, LEAF),
    (1, 3, 0, LEAF),
    (2, 3, 0, LEAF),
    (3, 3, 0, LEAF),
    (4, 3, 0, LEAF),
    (-1, 3, 0, LEAF),
    (-2, 3, 0, LEAF),
    (-3, 3, 0, LEAF),
    (-4, 3, 0, LEAF),
    (1, 3, 1, LEAF),
    (1, 3, 2, LEAF),
    (1, 3, 3, LEAF),
    (-1, 3, 1, LEAF),
    (-1, 3, 2, LEAF),
    (-1, 3, 3, LEAF),
    (1, 3, -1, LEAF),
    (1, 3, -2, LEAF),
    (1, 3, -3, LEAF),
    (-1, 3, -1, LEAF),
    (-1, 3, -2, LEAF),
    (-1, 3, -3, LEAF),
    (1, 3, 1, LEAF),
    (2, 3, 1, LEAF),
    (3, 3, 1, LEAF),
    (1, 3, -1, LEAF),
    (2, 3, -1, LEAF),
    (3, 3, -1, LEAF),
    (-1, 3, 1, LEAF),
    (-2, 3, 1, LEAF),
    (-3, 3, 1, LEAF),
    (-1, 3, -1, LEAF),
    (-2, 3, -1, LEAF),
    (-3, 3, -1, LEAF),
    (2, 3, 2, LEAF),
    (3, 3, 2, LEAF),
    (2, 3, 3, LEAF),
    (-2, 3, 2, LEAF),
    (-3, 3, 2, LEAF),
    (-2, 3, 3, LEAF),
    (2, 3, -2, LEAF),
    (3, 3, -2, LEAF),
    (2, 3, -3, LEAF),
    (-2, 3, -1, LEAF),
    (-3, 3, -2, LEAF),
    (-2, 3, -3, LEAF),
    (0, 4, 1, LEAF),
    (0, 4, 2, LEAF),
    (0, 4, 3, LEAF),
    (0, 4, -1, LEAF),
    (0, 4, -2, LEAF),
    (0, 4, -3, LEAF),
    (1, 4, 0, LEAF),
    (2, 4, 0, LEAF),
    (3, 4, 0, LEAF),
    (-1, 4, 0, LEAF),
    (-2, 4, 0, LEAF),
    (-3, 4, 0, LEAF),
    (1, 4, 1, LEAF),
    (1, 4, 2, LEAF),
    (1, 4, 3, LEAF),
    (-1, 4, 1, LEAF),
    (-1, 4, 2, LEAF),
    (-1, 4, 3, LEAF),
    (1, 4, -1, LEAF),
    (1, 4, -2, LEAF),
    (1, 4, -3, LEAF),
    (-1, 4, -1, LEAF),
    (-1, 4, -2, LEAF),
    (-1, 4, -3, LEAF),
    (1, 4, 1, LEAF),
    (2, 4, 1, LEAF),
    (3, 4, 1, LEAF),
    (1, 4, -1, LEAF),
    (2, 4, -1, LEAF),
    (3, 4, -1, LEAF),
    (-1, 4, 1, LEAF),
    (-2, 4, 1, LEAF),
    (-3, 4, 1, LEAF),
    (-1, 4, -1, LEAF),
    (-2, 4, -1, LEAF),
    (-3, 4, -1, LEAF),
    (2, 4, 2, LEAF),
    (-2, 4, 2, LEAF),
    (2, 4, -2, LEAF),
    (-2, 4, -2, LEAF),
    (0, 5, 1, LEAF),
    (0, 5, 2, LEAF),
    (0, 5, 3, LEAF),
    (0, 5, -1, LEAF),
    (0, 5, -2, LEAF),
    (0, 5, -3, LEAF),
    (1, 5, 0, LEAF),
    (2, 5, 0, LEAF),
    (3, 5, 0, LEAF),
    (-1, 5, 0, LEAF),
    (-2, 5, 0, LEAF),
    (-3, 5, 0, LEAF),
    (1, 5, 1, LEAF),
    (1, 5, 2, LEAF),
    (-1, 5, 1, LOG),
    (-1, 5, 2, LEAF),
    (1, 5, -1, LEAF),
    (1, 5, -2, LEAF),
    (-1, 5, -1, LEAF),
    (-1, 5, -2, LEAF),
    (1, 5, 1, LEAF),
    (2, 5, 1, LEAF),
    (1, 5, -1, LEAF),
    (2, 5, -1, LEAF),
    (-1, 5, 1, LEAF),
    (-2, 5, 1, LOG),
    (-1, 5, -1, LEAF),
    (-2, 5, -1, LEAF),
    (2, 5, 2, LEAF),
    (-2, 5, 2, LEAF),
    (2, 5, -2, LEAF),
    (-2, 5, -2, LEAF),
    (0, 6, 1, LEAF),
    (0, 6, 2, LEAF),
    (0, 6, -1, LEAF),
    (0, 6, -2, LEAF),
    (1, 6, 0, LEAF),
    (2, 6, 0, LEAF),
    (-1, 6, 0, LEAF),
    (-2, 6, 0, LEAF),
    (1, 6, 1, LEAF),
    (-1, 6, 1, LEAF),
    (1, 6, -1, LEAF),
    (-1, 6, -1, LEAF),
    (1, 6, 1, LEAF),
    (1, 6, -1, LEAF),
    (-1, 6, 1, LEAF),
    (-1, 6, -1, LEAF),
    (0, 7, 1, LEAF),
    (0, 7, -1, LEAF),
    (1, 7, 0, LEAF),
    (-1, 7, 0, LEAF),
    (1, 7, 1, LEAF),
    (-1, 7, 1, LEAF),
    (1, 7, -1, LEAF),
    (-1, 7, -1, LEAF),
    (1, 7, 1, LEAF),
    (1, 7, -1, LEAF),
    (-1, 7, 1, LEAF),
    (-1, 7, -1, LEAF),
    (0, 8, 1, LEAF),
    (0, 8, -1, LEAF),
    (1, 8, 0, LEAF),
    (-1, 8, 0, LEAF),
];

// AetherGenHolidayTree.getBlock
fn leaf(random: &Random) -> &'static str
{
    if random.nextIntBounded(tree::DECORATED_CHANCE) == 0
    {
        DECORATED_LEAVES
    }
    else
    {
        HOLIDAY_LEAVES
    }
}

// snow settles on the ring around the tree, water under it freezes and a few presents lie on the grass
fn snow(world: &GenWorld, random: &Random, x: i32, z: i32, presents: i32)
{
    for spotX in x - tree::RING..x + tree::RING
    {
        for spotZ in z - tree::RING..z + tree::RING
        {
            let (dx, dz) = (spotX - x, spotZ - z);
            let top: i32 = world.height(spotX, spotZ);
            if dx * dx + dz * dz >= tree::RING_SQUARED || world.isAir(spotX, top - 1, spotZ)
            {
                continue;
            }

            // the Java draws for a present before it looks at the ground
            let gift: bool = random.nextIntBounded(presents) == 0;
            if gift && world.is(spotX, top - 1, spotZ, block::AETHER_GRASS)
            {
                world.set(spotX, top, spotZ, PRESENT);
                continue;
            }

            if world.is(spotX, top - 1, spotZ, WATER)
            {
                world.set(spotX, top - 1, spotZ, ICE);
            }

            world.set(spotX, top, spotZ, SNOW);
        }
    }
}

// AetherGenHolidayTree.generate, only on aether grass
pub(crate) fn holidayTree(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32)
{
    if !world.is(x, y - 1, z, block::AETHER_GRASS)
    {
        return;
    }

    snow(world, random, x, z, tree::PRESENTS_BEFORE);

    for rise in 0..=tree::TRUNK
    {
        world.set(x, y + rise, z, block::SKYROOT_LOG);
    }

    world.set(x, y + tree::TRUNK + 1, z, leaf(random));

    for (dx, dy, dz, wood) in CANOPY
    {
        let piece: &str = if wood
        {
            block::SKYROOT_LOG
        }
        else
        {
            leaf(random)
        };
        world.set(x + dx, y + dy, z + dz, piece);
    }

    world.set(x, y - 1, z, block::AETHER_DIRT);
    snow(world, random, x, z, tree::PRESENTS_AFTER);
}
