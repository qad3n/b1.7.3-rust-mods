use b173_api::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

// the world a generator writes into, with block ids and properties cached for one populate call
pub struct GenWorld
{
    pub m_world: World,
    m_info: RefCell<HashMap<i32, BlockInfo>>,
    m_ids: RefCell<HashMap<&'static str, i32>>,
}

impl GenWorld
{
    pub fn new(world: World) -> GenWorld
    {
        GenWorld { m_world: world, m_info: RefCell::new(HashMap::new()), m_ids: RefCell::new(HashMap::new()) }
    }

    pub fn block(&self, x: i32, y: i32, z: i32) -> i32 { self.m_world.block(BlockPos::new(x, y, z)) }

    pub fn meta(&self, x: i32, y: i32, z: i32) -> i32 { self.m_world.meta(BlockPos::new(x, y, z)) }

    pub fn isAir(&self, x: i32, y: i32, z: i32) -> bool { self.block(x, y, z) == 0 }

    // world generation places blocks without neighbour updates, as WorldGenerator.setBlockAndNotifyAdequately
    pub fn set(&self, x: i32, y: i32, z: i32, block: i32) { self.setMeta(x, y, z, block, 0); }

    pub fn setMeta(&self, x: i32, y: i32, z: i32, block: i32, meta: i32)
    {
        if !(0..crate::world::WORLD_HEIGHT).contains(&y)
        {
            return;
        }
        let _ = self.m_world.setBlockQuiet(BlockPos::new(x, y, z), block, meta);
    }

    // a sapling growing into a tree notifies its neighbours as World.setBlock does
    pub fn setNotify(&self, x: i32, y: i32, z: i32, block: i32, meta: i32)
    {
        let _ = self.m_world.setBlock(BlockPos::new(x, y, z), block, meta);
    }

    // World.getHeightValue, the first air above the topmost light blocking block
    pub fn height(&self, x: i32, z: i32) -> i32 { self.m_world.height(x, z) }

    // World.getTopSolidOrLiquidBlock
    pub fn topSolid(&self, x: i32, z: i32) -> i32
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

    pub fn id(&self, name: &'static str) -> i32
    {
        *self.m_ids.borrow_mut().entry(name).or_insert_with(|| crate::ids::blockId(name))
    }

    pub fn info(&self, block: i32) -> BlockInfo
    {
        *self.m_info.borrow_mut().entry(block).or_insert_with(|| World::blockInfo(block))
    }

    pub fn isOpaque(&self, x: i32, y: i32, z: i32) -> bool { self.info(self.block(x, y, z)).opaque }

    pub fn isSolid(&self, x: i32, y: i32, z: i32) -> bool { self.info(self.block(x, y, z)).solid }

    pub fn isLiquid(&self, x: i32, y: i32, z: i32) -> bool { self.info(self.block(x, y, z)).liquid }

    pub fn isReplaceable(&self, x: i32, y: i32, z: i32) -> bool
    {
        let block: i32 = self.block(x, y, z);
        block == 0 || self.info(block).replaceable
    }

    fn leafy(&self, block: i32) -> bool { block == self.id("minecraft:leaves") || crate::ids::leaves(block) }
}
