use b173_api::world::{itemSlot, terrainSlot};
use std::cell::RefCell;
use std::collections::HashMap;

thread_local!
{
    static TERRAIN: RefCell<HashMap<String, i32>> = RefCell::new(HashMap::new());
    static ITEMS: RefCell<HashMap<String, i32>> = RefCell::new(HashMap::new());
}

fn cached(
    cache: &'static std::thread::LocalKey<RefCell<HashMap<String, i32>>>,
    name: &str,
    find: fn(&str) -> Option<i32>,
) -> i32
{
    if let Some(slot) = cache.with(|slots| slots.borrow().get(name).copied())
    {
        return slot;
    }

    // a lookup while registering only requests the sprite so a miss is never kept
    let Some(slot) = find(name)
    else
    {
        return 0;
    };

    cache.with(|slots| slots.borrow_mut().insert(name.to_owned(), slot));
    slot
}

// a terrain sprite's slot for the texture hooks, looked up once since slots never move after loading
pub(crate) fn terrain(sprite: &str) -> i32 { cached(&TERRAIN, sprite, terrainSlot) }

// an item sheet sprite's slot for icon hooks
pub(crate) fn item(sprite: &str) -> i32 { cached(&ITEMS, sprite, itemSlot) }
