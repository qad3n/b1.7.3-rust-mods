use super::biomes::{self, Biome};
use super::features;
use b173_api::prelude::*;
use b173_api::terrain::{Random, math};

// the blocks caves cut through or stop at, numeric ids resolved by the terrain generator
pub struct Carvable
{
    pub m_stone: i32,
    pub m_dirt: i32,
    pub m_grass: i32,
    pub m_water: i32,
    pub m_flowingWater: i32,
    pub m_lava: i32,
    pub m_gravel: i32,
    pub m_sand: i32,
    pub m_sandstone: i32,
    pub m_ice: i32,
    pub m_deadrock: [i32; 3],
    pub m_height: usize,
}

// MapGenBase reaches eight chunks out for caves that wander into this one
const RANGE: i32 = 8;
const PI: f32 = std::f32::consts::PI;

struct Carver<'a>
{
    m_blocks: &'a mut [u16],
    m_ids: &'a Carvable,
    m_trollsteinn: i32,
    m_seed: i64,
    m_chunkX: i32,
    m_chunkZ: i32,
}

impl Carver<'_>
{
    fn index(&self, x: i32, y: i32, z: i32) -> usize { ((x * 16 + z) as usize) * self.m_ids.m_height + y as usize }

    fn at(&self, index: usize) -> i32 { i32::from(self.m_blocks[index]) }

    fn water(&self, block: i32) -> bool { block == self.m_ids.m_water || block == self.m_ids.m_flowingWater }

    fn topBlock(&self, x: i32, z: i32) -> i32
    {
        match biomes::at(self.m_seed, x, z)
        {
            Biome::Thornlands | Biome::FinalPlateau => self.m_ids.m_deadrock[0],
            _ => self.m_ids.m_grass,
        }
    }

    // MapGenCaves water check over the box a step would carve, any water stops the step
    fn wet(&self, minX: i32, maxX: i32, minY: i32, maxY: i32, minZ: i32, maxZ: i32) -> bool
    {
        for x in minX..maxX
        {
            for z in minZ..maxZ
            {
                let mut y: i32 = maxY + 1;
                while y >= minY - 1
                {
                    if (0..self.m_ids.m_height as i32).contains(&y)
                    {
                        if self.water(self.at(self.index(x, y, z)))
                        {
                            return true;
                        }
                        if y != minY - 1 && x != minX && x != maxX - 1 && z != minZ && z != maxZ - 1
                        {
                            y = minY;
                        }
                    }
                    y -= 1;
                }
            }
        }
        false
    }
}

// one tunnel of TFGenCaves.generateCaveNode, walked step by step
struct Tunnel
{
    m_seed: i64,
    m_pos: [f64; 3],
    m_size: f32,
    m_yaw: f32,
    m_pitch: f32,
    m_step: i32,
    m_end: i32,
    m_yScale: f64,
}

fn carveTunnel(carver: &mut Carver, tunnel: Tunnel, highlands: bool)
{
    let Tunnel
    {
        m_seed: seed,
        m_pos: mut pos,
        m_size: mut size,
        m_yaw: mut yaw,
        m_pitch: mut pitch,
        m_step: mut step,
        m_end: mut end,
        m_yScale: yScale,
    } = tunnel;
    let centerX: f64 = f64::from(carver.m_chunkX * 16 + 8);
    let centerZ: f64 = f64::from(carver.m_chunkZ * 16 + 8);
    let (mut yawTurn, mut pitchTurn): (f32, f32) = (0.0, 0.0);
    let random: Random = Random::withSeed(seed);
    let moss: Random = Random::withSeed(seed);
    if highlands && size < 6.0
    {
        size *= 2.5;
    }
    if end <= 0
    {
        let blocks: i32 = RANGE * 16 - 16;
        end = blocks - random.nextIntBounded(blocks / 4);
    }
    let single: bool = step == -1;
    if single
    {
        step = end / 2;
    }
    let branchAt: i32 = random.nextIntBounded(end / 2) + end / 4;
    let steep: bool = random.nextIntBounded(6) == 0;
    while step < end
    {
        let width: f64 = 1.5 + f64::from(math::sin(step as f32 * PI / end as f32) * size);
        let height: f64 = width * yScale;
        let cosPitch: f32 = math::cos(pitch);
        pos[0] += f64::from(math::cos(yaw) * cosPitch);
        pos[1] += f64::from(math::sin(pitch));
        pos[2] += f64::from(math::sin(yaw) * cosPitch);
        pitch *= if steep { 0.92 } else { 0.7 };
        pitch += pitchTurn * 0.1;
        yaw += yawTurn * 0.1;
        pitchTurn *= 0.9;
        yawTurn *= 0.75;
        pitchTurn += (random.nextFloat() - random.nextFloat()) * random.nextFloat() * 2.0;
        yawTurn += (random.nextFloat() - random.nextFloat()) * random.nextFloat() * 4.0;
        if !single && step == branchAt && size > 1.0 && end > 0
        {
            for turn in [-PI / 2.0, PI / 2.0]
            {
                let branch: Tunnel = Tunnel
                {
                    m_seed: random.nextLong(),
                    m_pos: pos,
                    m_size: random.nextFloat() * 0.5 + 0.5,
                    m_yaw: yaw + turn,
                    m_pitch: pitch / 3.0,
                    m_step: step,
                    m_end: end,
                    m_yScale: 1.0,
                };
                carveTunnel(carver, branch, highlands);
            }
            return;
        }
        if single || random.nextIntBounded(4) != 0
        {
            let (dx, dz): (f64, f64) = (pos[0] - centerX, pos[2] - centerZ);
            let left: f64 = f64::from(end - step);
            let reach: f64 = f64::from(size + 2.0 + 16.0);
            if dx * dx + dz * dz - left * left > reach * reach
            {
                return;
            }
            let margin: f64 = 16.0 + width * 2.0;
            if pos[0] >= centerX - margin
                && pos[2] >= centerZ - margin
                && pos[0] <= centerX + margin
                && pos[2] <= centerZ + margin
            {
                if carveStep(carver, &moss, pos, width, height, highlands) && single
                {
                    break;
                }
            }
        }
        step += 1;
    }
}

// carves one step of a tunnel, false when water nearby stops it
fn carveStep(carver: &mut Carver, moss: &Random, pos: [f64; 3], width: f64, height: f64, highlands: bool) -> bool
{
    let originX: i32 = carver.m_chunkX * 16;
    let originZ: i32 = carver.m_chunkZ * 16;
    let minX: i32 = (math::floor(pos[0] - width) - originX - 1).max(0);
    let maxX: i32 = (math::floor(pos[0] + width) - originX + 1).min(16);
    let minY: i32 = (math::floor(pos[1] - height) - 1).max(1);
    let maxY: i32 = (math::floor(pos[1] + height) + 1).min(120);
    let minZ: i32 = (math::floor(pos[2] - width) - originZ - 1).max(0);
    let maxZ: i32 = (math::floor(pos[2] + width) - originZ + 1).min(16);
    if carver.wet(minX, maxX, minY, maxY, minZ, maxZ)
    {
        return false;
    }
    let trollsteinn: i32 = carver.m_trollsteinn;
    for x in minX..maxX
    {
        let fx: f64 = (f64::from(x + originX) + 0.5 - pos[0]) / width;
        for z in minZ..maxZ
        {
            let fz: f64 = (f64::from(z + originZ) + 0.5 - pos[2]) / width;
            if fx * fx + fz * fz >= 1.0
            {
                continue;
            }
            let mut hitGrass: bool = false;
            for y in (minY..maxY).rev()
            {
                let fy: f64 = (f64::from(y) + 0.5 - pos[1]) / height;
                let distance: f64 = fx * fx + fy * fy + fz * fz;
                if fy <= -0.7 || distance >= 20.0
                {
                    continue;
                }
                // MapGenCaves reads one block above the one it measures, kept so tunnels keep their shape
                let index: usize = carver.index(x, y + 1, z);
                let block: i32 = carver.at(index);
                let ids: &Carvable = carver.m_ids;
                if block == ids.m_grass
                {
                    hitGrass = true;
                }
                let carvable: bool =
                    block == ids.m_stone || block == trollsteinn || block == ids.m_dirt || block == ids.m_grass;
                if !carvable
                {
                    continue;
                }
                let replacement: i32 = if distance < 0.85
                {
                    // TF caves flood with water near the bottom instead of lava
                    let floor: i32 = 10;
                    if y < floor { ids.m_water } else { 0 }
                }
                else
                {
                    let wall: i32 = if highlands
                    {
                        if moss.nextIntBounded(6) == 0
                        {
                            trollsteinn
                        }
                        else
                        {
                            ids.m_stone
                        }
                    }
                    else
                    {
                        ids.m_dirt
                    };
                    let placed: i32 = if hitGrass { ids.m_grass } else { wall };
                    hitGrass = false;
                    placed
                };
                carver.m_blocks[index] = replacement as u16;
                if hitGrass && index > 0 && carver.at(index - 1) == ids.m_dirt
                {
                    let top: i32 = carver.topBlock(x + originX, z + originZ);
                    carver.m_blocks[index - 1] = top as u16;
                }
            }
        }
    }
    true
}

// TFGenCaves.func_151538_a, the caves one source chunk sends out
fn caves(carver: &mut Carver, random: &Random, sourceX: i32, sourceZ: i32)
{
    let mut count: i32 = random.nextIntBounded(random.nextIntBounded(random.nextIntBounded(40) + 1) + 1);
    if random.nextIntBounded(15) != 0
    {
        count = 0;
    }
    if count == 0
    {
        return;
    }
    let highlands: bool = biomes::at(carver.m_seed, sourceX * 16, sourceZ * 16) == Biome::Highlands;
    for _ in 0..count
    {
        let start: [f64; 3] = [
            f64::from(sourceX * 16 + random.nextIntBounded(16)),
            f64::from(random.nextIntBounded(random.nextIntBounded(120) + 8)),
            f64::from(sourceZ * 16 + random.nextIntBounded(16)),
        ];
        let mut tunnels: i32 = 1;
        if random.nextIntBounded(4) == 0
        {
            let room: Tunnel = Tunnel
            {
                m_seed: random.nextLong(),
                m_pos: start,
                m_size: 1.0 + random.nextFloat() * 6.0,
                m_yaw: 0.0,
                m_pitch: 0.0,
                m_step: -1,
                m_end: -1,
                m_yScale: 0.5,
            };
            carveTunnel(carver, room, highlands);
            tunnels += random.nextIntBounded(4);
        }
        for _ in 0..tunnels
        {
            let yaw: f32 = random.nextFloat() * PI * 2.0;
            let pitch: f32 = (random.nextFloat() - 0.5) * 2.0 / 8.0;
            let mut size: f32 = random.nextFloat() * 2.0 + random.nextFloat();
            if random.nextIntBounded(10) == 0
            {
                size *= random.nextFloat() * random.nextFloat() * 3.0 + 1.0;
            }
            let tunnel: Tunnel = Tunnel
            {
                m_seed: random.nextLong(),
                m_pos: start,
                m_size: size,
                m_yaw: yaw,
                m_pitch: pitch,
                m_step: 0,
                m_end: 0,
                m_yScale: 1.0,
            };
            carveTunnel(carver, tunnel, highlands);
        }
    }
}

// TFGenRavine.generateRavine, a tall narrow gash with layered walls
fn carveRavine(carver: &mut Carver, seed: i64, mut pos: [f64; 3], size: f32, mut yaw: f32, mut pitch: f32, yScale: f64)
{
    let random: Random = Random::withSeed(seed);
    let centerX: f64 = f64::from(carver.m_chunkX * 16 + 8);
    let centerZ: f64 = f64::from(carver.m_chunkZ * 16 + 8);
    let (mut yawTurn, mut pitchTurn): (f32, f32) = (0.0, 0.0);
    let blocks: i32 = RANGE * 16 - 16;
    let end: i32 = blocks - random.nextIntBounded(blocks / 4);
    let height: usize = carver.m_ids.m_height;
    let mut layers: Vec<f32> = vec![0.0; height];
    let mut layer: f32 = 1.0;
    for (index, slot) in layers.iter_mut().enumerate()
    {
        if index == 0 || random.nextIntBounded(3) == 0
        {
            layer = 1.0 + random.nextFloat() * random.nextFloat();
        }
        *slot = layer * layer;
    }
    for step in 0..end
    {
        let mut width: f64 = 1.5 + f64::from(math::sin(step as f32 * 3.141593 / end as f32) * size);
        let mut tall: f64 = width * yScale;
        width *= f64::from(random.nextFloat()) * 0.25 + 0.75;
        tall *= f64::from(random.nextFloat()) * 0.25 + 0.75;
        let cosPitch: f32 = math::cos(pitch);
        pos[0] += f64::from(math::cos(yaw) * cosPitch);
        pos[1] += f64::from(math::sin(pitch));
        pos[2] += f64::from(math::sin(yaw) * cosPitch);
        pitch *= 0.7;
        pitch += pitchTurn * 0.05;
        yaw += yawTurn * 0.05;
        pitchTurn *= 0.8;
        yawTurn *= 0.5;
        pitchTurn += (random.nextFloat() - random.nextFloat()) * random.nextFloat() * 2.0;
        yawTurn += (random.nextFloat() - random.nextFloat()) * random.nextFloat() * 4.0;
        if random.nextIntBounded(4) == 0
        {
            continue;
        }
        let (dx, dz): (f64, f64) = (pos[0] - centerX, pos[2] - centerZ);
        let left: f64 = f64::from(end - step);
        let reach: f64 = f64::from(size + 2.0 + 16.0);
        if dx * dx + dz * dz - left * left > reach * reach
        {
            return;
        }
        let margin: f64 = 16.0 + width * 2.0;
        if pos[0] < centerX - margin
            || pos[2] < centerZ - margin
            || pos[0] > centerX + margin
            || pos[2] > centerZ + margin
        {
            continue;
        }
        let originX: i32 = carver.m_chunkX * 16;
        let originZ: i32 = carver.m_chunkZ * 16;
        let minX: i32 = (math::floor(pos[0] - width) - originX - 1).max(0);
        let maxX: i32 = (math::floor(pos[0] + width) - originX + 1).min(16);
        let minY: i32 = (math::floor(pos[1] - tall) - 1).max(1);
        let maxY: i32 = (math::floor(pos[1] + tall) + 1).min(height as i32 - 8);
        let minZ: i32 = (math::floor(pos[2] - width) - originZ - 1).max(0);
        let maxZ: i32 = (math::floor(pos[2] + width) - originZ + 1).min(16);
        if carver.wet(minX, maxX, minY, maxY, minZ, maxZ)
        {
            continue;
        }
        for x in minX..maxX
        {
            let fx: f64 = (f64::from(x + originX) + 0.5 - pos[0]) / width;
            for z in minZ..maxZ
            {
                let fz: f64 = (f64::from(z + originZ) + 0.5 - pos[2]) / width;
                if fx * fx + fz * fz >= 1.0
                {
                    continue;
                }
                let mut hitGrass: bool = false;
                for y in (minY..maxY).rev()
                {
                    let fy: f64 = (f64::from(y) + 0.5 - pos[1]) / tall;
                    if (fx * fx + fz * fz) * f64::from(layers[y as usize]) + fy * fy / 6.0 >= 1.0
                    {
                        continue;
                    }
                    let index: usize = carver.index(x, y + 1, z);
                    let block: i32 = carver.at(index);
                    let ids: &Carvable = carver.m_ids;
                    if block == ids.m_grass
                    {
                        hitGrass = true;
                    }
                    if block == ids.m_stone
                        || block == carver.m_trollsteinn
                        || block == ids.m_dirt
                        || block == ids.m_grass
                    {
                        carver.m_blocks[index] = 0;
                        if hitGrass && index > 0 && carver.at(index - 1) == ids.m_dirt
                        {
                            let top: i32 = carver.topBlock(x + originX, z + originZ);
                            carver.m_blocks[index - 1] = top as u16;
                        }
                    }
                }
            }
        }
    }
}

// TFGenRavine.func_151538_a, one ravine in 127 source chunks where decorations are allowed
fn ravines(carver: &mut Carver, random: &Random, sourceX: i32, sourceZ: i32)
{
    let odds: i32 = 127;
    if random.nextIntBounded(odds) != 0
    {
        return;
    }
    let (near, _) = features::nearest(carver.m_seed, carver.m_chunkX, carver.m_chunkZ);
    if !features::row(near).m_decorated
    {
        return;
    }
    let start: [f64; 3] = [
        f64::from(sourceX * 16 + random.nextIntBounded(16)),
        f64::from(random.nextIntBounded(random.nextIntBounded(40) + 8) + 20),
        f64::from(sourceZ * 16 + random.nextIntBounded(16)),
    ];
    let yaw: f32 = random.nextFloat() * 3.141593 * 2.0;
    let pitch: f32 = (random.nextFloat() - 0.5) * 2.0 / 8.0;
    let size: f32 = (random.nextFloat() * 2.0 + random.nextFloat()) * 2.0;
    let yScale: f64 = 3.0;
    carveRavine(carver, random.nextLong(), start, size, yaw, pitch, yScale);
}

// MapGenBase.generate, every chunk in range seeds its own share of tunnels
fn sweep(carver: &mut Carver, run: fn(&mut Carver, &Random, i32, i32))
{
    let random: Random = Random::withSeed(carver.m_seed);
    let xFactor: i64 = random.nextLong();
    let zFactor: i64 = random.nextLong();
    for sourceX in carver.m_chunkX - RANGE..=carver.m_chunkX + RANGE
    {
        for sourceZ in carver.m_chunkZ - RANGE..=carver.m_chunkZ + RANGE
        {
            random.setSeed(
                i64::from(sourceX).wrapping_mul(xFactor) ^ i64::from(sourceZ).wrapping_mul(zFactor) ^ carver.m_seed,
            );
            run(carver, &random, sourceX, sourceZ);
        }
    }
}

pub fn carve(chunk: &ChunkInfo, blocks: &mut [u16], ids: &Carvable)
{
    let mut carver: Carver = Carver
    {
        m_blocks: blocks,
        m_ids: ids,
        m_trollsteinn: crate::ids::blockId(crate::ids::block::TROLLSTEINN),
        m_seed: chunk.seed,
        m_chunkX: chunk.x,
        m_chunkZ: chunk.z,
    };
    sweep(&mut carver, caves);
    sweep(&mut carver, ravines);
}
