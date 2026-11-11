mod minable;
pub(crate) mod tree;

use crate::blocks::{COPPER_ORE, TIN_ORE, URANIUM_ORE};
use crate::core::ids;
use b173_api::prelude::*;
use b173_api::terrain::Random;

const FEATURE: &str = "ic2:surface";
const CHUNK: i32 = 16;
const PERCENT: i32 = 100;

// one ore of mod_IC2.GenerateSurface: veins per chunk, vein size and the height draw
struct Ore
{
    m_block: &'static str,
    m_veins: i32,
    m_size: i32,
    m_height: fn(&Random) -> i32,
}

const ORES: [Ore; 3] = [
    Ore { m_block: COPPER_ORE, m_veins: 15, m_size: 10, m_height: copperHeight },
    Ore { m_block: TIN_ORE, m_veins: 25, m_size: 6, m_height: tinHeight },
    Ore { m_block: URANIUM_ORE, m_veins: 20, m_size: 3, m_height: uraniumHeight },
];

// copper favours the middle depths, two draws added up from ten blocks above the bottom
fn copperHeight(random: &Random) -> i32
{
    const DEEP: i32 = 40;
    const SHALLOW: i32 = 20;
    const FLOOR: i32 = 10;

    random.nextIntBounded(DEEP) + random.nextIntBounded(SHALLOW) + FLOOR
}

fn tinHeight(random: &Random) -> i32
{
    const DEPTH: i32 = 40;

    random.nextIntBounded(DEPTH)
}

fn uraniumHeight(random: &Random) -> i32
{
    const DEPTH: i32 = 64;

    random.nextIntBounded(DEPTH)
}

// rubber trees in a chunk: forests sometimes, rainforests often and swamps most of all
fn treeCount(world: World, random: &Random, x: i32, z: i32) -> i32
{
    const FOREST: i32 = 3;
    const RAINFOREST: i32 = 5;
    const SWAMP: i32 = 10;
    const SWAMP_FLOOR: i32 = 5;

    let biome: String = world.biomeAt(x + CHUNK, z + CHUNK).map(|id| id.to_string()).unwrap_or_default();

    match biome.as_str()
    {
        "minecraft:forest" | "minecraft:seasonal_forest" => random.nextIntBounded(FOREST),
        "minecraft:rainforest" => random.nextIntBounded(RAINFOREST) + 1,
        "minecraft:swampland" => random.nextIntBounded(SWAMP) + SWAMP_FLOOR,
        _ => 0,
    }
}

// a stable seed per chunk from the world seed and the feature's name, so other mods never shift the ores
fn seed(world: World, chunkX: i32, chunkZ: i32) -> i64
{
    const SPREAD_X: i64 = 341873128712;
    const SPREAD_Z: i64 = 132897987541;
    const HASH: i64 = 31;

    let name: i64 = FEATURE.bytes().fold(0i64, |hash, byte| hash.wrapping_mul(HASH).wrapping_add(i64::from(byte)));
    world.seed() ^ name ^ i64::from(chunkX).wrapping_mul(SPREAD_X) ^ i64::from(chunkZ).wrapping_mul(SPREAD_Z)
}

// mod_IC2.GenerateSurface
fn populate(world: World, chunkX: i32, chunkZ: i32)
{
    let random: Random = Random::withSeed(seed(world, chunkX, chunkZ));
    let (x, z): (i32, i32) = (chunkX * CHUNK, chunkZ * CHUNK);
    let trees: i32 = treeCount(world, &random, x, z);

    if random.nextIntBounded(PERCENT) < trees * 2
    {
        let start: [i32; 2] = [x + random.nextIntBounded(CHUNK), z + random.nextIntBounded(CHUNK)];
        tree::generate(world, &random, start, trees);
    }

    for ore in &ORES
    {
        let Some(block) = ids::block(ore.m_block)
        else
        {
            continue;
        };

        for _ in 0..ore.m_veins
        {
            let at: [i32; 3] =
                [x + random.nextIntBounded(CHUNK), (ore.m_height)(&random), z + random.nextIntBounded(CHUNK)];
            minable::generate(world, &random, block, ore.m_size, at);
        }
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.events().on::<ChunkPopulate>(Priority::Normal, |event|
    {
        let EventData::Chunk { world, x, z } = *event.data()
        else
        {
            return;
        };
        let world: World = World { dimension: world };

        if world.dimension == World::OVERWORLD.dimension && world.isAuthority()
        {
            populate(world, x, z);
        }
    })
}
