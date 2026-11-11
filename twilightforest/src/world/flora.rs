use super::biomes::Biome;
use b173_api::terrain::Random;

// what each twilight biome sets on its TFBiomeDecorator, rows in Biome order
pub(crate) struct Decor
{
    pub m_trees: i32,
    pub m_grass: i32,
    pub m_flowers: i32,
    pub m_deadBush: i32,
    pub m_mushrooms: i32,
    pub m_bigMushrooms: i32,
    pub m_reeds: i32,
    pub m_clay: i32,
    pub m_waterlily: i32,
    pub m_canopy: f32,
    pub m_altCanopy: f32,
    pub m_mycelium: i32,
    pub m_mangroves: i32,
    pub m_lakes: i32,
    pub m_lava: f32,
    // BiomeDecorator.generateLakes, which also gates the cave springs
    pub m_springs: bool,
}

// TwilightForestMod.canopyCoverage at its config default
pub(crate) const CANOPY: f32 = 1.7;
// the source turns a decoration off with a large negative count
const NONE: i32 = -999;
const NO_CANOPY: f32 = -999.0;

const BASE: Decor = Decor
{
    m_trees: 10,
    m_grass: 2,
    m_flowers: 2,
    m_deadBush: 0,
    m_mushrooms: 0,
    m_bigMushrooms: 0,
    m_reeds: 0,
    m_clay: 1,
    m_waterlily: 0,
    m_canopy: CANOPY,
    m_altCanopy: 0.0,
    m_mycelium: 0,
    m_mangroves: 0,
    m_lakes: 0,
    m_lava: 0.0,
    m_springs: true,
};

const DARK: Decor =
    Decor { m_canopy: 5.5, m_grass: -99, m_flowers: -99, m_mushrooms: 2, m_deadBush: 10, m_springs: false, ..BASE };

const DECOR: [Decor; 19] = [
    BASE,
    BASE,
    Decor { m_trees: 25, m_grass: 15, m_flowers: 8, ..BASE },
    Decor { m_canopy: NO_CANOPY, m_grass: 7, m_deadBush: 1, m_springs: false, ..BASE },
    Decor { m_trees: 8, m_mushrooms: 8, m_bigMushrooms: 2, m_altCanopy: 0.2, ..BASE },
    Decor
    {
        m_deadBush: 1,
        m_mushrooms: 8,
        m_reeds: 10,
        m_trees: 2,
        m_waterlily: 20,
        m_canopy: NO_CANOPY,
        m_lakes: 2,
        m_mangroves: 3,
        ..BASE
    },
    Decor { m_waterlily: 2, ..BASE },
    Decor { m_trees: 7, m_grass: 1, m_canopy: NO_CANOPY, m_springs: false, ..BASE },
    Decor { m_trees: 1, m_grass: 0, m_canopy: NO_CANOPY, ..BASE },
    Decor { m_canopy: NO_CANOPY, m_trees: NONE, m_flowers: 4, m_grass: 10, ..BASE },
    Decor { m_trees: 1, m_flowers: 4, m_grass: 20, m_altCanopy: 0.8, ..BASE },
    Decor { m_flowers: 4, m_grass: 1, m_trees: 2, ..BASE },
    Decor { m_trees: 1, m_mushrooms: 12, m_bigMushrooms: 8, m_mycelium: 3, m_altCanopy: 0.9, ..BASE },
    DARK,
    Decor { m_grass: 12, m_flowers: 8, ..BASE },
    Decor
    {
        m_deadBush: 2,
        m_mushrooms: 8,
        m_reeds: 4,
        m_trees: 3,
        m_waterlily: 6,
        m_canopy: NO_CANOPY,
        m_lava: 0.125,
        m_mangroves: 3,
        ..BASE
    },
    DARK,
    Decor { m_canopy: NO_CANOPY, m_trees: NONE, m_springs: false, ..BASE },
    Decor { m_canopy: NO_CANOPY, m_trees: NONE, m_deadBush: 2, m_springs: false, ..BASE },
];

pub(crate) fn decor(biome: Biome) -> &'static Decor { &DECOR[biome as usize] }

// the trees a biome's func_150567_a hands out, beta substitutes noted where a kind is missing
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Grow
{
    Oak,
    Birch,
    BigOak,
    // WorldGenShrub of jungle wood, a beta oak bush
    Shrub,
    // WorldGenSwamp drapes vines that beta lacks, a plain oak stands in
    Swamp,
    Taiga1,
    Taiga2,
    // WorldGenMegaPineTree has no beta wood, the tall taiga spruce stands in
    MegaPine,
    LargeWinter,
    SmallRainbow,
    LargeRainbow,
}

fn chance(random: &Random, odds: i32) -> bool { random.nextIntBounded(odds) == 0 }

// TFBiomeBase.func_150567_a
fn plainTree(random: &Random) -> Grow
{
    if chance(random, 5)
    {
        return Grow::Birch;
    }

    if chance(random, 10)
    {
        Grow::BigOak
    }
    else
    {
        Grow::Oak
    }
}

pub(crate) fn tree(biome: Biome, random: &Random) -> Grow
{
    match biome
    {
        Biome::DenseForest if chance(random, 5) => Grow::Shrub,

        Biome::DenseForest | Biome::OakSavanna =>
        {
            if chance(random, 10)
            {
                Grow::BigOak
            }
            else
            {
                Grow::Oak
            }
        }

        Biome::EnchantedForest if chance(random, 15) => Grow::SmallRainbow,
        Biome::EnchantedForest if chance(random, 50) => Grow::LargeRainbow,

        Biome::Swamp | Biome::FireSwamp =>
        {
            if chance(random, 3)
            {
                Grow::Shrub
            }
            else
            {
                Grow::Swamp
            }
        }

        Biome::SnowyForest if chance(random, 3) => Grow::Taiga1,

        Biome::SnowyForest =>
        {
            if chance(random, 8)
            {
                Grow::LargeWinter
            }
            else
            {
                Grow::Taiga2
            }
        }

        Biome::Glacier =>
        {
            if chance(random, 3)
            {
                Grow::Taiga1
            }
            else
            {
                Grow::Taiga2
            }
        }

        Biome::Highlands if chance(random, 4) => Grow::Taiga1,
        Biome::Highlands if chance(random, 10) => Grow::Taiga2,
        Biome::Highlands if chance(random, 3) => Grow::MegaPine,

        Biome::Highlands =>
        {
            if chance(random, 13)
            {
                Grow::MegaPine
            }
            else
            {
                Grow::Birch
            }
        }

        Biome::DarkForest | Biome::DarkForestCenter if chance(random, 5) => Grow::Shrub,

        Biome::DarkForest | Biome::DarkForestCenter =>
        {
            if chance(random, 8)
            {
                Grow::Birch
            }
            else
            {
                Grow::Oak
            }
        }

        _ => plainTree(random),
    }
}

// the plant a biome's func_76730_b hands to WorldGenTallGrass
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Grass
{
    Grass,
    Fern,
    Twilight(&'static str),
}

const MAYAPPLE: Grass = Grass::Twilight("mayapple");
const FIDDLEHEAD: Grass = Grass::Twilight(crate::names::block::FIDDLEHEAD);

pub(crate) fn grass(biome: Biome, random: &Random) -> Grass
{
    match biome
    {
        Biome::DenseForest if random.nextIntBounded(4) != 0 => Grass::Fern,

        Biome::DenseForest =>
        {
            if random.nextBoolean()
            {
                MAYAPPLE
            }
            else
            {
                Grass::Grass
            }
        }

        Biome::EnchantedForest if random.nextIntBounded(3) > 0 => Grass::Fern,

        Biome::EnchantedForest =>
        {
            if chance(random, 3)
            {
                FIDDLEHEAD
            }
            else
            {
                Grass::Grass
            }
        }

        Biome::Clearing => Grass::Grass,
        Biome::OakSavanna if chance(random, 10) => Grass::Fern,

        Biome::OakSavanna =>
        {
            if chance(random, 10)
            {
                MAYAPPLE
            }
            else
            {
                Grass::Grass
            }
        }

        Biome::Swamp if chance(random, 4) => Grass::Fern,

        Biome::Swamp =>
        {
            if chance(random, 4)
            {
                MAYAPPLE
            }
            else
            {
                Grass::Grass
            }
        }

        Biome::Highlands =>
        {
            if random.nextIntBounded(5) > 0
            {
                Grass::Fern
            }
            else
            {
                Grass::Grass
            }
        }

        _ =>
        {
            if chance(random, 4)
            {
                Grass::Fern
            }
            else
            {
                Grass::Grass
            }
        }
    }
}

// the 1.7 BlockFlower kinds, beta keeps only the rose and the dandelion so each takes the nearer colour
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Flower
{
    Rose,
    Dandelion,
}

// BlockFlower.field_149859_a in order, poppy orchid allium houstonia then the tulips and the oxeye daisy
const KINDS: [Flower; 9] = [
    Flower::Rose,
    Flower::Rose,
    Flower::Rose,
    Flower::Dandelion,
    Flower::Rose,
    Flower::Dandelion,
    Flower::Dandelion,
    Flower::Rose,
    Flower::Dandelion,
];

const POPPY: usize = 0;
const HOUSTONIA: usize = 3;
const TULIPS: usize = 4;
const OXEYE: usize = 8;

// func_150572_a of each biome, the flower picked for one flower patch
pub(crate) fn flower(biome: Biome, random: &Random, x: i32, z: i32) -> Flower
{
    match biome
    {
        Biome::EnchantedForest =>
        {
            let kind: usize = if random.nextIntBounded(3) > 0
            {
                1
            }
            else if random.nextBoolean()
            {
                2
            }
            else
            {
                3
            };
            KINDS[kind]
        }

        Biome::OakSavanna =>
        {
            let scale: f64 = 200.0;
            let tulipField: f64 = -0.8;
            if super::noise::plant(f64::from(x) / scale, f64::from(z) / scale) < tulipField
            {
                return KINDS[TULIPS + random.nextIntBounded(4) as usize];
            }
            if random.nextIntBounded(3) == 0
            {
                return Flower::Dandelion;
            }

            KINDS[[POPPY, HOUSTONIA, OXEYE][random.nextIntBounded(3) as usize]]
        }

        Biome::FireflyForest =>
        {
            let scale: f64 = 48.0;
            let field: f64 =
                ((1.0 + super::noise::plant(f64::from(x) / scale, f64::from(z) / scale)) / 2.0).clamp(0.0, 0.9999);
            let kind: usize = (field * KINDS.len() as f64) as usize;
            // the source skips the blue orchid
            KINDS[if kind == 1 { 0 } else { kind }]
        }

        Biome::Highlands =>
        {
            if random.nextBoolean()
            {
                Flower::Dandelion
            }
            else
            {
                KINDS[OXEYE]
            }
        }

        _ =>
        {
            if random.nextIntBounded(3) > 0
            {
                Flower::Dandelion
            }
            else
            {
                Flower::Rose
            }
        }
    }
}
