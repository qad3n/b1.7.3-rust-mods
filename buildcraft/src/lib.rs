mod builders;
mod core;
mod energy;
mod factory;
mod machines;
mod transport;

use b173_api::prelude::*;

// BuildCraft 2.1.1's five mods in one: core, transport, energy, factory and builders
#[mod_main]
fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    core::register(ctx)?;
    transport::register(ctx)?;
    energy::register(ctx)?;
    factory::register(ctx)?;
    builders::register(ctx)
}
