use super::trees::{Builder, Placed, Woods};
use b173_api::terrain::Random;

// TFGenLargeWinter, a two wide spruce of the snowy forest with layered leaves and rows of branches
mod winter
{
    pub const HEIGHT: i32 = 35;
    pub const EXTRA: i32 = 10;
    pub const OFF_GROUND: i32 = 3;
    pub const SPRUCE: i32 = 1;
}

// TFGenLargeWinter.leafRadius with the source's leaf function one
fn leafRadius(height: i32, dy: i32) -> i32 { (4.0 * dy as f32 / height as f32 + 0.75 * dy as f32 % 3.0) as i32 }

// beta logs have no orientation, so the branch rows lie as upright spruce logs
fn branches(builder: &Builder, x: i32, y: i32, z: i32, radius: i32, wood: Placed)
{
    let length: i32 = if radius > 4 { radius - 1 } else { radius - 2 };
    for i in 1..=length
    {
        let cells: [[i32; 2]; 4] = if y % 2 == 0
        {
            [[x - i, z], [x, z + 1 + i], [x + 1 + i, z + 1], [x + 1, z - i]]
        }
        else
        {
            [[x - i, z + 1], [x + 1, z + 1 + i], [x + 1 + i, z], [x, z - i]]
        };
        for [bx, bz] in cells
        {
            builder.set(bx, y, bz, wood);
        }
    }
}

pub fn tree(builder: &Builder, random: &Random, x: i32, y: i32, z: i32) -> bool
{
    let mut height: i32 = winter::HEIGHT;
    if random.nextIntBounded(3) == 0
    {
        height += random.nextIntBounded(winter::EXTRA);
        if random.nextIntBounded(8) == 0
        {
            height += random.nextIntBounded(winter::EXTRA);
        }
    }
    let world: &super::generate::GenWorld = builder.m_world;
    let below: i32 = world.block(x, y - 1, z);
    let grounded: bool = below == world.id("minecraft:grass") || below == world.id("minecraft:dirt");
    if !grounded || y >= super::terrain::height() - height
    {
        return false;
    }
    let log: Placed = (world.id("minecraft:log"), winter::SPRUCE);
    let woods: Woods = Woods
    {
        m_trunk: log,
        m_branch: log,
        m_leaf: (world.id("minecraft:leaves"), winter::SPRUCE),
        m_root: super::trees::root(),
    };
    for dy in 0..height
    {
        for [dx, dz] in [[0, 0], [1, 0], [0, 1], [1, 1]]
        {
            builder.set(x + dx, y + dy, z + dz, log);
        }
    }
    for dy in 0..height
    {
        let radius: i32 = leafRadius(height, dy);
        let layer: i32 = y + winter::OFF_GROUND + height - dy;
        builder.leafCircle2(x, layer, z, radius, woods.m_leaf);
        branches(builder, x, layer, z, radius, log);
    }
    let roots: i32 = 4 + random.nextIntBounded(3);
    let offset: f32 = random.nextFloat();
    for index in 0..roots
    {
        builder.buildRoot(x, y, z, f64::from(offset), index, &woods);
    }
    true
}
