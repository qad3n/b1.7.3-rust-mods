use crate::names::{Pick, handle};
use b173_api::{BlockPos, World, blockIs, blockTagged};
use std::cell::RefCell;
use std::collections::HashMap;

// the world a generator writes into, with block properties cached for the length of one populate call
pub(crate) struct GenWorld
{
    pub m_world: World,
    m_opaque: RefCell<HashMap<i32, bool>>,
}

impl GenWorld
{
    pub(crate) fn new(world: World) -> Self { Self { m_world: world, m_opaque: RefCell::new(HashMap::new()) } }

    pub(crate) fn block(&self, x: i32, y: i32, z: i32) -> i32 { self.m_world.block(BlockPos::new(x, y, z)) }

    pub(crate) fn isAir(&self, x: i32, y: i32, z: i32) -> bool { self.block(x, y, z) == 0 }

    pub(crate) fn is(&self, x: i32, y: i32, z: i32, name: &str) -> bool { blockIs(self.block(x, y, z), name) }

    pub(crate) fn tagged(&self, x: i32, y: i32, z: i32, tag: &str) -> bool { blockTagged(self.block(x, y, z), tag) }

    // the spot holds exactly this state
    pub(crate) fn holds(&self, x: i32, y: i32, z: i32, pick: Pick) -> bool
    {
        crate::names::holds(self.m_world, BlockPos::new(x, y, z), pick)
    }

    // world generation places blocks without neighbour updates, as WorldGenerator.setBlockAndNotifyAdequately
    pub(crate) fn put(&self, x: i32, y: i32, z: i32, pick: Pick)
    {
        let _ = handle(pick).place(self.m_world, BlockPos::new(x, y, z), false);
    }

    pub(crate) fn set(&self, x: i32, y: i32, z: i32, block: &'static str) { self.put(x, y, z, Pick::of(block)); }

    pub(crate) fn height(&self, x: i32, z: i32) -> i32 { self.m_world.height(x, z) }

    pub(crate) fn isOpaque(&self, x: i32, y: i32, z: i32) -> bool
    {
        let block: i32 = self.block(x, y, z);
        *self.m_opaque.borrow_mut().entry(block).or_insert_with(|| World::blockInfo(block).opaque)
    }
}
