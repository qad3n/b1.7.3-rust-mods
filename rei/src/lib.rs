mod catalog;
mod cheat;
mod config;
mod display;
mod overlay;
mod spawner;

use b173_api::prelude::*;

#[mod_main]
fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    config::register(ctx)?;
    cheat::register(ctx)?;
    spawner::register(ctx)?;
    overlay::register(ctx)
}
