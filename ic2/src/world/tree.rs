use crate::blocks::{RUBBER_LEAVES, RUBBER_SAPLING, RUBBER_WOOD, plain};
use crate::core::ids;
use b173_api::prelude::*;
use b173_api::terrain::Random;

mod shape
{
    pub const MAX_HEIGHT: i32 = 8;
    pub const TOP: i32 = 128;
    // WorldGenRubTree.grow: a quarter of logs, falling by ten points each time, get a resin hole
    pub const HOLE_CHANCE: i32 = 25;
    pub const HOLE_FALLOFF: i32 = 10;
    pub const PERCENT: i32 = 100;
    pub const HOLE_FACES: i32 = 4;
    pub const FIRST_HOLE: i32 = 2;
    pub const RADIUS: i32 = 2;
    pub const LEAF_START: i32 = 4;
    pub const SHORT: i32 = 4;
    pub const MEDIUM: i32 = 7;
    pub const CROWN_DIVISOR: i32 = 4;
    pub const CROWN_EXTRA: i32 = 2;
    // generate wanders up to seven blocks between attempts and loses three tries on a failed one
    pub const WANDER: i32 = 15;
    pub const WANDER_BACK: i32 = 7;
    pub const FAILED: i32 = 3;
}

struct Grower
{
    m_world: World,
    m_wood: i32,
    m_leaves: i32,
}

impl Grower
{
    fn set(&self, pos: BlockPos, block: i32, meta: i32)
    {
        if let Err(error) = self.m_world.setBlock(pos, block, meta)
        {
            warn(&format!("a rubber tree could not place a block at {pos:?}: {error}"));
        }
    }

    // WorldGenRubTree.getGrowHeight: open air above grass, up to eight blocks
    fn room(&self, pos: BlockPos) -> i32
    {
        let world: World = self.m_world;
        let grass: bool = ids::block("minecraft:grass") == Some(world.block(pos.offset(0, -1, 0)));
        let here: i32 = world.block(pos);
        let open: bool = here == 0 || ids::block(RUBBER_SAPLING) == Some(here);
        if !grass || !open
        {
            return 0;
        }

        let mut height: i32 = 1;
        let mut y: i32 = pos.y;

        while world.block(BlockPos::new(pos.x, y + 1, pos.z)) == 0 && height < shape::MAX_HEIGHT
        {
            height += 1;
            y += 1;
        }

        height
    }

    fn leafAt(&self, pos: BlockPos)
    {
        if self.m_world.block(pos) == 0
        {
            self.set(pos, self.m_leaves, 0);
        }
    }

    // one layer of leaves around the trunk, the corners thinning out near the top
    fn layer(&self, random: &Random, base: BlockPos, level: i32, height: i32)
    {
        let chance: i32 = (level + shape::LEAF_START - height).max(1);

        for a in base.x - shape::RADIUS..=base.x + shape::RADIUS
        {
            for b in base.z - shape::RADIUS..=base.z + shape::RADIUS
            {
                let innerX: bool = a > base.x - shape::RADIUS && a < base.x + shape::RADIUS;
                let innerZ: bool = b > base.z - shape::RADIUS && b < base.z + shape::RADIUS;
                let grows: bool = innerX && innerZ
                    || innerX && random.nextIntBounded(chance) == 0
                    || innerZ && random.nextIntBounded(chance) == 0;

                if grows
                {
                    self.leafAt(BlockPos::new(a, base.y + level, b));
                }
            }
        }
    }

    fn grow(&self, random: &Random, base: BlockPos) -> bool
    {
        let mut room: i32 = self.room(base);
        if room < 2
        {
            return false;
        }

        let mut height: i32 = room / 2;
        room -= room / 2;
        height += random.nextIntBounded(room + 1);
        let mut holeChance: i32 = shape::HOLE_CHANCE;

        for level in 0..height
        {
            let log: BlockPos = base.offset(0, level, 0);
            let meta: i32 = if random.nextIntBounded(shape::PERCENT) <= holeChance
            {
                holeChance -= shape::HOLE_FALLOFF;
                random.nextIntBounded(shape::HOLE_FACES) + shape::FIRST_HOLE
            }
            else
            {
                plain()
            };
            self.set(log, self.m_wood, meta);

            let leafy: bool =
                height < shape::SHORT || height < shape::MEDIUM && level > 1 || level > shape::CROWN_EXTRA;
            if leafy
            {
                self.layer(random, base, level, height);
            }
        }

        let crown: i32 = height / shape::CROWN_DIVISOR + random.nextIntBounded(shape::CROWN_EXTRA);
        for level in 0..=crown
        {
            self.leafAt(base.offset(0, height + level, 0));
        }

        true
    }
}

fn grower(world: World) -> Option<Grower>
{
    Some(Grower { m_world: world, m_wood: ids::block(RUBBER_WOOD)?, m_leaves: ids::block(RUBBER_LEAVES)? })
}

// BlockRubSapling.growTree, with the world's random
pub(crate) fn grow(world: World, pos: BlockPos)
{
    let Some(grower) = grower(world)
    else
    {
        return;
    };
    let random: Random = Random::withSeed(i64::from(world.random(i32::MAX)));
    grower.grow(&random, pos);
}

// WorldGenRubTree.generate: count tries across the area, each from the surface of its column
pub(crate) fn generate(world: World, random: &Random, start: [i32; 2], count: i32)
{
    let Some(grower) = grower(world)
    else
    {
        return;
    };
    let [mut x, mut z] = start;
    let mut tries: i32 = count;

    while tries > 0
    {
        let mut y: i32 = shape::TOP;
        while y > 0 && world.block(BlockPos::new(x, y - 1, z)) == 0
        {
            y -= 1;
        }

        if !grower.grow(random, BlockPos::new(x, y, z))
        {
            tries -= shape::FAILED;
        }

        x += random.nextIntBounded(shape::WANDER) - shape::WANDER_BACK;
        z += random.nextIntBounded(shape::WANDER) - shape::WANDER_BACK;
        tries -= 1;
    }
}
