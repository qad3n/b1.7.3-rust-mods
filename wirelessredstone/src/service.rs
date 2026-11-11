use crate::blocks::{CLOCKER, RECEIVER, TRANSMITTER, Tile};
use crate::ether;
use b173_api::prelude::*;
use b173_api::world::World;
use wirelessredstone_api::{LOOKUP, Near, Query, SERVICE, VERSION, methods};

// other mods read frequencies through the loader's service registry instead of reaching into the ether
struct Ether;

impl Service for Ether
{
    fn call(&self, method: &str, args: &[u8]) -> Result<Vec<u8>>
    {
        match method
        {
            methods::STATE =>
            {
                let query: Query = fromBytes(args)?;
                Ok(toBytes(&ether::state(World { dimension: query.world }, query.freq)))
            }

            methods::ACTIVE =>
            {
                let dimension: i32 = fromBytes(args)?;
                Ok(toBytes(&ether::activeFrequencies(World { dimension })))
            }

            methods::CLOSEST =>
            {
                let near: Near = fromBytes(args)?;
                Ok(toBytes(&ether::closestActive(World { dimension: near.world }, near.x, near.z, near.freq)))
            }

            _ => Err(Error(format!("ether has no method {method}"))),
        }
    }
}

struct Node;

impl Lookup for Node
{
    fn call(&self, world: i32, pos: BlockPos, _side: i32, method: &str, _args: &[u8]) -> Result<Vec<u8>>
    {
        match method
        {
            methods::FREQUENCY => Ok(toBytes(&World { dimension: world }.tileData::<Tile>(pos).m_freq)),
            _ => Err(Error(format!("wireless node has no method {method}"))),
        }
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.services().provide(SERVICE, VERSION, Ether)?;

    for block in [RECEIVER, TRANSMITTER, CLOCKER]
    {
        ctx.services().lookup(LOOKUP, block, Node)?;
    }

    Ok(())
}
