use crate::names::{block, item};
use b173_api::prelude::*;

// grass dirt and farmland as beta flowers take root in, uberous soil joins it at registration
pub(crate) const SOIL: &str = "minecraft:flower_soil";
// every block that keeps leaves alive, as BlockLog.canSustainLeaves and the blocks that override it
pub(crate) const LOGS: &str = "minecraft:logs";
pub(crate) const LEAVES: &str = "twilightforest:leaves";
// rock material blocks that troll roots hang from
pub(crate) const STONY: &str = "twilightforest:stony";
pub(crate) const GIANT: &str = "twilightforest:giant_blocks";
pub(crate) const MAZESTONE: &str = "twilightforest:mazestone";
// what uberous soil turns into farmland grass or dirt for, as IPlantable plant types sort them
pub(crate) const CROPS: &str = "minecraft:crops";
pub(crate) const PLAINS_PLANTS: &str = "twilightforest:plains_plants";
pub(crate) const CAVE_PLANTS: &str = "twilightforest:cave_plants";
// what thorns grow from beside the usual ground, TFBiomeThornlands' sand stone and clay
pub(crate) const THORN_GROUND: &str = "twilightforest:thorn_ground";
// mobs that wear armor for the knightly weapons, EntityLivingBase.getTotalArmorValue above zero
pub(crate) const ARMORED: &str = "twilightforest:armored";
// the plain monsters a lich tears apart, EntityTFLich.popNearbyMob's instanceof checks
pub(crate) const LICH_POPPABLE: &str = "twilightforest:lich_poppable";
// the animals a naga's body bites harder
pub(crate) const NAGA_PREY: &str = "twilightforest:naga_prey";

struct TagRow
{
    m_registry: RegistryKind,
    m_tag: &'static str,
    m_entries: &'static [&'static str],
}

// groups other mods share with the forest, most of its own blocks join their tags where they register
const ROWS: [TagRow; 22] = [
    blockTag(LEAVES, &["#minecraft:leaves"]),
    blockTag(
        STONY,
        &["#minecraft:base_stone_overworld", "#c:cobblestones", "#c:sandstone_blocks", "#c:ores", "#c:obsidians"],
    ),
    blockTag(PLAINS_PLANTS, &["#minecraft:small_flowers", "#minecraft:saplings", "minecraft:tallgrass"]),
    blockTag(CAVE_PLANTS, &["minecraft:brown_mushroom", "minecraft:red_mushroom"]),
    blockTag(THORN_GROUND, &["#c:sandstone_blocks", "#minecraft:sand", "minecraft:clay", "#twilightforest:stony"]),
    itemTag("c:ingots", &[item::IRONWOOD_INGOT, item::FIERY_INGOT, item::STEELEAF, item::KNIGHTMETAL]),
    itemTag("c:ingots/ironwood", &[item::IRONWOOD_INGOT]),
    itemTag("c:ingots/fiery", &[item::FIERY_INGOT]),
    itemTag("c:ingots/steeleaf", &[item::STEELEAF]),
    itemTag("c:ingots/knightmetal", &[item::KNIGHTMETAL]),
    itemTag("c:raw_materials", &[item::RAW_IRONWOOD]),
    itemTag("c:raw_materials/ironwood", &[item::RAW_IRONWOOD]),
    blockTag("c:storage_blocks", &[block::KNIGHTMETAL_BLOCK]),
    blockTag("c:storage_blocks/knightmetal", &[block::KNIGHTMETAL_BLOCK]),
    itemTag("c:storage_blocks", &[block::KNIGHTMETAL_BLOCK]),
    itemTag("c:storage_blocks/knightmetal", &[block::KNIGHTMETAL_BLOCK]),
    entityTag(
        ARMORED,
        &[
            "goblin_knight_upper",
            "goblin_knight_lower",
            "helmet_crab",
            "armored_giant",
            "knight_phantom",
            "block_goblin",
        ],
    ),
    entityTag(
        LICH_POPPABLE,
        &["minecraft:skeleton", "minecraft:zombie", "minecraft:spider", "minecraft:creeper", "swarm_spider"],
    ),
    // the roving cube runs ai with no target task so it never fights
    entityTag(
        "c:passive",
        &[
            "wild_boar",
            "bighorn_sheep",
            "wild_deer",
            "forest_bunny",
            "forest_squirrel",
            "tiny_bird",
            "forest_raven",
            "glacier_penguin",
            "firefly",
            "quest_ram",
            "roving_cube",
        ],
    ),
    // a loyal zombie hunts monsters and turns only on whoever hurts it
    entityTag("c:neutral", &["loyal_zombie"]),
    entityTag(
        "c:hostile",
        &[
            "redcap",
            "redcap_sapper",
            "kobold",
            "skeleton_druid",
            "swarm_spider",
            "hedge_spider",
            "king_spider",
            "tower_broodling",
            "hostile_wolf",
            "mist_wolf",
            "winter_wolf",
            "twilight_wraith",
            "mosquito_swarm",
            "death_tome",
            "minotaur",
            "fire_beetle",
            "slime_beetle",
            "pinch_beetle",
            "maze_slime",
            "tower_golem",
            "tower_termite",
            "block_goblin",
            "goblin_knight_upper",
            "goblin_knight_lower",
            "helmet_crab",
            "yeti",
            "snow_guardian",
            "stable_ice_core",
            "unstable_ice_core",
            "troll",
            "ice_crystal",
            "adherent",
            "giant_miner",
            "armored_giant",
            "harbinger_cube",
            "mini_ghast",
            "tower_ghast",
            "naga",
            "naga_segment",
            "twilight_lich",
            "lich_minion",
            "minoshroom",
            "hydra",
            "hydra_head",
            "hydra_neck",
            "knight_phantom",
            "ur_ghast",
            "yeti_alpha",
            "snow_queen",
        ],
    ),
    entityTag(NAGA_PREY, &["minecraft:pig", "minecraft:cow", "minecraft:sheep", "minecraft:chicken", "minecraft:wolf"]),
];

const fn blockTag(tag: &'static str, entries: &'static [&'static str]) -> TagRow
{
    TagRow { m_registry: RegistryKind::Block, m_tag: tag, m_entries: entries }
}

const fn entityTag(tag: &'static str, entries: &'static [&'static str]) -> TagRow
{
    TagRow { m_registry: RegistryKind::Entity, m_tag: tag, m_entries: entries }
}

// an entity's kind sits in a tag, other mods' entities count when they join it
pub(crate) fn entityTagged(info: &EntityInfo, tag: &str) -> bool
{
    info.kind.as_ref().is_some_and(|kind| tagMembers(RegistryKind::Entity, tag).contains(kind))
}

const fn itemTag(tag: &'static str, entries: &'static [&'static str]) -> TagRow
{
    TagRow { m_registry: RegistryKind::Item, m_tag: tag, m_entries: entries }
}

// a leading # names another tag and a bare name means one of the forest's own
fn entry(text: &str) -> Result<TagEntry>
{
    let target: TagRef = match text.strip_prefix('#')
    {
        Some(tag) => TagRef::Tag(Identifier::parseIn(tag, &modId())?),
        None => TagRef::Id(Identifier::parseIn(text, &modId())?),
    };
    Ok(TagEntry { target, required: true })
}

// a block and its item join the same tag, as a modern block tag and its item tag share one name
pub(crate) fn both(ctx: &mut RegisterCtx, tag: &str, name: &str) -> Result<()>
{
    ctx.tags().add(RegistryKind::Block, tag, &[name])?;
    ctx.tags().add(RegistryKind::Item, tag, &[name])
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for row in &ROWS
    {
        let entries: Vec<TagEntry> = row.m_entries.iter().map(|text| entry(text)).collect::<Result<Vec<TagEntry>>>()?;
        ctx.tags().addEntries(row.m_registry, row.m_tag, entries, false)?;
    }

    Ok(())
}
