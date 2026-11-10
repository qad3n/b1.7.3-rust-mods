use super::mobs::{GHASTS, MOBS, Mob, Spawning};
use crate::world::biomes::{self, Biome};
use crate::world::features::Feature;
use b173_api::prelude::*;

// ChunkProviderTwilightForest.getPossibleCreatures: inside a feature's pieces its own lists, nothing in a conquered
// feature, the biome's underground monsters below sea level and the biome's own lists elsewhere; beta lacks
// endermen, witches, blazes, bats and cave spiders, so those entries are left out and cave spiders become spiders

mod depth
{
    // TFWorld.SEALEVEL, below it a twilight biome offers its underground monsters
    pub const SEA: i32 = 31;
}

type Entries = &'static [(&'static str, i32)];

// TFBiomeBase.undergroundMonsterList, TFBiomeHighlands keeps its own
const UNDERGROUND: Entries = &[
    ("minecraft:spider", 10),
    ("minecraft:zombie", 10),
    ("minecraft:skeleton", 10),
    ("minecraft:creeper", 1),
    ("minecraft:slime", 10),
    ("twilightforest:kobold", 10),
];
const HIGHLANDS_UNDERGROUND: Entries =
    &[("minecraft:skeleton", 10), ("minecraft:creeper", 1), ("minecraft:slime", 10), ("twilightforest:troll", 10)];

// the monster lists a few biomes offer on a one in odds roll of their monsterRNG, every other biome has none
struct Surface
{
    m_biomes: &'static [Biome],
    m_odds: i32,
    m_entries: Entries,
}

const SURFACE: [Surface; 3] = [
    // TFBiomeDarkForest, the dark forest center included
    Surface
    {
        m_biomes: &[Biome::DarkForest, Biome::DarkForestCenter],
        m_odds: 20,
        m_entries: &[
            ("minecraft:zombie", 5),
            ("minecraft:skeleton", 5),
            ("twilightforest:mist_wolf", 10),
            ("twilightforest:skeleton_druid", 10),
            ("twilightforest:king_spider", 10),
            ("twilightforest:kobold", 10),
        ],
    },
    // TFBiomeSnow
    Surface
    {
        m_biomes: &[Biome::SnowyForest],
        m_odds: 10,
        m_entries: &[("twilightforest:yeti", 20), ("twilightforest:winter_wolf", 5)],
    },
    // TFBiomeSwamp
    Surface
    {
        m_biomes: &[Biome::Swamp],
        m_odds: 20,
        m_entries: &[("twilightforest:mosquito_swarm", 10), ("minecraft:creeper", 10), ("minecraft:zombie", 10)],
    },
];

// TFBiomeBase's creature list beside the twilight animals, which mobs.rs weighs
const VANILLA_CREATURES: Entries = &[("minecraft:chicken", 10), ("minecraft:wolf", 5)];
// TFBiomeTwilightLake
const LAKE_WATER: Entries = &[("minecraft:squid", 10)];
// TFFeature.waterCreatureList of the dark tower
const TOWER_WATER: Entries = &[("minecraft:squid", 10)];

// TFFeature.addMonster by feature and list index, with the source's weights
const FEATURE_MONSTERS: &[(Feature, i32, &str, i32)] = &[
    (Feature::LichTower, 0, "minecraft:zombie", 10),
    (Feature::LichTower, 0, "minecraft:skeleton", 10),
    (Feature::LichTower, 0, "minecraft:creeper", 1),
    (Feature::LichTower, 0, "twilightforest:death_tome", 10),
    (Feature::SmallHill, 0, "minecraft:spider", 10),
    (Feature::SmallHill, 0, "minecraft:zombie", 10),
    (Feature::SmallHill, 0, "twilightforest:redcap", 10),
    (Feature::SmallHill, 0, "twilightforest:swarm_spider", 10),
    (Feature::SmallHill, 0, "twilightforest:kobold", 10),
    (Feature::MediumHill, 0, "twilightforest:redcap", 10),
    (Feature::MediumHill, 0, "twilightforest:redcap_sapper", 1),
    (Feature::MediumHill, 0, "twilightforest:kobold", 10),
    (Feature::MediumHill, 0, "minecraft:skeleton", 10),
    (Feature::MediumHill, 0, "twilightforest:swarm_spider", 10),
    (Feature::MediumHill, 0, "minecraft:spider", 10),
    (Feature::MediumHill, 0, "minecraft:creeper", 10),
    (Feature::MediumHill, 0, "twilightforest:fire_beetle", 5),
    (Feature::MediumHill, 0, "twilightforest:slime_beetle", 5),
    (Feature::LargeHill, 0, "twilightforest:redcap", 10),
    (Feature::LargeHill, 0, "twilightforest:redcap_sapper", 2),
    (Feature::LargeHill, 0, "minecraft:skeleton", 10),
    (Feature::LargeHill, 0, "minecraft:spider", 10),
    (Feature::LargeHill, 0, "minecraft:creeper", 10),
    (Feature::LargeHill, 0, "twilightforest:twilight_wraith", 2),
    (Feature::LargeHill, 0, "twilightforest:fire_beetle", 10),
    (Feature::LargeHill, 0, "twilightforest:slime_beetle", 10),
    (Feature::LargeHill, 0, "twilightforest:pinch_beetle", 10),
    (Feature::Labyrinth, 0, "twilightforest:minotaur", 20),
    (Feature::Labyrinth, 0, "minecraft:spider", 10),
    (Feature::Labyrinth, 0, "minecraft:creeper", 10),
    (Feature::Labyrinth, 0, "twilightforest:maze_slime", 10),
    (Feature::Labyrinth, 0, "twilightforest:fire_beetle", 10),
    (Feature::Labyrinth, 0, "twilightforest:slime_beetle", 10),
    (Feature::Labyrinth, 0, "twilightforest:pinch_beetle", 10),
    (Feature::DarkTower, 0, "twilightforest:tower_golem", 10),
    (Feature::DarkTower, 0, "minecraft:skeleton", 10),
    (Feature::DarkTower, 0, "minecraft:creeper", 10),
    (Feature::DarkTower, 0, "twilightforest:mini_ghast", 10),
    (Feature::DarkTower, 0, "twilightforest:tower_broodling", 10),
    (Feature::DarkTower, 0, "twilightforest:pinch_beetle", 10),
    (Feature::DarkTower, 1, "twilightforest:tower_ghast", 10),
    (Feature::Stronghold, 0, "twilightforest:block_goblin", 10),
    (Feature::Stronghold, 0, "twilightforest:goblin_knight_lower", 5),
    (Feature::Stronghold, 0, "twilightforest:helmet_crab", 10),
    (Feature::Stronghold, 0, "twilightforest:slime_beetle", 10),
    (Feature::Stronghold, 0, "twilightforest:redcap_sapper", 2),
    (Feature::Stronghold, 0, "twilightforest:kobold", 10),
    (Feature::Stronghold, 0, "minecraft:creeper", 10),
    (Feature::Stronghold, 0, "minecraft:slime", 5),
    (Feature::YetiCave, 0, "twilightforest:yeti", 10),
    (Feature::IceTower, 0, "twilightforest:snow_guardian", 10),
    (Feature::IceTower, 0, "twilightforest:stable_ice_core", 10),
    (Feature::IceTower, 0, "twilightforest:unstable_ice_core", 5),
    (Feature::TrollCave, 0, "minecraft:creeper", 5),
    (Feature::TrollCave, 0, "minecraft:skeleton", 10),
    (Feature::TrollCave, 0, "twilightforest:troll", 20),
    (Feature::TrollCave, 1, "twilightforest:giant_miner", 10),
    (Feature::TrollCave, 1, "twilightforest:armored_giant", 10),
    (Feature::FinalCastle, 0, "twilightforest:kobold", 10),
    (Feature::FinalCastle, 0, "twilightforest:adherent", 10),
    (Feature::FinalCastle, 0, "twilightforest:harbinger_cube", 10),
    (Feature::FinalCastle, 1, "twilightforest:kobold", 10),
    (Feature::FinalCastle, 1, "twilightforest:adherent", 10),
    (Feature::FinalCastle, 1, "twilightforest:harbinger_cube", 10),
    (Feature::FinalCastle, 1, "twilightforest:armored_giant", 10),
    (Feature::FinalCastle, 2, "twilightforest:adherent", 10),
];

fn entries(list: impl IntoIterator<Item = (&'static str, i32)>) -> Vec<SpawnEntry>
{
    list.into_iter().filter_map(|(name, weight)| spawnEntry(name, weight).ok()).collect()
}

// TFFeature.getSpawnableList, creatures never spawn inside a feature's pieces
fn featureList(feature: Feature, index: i32, kind: CreatureKind) -> Vec<SpawnEntry>
{
    match kind
    {
        CreatureKind::Monster => entries(
            FEATURE_MONSTERS
                .iter()
                .filter(|(listed, list, ..)| *listed == feature && *list == index)
                .map(|(_, _, name, weight)| (*name, *weight)),
        ),
        CreatureKind::Water if feature == Feature::DarkTower => entries(TOWER_WATER.iter().copied()),
        CreatureKind::Water | CreatureKind::Creature => Vec::new(),
    }
}

// TFBiomeBase's creature list, the final plateau keeps only ravens, the thornlands and streams have none
fn creatureOffered(mob: &Mob, biome: Biome) -> Option<i32>
{
    let cleared: bool = matches!(biome, Biome::Thornlands | Biome::Stream);
    match mob.m_spawn
    {
        Spawning::Forest(weight) if !cleared && biome != Biome::FinalPlateau => Some(weight),
        Spawning::Everywhere(weight) if !cleared => Some(weight),
        Spawning::Only(weight, biomes) if biomes.contains(&biome) => Some(weight),
        _ => None,
    }
}

fn creatures(biome: Biome) -> Vec<SpawnEntry>
{
    let base: bool = !matches!(biome, Biome::Thornlands | Biome::Stream | Biome::FinalPlateau);
    let vanilla: Entries = if base { VANILLA_CREATURES } else { &[] };
    let twilight = MOBS
        .iter()
        .chain(GHASTS.iter())
        .filter_map(|mob| creatureOffered(mob, biome).map(|weight| (mob.m_name, weight)));
    let mut list: Vec<SpawnEntry> = entries(vanilla.iter().copied());
    list.extend(twilight.filter_map(|(name, weight)| spawnEntry(name, weight).ok()));
    list
}

// TFBiomeDarkForest, TFBiomeSnow and TFBiomeSwamp.getSpawnableList
fn monsters(world: World, biome: Biome) -> Vec<SpawnEntry>
{
    let Some(surface) = SURFACE.iter().find(|surface| surface.m_biomes.contains(&biome))
    else
    {
        return Vec::new();
    };

    if world.random(surface.m_odds) != 0
    {
        return Vec::new();
    }
    entries(surface.m_entries.iter().copied())
}

fn biomeList(world: World, biome: Biome, kind: CreatureKind, y: i32) -> Vec<SpawnEntry>
{
    match kind
    {
        CreatureKind::Monster if y < depth::SEA =>
        {
            let list: Entries = if biome == Biome::Highlands
            {
                HIGHLANDS_UNDERGROUND
            }
            else
            {
                UNDERGROUND
            };
            entries(list.iter().copied())
        }
        CreatureKind::Monster => monsters(world, biome),
        CreatureKind::Creature => creatures(biome),
        CreatureKind::Water if biome == Biome::Lake => entries(LAKE_WATER.iter().copied()),
        CreatureKind::Water => Vec::new(),
    }
}

// the dimension's spawn list handler, every natural spawn attempt in the twilight forest asks it
pub fn possible(query: &SpawnQuery) -> Option<Vec<SpawnEntry>>
{
    let world: World = query.world;
    let BlockPos { x, y, z } = query.pos;
    if let Some((feature, index)) = crate::world::structures::spawnListAt(world, x, y, z)
    {
        let center: [f64; 3] = [f64::from(x), f64::from(y), f64::from(z)];
        if crate::bosses::conquered(world, center)
        {
            return Some(Vec::new());
        }
        if index >= 0
        {
            return Some(featureList(feature, index, query.kind));
        }
    }

    let biome: Biome = biomes::at(world.seed(), x, z);
    Some(biomeList(world, biome, query.kind, y))
}
