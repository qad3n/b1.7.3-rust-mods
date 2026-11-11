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
    row(Table::Hill1, Tier::UltraRare, "transformation_powder", 12, 0, 10),
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
    row(Table::Hill2, Tier::Rare, "transformation_powder", 12, 0, 10),
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
    row(Table::Hill3, Tier::Rare, "transformation_powder", 12, 0, 10),
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
    row(Table::TowerRoom, Tier::Rare, "transformation_powder", 12, 0, 10),
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
    row(Table::TowerLibrary, Tier::Uncommon, "minecraft:map", 1, 0, 10),
    row(Table::TowerLibrary, Tier::Rare, "minecraft:bow", 1, 0, 10),
    row(Table::TowerLibrary, Tier::Rare, "minecraft:stone_sword", 1, 0, 10),
    row(Table::TowerLibrary, Tier::Rare, "minecraft:wooden_sword", 1, 0, 10),
    row(Table::TowerLibrary, Tier::Rare, "transformation_powder", 12, 0, 10),
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
    row(Table::Basement, Tier::Rare, "minecraft:map", 1, 0, 10),
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
    pub const COMMON: i32 = 4;
    pub const UNCOMMON: i32 = 2;
    pub const RARE: i32 = 1;
    // one draw in four comes from the useless or ultra rare side table when it has entries
    pub const SIDE_ODDS: i32 = 4;
}

fn entries(table: Table, tier: Tier) -> impl Iterator<Item = &'static Treasure>
{
    TREASURES.iter().filter(move |row| row.m_table == table && row.m_tier == tier)
}

// TFTreasureTable.getRandomItem
fn draw(random: &Random, table: Table, tier: Tier) -> Option<ItemStack>
{
    let total: i32 = entries(table, tier).map(|row| row.m_rarity).sum();
    if total <= 0
    {
        return None;
    }

    let mut value: i32 = random.nextIntBounded(total);

    for row in entries(table, tier)
    {
        if row.m_rarity > value
        {
            let count: i32 = random.nextIntBounded(row.m_quantity) + 1;
            return Some(ItemStack::of(row.m_item, count).withDamage(row.m_damage));
        }

        value -= row.m_rarity;
    }

    None
}

fn hasEntries(table: Table, tier: Tier) -> bool { entries(table, tier).next().is_some() }

// TFTreasure.getCommonItem and getRareItem
fn drawWithSide(random: &Random, table: Table, side: Tier, tier: Tier) -> Option<ItemStack>
{
    if hasEntries(table, side) && random.nextIntBounded(chest::SIDE_ODDS) == 0
    {
        draw(random, table, side)
    }
    else
    {
        draw(random, table, tier)
    }
}

// TFTreasure.generate, a chest filled from the table with a random seeded by its position
pub(crate) fn place(world: &GenWorld, _random: &Random, table: Table, x: i32, y: i32, z: i32)
{
    let seed: i64 = world.m_world.seed().wrapping_mul(i64::from(x)).wrapping_add(i64::from(y)) ^ i64::from(z);
    let random: Random = Random::withSeed(seed);
    let pos: BlockPos = BlockPos::new(x, y, z);

    world.set(x, y, z, palette::plain("minecraft:chest"));

    let mut filled: [bool; chest::SLOTS as usize] = [false; chest::SLOTS as usize];
    let draws: [(Tier, Tier, i32); 3] = [
        (Tier::Useless, Tier::Common, chest::COMMON),
        (Tier::Uncommon, Tier::Uncommon, chest::UNCOMMON),
        (Tier::UltraRare, Tier::Rare, chest::RARE),
    ];

    for (side, tier, count) in draws
    {
        for _ in 0..count
        {
            let stack: Option<ItemStack> = drawWithSide(&random, table, side, tier);
            let slot: Option<i32> =
                (0..chest::TRIES).map(|_| random.nextIntBounded(chest::SLOTS)).find(|slot| !filled[*slot as usize]);

            let (Some(stack), Some(slot)) = (stack, slot)
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
}
