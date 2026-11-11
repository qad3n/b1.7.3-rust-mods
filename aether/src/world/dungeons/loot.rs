use crate::config;
use b173_api::PoolBuilder;
use b173_api::prelude::*;
use b173_api::terrain::Random;
use b173_api::world::rollLoot;

// one pick of a dungeon table: its weight out of the table's total and its count range
struct Prize
{
    m_item: &'static str,
    m_weight: i32,
    m_count: [i32; 2],
}

const fn one(item: &'static str, weight: i32) -> Prize { Prize { m_item: item, m_weight: weight, m_count: [1, 1] } }

const fn some(item: &'static str, weight: i32, most: i32) -> Prize
{
    Prize { m_item: item, m_weight: weight, m_count: [1, most] }
}

// each table gives every Java switch case the same weight and splits it the way the case's own draws do, so a pick
// has exactly the odds of the source's nested rolls

// BronzeDungeon.getNormalLoot, 15 cases of 20
const BRONZE_ROOM: &[Prize] = &[
    one("zanite_pickaxe", 20),
    one("zanite_axe", 20),
    one("zanite_sword", 20),
    one("zanite_shovel", 20),
    one("swet_cape", 20),
    some("ambrosium_shard", 20, 10),
    some("golden_dart", 20, 5),
    some("poison_dart", 20, 3),
    some("enchanted_dart", 20, 3),
    one("aether_tune", 1),
    one("skyroot_poison_bucket", 20),
    one("minecraft:record_cat", 2),
    one("iron_ring", 5),
    one("golden_ring", 2),
    one("ambrosium_torch", 90),
];

// BronzeDungeon.getBronzeLoot, 10 cases of 2
const BRONZE_TREASURE: &[Prize] = &[
    some("blue_gummy_swet", 1, 7),
    some("golden_gummy_swet", 1, 7),
    one("phoenix_bow", 2),
    one("flaming_sword", 2),
    one("notch_hammer", 2),
    some("lightning_knife", 2, 20),
    one("valkyrie_lance", 2),
    one("agility_cape", 2),
    one("sentry_boots", 2),
    one("repulsion_shield", 2),
    one("cloud_staff", 2),
];

// ComponentSilverDungeon.getNormalLoot, 16 cases of 1200, its last three cases falling through the pendant odds
const SILVER_ROOM: &[Prize] = &[
    one("zanite_pickaxe", 1200),
    one("skyroot_bucket", 1200),
    one("golden_dart_shooter", 1200),
    one("blue_moa_egg", 1200),
    one("white_moa_egg", 1200),
    some("ambrosium_shard", 1200, 10),
    some("golden_dart", 1200, 5),
    some("poison_dart", 1200, 3),
    some("enchanted_dart", 1200, 3),
    one("aether_tune", 60),
    one("skyroot_poison_bucket", 1200),
    one("ascending_dawn", 120),
    one("zanite_boots", 600),
    one("zanite_helmet", 300),
    one("zanite_leggings", 150),
    one("zanite_chestplate", 75),
    one("iron_pendant", 300),
    one("golden_pendant", 210),
    one("zanite_ring", 206),
    some("ambrosium_torch", 5179, 4),
];

// ComponentSilverDungeon.getSilverLoot, 14 cases of 8; cases 12 and 13 hold the configurable cape and feather
const SILVER_TREASURE: &[Prize] = &[
    some("blue_gummy_swet", 4, 15),
    some("golden_gummy_swet", 4, 15),
    one("lightning_sword", 8),
    one("valkyrie_axe", 4),
    one("valkyrie_shovel", 2),
    one("valkyrie_pickaxe", 1),
    one("holy_sword", 8),
    one("valkyrie_helmet", 8),
    one("regeneration_stone", 8),
    one("neptune_helmet", 4),
    one("neptune_leggings", 2),
    one("neptune_chestplate", 1),
    one("neptune_boots", 4),
    one("neptune_gloves", 4),
    one("valkyrie_boots", 10),
    one("valkyrie_gloves", 10),
    one("valkyrie_leggings", 8),
    one("valkyrie_chestplate", 4),
    one("invisibility_cape", 2),
];
const SILVER_CASE: i32 = 8;

// ComponentGoldenDungeon.getGoldLoot, 10 cases of 8
const GOLD_TREASURE: &[Prize] = &[
    one("iron_bubble", 8),
    one("vampire_blade", 8),
    one("pig_slayer", 8),
    one("phoenix_helmet", 4),
    one("phoenix_leggings", 2),
    one("phoenix_chestplate", 1),
    one("phoenix_boots", 4),
    one("phoenix_gloves", 4),
    one("life_shard", 8),
    one("gravitite_helmet", 4),
    one("gravitite_leggings", 2),
    one("gravitite_chestplate", 1),
    one("gravitite_boots", 4),
    one("gravitite_gloves", 4),
    one("chain_gloves", 8),
    one("obsidian_chestplate", 10),
];

pub(crate) const BRONZE_ROOM_TABLE: &str = "bronze_room";
pub(crate) const BRONZE_TREASURE_TABLE: &str = "bronze_treasure";
pub(crate) const SILVER_ROOM_TABLE: &str = "silver_room";
pub(crate) const SILVER_TREASURE_TABLE: &str = "silver_treasure";
pub(crate) const GOLD_TREASURE_TABLE: &str = "gold_treasure";

const TABLES: [(&str, &[Prize]); 5] = [
    (BRONZE_ROOM_TABLE, BRONZE_ROOM),
    (BRONZE_TREASURE_TABLE, BRONZE_TREASURE),
    (SILVER_ROOM_TABLE, SILVER_ROOM),
    (SILVER_TREASURE_TABLE, SILVER_TREASURE),
    (GOLD_TREASURE_TABLE, GOLD_TREASURE),
];

fn pool(prizes: &[Prize]) -> Result<PoolBuilder>
{
    prizes.iter().try_fold(PoolBuilder::new(1, 1), |pool, prize|
    {
        pool.entry(prize.m_item, prize.m_weight, prize.m_count[0], prize.m_count[1])
    })
}

// the cape and feather follow the config as it stands when the mod loads; a case whose prize is off falls back to
// the invisibility cape as the Java switch does
fn silverExtras(pool: PoolBuilder) -> Result<PoolBuilder>
{
    let cape: bool = config::enabled(config::VALKYRIE_CAPE);
    let feather: bool = config::enabled(config::GOLDEN_FEATHER);
    let twelve: &str = match (cape, feather)
    {
        (true, _) => "valkyrie_cape",
        (false, true) => "golden_feather",
        (false, false) => "invisibility_cape",
    };
    let thirteen: &str = if feather
    {
        "golden_feather"
    }
    else
    {
        "invisibility_cape"
    };
    pool.entry(twelve, SILVER_CASE, 1, 1)?.entry(thirteen, SILVER_CASE, 1, 1)
}

// every dungeon chest table as a chest loot pool, which recipe viewers list and other mods may add to
pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for (name, prizes) in TABLES
    {
        let mut table: PoolBuilder = pool(prizes)?;
        if name == SILVER_TREASURE_TABLE
        {
            table = silverExtras(table)?;
        }

        ctx.loot().pool(LootTarget::Chest, name, table.m_spec)?;
    }

    Ok(())
}

// one pick of a table, seeded from the generator's random so a seed still builds the same chest
pub(crate) fn pick(random: &Random, table: &str) -> Option<ItemStack>
{
    let id: Identifier = Identifier::parseIn(table, "aether").ok()?;
    rollLoot(&id, random.nextLong()).into_iter().next()
}
