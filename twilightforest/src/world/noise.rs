use b173_api::terrain::Random;

// NoiseGeneratorSimplex from 1.7, two dimensional simplex noise over a shuffled permutation
const GRADIENTS: [[i32; 2]; 12] =
    [[1, 1], [-1, 1], [1, -1], [-1, -1], [1, 0], [-1, 0], [1, 0], [-1, 0], [0, 1], [0, -1], [0, 1], [0, -1]];

mod simplex
{
    pub const SCALE: f64 = 70.0;
    pub const REACH: f64 = 0.5;
    pub const TABLE: usize = 256;
}

fn skew() -> (f64, f64)
{
    let root: f64 = 3.0f64.sqrt();
    (0.5 * (root - 1.0), (3.0 - root) / 6.0)
}

fn fastFloor(value: f64) -> i32
{
    if value > 0.0
    {
        value as i32
    }
    else
    {
        value as i32 - 1
    }
}

pub struct Simplex
{
    m_table: [i32; simplex::TABLE * 2],
    m_offset: [f64; 3],
}

impl Simplex
{
    pub fn new(random: &Random) -> Simplex
    {
        let span: f64 = simplex::TABLE as f64;
        let offset: [f64; 3] = [random.nextDouble() * span, random.nextDouble() * span, random.nextDouble() * span];
        let mut table: [i32; simplex::TABLE * 2] = [0; simplex::TABLE * 2];
        for (index, slot) in table.iter_mut().take(simplex::TABLE).enumerate()
        {
            *slot = index as i32;
        }
        for index in 0..simplex::TABLE
        {
            let other: usize = (random.nextIntBounded((simplex::TABLE - index) as i32) as usize) + index;
            table.swap(index, other);
            table[index + simplex::TABLE] = table[index];
        }
        Simplex { m_table: table, m_offset: offset }
    }

    fn corner(gradient: usize, x: f64, y: f64) -> f64
    {
        let falloff: f64 = simplex::REACH - x * x - y * y;
        if falloff < 0.0
        {
            return 0.0;
        }
        let squared: f64 = falloff * falloff;
        let [gx, gy] = GRADIENTS[gradient];
        squared * squared * (f64::from(gx) * x + f64::from(gy) * y)
    }

    // NoiseGeneratorSimplex.func_151605_a
    pub fn value(&self, x: f64, y: f64) -> f64
    {
        let (skewIn, skewOut): (f64, f64) = skew();
        let stretch: f64 = (x + y) * skewIn;
        let cellX: i32 = fastFloor(x + stretch);
        let cellY: i32 = fastFloor(y + stretch);
        let unstretch: f64 = f64::from(cellX + cellY) * skewOut;
        let x0: f64 = x - (f64::from(cellX) - unstretch);
        let y0: f64 = y - (f64::from(cellY) - unstretch);
        let (stepX, stepY): (i32, i32) = if x0 > y0 { (1, 0) } else { (0, 1) };
        let x1: f64 = x0 - f64::from(stepX) + skewOut;
        let y1: f64 = y0 - f64::from(stepY) + skewOut;
        let x2: f64 = x0 - 1.0 + 2.0 * skewOut;
        let y2: f64 = y0 - 1.0 + 2.0 * skewOut;
        let wrapX: usize = (cellX & 255) as usize;
        let wrapY: usize = (cellY & 255) as usize;
        let table = |index: usize| self.m_table[index] as usize;
        let g0: usize = table(wrapX + table(wrapY)) % GRADIENTS.len();
        let g1: usize = table(wrapX + stepX as usize + table(wrapY + stepY as usize)) % GRADIENTS.len();
        let g2: usize = table(wrapX + 1 + table(wrapY + 1)) % GRADIENTS.len();
        simplex::SCALE * (Self::corner(g0, x0, y0) + Self::corner(g1, x1, y1) + Self::corner(g2, x2, y2))
    }

    // NoiseGeneratorSimplex.func_151606_a adds a scaled grid of noise into the buffer
    pub fn add(&self, buffer: &mut [f64], x: f64, z: f64, width: usize, depth: usize, scale: [f64; 2], amplitude: f64)
    {
        let mut index: usize = 0;
        for dz in 0..depth
        {
            let atZ: f64 = (z + dz as f64) * scale[1] + self.m_offset[1];
            for dx in 0..width
            {
                let atX: f64 = (x + dx as f64) * scale[0] + self.m_offset[0];
                buffer[index] += self.value(atX, atZ) * amplitude;
                index += 1;
            }
        }
    }
}

// NoiseGeneratorPerlin from 1.7, octaves of simplex noise
pub struct Octaves
{
    m_levels: Vec<Simplex>,
}

impl Octaves
{
    pub fn new(random: &Random, levels: usize) -> Octaves
    {
        Octaves { m_levels: (0..levels).map(|_| Simplex::new(random)).collect() }
    }

    // NoiseGeneratorPerlin.func_151601_a
    pub fn value(&self, x: f64, z: f64) -> f64
    {
        let mut total: f64 = 0.0;
        let mut frequency: f64 = 1.0;
        for level in &self.m_levels
        {
            total += level.value(x * frequency, z * frequency) / frequency;
            frequency /= 2.0;
        }
        total
    }

    // NoiseGeneratorPerlin.func_151599_a with the default half persistence
    pub fn region(&self, x: f64, z: f64, width: usize, depth: usize, scale: [f64; 2], lacunarity: f64) -> Vec<f64>
    {
        let persistence: f64 = 0.5;
        let base: f64 = 0.55;
        let mut buffer: Vec<f64> = vec![0.0; width * depth];
        let mut amplitude: f64 = 1.0;
        let mut frequency: f64 = 1.0;
        for level in &self.m_levels
        {
            level.add(&mut buffer, x, z, width, depth, [scale[0] * frequency, scale[1] * frequency], base / amplitude);
            frequency *= lacunarity;
            amplitude *= persistence;
        }
        buffer
    }
}

thread_local!
{
    // BiomeGenBase.plantNoise, seeded with 2345 and one octave
    static PLANT: Octaves = Octaves::new(&Random::withSeed(2345), 1);
}

pub fn plant(x: f64, z: f64) -> f64 { PLANT.with(|noise| noise.value(x, z)) }
