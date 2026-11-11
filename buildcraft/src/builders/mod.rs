mod blueprint;
mod filler;
mod landmark;
mod marker;
mod stairs;

use crate::core::area::Area;
use b173_api::prelude::*;

pub(crate) const TEMPLATE: &str = "buildcraft:template";

// the dyes BuildCraftBuilders' recipes take, by beta dye damage
pub(crate) mod dye
{
    use b173_api::prelude::*;

    pub const INK: i32 = 0;
    pub const LAPIS: i32 = 4;
    pub const YELLOW: i32 = 11;

    pub fn stack(damage: i32) -> ItemStack { ItemStack::of("minecraft:dye", 1).withDamage(damage) }

    // the filler, builder and template table share one shape around a landmark and a crafting table
    pub fn recipe(
        ctx: &mut RegisterCtx,
        name: &str,
        gear: &str,
        chest: impl FnOnce(ShapedBuilder) -> ShapedBuilder,
    ) -> Result<()>
    {
        let shape: ShapedBuilder = ctx
            .recipes()
            .shaped(name, ItemStack::of(format!("buildcraft:{name}").as_str(), 1))
            .pattern(["btb", "ycy", "gCg"])
            .keyStack('b', stack(INK))
            .key('t', "buildcraft:marker")
            .keyStack('y', stack(YELLOW))
            .keyTag('c', crate::core::items::material::CRAFTING_TABLE)
            .keyTag('g', gear);
        chest(shape).build()
    }
}

// Utils.getNearbyAreaProvider then IAreaProvider.removeFromWorld, the box a machine takes from a landmark beside it
pub(crate) fn takeArea(world: World, pos: BlockPos) -> Option<Area>
{
    let (marker, area) = marker::areaNear(world, pos)?;
    marker::remove(world, marker);
    Some(area)
}

// ItemTemplate.getItemNameIS: "Template #N" for a drawn template, "Blank Template" otherwise
struct TemplateItem;

#[item_behavior]
impl ItemBehavior for TemplateItem
{
    fn name(&self, stack: ItemStack) -> Option<String>
    {
        (stack.damage > 0).then(|| translate("buildcraft.template").replace("%s", &stack.damage.to_string()))
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    // ItemTemplate keeps the blueprint number in its damage, as beta maps keep theirs, and names itself after it
    ctx.items().register(
        "template",
        ItemSpecBuilder::new().texture("buildcraft:item/template").maxStack(1).behavior(TemplateItem),
    )?;
    ctx.recipes()
        .shaped("template", ItemStack::of(TEMPLATE, 1))
        .pattern(["ppp", "pip", "ppp"])
        .keyStack('i', dye::stack(dye::INK))
        .key('p', "minecraft:paper")
        .build()?;
    marker::register(ctx)?;
    filler::register(ctx)?;
    blueprint::register(ctx)
}
