use super::generate::GenWorld;
use b173_api::prelude::*;
use b173_api::terrain::{Random, math};

// the vanilla world generators the twilight decorator calls, ported from beta where beta has them
const TOP: i32 = 128;

fn grassOrDirt(world: &GenWorld, x: i32, y: i32, z: i32) -> bool
{
    let below: i32 = world.block(x, y, z);
    below == world.id("minecraft:grass") || below == world.id("minecraft:dirt")
}

fn clearForTree(world: &GenWorld, x: i32, y: i32, z: i32, height: i32, widthAt: impl Fn(i32) -> i32) -> bool
{
    let leaves: i32 = world.id("minecraft:leaves");

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
                if there != 0 && there != leaves
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

    let (leaves, log): (i32, i32) = (world.id("minecraft:leaves"), world.id("minecraft:log"));

    world.set(x, y - 1, z, world.id("minecraft:dirt"));

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
                    world.setMeta(cx, cy, cz, leaves, meta);
                }
            }
        }
    }

    for dy in 0..height
    {
        let there: i32 = world.block(x, y + dy, z);
        if there == 0 || there == leaves
        {
            world.setMeta(x, y + dy, z, log, meta);
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

    let (leaves, log): (i32, i32) = (world.id("minecraft:leaves"), world.id("minecraft:log"));

    world.set(x, y - 1, z, world.id("minecraft:dirt"));

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
                    world.setMeta(cx, cy, cz, leaves, 1);
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
        if there == 0 || there == leaves
        {
            world.setMeta(x, y + dy, z, log, 1);
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

    let (leaves, log): (i32, i32) = (world.id("minecraft:leaves"), world.id("minecraft:log"));

    world.set(x, y - 1, z, world.id("minecraft:dirt"));

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
                    world.setMeta(cx, cy, cz, leaves, 1);
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
        if there == 0 || there == leaves
        {
            world.setMeta(x, y + dy, z, log, 1);
        }
    }

    true
}

// WorldGenShrub from 1.7, a log stump wrapped in a leafy mound
pub(crate) fn shrub(world: &GenWorld, random: &Random, x: i32, mut y: i32, z: i32) -> bool
{
    let leaves: i32 = world.id("minecraft:leaves");

    while y > 0
    {
        let there: i32 = world.block(x, y, z);
        if there != 0 && there != leaves
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
    world.set(x, y, z, world.id("minecraft:log"));

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
                    world.set(cx, cy, cz, leaves);
                }
            }
        }
    }

    true
}

fn surfaceBelow(world: &GenWorld, x: i32, mut y: i32, z: i32) -> i32
{
    let leaves: i32 = world.id("minecraft:leaves");

    loop
    {
        let there: i32 = world.block(x, y, z);
        if !((there == 0 || there == leaves || crate::ids::leaves(there)) && y > 0)
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
            world.setMeta(px, py, pz, plant.0, plant.1);
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
    plant: i32,
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

pub(crate) fn onSoil(world: &GenWorld, x: i32, y: i32, z: i32) -> bool { crate::ids::soil(world.block(x, y - 1, z)) }

pub(crate) fn lit(world: &GenWorld, x: i32, y: i32, z: i32) -> bool
{
    let light: i32 = world.m_world.light(BlockPos::new(x, y, z));
    onSoil(world, x, y, z) && (light >= 8 || world.height(x, z) <= y)
}

// WorldGenDeadBush
pub(crate) fn deadBush(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32) -> bool
{
    let ground: i32 = surfaceBelow(world, x, y, z);
    let bush: i32 = world.id("minecraft:deadbush");
    let sand: i32 = world.id("minecraft:sand");

    for _ in 0..4
    {
        let (px, py, pz): (i32, i32, i32) = scatter(random, x, ground, z, 8, 4);
        // beta dead bushes need sand, the twilight forest scatters them on any soil
        let below: i32 = world.block(px, py - 1, pz);
        if world.isAir(px, py, pz) && (below == sand || crate::ids::soil(below))
        {
            world.set(px, py, pz, bush);
        }
    }

    true
}

// WorldGenReed, sugar cane beside water
pub(crate) fn reeds(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32) -> bool
{
    let water: [i32; 2] = [world.id("minecraft:water"), world.id("minecraft:flowing_water")];
    let reed: i32 = world.id("minecraft:reeds");
    let wet = |px: i32, py: i32, pz: i32| water.contains(&world.block(px, py, pz));

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
            if below == reed || crate::ids::soil(below) || below == world.id("minecraft:sand")
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
    let (pumpkin, grass): (i32, i32) = (world.id("minecraft:pumpkin"), world.id("minecraft:grass"));

    for _ in 0..64
    {
        let (px, py, pz): (i32, i32, i32) = scatter(random, x, y, z, 8, 4);
        if world.isAir(px, py, pz) && world.block(px, py - 1, pz) == grass
        {
            world.setMeta(px, py, pz, pumpkin, random.nextIntBounded(4));
        }
    }

    true
}

// WorldGenMinable, an ore vein of this many blocks through stone
pub(crate) fn ore(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32, block: i32, size: i32)
{
    let stone: i32 = world.id("minecraft:stone");
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
                    if dx * dx + dy * dy + dz * dz < 1.0 && world.block(bx, by, bz) == stone
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
    block: i32,
    radius: i32,
    depth: i32,
    replaces: &[i32],
)
{
    let [x, y, z] = at;
    let water: [i32; 2] = [world.id("minecraft:water"), world.id("minecraft:flowing_water")];
    if !water.contains(&world.block(x, y, z))
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
                if replaces.contains(&world.block(bx, by, bz))
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
pub(crate) fn lake(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32, liquid: i32) -> bool
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
                if ly < 4 && !world.isSolid(bx, by, bz) && world.block(bx, by, bz) != liquid
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
                    world.set(x + lx, y + ly, z + lz, if ly >= 4 { 0 } else { liquid });
                }
            }
        }
    }

    let (dirt, grass): (i32, i32) = (world.id("minecraft:dirt"), world.id("minecraft:grass"));

    for lx in 0..LAKE_WIDTH
    {
        for lz in 0..LAKE_WIDTH
        {
            for ly in 4..LAKE_HEIGHT
            {
                let (bx, by, bz): (i32, i32, i32) = (x + lx, y + ly, z + lz);
                if shape[lakeIndex(lx, lz, ly)] && world.block(bx, by - 1, bz) == dirt && world.height(bx, bz) <= by
                {
                    world.set(bx, by - 1, bz, grass);
                }
            }
        }
    }

    if liquid == world.id("minecraft:lava") || liquid == world.id("minecraft:flowing_lava")
    {
        let stone: i32 = world.id("minecraft:stone");

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

    let (cobble, mossy): (i32, i32) = (world.id("minecraft:cobblestone"), world.id("minecraft:mossy_cobblestone"));

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
                    world.setNotify(bx, by, bz, 0, 0);
                    continue;
                }

                if by >= 0 && !world.isSolid(bx, by - 1, bz)
                {
                    world.setNotify(bx, by, bz, 0, 0);
                    continue;
                }

                if !world.isSolid(bx, by, bz)
                {
                    continue;
                }

                let stone: i32 = if by == y - 1 && random.nextIntBounded(4) != 0
                {
                    mossy
                }
                else
                {
                    cobble
                };
                world.setNotify(bx, by, bz, stone, 0);
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

            world.setNotify(cx, y, cz, world.id("minecraft:chest"), 0);

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
pub(crate) fn spring(world: &GenWorld, x: i32, y: i32, z: i32, liquid: i32) -> bool
{
    let stone: i32 = world.id("minecraft:stone");
    if world.block(x, y + 1, z) != stone || world.block(x, y - 1, z) != stone
    {
        return false;
    }
    let there: i32 = world.block(x, y, z);
    if there != 0 && there != stone
    {
        return false;
    }

    let sides: [[i32; 2]; 4] = [[-1, 0], [1, 0], [0, -1], [0, 1]];
    let walls: usize = sides.iter().filter(|[dx, dz]| world.block(x + dx, y, z + dz) == stone).count();
    let open: usize = sides.iter().filter(|[dx, dz]| world.isAir(x + dx, y, z + dz)).count();
    if walls == 3 && open == 1
    {
        world.setNotify(x, y, z, liquid, 0);
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

    let leaves: i32 = world.id("minecraft:leaves");

    for by in y..=y + 1 + height
    {
        let reach: i32 = if by <= y + 3 { 0 } else { 3 };
        for bx in x - reach..=x + reach
        {
            for bz in z - reach..=z + reach
            {
                let there: i32 = world.block(bx, by, bz);
                if there != 0 && there != leaves && !crate::ids::leaves(there)
                {
                    return false;
                }
            }
        }
    }

    let below: i32 = world.block(x, y - 1, z);
    if below != world.id("minecraft:dirt") && below != world.id("minecraft:grass")
    {
        return false;
    }

    let block: i32 = world.id(
        if red
        {
            crate::ids::block::HUGE_RED_MUSHROOM
        }
        else
        {
            crate::ids::block::HUGE_BROWN_MUSHROOM
        },
    );

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
            world.setMeta(x, y + dy, z, block, 1);
        }
    }

    true
}

// WorldGenBlockBlob, three overlapping balls rolling downhill from the ground
pub(crate) fn boulder(world: &GenWorld, random: &Random, x: i32, mut y: i32, z: i32, block: i32) -> bool
{
    let ground: [i32; 3] = [world.id("minecraft:grass"), world.id("minecraft:dirt"), world.id("minecraft:stone")];
    let lowest: i32 = 3;
    while y > lowest && (world.isAir(x, y - 1, z) || !ground.contains(&world.block(x, y - 1, z)))
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
    let plant: (i32, i32) = match kind
    {
        0 => (world.id("minecraft:yellow_flower"), 0),
        2 => (world.id("minecraft:tallgrass"), 1),
        3 => (world.id("minecraft:tallgrass"), 2),
        _ => (world.id("minecraft:red_flower"), 0),
    };

    let mut placed: bool = false;

    for _ in 0..64
    {
        let (px, py, pz): (i32, i32, i32) = scatter(random, x, y, z, 8, 4);
        if world.isAir(px, py, pz) && world.isAir(px, py + 1, pz) && lit(world, px, py, pz)
        {
            world.setMeta(px, py, pz, plant.0, plant.1);
            placed = true;
        }
    }

    placed
}
