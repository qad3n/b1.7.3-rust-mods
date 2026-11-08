use crate::ids::Palette;
use b173_api::{BlockPos, World};
use std::cell::RefCell;
use std::collections::HashMap;

// the world a generator writes into, with block properties cached for the length of one populate call
pub struct GenWorld
{
    pub m_world: World,
    pub m_palette: Palette,
    m_opaque: RefCell<HashMap<i32, bool>>,
    m_ids: RefCell<HashMap<&'static str, i32>>,
}

impl GenWorld
{
    pub fn new(world: World) -> GenWorld
    {
        GenWorld {
            m_world: world,
            m_palette: Palette::load(),
            m_opaque: RefCell::new(HashMap::new()),
            m_ids: RefCell::new(HashMap::new()),
        }
    }

    pub fn block(&self, x: i32, y: i32, z: i32) -> i32 { self.m_world.block(BlockPos::new(x, y, z)) }

    pub fn meta(&self, x: i32, y: i32, z: i32) -> i32 { self.m_world.meta(BlockPos::new(x, y, z)) }

    pub fn isAir(&self, x: i32, y: i32, z: i32) -> bool { self.block(x, y, z) == 0 }

    // world generation places blocks without neighbour updates, as WorldGenerator.setBlockAndNotifyAdequately
    pub fn set(&self, x: i32, y: i32, z: i32, block: i32) { self.setWithMeta(x, y, z, block, 0); }

    pub fn setWithMeta(&self, x: i32, y: i32, z: i32, block: i32, meta: i32)
    {
        let _ = self.m_world.setBlockQuiet(BlockPos::new(x, y, z), block, meta);
    }

    pub fn height(&self, x: i32, z: i32) -> i32 { self.m_world.height(x, z) }

    // a block id by registry name, resolved once per populate call
    pub fn id(&self, name: &'static str) -> i32
    {
        *self.m_ids.borrow_mut().entry(name).or_insert_with(|| crate::ids::blockId(name))
    }

    pub fn isOpaque(&self, x: i32, y: i32, z: i32) -> bool
    {
        let block: i32 = self.block(x, y, z);
        *self.m_opaque.borrow_mut().entry(block).or_insert_with(|| World::blockInfo(block).opaque)
    }
}
