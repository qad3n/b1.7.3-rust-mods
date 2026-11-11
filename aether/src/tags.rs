use b173_api::prelude::*;

// blocks Aether plants, saplings and trees take root in, BlockAetherFlower.canThisPlantGrowOnThisBlock
pub(crate) const SOIL: &str = "aether:soil";
// BlockAetherLog.harvestBlock: axes strong enough to chip golden amber out of golden oak
pub(crate) const AMBER_AXES: &str = "aether:amber_axes";
// the Gravitite tools achievement
pub(crate) const GRAVITITE_TOOLS: &str = "aether:gravitite_tools";
// ItemValkyrieTool reaches further than a plain tool
pub(crate) const VALKYRIE_TOOLS: &str = "aether:valkyrie_tools";
// EntityLiving.isEntityUndead in b1.7.3, what the holy sword smites
pub(crate) const UNDEAD: &str = "minecraft:undead";
// BlockDungeonBase, the stone the slider never breaks
pub(crate) const DUNGEON_STONE: &str = "aether:dungeon_stone";
// items whose getRarity is ItemsAether.aether_loot, the lore book shows their names green
pub(crate) const DUNGEON_LOOT: &str = "aether:dungeon_loot";
// ItemPigSlayer: anything pig like, the Java matched pig, phyg and taegore in the entity name
pub(crate) const PIGS: &str = "aether:pigs";

struct TagRow
{
    m_registry: RegistryKind,
    m_tag: &'static str,
    m_members: &'static [&'static str],
}

const SOILS: &[&str] = &["aether_grass", "enchanted_aether_grass", "aether_dirt"];
const LOGS: &[&str] = &["skyroot_log", "golden_oak_log"];
const LEAVES: &[&str] = &["skyroot_leaves", "golden_oak_leaves", "crystal_leaves", "crystal_fruit_leaves"];
const LOOT: &[&str] = &[
    "lore_book",
    "cloud_staff",
    "notch_hammer",
    "victory_medal",
    "lightning_knife",
    "pig_slayer",
    "vampire_blade",
    "life_shard",
    "valkyrie_lance",
    "phoenix_bow",
    "aerogel_slab",
    "flaming_sword",
    "lightning_sword",
    "holy_sword",
    "bronze_key",
    "silver_key",
    "golden_key",
    "valkyrie_pickaxe",
    "valkyrie_axe",
    "valkyrie_shovel",
    "neptune_helmet",
    "neptune_chestplate",
    "neptune_leggings",
    "neptune_boots",
    "phoenix_helmet",
    "phoenix_chestplate",
    "phoenix_leggings",
    "phoenix_boots",
    "obsidian_helmet",
    "obsidian_chestplate",
    "obsidian_leggings",
    "obsidian_boots",
    "valkyrie_helmet",
    "valkyrie_chestplate",
    "valkyrie_leggings",
    "valkyrie_boots",
    "sentry_boots",
    "neptune_gloves",
    "obsidian_gloves",
    "valkyrie_gloves",
    "phoenix_gloves",
    "swet_cape",
    "invisibility_cape",
    "agility_cape",
    "valkyrie_cape",
    "golden_feather",
    "regeneration_stone",
    "iron_bubble",
    "repulsion_shield",
    "ice_ring",
    "ice_pendant",
    "chain_gloves",
];
const SAPLINGS: &[&str] = &["skyroot_sapling", "golden_oak_sapling"];
const FLOWERS: &[&str] = &["white_flower", "purple_flower"];
const ORES: &[&str] = &["ambrosium_ore", "zanite_ore", "gravitite_ore"];

// logs and leaves blocks join minecraft:logs and minecraft:leaves through their block specs
const TAGS: [TagRow; 27] = [
    TagRow { m_registry: RegistryKind::Item, m_tag: DUNGEON_LOOT, m_members: LOOT },
    TagRow { m_registry: RegistryKind::Block, m_tag: SOIL, m_members: SOILS },
    TagRow { m_registry: RegistryKind::Block, m_tag: "minecraft:dirt", m_members: SOILS },
    TagRow { m_registry: RegistryKind::Block, m_tag: "minecraft:planks", m_members: &["skyroot_planks"] },
    TagRow { m_registry: RegistryKind::Block, m_tag: "minecraft:saplings", m_members: SAPLINGS },
    TagRow { m_registry: RegistryKind::Block, m_tag: "minecraft:small_flowers", m_members: FLOWERS },
    TagRow { m_registry: RegistryKind::Block, m_tag: "minecraft:flowers", m_members: FLOWERS },
    TagRow { m_registry: RegistryKind::Block, m_tag: "c:ores", m_members: ORES },
    TagRow { m_registry: RegistryKind::Block, m_tag: "c:ores/ambrosium", m_members: &["ambrosium_ore"] },
    TagRow { m_registry: RegistryKind::Block, m_tag: "c:ores/zanite", m_members: &["zanite_ore"] },
    TagRow { m_registry: RegistryKind::Block, m_tag: "c:ores/gravitite", m_members: &["gravitite_ore"] },
    TagRow { m_registry: RegistryKind::Block, m_tag: "c:storage_blocks/zanite", m_members: &["zanite_block"] },
    TagRow
    {
        m_registry: RegistryKind::Block,
        m_tag: DUNGEON_STONE,
        m_members: &[
            "carved_stone",
            "sentry_stone",
            "angelic_stone",
            "light_angelic_stone",
            "hellfire_stone",
            "light_hellfire_stone",
            "locked_carved_stone",
            "locked_sentry_stone",
            "locked_angelic_stone",
            "locked_light_angelic_stone",
            "locked_hellfire_stone",
            "locked_light_hellfire_stone",
        ],
    },
    TagRow { m_registry: RegistryKind::Item, m_tag: "minecraft:logs", m_members: LOGS },
    TagRow { m_registry: RegistryKind::Item, m_tag: "minecraft:leaves", m_members: LEAVES },
    TagRow { m_registry: RegistryKind::Item, m_tag: "minecraft:planks", m_members: &["skyroot_planks"] },
    TagRow { m_registry: RegistryKind::Item, m_tag: "minecraft:saplings", m_members: SAPLINGS },
    TagRow { m_registry: RegistryKind::Item, m_tag: "minecraft:small_flowers", m_members: FLOWERS },
    TagRow { m_registry: RegistryKind::Item, m_tag: "c:ores", m_members: ORES },
    TagRow { m_registry: RegistryKind::Item, m_tag: "c:gems/zanite", m_members: &["zanite_gemstone"] },
    TagRow { m_registry: RegistryKind::Item, m_tag: "c:rods/wooden", m_members: &["skyroot_stick"] },
    TagRow { m_registry: RegistryKind::Item, m_tag: "c:storage_blocks/zanite", m_members: &["zanite_block"] },
    TagRow
    {
        m_registry: RegistryKind::Item,
        m_tag: AMBER_AXES,
        m_members: &["zanite_axe", "gravitite_axe", "valkyrie_axe", "minecraft:diamond_axe"],
    },
    TagRow
    {
        m_registry: RegistryKind::Item,
        m_tag: GRAVITITE_TOOLS,
        m_members: &["gravitite_shovel", "gravitite_axe", "gravitite_pickaxe"],
    },
    TagRow
    {
        m_registry: RegistryKind::Item,
        m_tag: VALKYRIE_TOOLS,
        m_members: &["valkyrie_pickaxe", "valkyrie_axe", "valkyrie_shovel"],
    },
    TagRow
    {
        m_registry: RegistryKind::Entity,
        m_tag: UNDEAD,
        m_members: &["minecraft:zombie", "minecraft:skeleton", "minecraft:pig_zombie"],
    },
    TagRow
    {
        m_registry: RegistryKind::Entity,
        m_tag: PIGS,
        m_members: &["minecraft:pig", "minecraft:pig_zombie", "phyg"],
    },
];

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for row in &TAGS
    {
        ctx.tags().add(row.m_registry, row.m_tag, row.m_members)?;
    }

    Ok(())
}
