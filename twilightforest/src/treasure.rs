use crate::world::generate::GenWorld;
use crate::world::palette;
use b173_api::prelude::*;
use b173_api::terrain::Random;

// TFTreasure loot tables, each chest draws four common two uncommon and one rare stack
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Table
{
    Hill1,
    Hill2,
    Hill3,
    HedgeMaze,
    LabyrinthRoom,
    LabyrinthDeadEnd,
    TowerRoom,
    TowerLibrary,
    Basement,
    LabyrinthVault,
    DarkTowerCache,
    DarkTowerKey,
    DarkTowerBoss,
    TreeCache,
    StrongholdCache,
    StrongholdRoom,
    StrongholdBoss,
    AuroraCache,
    AuroraRoom,
    AuroraBoss,
    TrollGarden,
    TrollVault,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Tier
{
    Useless,
    Common,
    Uncommon,
    Rare,
    UltraRare,
}

// one entry of a tier, quantity is the most a stack rolls and rarity its weight
struct Treasure
{
    m_table: Table,
    m_tier: Tier,
    m_item: &'static str,
    m_quantity: i32,
    m_damage: i32,
    m_rarity: i32,
}

const fn row(table: Table, tier: Tier, item: &'static str, quantity: i32, damage: i32, rarity: i32) -> Treasure
{
    Treasure { m_table: table, m_tier: tier, m_item: item, m_quantity: quantity, m_damage: damage, m_rarity: rarity }
}

// TFTreasure.fill with every entry b1.7.3 has, potions enchanted books and later items left out
const TREASURES: [Treasure; 405] = [
    row(Table::Hill1, Tier::Useless, "minecraft:red_flower", 4, 0, 10),
    row(Table::Hill1, Tier::Useless, "minecraft:yellow_flower", 4, 0, 10),
    row(Table::Hill1, Tier::Useless, "minecraft:feather", 3, 0, 10),
    row(Table::Hill1, Tier::Useless, "minecraft:wheat_seeds", 2, 0, 10),
    row(Table::Hill1, Tier::Useless, "minecraft:flint", 2, 0, 10),
    row(Table::Hill1, Tier::Useless, "minecraft:cactus", 2, 0, 10),
    row(Table::Hill1, Tier::Useless, "minecraft:reeds", 4, 0, 10),
    row(Table::Hill1, Tier::Useless, "minecraft:sand", 4, 0, 10),
    row(Table::Hill1, Tier::Useless, "minecraft:dye", 1, 0, 10),
    row(Table::Hill1, Tier::Common, "minecraft:iron_ingot", 4, 0, 10),
    row(Table::Hill1, Tier::Common, "minecraft:wheat", 4, 0, 10),
    row(Table::Hill1, Tier::Common, "minecraft:string", 4, 0, 10),
    row(Table::Hill1, Tier::Common, "minecraft:bucket", 1, 0, 10),
    row(Table::Hill1, Tier::Uncommon, "minecraft:bread", 1, 0, 10),
    row(Table::Hill1, Tier::Uncommon, "ore_magnet", 1, 0, 10),
    row(Table::Hill1, Tier::Uncommon, "minecraft:gunpowder", 4, 0, 10),
    row(Table::Hill1, Tier::Uncommon, "minecraft:arrow", 12, 0, 10),
    row(Table::Hill1, Tier::Uncommon, "minecraft:torch", 12, 0, 10),
    row(Table::Hill1, Tier::Rare, "minecraft:gold_ingot", 3, 0, 10),
    row(Table::Hill1, Tier::Rare, "minecraft:iron_pickaxe", 1, 0, 10),
    row(Table::Hill1, Tier::Rare, "liveroot", 3, 0, 10),
    row(Table::Hill1, Tier::UltraRare, "transform_powder", 12, 0, 10),
    row(Table::Hill1, Tier::UltraRare, "minecraft:diamond", 1, 0, 10),
    row(Table::Hill1, Tier::UltraRare, "steeleaf_ingot", 3, 0, 10),
    row(Table::Hill2, Tier::Useless, "minecraft:red_flower", 4, 0, 10),
    row(Table::Hill2, Tier::Useless, "minecraft:yellow_flower", 4, 0, 10),
    row(Table::Hill2, Tier::Useless, "minecraft:feather", 3, 0, 10),
    row(Table::Hill2, Tier::Useless, "minecraft:wheat_seeds", 2, 0, 10),
    row(Table::Hill2, Tier::Useless, "minecraft:flint", 2, 0, 10),
    row(Table::Hill2, Tier::Useless, "minecraft:cactus", 2, 0, 10),
    row(Table::Hill2, Tier::Useless, "minecraft:reeds", 4, 0, 10),
    row(Table::Hill2, Tier::Useless, "minecraft:sand", 4, 0, 10),
    row(Table::Hill2, Tier::Useless, "minecraft:dye", 1, 0, 10),
    row(Table::Hill2, Tier::Common, "minecraft:iron_ingot", 4, 0, 10),
    row(Table::Hill2, Tier::Common, "minecraft:ladder", 6, 0, 10),
    row(Table::Hill2, Tier::Common, "minecraft:bucket", 1, 0, 10),
    row(Table::Hill2, Tier::Uncommon, "ore_magnet", 1, 0, 10),
    row(Table::Hill2, Tier::Uncommon, "ironwood_ingot", 4, 0, 10),
    row(Table::Hill2, Tier::Uncommon, "minecraft:arrow", 12, 0, 10),
    row(Table::Hill2, Tier::Uncommon, "minecraft:torch", 12, 0, 10),
    row(Table::Hill2, Tier::Rare, "naga_scale", 1, 0, 10),
    row(Table::Hill2, Tier::Rare, "uncrafting_table", 1, 0, 10),
    row(Table::Hill2, Tier::Rare, "transform_powder", 12, 0, 10),
    row(Table::Hill2, Tier::UltraRare, "peacock_fan", 1, 0, 10),
    row(Table::Hill2, Tier::UltraRare, "minecraft:diamond", 1, 0, 10),
    row(Table::Hill2, Tier::UltraRare, "charm_of_life_1", 1, 0, 10),
    row(Table::Hill3, Tier::Useless, "minecraft:red_flower", 4, 0, 10),
    row(Table::Hill3, Tier::Useless, "minecraft:yellow_flower", 4, 0, 10),
    row(Table::Hill3, Tier::Useless, "minecraft:feather", 3, 0, 10),
    row(Table::Hill3, Tier::Useless, "minecraft:wheat_seeds", 2, 0, 10),
    row(Table::Hill3, Tier::Useless, "minecraft:flint", 2, 0, 10),
    row(Table::Hill3, Tier::Useless, "minecraft:cactus", 2, 0, 10),
    row(Table::Hill3, Tier::Useless, "minecraft:reeds", 4, 0, 10),
    row(Table::Hill3, Tier::Useless, "minecraft:sand", 4, 0, 10),
    row(Table::Hill3, Tier::Useless, "minecraft:dye", 1, 0, 10),
    row(Table::Hill3, Tier::Common, "minecraft:fish", 4, 0, 10),
    row(Table::Hill3, Tier::Common, "torchberries", 5, 0, 10),
    row(Table::Hill3, Tier::Uncommon, "ore_magnet", 1, 0, 10),
    row(Table::Hill3, Tier::Uncommon, "minecraft:gunpowder", 4, 0, 10),
    row(Table::Hill3, Tier::Uncommon, "minecraft:arrow", 12, 0, 10),
    row(Table::Hill3, Tier::Uncommon, "minecraft:torch", 12, 0, 10),
    row(Table::Hill3, Tier::Uncommon, "steeleaf_ingot", 4, 0, 10),
    row(Table::Hill3, Tier::Rare, "naga_scale", 1, 0, 10),
    row(Table::Hill3, Tier::Rare, "ironwood_pickaxe", 1, 0, 10),
    row(Table::Hill3, Tier::Rare, "transform_powder", 12, 0, 10),
    row(Table::Hill3, Tier::UltraRare, "moonworm_queen", 1, 0, 10),
    row(Table::Hill3, Tier::UltraRare, "twilight_oak_sapling", 1, 0, 4),
    row(Table::Hill3, Tier::UltraRare, "minecraft:diamond", 2, 0, 10),
    row(Table::Hill3, Tier::UltraRare, "charm_of_life_1", 1, 0, 10),
    row(Table::Hill3, Tier::UltraRare, "charm_of_keeping_1", 1, 0, 10),
    row(Table::HedgeMaze, Tier::Useless, "minecraft:red_flower", 4, 0, 10),
    row(Table::HedgeMaze, Tier::Useless, "minecraft:yellow_flower", 4, 0, 10),
    row(Table::HedgeMaze, Tier::Useless, "minecraft:feather", 3, 0, 10),
    row(Table::HedgeMaze, Tier::Useless, "minecraft:wheat_seeds", 2, 0, 10),
    row(Table::HedgeMaze, Tier::Useless, "minecraft:flint", 2, 0, 10),
    row(Table::HedgeMaze, Tier::Useless, "minecraft:cactus", 2, 0, 10),
    row(Table::HedgeMaze, Tier::Useless, "minecraft:reeds", 4, 0, 10),
    row(Table::HedgeMaze, Tier::Useless, "minecraft:sand", 4, 0, 10),
    row(Table::HedgeMaze, Tier::Useless, "minecraft:dye", 1, 0, 10),
    row(Table::HedgeMaze, Tier::Common, "minecraft:planks", 4, 0, 10),
    row(Table::HedgeMaze, Tier::Common, "minecraft:brown_mushroom", 4, 0, 10),
    row(Table::HedgeMaze, Tier::Common, "minecraft:red_mushroom", 4, 0, 10),
    row(Table::HedgeMaze, Tier::Common, "minecraft:wheat", 4, 0, 10),
    row(Table::HedgeMaze, Tier::Common, "minecraft:string", 4, 0, 10),
    row(Table::HedgeMaze, Tier::Common, "minecraft:stick", 6, 0, 10),
    row(Table::HedgeMaze, Tier::Uncommon, "minecraft:arrow", 12, 0, 10),
    row(Table::HedgeMaze, Tier::Uncommon, "firefly", 4, 0, 10),
    row(Table::HedgeMaze, Tier::Rare, "minecraft:web", 3, 0, 10),
    row(Table::HedgeMaze, Tier::Rare, "minecraft:shears", 1, 0, 10),
    row(Table::HedgeMaze, Tier::Rare, "minecraft:saddle", 1, 0, 10),
    row(Table::HedgeMaze, Tier::Rare, "minecraft:bow", 1, 0, 10),
    row(Table::HedgeMaze, Tier::Rare, "minecraft:apple", 2, 0, 10),
    row(Table::HedgeMaze, Tier::UltraRare, "minecraft:diamond_hoe", 1, 0, 10),
    row(Table::HedgeMaze, Tier::UltraRare, "minecraft:diamond", 1, 0, 10),
    row(Table::HedgeMaze, Tier::UltraRare, "minecraft:mushroom_stew", 1, 0, 10),
    row(Table::HedgeMaze, Tier::UltraRare, "minecraft:golden_apple", 1, 0, 10),
    row(Table::LabyrinthRoom, Tier::Common, "minecraft:iron_ingot", 4, 0, 10),
    row(Table::LabyrinthRoom, Tier::Common, "maze_wafer", 12, 0, 10),
    row(Table::LabyrinthRoom, Tier::Common, "minecraft:gunpowder", 4, 0, 10),
    row(Table::LabyrinthRoom, Tier::Common, "ironwood_ingot", 4, 0, 10),
    row(Table::LabyrinthRoom, Tier::Common, "firefly", 5, 0, 10),
    row(Table::LabyrinthRoom, Tier::Common, "minecraft:milk_bucket", 1, 0, 10),
    row(Table::LabyrinthRoom, Tier::Uncommon, "steeleaf_ingot", 6, 0, 10),
    row(Table::LabyrinthRoom, Tier::Uncommon, "steeleaf_leggings", 1, 0, 10),
    row(Table::LabyrinthRoom, Tier::Uncommon, "steeleaf_chestplate", 1, 0, 10),
    row(Table::LabyrinthRoom, Tier::Uncommon, "steeleaf_helmet", 1, 0, 10),
    row(Table::LabyrinthRoom, Tier::Uncommon, "steeleaf_boots", 1, 0, 10),
    row(Table::LabyrinthRoom, Tier::Uncommon, "steeleaf_pickaxe", 1, 0, 10),
    row(Table::LabyrinthRoom, Tier::Uncommon, "ironwood_chestplate", 1, 0, 10),
    row(Table::LabyrinthRoom, Tier::Uncommon, "ironwood_sword", 1, 0, 10),
    row(Table::LabyrinthRoom, Tier::Uncommon, "charm_of_keeping_1", 1, 0, 10),
    row(Table::LabyrinthRoom, Tier::Rare, "maze_map_focus", 1, 0, 10),
    row(Table::LabyrinthRoom, Tier::Rare, "minecraft:tnt", 3, 0, 10),
    row(Table::LabyrinthDeadEnd, Tier::Useless, "minecraft:red_flower", 4, 0, 10),
    row(Table::LabyrinthDeadEnd, Tier::Useless, "minecraft:yellow_flower", 4, 0, 10),
    row(Table::LabyrinthDeadEnd, Tier::Useless, "minecraft:feather", 3, 0, 10),
    row(Table::LabyrinthDeadEnd, Tier::Useless, "minecraft:wheat_seeds", 2, 0, 10),
    row(Table::LabyrinthDeadEnd, Tier::Useless, "minecraft:flint", 2, 0, 10),
    row(Table::LabyrinthDeadEnd, Tier::Useless, "minecraft:cactus", 2, 0, 10),
    row(Table::LabyrinthDeadEnd, Tier::Useless, "minecraft:reeds", 4, 0, 10),
    row(Table::LabyrinthDeadEnd, Tier::Useless, "minecraft:sand", 4, 0, 10),
    row(Table::LabyrinthDeadEnd, Tier::Useless, "minecraft:dye", 1, 0, 10),
    row(Table::LabyrinthDeadEnd, Tier::Common, "minecraft:stick", 12, 0, 10),
    row(Table::LabyrinthDeadEnd, Tier::Common, "minecraft:coal", 12, 0, 10),
    row(Table::LabyrinthDeadEnd, Tier::Common, "minecraft:arrow", 12, 0, 10),
    row(Table::LabyrinthDeadEnd, Tier::Common, "maze_wafer", 9, 0, 10),
    row(Table::LabyrinthDeadEnd, Tier::Common, "minecraft:paper", 12, 0, 10),
    row(Table::LabyrinthDeadEnd, Tier::Common, "minecraft:leather", 4, 0, 10),
    row(Table::LabyrinthDeadEnd, Tier::Common, "minecraft:mushroom_stew", 1, 0, 10),
    row(Table::LabyrinthDeadEnd, Tier::Uncommon, "minecraft:milk_bucket", 1, 0, 10),
    row(Table::LabyrinthDeadEnd, Tier::Uncommon, "minecraft:paper", 5, 0, 10),
    row(Table::LabyrinthDeadEnd, Tier::Uncommon, "minecraft:iron_ingot", 6, 0, 10),
    row(Table::LabyrinthDeadEnd, Tier::Uncommon, "ironwood_ingot", 8, 0, 10),
    row(Table::LabyrinthDeadEnd, Tier::Uncommon, "firefly", 5, 0, 10),
    row(Table::LabyrinthDeadEnd, Tier::Uncommon, "charm_of_keeping_1", 1, 0, 10),
    row(Table::LabyrinthDeadEnd, Tier::Rare, "steeleaf_ingot", 8, 0, 10),
    row(Table::LabyrinthDeadEnd, Tier::Rare, "minecraft:golden_apple", 1, 0, 10),
    row(Table::TowerRoom, Tier::Useless, "minecraft:red_flower", 4, 0, 10),
    row(Table::TowerRoom, Tier::Useless, "minecraft:yellow_flower", 4, 0, 10),
    row(Table::TowerRoom, Tier::Useless, "minecraft:feather", 3, 0, 10),
    row(Table::TowerRoom, Tier::Useless, "minecraft:wheat_seeds", 2, 0, 10),
    row(Table::TowerRoom, Tier::Useless, "minecraft:flint", 2, 0, 10),
    row(Table::TowerRoom, Tier::Useless, "minecraft:cactus", 2, 0, 10),
    row(Table::TowerRoom, Tier::Useless, "minecraft:reeds", 4, 0, 10),
    row(Table::TowerRoom, Tier::Useless, "minecraft:sand", 4, 0, 10),
    row(Table::TowerRoom, Tier::Useless, "minecraft:dye", 1, 0, 10),
    row(Table::TowerRoom, Tier::Common, "minecraft:sugar", 5, 0, 10),
    row(Table::TowerRoom, Tier::Common, "minecraft:paper", 6, 0, 10),
    row(Table::TowerRoom, Tier::Uncommon, "minecraft:golden_sword", 1, 0, 10),
    row(Table::TowerRoom, Tier::Uncommon, "minecraft:golden_boots", 1, 0, 10),
    row(Table::TowerRoom, Tier::Rare, "minecraft:golden_helmet", 1, 0, 10),
    row(Table::TowerRoom, Tier::Rare, "transform_powder", 12, 0, 10),
    row(Table::TowerRoom, Tier::Rare, "charm_of_life_1", 1, 0, 10),
    row(Table::TowerRoom, Tier::Rare, "charm_of_keeping_1", 1, 0, 10),
    row(Table::TowerRoom, Tier::UltraRare, "minecraft:golden_axe", 1, 0, 10),
    row(Table::TowerRoom, Tier::UltraRare, "minecraft:obsidian", 4, 0, 10),
    row(Table::TowerRoom, Tier::UltraRare, "minecraft:diamond", 1, 0, 10),
    row(Table::TowerRoom, Tier::UltraRare, "moonworm_queen", 1, 0, 10),
    row(Table::TowerRoom, Tier::UltraRare, "peacock_fan", 1, 0, 10),
    row(Table::TowerLibrary, Tier::Useless, "minecraft:red_flower", 4, 0, 10),
    row(Table::TowerLibrary, Tier::Useless, "minecraft:yellow_flower", 4, 0, 10),
    row(Table::TowerLibrary, Tier::Useless, "minecraft:feather", 3, 0, 10),
    row(Table::TowerLibrary, Tier::Useless, "minecraft:wheat_seeds", 2, 0, 10),
    row(Table::TowerLibrary, Tier::Useless, "minecraft:flint", 2, 0, 10),
    row(Table::TowerLibrary, Tier::Useless, "minecraft:cactus", 2, 0, 10),
    row(Table::TowerLibrary, Tier::Useless, "minecraft:reeds", 4, 0, 10),
    row(Table::TowerLibrary, Tier::Useless, "minecraft:sand", 4, 0, 10),
    row(Table::TowerLibrary, Tier::Useless, "minecraft:dye", 1, 0, 10),
    row(Table::TowerLibrary, Tier::Common, "minecraft:ladder", 6, 0, 10),
    row(Table::TowerLibrary, Tier::Common, "minecraft:paper", 6, 0, 10),
    row(Table::TowerLibrary, Tier::Common, "minecraft:bone", 6, 0, 10),
    row(Table::TowerLibrary, Tier::Common, "minecraft:clay_ball", 12, 0, 10),
    row(Table::TowerLibrary, Tier::Uncommon, "minecraft:iron_leggings", 1, 0, 10),
    row(Table::TowerLibrary, Tier::Uncommon, "minecraft:book", 5, 0, 10),
    row(Table::TowerLibrary, Tier::Uncommon, "minecraft:filled_map", 1, 0, 10),
    row(Table::TowerLibrary, Tier::Rare, "minecraft:bow", 1, 0, 10),
    row(Table::TowerLibrary, Tier::Rare, "minecraft:stone_sword", 1, 0, 10),
    row(Table::TowerLibrary, Tier::Rare, "minecraft:wooden_sword", 1, 0, 10),
    row(Table::TowerLibrary, Tier::Rare, "transform_powder", 12, 0, 10),
    row(Table::TowerLibrary, Tier::Rare, "charm_of_keeping_1", 1, 0, 10),
    row(Table::TowerLibrary, Tier::UltraRare, "minecraft:golden_pickaxe", 1, 0, 10),
    row(Table::TowerLibrary, Tier::UltraRare, "minecraft:iron_sword", 1, 0, 10),
    row(Table::TowerLibrary, Tier::UltraRare, "minecraft:bow", 1, 0, 10),
    row(Table::TowerLibrary, Tier::UltraRare, "minecraft:bookshelf", 5, 0, 10),
    row(Table::Basement, Tier::Useless, "minecraft:red_flower", 4, 0, 10),
    row(Table::Basement, Tier::Useless, "minecraft:yellow_flower", 4, 0, 10),
    row(Table::Basement, Tier::Useless, "minecraft:feather", 3, 0, 10),
    row(Table::Basement, Tier::Useless, "minecraft:wheat_seeds", 2, 0, 10),
    row(Table::Basement, Tier::Useless, "minecraft:flint", 2, 0, 10),
    row(Table::Basement, Tier::Useless, "minecraft:cactus", 2, 0, 10),
    row(Table::Basement, Tier::Useless, "minecraft:reeds", 4, 0, 10),
    row(Table::Basement, Tier::Useless, "minecraft:sand", 4, 0, 10),
    row(Table::Basement, Tier::Useless, "minecraft:dye", 1, 0, 10),
    row(Table::Basement, Tier::Common, "minecraft:wheat", 6, 0, 10),
    row(Table::Basement, Tier::Common, "minecraft:water_bucket", 1, 0, 10),
    row(Table::Basement, Tier::Common, "minecraft:torch", 12, 0, 10),
    row(Table::Basement, Tier::Common, "minecraft:mushroom_stew", 1, 0, 10),
    row(Table::Basement, Tier::Common, "minecraft:milk_bucket", 1, 0, 10),
    row(Table::Basement, Tier::Uncommon, "minecraft:bread", 8, 0, 10),
    row(Table::Basement, Tier::Uncommon, "minecraft:cooked_porkchop", 8, 0, 10),
    row(Table::Basement, Tier::Uncommon, "minecraft:cooked_fish", 8, 0, 10),
    row(Table::Basement, Tier::Rare, "minecraft:apple", 12, 0, 10),
    row(Table::Basement, Tier::Rare, "minecraft:filled_map", 1, 0, 10),
    row(Table::Basement, Tier::Rare, "charm_of_keeping_1", 1, 0, 10),
    row(Table::Basement, Tier::UltraRare, "minecraft:golden_apple", 2, 0, 10),
    row(Table::Basement, Tier::UltraRare, "minecraft:cake", 1, 0, 10),
    row(Table::Basement, Tier::UltraRare, "minecraft:boat", 1, 0, 10),
    row(Table::Basement, Tier::UltraRare, "hollow_oak_sapling", 1, 0, 10),
    row(Table::LabyrinthVault, Tier::Common, "minecraft:iron_ingot", 9, 0, 10),
    row(Table::LabyrinthVault, Tier::Common, "maze_wafer", 12, 0, 10),
    row(Table::LabyrinthVault, Tier::Common, "ironwood_ingot", 9, 0, 10),
    row(Table::LabyrinthVault, Tier::Uncommon, "minecraft:bow", 1, 0, 10),
    row(Table::LabyrinthVault, Tier::Uncommon, "minecraft:bow", 1, 0, 10),
    row(Table::LabyrinthVault, Tier::Uncommon, "steeleaf_shovel", 1, 0, 10),
    row(Table::LabyrinthVault, Tier::Uncommon, "steeleaf_axe", 1, 0, 10),
    row(Table::LabyrinthVault, Tier::Uncommon, "steeleaf_ingot", 12, 0, 10),
    row(Table::LabyrinthVault, Tier::Uncommon, "steeleaf_chestplate", 1, 0, 10),
    row(Table::LabyrinthVault, Tier::Uncommon, "steeleaf_leggings", 1, 0, 10),
    row(Table::LabyrinthVault, Tier::Uncommon, "steeleaf_boots", 1, 0, 10),
    row(Table::LabyrinthVault, Tier::Uncommon, "steeleaf_helmet", 1, 0, 10),
    row(Table::LabyrinthVault, Tier::Rare, "steeleaf_pickaxe", 1, 0, 10),
    row(Table::LabyrinthVault, Tier::Rare, "steeleaf_sword", 1, 0, 10),
    row(Table::LabyrinthVault, Tier::Rare, "steeleaf_sword", 1, 0, 10),
    row(Table::DarkTowerCache, Tier::Useless, "minecraft:red_flower", 4, 0, 10),
    row(Table::DarkTowerCache, Tier::Useless, "minecraft:yellow_flower", 4, 0, 10),
    row(Table::DarkTowerCache, Tier::Useless, "minecraft:feather", 3, 0, 10),
    row(Table::DarkTowerCache, Tier::Useless, "minecraft:wheat_seeds", 2, 0, 10),
    row(Table::DarkTowerCache, Tier::Useless, "minecraft:flint", 2, 0, 10),
    row(Table::DarkTowerCache, Tier::Useless, "minecraft:cactus", 2, 0, 10),
    row(Table::DarkTowerCache, Tier::Useless, "minecraft:reeds", 4, 0, 10),
    row(Table::DarkTowerCache, Tier::Useless, "minecraft:sand", 4, 0, 10),
    row(Table::DarkTowerCache, Tier::Useless, "minecraft:dye", 1, 0, 10),
    row(Table::DarkTowerCache, Tier::Common, "minecraft:stick", 12, 0, 10),
    row(Table::DarkTowerCache, Tier::Common, "minecraft:coal", 12, 1, 10),
    row(Table::DarkTowerCache, Tier::Common, "minecraft:arrow", 12, 0, 10),
    row(Table::DarkTowerCache, Tier::Common, "experiment_115", 9, 0, 10),
    row(Table::DarkTowerCache, Tier::Common, "minecraft:wool", 1, 14, 10),
    row(Table::DarkTowerCache, Tier::Common, "minecraft:redstone", 6, 0, 10),
    row(Table::DarkTowerCache, Tier::Uncommon, "minecraft:iron_ingot", 6, 0, 10),
    row(Table::DarkTowerCache, Tier::Uncommon, "ironwood_ingot", 8, 0, 10),
    row(Table::DarkTowerCache, Tier::Uncommon, "firefly", 5, 0, 10),
    row(Table::DarkTowerCache, Tier::Uncommon, "charm_of_keeping_1", 1, 0, 10),
    row(Table::DarkTowerCache, Tier::Rare, "steeleaf_ingot", 8, 0, 10),
    row(Table::DarkTowerCache, Tier::Rare, "minecraft:diamond", 2, 0, 10),
    row(Table::DarkTowerKey, Tier::Common, "minecraft:iron_ingot", 4, 0, 10),
    row(Table::DarkTowerKey, Tier::Common, "experiment_115", 12, 0, 10),
    row(Table::DarkTowerKey, Tier::Common, "minecraft:gunpowder", 4, 0, 10),
    row(Table::DarkTowerKey, Tier::Common, "ironwood_ingot", 4, 0, 10),
    row(Table::DarkTowerKey, Tier::Common, "firefly", 5, 0, 10),
    row(Table::DarkTowerKey, Tier::Common, "minecraft:redstone", 12, 0, 10),
    row(Table::DarkTowerKey, Tier::Common, "minecraft:glowstone_dust", 12, 0, 10),
    row(Table::DarkTowerKey, Tier::Uncommon, "steeleaf_ingot", 6, 0, 10),
    row(Table::DarkTowerKey, Tier::Uncommon, "steeleaf_leggings", 1, 0, 10),
    row(Table::DarkTowerKey, Tier::Uncommon, "steeleaf_chestplate", 1, 0, 10),
    row(Table::DarkTowerKey, Tier::Uncommon, "steeleaf_helmet", 1, 0, 10),
    row(Table::DarkTowerKey, Tier::Uncommon, "steeleaf_boots", 1, 0, 10),
    row(Table::DarkTowerKey, Tier::Uncommon, "steeleaf_pickaxe", 1, 0, 10),
    row(Table::DarkTowerKey, Tier::Uncommon, "ironwood_chestplate", 1, 0, 10),
    row(Table::DarkTowerKey, Tier::Uncommon, "ironwood_sword", 1, 0, 10),
    row(Table::DarkTowerKey, Tier::Uncommon, "charm_of_keeping_1", 1, 0, 10),
    row(Table::DarkTowerKey, Tier::Rare, "charm_of_life_1", 1, 0, 10),
    row(Table::DarkTowerBoss, Tier::Common, "carminite", 3, 0, 10),
    row(Table::DarkTowerBoss, Tier::Uncommon, "fiery_tears", 5, 0, 10),
    row(Table::DarkTowerBoss, Tier::Rare, "ur_ghast_trophy", 1, 0, 10),
    row(Table::TreeCache, Tier::Useless, "minecraft:red_flower", 4, 0, 10),
    row(Table::TreeCache, Tier::Useless, "minecraft:yellow_flower", 4, 0, 10),
    row(Table::TreeCache, Tier::Useless, "minecraft:feather", 3, 0, 10),
    row(Table::TreeCache, Tier::Useless, "minecraft:wheat_seeds", 2, 0, 10),
    row(Table::TreeCache, Tier::Useless, "minecraft:flint", 2, 0, 10),
    row(Table::TreeCache, Tier::Useless, "minecraft:cactus", 2, 0, 10),
    row(Table::TreeCache, Tier::Useless, "minecraft:reeds", 4, 0, 10),
    row(Table::TreeCache, Tier::Useless, "minecraft:sand", 4, 0, 10),
    row(Table::TreeCache, Tier::Useless, "minecraft:dye", 1, 0, 10),
    row(Table::TreeCache, Tier::Common, "minecraft:wheat", 6, 0, 10),
    row(Table::TreeCache, Tier::Common, "minecraft:water_bucket", 1, 0, 10),
    row(Table::TreeCache, Tier::Common, "minecraft:milk_bucket", 1, 0, 10),
    row(Table::TreeCache, Tier::Uncommon, "firefly", 12, 0, 10),
    row(Table::TreeCache, Tier::Uncommon, "twilight_oak_sapling", 4, 0, 10),
    row(Table::TreeCache, Tier::Uncommon, "canopy_sapling", 4, 0, 10),
    row(Table::TreeCache, Tier::Uncommon, "mangrove_sapling", 4, 0, 10),
    row(Table::TreeCache, Tier::Uncommon, "darkwood_sapling", 4, 0, 10),
    row(Table::TreeCache, Tier::Rare, "minecraft:apple", 12, 0, 10),
    row(Table::TreeCache, Tier::Rare, "charm_of_life_1", 1, 0, 10),
    row(Table::TreeCache, Tier::Rare, "charm_of_keeping_1", 1, 0, 10),
    row(Table::TreeCache, Tier::UltraRare, "hollow_oak_sapling", 1, 0, 10),
    row(Table::TreeCache, Tier::UltraRare, "time_sapling", 1, 0, 10),
    row(Table::TreeCache, Tier::UltraRare, "transformation_sapling", 1, 0, 10),
    row(Table::TreeCache, Tier::UltraRare, "mining_sapling", 1, 0, 10),
    row(Table::TreeCache, Tier::UltraRare, "sorting_sapling", 1, 0, 10),
    row(Table::StrongholdCache, Tier::Useless, "minecraft:red_flower", 4, 0, 10),
    row(Table::StrongholdCache, Tier::Useless, "minecraft:yellow_flower", 4, 0, 10),
    row(Table::StrongholdCache, Tier::Useless, "minecraft:feather", 3, 0, 10),
    row(Table::StrongholdCache, Tier::Useless, "minecraft:wheat_seeds", 2, 0, 10),
    row(Table::StrongholdCache, Tier::Useless, "minecraft:flint", 2, 0, 10),
    row(Table::StrongholdCache, Tier::Useless, "minecraft:cactus", 2, 0, 10),
    row(Table::StrongholdCache, Tier::Useless, "minecraft:reeds", 4, 0, 10),
    row(Table::StrongholdCache, Tier::Useless, "minecraft:sand", 4, 0, 10),
    row(Table::StrongholdCache, Tier::Useless, "minecraft:dye", 1, 0, 10),
    row(Table::StrongholdCache, Tier::Common, "minecraft:stick", 12, 0, 10),
    row(Table::StrongholdCache, Tier::Common, "minecraft:coal", 12, 0, 10),
    row(Table::StrongholdCache, Tier::Common, "minecraft:arrow", 12, 0, 10),
    row(Table::StrongholdCache, Tier::Common, "maze_wafer", 9, 0, 10),
    row(Table::StrongholdCache, Tier::Common, "minecraft:wool", 1, 11, 10),
    row(Table::StrongholdCache, Tier::Common, "minecraft:iron_ingot", 2, 0, 10),
    row(Table::StrongholdCache, Tier::Uncommon, "minecraft:bucket", 1, 0, 10),
    row(Table::StrongholdCache, Tier::Uncommon, "minecraft:iron_ingot", 6, 0, 10),
    row(Table::StrongholdCache, Tier::Uncommon, "ironwood_ingot", 6, 0, 10),
    row(Table::StrongholdCache, Tier::Uncommon, "firefly", 5, 0, 10),
    row(Table::StrongholdCache, Tier::Uncommon, "charm_of_keeping_1", 1, 0, 10),
    row(Table::StrongholdCache, Tier::Uncommon, "armor_shard", 3, 0, 10),
    row(Table::StrongholdCache, Tier::Rare, "knightmetal_ingot", 8, 0, 10),
    row(Table::StrongholdCache, Tier::Rare, "minecraft:bow", 1, 0, 10),
    row(Table::StrongholdCache, Tier::Rare, "minecraft:iron_sword", 1, 0, 10),
    row(Table::StrongholdCache, Tier::Rare, "ironwood_sword", 1, 0, 10),
    row(Table::StrongholdCache, Tier::Rare, "steeleaf_sword", 1, 0, 10),
    row(Table::StrongholdRoom, Tier::Common, "minecraft:iron_ingot", 4, 0, 10),
    row(Table::StrongholdRoom, Tier::Common, "maze_wafer", 12, 0, 10),
    row(Table::StrongholdRoom, Tier::Common, "minecraft:gunpowder", 4, 0, 10),
    row(Table::StrongholdRoom, Tier::Common, "ironwood_ingot", 4, 0, 10),
    row(Table::StrongholdRoom, Tier::Common, "firefly", 5, 0, 10),
    row(Table::StrongholdRoom, Tier::Common, "minecraft:milk_bucket", 1, 0, 10),
    row(Table::StrongholdRoom, Tier::Uncommon, "steeleaf_ingot", 6, 0, 10),
    row(Table::StrongholdRoom, Tier::Uncommon, "steeleaf_leggings", 1, 0, 10),
    row(Table::StrongholdRoom, Tier::Uncommon, "steeleaf_chestplate", 1, 0, 10),
    row(Table::StrongholdRoom, Tier::Uncommon, "steeleaf_helmet", 1, 0, 10),
    row(Table::StrongholdRoom, Tier::Uncommon, "steeleaf_boots", 1, 0, 10),
    row(Table::StrongholdRoom, Tier::Uncommon, "steeleaf_pickaxe", 1, 0, 10),
    row(Table::StrongholdRoom, Tier::Uncommon, "ironwood_chestplate", 1, 0, 10),
    row(Table::StrongholdRoom, Tier::Uncommon, "ironwood_sword", 1, 0, 10),
    row(Table::StrongholdRoom, Tier::Uncommon, "charm_of_life_1", 1, 0, 10),
    row(Table::StrongholdRoom, Tier::Rare, "maze_map_focus", 1, 0, 10),
    row(Table::StrongholdRoom, Tier::Rare, "minecraft:bow", 1, 0, 10),
    row(Table::StrongholdRoom, Tier::Rare, "minecraft:iron_sword", 1, 0, 10),
    row(Table::StrongholdRoom, Tier::Rare, "ironwood_sword", 1, 0, 10),
    row(Table::StrongholdRoom, Tier::Rare, "steeleaf_sword", 1, 0, 10),
    row(Table::StrongholdRoom, Tier::Rare, "minecraft:diamond_sword", 1, 0, 10),
    row(Table::StrongholdBoss, Tier::Common, "knightly_sword", 1, 0, 10),
    row(Table::StrongholdBoss, Tier::Common, "knightly_pickaxe", 1, 0, 10),
    row(Table::StrongholdBoss, Tier::Common, "knightly_axe", 1, 0, 10),
    row(Table::StrongholdBoss, Tier::Uncommon, "phantom_helmet", 1, 0, 10),
    row(Table::StrongholdBoss, Tier::Uncommon, "phantom_chestplate", 1, 0, 10),
    row(Table::StrongholdBoss, Tier::Rare, "phantom_helmet", 1, 0, 10),
    row(Table::StrongholdBoss, Tier::Rare, "phantom_chestplate", 1, 0, 10),
    row(Table::AuroraCache, Tier::Useless, "minecraft:red_flower", 4, 0, 10),
    row(Table::AuroraCache, Tier::Useless, "minecraft:yellow_flower", 4, 0, 10),
    row(Table::AuroraCache, Tier::Useless, "minecraft:feather", 3, 0, 10),
    row(Table::AuroraCache, Tier::Useless, "minecraft:wheat_seeds", 2, 0, 10),
    row(Table::AuroraCache, Tier::Useless, "minecraft:flint", 2, 0, 10),
    row(Table::AuroraCache, Tier::Useless, "minecraft:cactus", 2, 0, 10),
    row(Table::AuroraCache, Tier::Useless, "minecraft:reeds", 4, 0, 10),
    row(Table::AuroraCache, Tier::Useless, "minecraft:sand", 4, 0, 10),
    row(Table::AuroraCache, Tier::Useless, "minecraft:dye", 1, 0, 10),
    row(Table::AuroraCache, Tier::Common, "minecraft:stick", 12, 0, 10),
    row(Table::AuroraCache, Tier::Common, "minecraft:coal", 12, 0, 10),
    row(Table::AuroraCache, Tier::Common, "minecraft:arrow", 12, 0, 10),
    row(Table::AuroraCache, Tier::Common, "maze_wafer", 9, 0, 10),
    row(Table::AuroraCache, Tier::Common, "minecraft:ice", 4, 0, 10),
    row(Table::AuroraCache, Tier::Common, "minecraft:ice", 4, 0, 10),
    row(Table::AuroraCache, Tier::Common, "ironwood_ingot", 2, 0, 10),
    row(Table::AuroraCache, Tier::Uncommon, "aurora_brick", 12, 0, 10),
    row(Table::AuroraCache, Tier::Uncommon, "ironwood_ingot", 6, 0, 10),
    row(Table::AuroraCache, Tier::Uncommon, "firefly", 5, 0, 10),
    row(Table::AuroraCache, Tier::Uncommon, "charm_of_keeping_1", 1, 0, 10),
    row(Table::AuroraCache, Tier::Uncommon, "arctic_fur", 3, 0, 10),
    row(Table::AuroraCache, Tier::Rare, "arctic_fur", 8, 0, 10),
    row(Table::AuroraCache, Tier::Rare, "ice_bow", 1, 0, 10),
    row(Table::AuroraCache, Tier::Rare, "ender_bow", 1, 0, 10),
    row(Table::AuroraCache, Tier::Rare, "ice_sword", 1, 0, 10),
    row(Table::AuroraRoom, Tier::Common, "minecraft:ice", 4, 0, 10),
    row(Table::AuroraRoom, Tier::Common, "minecraft:ice", 4, 0, 10),
    row(Table::AuroraRoom, Tier::Common, "maze_wafer", 12, 0, 10),
    row(Table::AuroraRoom, Tier::Common, "ice_bomb", 4, 0, 10),
    row(Table::AuroraRoom, Tier::Common, "firefly", 5, 0, 10),
    row(Table::AuroraRoom, Tier::Uncommon, "arctic_fur", 6, 0, 10),
    row(Table::AuroraRoom, Tier::Uncommon, "arctic_leggings", 1, 0, 10),
    row(Table::AuroraRoom, Tier::Uncommon, "arctic_chestplate", 1, 0, 10),
    row(Table::AuroraRoom, Tier::Uncommon, "arctic_helmet", 1, 0, 10),
    row(Table::AuroraRoom, Tier::Uncommon, "arctic_boots", 1, 0, 10),
    row(Table::AuroraRoom, Tier::Uncommon, "knightly_chestplate", 1, 0, 10),
    row(Table::AuroraRoom, Tier::Uncommon, "knightly_sword", 1, 0, 10),
    row(Table::AuroraRoom, Tier::Uncommon, "charm_of_life_1", 1, 0, 10),
    row(Table::AuroraRoom, Tier::Rare, "ice_bow", 1, 0, 10),
    row(Table::AuroraRoom, Tier::Rare, "ender_bow", 1, 0, 10),
    row(Table::AuroraRoom, Tier::Rare, "ice_sword", 1, 0, 10),
    row(Table::AuroraRoom, Tier::Rare, "glass_sword", 1, 0, 10),
    row(Table::AuroraBoss, Tier::Useless, "minecraft:red_flower", 4, 0, 10),
    row(Table::AuroraBoss, Tier::Useless, "minecraft:yellow_flower", 4, 0, 10),
    row(Table::AuroraBoss, Tier::Useless, "minecraft:feather", 3, 0, 10),
    row(Table::AuroraBoss, Tier::Useless, "minecraft:wheat_seeds", 2, 0, 10),
    row(Table::AuroraBoss, Tier::Useless, "minecraft:flint", 2, 0, 10),
    row(Table::AuroraBoss, Tier::Useless, "minecraft:cactus", 2, 0, 10),
    row(Table::AuroraBoss, Tier::Useless, "minecraft:reeds", 4, 0, 10),
    row(Table::AuroraBoss, Tier::Useless, "minecraft:sand", 4, 0, 10),
    row(Table::AuroraBoss, Tier::Useless, "minecraft:dye", 1, 0, 10),
    row(Table::TrollGarden, Tier::Common, "minecraft:red_mushroom", 4, 0, 10),
    row(Table::TrollGarden, Tier::Common, "minecraft:brown_mushroom", 4, 0, 10),
    row(Table::TrollGarden, Tier::Common, "minecraft:wheat_seeds", 6, 0, 10),
    row(Table::TrollGarden, Tier::Common, "minecraft:dye", 12, 15, 10),
    row(Table::TrollGarden, Tier::Uncommon, "uberous_soil", 6, 0, 10),
    row(Table::TrollGarden, Tier::Rare, "magic_beans", 1, 0, 10),
    row(Table::TrollVault, Tier::Common, "minecraft:coal", 32, 0, 10),
    row(Table::TrollVault, Tier::Common, "torchberries", 16, 0, 10),
    row(Table::TrollVault, Tier::Uncommon, "trollsteinn", 6, 0, 10),
    row(Table::TrollVault, Tier::Uncommon, "minecraft:obsidian", 6, 0, 10),
    row(Table::TrollVault, Tier::Rare, "lamp_of_cinders", 1, 0, 10),
];

mod chest
{
    pub const SLOTS: i32 = 27;
    pub const TRIES: i32 = 100;
    // one draw in four comes from the useless or ultra rare side table when it has entries
    pub const SIDE_ODDS: i32 = 4;
}

// the loot table each TFTreasure registers as, in the loader's chest loot so recipe viewers and other mods see it
struct TableRow
{
    m_table: Table,
    m_name: &'static str,
}

const TABLES: [TableRow; 22] = [
    TableRow { m_table: Table::Hill1, m_name: "hill_1" },
    TableRow { m_table: Table::Hill2, m_name: "hill_2" },
    TableRow { m_table: Table::Hill3, m_name: "hill_3" },
    TableRow { m_table: Table::HedgeMaze, m_name: "hedge_maze" },
    TableRow { m_table: Table::LabyrinthRoom, m_name: "labyrinth_room" },
    TableRow { m_table: Table::LabyrinthDeadEnd, m_name: "labyrinth_dead_end" },
    TableRow { m_table: Table::TowerRoom, m_name: "tower_room" },
    TableRow { m_table: Table::TowerLibrary, m_name: "tower_library" },
    TableRow { m_table: Table::Basement, m_name: "basement" },
    TableRow { m_table: Table::LabyrinthVault, m_name: "labyrinth_vault" },
    TableRow { m_table: Table::DarkTowerCache, m_name: "dark_tower_cache" },
    TableRow { m_table: Table::DarkTowerKey, m_name: "dark_tower_key" },
    TableRow { m_table: Table::DarkTowerBoss, m_name: "dark_tower_boss" },
    TableRow { m_table: Table::TreeCache, m_name: "tree_cache" },
    TableRow { m_table: Table::StrongholdCache, m_name: "stronghold_cache" },
    TableRow { m_table: Table::StrongholdRoom, m_name: "stronghold_room" },
    TableRow { m_table: Table::StrongholdBoss, m_name: "stronghold_boss" },
    TableRow { m_table: Table::AuroraCache, m_name: "aurora_cache" },
    TableRow { m_table: Table::AuroraRoom, m_name: "aurora_room" },
    TableRow { m_table: Table::AuroraBoss, m_name: "aurora_boss" },
    TableRow { m_table: Table::TrollGarden, m_name: "troll_garden" },
    TableRow { m_table: Table::TrollVault, m_name: "troll_vault" },
];

// TFTreasure.generate's draws: four from the common tier or its useless side, two uncommon, one rare or ultra rare
struct Draw
{
    m_side: Tier,
    m_tier: Tier,
    m_rolls: i32,
}

const DRAWS: [Draw; 3] = [
    Draw { m_side: Tier::Useless, m_tier: Tier::Common, m_rolls: 4 },
    Draw { m_side: Tier::Uncommon, m_tier: Tier::Uncommon, m_rolls: 2 },
    Draw { m_side: Tier::UltraRare, m_tier: Tier::Rare, m_rolls: 1 },
];

fn tableName(table: Table) -> &'static str
{
    TABLES.iter().find(|row| row.m_table == table).expect("every treasure table has a row").m_name
}

fn entries(table: Table, tier: Tier) -> impl Iterator<Item = &'static Treasure>
{
    TREASURES.iter().filter(move |row| row.m_table == table && row.m_tier == tier)
}

fn total(table: Table, tier: Tier) -> i32 { entries(table, tier).map(|row| row.m_rarity).sum() }

// TFTreasure.getCommonItem and getRareItem pick the side tier one time in four when it has entries and the main tier
// otherwise, as one weighted pick: side rarities scale by the main total and main rarities by three side totals, and
// an empty main tier leaves its three quarters to nothing; answers side and main multipliers and the nothing weight
fn scales(side: i32, main: i32) -> (i32, i32, i32)
{
    let rest: i32 = chest::SIDE_ODDS - 1;

    match (side, main)
    {
        (0, _) => (0, 1, 0),
        (_, 0) => (1, 0, rest * side),
        _ => (main, rest * side, 0),
    }
}

fn add(pool: PoolBuilder, table: Table, tier: Tier, scale: i32) -> Result<PoolBuilder>
{
    entries(table, tier).filter(|_| scale > 0).try_fold(pool, |pool, row|
    {
        Ok(pool.entry(row.m_item, row.m_rarity * scale, 1, row.m_quantity)?.damage(row.m_damage))
    })
}

fn pool(table: Table, draw: &Draw) -> Result<Option<PoolBuilder>>
{
    let main: i32 = total(table, draw.m_tier);
    let side: i32 = if draw.m_side == draw.m_tier
    {
        0
    }
    else
    {
        total(table, draw.m_side)
    };
    if main + side == 0
    {
        return Ok(None);
    }

    let (sideScale, mainScale, nothing): (i32, i32, i32) = scales(side, main);
    let mut pool: PoolBuilder = PoolBuilder::new(draw.m_rolls, draw.m_rolls);
    pool = add(pool, table, draw.m_side, sideScale)?;
    pool = add(pool, table, draw.m_tier, mainScale)?;
    if nothing > 0
    {
        pool = pool.nothing(nothing);
    }

    Ok(Some(pool))
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for row in &TABLES
    {
        for draw in &DRAWS
        {
            if let Some(pool) = pool(row.m_table, draw)?
            {
                ctx.loot().pool(LootTarget::Chest, row.m_name, pool.m_spec)?;
            }
        }
    }

    Ok(())
}

// TFTreasure.generate, a chest filled from its loot table with a random seeded by its position
pub(crate) fn place(world: &GenWorld, _random: &Random, table: Table, x: i32, y: i32, z: i32)
{
    let seed: i64 = world.m_world.seed().wrapping_mul(i64::from(x)).wrapping_add(i64::from(y)) ^ i64::from(z);
    let random: Random = Random::withSeed(seed);
    let pos: BlockPos = BlockPos::new(x, y, z);

    world.set(x, y, z, palette::plain("minecraft:chest"));

    let Ok(id) = Identifier::parseIn(tableName(table), &modId())
    else
    {
        return;
    };

    let mut filled: [bool; chest::SLOTS as usize] = [false; chest::SLOTS as usize];

    for stack in rollLoot(&id, seed)
    {
        let slot: Option<i32> =
            (0..chest::TRIES).map(|_| random.nextIntBounded(chest::SLOTS)).find(|slot| !filled[*slot as usize]);
        let Some(slot) = slot
        else
        {
            continue;
        };

        filled[slot as usize] = true;

        if let Err(error) = world.m_world.setTileSlot(pos, slot, Some(stack))
        {
            b173_api::warn(&format!("treasure {table:?} at {x} {y} {z}: {}", error.0));
        }
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn side_tier_takes_a_quarter_of_the_picks()
    {
        let (side, main): (i32, i32) = (30, 70);
        let (sideScale, mainScale, nothing): (i32, i32, i32) = scales(side, main);
        let sideWeight: i32 = side * sideScale;
        let all: i32 = sideWeight + main * mainScale + nothing;
        assert_eq!(sideWeight * chest::SIDE_ODDS, all);
    }

    #[test]
    fn empty_main_tier_leaves_three_quarters_to_nothing()
    {
        let side: i32 = 20;
        let (sideScale, mainScale, nothing): (i32, i32, i32) = scales(side, 0);
        assert_eq!(mainScale, 0);
        assert_eq!(side * sideScale * chest::SIDE_ODDS, side * sideScale + nothing);
    }

    #[test]
    fn without_a_side_tier_the_main_tier_takes_every_pick()
    {
        assert_eq!(scales(0, 50), (0, 1, 0));
    }

    #[test]
    fn every_table_has_a_unique_name_and_some_treasure()
    {
        for (index, row) in TABLES.iter().enumerate()
        {
            assert!(TABLES.iter().skip(index + 1).all(|other| other.m_name != row.m_name));
            assert!(DRAWS.iter().any(|draw| total(row.m_table, draw.m_tier) + total(row.m_table, draw.m_side) > 0));
        }
    }
}
