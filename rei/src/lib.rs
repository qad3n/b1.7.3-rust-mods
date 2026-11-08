mod catalog;
mod cheat;
mod overlay;

use b173_api::prelude::*;

#[mod_main]
fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    cheat::register(ctx)?;
    overlay::register(ctx)
}
