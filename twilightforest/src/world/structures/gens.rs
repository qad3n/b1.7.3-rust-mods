use super::component::Blk;
use crate::world::generate::GenWorld;
use crate::world::scatter::{self, Spike};
use crate::world::trees::{self, Builder};
use crate::world::vanilla;
use b173_api::terrain::Random;

// the WorldGenerator objects structures keep and call, as the port's generator functions
#[derive(Clone, Copy, Default)]
pub enum Gen
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
    pub fn stalactite(block: Blk, size: f32, hang: bool) -> Gen
    {
        Gen::Spike(Spike { m_block: block.qualified(), m_size: size, m_longest: -1, m_gap: -1, m_hang: hang })
    }

    // TFGenCaveStalactite.makeRandomOreStalactite
    pub fn ore(random: &Random, hill: i32) -> Gen { Gen::Spike(*scatter::randomOre(random, hill)) }

    // WorldGenerator.generate
    pub fn generate(self, world: &GenWorld, random: &Random, x: i32, y: i32, z: i32) -> bool
    {
        let builder: Builder = Builder { m_world: world, m_notify: false };
        match self
        {
            Gen::Nothing => false,
            Gen::Spike(spike) => scatter::stalactite(world, random, x, y, z, &spike),
            Gen::Canopy => trees::canopyTree(&builder, random, x, y, z),
            Gen::TwilightOak => trees::twilightOak(&builder, random, x, y, z),
            Gen::Rainboak => trees::smallRainbow(&builder, random, x, y, z),
            Gen::Trees { minimum, meta } => vanilla::tree(world, random, x, y, z, minimum, meta),
            Gen::Birch => vanilla::birch(world, random, x, y, z),
            Gen::BigMushgloom => scatter::bigMushgloom(world, random, x, y, z),
            Gen::BigMushroom => vanilla::bigMushroom(world, random, x, y, z),
            Gen::Blob { block, size } =>
            {
                scatter::groundBlob(world, random, x, y, z, block.id(), size);
                true
            }
        }
    }
}

// TFGenerator.translateCoords
pub fn translateCoords(x: i32, y: i32, z: i32, distance: impl Into<f64>, angle: f64, tilt: f64) -> [i32; 3]
{
    trees::translate([x, y, z], distance.into(), angle, tilt)
}

// TFGenerator.getBresehnamArrayCoords
pub fn bresenham(x1: i32, y1: i32, z1: i32, x2: i32, y2: i32, z2: i32) -> Vec<[i32; 3]>
{
    trees::line([x1, y1, z1], [x2, y2, z2])
}
