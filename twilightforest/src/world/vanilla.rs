use super::generate::GenWorld;
use super::palette;
use crate::tags;
use crate::world::palette::Placed;
use b173_api::prelude::*;
use b173_api::terrain::{Random, math};

// the vanilla world generators the twilight decorator calls, ported from beta where beta has them
const TOP: i32 = 128;

fn grassOrDirt(world: &GenWorld, x: i32, y: i32, z: i32) -> bool
{
    world.is(x, y, z, "minecraft:grass") || world.is(x, y, z, "minecraft:dirt")
}

fn clearForTree(world: &GenWorld, x: i32, y: i32, z: i32, height: i32, widthAt: impl Fn(i32) -> i32) -> bool
{
    for cy in y..=y + 1 + height
    {
        let width: i32 = widthAt(cy);
        for cx in x - width..=x + width
        {
            for cz in z - width..=z + width
            {
                if !(0..TOP).contains(&cy)
                {
                    return false;
                }
                let there: i32 = world.block(cx, cy, cz);
                if there != 0 && !blockIs(there, "minecraft:leaves")
                {
                    return false;
                }
            }
        }
    }

    true
}

// WorldGenTrees and WorldGenForest, the round oak or birch
pub(crate) fn tree(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32, minimum: i32, meta: i32) -> bool
{
    let height: i32 = random.nextIntBounded(3) + minimum;
    if y < 1 || y + height + 1 > TOP
    {
        return false;
    }

    let width = |cy: i32|
    {
        if cy == y
        {
            0
        }
        else if cy >= y + 1 + height - 2
        {
            2
        }
        else
        {
            1
        }
    };
    if !clearForTree(world, x, y, z, height, width) || !grassOrDirt(world, x, y - 1, z) || y >= TOP - height - 1
    {
        return false;
    }

    let (leaves, log): (Placed, Placed) =
        (palette::meta("minecraft:leaves", meta), palette::meta("minecraft:log", meta));

    world.set(x, y - 1, z, palette::plain("minecraft:dirt"));

    for cy in y - 3 + height..=y + height
    {
        let layer: i32 = cy - (y + height);
        let radius: i32 = 1 - layer / 2;
        for cx in x - radius..=x + radius
        {
            for cz in z - radius..=z + radius
            {
                if (cx - x).abs() == radius && (cz - z).abs() == radius && (random.nextIntBounded(2) == 0 || layer == 0)
                {
                    continue;
                }

                if !world.isOpaque(cx, cy, cz)
                {
                    world.set(cx, cy, cz, leaves);
                }
            }
        }
    }

    for dy in 0..height
    {
        let there: i32 = world.block(x, y + dy, z);
        if there == 0 || blockIs(there, "minecraft:leaves")
        {
            world.set(x, y + dy, z, log);
        }
    }

    true
}

pub(crate) fn oak(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32) -> bool
{
    tree(world, random, x, y, z, 4, 0)
}

pub(crate) fn birch(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32) -> bool
{
    tree(world, random, x, y, z, 5, 2)
}

fn spruceFits(world: &GenWorld, x: i32, y: i32, z: i32, height: i32, trunk: i32, radius: i32) -> bool
{
    clearForTree(world, x, y, z, height, |cy| if cy - y < trunk { 0 } else { radius })
}

// WorldGenTaiga1, the lean spruce
pub(crate) fn taiga1(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32) -> bool
{
    let height: i32 = random.nextIntBounded(5) + 7;
    let trunk: i32 = height - random.nextIntBounded(2) - 3;
    let radiusMax: i32 = 1 + random.nextIntBounded(height - trunk + 1);
    if y < 1 || y + height + 1 > TOP || !spruceFits(world, x, y, z, height, trunk, radiusMax)
    {
        return false;
    }
    if !grassOrDirt(world, x, y - 1, z) || y >= TOP - height - 1
    {
        return false;
    }

    // spruce leaves and logs are metadata one
    let (leaves, log): (Placed, Placed) = (palette::meta("minecraft:leaves", 1), palette::meta("minecraft:log", 1));

    world.set(x, y - 1, z, palette::plain("minecraft:dirt"));

    let mut radius: i32 = 0;

    for cy in (y + trunk..=y + height).rev()
    {
        for cx in x - radius..=x + radius
        {
            for cz in z - radius..=z + radius
            {
                let corner: bool = (cx - x).abs() == radius && (cz - z).abs() == radius && radius > 0;
                if !corner && !world.isOpaque(cx, cy, cz)
                {
                    world.set(cx, cy, cz, leaves);
                }
            }
        }

        if radius >= 1 && cy == y + trunk + 1
        {
            radius -= 1;
        }
        else if radius < radiusMax
        {
            radius += 1;
        }
    }

    for dy in 0..height - 1
    {
        let there: i32 = world.block(x, y + dy, z);
        if there == 0 || blockIs(there, "minecraft:leaves")
        {
            world.set(x, y + dy, z, log);
        }
    }

    true
}

// WorldGenTaiga2, the layered spruce
pub(crate) fn taiga2(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32) -> bool
{
    let height: i32 = random.nextIntBounded(4) + 6;
    let bare: i32 = 1 + random.nextIntBounded(2);
    let leafy: i32 = height - bare;
    let radiusMax: i32 = 2 + random.nextIntBounded(2);
    if y < 1 || y + height + 1 > TOP || !spruceFits(world, x, y, z, height, bare, radiusMax)
    {
        return false;
    }
    if !grassOrDirt(world, x, y - 1, z) || y >= TOP - height - 1
    {
        return false;
    }

    let (leaves, log): (Placed, Placed) = (palette::meta("minecraft:leaves", 1), palette::meta("minecraft:log", 1));

    world.set(x, y - 1, z, palette::plain("minecraft:dirt"));

    let mut radius: i32 = random.nextIntBounded(2);
    let mut limit: i32 = 1;
    let mut restart: i32 = 0;

    for layer in 0..=leafy
    {
        let cy: i32 = y + height - layer;

        for cx in x - radius..=x + radius
        {
            for cz in z - radius..=z + radius
            {
                let corner: bool = (cx - x).abs() == radius && (cz - z).abs() == radius && radius > 0;
                if !corner && !world.isOpaque(cx, cy, cz)
                {
                    world.set(cx, cy, cz, leaves);
                }
            }
        }

        if radius >= limit
        {
            radius = restart;
            restart = 1;
            limit = (limit + 1).min(radiusMax);
        }
        else
        {
            radius += 1;
        }
    }

    let cut: i32 = random.nextIntBounded(3);
    for dy in 0..height - cut
    {
        let there: i32 = world.block(x, y + dy, z);
        if there == 0 || blockIs(there, "minecraft:leaves")
        {
            world.set(x, y + dy, z, log);
        }
    }

    true
}

// WorldGenShrub from 1.7, a log stump wrapped in a leafy mound
pub(crate) fn shrub(world: &GenWorld, random: &Random, x: i32, mut y: i32, z: i32) -> bool
{
    while y > 0
    {
        let there: i32 = world.block(x, y, z);
        if there != 0 && !blockIs(there, "minecraft:leaves")
        {
            break;
        }

        y -= 1;
    }

    if !grassOrDirt(world, x, y, z)
    {
        return true;
    }

    y += 1;
    world.set(x, y, z, palette::plain("minecraft:log"));

    for cy in y..=y + 2
    {
        let radius: i32 = 2 - (cy - y);
        for cx in x - radius..=x + radius
        {
            for cz in z - radius..=z + radius
            {
                let corner: bool =
                    (cx - x).abs() == radius && (cz - z).abs() == radius && random.nextIntBounded(2) == 0;

                if !corner && !world.isOpaque(cx, cy, cz)
                {
                    world.set(cx, cy, cz, palette::plain("minecraft:leaves"));
                }
            }
        }
    }

    true
}

fn surfaceBelow(world: &GenWorld, x: i32, mut y: i32, z: i32) -> i32
{
    loop
    {
        let there: i32 = world.block(x, y, z);
        if !((there == 0 || blockTagged(there, tags::LEAVES)) && y > 0)
        {
            return y;
        }

        y -= 1;
    }
}

fn scatter(random: &Random, x: i32, y: i32, z: i32, reach: i32, rise: i32) -> (i32, i32, i32)
{
    (
        x + random.nextIntBounded(reach) - random.nextIntBounded(reach),
        y + random.nextIntBounded(rise) - random.nextIntBounded(rise),
        z + random.nextIntBounded(reach) - random.nextIntBounded(reach),
    )
}

// WorldGenTallGrass, 128 tries at a plant around the first ground below
pub(crate) fn tallGrass(
    world: &GenWorld,
    random: &Random,
    x: i32,
    y: i32,
    z: i32,
    plant: (i32, i32),
    stays: impl Fn(&GenWorld, i32, i32, i32) -> bool,
) -> bool
{
    let ground: i32 = surfaceBelow(world, x, y, z);

    for _ in 0..128
    {
        let (px, py, pz): (i32, i32, i32) = scatter(random, x, ground, z, 8, 4);
        if world.isAir(px, py, pz) && stays(world, px, py, pz)
        {
            world.set(px, py, pz, plant);
        }
    }

    true
}

// WorldGenFlowers and WorldGenPumpkin, 64 tries around the spot
pub(crate) fn flowers(
    world: &GenWorld,
    random: &Random,
    x: i32,
    y: i32,
    z: i32,
    plant: Placed,
    stays: impl Fn(&GenWorld, i32, i32, i32) -> bool,
) -> bool
{
    for _ in 0..64
    {
        let (px, py, pz): (i32, i32, i32) = scatter(random, x, y, z, 8, 4);
        if world.isAir(px, py, pz) && stays(world, px, py, pz)
        {
            world.set(px, py, pz, plant);
        }
    }

    true
}

pub(crate) fn onSoil(world: &GenWorld, x: i32, y: i32, z: i32) -> bool
{
    blockTagged(world.block(x, y - 1, z), tags::SOIL)
}

pub(crate) fn lit(world: &GenWorld, x: i32, y: i32, z: i32) -> bool
{
    let light: i32 = world.m_world.light(BlockPos::new(x, y, z));
    onSoil(world, x, y, z) && (light >= 8 || world.height(x, z) <= y)
}

// WorldGenDeadBush
pub(crate) fn deadBush(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32) -> bool
{
    let ground: i32 = surfaceBelow(world, x, y, z);
    let bush: Placed = palette::plain("minecraft:deadbush");

    for _ in 0..4
    {
        let (px, py, pz): (i32, i32, i32) = scatter(random, x, ground, z, 8, 4);
        // beta dead bushes need sand, the twilight forest scatters them on any soil
        let below: i32 = world.block(px, py - 1, pz);
        if world.isAir(px, py, pz) && (blockIs(below, "minecraft:sand") || blockTagged(below, tags::SOIL))
        {
            world.set(px, py, pz, bush);
        }
    }

    true
}

// WorldGenReed, sugar cane beside water
pub(crate) fn reeds(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32) -> bool
{
    let reed: Placed = palette::plain("minecraft:reeds");
    let wet = |px: i32, py: i32, pz: i32| blockTagged(world.block(px, py, pz), "minecraft:water");

    for _ in 0..20
    {
        let px: i32 = x + random.nextIntBounded(4) - random.nextIntBounded(4);
        let pz: i32 = z + random.nextIntBounded(4) - random.nextIntBounded(4);
        if !world.isAir(px, y, pz)
            || !(wet(px - 1, y - 1, pz) || wet(px + 1, y - 1, pz) || wet(px, y - 1, pz - 1) || wet(px, y - 1, pz + 1))
        {
            continue;
        }

        let tall: i32 = 2 + random.nextIntBounded(random.nextIntBounded(3) + 1);
        for dy in 0..tall
        {
            let below: i32 = world.block(px, y + dy - 1, pz);
            if blockIs(below, "minecraft:reeds") || blockTagged(below, tags::SOIL) || blockIs(below, "minecraft:sand")
            {
                world.set(px, y + dy, pz, reed);
            }
        }
    }

    true
}

// WorldGenPumpkin
pub(crate) fn pumpkins(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32) -> bool
{
    for _ in 0..64
    {
        let (px, py, pz): (i32, i32, i32) = scatter(random, x, y, z, 8, 4);
        if world.isAir(px, py, pz) && world.is(px, py - 1, pz, "minecraft:grass")
        {
            world.set(px, py, pz, palette::meta("minecraft:pumpkin", random.nextIntBounded(4)));
        }
    }

    true
}

// WorldGenMinable, an ore vein of this many blocks through stone
pub(crate) fn ore(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32, block: Placed, size: i32)
{
    let pi: f32 = std::f32::consts::PI;
    let angle: f32 = random.nextFloat() * pi;
    let spread: f32 = size as f32 / 8.0;
    let fromX: f64 = f64::from((x + 8) as f32 + math::sin(angle) * spread);
    let toX: f64 = f64::from((x + 8) as f32 - math::sin(angle) * spread);
    let fromZ: f64 = f64::from((z + 8) as f32 + math::cos(angle) * spread);
    let toZ: f64 = f64::from((z + 8) as f32 - math::cos(angle) * spread);
    let fromY: f64 = f64::from(y + random.nextIntBounded(3) + 2);
    let toY: f64 = f64::from(y + random.nextIntBounded(3) + 2);

    for step in 0..=size
    {
        let along: f64 = f64::from(step) / f64::from(size);
        let cx: f64 = fromX + (toX - fromX) * along;
        let cy: f64 = fromY + (toY - fromY) * along;
        let cz: f64 = fromZ + (toZ - fromZ) * along;
        let girth: f64 = random.nextDouble() * f64::from(size) / 16.0;
        let swell: f64 = f64::from(math::sin(step as f32 * pi / size as f32) + 1.0) * girth + 1.0;
        let (minX, maxX): (i32, i32) = (math::floor(cx - swell / 2.0), math::floor(cx + swell / 2.0));
        let (minY, maxY): (i32, i32) = (math::floor(cy - swell / 2.0), math::floor(cy + swell / 2.0));
        let (minZ, maxZ): (i32, i32) = (math::floor(cz - swell / 2.0), math::floor(cz + swell / 2.0));

        for bx in minX..=maxX
        {
            let dx: f64 = (f64::from(bx) + 0.5 - cx) / (swell / 2.0);
            if dx * dx >= 1.0
            {
                continue;
            }

            for by in minY..=maxY
            {
                let dy: f64 = (f64::from(by) + 0.5 - cy) / (swell / 2.0);
                if dx * dx + dy * dy >= 1.0
                {
                    continue;
                }

                for bz in minZ..=maxZ
                {
                    let dz: f64 = (f64::from(bz) + 0.5 - cz) / (swell / 2.0);
                    if dx * dx + dy * dy + dz * dz < 1.0 && world.is(bx, by, bz, "minecraft:stone")
                    {
                        world.set(bx, by, bz, block);
                    }
                }
            }
        }
    }
}

// WorldGenSand and WorldGenClay from 1.7, a disc of sand gravel or clay under shallow water
pub(crate) fn disc(
    world: &GenWorld,
    random: &Random,
    at: [i32; 3],
    block: Placed,
    radius: i32,
    depth: i32,
    replaces: &[&str],
)
{
    let [x, y, z] = at;
    if !blockTagged(world.block(x, y, z), "minecraft:water")
    {
        return;
    }

    let size: i32 = random.nextIntBounded(radius - 2) + 2;
    for bx in x - size..=x + size
    {
        for bz in z - size..=z + size
        {
            if (bx - x) * (bx - x) + (bz - z) * (bz - z) > size * size
            {
                continue;
            }

            for by in y - depth..=y + depth
            {
                let there: i32 = world.block(bx, by, bz);
                if replaces.iter().any(|name| blockIs(there, name))
                {
                    world.set(bx, by, bz, block);
                }
            }
        }
    }
}

const LAKE_WIDTH: i32 = 16;
const LAKE_HEIGHT: i32 = 8;

fn lakeIndex(x: i32, z: i32, y: i32) -> usize { ((x * LAKE_WIDTH + z) * LAKE_HEIGHT + y) as usize }

fn lakeEdge(shape: &[bool], x: i32, z: i32, y: i32) -> bool
{
    if shape[lakeIndex(x, z, y)]
    {
        return false;
    }

    let at = |dx: i32, dz: i32, dy: i32|
    {
        let (nx, nz, ny): (i32, i32, i32) = (x + dx, z + dz, y + dy);
        (0..LAKE_WIDTH).contains(&nx)
            && (0..LAKE_WIDTH).contains(&nz)
            && (0..LAKE_HEIGHT).contains(&ny)
            && shape[lakeIndex(nx, nz, ny)]
    };
    at(1, 0, 0) || at(-1, 0, 0) || at(0, 1, 0) || at(0, -1, 0) || at(0, 0, 1) || at(0, 0, -1)
}

// WorldGenLakes, a pond of water or lava sunk four blocks into the ground
pub(crate) fn lake(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32, liquid: Placed) -> bool
{
    let (x, z): (i32, i32) = (x - 8, z - 8);
    let mut y: i32 = y;
    while y > 0 && world.isAir(x, y, z)
    {
        y -= 1;
    }

    y -= 4;

    let mut shape: Vec<bool> = vec![false; (LAKE_WIDTH * LAKE_WIDTH * LAKE_HEIGHT) as usize];
    let blobs: i32 = random.nextIntBounded(4) + 4;
    for _ in 0..blobs
    {
        let size: [f64; 3] =
            [random.nextDouble() * 6.0 + 3.0, random.nextDouble() * 4.0 + 2.0, random.nextDouble() * 6.0 + 3.0];
        let center: [f64; 3] = [
            random.nextDouble() * (16.0 - size[0] - 2.0) + 1.0 + size[0] / 2.0,
            random.nextDouble() * (8.0 - size[1] - 4.0) + 2.0 + size[1] / 2.0,
            random.nextDouble() * (16.0 - size[2] - 2.0) + 1.0 + size[2] / 2.0,
        ];

        for lx in 1..15
        {
            for lz in 1..15
            {
                for ly in 1..7
                {
                    let dx: f64 = (f64::from(lx) - center[0]) / (size[0] / 2.0);
                    let dy: f64 = (f64::from(ly) - center[1]) / (size[1] / 2.0);
                    let dz: f64 = (f64::from(lz) - center[2]) / (size[2] / 2.0);
                    if dx * dx + dy * dy + dz * dz < 1.0
                    {
                        shape[lakeIndex(lx, lz, ly)] = true;
                    }
                }
            }
        }
    }

    for lx in 0..LAKE_WIDTH
    {
        for lz in 0..LAKE_WIDTH
        {
            for ly in 0..LAKE_HEIGHT
            {
                if !lakeEdge(&shape, lx, lz, ly)
                {
                    continue;
                }
                let (bx, by, bz): (i32, i32, i32) = (x + lx, y + ly, z + lz);
                if ly >= 4 && world.isLiquid(bx, by, bz)
                {
                    return false;
                }
                if ly < 4 && !world.isSolid(bx, by, bz) && world.block(bx, by, bz) != liquid.0
                {
                    return false;
                }
            }
        }
    }

    for lx in 0..LAKE_WIDTH
    {
        for lz in 0..LAKE_WIDTH
        {
            for ly in 0..LAKE_HEIGHT
            {
                if shape[lakeIndex(lx, lz, ly)]
                {
                    world.set(x + lx, y + ly, z + lz, if ly >= 4 { palette::AIR } else { liquid });
                }
            }
        }
    }

    let grass: Placed = palette::plain("minecraft:grass");

    for lx in 0..LAKE_WIDTH
    {
        for lz in 0..LAKE_WIDTH
        {
            for ly in 4..LAKE_HEIGHT
            {
                let (bx, by, bz): (i32, i32, i32) = (x + lx, y + ly, z + lz);
                if shape[lakeIndex(lx, lz, ly)]
                    && world.is(bx, by - 1, bz, "minecraft:dirt")
                    && world.height(bx, bz) <= by
                {
                    world.set(bx, by - 1, bz, grass);
                }
            }
        }
    }

    if blockTagged(liquid.0, "minecraft:lava")
    {
        let stone: Placed = palette::plain("minecraft:stone");

        for lx in 0..LAKE_WIDTH
        {
            for lz in 0..LAKE_WIDTH
            {
                for ly in 0..LAKE_HEIGHT
                {
                    let (bx, by, bz): (i32, i32, i32) = (x + lx, y + ly, z + lz);
                    if lakeEdge(&shape, lx, lz, ly)
                        && (ly < 4 || random.nextIntBounded(2) != 0)
                        && world.isSolid(bx, by, bz)
                    {
                        world.set(bx, by, bz, stone);
                    }
                }
            }
        }
    }

    true
}

// WorldGenDungeons.pickCheckLootItem
fn dungeonLoot(random: &Random) -> Option<ItemStack>
{
    let stack = |name: &str, count: i32| Some(ItemStack::of(name, count));

    match random.nextIntBounded(11)
    {
        0 => stack("minecraft:saddle", 1),
        1 => stack("minecraft:iron_ingot", random.nextIntBounded(4) + 1),
        2 => stack("minecraft:bread", 1),
        3 => stack("minecraft:wheat", random.nextIntBounded(4) + 1),
        4 => stack("minecraft:gunpowder", random.nextIntBounded(4) + 1),
        5 => stack("minecraft:string", random.nextIntBounded(4) + 1),
        6 => stack("minecraft:bucket", 1),
        7 if random.nextIntBounded(100) == 0 => stack("minecraft:golden_apple", 1),
        8 if random.nextIntBounded(2) == 0 => stack("minecraft:redstone", random.nextIntBounded(4) + 1),
        9 if random.nextIntBounded(10) == 0 => stack(
            if random.nextIntBounded(2) == 0
            {
                "minecraft:record_13"
            }
            else
            {
                "minecraft:record_cat"
            },
            1,
        ),
        10 => Some(ItemStack::of("minecraft:dye", 1).withDamage(3)),
        _ => None,
    }
}

// WorldGenDungeons, a cobblestone room with a spawner and chests
pub(crate) fn dungeon(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32) -> bool
{
    let roof: i32 = 3;
    let halfX: i32 = random.nextIntBounded(2) + 2;
    let halfZ: i32 = random.nextIntBounded(2) + 2;
    let mut openings: i32 = 0;

    for bx in x - halfX - 1..=x + halfX + 1
    {
        for by in y - 1..=y + roof + 1
        {
            for bz in z - halfZ - 1..=z + halfZ + 1
            {
                let solid: bool = world.isSolid(bx, by, bz);
                if (by == y - 1 || by == y + roof + 1) && !solid
                {
                    return false;
                }

                let wall: bool =
                    bx == x - halfX - 1 || bx == x + halfX + 1 || bz == z - halfZ - 1 || bz == z + halfZ + 1;

                if wall && by == y && world.isAir(bx, by, bz) && world.isAir(bx, by + 1, bz)
                {
                    openings += 1;
                }
            }
        }
    }

    if !(1..=5).contains(&openings)
    {
        return false;
    }

    let (cobble, mossy): (Placed, Placed) =
        (palette::plain("minecraft:cobblestone"), palette::plain("minecraft:mossy_cobblestone"));

    for bx in x - halfX - 1..=x + halfX + 1
    {
        for by in (y - 1..=y + roof).rev()
        {
            for bz in z - halfZ - 1..=z + halfZ + 1
            {
                let shell: bool = bx == x - halfX - 1
                    || by == y - 1
                    || bz == z - halfZ - 1
                    || bx == x + halfX + 1
                    || by == y + roof + 1
                    || bz == z + halfZ + 1;

                if !shell
                {
                    world.setNotify(bx, by, bz, palette::AIR);
                    continue;
                }

                if by >= 0 && !world.isSolid(bx, by - 1, bz)
                {
                    world.setNotify(bx, by, bz, palette::AIR);
                    continue;
                }

                if !world.isSolid(bx, by, bz)
                {
                    continue;
                }

                let stone: Placed = if by == y - 1 && random.nextIntBounded(4) != 0
                {
                    mossy
                }
                else
                {
                    cobble
                };
                world.setNotify(bx, by, bz, stone);
            }
        }
    }

    'chests: for _ in 0..2
    {
        for _ in 0..3
        {
            let cx: i32 = x + random.nextIntBounded(halfX * 2 + 1) - halfX;
            let cz: i32 = z + random.nextIntBounded(halfZ * 2 + 1) - halfZ;
            if !world.isAir(cx, y, cz)
            {
                continue;
            }
            let walls: usize =
                [[-1, 0], [1, 0], [0, -1], [0, 1]].iter().filter(|[dx, dz]| world.isSolid(cx + dx, y, cz + dz)).count();

            if walls != 1
            {
                continue;
            }

            world.setNotify(cx, y, cz, palette::plain("minecraft:chest"));

            let slots: i32 = 27;

            for _ in 0..8
            {
                if let Some(stack) = dungeonLoot(random)
                {
                    let _ =
                        world.m_world.setTileSlot(BlockPos::new(cx, y, cz), random.nextIntBounded(slots), Some(stack));
                }
            }

            continue 'chests;
        }
    }

    let mob: &str = match random.nextIntBounded(4)
    {
        0 => "Skeleton",
        3 => "Spider",
        _ => "Zombie",
    };
    if let Err(error) = world.m_world.setSpawner(BlockPos::new(x, y, z), mob)
    {
        b173_api::warn(&format!("dungeon spawner at {x} {y} {z}: {}", error.0));
    }

    true
}

// WorldGenLiquids, a spring in a stone wall
pub(crate) fn spring(world: &GenWorld, x: i32, y: i32, z: i32, liquid: Placed) -> bool
{
    let stone = |x: i32, y: i32, z: i32| world.is(x, y, z, "minecraft:stone");
    if !stone(x, y + 1, z) || !stone(x, y - 1, z)
    {
        return false;
    }
    let there: i32 = world.block(x, y, z);
    if there != 0 && !blockIs(there, "minecraft:stone")
    {
        return false;
    }

    let sides: [[i32; 2]; 4] = [[-1, 0], [1, 0], [0, -1], [0, 1]];
    let walls: usize = sides.iter().filter(|[dx, dz]| stone(x + dx, y, z + dz)).count();
    let open: usize = sides.iter().filter(|[dx, dz]| world.isAir(x + dx, y, z + dz)).count();
    if walls == 3 && open == 1
    {
        world.setNotify(x, y, z, liquid);
    }

    true
}

// WorldGenBigMushroom, the 1.7 huge mushroom drawn in the port's own huge mushroom blocks
pub(crate) fn bigMushroom(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32) -> bool
{
    let red: bool = random.nextIntBounded(2) == 1;
    let height: i32 = random.nextIntBounded(3) + 4;
    if y < 1 || y + height + 1 >= crate::world::terrain::height()
    {
        return false;
    }

    for by in y..=y + 1 + height
    {
        let reach: i32 = if by <= y + 3 { 0 } else { 3 };
        for bx in x - reach..=x + reach
        {
            for bz in z - reach..=z + reach
            {
                let there: i32 = world.block(bx, by, bz);
                if there != 0 && !blockTagged(there, tags::LEAVES)
                {
                    return false;
                }
            }
        }
    }

    let below: i32 = world.block(x, y - 1, z);
    if !blockTagged(below, "minecraft:dirt")
    {
        return false;
    }

    let name: &'static str = if red
    {
        crate::names::block::HUGE_RED_MUSHROOM
    }
    else
    {
        crate::names::block::HUGE_BROWN_MUSHROOM
    };
    let block: Placed = palette::plain(name);

    let top: i32 = y + height;
    let bottom: i32 = if red { top - 3 } else { top };
    for by in bottom..=top
    {
        let reach: i32 = if red { if by < top { 2 } else { 1 } } else { 3 };
        for bx in x - reach..=x + reach
        {
            for bz in z - reach..=z + reach
            {
                let edgeX: bool = bx == x - reach || bx == x + reach;
                let edgeZ: bool = bz == z - reach || bz == z + reach;
                // a brown cap or the walls of a red one skip their corners
                if (!red || by < top) && edgeX && edgeZ
                {
                    continue;
                }

                // the inside of a red cap stays hollow
                let inside: bool = by < top && !edgeX && !edgeZ;
                if !inside && world.isReplaceable(bx, by, bz)
                {
                    world.set(bx, by, bz, block);
                }
            }
        }
    }

    for dy in 0..height
    {
        if world.isReplaceable(x, y + dy, z)
        {
            world.set(x, y + dy, z, palette::with(name, "part", "stem"));
        }
    }

    true
}

// WorldGenBlockBlob, three overlapping balls rolling downhill from the ground
pub(crate) fn boulder(world: &GenWorld, random: &Random, x: i32, mut y: i32, z: i32, block: Placed) -> bool
{
    let ground = |block: i32| blockTagged(block, "minecraft:dirt") || blockIs(block, "minecraft:stone");
    let lowest: i32 = 3;
    while y > lowest && (world.isAir(x, y - 1, z) || !ground(world.block(x, y - 1, z)))
    {
        y -= 1;
    }

    if y <= lowest
    {
        return false;
    }

    let (mut x, mut z): (i32, i32) = (x, z);

    for _ in 0..3
    {
        let (rx, ry, rz): (i32, i32, i32) =
            (random.nextIntBounded(2), random.nextIntBounded(2), random.nextIntBounded(2));
        let reach: f32 = (rx + ry + rz) as f32 * 0.333 + 0.5;

        for bx in x - rx..=x + rx
        {
            for bz in z - rz..=z + rz
            {
                for by in y - ry..=y + ry
                {
                    let (dx, dy, dz): (f32, f32, f32) = ((bx - x) as f32, (by - y) as f32, (bz - z) as f32);
                    if dx * dx + dz * dz + dy * dy <= reach * reach
                    {
                        world.set(bx, by, bz, block);
                    }
                }
            }
        }

        x += random.nextIntBounded(2) - 1;
        z += random.nextIntBounded(2) - 1;
        y -= random.nextIntBounded(2);
    }

    true
}

// WorldGenVines, beta has no vines so only the random draws stay to keep later decorations in place
pub(crate) fn vines(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32)
{
    let top: i32 = 128;
    let (mut bx, mut bz): (i32, i32) = (x, z);

    for by in y..top
    {
        if !world.isAir(bx, by, bz)
        {
            bx = x + random.nextIntBounded(4) - random.nextIntBounded(4);
            bz = z + random.nextIntBounded(4) - random.nextIntBounded(4);
        }
    }
}

// WorldGenWaterlily, beta has no lily pads so only the random draws stay
pub(crate) fn waterlily(random: &Random, x: i32, y: i32, z: i32)
{
    for _ in 0..10
    {
        let _ = scatter(random, x, y, z, 8, 4);
    }
}

// WorldGenDoublePlant, beta has no double plants so each kind places its closest single plant
pub(crate) fn doublePlant(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32, kind: i32) -> bool
{
    let plant: Placed = match kind
    {
        0 => palette::plain("minecraft:yellow_flower"),
        2 => palette::meta("minecraft:tallgrass", 1),
        3 => palette::meta("minecraft:tallgrass", 2),
        _ => palette::plain("minecraft:red_flower"),
    };

    let mut placed: bool = false;

    for _ in 0..64
    {
        let (px, py, pz): (i32, i32, i32) = scatter(random, x, y, z, 8, 4);
        if world.isAir(px, py, pz) && world.isAir(px, py + 1, pz) && lit(world, px, py, pz)
        {
            world.set(px, py, pz, plant);
            placed = true;
        }
    }

    placed
}
