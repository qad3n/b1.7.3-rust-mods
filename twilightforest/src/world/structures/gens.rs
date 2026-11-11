use super::component::Blk;
use crate::world::generate::GenWorld;
use crate::world::scatter::{self, Spike};
use crate::world::trees::{self, Builder};
use crate::world::vanilla;
use b173_api::terrain::Random;

// the WorldGenerator objects structures keep and call, as the port's generator functions
#[derive(Clone, Copy, Default)]
pub(crate) enum Gen
{
    #[default]
    Nothing,
    // TFGenCaveStalactite
    Spike(Spike),
    // TFGenCanopyTree
    Canopy,
    // TFGenSmallTwilightOak
    TwilightOak,
    // TFGenSmallRainboak
    Rainboak,
    // WorldGenTrees with its minimum height and wood metadata
    Trees
    {
        minimum: i32,
        meta: i32,
    },
    // WorldGenForest
    Birch,
    // TFGenBigMushgloom
    BigMushgloom,
    // WorldGenBigMushroom
    BigMushroom,
    // TFGenMyceliumBlob
    Blob
    {
        block: Blk,
        size: i32,
    },
}

impl Gen
{
    // TFGenCaveStalactite(block, length, up), a plain spike of one block
    pub(crate) fn stalactite(block: Blk, size: f32, hang: bool) -> Self
    {
        Self::Spike(Spike { m_block: block.qualified(), m_size: size, m_longest: -1, m_gap: -1, m_hang: hang })
    }

    // TFGenCaveStalactite.makeRandomOreStalactite
    pub(crate) fn ore(random: &Random, hill: i32) -> Self { Self::Spike(*scatter::randomOre(random, hill)) }

    // WorldGenerator.generate
    pub(crate) fn generate(self, world: &GenWorld, random: &Random, x: i32, y: i32, z: i32) -> bool
    {
        let builder: Builder = Builder { m_world: world, m_notify: false };

        match self
        {
            Self::Nothing => false,
            Self::Spike(spike) => scatter::stalactite(world, random, x, y, z, &spike),
            Self::Canopy => trees::canopyTree(&builder, random, x, y, z),
            Self::TwilightOak => trees::twilightOak(&builder, random, x, y, z),
            Self::Rainboak => trees::smallRainbow(&builder, random, x, y, z),
            Self::Trees { minimum, meta } => vanilla::tree(world, random, x, y, z, minimum, meta),
            Self::Birch => vanilla::birch(world, random, x, y, z),
            Self::BigMushgloom => scatter::bigMushgloom(world, random, x, y, z),
            Self::BigMushroom => vanilla::bigMushroom(world, random, x, y, z),

            Self::Blob { block, size } =>
            {
                scatter::groundBlob(world, random, x, y, z, block.resolve(0), size);
                true
            }
        }
    }
}

// TFGenerator.translateCoords
pub(crate) fn translateCoords(x: i32, y: i32, z: i32, distance: impl Into<f64>, angle: f64, tilt: f64) -> [i32; 3]
{
    trees::translate([x, y, z], distance.into(), angle, tilt)
}

// TFGenerator.getBresehnamArrayCoords
pub(crate) fn bresenham(x1: i32, y1: i32, z1: i32, x2: i32, y2: i32, z2: i32) -> Vec<[i32; 3]>
{
    trees::line([x1, y1, z1], [x2, y2, z2])
}
