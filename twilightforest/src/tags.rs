use crate::names::{block, item};
use b173_api::prelude::*;

// grass dirt and farmland as beta flowers take root in, uberous soil joins it at registration
pub(crate) const SOIL: &str = "minecraft:flower_soil";
// every block that keeps leaves alive, as BlockLog.canSustainLeaves and the blocks that override it
pub(crate) const LOGS: &str = "minecraft:logs";
// leaves a broken log tells to decay
pub(crate) const DECAYING_LEAVES: &str = "minecraft:leaves";
pub(crate) const LEAVES: &str = "twilightforest:leaves";
// rock material blocks that troll roots hang from
pub(crate) const STONY: &str = "twilightforest:stony";
pub(crate) const GIANT: &str = "twilightforest:giant_blocks";
pub(crate) const MAZESTONE: &str = "twilightforest:mazestone";
// what uberous soil turns into farmland grass or dirt for, as IPlantable plant types sort them
pub(crate) const CROPS: &str = "minecraft:crops";
pub(crate) const PLAINS_PLANTS: &str = "twilightforest:plains_plants";
pub(crate) const CAVE_PLANTS: &str = "twilightforest:cave_plants";

struct TagRow
{
    m_registry: RegistryKind,
    m_tag: &'static str,
    m_entries: &'static [&'static str],
}

// groups other mods share with the forest, most of its own blocks join their tags where they register
const ROWS: [TagRow; 13] = [
    blockTag(LEAVES, &["#minecraft:leaves"]),
    blockTag(
        STONY,
        &["#minecraft:base_stone_overworld", "#c:cobblestones", "#c:sandstone_blocks", "#c:ores", "#c:obsidians"],
    ),
    blockTag(PLAINS_PLANTS, &["#minecraft:small_flowers", "#minecraft:saplings", "minecraft:tallgrass"]),
    blockTag(CAVE_PLANTS, &["minecraft:brown_mushroom", "minecraft:red_mushroom"]),
    itemTag("c:ingots", &[item::IRONWOOD_INGOT, item::FIERY_INGOT, item::STEELEAF, item::KNIGHTMETAL]),
    itemTag("c:ingots/ironwood", &[item::IRONWOOD_INGOT]),
    itemTag("c:ingots/fiery", &[item::FIERY_INGOT]),
    itemTag("c:ingots/steeleaf", &[item::STEELEAF]),
    itemTag("c:ingots/knightmetal", &[item::KNIGHTMETAL]),
    blockTag("c:storage_blocks", &[block::KNIGHTMETAL_BLOCK]),
    blockTag("c:storage_blocks/knightmetal", &[block::KNIGHTMETAL_BLOCK]),
    itemTag("c:storage_blocks", &[block::KNIGHTMETAL_BLOCK]),
    itemTag("c:storage_blocks/knightmetal", &[block::KNIGHTMETAL_BLOCK]),
];

const fn blockTag(tag: &'static str, entries: &'static [&'static str]) -> TagRow
{
    TagRow { m_registry: RegistryKind::Block, m_tag: tag, m_entries: entries }
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
