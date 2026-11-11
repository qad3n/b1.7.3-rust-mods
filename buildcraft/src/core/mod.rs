pub(crate) mod area;
pub(crate) mod dir;
pub(crate) mod ids;
pub(crate) mod items;
pub(crate) mod power;
pub(crate) mod random;
pub(crate) mod render;
pub(crate) mod stacks;

use b173_api::prelude::*;

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.config().options([
        Setting::choice(power::FRAMEWORK, power::PNEUMATIC, &[power::PNEUMATIC, power::REDSTONE])
            .label("Power framework")
            .comment("pneumatic machines run on engines, redstone machines work once per redstone pulse")
            .restart(),
        Setting::bool(power::CONTINUOUS, false)
            .label("Continuous current")
            .comment("set to true for allowing machines to be driven by continuous current"),
    ])?;

    power::register(ctx)?;
    items::register(ctx)
}
