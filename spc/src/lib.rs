mod abilities;
mod commands;

use b173_api::prelude::*;

#[mod_main]
fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    abilities::register(ctx)?;
    commands::register(ctx)
}
