use b173_api::prelude::*;

const MOD_NAME: &str = "tooltip.mod_name";

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.config().options([Setting::bool(MOD_NAME, true)
        .label("Mod Name")
        .comment("Shows the mod an item comes from under its name in tooltips")
        .client()])
}

pub(crate) fn modName() -> bool { Config.getBool(MOD_NAME) }
