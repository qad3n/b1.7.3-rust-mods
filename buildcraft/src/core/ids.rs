use b173_api::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

thread_local!
{
    // block ids by name for the current content generation, pipes ask for them many times a tick
    static BLOCKS: RefCell<(i32, HashMap<String, i32>)> = RefCell::new((-1, HashMap::new()));
}

// a block's numeric id, None while it is unknown
pub(crate) fn block(name: &str) -> Option<i32>
{
    let generation: i32 = contentGeneration();

    BLOCKS.with(|cache|
    {
        let mut cache = cache.borrow_mut();
        if cache.0 != generation
        {
            *cache = (generation, HashMap::new());
        }

        if let Some(id) = cache.1.get(name)
        {
            return Some(*id).filter(|id| *id > 0);
        }

        let id: i32 = Identifier::parse(name).ok().and_then(|id| resolve(RegistryKind::Block, &id).ok()).unwrap_or(0);
        cache.1.insert(name.to_owned(), id);
        Some(id).filter(|id| *id > 0)
    })
}

pub(crate) fn is(world: World, pos: BlockPos, name: &str) -> bool
{
    block(name).is_some_and(|id| world.block(pos) == id)
}
