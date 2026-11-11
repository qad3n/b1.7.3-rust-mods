use super::generate::GenWorld;
use super::scatter::suitable;
use super::trees::{Placed, leaves, log};
use crate::ids::block;
use crate::treasure::{self, Table};
use b173_api::prelude::*;
use b173_api::terrain::Random;

// the small ruins TFBiomeDecorator picks from, with their weights
#[derive(Clone, Copy)]
enum Ruin
{
    StoneCircle,
    Well,
    WitchHut,
    Stalagmite,
    Foundation,
    Monolith,
    GroveRuins,
    HollowStump,
    HollowLog,
    SmallLog,
}

const RUINS: [(Ruin, i32); 10] = [
    (Ruin::StoneCircle, 10),
    (Ruin::Well, 10),
    (Ruin::WitchHut, 5),
    (Ruin::Stalagmite, 12),
    (Ruin::Foundation, 10),
    (Ruin::Monolith, 10),
    (Ruin::GroveRuins, 5),
    (Ruin::HollowStump, 12),
    (Ruin::HollowLog, 10),
    (Ruin::SmallLog, 10),
];

// WeightedRandom.getRandomItem
fn pick(random: &Random) -> Ruin
{
    let total: i32 = RUINS.iter().map(|(_, weight)| weight).sum();
    let mut roll: i32 = random.nextIntBounded(total);

    for (ruin, weight) in RUINS
    {
        roll -= weight;
        if roll < 0
        {
            return ruin;
        }
    }

    Ruin::StoneCircle
}

// one random ruin at the spot
pub(crate) fn generate(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32)
{
    let builder: super::trees::Builder = super::trees::Builder { m_world: world, m_notify: false };

    match pick(random)
    {
        Ruin::StoneCircle => stoneCircle(world, x, y, z),
        Ruin::Well => well(world, random, x, y, z),
        Ruin::WitchHut => witchHut(world, random, x, y, z),

        Ruin::Stalagmite =>
        {
            super::scatter::stalagmite(world, random, x, y, z);
        }

        Ruin::Foundation => foundation(world, random, x, y, z),
        Ruin::Monolith => monolith(world, random, x, y, z),
        Ruin::GroveRuins => groveRuins(world, random, x, y, z),

        Ruin::HollowStump =>
        {
            super::hollow::stump(&builder, random, x, y, z);
        }

        Ruin::HollowLog => hollowLog(world, random, x, y, z),
        Ruin::SmallLog => smallLog(world, random, x, y, z),
    }
}

fn mossy(world: &GenWorld) -> i32 { world.id("minecraft:mossy_cobblestone") }

// TFGenerator.randStone, mossy one time in n
fn randStone(world: &GenWorld, random: &Random, odds: i32) -> i32
{
    if random.nextIntBounded(odds) >= 1
    {
        world.id("minecraft:cobblestone")
    }
    else
    {
        mossy(world)
    }
}

// TFGenStoneCircle
fn stoneCircle(world: &GenWorld, x: i32, y: i32, z: i32)
{
    if !suitable(world, x - 3, y, z - 3, [6, 4, 6])
    {
        return;
    }

    let stones: [[i32; 2]; 8] = [[-3, 0], [3, 0], [0, -3], [0, 3], [-2, -2], [2, -2], [-2, 2], [2, 2]];

    for cy in 0..=2
    {
        for [dx, dz] in stones
        {
            world.set(x + dx, y + cy, z + dz, mossy(world));
        }
    }
}

// beta keeps the wooden slab as metadata two of the stone slab
const WOODEN: i32 = 2;

// TFGenWell, a three or four wide well whose shaft drops through the ground
fn well(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32)
{
    let wide: i32 = if random.nextIntBounded(4) == 0 { 4 } else { 3 };
    if !suitable(world, x, y, z, [wide, 4, wide])
    {
        return;
    }

    let last: i32 = wide - 1;
    let (fence, slab, planks, water): (i32, i32, i32, i32) = (
        world.id("minecraft:fence"),
        world.id("minecraft:stone_slab"),
        world.id("minecraft:planks"),
        world.id("minecraft:water"),
    );

    for dx in 0..wide
    {
        for dz in 0..wide
        {
            let rim: bool = dx == 0 || dx == last || dz == 0 || dz == last;
            world.set(x + dx, y, z + dz, if rim { mossy(world) } else { water });

            if rim
            {
                world.setMeta(x + dx, y + 3, z + dz, slab, WOODEN);
            }
            else
            {
                world.set(x + dx, y + 3, z + dz, planks);
            }
        }
    }

    for [dx, dz] in [[0, 0], [last, 0], [0, last], [last, last]]
    {
        world.set(x + dx, y + 1, z + dz, fence);
        world.set(x + dx, y + 2, z + dz, fence);
    }

    for dx in 1..last
    {
        for dz in 1..last
        {
            let treasure: bool = wide == 3;
            shaft(world, random, [x + dx, y, z + dz], treasure);
        }
    }
}

// the well shaft fills with water down through dirt and stone, the small well may hide a basement
fn shaft(world: &GenWorld, random: &Random, top: [i32; 3], treasure: bool)
{
    let [x, y, z] = top;
    let digs: [i32; 4] = [
        world.id("minecraft:dirt"),
        world.id("minecraft:grass"),
        world.id("minecraft:gravel"),
        world.id("minecraft:stone"),
    ];

    let water: i32 = world.id("minecraft:water");
    let deepest: i32 = 20;
    let hidden: i32 = 15;
    let mut found: bool = false;

    for depth in 1..=deepest
    {
        let by: i32 = y - depth;
        if !digs.contains(&world.block(x, by, z)) || !world.isSolid(x, by - 1, z)
        {
            break;
        }

        world.set(x, by, z, water);

        if treasure && depth > hidden && !found && random.nextIntBounded(8) == 0
        {
            world.set(x + 1, by, z, water);
            world.set(x + 2, by + 1, z, 0);
            world.set(x + 2, by, z, 0);
            treasure::place(world, random, Table::Basement, x + 2, by, z);
            found = true;
        }
    }
}

#[derive(Clone, Copy)]
enum Hut
{
    Stone(i32),
    Brick,
    Roof,
    Eave,
    Fire,
}

const HUT: [(i32, i32, i32, Hut); 127] = [
    (1, 0, 1, Hut::Stone(1)),
    (2, 0, 1, Hut::Stone(1)),
    (3, 0, 1, Hut::Stone(1)),
    (5, 0, 1, Hut::Stone(1)),
    (0, 0, 2, Hut::Brick),
    (1, 0, 2, Hut::Brick),
    (5, 0, 2, Hut::Stone(1)),
    (0, 0, 3, Hut::Brick),
    (5, 0, 3, Hut::Stone(1)),
    (0, 0, 4, Hut::Brick),
    (1, 0, 4, Hut::Brick),
    (5, 0, 4, Hut::Stone(1)),
    (1, 0, 5, Hut::Stone(1)),
    (2, 0, 5, Hut::Stone(1)),
    (3, 0, 5, Hut::Stone(1)),
    (5, 0, 5, Hut::Stone(1)),
    (1, 1, 1, Hut::Stone(2)),
    (3, 1, 1, Hut::Stone(2)),
    (5, 1, 1, Hut::Stone(2)),
    (0, 1, 2, Hut::Brick),
    (1, 1, 2, Hut::Brick),
    (5, 1, 2, Hut::Stone(2)),
    (0, 1, 3, Hut::Brick),
    (0, 1, 4, Hut::Brick),
    (1, 1, 4, Hut::Brick),
    (5, 1, 4, Hut::Stone(2)),
    (1, 1, 5, Hut::Stone(2)),
    (3, 1, 5, Hut::Stone(2)),
    (5, 1, 5, Hut::Stone(2)),
    (1, 2, 1, Hut::Stone(3)),
    (2, 2, 1, Hut::Stone(3)),
    (3, 2, 1, Hut::Stone(3)),
    (4, 2, 1, Hut::Stone(3)),
    (5, 2, 1, Hut::Stone(3)),
    (0, 2, 2, Hut::Brick),
    (1, 2, 2, Hut::Brick),
    (5, 2, 2, Hut::Stone(3)),
    (0, 2, 3, Hut::Brick),
    (5, 2, 3, Hut::Stone(3)),
    (0, 2, 4, Hut::Brick),
    (1, 2, 4, Hut::Brick),
    (5, 2, 4, Hut::Stone(1)),
    (1, 2, 5, Hut::Stone(3)),
    (2, 2, 5, Hut::Stone(3)),
    (3, 2, 5, Hut::Stone(3)),
    (4, 2, 5, Hut::Stone(3)),
    (5, 2, 5, Hut::Stone(3)),
    (0, 3, 2, Hut::Brick),
    (0, 3, 3, Hut::Brick),
    (0, 3, 4, Hut::Brick),
    (2, 3, 1, Hut::Stone(4)),
    (3, 3, 1, Hut::Stone(4)),
    (4, 3, 1, Hut::Stone(4)),
    (2, 3, 5, Hut::Stone(4)),
    (3, 3, 5, Hut::Stone(4)),
    (4, 3, 5, Hut::Stone(4)),
    (0, 4, 3, Hut::Brick),
    (3, 4, 1, Hut::Stone(5)),
    (3, 4, 5, Hut::Stone(5)),
    (0, 5, 3, Hut::Brick),
    (0, 6, 3, Hut::Brick),
    (0, 2, 0, Hut::Roof),
    (0, 2, 1, Hut::Roof),
    (0, 2, 5, Hut::Roof),
    (0, 2, 6, Hut::Roof),
    (6, 2, 0, Hut::Roof),
    (6, 2, 1, Hut::Roof),
    (6, 2, 2, Hut::Roof),
    (6, 2, 3, Hut::Roof),
    (6, 2, 4, Hut::Roof),
    (6, 2, 5, Hut::Roof),
    (6, 2, 6, Hut::Roof),
    (1, 3, 0, Hut::Roof),
    (1, 3, 1, Hut::Roof),
    (1, 3, 2, Hut::Roof),
    (1, 3, 4, Hut::Roof),
    (1, 3, 5, Hut::Roof),
    (1, 3, 6, Hut::Roof),
    (5, 3, 0, Hut::Roof),
    (5, 3, 1, Hut::Roof),
    (5, 3, 2, Hut::Roof),
    (5, 3, 3, Hut::Roof),
    (5, 3, 4, Hut::Roof),
    (5, 3, 5, Hut::Roof),
    (5, 3, 6, Hut::Roof),
    (1, 4, 0, Hut::Eave),
    (2, 4, 0, Hut::Roof),
    (2, 4, 1, Hut::Roof),
    (2, 4, 2, Hut::Roof),
    (2, 4, 3, Hut::Roof),
    (2, 4, 4, Hut::Roof),
    (2, 4, 5, Hut::Roof),
    (2, 4, 6, Hut::Roof),
    (1, 4, 6, Hut::Eave),
    (5, 4, 0, Hut::Eave),
    (4, 4, 0, Hut::Roof),
    (4, 4, 1, Hut::Roof),
    (4, 4, 2, Hut::Roof),
    (4, 4, 3, Hut::Roof),
    (4, 4, 4, Hut::Roof),
    (4, 4, 5, Hut::Roof),
    (4, 4, 6, Hut::Roof),
    (5, 4, 6, Hut::Eave),
    (2, 5, 0, Hut::Eave),
    (2, 5, 1, Hut::Eave),
    (4, 5, 0, Hut::Eave),
    (4, 5, 1, Hut::Eave),
    (3, 5, 0, Hut::Roof),
    (3, 5, 1, Hut::Roof),
    (3, 5, 2, Hut::Roof),
    (3, 5, 3, Hut::Roof),
    (3, 5, 4, Hut::Roof),
    (3, 5, 5, Hut::Roof),
    (3, 5, 6, Hut::Roof),
    (2, 5, 5, Hut::Eave),
    (2, 5, 6, Hut::Eave),
    (4, 5, 5, Hut::Eave),
    (4, 5, 6, Hut::Eave),
    (3, 6, 0, Hut::Roof),
    (3, 6, 1, Hut::Roof),
    (3, 6, 2, Hut::Eave),
    (3, 6, 4, Hut::Eave),
    (3, 6, 5, Hut::Roof),
    (3, 6, 6, Hut::Roof),
    (3, 7, 0, Hut::Eave),
    (3, 7, 6, Hut::Eave),
    (1, 0, 3, Hut::Fire),
];

// TFGenWitchHut, a tiny stone hut with a slab roof, a chimney and a skeleton druid spawner
fn witchHut(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32)
{
    if !suitable(world, x, y, z, [5, 7, 6])
    {
        return;
    }

    let (brick, roof, eave, fire): (i32, i32, i32, i32) = (
        world.id("minecraft:brick_block"),
        world.id("minecraft:double_stone_slab"),
        world.id("minecraft:stone_slab"),
        world.id("minecraft:fire"),
    );
    world.set(x + 1, y - 1, z + 3, world.id("minecraft:netherrack"));

    for (dx, dy, dz, part) in HUT
    {
        let placed: Placed = match part
        {
            Hut::Stone(odds) => (randStone(world, random, odds), 0),
            Hut::Brick => (brick, 0),
            Hut::Roof => (roof, WOODEN),
            Hut::Eave => (eave, WOODEN),
            Hut::Fire => (fire, 0),
        };
        world.setMeta(x + dx, y + dy, z + dz, placed.0, placed.1);
    }

    if let Err(error) = world.m_world.setSpawner(BlockPos::new(x + 3, y + 1, z + 3), crate::entities::SKELETON_DRUID)
    {
        b173_api::warn(&format!("witch hut spawner: {}", error.0));
    }
}

// TFGenFoundation, the stone outline of a lost house over a cellar that sometimes holds a chest
fn foundation(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32)
{
    let width: i32 = 5 + random.nextIntBounded(5);
    let depth: i32 = 5 + random.nextIntBounded(5);
    if !suitable(world, x, y, z, [width, 4, depth])
    {
        return;
    }

    let planks: i32 = world.id("minecraft:planks");

    for cx in 0..=width
    {
        for cz in 0..=depth
        {
            if cx != 0 && cx != width && cz != 0 && cz != depth
            {
                if random.nextIntBounded(3) != 0
                {
                    world.set(x + cx, y - 1, z + cz, planks);
                }

                continue;
            }

            let height: i32 = random.nextIntBounded(4) + 1;
            for cy in 0..=height
            {
                let stone: i32 = randStone(world, random, cy + 1);
                world.set(x + cx, y + cy - 1, z + cz, stone);
            }
        }
    }

    if random.nextIntBounded(2) != 0
    {
        return;
    }

    for cx in 1..width
    {
        for cz in 1..depth
        {
            world.set(x + cx, y - 3, z + cz, 0);
            world.set(x + cx, y - 4, z + cz, 0);
        }
    }

    let cx: i32 = random.nextIntBounded(width - 1) + 1;
    let cz: i32 = random.nextIntBounded(depth - 1) + 1;
    treasure::place(world, random, Table::Basement, x + cx, y - 4, z + cz);
}

// TFGenMonolith, four obsidian columns of falling heights topped with lapis, with ravens about
fn monolith(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32)
{
    let tallest: i32 = random.nextIntBounded(10) + 10;
    let turn: usize = random.nextIntBounded(4) as usize;

    if !suitable(world, x, y, z, [2, tallest, 2])
    {
        return;
    }

    let (full, most, half): (i32, i32, i32) =
        (tallest, (f64::from(tallest) * 0.75) as i32, (f64::from(tallest) * 0.5) as i32);
    let shapes: [[i32; 4]; 4] =
        [[full, most, most, half], [half, full, most, most], [most, half, full, most], [most, most, half, full]];
    let (lapis, obsidian): (i32, i32) = (world.id("minecraft:lapis_block"), world.id("minecraft:obsidian"));

    for (column, [dx, dz]) in [[0, 0], [1, 0], [0, 1], [1, 1]].iter().enumerate()
    {
        for cy in 0..=shapes[turn][column]
        {
            world.set(x + dx, y + cy - 1, z + dz, if cy == tallest { lapis } else { obsidian });
        }
    }

    super::scatter::critters(world, random, x, z, crate::entities::RAVEN, 2);
}

// beta has no stone bricks, mossy stone bricks become mossy cobblestone and chiseled ones plain stone
fn groveRuins(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32)
{
    let (moss, chiseled): (i32, i32) = (mossy(world), world.id("minecraft:stone"));

    let put = |cells: &[[i32; 3]], block: i32|
    {
        for [dx, dy, dz] in cells
        {
            world.set(x + dx, y + dy, z + dz, block);
        }
    };

    if random.nextBoolean()
    {
        if !suitable(world, x, y, z, [2, 7, 6])
        {
            return;
        }

        for dy in -2..=7
        {
            put(&[[0, dy, 1], [1, dy, 1], [0, dy, 2], [1, dy, 2]], moss);
        }

        put(
            &[
                [0, -1, 3],
                [1, -1, 3],
                [0, -2, 3],
                [1, -2, 3],
                [0, -1, 4],
                [1, -1, 4],
                [0, -2, 4],
                [1, -2, 4],
                [0, -1, 5],
                [1, -2, 5],
                [0, 6, 3],
                [1, 6, 3],
                [0, 7, 3],
                [1, 7, 3],
                [0, 6, 4],
                [1, 6, 4],
                [0, 7, 4],
                [1, 7, 4],
                [1, 7, 5],
            ],
            moss,
        );
        put(&[[0, 5, 0]], chiseled);
        return;
    }

    if !suitable(world, x, y, z, [7, 5, 9])
    {
        return;
    }

    put(&[[0, 4, 0], [0, 3, 0], [1, 4, 0], [2, 4, 0], [0, 4, 1], [0, 4, 2]], chiseled);

    for dy in -1..=5
    {
        put(&[[3, dy, 0]], moss);
    }

    put(&[[4, -1, 0], [5, -1, 0], [6, -1, 0], [4, 5, 0], [5, 5, 0]], moss);

    for dy in -1..=5
    {
        put(&[[0, dy, 3], [0, dy, 7]], moss);
    }

    for dz in 4..7
    {
        put(&[[0, -1, dz], [0, 5, dz]], moss);
    }

    put(&[[0, 4, 8]], chiseled);
}

// TFGenFallenHollowLog, a fallen hollow log in twilight oak with moss and leaves on it
fn hollowLog(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32)
{
    let alongZ: bool = random.nextBoolean();
    let size: [i32; 3] = if alongZ { [9, 3, 4] } else { [4, 3, 9] };
    if !suitable(world, x, y, z, size)
    {
        return;
    }

    // the source draws the z log in x z order and the x log with the axes swapped, one routine covers both
    let at = |across: i32, dy: i32, along: i32|
    {
        if alongZ
        {
            [x + across, y + dy, z + along]
        }
        else
        {
            [x + along, y + dy, z + across]
        }
    };

    let wood: Placed = log(if alongZ { 8 } else { 4 });
    let (moss, dirt, leaf): (Placed, Placed, Placed) =
        ((world.id("moss_patch"), 0), (world.id("minecraft:dirt"), 0), leaves(0));
    let set = |cell: [i32; 3], placed: Placed| world.setMeta(cell[0], cell[1], cell[2], placed.0, placed.1);
    let jaggies: [[i32; 2]; 6] = [[0, 0], [3, 0], [0, 1], [3, 1], [1, 2], [2, 2]];
    for [across, dy] in jaggies
    {
        for along in -random.nextIntBounded(3)..0
        {
            set(at(across, dy, along + 3), wood);
        }
    }

    for [across, dy] in jaggies
    {
        for along in 0..random.nextIntBounded(3)
        {
            set(at(across, dy, along + 7), wood);
        }
    }

    for along in 3..7
    {
        for across in [1, 2]
        {
            if random.nextBoolean()
            {
                set(at(across, -1, along), wood);

                if random.nextBoolean()
                {
                    set(at(across, 0, along), moss);
                }
            }
            else
            {
                set(at(across, -1, along), dirt);
                set(at(across, 0, along), moss);
            }
        }

        for [across, dy] in [[0, 0], [3, 0], [0, 1], [3, 1], [1, 2], [2, 2]]
        {
            set(at(across, dy, along), wood);
        }

        for across in [1, 2]
        {
            if random.nextBoolean()
            {
                set(at(across, 3, along), moss);
            }
        }
    }

    let offset: i32 = random.nextIntBounded(3) + 2;
    let plus: bool = random.nextBoolean();

    for along in 0..3
    {
        if !random.nextBoolean()
        {
            continue;
        }

        set(at(if plus { 3 } else { 0 }, 2, along + offset), leaf);

        if random.nextBoolean()
        {
            set(at(if plus { 3 } else { 0 }, 3, along + offset), leaf);
        }

        if random.nextBoolean()
        {
            set(at(if plus { 4 } else { -1 }, 2, along + offset), leaf);
        }
    }
    // the firefly clings to the side of the upper log like a torch
    let side: i32 = if plus { 0 } else { 3 };

    let facing: i32 = match (alongZ, plus)
    {
        (true, true) => 2,
        (true, false) => 1,
        (false, true) => 4,
        (false, false) => 3,
    };
    set(at(side, 2, random.nextIntBounded(4) + 3), (world.id(block::FIREFLY), facing));
}

// TFGenFallenSmallLog, a short fallen log of a random wood with a side branch
fn smallLog(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32)
{
    let alongX: bool = random.nextBoolean();
    let length: i32 = random.nextIntBounded(4) + 3;

    // the source swaps height and depth for a log along z
    let size: [i32; 3] = if alongX
    {
        [length, 3, 2]
    }
    else
    {
        [3, length, 2]
    };

    if !suitable(world, x, y, z, size)
    {
        return;
    }

    let vanilla: i32 = world.id("minecraft:log");
    let kind: i32 = random.nextIntBounded(7);

    // beta logs have no orientation, its spruce and birch lie upright and jungle becomes oak
    let wood = |bits: i32| -> Placed
    {
        match kind
        {
            1 | 2 => log(kind | bits),
            3 | 6 => (vanilla, 0),
            4 => (vanilla, 1),
            5 => (vanilla, 2),
            _ => log(bits),
        }
    };

    let (lying, branch): (Placed, Placed) = if alongX
    {
        (wood(4), wood(8))
    }
    else
    {
        (wood(8), wood(4))
    };
    let moss: i32 = world.id("moss_patch");

    let at = |along: i32, side: i32|
    {
        if alongX
        {
            (x + along, z + side)
        }
        else
        {
            (x + side, z + along)
        }
    };

    for along in 0..length
    {
        let (bx, bz): (i32, i32) = at(along, 1);
        world.setMeta(bx, y, bz, lying.0, lying.1);

        if random.nextIntBounded(3) > 0
        {
            world.set(bx, y + 1, bz, moss);
        }
    }

    if random.nextIntBounded(3) == 0
    {
        return;
    }

    let (along, side): (i32, i32) = if alongX
    {
        let along: i32 = random.nextIntBounded(length);
        (along, if random.nextBoolean() { 2 } else { 0 })
    }
    else
    {
        let side: i32 = if random.nextBoolean() { 2 } else { 0 };
        (random.nextIntBounded(length), side)
    };
    let (bx, bz): (i32, i32) = at(along, side);
    world.setMeta(bx, y, bz, branch.0, branch.1);

    if random.nextBoolean()
    {
        world.set(bx, y + 1, bz, moss);
    }
}
