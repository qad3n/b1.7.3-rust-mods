use b173_api::prelude::*;

// BuildCraftCore's gears and wrench: names, sprites and the shared tag other mods' gears join
pub(crate) const WOODEN_GEAR: &str = "buildcraft:wooden_gear";
pub(crate) const STONE_GEAR: &str = "buildcraft:stone_gear";
pub(crate) const IRON_GEAR: &str = "buildcraft:iron_gear";
pub(crate) const GOLD_GEAR: &str = "buildcraft:gold_gear";
pub(crate) const DIAMOND_GEAR: &str = "buildcraft:diamond_gear";
pub(crate) const WRENCH: &str = "buildcraft:wrench";

struct Gear
{
    m_name: &'static str,
    m_tag: &'static str,
    // the ring around the previous gear, a stick cross for the wooden one
    m_ring: &'static str,
    m_core: Option<&'static str>,
}

// BuildCraftCore.initializeGears in crafting order
const GEARS: [Gear; 5] = [
    Gear { m_name: "wooden_gear", m_tag: "c:gears/wood", m_ring: "minecraft:stick", m_core: None },
    Gear { m_name: "stone_gear", m_tag: "c:gears/stone", m_ring: "minecraft:cobblestone", m_core: Some(WOODEN_GEAR) },
    Gear { m_name: "iron_gear", m_tag: "c:gears/iron", m_ring: "minecraft:iron_ingot", m_core: Some(STONE_GEAR) },
    Gear { m_name: "gold_gear", m_tag: "c:gears/gold", m_ring: "minecraft:gold_ingot", m_core: Some(IRON_GEAR) },
    Gear { m_name: "diamond_gear", m_tag: "c:gears/diamond", m_ring: "minecraft:diamond", m_core: Some(GOLD_GEAR) },
];

// any mod's wrench in the shared tag turns machines, as BuildCraft's own does
const WRENCHES: &str = "c:tools/wrench";

pub(crate) fn holdsWrench(player: Player) -> bool
{
    let Some(held) = player.held().and_then(|stack| stack.item.name())
    else
    {
        return false;
    };

    tagValues(RegistryKind::Item, WRENCHES).contains(&held)
}

fn registerGear(ctx: &mut RegisterCtx, gear: &Gear) -> Result<()>
{
    let texture: String = format!("buildcraft:item/{}", gear.m_name);
    ctx.items().register(gear.m_name, ItemSpecBuilder::new().texture(&texture).tag(gear.m_tag))?;

    let result: ItemStack = ItemStack::of(format!("buildcraft:{}", gear.m_name).as_str(), 1);
    match gear.m_core
    {
        None => ctx.recipes().shaped(gear.m_name, result).pattern([" S ", "S S", " S "]).key('S', gear.m_ring).build(),

        Some(core) => ctx
            .recipes()
            .shaped(gear.m_name, result)
            .pattern([" I ", "IGI", " I "])
            .key('I', gear.m_ring)
            .key('G', core)
            .build(),
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for gear in &GEARS
    {
        registerGear(ctx, gear)?;
    }

    ctx.items().register("wrench", ItemSpecBuilder::new().texture("buildcraft:item/wrench").tag(WRENCHES))?;
    ctx.recipes()
        .shaped("wrench", ItemStack::of(WRENCH, 1))
        .pattern(["I I", " G ", " I "])
        .key('I', "minecraft:iron_ingot")
        .key('G', STONE_GEAR)
        .build()
}
