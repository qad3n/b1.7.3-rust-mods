use super::palette::{self, Placed};
use b173_api::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

// the world a generator writes into, with block properties cached for one populate call
pub(crate) struct GenWorld
{
    pub m_world: World,
    m_info: RefCell<HashMap<i32, BlockInfo>>,
}

impl GenWorld
{
    pub(crate) fn new(world: World) -> Self { Self { m_world: world, m_info: RefCell::new(HashMap::new()) } }

    pub(crate) fn block(&self, x: i32, y: i32, z: i32) -> i32 { self.m_world.block(BlockPos::new(x, y, z)) }

    pub(crate) fn meta(&self, x: i32, y: i32, z: i32) -> i32 { self.m_world.meta(BlockPos::new(x, y, z)) }

    pub(crate) fn isAir(&self, x: i32, y: i32, z: i32) -> bool { self.block(x, y, z) == 0 }

    // world generation places blocks without neighbour updates, as WorldGenerator.setBlockAndNotifyAdequately
    pub(crate) fn set(&self, x: i32, y: i32, z: i32, placed: Placed)
    {
        if !(0..crate::world::WORLD_HEIGHT).contains(&y)
        {
            return;
        }

        let _ = self.m_world.setBlockQuiet(BlockPos::new(x, y, z), placed.0, placed.1);
    }

    pub(crate) fn clear(&self, x: i32, y: i32, z: i32) { self.set(x, y, z, palette::AIR); }

    // a sapling growing into a tree notifies its neighbours as World.setBlock does
    pub(crate) fn setNotify(&self, x: i32, y: i32, z: i32, placed: Placed)
    {
        let _ = self.m_world.setBlock(BlockPos::new(x, y, z), placed.0, placed.1);
    }

    // World.getHeightValue, the first air above the topmost light blocking block
    pub(crate) fn height(&self, x: i32, z: i32) -> i32 { self.m_world.height(x, z) }

    // World.getTopSolidOrLiquidBlock
    pub(crate) fn topSolid(&self, x: i32, z: i32) -> i32
    {
        let mut y: i32 = self.height(x, z).min(crate::world::WORLD_HEIGHT - 1);
        while y > 0
        {
            let block: i32 = self.block(x, y - 1, z);
            if block != 0 && (self.info(block).solid || self.info(block).liquid) && !self.leafy(block)
            {
                return y;
            }

            y -= 1;
        }
        -1
    }

    pub(crate) fn is(&self, x: i32, y: i32, z: i32, name: &str) -> bool { blockIs(self.block(x, y, z), name) }

    pub(crate) fn tagged(&self, x: i32, y: i32, z: i32, tag: &str) -> bool { blockTagged(self.block(x, y, z), tag) }

    pub(crate) fn info(&self, block: i32) -> BlockInfo
    {
        *self.m_info.borrow_mut().entry(block).or_insert_with(|| World::blockInfo(block))
    }

    pub(crate) fn isOpaque(&self, x: i32, y: i32, z: i32) -> bool { self.info(self.block(x, y, z)).opaque }

    pub(crate) fn isSolid(&self, x: i32, y: i32, z: i32) -> bool { self.info(self.block(x, y, z)).solid }

    pub(crate) fn isLiquid(&self, x: i32, y: i32, z: i32) -> bool { self.info(self.block(x, y, z)).liquid }

    pub(crate) fn isReplaceable(&self, x: i32, y: i32, z: i32) -> bool
    {
        let block: i32 = self.block(x, y, z);
        block == 0 || self.info(block).replaceable
    }

    fn leafy(&self, block: i32) -> bool { blockTagged(block, crate::tags::LEAVES) }
}
