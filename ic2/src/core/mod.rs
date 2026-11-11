pub(crate) mod dir;
pub(crate) mod math;
pub(crate) mod random;
pub(crate) mod sprites;
pub(crate) mod stacks;

// the values of a true or false block state property, false first so it is the default
pub(crate) const FLAG: [&str; 2] = ["false", "true"];

pub(crate) const NAMESPACE: &str = "ic2";

// the namespaced name of one of this mod's blocks or items
pub(crate) fn own(name: &str) -> String { format!("{NAMESPACE}:{name}") }

pub(crate) fn blockTexture(name: &str) -> String { format!("ic2:block/{name}") }

pub(crate) fn itemTexture(name: &str) -> String { format!("ic2:item/{name}") }

// IC2 named its sounds without a namespace, a pack may add them under assets/ic2/sounds
pub(crate) fn sound(name: &str) -> String { format!("ic2.{name}") }
