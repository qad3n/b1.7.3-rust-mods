use super::biomes::{self, Biome};
use crate::progress::Gate;
use b173_api::prelude::*;
use b173_api::terrain::Random;

// TFFeature, one landmark per 16x16 chunk region, its kind picked from the biome under the region center
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub(crate) enum Feature
{
    Nothing,
    SmallHill,
    MediumHill,
    LargeHill,
    HedgeMaze,
    NagaCourtyard,
    LichTower,
    IceTower,
    QuestIsland,
    QuestGrove,
    DruidGrove,
    FloatingRuins,
    HydraLair,
    Labyrinth,
    DarkTower,
    Stronghold,
    WorldTree,
    YetiCave,
    TrollCave,
    FinalCastle,
    MushroomTower,
}

pub(crate) struct FeatureRow
{
    pub m_feature: Feature,
    // the structure identifier path under twilightforest
    pub m_name: &'static str,
    pub m_label: &'static str,
    // radius in chunks of the area the feature claims
    pub m_size: i32,
    pub m_decorated: bool,
    pub m_structure: bool,
    pub m_terrain: bool,
    pub m_aura: bool,
    pub m_gate: Option<Gate>,
}

const fn feature(feature: Feature, name: &'static str, label: &'static str, size: i32) -> FeatureRow
{
    FeatureRow
    {
        m_feature: feature,
        m_name: name,
        m_label: label,
        m_size: size,
        m_decorated: false,
        m_structure: true,
        m_terrain: false,
        m_aura: true,
        m_gate: None,
    }
}

const fn decorated(mut row: FeatureRow) -> FeatureRow
{
    row.m_decorated = true;
    row
}

const fn shaping(mut row: FeatureRow) -> FeatureRow
{
    row.m_terrain = true;
    row
}

const fn bare(mut row: FeatureRow) -> FeatureRow
{
    row.m_structure = false;
    row
}

const fn gated(mut row: FeatureRow, gate: Gate) -> FeatureRow
{
    row.m_gate = Some(gate);
    row
}

const fn auraless(mut row: FeatureRow) -> FeatureRow
{
    row.m_aura = false;
    row
}

pub(crate) const FEATURES: [FeatureRow; 21] = [
    bare(decorated(feature(Feature::Nothing, "nothing", "No Feature", 0))),
    shaping(decorated(feature(Feature::SmallHill, "small_hollow_hill", "Small Hollow Hill", 1))),
    shaping(decorated(feature(Feature::MediumHill, "medium_hollow_hill", "Medium Hollow Hill", 2))),
    shaping(decorated(feature(Feature::LargeHill, "large_hollow_hill", "Large Hollow Hill", 3))),
    shaping(feature(Feature::HedgeMaze, "hedge_maze", "Hedge Maze", 2)),
    shaping(feature(Feature::NagaCourtyard, "naga_courtyard", "Naga Courtyard", 3)),
    gated(feature(Feature::LichTower, "lich_tower", "Lich Tower", 1), Gate::KillNaga),
    gated(feature(Feature::IceTower, "ice_tower", "Ice Tower", 2), Gate::Yeti),
    bare(feature(Feature::QuestIsland, "quest_island", "Quest Island", 1)),
    shaping(feature(Feature::QuestGrove, "quest_grove", "Quest Grove", 1)),
    bare(feature(Feature::DruidGrove, "druid_grove", "Druid Grove", 1)),
    bare(feature(Feature::FloatingRuins, "floating_ruins", "Floating Ruins", 3)),
    shaping(gated(feature(Feature::HydraLair, "hydra_lair", "Hydra Lair", 2), Gate::Labyrinth)),
    gated(decorated(feature(Feature::Labyrinth, "labyrinth", "Labyrinth", 3)), Gate::KillLich),
    gated(feature(Feature::DarkTower, "dark_tower", "Dark Tower", 1), Gate::Knights),
    auraless(gated(decorated(feature(Feature::Stronghold, "knight_stronghold", "Knight Stronghold", 3)), Gate::Trophy)),
    bare(feature(Feature::WorldTree, "world_tree", "World Tree", 3)),
    gated(shaping(decorated(feature(Feature::YetiCave, "yeti_lairs", "Yeti Lairs", 2))), Gate::UrGhast),
    auraless(gated(shaping(decorated(feature(Feature::TrollCave, "troll_lairs", "Troll Lairs", 3))), Gate::Glacier)),
    feature(Feature::FinalCastle, "final_castle", "Final Castle", 3),
    feature(Feature::MushroomTower, "mushroom_tower", "Mushroom Tower", 2),
];

pub(crate) fn row(feature: Feature) -> &'static FeatureRow { &FEATURES[feature as usize] }

mod region
{
    // features sit one per 16x16 chunks, a little off the region center
    pub const CHUNKS: i32 = 16;
    pub const SPREAD_X: i64 = 25117;
    pub const SPREAD_Z: i64 = 151121;
    pub const ROLLS: i32 = 16;
    pub const RINGS: i32 = 8;
}

// Java Math.round for floats
fn round(value: f32) -> i32 { (value + 0.5).floor() as i32 }

fn snap(chunk: i32) -> i32 { round(chunk as f32 / region::CHUNKS as f32) * region::CHUNKS }

// TFFeature.generateFeatureFor1Point7
pub(crate) fn forRegion(seed: i64, chunkX: i32, chunkZ: i32) -> Feature
{
    let (chunkX, chunkZ): (i32, i32) = (snap(chunkX), snap(chunkZ));
    let biome: Biome = biomes::at(seed, (chunkX << 4) + 8, (chunkZ << 4) + 8);
    let random: Random =
        Random::withSeed(seed + i64::from(chunkX) * region::SPREAD_X + i64::from(chunkZ) * region::SPREAD_Z);
    let roll: i32 = random.nextIntBounded(region::ROLLS);

    let byBiome: Option<Feature> = match biome
    {
        Biome::Glacier => Some(Feature::IceTower),
        Biome::SnowyForest => Some(Feature::YetiCave),
        Biome::Lake => Some(Feature::QuestIsland),
        Biome::EnchantedForest => Some(Feature::QuestGrove),
        Biome::FireSwamp => Some(Feature::HydraLair),
        Biome::Swamp => Some(Feature::Labyrinth),
        Biome::DarkForest => Some(Feature::Stronghold),
        Biome::DarkForestCenter => Some(Feature::DarkTower),
        Biome::FinalPlateau => Some(Feature::FinalCastle),
        Biome::Highlands => Some(Feature::TrollCave),
        Biome::DeepMushrooms => Some(Feature::MushroomTower),
        _ => None,
    };
    if let Some(feature) = byBiome
    {
        return feature;
    }
    let offsetX: i32 = (((chunkX + 64) >> 4) % region::RINGS).abs();
    let offsetZ: i32 = (((chunkZ + 64) >> 4) % region::RINGS).abs();
    // the regions next to the world center always hold a lich tower and a naga courtyard
    if offsetX == 4 && (offsetZ == 5 || offsetZ == 3)
    {
        return Feature::LichTower;
    }
    if offsetZ == 4 && (offsetX == 5 || offsetX == 3)
    {
        return Feature::NagaCourtyard;
    }

    match roll
    {
        6..=8 => Feature::MediumHill,
        9 => Feature::LargeHill,
        10 | 11 => Feature::HedgeMaze,
        12 | 13 => Feature::NagaCourtyard,
        14 | 15 => Feature::LichTower,
        _ => Feature::SmallHill,
    }
}

// TFFeature.getNearestCenterXYZ, the block the feature of this chunk's region centers on
pub(crate) fn regionCenter(chunkX: i32, chunkZ: i32) -> (i32, i32)
{
    let regionX: i32 = (chunkX + 8) >> 4;
    let regionZ: i32 = (chunkZ + 8) >> 4;
    let mut hash: i64 = i64::from(regionX).wrapping_mul(3129871) ^ i64::from(regionZ).wrapping_mul(116129781);
    hash = hash.wrapping_mul(hash).wrapping_mul(42317861).wrapping_add(hash.wrapping_mul(7));
    let bits = |shift: i64| ((hash >> shift) & 3) as i32;
    let centerX: i32 = 8 + bits(12) - bits(15);
    let centerZ: i32 = 8 + bits(18) - bits(21);

    let place = |region: i32, center: i32|
    {
        if region >= 0
        {
            (region * 16 + center - 8) * 16 + 8
        }
        else
        {
            (region * 16 + (16 - center) - 8) * 16 + 9
        }
    };
    (place(regionX, centerX), place(regionZ, centerZ))
}

// TFWorldChunkManager.isInFeatureChunk
pub(crate) fn featureChunk(chunkX: i32, chunkZ: i32) -> bool
{
    let (x, z): (i32, i32) = regionCenter(chunkX, chunkZ);
    chunkX == x >> 4 && chunkZ == z >> 4
}

// TFFeature.getFeatureDirectlyAt
pub(crate) fn directlyAt(seed: i64, chunkX: i32, chunkZ: i32) -> Feature
{
    if featureChunk(chunkX, chunkZ)
    {
        forRegion(seed, chunkX, chunkZ)
    }
    else
    {
        Feature::Nothing
    }
}

// TFFeature.getNearestFeature and getNearestCenter, the closest feature whose area covers this chunk
pub(crate) fn nearest(seed: i64, chunkX: i32, chunkZ: i32) -> (Feature, [i32; 2])
{
    for radius in 1..=3
    {
        for dx in -radius..=radius
        {
            for dz in -radius..=radius
            {
                let found: Feature = directlyAt(seed, dx + chunkX, dz + chunkZ);
                if row(found).m_size == radius
                {
                    return (found, [dx * 16 + 8, dz * 16 + 8]);
                }
            }
        }
    }

    (Feature::Nothing, [0, 0])
}

mod search
{
    // regions are 256 blocks wide and locate looks this many regions out from the world center
    pub const REGION: i32 = 256;
    pub const RINGS: i32 = 24;
}

// a test aid the source lacks, the closest center of a feature whose label holds the given words
fn locate(seed: i64, from: i32, wanted: &str) -> String
{
    for ring in from..from + search::RINGS
    {
        for regionX in -ring..=ring
        {
            for regionZ in -ring..=ring
            {
                if regionX.abs() != ring && regionZ.abs() != ring
                {
                    continue;
                }

                let (x, z): (i32, i32) = regionCenter(regionX * search::REGION >> 4, regionZ * search::REGION >> 4);
                let feature: Feature = directlyAt(seed, x >> 4, z >> 4);
                if row(feature).m_label.to_lowercase().contains(wanted)
                {
                    return format!("The center of {} is at {x}, {z}", row(feature).m_label);
                }
            }
        }
    }

    format!("no {wanted} within {} regions", search::RINGS)
}

// CommandTFFeature, the nearest feature to a player, or to the coordinates a console gives
pub(crate) fn registerCommand(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.commands().registerWithUsage(
        "tffeature",
        Permission::Op,
        "info|center [x z]|locate [ring] <label words>",
        |command|
        {
            if command.arg(0) == Some("locate")
            {
                let skip: Option<i32> = command.arg(1).and_then(|ring| ring.parse().ok());
                let first: usize = if skip.is_some() { 2 } else { 1 };
                let wanted: String =
                    (first..).map_while(|index| command.arg(index)).collect::<Vec<&str>>().join(" ").to_lowercase();

                if let Some(numeric) = super::dimension()
                {
                    command.reply(&locate(World { dimension: numeric }.seed(), skip.unwrap_or(0), &wanted));
                }

                return Ok(());
            }

            let position: Option<[i32; 2]> =
                match (command.arg(1).and_then(|x| x.parse().ok()), command.arg(2).and_then(|z| z.parse().ok()))
                {
                    (Some(x), Some(z)) => Some([x, z]),
                    _ => command
                        .player()
                        .and_then(Player::info)
                        .map(|info| [info.pos[0].floor() as i32, info.pos[2].floor() as i32]),
                };

            let Some([x, z]) = position
            else
            {
                command.reply("tffeature accepts the following arguments: info, center, a console adds x z");
                return Ok(());
            };

            let Some(numeric) = super::dimension()
            else
            {
                return Ok(());
            };

            let seed: i64 = World { dimension: numeric }.seed();
            let (feature, [offsetX, offsetZ]) = nearest(seed, x >> 4, z >> 4);
            let (centerX, centerZ): (i32, i32) = ((x >> 4 << 4) + offsetX, (z >> 4 << 4) + offsetZ);

            match command.arg(0)
            {
                Some("center") =>
                {
                    command.reply(&format!("The center of {} is at {centerX}, {centerZ}", row(feature).m_label))
                }

                _ => command.reply(&format!("The nearest feature is {}", row(feature).m_label)),
            }

            Ok(())
        },
    )
}
