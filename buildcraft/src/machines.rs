use crate::core::{dir, ids};
use b173_api::prelude::*;

// BuildCraft's IMachine blocks, which pipes join though they hold no items
const MACHINES: [&str; 3] = ["buildcraft:quarry", "buildcraft:mining_well", "buildcraft:pump"];
const FRAME: &str = "buildcraft:frame";
const MINING_PIPE: &str = "buildcraft:mining_pipe";

pub(crate) fn isMachine(world: World, pos: BlockPos) -> bool { MACHINES.iter().any(|name| ids::is(world, pos, name)) }

// IMachine.isActive: a quarry or mining well while it digs, a pump always
pub(crate) fn active(world: World, pos: BlockPos) -> bool
{
    match MACHINES.iter().position(|name| ids::is(world, pos, name))
    {
        Some(0) => crate::factory::quarryActive(world, pos),
        Some(1) => crate::factory::wellActive(world, pos),
        Some(_) => true,
        None => false,
    }
}

fn faceToward(from: BlockPos, to: BlockPos) -> i32
{
    dir::all().find(|face| dir::step(from, *face) == to).unwrap_or(dir::UNKNOWN)
}

// blocks that implement IPipeConnection
fn connector(world: World, pos: BlockPos) -> bool
{
    crate::transport::isPipe(world, pos)
        || ids::is(world, pos, FRAME)
        || ids::is(world, pos, MINING_PIPE)
        || crate::energy::isEngine(world, pos)
        || ids::is(world, pos, MACHINES[0])
        || ids::is(world, pos, MACHINES[1])
}

// Utils.checkPipesConnections: one of the two must be a connector and every connector must agree
pub(crate) fn connected(world: World, first: BlockPos, second: BlockPos) -> bool
{
    let (firstJoins, secondJoins): (bool, bool) = (connector(world, first), connector(world, second));
    if !firstJoins && !secondJoins
    {
        return false;
    }

    (!firstJoins || acceptsPipe(world, first, second)) && (!secondJoins || acceptsPipe(world, second, first))
}

// the IPipeConnection side of Utils.checkPipesConnections: whether the block at pos lets a pipe at `from` join it
pub(crate) fn acceptsPipe(world: World, pos: BlockPos, from: BlockPos) -> bool
{
    let face: i32 = faceToward(pos, from);

    if crate::transport::isPipe(world, pos)
    {
        return crate::transport::pipeAccepts(world, pos, face);
    }

    if ids::is(world, pos, FRAME)
    {
        return ids::is(world, from, FRAME);
    }

    if ids::is(world, pos, MINING_PIPE)
    {
        return false;
    }

    if crate::energy::isEngine(world, pos)
    {
        return crate::energy::acceptsPipe(world, pos, face);
    }

    true
}
