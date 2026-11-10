use super::biomes::Biome;

// GenLayer from 1.7, the biome map is a chain of layers each reading the one below it at a larger scale
mod lcg
{
    pub const MULTIPLIER: i64 = 6364136223846793005;
    pub const INCREMENT: i64 = 1442695040888963407;
}

fn step(value: i64) -> i64 { value.wrapping_mul(value.wrapping_mul(lcg::MULTIPLIER).wrapping_add(lcg::INCREMENT)) }

// the seed state every layer keeps, GenLayer.baseSeed worldGenSeed and chunkSeed
#[derive(Clone, Copy, Default)]
struct Seeds
{
    m_base: i64,
    m_world: i64,
    m_chunk: i64,
}

impl Seeds
{
    fn new(seed: i64) -> Seeds
    {
        let mut base: i64 = seed;
        for _ in 0..3
        {
            base = step(base).wrapping_add(seed);
        }
        Seeds { m_base: base, m_world: 0, m_chunk: 0 }
    }

    fn initWorld(&mut self, seed: i64)
    {
        let mut world: i64 = seed;
        for _ in 0..3
        {
            world = step(world).wrapping_add(self.m_base);
        }
        self.m_world = world;
    }

    fn initChunk(&mut self, x: i64, z: i64)
    {
        let mut chunk: i64 = self.m_world;
        for offset in [x, z, x, z]
        {
            chunk = step(chunk).wrapping_add(offset);
        }
        self.m_chunk = chunk;
    }

    fn next(&mut self, bound: i32) -> i32
    {
        let mut value: i32 = ((self.m_chunk >> 24) % i64::from(bound)) as i32;
        if value < 0
        {
            value += bound;
        }
        self.m_chunk = step(self.m_chunk).wrapping_add(self.m_world);
        value
    }

    fn pick(&mut self, choices: &[i32]) -> i32 { choices[self.next(choices.len() as i32) as usize] }

    // GenLayer.selectModeOrRandom
    fn mode(&mut self, a: i32, b: i32, c: i32, d: i32) -> i32
    {
        if b == c && c == d
        {
            return b;
        }
        if a == b && a == c || a == b && a == d || a == c && a == d
        {
            return a;
        }
        if a == b && c != d || a == c && b != d || a == d && b != c
        {
            return a;
        }
        if b == c && a != d || b == d && a != c
        {
            return b;
        }
        if c == d && a != b
        {
            return c;
        }
        self.pick(&[a, b, c, d])
    }
}

// every kind of layer in the twilight forest chain, GenLayerTF.makeTheWorld
enum Kind
{
    Biomes,
    KeyBiomes,
    Companions,
    Zoom,
    Stabilize,
    ThornBorder,
    Stream,
    Smooth,
    RiverMix(Box<Layer>),
    Voronoi,
}

pub struct Layer
{
    m_kind: Kind,
    m_seeds: Seeds,
    m_parent: Option<Box<Layer>>,
}

const NONE: i32 = -1;

fn id(biome: Biome) -> i32 { biome as i32 }

// GenLayerTFBiomes1Point7 common and rare biome draws, one rare biome in fifteen
const COMMON: [Biome; 5] =
    [Biome::TwilightForest, Biome::DenseForest, Biome::Mushrooms, Biome::OakSavanna, Biome::FireflyForest];
const RARE: [Biome; 4] = [Biome::Lake, Biome::DeepMushrooms, Biome::EnchantedForest, Biome::Clearing];
const RARE_ODDS: i32 = 15;

// GenLayerTFStream biome pairs that meet without a stream between them
const NO_STREAM: [(Biome, Biome); 5] = [
    (Biome::Glacier, Biome::SnowyForest),
    (Biome::DeepMushrooms, Biome::Mushrooms),
    (Biome::Swamp, Biome::FireSwamp),
    (Biome::Highlands, Biome::FinalPlateau),
    (Biome::DarkForest, Biome::DarkForestCenter),
];

fn streamBetween(first: i32, second: i32) -> bool
{
    if first == second || first == -second
    {
        return false;
    }
    let paired: bool = NO_STREAM.iter().any(|(left, right)|
    {
        (first == id(*left) && second == id(*right)) || (first == id(*right) && second == id(*left))
    });
    if paired
    {
        return false;
    }
    let lake: i32 = id(Biome::Lake);
    let clearing: i32 = id(Biome::Clearing);
    let savanna: i32 = id(Biome::OakSavanna);
    let thorns: i32 = id(Biome::Thornlands);
    // the source checks clearing and savanna on crossed sides, kept as written
    first != lake
        && second != lake
        && first != clearing
        && second != savanna
        && first != savanna
        && second != clearing
        && first != thorns
        && second != thorns
}

fn neighbours(input: &[i32], width: usize, dx: usize, dz: usize) -> [i32; 5]
{
    let at = |x: usize, z: usize| input[x + z * width];
    [at(dx, dz + 1), at(dx + 2, dz + 1), at(dx + 1, dz), at(dx + 1, dz + 2), at(dx + 1, dz + 1)]
}

impl Layer
{
    fn new(seed: i64, kind: Kind, parent: Option<Layer>) -> Layer
    {
        Layer { m_kind: kind, m_seeds: Seeds::new(seed), m_parent: parent.map(Box::new) }
    }

    fn initWorld(&mut self, seed: i64)
    {
        if let Some(parent) = self.m_parent.as_mut()
        {
            parent.initWorld(seed);
        }
        if let Kind::RiverMix(rivers) = &mut self.m_kind
        {
            rivers.initWorld(seed);
        }
        self.m_seeds.initWorld(seed);
    }

    fn parent(&mut self) -> &mut Layer { self.m_parent.as_mut().expect("a derived layer reads a parent") }

    pub fn ints(&mut self, x: i32, z: i32, width: i32, depth: i32) -> Vec<i32>
    {
        match self.m_kind
        {
            Kind::Biomes => self.biomes(x, z, width, depth),
            Kind::KeyBiomes => self.keyBiomes(x, z, width, depth),
            Kind::Companions | Kind::ThornBorder | Kind::Stream | Kind::Stabilize => self.border(x, z, width, depth),
            Kind::Zoom => self.zoom(x, z, width, depth),
            Kind::Smooth => self.smooth(x, z, width, depth),
            Kind::RiverMix(_) => self.riverMix(x, z, width, depth),
            Kind::Voronoi => self.voronoi(x, z, width, depth),
        }
    }

    fn biomes(&mut self, x: i32, z: i32, width: i32, depth: i32) -> Vec<i32>
    {
        let mut output: Vec<i32> = Vec::with_capacity((width * depth) as usize);
        for dz in 0..depth
        {
            for dx in 0..width
            {
                self.m_seeds.initChunk(i64::from(dx + x), i64::from(dz + z));
                let biome: Biome = if self.m_seeds.next(RARE_ODDS) == 0
                {
                    RARE[self.m_seeds.next(RARE.len() as i32) as usize]
                }
                else
                {
                    COMMON[self.m_seeds.next(COMMON.len() as i32) as usize]
                };
                output.push(id(biome));
            }
        }
        output
    }

    // GenLayerTFKeyBiomes, one key biome per 8x8 region quarter at a random spot of its 4x4 cell
    fn keyBiomes(&mut self, x: i32, z: i32, width: i32, depth: i32) -> Vec<i32>
    {
        let source: Vec<i32> = self.parent().ints(x, z, width, depth);
        let mut output: Vec<i32> = source.clone();
        for dz in 0..depth
        {
            for dx in 0..width
            {
                let (mapX, mapZ): (i32, i32) = (dx + x, dz + z);
                self.m_seeds.initChunk(i64::from(mapX | 3), i64::from(mapZ | 3));
                let ox: i32 = self.m_seeds.next(3) + 1;
                let oz: i32 = self.m_seeds.next(3) + 1;
                if (mapX & 3) != ox || (mapZ & 3) != oz
                {
                    continue;
                }
                let index: i32 = match (mapX & 4 == 0, mapZ & 4 == 0)
                {
                    (true, true) => 0,
                    (true, false) => 1,
                    (false, true) => 2,
                    (false, false) => 3,
                };
                output[(dx + dz * width) as usize] = self.keyBiome(mapX, mapZ, index);
            }
        }
        output
    }

    fn keyBiome(&mut self, mapX: i32, mapZ: i32, index: i32) -> i32
    {
        self.m_seeds.initChunk(i64::from((mapX + 4) >> 3), i64::from((mapZ + 4) >> 3));
        let offset: i32 = self.m_seeds.next(4);
        let biome: Biome = match (index + offset) % 4
        {
            1 => Biome::FireSwamp,
            2 => Biome::DarkForestCenter,
            3 => Biome::FinalPlateau,
            _ => Biome::Glacier,
        };
        id(biome)
    }

    // the layers that read their four neighbours and themselves from a parent read one cell larger on every side
    fn border(&mut self, x: i32, z: i32, width: i32, depth: i32) -> Vec<i32>
    {
        let wide: usize = (width + 2) as usize;
        let input: Vec<i32> = self.parent().ints(x - 1, z - 1, width + 2, depth + 2);
        let mut output: Vec<i32> = Vec::with_capacity((width * depth) as usize);
        let (offX, offZ): (i32, i32) = (x & 3, z & 3);
        for dz in 0..depth as usize
        {
            for dx in 0..width as usize
            {
                let [right, left, up, down, center] = neighbours(&input, wide, dx, dz);
                let value: i32 = match self.m_kind
                {
                    Kind::Companions => companion(center, [right, left, up, down]),
                    Kind::ThornBorder =>
                    {
                        let corners: [i32; 4] = [
                            input[dx + dz * wide],
                            input[dx + 2 + dz * wide],
                            input[dx + (dz + 2) * wide],
                            input[dx + 2 + (dz + 2) * wide],
                        ];
                        thorns(center, [right, left, up, down], corners)
                    }
                    Kind::Stream =>
                    {
                        // GenLayerTFStream names the same cells left right down up
                        let streams: bool = [right, left, up, down].iter().any(|other| streamBetween(center, *other));
                        if streams { id(Biome::Stream) } else { NONE }
                    }
                    _ =>
                    {
                        let centerX: i32 = ((dx as i32 + offX + 1) & -4) - offX;
                        let centerZ: i32 = ((dz as i32 + offZ + 1) & -4) - offZ;
                        let near: bool = (centerX - 1..=centerX + 1).contains(&(dx as i32))
                            && (centerZ - 1..=centerZ + 1).contains(&(dz as i32));
                        if near
                        {
                            input[(centerX + 1) as usize + (centerZ + 1) as usize * wide]
                        }
                        else
                        {
                            center
                        }
                    }
                };
                output.push(value);
            }
        }
        output
    }

    // GenLayerZoom doubles the resolution, picking among neighbours at random
    fn zoom(&mut self, x: i32, z: i32, width: i32, depth: i32) -> Vec<i32>
    {
        let (parentX, parentZ): (i32, i32) = (x >> 1, z >> 1);
        let parentWidth: i32 = (width >> 1) + 2;
        let parentDepth: i32 = (depth >> 1) + 2;
        let input: Vec<i32> = self.parent().ints(parentX, parentZ, parentWidth, parentDepth);
        let zoomedWidth: i32 = (parentWidth - 1) << 1;
        let zoomedDepth: i32 = (parentDepth - 1) << 1;
        let mut zoomed: Vec<i32> = vec![0; (zoomedWidth * zoomedDepth) as usize];
        let at = |px: i32, pz: i32| input[(px + pz * parentWidth) as usize];
        for pz in 0..parentDepth - 1
        {
            let mut index: i32 = (pz << 1) * zoomedWidth;
            let mut topLeft: i32 = at(0, pz);
            let mut bottomLeft: i32 = at(0, pz + 1);
            for px in 0..parentWidth - 1
            {
                self.m_seeds.initChunk(i64::from((px + parentX) << 1), i64::from((pz + parentZ) << 1));
                let topRight: i32 = at(px + 1, pz);
                let bottomRight: i32 = at(px + 1, pz + 1);
                zoomed[index as usize] = topLeft;
                zoomed[(index + zoomedWidth) as usize] = self.m_seeds.pick(&[topLeft, bottomLeft]);
                index += 1;
                zoomed[index as usize] = self.m_seeds.pick(&[topLeft, topRight]);
                zoomed[(index + zoomedWidth) as usize] = self.m_seeds.mode(topLeft, topRight, bottomLeft, bottomRight);
                index += 1;
                topLeft = topRight;
                bottomLeft = bottomRight;
            }
        }
        crop(&zoomed, zoomedWidth, x & 1, z & 1, width, depth)
    }

    // GenLayerSmooth evens out single cells between matching neighbours
    fn smooth(&mut self, x: i32, z: i32, width: i32, depth: i32) -> Vec<i32>
    {
        let wide: usize = (width + 2) as usize;
        let input: Vec<i32> = self.parent().ints(x - 1, z - 1, width + 2, depth + 2);
        let mut output: Vec<i32> = Vec::with_capacity((width * depth) as usize);
        for dz in 0..depth as usize
        {
            for dx in 0..width as usize
            {
                let [left, right, up, down, mut center] = neighbours(&input, wide, dx, dz);
                if left == right && up == down
                {
                    self.m_seeds.initChunk(i64::from(dx as i32 + x), i64::from(dz as i32 + z));
                    center = if self.m_seeds.next(2) == 0 { left } else { up };
                }
                else
                {
                    if left == right
                    {
                        center = left;
                    }
                    if up == down
                    {
                        center = up;
                    }
                }
                output.push(center);
            }
        }
        output
    }

    // GenLayerTFRiverMix lays streams over the biome map
    fn riverMix(&mut self, x: i32, z: i32, width: i32, depth: i32) -> Vec<i32>
    {
        let biomes: Vec<i32> = self.parent().ints(x, z, width, depth);
        let Kind::RiverMix(rivers) = &mut self.m_kind
        else
        {
            return biomes;
        };
        let streams: Vec<i32> = rivers.ints(x, z, width, depth);
        let stream: i32 = id(Biome::Stream);
        biomes
            .iter()
            .zip(streams)
            .map(|(biome, river)|
            {
                if river == stream
                {
                    river & 0xFF
                }
                else
                {
                    *biome
                }
            })
            .collect()
    }

    // GenLayerVoronoiZoom blows the map up four times with jittered cell borders
    fn voronoi(&mut self, x: i32, z: i32, width: i32, depth: i32) -> Vec<i32>
    {
        let (x, z): (i32, i32) = (x - 2, z - 2);
        let (parentX, parentZ): (i32, i32) = (x >> 2, z >> 2);
        let parentWidth: i32 = (width >> 2) + 2;
        let parentDepth: i32 = (depth >> 2) + 2;
        let input: Vec<i32> = self.parent().ints(parentX, parentZ, parentWidth, parentDepth);
        let zoomedWidth: i32 = (parentWidth - 1) << 2;
        let zoomedDepth: i32 = (parentDepth - 1) << 2;
        let mut zoomed: Vec<i32> = vec![0; (zoomedWidth * zoomedDepth) as usize];
        let at = |px: i32, pz: i32| input[(px + pz * parentWidth) as usize];
        let jitter: f64 = 3.6;
        let resolution: i32 = 1024;
        for pz in 0..parentDepth - 1
        {
            let mut topLeft: i32 = at(0, pz);
            let mut bottomLeft: i32 = at(0, pz + 1);
            for px in 0..parentWidth - 1
            {
                let mut corner = |cx: i32, cz: i32, shiftX: f64, shiftZ: f64| -> (f64, f64)
                {
                    self.m_seeds.initChunk(i64::from((px + parentX + cx) << 2), i64::from((pz + parentZ + cz) << 2));
                    let first: f64 =
                        (f64::from(self.m_seeds.next(resolution)) / f64::from(resolution) - 0.5) * jitter + shiftX;
                    let second: f64 =
                        (f64::from(self.m_seeds.next(resolution)) / f64::from(resolution) - 0.5) * jitter + shiftZ;
                    (first, second)
                };
                let (d1, d2): (f64, f64) = corner(0, 0, 0.0, 0.0);
                let (d3, d4): (f64, f64) = corner(1, 0, 4.0, 0.0);
                let (d5, d6): (f64, f64) = corner(0, 1, 0.0, 4.0);
                let (d7, d8): (f64, f64) = corner(1, 1, 4.0, 4.0);
                let topRight: i32 = at(px + 1, pz) & 0xFF;
                let bottomRight: i32 = at(px + 1, pz + 1) & 0xFF;
                for row in 0..4
                {
                    let mut index: usize = (((pz << 2) + row) * zoomedWidth + (px << 2)) as usize;
                    for column in 0..4
                    {
                        let (r, c): (f64, f64) = (f64::from(row), f64::from(column));
                        let distance =
                            |rowAt: f64, columnAt: f64| (r - rowAt) * (r - rowAt) + (c - columnAt) * (c - columnAt);
                        let first: f64 = distance(d2, d1);
                        let second: f64 = distance(d4, d3);
                        let third: f64 = distance(d6, d5);
                        let fourth: f64 = distance(d8, d7);
                        zoomed[index] = if first < second && first < third && first < fourth
                        {
                            topLeft
                        }
                        else if second < first && second < third && second < fourth
                        {
                            topRight
                        }
                        else if third < first && third < second && third < fourth
                        {
                            bottomLeft
                        }
                        else
                        {
                            bottomRight
                        };
                        index += 1;
                    }
                }
                topLeft = topRight;
                bottomLeft = bottomRight;
            }
        }
        crop(&zoomed, zoomedWidth, x & 3, z & 3, width, depth)
    }
}

fn crop(source: &[i32], sourceWidth: i32, offX: i32, offZ: i32, width: i32, depth: i32) -> Vec<i32>
{
    let mut output: Vec<i32> = Vec::with_capacity((width * depth) as usize);
    for row in 0..depth
    {
        let start: usize = ((row + offZ) * sourceWidth + offX) as usize;
        output.extend_from_slice(&source[start..start + width as usize]);
    }
    output
}

// GenLayerTFCompanionBiomes rings each key biome with its companion
fn companion(center: i32, around: [i32; 4]) -> i32
{
    const PAIRS: [(Biome, Biome); 4] = [
        (Biome::FireSwamp, Biome::Swamp),
        (Biome::Glacier, Biome::SnowyForest),
        (Biome::DarkForestCenter, Biome::DarkForest),
        (Biome::FinalPlateau, Biome::Highlands),
    ];
    PAIRS.iter().find(|(key, _)| center != id(*key) && around.contains(&id(*key))).map_or(center, |(_, ring)| id(*ring))
}

// GenLayerTFThornBorder fences the final plateau in thornlands
fn thorns(center: i32, around: [i32; 4], corners: [i32; 4]) -> i32
{
    let plateau: i32 = id(Biome::FinalPlateau);
    let touches = |cells: [i32; 4]| center != plateau && cells.contains(&plateau);
    if touches(around) || touches(corners)
    {
        id(Biome::Thornlands)
    }
    else
    {
        center
    }
}

// the two maps the world reads, one cell per four blocks for terrain and one per block for everything else
pub struct BiomeMaps
{
    pub m_coarse: Layer,
    pub m_fine: Layer,
}

fn layer(seed: i64, kind: Kind, parent: Layer) -> Layer { Layer::new(seed, kind, Some(parent)) }

fn zooms(mut parent: Layer, seeds: std::ops::Range<i64>) -> Layer
{
    for seed in seeds
    {
        parent = layer(seed, Kind::Zoom, parent);
    }
    parent
}

// GenLayerTF.makeTheWorld, the stream branch reads its own copy of the zoomed biome chain
fn zoomedChain() -> Layer
{
    let biomes: Layer = Layer::new(1, Kind::Biomes, None);
    let keys: Layer = layer(1000, Kind::KeyBiomes, biomes);
    let companions: Layer = layer(1000, Kind::Companions, keys);
    let zoomed: Layer = zooms(companions, 1000..1002);
    let stable: Layer = layer(700, Kind::Stabilize, zoomed);
    let thorns: Layer = layer(500, Kind::ThornBorder, stable);
    zooms(thorns, 1002..1006)
}

pub fn build(seed: i64) -> BiomeMaps
{
    let streams: Layer = layer(1, Kind::Stream, zoomedChain());
    let smooth: Layer = layer(7000, Kind::Smooth, streams);
    let mut mixed: Layer = Layer::new(100, Kind::RiverMix(Box::new(smooth)), Some(zoomedChain()));
    let mut fine: Layer = layer(10, Kind::Voronoi,
    {
        let streams: Layer = layer(1, Kind::Stream, zoomedChain());
        let smooth: Layer = layer(7000, Kind::Smooth, streams);
        Layer::new(100, Kind::RiverMix(Box::new(smooth)), Some(zoomedChain()))
    });
    mixed.initWorld(seed);
    fine.initWorld(seed);
    BiomeMaps { m_coarse: mixed, m_fine: fine }
}
