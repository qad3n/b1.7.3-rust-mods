use super::layers::{self, BiomeMaps};
use crate::colors;
use b173_api::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

// TFBiomeBase biomes in a fixed order, the order is the biome index the loader keeps per column
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Biome
{
    Lake,
    TwilightForest,
    DenseForest,
    Highlands,
    Mushrooms,
    Swamp,
    Stream,
    SnowyForest,
    Glacier,
    Clearing,
    OakSavanna,
    FireflyForest,
    DeepMushrooms,
    DarkForest,
    EnchantedForest,
    FireSwamp,
    DarkForestCenter,
    FinalPlateau,
    Thornlands,
}

pub struct BiomeRow
{
    pub m_biome: Biome,
    pub m_name: &'static str,
    pub m_label: &'static str,
    pub m_map: u32,
    pub m_temperature: f32,
    pub m_rainfall: f32,
    // BiomeGenBase.Height root height and variation
    pub m_height: (f32, f32),
}

mod height
{
    pub const DEFAULT: (f32, f32) = (0.1, 0.2);
    pub const SHALLOW_WATERS: (f32, f32) = (-0.5, 0.0);
    pub const DEEP_OCEANS: (f32, f32) = (-1.8, 0.1);
    pub const LOW_PLAINS: (f32, f32) = (0.125, 0.05);
    pub const MID_PLAINS: (f32, f32) = (0.2, 0.2);
}

const fn row(
    biome: Biome,
    name: &'static str,
    label: &'static str,
    map: u32,
    climate: (f32, f32),
    height: (f32, f32),
) -> BiomeRow
{
    BiomeRow
    {
        m_biome: biome,
        m_name: name,
        m_label: label,
        m_map: map,
        m_temperature: climate.0,
        m_rainfall: climate.1,
        m_height: height,
    }
}

// TFBiomeBase registrations with each biome constructor's temperature and rainfall
pub const BIOMES: [BiomeRow; 19] = [
    row(Biome::Lake, "twilight_lake", "Twilight Lake", 0x0000FF, (0.66, 1.0), height::DEEP_OCEANS),
    row(Biome::TwilightForest, "twilight_forest", "Twilight Forest", 0x005500, (0.5, 0.5), height::DEFAULT),
    row(Biome::DenseForest, "dense_twilight_forest", "Dense Twilight Forest", 0x005522, (0.7, 0.8), height::MID_PLAINS),
    row(Biome::Highlands, "twilight_highlands", "Twilight Highlands", 0x556644, (0.4, 0.7), (3.5, 0.05)),
    row(Biome::Mushrooms, "mushroom_forest", "Mushroom Forest", 0x226622, (0.8, 0.8), height::DEFAULT),
    row(Biome::Swamp, "twilight_swamp", "Twilight Swamp", 0x334422, (0.8, 0.9), (-0.125, 0.125)),
    row(Biome::Stream, "twilight_stream", "Twilight Stream", 0x3253B7, (0.5, 1.0), height::SHALLOW_WATERS),
    row(Biome::SnowyForest, "snowy_forest", "Snowy Forest", 0xCCFFFF, (0.125, 0.9), height::MID_PLAINS),
    row(Biome::Glacier, "twilight_glacier", "Twilight Glacier", 0x7777BB, (0.0, 0.1), height::DEFAULT),
    row(Biome::Clearing, "twilight_clearing", "Twilight Clearing", 0x349B34, (0.8, 0.4), height::LOW_PLAINS),
    row(Biome::OakSavanna, "oak_savanna", "Oak Savanna", 0x446622, (0.9, 0.0), height::MID_PLAINS),
    row(Biome::FireflyForest, "firefly_forest", "Firefly Forest", 0x006633, (0.5, 1.0), height::LOW_PLAINS),
    row(Biome::DeepMushrooms, "deep_mushroom_forest", "Deep Mushroom Forest", 0x663322, (0.8, 1.0), height::LOW_PLAINS),
    row(Biome::DarkForest, "dark_forest", "Dark Forest", 0x003311, (0.7, 0.8), height::LOW_PLAINS),
    row(Biome::EnchantedForest, "enchanted_forest", "Enchanted Forest", 0x115566, (0.5, 0.5), height::DEFAULT),
    row(Biome::FireSwamp, "fire_swamp", "Fire Swamp", 0x42231A, (1.0, 0.4), height::DEFAULT),
    row(Biome::DarkForestCenter, "dark_forest_center", "Dark Forest Center", 0x002200, (0.7, 0.8), height::LOW_PLAINS),
    row(Biome::FinalPlateau, "highlands_center", "Highlands Center", 0x334422, (0.3, 0.2), (10.5, 0.025)),
    row(Biome::Thornlands, "thornlands", "Thornlands", 0x223322, (0.3, 0.2), (6.0, 0.1)),
];

pub fn rowOf(biome: Biome) -> &'static BiomeRow { &BIOMES[biome as usize] }

pub fn fromIndex(index: i32) -> Biome
{
    BIOMES.get(index.max(0) as usize).map_or(Biome::TwilightForest, |row| row.m_biome)
}

// per chunk biome columns, x fastest, as the loader and the terrain ask for them
pub type Columns = [u8; 256];

const CACHE_LIMIT: usize = 1024;

#[derive(Default)]
struct Cache
{
    m_seed: Option<i64>,
    m_maps: Option<BiomeMaps>,
    m_chunks: HashMap<(i32, i32), Columns>,
}

thread_local!
{
    static CACHE: RefCell<Cache> = RefCell::new(Cache::default());
}

fn withMaps<T>(seed: i64, run: impl FnOnce(&mut BiomeMaps, &mut HashMap<(i32, i32), Columns>) -> T) -> T
{
    CACHE.with(|cache|
    {
        let mut cache = cache.borrow_mut();
        if cache.m_seed != Some(seed)
        {
            cache.m_seed = Some(seed);
            cache.m_maps = Some(layers::build(seed));
            cache.m_chunks.clear();
        }
        let Cache { m_maps: maps, m_chunks: chunks, .. } = &mut *cache;
        run(maps.as_mut().expect("maps are built with the seed"), chunks)
    })
}

// TFWorldChunkManager.loadBlockGeneratorData through its biome cache, one biome per block
pub fn chunk(seed: i64, chunkX: i32, chunkZ: i32) -> Columns
{
    withMaps(seed, |maps, chunks|
    {
        if let Some(columns) = chunks.get(&(chunkX, chunkZ))
        {
            return *columns;
        }
        let ids: Vec<i32> = maps.m_fine.ints(chunkX * 16, chunkZ * 16, 16, 16);
        let mut columns: Columns = [0; 256];
        for (column, id) in columns.iter_mut().zip(ids)
        {
            *column = id.max(0) as u8;
        }
        if chunks.len() >= CACHE_LIMIT
        {
            chunks.clear();
        }
        chunks.insert((chunkX, chunkZ), columns);
        columns
    })
}

pub fn at(seed: i64, x: i32, z: i32) -> Biome
{
    let columns: Columns = chunk(seed, x >> 4, z >> 4);
    fromIndex(i32::from(columns[((z & 15) * 16 + (x & 15)) as usize]))
}

// TFWorldChunkManager.getBiomesForGeneration, one biome per four blocks for the terrain shape
pub fn coarse(seed: i64, x: i32, z: i32, width: i32, depth: i32) -> Vec<Biome>
{
    withMaps(seed, |maps, _| maps.m_coarse.ints(x, z, width, depth).into_iter().map(fromIndex).collect())
}

thread_local!
{
    // the colorizer colors of every biome, asked of the host once since a biome's climate never changes
    static CLIMATES: RefCell<HashMap<Biome, (u32, u32)>> = RefCell::new(HashMap::new());
}

fn climateColors(row: &BiomeRow) -> (u32, u32)
{
    if let Some(colors) = CLIMATES.with(|climates| climates.borrow().get(&row.m_biome).copied())
    {
        return colors;
    }
    let temperature: f64 = f64::from(row.m_temperature.clamp(0.0, 1.0));
    let rainfall: f64 = f64::from(row.m_rainfall.clamp(0.0, 1.0));
    let colors: (u32, u32) = (grassColor(temperature, rainfall), foliageColor(temperature, rainfall));
    // the server has no colormaps and answers white, ask again once a client has them
    if colors.0 != 0xFFFFFF
    {
        CLIMATES.with(|climates| climates.borrow_mut().insert(row.m_biome, colors));
    }
    colors
}

// a column's colorizer colour from its biome's climate, as beta colours vanilla grass and leaves
fn columnColor(world: World, pos: BlockPos, grass: bool) -> u32
{
    let (baseGrass, baseFoliage): (u32, u32) = climateColors(rowOf(at(world.seed(), pos.x, pos.z)));
    if grass { baseGrass } else { baseFoliage }
}

pub fn grass(world: World, pos: BlockPos) -> u32 { columnColor(world, pos, true) }

pub fn foliage(world: World, pos: BlockPos) -> u32 { columnColor(world, pos, false) }

pub fn grassAround(world: World, pos: BlockPos) -> u32
{
    let mut sums: [u32; 3] = [0; 3];
    for dz in -1..=1
    {
        for dx in -1..=1
        {
            for (sum, value) in sums.iter_mut().zip(colors::channels(grass(world, pos.offset(dx, 0, dz))))
            {
                *sum += value;
            }
        }
    }
    let around: u32 = 9;
    colors::rgb((sums[0] / around) as i32, (sums[1] / around) as i32, (sums[2] / around) as i32)
}

pub fn biomeAt(world: World, pos: BlockPos) -> Biome { at(world.seed(), pos.x, pos.z) }

// StructureTFMajorFeatureStart.moveToAvgGroundLevel, how far a tower rises with the terrain of its biome
pub fn surfaceRise(seed: i64, x: i32, z: i32) -> i32
{
    // the dark forest canopy hides towers built at the plain height
    const DARK_FOREST_RISE: i32 = 4;
    const BLOCKS_PER_HEIGHT: f32 = 8.0;
    let biome: Biome = at(seed, x, z);
    let (root, variation): (f32, f32) = rowOf(biome).m_height;
    let rise: i32 = ((root + variation) * BLOCKS_PER_HEIGHT) as i32;
    if matches!(biome, Biome::DarkForest)
    {
        rise + DARK_FOREST_RISE
    }
    else
    {
        rise
    }
}
