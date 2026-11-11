use crate::names;
use b173_api::prelude::*;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;

// up to three named properties of a state, unused places empty
pub(crate) type Props = [Option<(&'static str, &'static str)>; 3];

// a state as generators name it, a block with some named properties, or a vanilla block's metadata
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Key
{
    Plain(&'static str),
    Props(&'static str, Props),
    Meta(&'static str, i32),
}

thread_local!
{
    // each handle resolves its id and metadata again after a remap, so generators never keep stale numbers
    static HANDLES: RefCell<HashMap<Key, StateHandle>> = RefCell::new(HashMap::new());
}

fn handled<R>(key: Key, run: impl FnOnce(&StateHandle) -> R) -> R
{
    HANDLES.with(|handles|
    {
        let mut handles: RefMut<HashMap<Key, StateHandle>> = handles.borrow_mut();
        let handle: &StateHandle = handles.entry(key).or_insert_with(|| StateHandle::new(build(key)));
        run(handle)
    })
}

fn resolve(key: Key) -> (i32, i32) { handled(key, StateHandle::numeric) }

// the cached handle of a block's default state, for engine calls that take a state, a bare name means the forest's
pub(crate) fn withPlain<R>(name: &'static str, run: impl FnOnce(&StateHandle) -> R) -> R
{
    handled(Key::Plain(name), run)
}

fn build(key: Key) -> StateData
{
    match key
    {
        Key::Plain(name) => names::state(name),
        Key::Props(name, props) =>
        {
            props.into_iter().flatten().fold(names::state(name), |state, (property, value)| state.with(property, value))
        }
        Key::Meta(name, meta) => names::state(name).with(names::META, meta),
    }
}

// the id and metadata a block's default state is stored as this epoch, a bare name means one of the forest's own
pub(crate) fn plain(name: &'static str) -> (i32, i32) { resolve(Key::Plain(name)) }

pub(crate) fn with(name: &'static str, property: &'static str, value: &'static str) -> (i32, i32)
{
    props(name, [Some((property, value)), None, None])
}

pub(crate) fn props(name: &'static str, props: Props) -> (i32, i32) { resolve(Key::Props(name, props)) }

pub(crate) fn withTwo(name: &'static str, [first, second]: [(&'static str, &'static str); 2]) -> (i32, i32)
{
    props(name, [Some(first), Some(second), None])
}

// a block without named states placed with the metadata beta stores for it
pub(crate) fn meta(name: &'static str, meta: i32) -> (i32, i32) { resolve(Key::Meta(name, meta)) }

pub(crate) fn id(name: &'static str) -> i32 { plain(name).0 }

// a block and its metadata as generators place it, resolved from a named state this epoch
pub(crate) type Placed = (i32, i32);

pub(crate) const AIR: Placed = (0, 0);
