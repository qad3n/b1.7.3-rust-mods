use super::gen_world::GenWorld;
use crate::ids::Palette;
use b173_api::terrain::Random;

mod skyroot
{
    pub const MIN_HEIGHT: i32 = 4;
    pub const HEIGHT_SPREAD: i32 = 3;
    pub const CANOPY_LAYERS: i32 = 3;
}

mod oak
{
    pub const HEIGHT: i32 = 9;
    pub const RADIUS: i32 = 3;
    pub const CANOPY_LOW: i32 = 5;
    pub const CANOPY_HIGH: i32 = 12;
    pub const CANOPY_CENTER: i32 = 8;
    pub const CANOPY_REACH: i32 = 12;
    pub const CANOPY_JITTER: i32 = 5;
    pub const BRANCH_START: i32 = 4;
    pub const BRANCH_CHANCE: i32 = 3;
    pub const BRANCH_SLANT: i32 = 4;
    // grass meta the golden dungeon writes under its tree spots
    pub const DUNGEON_GRASS_META: i32 = 3;
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tree
{
    Skyroot,
    GoldenOak,
}

impl Tree
{
    pub(crate) fn generate(self, world: &GenWorld, random: &Random, x: i32, y: i32, z: i32) -> bool
    {
        match self
        {
            Self::Skyroot => skyrootTree(world, random, x, y, z),
            Self::GoldenOak => goldenOakTree(world, random, x, y, z),
        }
    }
}

fn onSoil(world: &GenWorld, x: i32, y: i32, z: i32) -> bool
{
    let below: i32 = world.block(x, y - 1, z);
    below == world.m_palette.m_grass || below == world.m_palette.m_dirt
}

// AetherGenSkyrootTree.generate
pub(crate) fn skyrootTree(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32) -> bool
{
    let palette: &Palette = &world.m_palette;
    let height: i32 = random.nextIntBounded(skyroot::HEIGHT_SPREAD) + skyroot::MIN_HEIGHT;

    if !onSoil(world, x, y, z)
    {
        return false;
    }

    world.set(x, y - 1, z, palette.m_dirt);

    for leafY in y - skyroot::CANOPY_LAYERS + height..=y + height
    {
        let fromTop: i32 = leafY - (y + height);
        let radius: i32 = 1 - fromTop / 2;
        for leafX in x - radius..=x + radius
        {
            for leafZ in z - radius..=z + radius
            {
                let corner: bool = (leafX - x).abs() == radius && (leafZ - z).abs() == radius;
                // the Java skips the random draw on non corners, so it short circuits the same way
                let keep: bool = !corner || random.nextIntBounded(2) != 0 && fromTop != 0;
                if keep && !world.isOpaque(leafX, leafY, leafZ)
                {
                    world.set(leafX, leafY, leafZ, palette.m_skyrootLeaves);
                }
            }
        }
    }

    for trunk in 0..height
    {
        let block: i32 = world.block(x, y + trunk, z);
        if block == 0 || block == palette.m_skyrootLeaves
        {
            world.set(x, y + trunk, z, palette.m_skyrootLog);
        }
    }

    true
}

// AetherGenOakTree.branch, a short diagonal arm mirrored on the far side of the trunk
fn branch(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32, slant: i32)
{
    let palette: &Palette = &world.m_palette;
    let directionX: i32 = random.nextIntBounded(3) - 1;
    let directionZ: i32 = random.nextIntBounded(3) - 1;
    let (mut x, mut y, mut z): (i32, i32, i32) = (x, y, z);
    let (mut mirrorX, mut mirrorZ): (i32, i32) = (x, z);
    let mut n: i32 = 0;
    // the loop bound draws a new random on every check, as the Java condition does
    while n < random.nextIntBounded(2)
    {
        x += directionX;
        y += slant;
        z += directionZ;
        mirrorX -= directionX;
        mirrorZ -= directionZ;

        if world.block(x, y, z) == palette.m_goldenOakLeaves
        {
            world.set(x, y, z, palette.m_goldenOakLog);
            world.set(mirrorX, y, mirrorZ, palette.m_goldenOakLog);
        }

        n += 1;
    }
}

// AetherGenOakTree.generate body, shared with the dungeon variant
fn oakBody(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32)
{
    let palette: &Palette = &world.m_palette;

    for leafX in x - oak::RADIUS..x + oak::RADIUS + 1
    {
        for leafY in y + oak::CANOPY_LOW..y + oak::CANOPY_HIGH
        {
            for leafZ in z - oak::RADIUS..z + oak::RADIUS + 1
            {
                let dx: i32 = leafX - x;
                let dy: i32 = leafY - y - oak::CANOPY_CENTER;
                let dz: i32 = leafZ - z;
                if dx * dx + dy * dy + dz * dz < oak::CANOPY_REACH + random.nextIntBounded(oak::CANOPY_JITTER)
                    && world.isAir(leafX, leafY, leafZ)
                {
                    world.set(leafX, leafY, leafZ, palette.m_goldenOakLeaves);
                }
            }
        }
    }

    for n in 0..oak::HEIGHT
    {
        if n > oak::BRANCH_START && random.nextIntBounded(oak::BRANCH_CHANCE) > 0
        {
            branch(world, random, x, y + n, z, n / oak::BRANCH_SLANT - 1);
        }

        world.set(x, y + n, z, palette.m_goldenOakLog);
    }
}

// AetherGenOakTree.generate
pub(crate) fn goldenOakTree(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32) -> bool
{
    if !onSoil(world, x, y, z)
    {
        return false;
    }

    oakBody(world, random, x, y, z);
    true
}

// AetherGenDungeonOakTree.generate, grows only where the golden dungeon marked its grass
pub(crate) fn dungeonOakTree(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32) -> bool
{
    if world.block(x, y - 1, z) != world.m_palette.m_grass || world.meta(x, y - 1, z) != oak::DUNGEON_GRASS_META
    {
        return false;
    }

    oakBody(world, random, x, y, z);
    true
}
