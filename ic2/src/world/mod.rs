pub(crate) mod tree;

use crate::blocks::{COPPER_ORE, TIN_ORE, URANIUM_ORE};
use b173_api::prelude::*;
use b173_api::terrain::Random;

const TREES: &str = "rubber_trees";
const CHUNK: i32 = 16;
const PERCENT: i32 = 100;

// one ore of mod_IC2.GenerateSurface: veins per chunk, vein size and the draws its height sums
struct Ore
{
    m_name: &'static str,
    m_block: &'static str,
    m_veins: i32,
    m_size: i32,
    m_draws: &'static [i32],
    m_floor: i32,
}

// copper favours the middle depths, two draws added up from ten blocks above the bottom
const ORES: [Ore; 3] = [
    Ore { m_name: "copper_ore", m_block: COPPER_ORE, m_veins: 15, m_size: 10, m_draws: &[40, 20], m_floor: 10 },
    Ore { m_name: "tin_ore", m_block: TIN_ORE, m_veins: 25, m_size: 6, m_draws: &[40], m_floor: 0 },
    Ore { m_name: "uranium_ore", m_block: URANIUM_ORE, m_veins: 20, m_size: 3, m_draws: &[64], m_floor: 0 },
];

// rubber trees in a chunk: forests sometimes, rainforests often and swamps most of all, any mod's biomes by tag
fn treeCount(world: World, random: &Random, x: i32, z: i32) -> i32
{
    const FOREST: i32 = 3;
    const RAINFOREST: i32 = 5;
    const SWAMP: i32 = 10;
    const SWAMP_FLOOR: i32 = 5;

    let Some(biome) = world.biomeAt(x + CHUNK, z + CHUNK)
    else
    {
        return 0;
    };
    let tagged = |tag: &str| tagMembers(RegistryKind::Biome, tag).contains(&biome);

    if tagged("minecraft:is_jungle")
    {
        return random.nextIntBounded(RAINFOREST) + 1;
    }
    if tagged("c:is_swamp")
    {
        return random.nextIntBounded(SWAMP) + SWAMP_FLOOR;
    }
    if tagged("minecraft:is_forest")
    {
        return random.nextIntBounded(FOREST);
    }

    0
}

// the rubber tree half of mod_IC2.GenerateSurface, the loader seeds the random from the feature's name
fn populate(ctx: &FeatureCtx) -> bool
{
    let (x, z): (i32, i32) = (ctx.origin.x, ctx.origin.z);
    let trees: i32 = treeCount(ctx.world, ctx.random, x, z);
    if ctx.random.nextIntBounded(PERCENT) >= trees * 2
    {
        return false;
    }

    let start: [i32; 2] = [x + ctx.random.nextIntBounded(CHUNK), z + ctx.random.nextIntBounded(CHUNK)];
    tree::generate(ctx.world, ctx.random, start, trees);
    true
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for ore in &ORES
    {
        let height: HeightSpec = HeightSpec::Sum { draws: ore.m_draws.to_vec(), offset: ore.m_floor };
        ctx.worldgen().add(ore.m_name, FeatureBuilder::ore(ore.m_block, ore.m_size, ore.m_veins, height)?)?;
    }

    ctx.worldgen().custom(TREES, FeatureBuilder::custom(FeatureStep::VegetalDecoration), populate)?;
    ctx.worldgen().custom(tree::TREE, FeatureBuilder::custom(FeatureStep::VegetalDecoration).placed(), tree::place)
}
