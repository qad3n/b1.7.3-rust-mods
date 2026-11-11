use crate::core::dir;
use b173_api::prelude::*;
use std::rc::Rc;

// BlockQuarry, BlockMiningWell, BlockBuilder and BlockTemplate face a side, an unplaced item faces down
pub(crate) const FACING: &str = "facing";

// one property listed in the order its values were stored as metadata before blocks had states
pub(crate) fn register(ctx: &mut RegisterCtx, block: &str, name: &str, values: &[&str]) -> Result<()>
{
    let properties: Vec<PropertySpec> = vec![PropertySpec::new(name, values)];
    ctx.blocks().states(block, BlockStatesSpec { properties, ..BlockStatesSpec::default() })
}

// model cuboids pick states by metadata, and a block under 16 states stores its states in declaration order
pub(crate) fn metas(values: &[&str], shown: impl Fn(&str) -> bool) -> Vec<i32>
{
    (0..).zip(values).filter(|(_, value)| shown(value)).map(|(meta, _)| meta).collect()
}

// the face a property names in a cached state, None for a value that names no face
pub(crate) fn face(state: Option<Rc<StateData>>, property: &str) -> Option<i32> { dir::named(state?.get(property)?) }

pub(crate) fn faceAt(world: World, pos: BlockPos, property: &str) -> Option<i32> { face(world.stateAt(pos), property) }

// another state of the same block keeps its tile entity, as BuildCraft's metadata writes did
pub(crate) fn set(world: World, pos: BlockPos, property: &str, value: &str)
{
    let Some(block) = world.blockName(pos)
    else
    {
        warn(&format!("no block at {pos:?} to set {property} on"));
        return;
    };

    if let Err(error) = world.setBlockState(pos, StateData::of(block).with(property, value))
    {
        warn(&format!("could not set {property} to {value} at {pos:?}: {error}"));
    }
}
