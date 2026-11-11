use crate::core::dir;
use b173_api::prelude::*;

// BuildCraft's IMachine blocks, which pipes join though they hold no items
pub(crate) const MACHINES: &str = "buildcraft:machines";
// blocks besides pipes that implement IPipeConnection
pub(crate) const CONNECTORS: &str = "buildcraft:pipe_connections";
const FRAME: &str = "buildcraft:frame";
const MINING_PIPE: &str = "buildcraft:mining_pipe";

// IMachine.isActive: a quarry or mining well while it digs, a pump always
struct Machine
{
    m_name: &'static str,
    m_active: fn(World, BlockPos) -> bool,
}

const ACTIVITY: [Machine; 3] = [
    Machine { m_name: "quarry", m_active: crate::factory::quarryActive },
    Machine { m_name: "mining_well", m_active: crate::factory::wellActive },
    Machine { m_name: "pump", m_active: always },
];

fn always(_world: World, _pos: BlockPos) -> bool { true }

pub(crate) fn isMachine(world: World, pos: BlockPos) -> bool { world.tagged(pos, MACHINES) }

pub(crate) fn active(world: World, pos: BlockPos) -> bool
{
    let block: i32 = world.block(pos);
    ACTIVITY.iter().find(|machine| blockIs(block, machine.m_name)).is_some_and(|machine| (machine.m_active)(world, pos))
}

fn faceToward(from: BlockPos, to: BlockPos) -> i32
{
    dir::all().find(|face| dir::step(from, *face) == to).unwrap_or(dir::UNKNOWN)
}

fn connector(world: World, pos: BlockPos) -> bool
{
    crate::transport::isPipe(world, pos) || world.tagged(pos, CONNECTORS)
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

    if world.is(pos, FRAME)
    {
        return world.is(from, FRAME);
    }

    if world.is(pos, MINING_PIPE)
    {
        return false;
    }

    if crate::energy::isEngine(world, pos)
    {
        return crate::energy::acceptsPipe(world, pos, face);
    }

    true
}
