use crate::names::{block, numeric};
use b173_api::terrain::{ChunkBlocks, NoiseGeneratorOctaves, Random, chunk, structure};
use b173_api::{ChunkInfo, blockIs};
use std::cell::{RefCell, RefMut};

mod noise
{
    pub const CELLS: usize = 3;
    pub const LAYERS: usize = 33;
    pub const SIZE: usize = CELLS * CELLS * LAYERS;
    pub const HORIZONTAL: f64 = 1368.824;
    pub const VERTICAL: f64 = 684.412;
    pub const PERLIN_HORIZONTAL_DIVISOR: f64 = 80.0;
    pub const PERLIN_VERTICAL_DIVISOR: f64 = 160.0;
    pub const NOISE1_OCTAVES: i32 = 16;
    pub const PERLIN1_OCTAVES: i32 = 8;
    pub const AMPLITUDE: f64 = 512.0;
    pub const BIAS: f64 = 8.0;
    pub const BLEND_SCALE: f64 = 10.0;
    // density the top and bottom layers fade to, which keeps islands off the world edges
    pub const EDGE: f64 = -30.0;
    pub const TOP_FADE_LAYERS: f32 = 31.0;
    pub const BOTTOM_FADE_START: usize = 8;
    pub const BOTTOM_FADE_LAYERS: f32 = 7.0;
}

mod cell
{
    pub const PER_AXIS: usize = 2;
    pub const WIDTH: usize = 8;
    pub const HEIGHT: usize = 4;
    pub const STACK: usize = 32;
}

mod surface
{
    pub const DEPTH: f64 = 3.0;
    pub const DEPTH_JITTER: f64 = 0.25;
}

mod quicksoil
{
    pub const CHANCE: i32 = 10;
    pub const FIRST: i32 = 3;
    pub const LAST: i32 = 12;
    pub const LOWEST: i32 = 3;
    pub const HIGHEST: i32 = 48;
    pub const RADIUS: i32 = 3;
    pub const RADIUS_SQUARED: i32 = 12;
}

mod cloud
{
    // MapGenBase.range, chunks searched around the generated one for structure starts
    pub const RANGE: i32 = 8;
    pub const CHANCE: i32 = 50;
    pub const PUFFS: usize = 64;
    pub const EXTENT: i32 = 100;
    pub const BOX_INSET: i32 = 2;
    pub const ORIGIN: [i32; 3] = [50, 4, 100];
    pub const RISE_CHANCE: i32 = 10;
    pub const MIN_WIDTH: i32 = 9;
    pub const WIDTH_SPREAD: i32 = 4;
    pub const HEIGHT: i32 = 2;
    pub const REACH: i32 = 12;
    pub const SHAPES: i32 = 2;
}

// ChunkProviderAether.provideChunk seeds its random with these
mod seed
{
    pub const X: i64 = 341873128712;
    pub const Z: i64 = 132897987541;
}

// ChunkProviderAether noise fields, built once per world seed in the order the Java constructor draws them
struct Noise
{
    m_seed: i64,
    m_noise1: NoiseGeneratorOctaves,
    m_perlin1: NoiseGeneratorOctaves,
}

thread_local!
{
    static NOISE: RefCell<Option<Noise>> = const { RefCell::new(None) };
}

fn withNoise<R>(seed: i64, body: impl FnOnce(&Noise) -> R) -> R
{
    NOISE.with(|slot|
    {
        let mut slot: RefMut<Option<Noise>> = slot.borrow_mut();
        if slot.as_ref().is_none_or(|noise| noise.m_seed != seed)
        {
            let random: Random = Random::withSeed(seed);
            let noise1: NoiseGeneratorOctaves = NoiseGeneratorOctaves::new(&random, noise::NOISE1_OCTAVES);
            let perlin1: NoiseGeneratorOctaves = NoiseGeneratorOctaves::new(&random, noise::PERLIN1_OCTAVES);
            *slot = Some(Noise { m_seed: seed, m_noise1: noise1, m_perlin1: perlin1 });
        }

        body(slot.as_ref().expect("noise was just built"))
    })
}

fn sample(octaves: &NoiseGeneratorOctaves, x: i32, z: i32, horizontal: f64, vertical: f64) -> Vec<f64>
{
    let size: i32 = noise::CELLS as i32;
    octaves.generateNoiseOctaves(
        None,
        x as f64,
        0.0,
        z as f64,
        size,
        noise::LAYERS as i32,
        size,
        horizontal,
        vertical,
        horizontal,
    )
}

fn fadeTo(value: f64, fade: f64) -> f64 { value * (1.0 - fade) + noise::EDGE * fade }

// ChunkProviderAether.setupNoiseGenerators
fn density(noise: &Noise, x: i32, z: i32) -> Vec<f64>
{
    let pnr: Vec<f64> = sample(
        &noise.m_perlin1,
        x,
        z,
        noise::HORIZONTAL / noise::PERLIN_HORIZONTAL_DIVISOR,
        noise::VERTICAL / noise::PERLIN_VERTICAL_DIVISOR,
    );

    let ar: Vec<f64> = sample(&noise.m_noise1, x, z, noise::HORIZONTAL, noise::VERTICAL);
    let br: Vec<f64> = sample(&noise.m_noise1, x, z, noise::HORIZONTAL, noise::VERTICAL);
    (0..noise::SIZE)
        .map(|id|
        {
            let layer: usize = id % noise::LAYERS;
            let low: f64 = ar[id] / noise::AMPLITUDE;
            let high: f64 = br[id] / noise::AMPLITUDE;
            let blend: f64 = (pnr[id] / noise::BLEND_SCALE + 1.0) * 0.5;
            let mut value: f64 = low + (high - low) * blend.clamp(0.0, 1.0) - noise::BIAS;

            if layer > 1
            {
                value = fadeTo(value, ((layer - 1) as f32 / noise::TOP_FADE_LAYERS) as f64);
            }

            if layer < noise::BOTTOM_FADE_START
            {
                value = fadeTo(value, ((noise::BOTTOM_FADE_START - layer) as f32 / noise::BOTTOM_FADE_LAYERS) as f64);
            }

            value
        })
        .collect()
}

// ChunkProviderAether.setBlocksInChunk, trilinear interpolation of the 3x33x3 density into holystone
fn shape(blocks: &mut ChunkBlocks, buffer: &[f64], holystone: u16)
{
    let at = |cellX: usize, cellZ: usize, layer: usize| buffer[(cellX * noise::CELLS + cellZ) * noise::LAYERS + layer];
    let vertical: f64 = 1.0 / cell::HEIGHT as f64;
    let horizontal: f64 = 1.0 / cell::WIDTH as f64;

    for cellX in 0..cell::PER_AXIS
    {
        for cellZ in 0..cell::PER_AXIS
        {
            for layer in 0..cell::STACK
            {
                let mut corners: [f64; 4] = [
                    at(cellX, cellZ, layer),
                    at(cellX, cellZ + 1, layer),
                    at(cellX + 1, cellZ, layer),
                    at(cellX + 1, cellZ + 1, layer),
                ];

                let rise: [f64; 4] = [
                    (at(cellX, cellZ, layer + 1) - corners[0]) * vertical,
                    (at(cellX, cellZ + 1, layer + 1) - corners[1]) * vertical,
                    (at(cellX + 1, cellZ, layer + 1) - corners[2]) * vertical,
                    (at(cellX + 1, cellZ + 1, layer + 1) - corners[3]) * vertical,
                ];

                for step in 0..cell::HEIGHT
                {
                    let mut near: f64 = corners[0];
                    let mut far: f64 = corners[1];
                    let nearStep: f64 = (corners[2] - corners[0]) * horizontal;
                    let farStep: f64 = (corners[3] - corners[1]) * horizontal;

                    for x in 0..cell::WIDTH
                    {
                        let mut value: f64 = near;
                        let zStep: f64 = (far - near) * horizontal;

                        for z in 0..cell::WIDTH
                        {
                            let index: usize = ChunkBlocks::index(
                                (x + cellX * cell::WIDTH) as i32,
                                (layer * cell::HEIGHT + step) as i32,
                                (z + cellZ * cell::WIDTH) as i32,
                            );
                            blocks.m_blocks[index] = if value > 0.0 { holystone } else { 0 };
                            value += zStep;
                        }

                        near += nearStep;
                        far += farStep;
                    }

                    for (corner, delta) in corners.iter_mut().zip(rise)
                    {
                        *corner += delta;
                    }
                }
            }
        }
    }
}

// the ids a chunk buffer stores for the island layers, resolved once per chunk
struct Layers
{
    m_stone: u16,
    m_top: u16,
    m_filler: u16,
}

fn stored(name: &'static str) -> u16 { numeric(name) as u16 }

// ChunkProviderAether.buildSurfaces, z outside and x inside as the Java draws its randoms
fn surface(blocks: &mut ChunkBlocks, random: &Random, layers: &Layers)
{
    for z in 0..chunk::WIDTH
    {
        for x in 0..chunk::WIDTH
        {
            let mut depth: i32 = -1;
            let thickness: i32 = (surface::DEPTH + random.nextDouble() * surface::DEPTH_JITTER) as i32;
            let mut top: u16 = layers.m_top;
            let mut filler: u16 = layers.m_filler;

            for y in (0..chunk::HEIGHT).rev()
            {
                let index: usize = ChunkBlocks::index(x, y, z);
                let block: u16 = blocks.m_blocks[index];
                if block == 0
                {
                    depth = -1;
                }
                // shape fills only stone and air so every solid block here is stone
                else
                {
                    if depth == -1
                    {
                        if thickness <= 0
                        {
                            top = 0;
                            filler = layers.m_stone;
                        }

                        depth = thickness;
                        blocks.m_blocks[index] = top;
                    }
                    else if depth > 0
                    {
                        depth -= 1;
                        blocks.m_blocks[index] = filler;
                    }
                }
            }
        }
    }
}

// MapGenQuicksoil, a quicksoil disc under grass overhangs; its loops name the array x axis z as the Java does,
// and the Java draws from an unseeded random where the port uses the chunk random so every side agrees
fn quicksoil(blocks: &mut ChunkBlocks, random: &Random)
{
    if random.nextIntBounded(quicksoil::CHANCE) != 0
    {
        return;
    }

    for x in quicksoil::FIRST..quicksoil::LAST
    {
        for z in quicksoil::FIRST..quicksoil::LAST
        {
            let found: Option<i32> = (quicksoil::LOWEST..quicksoil::HIGHEST).find(|y|
            {
                blocks.get(z, *y, x) == 0
                    && blockIs(blocks.get(z, y + 1, x), block::AETHER_GRASS)
                    && blocks.get(z, y + 2, x) == 0
            });
            if let Some(y) = found
            {
                quicksoilDisc(blocks, x, y, z, i32::from(stored(block::QUICKSOIL)));
            }
        }
    }
}

fn quicksoilDisc(blocks: &mut ChunkBlocks, posX: i32, posY: i32, posZ: i32, quicksoilBlock: i32)
{
    for x in (posX - quicksoil::RADIUS)..=(posX + quicksoil::RADIUS)
    {
        for z in (posZ - quicksoil::RADIUS)..=(posZ + quicksoil::RADIUS)
        {
            let inside: bool = (x - posX) * (x - posX) + (z - posZ) * (z - posZ) < quicksoil::RADIUS_SQUARED;
            if inside && blocks.get(z, posY, x) == 0
            {
                blocks.set(z, posY, x, quicksoilBlock);
            }
        }
    }
}

struct Puff
{
    m_offset: [i32; 3],
    m_size: [i32; 3],
    m_shape: i32,
}

// one of -1, 0 or 1
fn drift(random: &Random) -> i32 { random.nextIntBounded(3) - 1 }

fn puff(random: &Random) -> Puff
{
    let xOffset: i32 = drift(random);
    let yOffset: i32 = drift(random);
    let zOffset: i32 = drift(random);
    let rises: bool = random.nextIntBounded(cloud::RISE_CHANCE) == 0;
    let xMax: i32 = random.nextIntBounded(cloud::WIDTH_SPREAD) + cloud::MIN_WIDTH;
    // nextInt(1) is always zero but still advances the random
    let yMax: i32 = random.nextIntBounded(1) + cloud::HEIGHT;
    let zMax: i32 = random.nextIntBounded(cloud::WIDTH_SPREAD) + cloud::MIN_WIDTH;

    Puff
    {
        m_offset: [xOffset, if rises { yOffset } else { 0 }, zOffset],
        m_size: [xMax, yMax, zMax],
        m_shape: random.nextIntBounded(cloud::SHAPES),
    }
}

// ComponentLargeColdAercloud, a 64 puff cold cloud sheet near the bottom of the world clipped to its start box
fn largeCloud(blocks: &mut ChunkBlocks, chunkX: i32, chunkZ: i32, start: [i32; 2], random: &Random, cloudBlock: i32)
{
    let xTendency: i32 = drift(random);
    let zTendency: i32 = drift(random);
    let puffs: Vec<Puff> = (0..cloud::PUFFS).map(|_| puff(random)).collect();
    let minX: i32 = start[0] * chunk::WIDTH + cloud::BOX_INSET;
    let minZ: i32 = start[1] * chunk::WIDTH + cloud::BOX_INSET;
    let origin: [i32; 3] = [minX + cloud::ORIGIN[0], cloud::ORIGIN[1], minZ + cloud::ORIGIN[2]];
    let [mut x, mut y, mut z]: [i32; 3] = [0, 0, 0];

    for puff in &puffs
    {
        x += puff.m_offset[0] + xTendency;
        y += puff.m_offset[1];
        z += puff.m_offset[2] + zTendency;

        for x1 in x..x + puff.m_size[0]
        {
            for y1 in y..y + puff.m_size[1]
            {
                for z1 in z..z + puff.m_size[2]
                {
                    if (x1 - x).abs() + (y1 - y).abs() + (z1 - z).abs() >= cloud::REACH + puff.m_shape
                    {
                        continue;
                    }

                    let worldX: i32 = origin[0] + x1;
                    let worldZ: i32 = origin[2] + z1;
                    let inStart: bool = (minX..=minX + cloud::EXTENT).contains(&worldX)
                        && (minZ..=minZ + cloud::EXTENT).contains(&worldZ);
                    let localX: i32 = worldX - chunkX * chunk::WIDTH;
                    let localZ: i32 = worldZ - chunkZ * chunk::WIDTH;
                    let localY: i32 = origin[1] + y1;
                    if inStart
                        && (0..chunk::WIDTH).contains(&localX)
                        && (0..chunk::WIDTH).contains(&localZ)
                        && blocks.get(localX, localY, localZ) == 0
                    {
                        blocks.set(localX, localY, localZ, cloudBlock);
                    }
                }
            }
        }
    }
}

// MapGenLargeColdAercloud.canSpawnStructureAtCoords: one chunk in fifty starts a cloud sheet
fn largeClouds(blocks: &mut ChunkBlocks, chunkInfo: &ChunkInfo, cloudBlock: i32)
{
    structure::candidates(chunkInfo.seed, chunkInfo.x, chunkInfo.z, cloud::RANGE, |startX, startZ, random|
    {
        if random.nextIntBounded(cloud::CHANCE) == 0
        {
            largeCloud(blocks, chunkInfo.x, chunkInfo.z, [startX, startZ], random, cloudBlock);
        }
    });
}

// ChunkProviderAether.provideChunk
pub(crate) fn generate(chunkInfo: &ChunkInfo, blocks: &mut ChunkBlocks)
{
    let layers: Layers = Layers
    {
        m_stone: stored(block::HOLYSTONE),
        m_top: stored(block::AETHER_GRASS),
        m_filler: stored(block::AETHER_DIRT),
    };
    let random: Random = Random::withSeed(
        (chunkInfo.x as i64).wrapping_mul(seed::X).wrapping_add((chunkInfo.z as i64).wrapping_mul(seed::Z)),
    );

    let buffer: Vec<f64> = withNoise(chunkInfo.seed, |noise|
    {
        density(noise, chunkInfo.x * cell::PER_AXIS as i32, chunkInfo.z * cell::PER_AXIS as i32)
    });
    shape(blocks, &buffer, layers.m_stone);
    surface(blocks, &random, &layers);
    quicksoil(blocks, &random);
    largeClouds(blocks, chunkInfo, i32::from(stored(block::COLD_AERCLOUD)));
}
