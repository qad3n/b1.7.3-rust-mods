use super::generate::GenWorld;
use super::palette;
use crate::names::block;
use crate::tags;
use crate::world::palette::Placed;
use b173_api::prelude::*;
use b173_api::terrain::Random;

// the small twilight decorations, TFGenPlantRoots through TFGenCaveStalactite
const DIRECTIONS: [[i32; 3]; 6] = crate::math::NEIGHBORS;

mod reach
{
    // roots and berries hang down to here
    pub const ROOT_FLOOR: i32 = 5;
    // how far a hanging root wanders from its column
    pub const WANDER: i32 = 4;
    // the WorldGenerator scatter around a spot
    pub const SPREAD: i32 = 8;
    pub const RISE: i32 = 4;
}

mod odds
{
    // a root keeps growing down five times in six
    pub const ROOT: i32 = 6;
    pub const TROLLBER: i32 = 10;
    pub const THORN_FLOWER: i32 = 3;
    pub const THORN_ROSE: i32 = 50;
    pub const THORN_BRANCH: i32 = 3;
    pub const ROOT_BRANCH: i32 = 3;
    pub const ROOT_KNOT: i32 = 4;
}

fn wander(random: &Random, origin: i32) -> i32
{
    origin + random.nextIntBounded(reach::WANDER) - random.nextIntBounded(reach::WANDER)
}

fn scatter(random: &Random, x: i32, y: i32, z: i32) -> [i32; 3]
{
    [
        x + random.nextIntBounded(reach::SPREAD) - random.nextIntBounded(reach::SPREAD),
        y + random.nextIntBounded(reach::RISE) - random.nextIntBounded(reach::RISE),
        z + random.nextIntBounded(reach::SPREAD) - random.nextIntBounded(reach::SPREAD),
    ]
}

fn rootHold(world: &GenWorld, x: i32, y: i32, z: i32) -> bool
{
    let block: i32 = world.block(x, y, z);
    blockTagged(block, tags::SOIL) || blockIs(block, block::ROOT_STRAND) || blockIs(block, block::ROOT)
}

fn trollHold(world: &GenWorld, x: i32, y: i32, z: i32) -> bool
{
    let block: i32 = world.block(x, y, z);
    blockTagged(block, tags::STONY)
        || [block::TROLL_ROOT, block::TROLLBER, block::UNRIPE_TROLLBER].iter().any(|name| blockIs(block, name))
}

fn water(world: &GenWorld, x: i32, y: i32, z: i32) -> bool
{
    let block: i32 = world.block(x, y, z);
    blockTagged(block, "minecraft:water")
}

fn lava(world: &GenWorld, x: i32, y: i32, z: i32) -> bool
{
    let block: i32 = world.block(x, y, z);
    blockTagged(block, "minecraft:lava")
}

fn leafy(block: i32) -> bool { blockTagged(block, tags::LEAVES) }

// Material.ground and Material.rock, what TFGenerator.isAreaSuitable lets a ruin stand on
fn ground(world: &GenWorld, x: i32, y: i32, z: i32) -> bool
{
    let block: i32 = world.block(x, y, z);
    blockTagged(block, tags::SOIL) || blockTagged(block, tags::STONY)
}

// TFGenerator.isAreaSuitable, solid ground under the whole footprint and air above it
pub(crate) fn suitable(world: &GenWorld, x: i32, y: i32, z: i32, size: [i32; 3]) -> bool
{
    let [width, height, depth] = size;
    for cx in 0..width
    {
        for cz in 0..depth
        {
            if !ground(world, x + cx, y - 1, z + cz)
            {
                return false;
            }
            if (0..height).any(|cy| !world.isAir(x + cx, y + cy, z + cz))
            {
                return false;
            }
        }
    }

    true
}

// TFGenPlantRoots, root strands hanging under the dirt
pub(crate) fn plantRoots(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32)
{
    let strand: Placed = palette::plain(block::ROOT_STRAND);
    let (mut bx, mut bz): (i32, i32) = (x, z);

    for by in (reach::ROOT_FLOOR + 1..=y).rev()
    {
        if world.isAir(bx, by, bz) && rootHold(world, bx, by + 1, bz) && random.nextIntBounded(odds::ROOT) > 0
        {
            world.set(bx, by, bz, strand);
        }
        else
        {
            bx = wander(random, x);
            bz = wander(random, z);
        }
    }
}

// TFGenTorchBerries, one torchberry plant under the first root that takes it
pub(crate) fn torchBerries(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32)
{
    let berries: Placed = palette::plain(block::TORCHBERRY_PLANT);
    let (mut bx, mut bz): (i32, i32) = (x, z);

    for by in (reach::ROOT_FLOOR + 1..=y).rev()
    {
        if world.isAir(bx, by, bz) && rootHold(world, bx, by + 1, bz) && random.nextIntBounded(odds::ROOT) > 0
        {
            world.set(bx, by, bz, berries);
            return;
        }

        bx = wander(random, x);
        bz = wander(random, z);
    }
}

// TFGenTrollRoots, troll roots and the odd unripe trollber under the highland stone
pub(crate) fn trollRoots(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32)
{
    let (root, berry): (Placed, Placed) = (palette::plain(block::TROLL_ROOT), palette::plain(block::UNRIPE_TROLLBER));
    let (mut bx, mut bz): (i32, i32) = (x, z);

    for by in (reach::ROOT_FLOOR + 1..=y).rev()
    {
        if !world.isAir(bx, by, bz) || !trollHold(world, bx, by + 1, bz) || random.nextIntBounded(odds::ROOT) <= 0
        {
            bx = wander(random, x);
            bz = wander(random, z);
        }
        else if random.nextIntBounded(odds::TROLLBER) == 0
        {
            world.set(bx, by, bz, berry);
        }
        else
        {
            world.set(bx, by, bz, root);
        }
    }
}

// TFGenWoodRoots, a root line through the stone with an occasional knot of liveroot
pub(crate) fn woodRoots(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32)
{
    if !world.is(x, y, z, "minecraft:stone")
    {
        return;
    }

    let length: f32 = (random.nextFloat() * 6.0 + random.nextFloat() * 6.0 + 4.0).min(y as f32);
    let tilt: f32 = 0.6 + random.nextFloat() * 0.3;
    let angle: f32 = random.nextFloat();
    woodRoot(world, random, [x, y, z], [x, y, z], length, angle, tilt);
}

fn placeRoot(world: &GenWorld, at: [i32; 3], placed: (i32, i32))
{
    let builder: super::trees::Builder = super::trees::Builder { m_world: world, m_notify: false };
    if builder.rootFits(at[0], at[1], at[2])
    {
        world.set(at[0], at[1], at[2], placed);
    }
}

fn woodRoot(world: &GenWorld, random: &Random, origin: [i32; 3], from: [i32; 3], length: f32, angle: f32, tilt: f32)
{
    let limit: i32 = 6;
    let mut dest: [i32; 3] = super::trees::translate(from, f64::from(length), f64::from(angle), f64::from(tilt));
    dest[0] = dest[0].clamp(origin[0] - limit, origin[0] + limit);
    dest[2] = dest[2].clamp(origin[2] - limit, origin[2] + limit);
    if !world.is(dest[0], dest[1], dest[2], "minecraft:stone")
    {
        return;
    }

    let (root, ore): (Placed, Placed) = (palette::plain(block::ROOT), palette::plain(block::LIVEROOT));

    for at in super::trees::line(from, dest)
    {
        placeRoot(world, at, root);
    }

    let branchLength: f32 = 8.0;
    if length > branchLength && random.nextIntBounded(odds::ROOT_BRANCH) > 0
    {
        let next: [i32; 3] = super::trees::translate(from, f64::from(length / 2.0), f64::from(angle), f64::from(tilt));
        let nextAngle: f32 = (angle + 0.25 + random.nextFloat() * 0.5) % 1.0;
        let nextTilt: f32 = 0.6 + random.nextFloat() * 0.3;
        woodRoot(world, random, origin, next, length / 2.0, nextAngle, nextTilt);
    }

    let knotLength: f32 = 6.0;
    if length > knotLength && random.nextIntBounded(odds::ROOT_KNOT) == 0
    {
        let ball: [i32; 3] = super::trees::translate(from, f64::from(length / 2.0), f64::from(angle), f64::from(tilt));
        let across: [i32; 3] = super::trees::translate(ball, 1.5, f64::from((angle + 0.5) % 1.0), 0.75);
        for bx in [ball[0], across[0]]
        {
            for by in [ball[1], across[1]]
            {
                for bz in [ball[2], across[2]]
                {
                    placeRoot(world, [bx, by, bz], ore);
                }
            }
        }
    }
}

// TFGenMyceliumBlob, a disc that turns the ground over, beta has no mycelium so it lays dirt
pub(crate) fn groundBlob(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32, block: Placed, size: i32)
{
    let range: i32 = random.nextIntBounded(size - 2) + 2;
    let turns: [&str; 2] = ["minecraft:dirt", "minecraft:base_stone_overworld"];

    for bx in x - range..=x + range
    {
        for bz in z - range..=z + range
        {
            if (bx - x) * (bx - x) + (bz - z) * (bz - z) > range * range
            {
                continue;
            }

            for by in y - 1..=y + 1
            {
                let there: i32 = world.block(bx, by, bz);
                if turns.iter().any(|tag| blockTagged(there, tag))
                {
                    world.set(bx, by, bz, block);
                }
            }
        }
    }
}

// TFGenTallGrass, plants scattered around the first non leaf block below
pub(crate) fn plants(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32, plant: &'static str, amount: i32)
{
    let mut y: i32 = y;
    while leafy(world.block(x, y, z))
    {
        y -= 1;
        if y <= 0
        {
            break;
        }
    }

    let id: Placed = palette::plain(plant);

    for _ in 0..amount
    {
        let [bx, by, bz] = scatter(random, x, y, z);
        if world.isAir(bx, by, bz) && crate::blocks::plantStays(world.m_world, BlockPos::new(bx, by, bz), plant)
        {
            world.set(bx, by, bz, id);
        }
    }
}

// the vanilla WorldGenTallGrass placing one of the twilight plants
pub(crate) fn plantPatch(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32, plant: &'static str)
{
    let id: Placed = palette::plain(plant);

    let stays = |world: &GenWorld, bx: i32, by: i32, bz: i32|
    {
        crate::blocks::plantStays(world.m_world, BlockPos::new(bx, by, bz), plant)
    };
    super::vanilla::tallGrass(world, random, x, y, z, id, stays);
}

fn surroundedByAir(world: &GenWorld, x: i32, y: i32, z: i32) -> bool
{
    DIRECTIONS.iter().all(|[dx, dy, dz]| world.isAir(x + dx, y + dy, z + dz))
}

const HANG: i32 = 8;

fn holdsLamp(world: &GenWorld, x: i32, y: i32, z: i32) -> bool
{
    let block: i32 = world.block(x, y, z);
    block != 0 && (world.info(block).solid || leafy(block))
}

// TFGenHangingLamps, a firefly jar on a fence chain under the canopy
pub(crate) fn hangingLamp(world: &GenWorld, x: i32, y: i32, z: i32)
{
    let below: i32 = 4;
    let canopy: bool = (1..HANG).any(|cy| holdsLamp(world, x, y + cy, z));
    let clear: bool = (1..below).all(|cy| !world.isOpaque(x, y - cy, z));
    if !world.isAir(x, y, z) || !surroundedByAir(world, x, y, z) || !canopy || !clear
    {
        return;
    }

    world.set(x, y, z, palette::plain(block::FIREFLY_JAR));

    let fence: Placed = palette::plain("minecraft:fence");

    for cy in 1..HANG
    {
        if holdsLamp(world, x, y + cy, z)
        {
            break;
        }

        world.set(x, y + cy, z, fence);
    }
}

// TFGenLampposts, a fence post on the grass topped with a firefly jar
pub(crate) fn lamppost(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32)
{
    if !world.is(x, y - 1, z, "minecraft:grass")
    {
        return;
    }
    let height: i32 = 1 + random.nextIntBounded(4);
    if !(0..height).all(|dy| world.isReplaceable(x, y + dy, z))
    {
        return;
    }

    let fence: Placed = palette::plain("minecraft:fence");

    for dy in 0..height
    {
        world.set(x, y + dy, z, fence);
    }

    world.set(x, y + height, z, palette::plain(block::FIREFLY_JAR));
}

// TFGenFireJet, a smoker or fire jet in the grass over a pocket of lava ringed in stone
pub(crate) fn fireJet(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32, jet: &'static str)
{
    let (stone, flowing): (Placed, Placed) =
        (palette::plain("minecraft:stone"), palette::plain("minecraft:flowing_lava"));

    for _ in 0..4
    {
        let [bx, by, bz] = scatter(random, x, y, z);
        let grassy: bool = [[0, 0], [1, 0], [-1, 0], [0, 1], [0, -1]]
            .iter()
            .all(|[dx, dz]| world.is(bx + dx, by - 1, bz + dz, "minecraft:grass"));

        if !world.isAir(bx, by, bz) || world.height(bx, bz) > by || !grassy
        {
            continue;
        }

        world.set(bx, by - 1, bz, palette::plain(jet));

        for rx in -2..=2i32
        {
            for rz in -2..=2i32
            {
                if rx.abs() <= 1 && rz.abs() <= 1
                {
                    world.set(bx + rx, by - 2, bz + rz, flowing);
                }
                else if !lava(world, bx + rx, by - 2, bz + rz)
                {
                    world.set(bx + rx, by - 2, bz + rz, stone);
                }
                world.set(bx + rx, by - 3, bz + rz, stone);
            }
        }
    }
}

// spawns a mob during decoration facing a random way, TFGenPenguins and TFGenMonolith
pub(crate) fn critters(world: &GenWorld, random: &Random, x: i32, z: i32, kind: &str, count: i32)
{
    for _ in 0..count
    {
        let bx: i32 = x + random.nextIntBounded(reach::SPREAD) - random.nextIntBounded(reach::SPREAD);
        let bz: i32 = z + random.nextIntBounded(reach::SPREAD) - random.nextIntBounded(reach::SPREAD);
        let by: i32 = world.topSolid(bx, bz);
        let yaw: f32 = random.nextFloat() * 360.0;
        let _ = crate::entities::spawnAt(world.m_world, kind, [f64::from(bx), f64::from(by), f64::from(bz)], yaw);
    }
}

// TFGenThorns, a thorn bush branching from the ground
pub(crate) fn thorns(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32)
{
    let length: i32 = 2 + random.nextIntBounded(4);
    let longest: i32 = 2 + random.nextIntBounded(4) + random.nextIntBounded(4) + random.nextIntBounded(4);
    thornArm(world, random, [x, y, z], length, DIRECTIONS[1], longest, [x, y, z]);
}

fn thornSpot(world: &GenWorld, at: [i32; 3]) -> bool
{
    let block: i32 = world.block(at[0], at[1], at[2]);
    block == 0 || leafy(block)
}

fn step(from: [i32; 3], dir: [i32; 3], distance: i32) -> [i32; 3]
{
    [from[0] + dir[0] * distance, from[1] + dir[1] * distance, from[2] + dir[2] * distance]
}

fn thornArm(
    world: &GenWorld,
    random: &Random,
    from: [i32; 3],
    length: i32,
    dir: [i32; 3],
    longest: i32,
    origin: [i32; 3],
)
{
    let spread: i32 = 7;
    let axis: &'static str = crate::blocks::AXES[crate::blocks::axisOf(dir)];
    let thorn: (i32, i32) = palette::with(block::BROWN_THORNS, "axis", axis);
    let (leaves, rose): (Placed, Placed) = (palette::plain(block::THORN_LEAVES), palette::plain(block::THORN_ROSE));
    let mut complete: bool = false;

    for i in 0..length
    {
        let at: [i32; 3] = step(from, dir, i);
        if (at[0] - origin[0]).abs() >= spread || (at[2] - origin[2]).abs() >= spread || !thornSpot(world, at)
        {
            break;
        }

        world.set(at[0], at[1], at[2], thorn);

        if i < length - 1
        {
            continue;
        }

        complete = true;

        let tip: [i32; 3] = step(at, dir, 1);
        if random.nextIntBounded(odds::THORN_FLOWER) == 0 && world.isAir(tip[0], tip[1], tip[2])
        {
            let flower: Placed = if random.nextIntBounded(odds::THORN_ROSE) > 0
            {
                leaves
            }
            else
            {
                rose
            };
            world.set(tip[0], tip[1], tip[2], flower);
        }
    }

    if !complete
    {
        return;
    }

    let branchFrom = |offset: i32|
    {
        let next: [i32; 3] = DIRECTIONS[random.nextIntBounded(DIRECTIONS.len() as i32) as usize];
        (next, step(step(from, dir, offset), next, 1))
    };

    if longest > 1
    {
        let (next, start): ([i32; 3], [i32; 3]) = branchFrom(length - 1);
        thornArm(world, random, start, 1 + random.nextIntBounded(longest), next, longest - 1, origin);
    }

    if length > 3 && random.nextIntBounded(odds::THORN_BRANCH) == 0
    {
        let middle: i32 = random.nextIntBounded(length);
        let (next, start): ([i32; 3], [i32; 3]) = branchFrom(middle);
        thornArm(world, random, start, 1 + random.nextIntBounded(longest), next, longest - 1, origin);
    }

    if length > 3 && random.nextIntBounded(odds::THORN_BRANCH) == 0
    {
        let middle: i32 = random.nextIntBounded(length);
        let (_, leaf): ([i32; 3], [i32; 3]) = branchFrom(middle);
        if world.isAir(leaf[0], leaf[1], leaf[2])
        {
            world.set(leaf[0], leaf[1], leaf[2], leaves);
        }
    }
}

// TFGenHugeLilyPad, two by two pads on still water turned by a hash of their spot
pub(crate) fn lilyPads(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32)
{
    let pieces: [[i32; 2]; 4] = crate::blocks::PAD_PIECES;

    for _ in 0..10
    {
        let [bx, by, bz] = scatter(random, x, y, z);
        let (bx, bz): (i32, i32) = (bx >> 1 << 1, bz >> 1 << 1);
        if !pieces.iter().all(|[dx, dz]| world.isAir(bx + dx, by, bz + dz) && water(world, bx + dx, by - 1, bz + dz))
        {
            continue;
        }

        let turn: i32 = crate::blocks::padTurn(bx, bz);

        for (piece, [dx, dz]) in pieces.iter().enumerate()
        {
            let quarter: [(&str, &str); 2] =
                [("piece", crate::blocks::QUARTERS[piece]), ("orient", crate::blocks::QUARTERS[turn as usize])];
            world.set(bx + dx, by, bz + dz, palette::withTwo(block::HUGE_LILY_PAD, quarter));
        }
    }
}

// TFGenHugeWaterLily
pub(crate) fn waterLilies(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32)
{
    let lily: Placed = palette::plain(block::HUGE_WATER_LILY);

    for _ in 0..4
    {
        let [bx, by, bz] = scatter(random, x, y, z);
        if world.isAir(bx, by, bz) && water(world, bx, by - 1, bz)
        {
            world.set(bx, by, bz, lily);
        }
    }
}

// TFGenBigMushgloom
pub(crate) fn bigMushgloom(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32) -> bool
{
    let height: i32 = 3 + random.nextIntBounded(2) + random.nextIntBounded(2);
    if !suitable(world, x - 1, y, z - 1, [3, height, 3])
    {
        return false;
    }
    let below: i32 = world.block(x, y - 1, z);
    if !blockTagged(below, "minecraft:dirt")
    {
        return false;
    }

    let gloom: Placed = palette::plain(block::HUGE_GLOOM);
    let stem: (i32, i32) = palette::with(block::HUGE_GLOOM, "part", "stem");

    for dy in 0..height - 2
    {
        world.set(x, y + dy, z, stem);
    }

    let cap = |cy: i32|
    {
        for dx in -1..=1
        {
            for dz in -1..=1
            {
                world.set(x + dx, cy, z + dz, gloom);
            }
        }
    };
    cap(y + height - 2);

    if random.nextBoolean()
    {
        cap(y + height - 1);
    }

    true
}

// one kind of TFGenCaveStalactite
#[derive(Clone, Copy)]
pub(crate) struct Spike
{
    pub m_block: &'static str,
    pub m_size: f32,
    pub m_longest: i32,
    pub m_gap: i32,
    pub m_hang: bool,
}

const fn ore(block: &'static str, size: f32, longest: i32, gap: i32) -> Spike
{
    Spike { m_block: block, m_size: size, m_longest: longest, m_gap: gap, m_hang: true }
}

pub(crate) mod spikes
{
    use super::{Spike, ore};

    pub(crate) const DIAMOND: Spike = ore("minecraft:diamond_ore", 0.5, 4, 16);
    pub(crate) const LAPIS: Spike = ore("minecraft:lapis_ore", 0.8, 8, 1);
    // beta has no emerald ore, the emerald spike is a smaller diamond one
    pub(crate) const EMERALD: Spike = ore("minecraft:diamond_ore", 0.5, 3, 12);
    pub(crate) const GOLD: Spike = ore("minecraft:gold_ore", 0.6, 6, 1);
    pub(crate) const REDSTONE: Spike = ore("minecraft:redstone_ore", 0.8, 8, 1);
    pub(crate) const IRON: Spike = ore("minecraft:iron_ore", 0.7, 8, 1);
    pub(crate) const COAL: Spike = ore("minecraft:coal_ore", 0.8, 12, 1);
    pub(crate) const GLOWSTONE: Spike = ore("minecraft:glowstone", 0.5, 8, 1);
    pub(crate) const STONE: Spike =
        Spike { m_block: "minecraft:stone", m_size: 1.0, m_longest: -1, m_gap: -1, m_hang: false };
}

// TFGenCaveStalactite.makeRandomOreStalactite, richer ores in the bigger hollow hills
pub(crate) fn randomOre(random: &Random, hill: i32) -> &'static Spike
{
    if hill >= 3 || hill >= 2 && random.nextIntBounded(5) == 0
    {
        match random.nextIntBounded(13)
        {
            0 | 1 => return &spikes::DIAMOND,
            2 | 3 => return &spikes::LAPIS,
            4 => return &spikes::EMERALD,
            _ =>
            {}
        }
    }

    if hill >= 2 || hill >= 1 && random.nextIntBounded(5) == 0
    {
        match random.nextIntBounded(6)
        {
            0 => return &spikes::GOLD,
            1 | 2 => return &spikes::REDSTONE,
            _ =>
            {}
        }
    }

    match random.nextIntBounded(5)
    {
        0 | 1 => &spikes::IRON,
        2 | 3 => &spikes::COAL,
        _ => &spikes::GLOWSTONE,
    }
}

// Material.ground and Material.rock, grass is neither
fn earthy(block: i32) -> bool { blockIs(block, "minecraft:dirt") || blockTagged(block, tags::STONY) }

// TFGenCaveStalactite.generate, a spike from the ceiling or floor of the cave around the spot
pub(crate) fn stalactite(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32, spike: &Spike) -> bool
{
    let top: i32 = super::terrain::height();
    let Some(ceiling) = (y..top).find(|ty| !world.isAir(x, *ty, z))
    else
    {
        return false;
    };
    if !earthy(world.block(x, ceiling, z))
    {
        return false;
    }

    let lowest: i32 = 4;
    let mut floor: i32 = -1;

    for ty in (lowest + 1..=y).rev()
    {
        let block: i32 = world.block(x, ty, z);
        if block == 0
        {
            continue;
        }
        if !earthy(block) && !spike.m_hang && !water(world, x, ty, z) && !lava(world, x, ty, z)
        {
            return false;
        }

        floor = ty;
        break;
    }

    let mut length: i32 = ((ceiling - floor) as f32 * spike.m_size * random.nextFloat()) as i32;

    if spike.m_longest > -1
    {
        length = length.min(spike.m_longest);
    }

    if spike.m_gap > -1 && ceiling - floor - length < spike.m_gap
    {
        return false;
    }

    makeSpike(world, random, [x, if spike.m_hang { ceiling } else { floor }, z], length, spike);
    true
}

// TFGenOutsideStalagmite, a stone spike standing on open ground
pub(crate) fn stalagmite(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32) -> bool
{
    let length: i32 = random.nextIntBounded(10) + 5;
    if !suitable(world, x, y, z, [1, length, 1])
    {
        return false;
    }

    makeSpike(world, random, [x, y - 1, z], length, &spikes::STONE);
    true
}

fn makeSpike(world: &GenWorld, random: &Random, at: [i32; 3], longest: i32, spike: &Spike)
{
    let [x, y, z] = at;
    let block: Placed = palette::plain(spike.m_block);
    let diameter: i32 = (f64::from(longest) / 4.5) as i32;
    let dir: i32 = if spike.m_hang { -1 } else { 1 };

    for dx in -diameter..=diameter
    {
        for dz in -diameter..=diameter
        {
            let (ax, az): (i32, i32) = (dx.abs(), dz.abs());
            let distance: i32 = (f64::from(ax.max(az)) + f64::from(ax.min(az)) * 0.5) as i32;
            let mut length: i32 = if distance == 0
            {
                longest
            }
            else
            {
                random.nextIntBounded(((f64::from(longest) / (f64::from(distance) + 0.25)) as i32).max(1))
            };

            if !world.isSolid(x + dx, y - dir, z + dz)
            {
                length = 0;
            }

            let mut dy: i32 = 0;
            while dy != length * dir
            {
                world.set(x + dx, y + dy, z + dz, block);
                dy += dir;
            }
        }
    }
}
