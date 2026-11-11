use b173_api::prelude::*;

pub(crate) const WRENCH: &str = "buildcraft:wrench";

// the shared tags BuildCraft's gears join, recipes take any mod's gear of the tier
pub(crate) mod gear
{
    pub const WOOD: &str = "c:gears/wood";
    pub const STONE: &str = "c:gears/stone";
    pub const IRON: &str = "c:gears/iron";
    pub const GOLD: &str = "c:gears/gold";
    pub const DIAMOND: &str = "c:gears/diamond";
}

// the vanilla materials recipes take by tag, so other mods' equivalents work too
pub(crate) mod material
{
    pub const PLANKS: &str = "minecraft:planks";
    pub const STONE: &str = "c:stones";
    pub const COBBLESTONE: &str = "c:cobblestones";
    pub const IRON: &str = "c:ingots/iron";
    pub const GOLD: &str = "c:ingots/gold";
    pub const DIAMOND: &str = "c:gems/diamond";
    pub const OBSIDIAN: &str = "c:obsidians";
    pub const STICK: &str = "c:rods/wooden";
    pub const REDSTONE: &str = "c:dusts/redstone";
    pub const GLASS: &str = "c:glass_blocks";
    pub const CHEST: &str = "c:chests";
    pub const CRAFTING_TABLE: &str = "c:player_workstations/crafting_tables";
}

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
    Gear { m_name: "wooden_gear", m_tag: gear::WOOD, m_ring: material::STICK, m_core: None },
    Gear { m_name: "stone_gear", m_tag: gear::STONE, m_ring: material::COBBLESTONE, m_core: Some(gear::WOOD) },
    Gear { m_name: "iron_gear", m_tag: gear::IRON, m_ring: material::IRON, m_core: Some(gear::STONE) },
    Gear { m_name: "gold_gear", m_tag: gear::GOLD, m_ring: material::GOLD, m_core: Some(gear::IRON) },
    Gear { m_name: "diamond_gear", m_tag: gear::DIAMOND, m_ring: material::DIAMOND, m_core: Some(gear::GOLD) },
];

// any mod's wrench in the shared tag turns machines, as BuildCraft's own does
const WRENCHES: &str = "c:tools/wrench";

pub(crate) fn holdsWrench(player: Player) -> bool { player.held().is_some_and(|stack| stack.item.tagged(WRENCHES)) }

fn registerGear(ctx: &mut RegisterCtx, gear: &Gear) -> Result<()>
{
    let texture: String = format!("buildcraft:item/{}", gear.m_name);
    ctx.items().register(gear.m_name, ItemSpecBuilder::new().texture(&texture).tag(gear.m_tag))?;

    let result: ItemStack = ItemStack::of(format!("buildcraft:{}", gear.m_name).as_str(), 1);
    match gear.m_core
    {
        None =>
        {
            ctx.recipes().shaped(gear.m_name, result).pattern([" S ", "S S", " S "]).keyTag('S', gear.m_ring).build()
        }

        Some(core) => ctx
            .recipes()
            .shaped(gear.m_name, result)
            .pattern([" I ", "IGI", " I "])
            .keyTag('I', gear.m_ring)
            .keyTag('G', core)
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
        .keyTag('I', material::IRON)
        .keyTag('G', gear::STONE)
        .build()
}
