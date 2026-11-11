mod blocks;
mod core;
mod electric;
mod energy;
mod explosives;
mod gear;
#[cfg(test)]
mod host_stub;
mod items;
mod keys;
mod machines;
mod recipes;
mod tools;
mod world;

use b173_api::prelude::*;

// IndustrialCraft 2 v1.00 for beta 1.7.3
#[mod_main]
fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    blocks::register(ctx)?;
    items::register(ctx)?;
    electric::register(ctx)?;
    energy::register(ctx)?;
    machines::register(ctx)?;
    keys::register(ctx)?;
    tools::register(ctx)?;
    gear::register(ctx)?;
    explosives::register(ctx)?;
    recipes::register(ctx)?;
    world::register(ctx)
}
