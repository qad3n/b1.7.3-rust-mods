use b173_api::prelude::*;

const RECEIVER: &str = "wirelessredstone:receiver";
const TRANSMITTER: &str = "wirelessredstone:transmitter";

// the AddRecipes of all six mods
pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.recipes()
        .shaped("receiver", ItemStack::of(RECEIVER, 1))
        .pattern(["IRI", "RLR", "IRI"])
        .key('I', vanilla::item::IRON_INGOT)
        .key('R', vanilla::item::REDSTONE)
        .key('L', vanilla::block::LEVER)
        .build()?;
    ctx.recipes()
        .shaped("transmitter", ItemStack::of(TRANSMITTER, 1))
        .pattern(["IRI", "RTR", "IRI"])
        .key('I', vanilla::item::IRON_INGOT)
        .key('R', vanilla::item::REDSTONE)
        .key('T', "minecraft:redstone_torch")
        .build()?;
    ctx.recipes()
        .shaped("clocker", ItemStack::of("wirelessredstone:clocker", 1))
        .pattern(["R R", " # ", "R R"])
        .key('R', "minecraft:repeater")
        .key('#', TRANSMITTER)
        .build()?;
    ctx.recipes()
        .shaped("remote", ItemStack::of("wirelessredstone:remote", 1))
        .pattern(["i", "#"])
        .key('i', "minecraft:redstone_torch")
        .key('#', TRANSMITTER)
        .build()?;
    ctx.recipes()
        .shaped("power_configurator", ItemStack::of("wirelessredstone:power_configurator", 1))
        .pattern(["R R", " X ", "R R"])
        .key('X', RECEIVER)
        .key('R', vanilla::item::REDSTONE)
        .build()?;
    ctx.recipes()
        .shaped("sniffer", ItemStack::of("wirelessredstone:sniffer", 1))
        .pattern(["X X", " X ", "X X"])
        .key('X', RECEIVER)
        .build()?;
    ctx.recipes()
        .shaped("triangulator", ItemStack::of("wirelessredstone:triangulator", 1))
        .pattern(["C", "X"])
        .key('X', RECEIVER)
        .key('C', "minecraft:compass")
        .build()
}
