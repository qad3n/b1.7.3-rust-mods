use crate::world::biomes;
use b173_api::prelude::*;

// leaves pick their tint from one of these, BlockTFLeaves and BlockTFMagicLeaves colorMultiplier
#[derive(Clone, Copy)]
pub enum Tint
{
    Foliage,
    Canopy,
    Mangrove,
    Rainbow,
    Time,
    Transformation,
    Mining,
    Sorting,
}

mod channel
{
    pub const MAX: i32 = 255;
    pub const MASK: u32 = 0xFF;
    // a stripe value crossing this bit runs back down
    pub const FOLD_BIT: i32 = 256;
    pub const RED_SHIFT: u32 = 16;
    pub const GREEN_SHIFT: u32 = 8;
    pub const WHITE: u32 = 0xFFFFFF;
}

// the 3x3 columns averaged for a leaf tint
const AROUND: u32 = 9;

mod blend
{
    pub const CANOPY: u32 = 0x469A66;
    pub const MANGROVE: u32 = 0xC0E694;
    // the low bit of every channel is cleared before two colors are averaged
    pub const HALF_MASK: u32 = 0xFEFEFE;
}

// magic leaves fade between a spring and a fall color along a diagonal stripe pattern
struct Fade
{
    m_tint: Tint,
    m_step: [i32; 3],
    m_spring: [f32; 3],
    m_fall: [f32; 3],
}

const FADES: [Fade; 4] = [
    Fade { m_tint: Tint::Time, m_step: [16, 16, 16], m_spring: [106.0, 156.0, 23.0], m_fall: [251.0, 108.0, 27.0] },
    Fade
    {
        m_tint: Tint::Transformation,
        m_step: [27, 63, 39],
        m_spring: [108.0, 204.0, 234.0],
        m_fall: [96.0, 107.0, 121.0],
    },
    Fade { m_tint: Tint::Mining, m_step: [31, 33, 32], m_spring: [252.0, 241.0, 68.0], m_fall: [237.0, 172.0, 9.0] },
    Fade { m_tint: Tint::Sorting, m_step: [63, 63, 63], m_spring: [54.0, 76.0, 3.0], m_fall: [168.0, 199.0, 43.0] },
];

// a value climbing in steps of 256 folds back down so the pattern ping pongs
pub fn fold(value: i32) -> i32
{
    let folded: i32 = if value & channel::FOLD_BIT != 0
    {
        channel::MAX - (value & channel::MAX)
    }
    else
    {
        value
    };
    folded & channel::MAX
}

pub fn rgb(red: i32, green: i32, blue: i32) -> u32
{
    let mask: i32 = channel::MAX;
    ((red & mask) << channel::RED_SHIFT | (green & mask) << channel::GREEN_SHIFT | (blue & mask)) as u32
}

pub fn channels(color: u32) -> [u32; 3]
{
    [
        (color >> channel::RED_SHIFT) & channel::MASK,
        (color >> channel::GREEN_SHIFT) & channel::MASK,
        color & channel::MASK,
    ]
}

pub fn average(color: u32, with: u32) -> u32 { ((color & blend::HALF_MASK) + with) / 2 }

// the biome foliage color averaged over the 3x3 columns around the block
pub fn foliageAround(world: World, pos: BlockPos) -> u32
{
    let mut sums: [u32; 3] = [0; 3];
    for dz in -1..=1
    {
        for dx in -1..=1
        {
            let color: [u32; 3] = channels(biomes::foliage(world, pos.offset(dx, 0, dz)));
            for (sum, value) in sums.iter_mut().zip(color)
            {
                *sum += value;
            }
        }
    }
    rgb((sums[0] / AROUND) as i32, (sums[1] / AROUND) as i32, (sums[2] / AROUND) as i32)
}

fn rainbow(pos: BlockPos) -> u32
{
    let red: i32 = fold(pos.x * 32 + pos.y * 16);
    let blue: i32 = fold(pos.y * 32 + pos.z * 16) ^ channel::MAX;
    let green: i32 = fold(pos.x * 16 + pos.z * 32);
    // the source packs these as red blue green
    rgb(red, blue, green)
}

fn fade(row: &Fade, pos: BlockPos) -> u32
{
    let value: i32 = fold(pos.x * row.m_step[0] + pos.y * row.m_step[1] + pos.z * row.m_step[2]);
    let fall: f32 = value as f32 / channel::MAX as f32;
    let spring: f32 = (channel::MAX - value) as f32 / channel::MAX as f32;
    let mix = |index: usize| (spring * row.m_spring[index] + fall * row.m_fall[index]) as i32;
    rgb(mix(0), mix(1), mix(2))
}

pub fn tint(kind: Tint, world: World, pos: BlockPos) -> u32
{
    match kind
    {
        Tint::Foliage => foliageAround(world, pos),
        Tint::Canopy => average(foliageAround(world, pos), blend::CANOPY),
        Tint::Mangrove => average(foliageAround(world, pos), blend::MANGROVE),
        Tint::Rainbow => rainbow(pos),
        _ => FADES.iter().find(|row| row.m_tint as u8 == kind as u8).map_or(channel::WHITE, |row| fade(row, pos)),
    }
}

// TFBlocks towerwood shades every plank a little by position
pub fn towerwood(pos: BlockPos) -> u32
{
    // planks keep at least half brightness
    let dimmest: i32 = 128;
    let value: i32 = (fold(pos.x * 31 + pos.y * 15 + pos.z * 33) >> 1) | dimmest;
    rgb(value, value, value)
}

// BlockTFAuroraBrick.colorMultiplier
pub fn aurora(x: i32, z: i32) -> u32
{
    let floor: i32 = 128;
    let red: i32 = 16;
    let blue: i32 = (fold(x * 12 + z * 6) ^ channel::MAX) & channel::MAX;
    let mut green: i32 = fold(x * 4 + z * 8);
    if green + blue < floor
    {
        green = floor - blue;
    }
    rgb(red, blue, green)
}

// BlockTFAuroraPillar.colorMultiplier, the aurora color washed out through hue saturation brightness
pub fn pale(color: u32) -> u32
{
    let [red, green, blue] = channels(color);
    // the source hands the channels over as red blue green
    let (hue, saturation, brightness): (f32, f32, f32) = crate::math::toHsb(red, blue, green);
    let lift: f32 = 0.4;
    let ceiling: f32 = 0.9;
    crate::math::fromHsb(hue, saturation * 0.5, (brightness + lift).min(ceiling))
}
