use crate::config;
use b173_api::terrain::Random;

// an item by registry name and how many
pub(crate) type Loot = (&'static str, i32);

fn one(name: &'static str) -> Loot { (name, 1) }

fn some(random: &Random, name: &'static str, spread: i32) -> Loot { (name, random.nextIntBounded(spread) + 1) }

// the first of several pieces that wins its coin flip
fn firstFlip(random: &Random, pieces: &[&'static str]) -> Option<Loot>
{
    pieces.iter().find(|_| random.nextBoolean()).map(|name| one(name))
}

// BronzeDungeon.getNormalLoot
pub(crate) fn bronzeRoom(random: &Random) -> Loot
{
    let found: Option<Loot> = match random.nextIntBounded(15)
    {
        0 => Some(one("zanite_pickaxe")),
        1 => Some(one("zanite_axe")),
        2 => Some(one("zanite_sword")),
        3 => Some(one("zanite_shovel")),
        4 => Some(one("swet_cape")),
        5 => Some(some(random, "ambrosium_shard", 10)),
        6 => Some(some(random, "golden_dart", 5)),
        7 => Some(some(random, "poison_dart", 3)),
        8 => Some(some(random, "enchanted_dart", 3)),
        9 => (random.nextIntBounded(20) == 0).then(|| one("aether_tune")),
        10 => Some(one("skyroot_poison_bucket")),
        11 => (random.nextIntBounded(10) == 0).then(|| one("minecraft:record_cat")),
        12 => (random.nextIntBounded(4) == 0).then(|| one("iron_ring")),
        13 => (random.nextIntBounded(10) == 0).then(|| one("golden_ring")),
        _ => None,
    };
    found.unwrap_or(one("ambrosium_torch"))
}

// BronzeDungeon.getBronzeLoot
pub(crate) fn bronzeTreasure(random: &Random) -> Loot
{
    match random.nextIntBounded(10)
    {
        0 => gummy(random, 7),
        1 => one("phoenix_bow"),
        2 => one("flaming_sword"),
        3 => one("notch_hammer"),
        4 => some(random, "lightning_knife", 20),
        5 => one("valkyrie_lance"),
        6 => one("agility_cape"),
        7 => one("sentry_boots"),
        8 => one("repulsion_shield"),
        _ => one("cloud_staff"),
    }
}

// ItemGummySwet meta 0 blue, 1 golden, the count drawn first
fn gummy(random: &Random, spread: i32) -> Loot
{
    let count: i32 = random.nextIntBounded(spread) + 1;
    (
        if random.nextIntBounded(2) == 0
        {
            "blue_gummy_swet"
        }
        else
        {
            "golden_gummy_swet"
        },
        count,
    )
}

// ComponentSilverDungeon.getNormalLoot, the last three cases fall through like the Java switch
pub(crate) fn silverRoom(random: &Random) -> Loot
{
    let roll: i32 = random.nextIntBounded(16);
    let found: Option<Loot> = match roll
    {
        0 => Some(one("zanite_pickaxe")),
        1 => Some(one("skyroot_bucket")),
        2 => Some(one("golden_dart_shooter")),
        3 => Some(one("blue_moa_egg")),
        4 => Some(one("white_moa_egg")),
        5 => Some(some(random, "ambrosium_shard", 10)),
        6 => Some(some(random, "golden_dart", 5)),
        7 => Some(some(random, "poison_dart", 3)),
        8 => Some(some(random, "enchanted_dart", 3)),
        9 => (random.nextIntBounded(20) == 0).then(|| one("aether_tune")),
        10 => Some(one("skyroot_poison_bucket")),
        11 => (random.nextIntBounded(10) == 0).then(|| one("ascending_dawn")),
        12 => firstFlip(random, &["zanite_boots", "zanite_helmet", "zanite_leggings", "zanite_chestplate"]),
        13..=15 => fallThrough(random, roll),
        _ => None,
    };
    found.unwrap_or_else(|| some(random, "ambrosium_torch", 4))
}

fn fallThrough(random: &Random, from: i32) -> Option<Loot>
{
    const PENDANTS: [(i32, &str); 3] = [(4, "iron_pendant"), (10, "golden_pendant"), (15, "zanite_ring")];
    const FIRST: i32 = 13;
    PENDANTS[(from - FIRST) as usize..]
        .iter()
        .find(|(odds, _)| random.nextIntBounded(*odds) == 0)
        .map(|(_, name)| one(name))
}

// ComponentSilverDungeon.getSilverLoot, the cape and feather cases fall through when their config is off
pub(crate) fn silverTreasure(random: &Random) -> Loot
{
    let found: Option<Loot> = match random.nextIntBounded(14)
    {
        0 => Some(gummy(random, 15)),
        1 => Some(one("lightning_sword")),
        2 => firstFlip(random, &["valkyrie_axe", "valkyrie_shovel", "valkyrie_pickaxe"]),
        3 => Some(one("holy_sword")),
        4 => Some(one("valkyrie_helmet")),
        5 => Some(one("regeneration_stone")),
        6 => firstFlip(random, &["neptune_helmet", "neptune_leggings", "neptune_chestplate"]),
        7 => Some(one(
            if random.nextBoolean()
            {
                "neptune_boots"
            }
            else
            {
                "neptune_gloves"
            },
        )),
        8 | 11 => Some(one(
            if random.nextBoolean()
            {
                "valkyrie_boots"
            }
            else
            {
                "valkyrie_gloves"
            },
        )),
        9 => Some(one("valkyrie_leggings")),
        10 => Some(firstFlip(random, &["valkyrie_chestplate"]).unwrap_or_else(||
        {
            one(
                if random.nextBoolean()
                {
                    "valkyrie_boots"
                }
                else
                {
                    "valkyrie_gloves"
                },
            )
        })),
        12 if config::enabled(config::VALKYRIE_CAPE) => Some(one("valkyrie_cape")),
        12 | 13 if config::enabled(config::GOLDEN_FEATHER) => Some(one("golden_feather")),
        _ => None,
    };
    found.unwrap_or(one("invisibility_cape"))
}

// ComponentGoldenDungeon.getGoldLoot
pub(crate) fn goldTreasure(random: &Random) -> Loot
{
    let found: Option<Loot> = match random.nextIntBounded(10)
    {
        0 => Some(one("iron_bubble")),
        1 => Some(one("vampire_blade")),
        2 => Some(one("pig_slayer")),
        3 => firstFlip(random, &["phoenix_helmet", "phoenix_leggings", "phoenix_chestplate"]),
        4 => Some(one(
            if random.nextBoolean()
            {
                "phoenix_boots"
            }
            else
            {
                "phoenix_gloves"
            },
        )),
        5 => Some(one("life_shard")),
        6 => firstFlip(random, &["gravitite_helmet", "gravitite_leggings", "gravitite_chestplate"]),
        7 => Some(one(
            if random.nextBoolean()
            {
                "gravitite_boots"
            }
            else
            {
                "gravitite_gloves"
            },
        )),
        8 => Some(one("chain_gloves")),
        _ => None,
    };
    found.unwrap_or(one("obsidian_chestplate"))
}
