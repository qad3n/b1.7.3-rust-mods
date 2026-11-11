use crate::names;
use b173_api::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

// a state as generators name it, a block and at most one property, or a vanilla block's metadata
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Key
{
    Plain(&'static str),
    With(&'static str, &'static str, &'static str),
    WithTwo(&'static str, [(&'static str, &'static str); 2]),
    Meta(&'static str, i32),
}

thread_local!
{
    // each handle resolves its id and metadata again after a remap, so generators never keep stale numbers
    static HANDLES: RefCell<HashMap<Key, StateHandle>> = RefCell::new(HashMap::new());
}

fn resolve(key: Key) -> (i32, i32)
{
    HANDLES.with(|handles|
    {
        let mut handles = handles.borrow_mut();
        let handle: &StateHandle = handles.entry(key).or_insert_with(|| StateHandle::new(build(key)));
        handle.numeric()
    })
}

fn build(key: Key) -> StateData
{
    match key
    {
        Key::Plain(name) => names::state(name),
        Key::With(name, property, value) => names::state(name).with(property, value),
        Key::WithTwo(name, [(first, one), (second, two)]) => names::state(name).with(first, one).with(second, two),
        Key::Meta(name, meta) => names::state(name).with(names::META, meta),
    }
}

// the id and metadata a block's default state is stored as this epoch, a bare name means one of the forest's own
pub(crate) fn plain(name: &'static str) -> (i32, i32) { resolve(Key::Plain(name)) }

pub(crate) fn with(name: &'static str, property: &'static str, value: &'static str) -> (i32, i32)
{
    resolve(Key::With(name, property, value))
}

pub(crate) fn withTwo(name: &'static str, properties: [(&'static str, &'static str); 2]) -> (i32, i32)
{
    resolve(Key::WithTwo(name, properties))
}

// a block without named states placed with the metadata beta stores for it
pub(crate) fn meta(name: &'static str, meta: i32) -> (i32, i32) { resolve(Key::Meta(name, meta)) }

pub(crate) fn id(name: &'static str) -> i32 { plain(name).0 }
