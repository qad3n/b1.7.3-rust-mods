use super::biomes::{self, Biome};
use super::features::{self, Feature};
use super::noise::Octaves;
use crate::ids::{self, block};
use b173_api::prelude::*;
use b173_api::terrain::{ChunkBlocks, NoiseGeneratorOctaves, Random};
use std::cell::RefCell;

// TFWorld, terrain is shaped 256 tall with sea level 63 and squished into half that
pub const SEA_LEVEL: i32 = 31;
const TALL: usize = 256;
const SHAPE_SEA_LEVEL: i32 = 63;
const HEIGHT: i32 = 128;

// the noise generators ChunkProviderTwilightForest draws from its seed, in its order
struct Noises
{
    m_canopy: NoiseGeneratorOctaves,
    m_depth: NoiseGeneratorOctaves,
    m_low: NoiseGeneratorOctaves,
    m_high: NoiseGeneratorOctaves,
    m_selector: NoiseGeneratorOctaves,
    m_stone: Octaves,
    m_parabolic: [f32; 25],
}

impl Noises
{
    fn new(seed: i64) -> Noises
    {
        let random: Random = Random::withSeed(seed);
        let canopy: NoiseGeneratorOctaves = NoiseGeneratorOctaves::new(&random, 4);
        // the next generators are kept by the source but never read
        let _: NoiseGeneratorOctaves = NoiseGeneratorOctaves::new(&random, 10);
        let depth: NoiseGeneratorOctaves = NoiseGeneratorOctaves::new(&random, 16);
        let _: NoiseGeneratorOctaves = NoiseGeneratorOctaves::new(&random, 8);
        let low: NoiseGeneratorOctaves = NoiseGeneratorOctaves::new(&random, 16);
        let high: NoiseGeneratorOctaves = NoiseGeneratorOctaves::new(&random, 16);
        let selector: NoiseGeneratorOctaves = NoiseGeneratorOctaves::new(&random, 8);
        let stone: Octaves = Octaves::new(&random, 4);
        let mut parabolic: [f32; 25] = [0.0; 25];
        for j in -2..=2_i32
        {
            for k in -2..=2_i32
            {
                parabolic[(j + 2 + (k + 2) * 5) as usize] = 10.0 / ((j * j + k * k) as f32 + 0.2).sqrt();
            }
        }
        Noises
        {
            m_canopy: canopy,
            m_depth: depth,
            m_low: low,
            m_high: high,
            m_selector: selector,
            m_stone: stone,
            m_parabolic: parabolic,
        }
    }
}

thread_local!
{
    static NOISES: RefCell<Option<(i64, Noises)>> = const { RefCell::new(None) };
}

fn withNoises<T>(seed: i64, run: impl FnOnce(&Noises) -> T) -> T
{
    NOISES.with(|noises|
    {
        let mut noises = noises.borrow_mut();
        if noises.as_ref().is_none_or(|(known, _)| *known != seed)
        {
            *noises = Some((seed, Noises::new(seed)));
        }
        run(&noises.as_ref().expect("noises are built for the seed").1)
    })
}

// the chunk while it is shaped, 256 tall as the source works on it
struct Tall
{
    m_blocks: Vec<u16>,
    m_meta: Vec<u8>,
}

impl Tall
{
    fn new() -> Tall { Tall { m_blocks: vec![0; 16 * 16 * TALL], m_meta: vec![0; 16 * 16 * TALL] } }

    fn index(x: i32, y: i32, z: i32) -> usize { ((x * 16 + z) as usize) * TALL + y as usize }

    fn get(&self, x: i32, y: i32, z: i32) -> i32
    {
        if !(0..TALL as i32).contains(&y)
        {
            return 0;
        }
        i32::from(self.m_blocks[Self::index(x, y, z)])
    }

    fn set(&mut self, x: i32, y: i32, z: i32, block: i32, meta: i32)
    {
        if !(0..TALL as i32).contains(&y)
        {
            return;
        }
        let index: usize = Self::index(x, y, z);
        self.m_blocks[index] = block as u16;
        self.m_meta[index] = meta as u8;
    }
}

// the block ids this chunk uses, numeric ids are only known once the world runs
struct Palette
{
    m_stone: i32,
    m_water: i32,
    m_ice: i32,
    m_gravel: i32,
    m_bedrock: i32,
    m_grass: i32,
    m_dirt: i32,
    m_sand: i32,
    m_sandstone: i32,
    m_darkLeaves: i32,
    m_wispy: i32,
    m_fluffy: i32,
    m_weathered: i32,
    m_cracked: i32,
    m_solid: i32,
}

impl Palette
{
    fn load() -> Palette
    {
        Palette
        {
            m_stone: ids::blockId("minecraft:stone"),
            m_water: ids::blockId("minecraft:water"),
            m_ice: ids::blockId("minecraft:ice"),
            m_gravel: ids::blockId("minecraft:gravel"),
            m_bedrock: ids::blockId("minecraft:bedrock"),
            m_grass: ids::blockId("minecraft:grass"),
            m_dirt: ids::blockId("minecraft:dirt"),
            m_sand: ids::blockId("minecraft:sand"),
            m_sandstone: ids::blockId("minecraft:sandstone"),
            m_darkLeaves: ids::blockId(block::DARK_LEAVES),
            m_wispy: ids::blockId(block::WISPY_CLOUD),
            m_fluffy: ids::blockId(block::FLUFFY_CLOUD),
            m_weathered: ids::blockId(block::DEADROCK),
            m_cracked: ids::blockId(block::CRACKED_DEADROCK),
            m_solid: ids::blockId(block::SOLID_DEADROCK),
        }
    }
}

// MathHelper.denormalizeClamp
fn clampedLerp(low: f64, high: f64, delta: f64) -> f64
{
    if delta < 0.0
    {
        low
    }
    else if delta > 1.0
    {
        high
    }
    else
    {
        low + (high - low) * delta
    }
}

// ChunkProviderTwilightForest.makeLandPerBiome2, a 5x33x5 density grid from the biome heights
fn density(noises: &Noises, biomes: &[Biome], x: i32, z: i32) -> Vec<f64>
{
    let depth: Vec<f64> = noises.m_depth.getRegion(None, x, z, 5, 5, 200.0, 200.0, 0.5);
    let selector: Vec<f64> = noises.m_selector.generateNoiseOctaves(
        None,
        f64::from(x),
        0.0,
        f64::from(z),
        5,
        33,
        5,
        8.555150000000001,
        4.277575000000001,
        8.555150000000001,
    );
    let low: Vec<f64> =
        noises.m_low.generateNoiseOctaves(None, f64::from(x), 0.0, f64::from(z), 5, 33, 5, 684.412, 684.412, 684.412);
    let high: Vec<f64> =
        noises.m_high.generateNoiseOctaves(None, f64::from(x), 0.0, f64::from(z), 5, 33, 5, 684.412, 684.412, 684.412);
    let mut values: Vec<f64> = Vec::with_capacity(5 * 33 * 5);
    let mut noiseIndex: usize = 0;
    for ax in 0..5_i32
    {
        for az in 0..5_i32
        {
            let center: &biomes::BiomeRow = biomes::rowOf(biomes[(ax + 2 + (az + 2) * 10) as usize]);
            let (mut variation, mut height, mut total): (f32, f32, f32) = (0.0, 0.0, 0.0);
            for ox in -2..=2_i32
            {
                for oz in -2..=2_i32
                {
                    let near: &biomes::BiomeRow = biomes::rowOf(biomes[(ax + ox + 2 + (az + oz + 2) * 10) as usize]);
                    let (root, spread): (f32, f32) = near.m_height;
                    let mut factor: f32 = noises.m_parabolic[(ox + 2 + (oz + 2) * 5) as usize] / (root + 2.0);
                    if root > center.m_height.0
                    {
                        factor /= 2.0;
                    }
                    variation += spread * factor;
                    height += root * factor;
                    total += factor;
                }
            }
            variation = variation / total * 0.9 + 0.1;
            height = (height / total * 4.0 - 1.0) / 8.0;
            let mut terrain: f64 = depth[noiseIndex] / 8000.0;
            noiseIndex += 1;
            if terrain < 0.0
            {
                terrain = -terrain * 0.3;
            }
            terrain = terrain * 3.0 - 2.0;
            if terrain < 0.0
            {
                terrain = (terrain / 2.0).max(-1.0) / 1.4 / 2.0;
            }
            else
            {
                terrain = terrain.min(1.0) / 8.0;
            }
            let heightCalc: f64 = (f64::from(height) + terrain * 0.2) * 8.5 / 8.0;
            let base: f64 = 8.5 + heightCalc * 4.0;
            for ay in 0..33
            {
                let index: usize = values.len();
                let mut falloff: f64 = (f64::from(ay) - base) * 12.0 * 128.0 / 256.0 / f64::from(variation);
                if falloff < 0.0
                {
                    falloff *= 4.0;
                }
                let lower: f64 = low[index] / 512.0;
                let upper: f64 = high[index] / 512.0;
                let blend: f64 = (selector[index] / 10.0 + 1.0) / 2.0;
                let mut value: f64 = clampedLerp(lower, upper, blend) - falloff;
                if ay > 29
                {
                    let top: f64 = f64::from(ay - 29) / 3.0;
                    value = value * (1.0 - top) + -10.0 * top;
                }
                values.push(value);
            }
        }
    }
    values
}

// ChunkProviderTwilightForest.generateTerrain2, trilinear fill of stone and sea from the density grid
fn shape(tall: &mut Tall, palette: &Palette, grid: &[f64])
{
    for k in 0..4_usize
    {
        for j1 in 0..4_usize
        {
            let c00: usize = (k * 5 + j1) * 33;
            let c01: usize = (k * 5 + j1 + 1) * 33;
            let c10: usize = ((k + 1) * 5 + j1) * 33;
            let c11: usize = ((k + 1) * 5 + j1 + 1) * 33;
            for k2 in 0..32_usize
            {
                let eighth: f64 = 0.125;
                let mut d1: f64 = grid[c00 + k2];
                let mut d2: f64 = grid[c01 + k2];
                let mut d3: f64 = grid[c10 + k2];
                let mut d4: f64 = grid[c11 + k2];
                let d5: f64 = (grid[c00 + k2 + 1] - d1) * eighth;
                let d6: f64 = (grid[c01 + k2 + 1] - d2) * eighth;
                let d7: f64 = (grid[c10 + k2 + 1] - d3) * eighth;
                let d8: f64 = (grid[c11 + k2 + 1] - d4) * eighth;
                for l2 in 0..8_usize
                {
                    let quarter: f64 = 0.25;
                    let mut d10: f64 = d1;
                    let mut d11: f64 = d2;
                    let d12: f64 = (d3 - d1) * quarter;
                    let d13: f64 = (d4 - d2) * quarter;
                    let y: i32 = (k2 * 8 + l2) as i32;
                    for i3 in 0..4_usize
                    {
                        let x: i32 = (i3 + k * 4) as i32;
                        let d16: f64 = (d11 - d10) * quarter;
                        let mut d15: f64 = d10 - d16;
                        for k3 in 0..4_usize
                        {
                            let z: i32 = (j1 * 4 + k3) as i32;
                            d15 += d16;
                            let block: i32 = if d15 > 0.0
                            {
                                palette.m_stone
                            }
                            else if y < SHAPE_SEA_LEVEL
                            {
                                palette.m_water
                            }
                            else
                            {
                                0
                            };
                            tall.set(x, y, z, block, 0);
                        }
                        d10 += d12;
                        d11 += d13;
                    }
                    d1 += d5;
                    d2 += d6;
                    d3 += d7;
                    d4 += d8;
                }
            }
        }
    }
}

// ChunkProviderTwilightForest.squishTerrain keeps every second block, halving the height
fn squish(tall: &mut Tall)
{
    let half: i32 = TALL as i32 / 2;
    for x in 0..16
    {
        for z in 0..16
        {
            for y in 0..TALL as i32
            {
                let block: i32 = if y < half
                {
                    tall.get(x, y * 2 + 1, z)
                }
                else
                {
                    0
                };
                tall.set(x, y, z, block, 0);
            }
        }
    }
}

fn darkBiome(biome: Biome) -> bool { matches!(biome, Biome::DarkForest | Biome::DarkForestCenter) }

// ChunkProviderTwilightForest.addDarkForestCanopy2, a roof of dark leaves wherever dark forest biomes crowd
fn canopy(
    tall: &mut Tall,
    palette: &Palette,
    noises: &Noises,
    coarse: &[Biome],
    chunkX: i32,
    chunkZ: i32,
    near: (Feature, [i32; 2]),
)
{
    let mut thicks: [i32; 25] = [0; 25];
    for z in 0..5_i32
    {
        for x in 0..5_i32
        {
            for bx in -1..=1
            {
                for bz in -1..=1
                {
                    if darkBiome(coarse[(x + bx + 2 + (z + bz + 2) * 10) as usize])
                    {
                        thicks[(x + z * 5) as usize] += 1;
                    }
                }
            }
        }
    }
    if thicks.iter().all(|thick| *thick == 0)
    {
        return;
    }
    let scale: f64 = 0.03125 * 2.0;
    let noise: Vec<f64> = noises.m_canopy.generateNoiseOctaves(
        None,
        f64::from(chunkX * 16),
        0.0,
        f64::from(chunkZ * 16),
        16,
        1,
        16,
        scale,
        scale,
        scale,
    );
    for z in 0..16_i32
    {
        for x in 0..16_i32
        {
            let (qx, qz): (i32, i32) = (x / 4, z / 4);
            let xWeight: f32 = (x % 4) as f32 * 0.25 + 0.125;
            let zWeight: f32 = (z % 4) as f32 * 0.25 + 0.125;
            let thick = |dx: i32, dz: i32| thicks[(qx + dx + (qz + dz) * 5) as usize] as f32;
            let mut thickness: f32 = thick(0, 0) * (1.0 - xWeight) * (1.0 - zWeight)
                + thick(1, 0) * xWeight * (1.0 - zWeight)
                + thick(0, 1) * (1.0 - xWeight) * zWeight
                + thick(1, 1) * xWeight * zWeight
                - 4.0;
            if near.0 == Feature::DarkTower
            {
                let (dx, dz): (i32, i32) = (x - near.1[0], z - near.1[1]);
                let clearing: i32 = 24;
                let distance: i32 = f64::from(dx * dx + dz * dz).sqrt() as i32;
                if distance < clearing
                {
                    thickness -= (clearing - distance) as f32;
                }
            }
            if thickness <= 1.0
            {
                continue;
            }
            let Some(top) = (0..HEIGHT)
                .rev()
                .map(|y| (y, tall.get(x, y, z)))
                .find(|(_, block)| *block == palette.m_water || *block == palette.m_stone)
            else
            {
                continue;
            };
            if top.1 == palette.m_water
            {
                continue;
            }
            let wobble: i32 = ((noise[((x & 15) << 4 | (z & 15)) as usize] / 1.25) as i32).min(3);
            let mut bottom: i32 = top.0 + 12 - (thickness * 0.5) as i32;
            let roof: i32 = bottom + (thickness * 1.5) as i32;
            bottom -= wobble;
            for y in bottom..roof
            {
                tall.set(x, y, z, palette.m_darkLeaves, 0);
            }
        }
    }
}

// ChunkProviderTwilightForest.addGlaciers, gravel under a wall of ice 32 high
fn glaciers(tall: &mut Tall, palette: &Palette, columns: &biomes::Columns)
{
    let thickness: i32 = 32;
    for z in 0..16_i32
    {
        for x in 0..16_i32
        {
            if biomes::fromIndex(i32::from(columns[(x | z << 4) as usize])) != Biome::Glacier
            {
                continue;
            }
            let top: i32 = (0..HEIGHT).rev().find(|y| tall.get(x, *y, z) == palette.m_stone).unwrap_or(-1);
            if top >= 0
            {
                tall.set(x, top, z, palette.m_gravel, 0);
            }
            for y in top + 1..=(top + thickness + 1).min(HEIGHT - 1)
            {
                tall.set(x, y, z, palette.m_ice, 0);
            }
        }
    }
}

// ChunkProviderTwilightForest.raiseHills, a mound of stone hollowed out underneath
fn raiseHills(
    tall: &mut Tall,
    palette: &Palette,
    feature: Feature,
    diameter: i32,
    x: i32,
    z: i32,
    offset: (i32, i32),
    hill: i32,
)
{
    let size: i32 = features::row(feature).m_size;
    let mut ground: Option<i32> = None;
    for y in 0..TALL as i32
    {
        if ground.is_none() && tall.get(x, y, z) != palette.m_stone
        {
            ground = Some(y + hill);
        }
        if ground.is_some_and(|top| y <= top)
        {
            tall.set(x, y, z, palette.m_stone, 0);
        }
    }
    let mut hollow: i32 = hill - 4 - size;
    if feature == Feature::HydraLair
    {
        let (mx, mz): (i32, i32) = (offset.0 + 16, offset.1 + 16);
        let distance: f64 = f64::from(mx * mx + mz * mz).sqrt().floor();
        let reach: f64 = f64::from(diameter) / 1.5;
        let mound: i32 = ((distance / reach * std::f64::consts::PI).cos() * reach) as i32;
        hollow = hollow.max(mound - 4);
    }
    hollow = hollow.max(0);
    let floor: i32 = if feature == Feature::HydraLair
    {
        SEA_LEVEL
    }
    else
    {
        SEA_LEVEL - 3 - hollow / 8
    };
    for y in 0..TALL as i32
    {
        if hill > 0 && y < SEA_LEVEL && tall.get(x, y, z) != palette.m_stone
        {
            tall.set(x, y, z, palette.m_stone, 0);
        }
        if y > floor && y < floor + hollow
        {
            tall.set(x, y, z, 0, 0);
        }
    }
}

// how far outside a feature's square edge this column lies, in eighths of a block, ChunkProviderTwilightForest
fn squishFactor(dx: i32, dz: i32, boundary: i32) -> f32
{
    let edge = |value: i32|
    {
        if value.abs() >= boundary
        {
            (value.abs() - boundary) as f32 / 8.0
        }
        else
        {
            0.0
        }
    };
    edge(dx).max(edge(dz))
}

fn firstAir(tall: &Tall, palette: &Palette, x: i32, z: i32) -> Option<i32>
{
    (0..HEIGHT).find(|y| tall.get(x, *y, z) != palette.m_stone)
}

// ChunkProviderTwilightForest.flattenTerrainForFeature, the maze and courtyard sit on a level floor
fn flatten(tall: &mut Tall, palette: &Palette, size: i32, x: i32, z: i32, dx: i32, dz: i32)
{
    let boundary: i32 = (size * 2 + 1) * 8 - 8;
    let squish: f32 = squishFactor(dx, dz, boundary);
    let mut level: i32 = SEA_LEVEL + 1;
    if squish > 0.0
        && let Some(ground) = firstAir(tall, palette, x, z)
    {
        level = (level as f32 + (ground - level) as f32 * squish) as i32;
    }
    for y in 0..HEIGHT
    {
        let block: i32 = tall.get(x, y, z);
        if y < level && (block == 0 || block == palette.m_water)
        {
            tall.set(x, y, z, palette.m_stone, 0);
        }
        if y >= level && block != palette.m_water
        {
            tall.set(x, y, z, 0, 0);
        }
    }
}

// ChunkProviderTwilightForest.deformTerrainForYetiLair, a hill with a cave under a packed ice floor
fn yetiLair(tall: &mut Tall, palette: &Palette, size: i32, x: i32, z: i32, dx: i32, dz: i32)
{
    let outer: i32 = (size * 2 + 1) * 8 - 8;
    let squish: f32 = squishFactor(dx, dz, outer);
    let cave: i32 = size * 2 * 8 - 8;
    let offset: i32 = dx.abs().min(dz.abs());
    let mut ceiling: i32 = if (-cave..=cave).contains(&dx) && (-cave..=cave).contains(&dz)
    {
        SEA_LEVEL + 16
    }
    else
    {
        SEA_LEVEL + 40 - offset * 4
    };
    ceiling -= offset / 6;
    ceiling = ceiling.min(SEA_LEVEL + 16);
    let mut top: i32 = SEA_LEVEL + 24;
    let mut floor: i32 = SEA_LEVEL - 1 + offset / 6;
    if squish > 0.0
        && let Some(ground) = firstAir(tall, palette, x, z)
    {
        top = (top as f32 + (ground - top) as f32 * squish) as i32;
        floor = (floor as f32 + (ground - floor) as f32 * squish) as i32;
    }
    for y in 0..HEIGHT
    {
        let block: i32 = tall.get(x, y, z);
        if y < top && (block == 0 || block == palette.m_water)
        {
            tall.set(x, y, z, palette.m_stone, 0);
        }
        if y > floor && y < ceiling
        {
            tall.set(x, y, z, 0, 0);
        }
        if y == floor && y < ceiling && y < SEA_LEVEL + 3
        {
            tall.set(x, y, z, palette.m_ice, 0);
        }
    }
}

// ChunkProviderTwilightForest.deformTerrainForTrollCloud2, a cloud layer floats over the troll caves
// the source puts it at 166, far above beta's 128 world top, so it floats just under the ceiling instead
fn trollClouds(tall: &mut Tall, palette: &Palette, chunkX: i32, center: [i32; 2], random: &Random)
{
    let cloudTop: i32 = HEIGHT - 8;
    for bx in 0..4
    {
        for bz in 0..4
        {
            let dx: i32 = bx * 4 - center[0] - 2;
            let dz: i32 = bz * 4 - center[1] - 2;
            // the source takes the region from chunkX twice, kept so the clouds match
            let region: i32 = (chunkX + 8) >> 4;
            let mut hash: i64 = i64::from(region).wrapping_mul(3129871) ^ i64::from(region).wrapping_mul(116129781);
            hash = hash.wrapping_mul(hash).wrapping_mul(42317861).wrapping_add(hash.wrapping_mul(7));
            let bits = |shift: i64| ((hash >> shift) & 3) as i32;
            let (dx2, dz2): (i32, i32) = (dx + bits(12) * 5 - bits(15) * 4, dz + bits(18) * 4 - bits(21) * 5);
            let (dx3, dz3): (i32, i32) = (dx + bits(9) * 5 - bits(6) * 4, dz + bits(3) * 4 - bits(0) * 5);
            let length = |a: i32, b: i32| f64::from(a * a + b * b).sqrt();
            let distance: f64 = (length(dx, dz) / 4.0).min((length(dx2, dz2) / 3.5).min(length(dx3, dz3) / 4.5));
            let roll: f32 = random.nextFloat();
            let edge: f64 = distance - 7.0 - f64::from(roll) * 3.0;
            let y: i32 = if roll < 0.1 { cloudTop + 1 } else { cloudTop };
            let depth: i32 = 4 + i32::from(roll > 0.6) + i32::from(roll > 0.9);
            for sx in 0..4
            {
                for sz in 0..4
                {
                    let (x, z): (i32, i32) = (bx * 4 + sx, bz * 4 + sz);
                    if distance < 7.0 || edge < 0.05
                    {
                        tall.set(x, y, z, palette.m_wispy, 0);
                        for d in 1..depth
                        {
                            tall.set(x, y - d, z, palette.m_fluffy, 0);
                        }
                        tall.set(x, y - depth, z, palette.m_wispy, 0);
                    }
                    else if distance < 8.0 || edge < 1.0
                    {
                        for d in 1..depth
                        {
                            tall.set(x, y - d, z, palette.m_wispy, 0);
                        }
                    }
                }
            }
        }
    }
}

fn deform(tall: &mut Tall, palette: &Palette, chunkX: i32, near: (Feature, [i32; 2]), random: &Random)
{
    let (feature, center): (Feature, [i32; 2]) = near;
    if !features::row(feature).m_terrain
    {
        return;
    }
    let size: i32 = features::row(feature).m_size;
    if feature == Feature::TrollCave
    {
        trollClouds(tall, palette, chunkX, center, random);
    }
    for x in 0..16
    {
        for z in 0..16
        {
            let (dx, dz): (i32, i32) = (x - center[0], z - center[1]);
            match feature
            {
                Feature::SmallHill | Feature::MediumHill | Feature::LargeHill | Feature::HydraLair =>
                {
                    let diameter: i32 = (size * 2 + 1) * 16;
                    let distance: f64 = f64::from(dx * dx + dz * dz).sqrt().floor();
                    let hill: i32 = ((distance / f64::from(diameter) * std::f64::consts::PI).cos()
                        * (f64::from(diameter) / 3.0)) as i32;
                    raiseHills(tall, palette, feature, diameter, x, z, (dx, dz), hill);
                }
                Feature::HedgeMaze | Feature::NagaCourtyard | Feature::QuestGrove =>
                {
                    flatten(tall, palette, size, x, z, dx, dz)
                }
                Feature::YetiCave => yetiLair(tall, palette, size, x, z, dx, dz),
                _ =>
                {}
            }
        }
    }
}

// TFBiomeBase.genTwilightBiomeTerrain, the surface and filler layers over stone and the bedrock floor
fn surface(tall: &mut Tall, palette: &Palette, biome: Biome, random: &Random, x: i32, z: i32, noise: f64)
{
    let sea: i32 = SEA_LEVEL + 1;
    let deadrock: bool = matches!(biome, Biome::Thornlands | Biome::FinalPlateau);
    let (topBlock, fillerBlock): ((i32, i32), (i32, i32)) = match biome
    {
        _ if deadrock => ((palette.m_weathered, 0), (palette.m_cracked, 0)),
        // TFBiomeHighlands picks coarse dirt or podzol by noise, beta has neither so dirt and grass stand in
        Biome::Highlands if noise > 1.75 => ((palette.m_dirt, 0), (palette.m_dirt, 0)),
        _ => ((palette.m_grass, 0), (palette.m_dirt, 0)),
    };
    let (mut top, mut filler): ((i32, i32), (i32, i32)) = (topBlock, fillerBlock);
    let mut depth: i32 = -1;
    let maxDepth: i32 = (noise / 3.0 + 3.0 + random.nextDouble() * 0.25) as i32;
    let temperature: f32 = biomes::rowOf(biome).m_temperature;
    for y in (0..TALL as i32).rev()
    {
        if y <= random.nextIntBounded(5)
        {
            tall.set(x, y, z, palette.m_bedrock, 0);
            continue;
        }
        if tall.get(x, y, z) != palette.m_stone
        {
            continue;
        }
        if deadrock
        {
            tall.set(x, y, z, palette.m_solid, 0);
        }
        if depth == -1
        {
            if maxDepth <= 0
            {
                top = (0, 0);
                filler = (palette.m_stone, 0);
            }
            else if y >= sea - 5 && y <= sea
            {
                top = topBlock;
                filler = fillerBlock;
            }
            if y < sea - 1 && top.0 == 0
            {
                top = if temperature < 0.15
                {
                    (palette.m_ice, 0)
                }
                else
                {
                    (palette.m_water, 0)
                };
            }
            depth = maxDepth;
            if y >= sea - 2
            {
                tall.set(x, y, z, top.0, top.1);
            }
            else if y < sea - 8 - maxDepth
            {
                top = (0, 0);
                filler = (palette.m_stone, 0);
                tall.set(x, y, z, palette.m_gravel, 0);
            }
            else
            {
                tall.set(x, y, z, filler.0, filler.1);
            }
        }
        else if depth > 0
        {
            depth -= 1;
            tall.set(x, y, z, filler.0, filler.1);
            if depth == 0 && filler.0 == palette.m_sand
            {
                depth = random.nextIntBounded(4) + (y - (sea - 1)).max(0);
                filler = (palette.m_sandstone, 0);
            }
        }
    }
}

fn surfaces(
    tall: &mut Tall,
    palette: &Palette,
    noises: &Noises,
    columns: &biomes::Columns,
    chunkX: i32,
    chunkZ: i32,
    random: &Random,
)
{
    let scale: f64 = 0.03125 * 2.0;
    let lacunarity: f64 = 1.0;
    let stone: Vec<f64> =
        noises.m_stone.region(f64::from(chunkX * 16), f64::from(chunkZ * 16), 16, 16, [scale, scale], lacunarity);
    for z in 0..16_i32
    {
        for x in 0..16_i32
        {
            let biome: Biome = biomes::fromIndex(i32::from(columns[(x + z * 16) as usize]));
            surface(tall, palette, biome, random, x, z, stone[(x + z * 16) as usize]);
        }
    }
}

// ChunkProviderTwilightForest.provideChunk
pub fn generate(chunk: &ChunkInfo, blocks: &mut ChunkBlocks)
{
    let palette: Palette = Palette::load();
    let random: Random = Random::withSeed(
        i64::from(chunk.x).wrapping_mul(341873128712).wrapping_add(i64::from(chunk.z).wrapping_mul(132897987541)),
    );
    let mut tall: Tall = Tall::new();
    let coarse: Vec<Biome> = biomes::coarse(chunk.seed, chunk.x * 4 - 2, chunk.z * 4 - 2, 10, 10);
    let near: (Feature, [i32; 2]) = features::nearest(chunk.seed, chunk.x, chunk.z);
    let columns: biomes::Columns = biomes::chunk(chunk.seed, chunk.x, chunk.z);
    withNoises(chunk.seed, |noises|
    {
        let grid: Vec<f64> = density(noises, &coarse, chunk.x * 4, chunk.z * 4);
        shape(&mut tall, &palette, &grid);
        squish(&mut tall);
        canopy(&mut tall, &palette, noises, &coarse, chunk.x, chunk.z, near);
        glaciers(&mut tall, &palette, &columns);
        // the source rolls the cloud shapes from the world random, a chunk seeded one keeps them repeatable
        let clouds: Random = Random::withSeed(chunk.seed ^ i64::from(chunk.x).wrapping_mul(31) ^ i64::from(chunk.z));
        deform(&mut tall, &palette, chunk.x, near, &clouds);
        surfaces(&mut tall, &palette, noises, &columns, chunk.x, chunk.z, &random);
    });
    super::caves::carve(chunk, &mut tall.m_blocks, &palette.carvable());
    for x in 0..16
    {
        for z in 0..16
        {
            for y in 0..HEIGHT
            {
                let index: usize = Tall::index(x, y, z);
                blocks.setWithMeta(x, y, z, i32::from(tall.m_blocks[index]), i32::from(tall.m_meta[index]));
            }
        }
    }
}

impl Palette
{
    fn carvable(&self) -> super::caves::Carvable
    {
        super::caves::Carvable
        {
            m_stone: self.m_stone,
            m_dirt: self.m_dirt,
            m_grass: self.m_grass,
            m_water: self.m_water,
            m_flowingWater: ids::blockId("minecraft:flowing_water"),
            m_deadrock: [self.m_weathered, self.m_cracked, self.m_solid],
            m_height: TALL,
        }
    }
}

pub fn height() -> i32 { HEIGHT }
