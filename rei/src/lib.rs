mod catalog;
mod cheat;
mod config;
mod overlay;

use b173_api::prelude::*;

#[mod_main]
fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    config::register(ctx)?;
    cheat::register(ctx)?;
    overlay::register(ctx)
}
