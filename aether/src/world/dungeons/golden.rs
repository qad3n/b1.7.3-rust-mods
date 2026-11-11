use super::writer::{AIR, Pick, RandomTracker, Writer};
use super::{START_BOX, START_LIFT, START_OFFSET};
use crate::entities::bosses;
use crate::world::GenWorld;
use b173_api::terrain::structure::StructureBox;
use b173_api::terrain::{Random, math};

mod gold
{
    // MapGenGoldenDungeon and its components
    pub const ODDS: [i32; 2] = [140, 170];
    // the island reaches 27 above and below its center
    pub const BASE: [i32; 2] = [50, 100];
    pub const GRID: i32 = 10;
    pub const STUBS_MIN: i32 = 8;
    pub const STUBS_SPREAD: i32 = 5;
    pub const STUB_REACH: f32 = 24.0;
    pub const STUB_DROP: f64 = 0.3;
    pub const STUB_SIZE: i32 = 8;
    pub const STUB_RADIUS: f64 = 8.0;
    pub const DEGREE: f32 = 0.01745329;
    pub const FULL_TURN: f32 = 360.0;
    pub const OFFSET: [i32; 3] = [60, 0, 60];
    pub const ISLAND: i32 = 24;
    pub const ISLAND_CHECK: i32 = 21;
    pub const ISLAND_STONE_LIMIT: i32 = 12;
    pub const TOP_SQUASH: f64 = 1.375;
    pub const BOTTOM_SQUASH: f32 = 1.35;
    pub const WIDEN: f64 = 0.8;
    pub const CAVES: i32 = 18;
    pub const FLOWER_ODDS: i32 = 48;
    pub const FLOWER_TRIES: i32 = 64;
    pub const HALL_SHRINK: f64 = 0.8;
    pub const HALL_CHANCE: i32 = 10;
    pub const MOSS_CHANCE: i32 = 5;
    pub const TUNNEL: i32 = 32;
    pub const ROOM_DEPTH: i32 = 6;
    // the Java writes pi as 3.141593F
    #[allow(clippy::approx_constant)]
    pub const PI: f32 = 3.141593;
}

#[derive(Clone, Copy)]
struct Shape
{
    m_size: i32,
    m_radius: f64,
    m_divisor: f64,
    m_squash: (i32, i32),
    m_floor: i32,
}

pub(crate) fn canSpawn(random: &Random, chunkX: i32, chunkZ: i32) -> bool
{
    let mut tracker: RandomTracker = RandomTracker::default();
    let rare: bool = tracker.test(random, gold::ODDS[0]) != 0 && tracker.test(random, gold::ODDS[1]) != 0;
    !rare && chunkX % gold::GRID == 0 && chunkZ % gold::GRID == 0
}

// MapGenGoldenDungeon.Start: the island, a ring of stub islands and the hall facing one of four ways
pub(crate) struct Start
{
    m_direction: i32,
    m_stubs: Vec<[i32; 3]>,
    pub m_box: StructureBox,
}

pub(crate) fn start(seed: i64, chunkX: i32, chunkZ: i32) -> Start
{
    let random: Random = super::startRandom(seed, chunkX, chunkZ);
    let min: [i32; 3] = [(chunkX << 4) + START_OFFSET, START_BOX[0], (chunkZ << 4) + START_OFFSET];
    let mut component: StructureBox =
        StructureBox::new(min, [min[0] + START_BOX[2], START_BOX[1], min[2] + START_BOX[2]]);
    let direction: i32 = random.nextIntBounded(4);
    let count: i32 = gold::STUBS_MIN + random.nextIntBounded(gold::STUBS_SPREAD);
    let stubs: Vec<[i32; 3]> = (0..count)
        .map(|_|
        {
            let angle: f32 = random.nextFloat() * gold::FULL_TURN;
            let reach: f32 = (random.nextFloat() * 0.125 + 0.7) * gold::STUB_REACH;
            let turn: f64 = (gold::DEGREE * angle) as f64;
            let x: i32 = math::floor(turn.cos() * reach as f64);
            let y: i32 = -math::floor(gold::STUB_REACH as f64 * random.nextFloat() as f64 * gold::STUB_DROP);
            let z: i32 = math::floor(-turn.sin() * reach as f64);
            [x, y, z]
        })
        .collect();
    super::fitLift(&mut component, random.nextIntBounded(START_LIFT), gold::BASE[0], gold::BASE[1]);
    Start { m_direction: direction, m_stubs: stubs, m_box: component }
}

struct Ids
{
    m_grass: Pick,
    m_dirt: Pick,
    m_holystone: Pick,
    m_mossy: Pick,
    m_cloud: i32,
    m_lockedAngelic: i32,
    m_locked: Pick,
    m_lockedLight: Pick,
}

fn ids(world: &GenWorld) -> Ids
{
    let pick = |name: &'static str| (world.id(name), 0);

    Ids
    {
        m_grass: (world.id("aether_grass"), 3),
        m_dirt: pick("aether_dirt"),
        m_holystone: pick("holystone"),
        m_mossy: pick("mossy_holystone"),
        m_cloud: world.id("cold_aercloud"),
        m_lockedAngelic: world.id("locked_angelic_stone"),
        m_locked: pick("locked_hellfire_stone"),
        m_lockedLight: pick("locked_light_hellfire_stone"),
    }
}

// BlocksAether.isGood: open air or cloud an island may grow into
fn good(ids: &Ids, block: i32) -> bool { block == 0 || block == ids.m_cloud }

pub(crate) fn generate(world: &GenWorld, random: &Random, start: &Start, clip: StructureBox)
{
    let ids: Ids = ids(world);
    let mut writer: Writer = Writer::structure(world, random, &start.m_box, clip);
    writer.m_replaceAir = true;
    writer.m_replaceSolid = true;
    writer.setOffset(gold::OFFSET);
    island(&writer, &ids);

    for stub in &start.m_stubs
    {
        stubIsland(&writer, &ids, *stub);
    }

    hall(&mut writer, &ids, start.m_direction);
}

// the squashed sphere shared by the island and its stubs: grass over dirt where open above, holystone below
// shape is the size, radius, horizontal divisor, squash start, squash shift and lowest grass layer
fn sphere(writer: &Writer, ids: &Ids, at: [i32; 3], shape: Shape, flowers: bool)
{
    let Shape { m_size: size, m_radius: radius, m_divisor: divisor, m_squash: squash, m_floor: floor } = shape;

    for i in -size..=size
    {
        for k in (-size..=size).rev()
        {
            for j in -size..=size
            {
                let x: i32 = math::floor(i as f64 / divisor);
                let mut y: i32 = k;

                if k > squash.0
                {
                    y = math::floor(y as f64 * gold::TOP_SQUASH) - squash.1;
                }
                else if k < -squash.0
                {
                    y = math::floor((y as f32 * gold::BOTTOM_SQUASH) as f64) + squash.1;
                }
                let z: i32 = math::floor(j as f64 / divisor);
                if (((x * x + y * y + z * z) as f64).sqrt()) > radius
                {
                    continue;
                }

                let [px, py, pz] = [at[0] + i, at[1] + k, at[2] + j];
                if good(ids, writer.getOffset(px, py + 1, pz)) && k > floor
                {
                    writer.putOffset(px, py, pz, ids.m_grass);
                    writer.putOffset(px, py - 1, pz, ids.m_dirt);
                    writer.putOffset(px, py - (1 + writer.m_random.nextIntBounded(2)), pz, ids.m_dirt);

                    if flowers
                    {
                        islandFlowers(writer, ids, px, py, pz);
                    }
                    else if k >= 4
                    {
                        let roll: i32 = writer.m_random.nextIntBounded(64);
                        if roll == 5
                        {
                            writer.m_random.nextIntBounded(3);
                        }
                    }
                }
                else if good(ids, writer.getOffset(px, py, pz))
                {
                    writer.putOffset(px, py, pz, ids.m_holystone);
                }
            }
        }
    }
}

fn islandFlowers(writer: &Writer, ids: &Ids, x: i32, y: i32, z: i32)
{
    const HIGH_GROUND: i32 = 12;

    if y < HIGH_GROUND
    {
        return;
    }

    let random: &Random = writer.m_random;
    match random.nextIntBounded(gold::FLOWER_ODDS)
    {
        3 =>
        {
            random.nextIntBounded(2);
        }

        4 =>
        {
            let name: &'static str = if random.nextIntBounded(2) == 0
            {
                "minecraft:red_flower"
            }
            else
            {
                "minecraft:yellow_flower"
            };
            let at: [i32; 3] = [
                x + random.nextIntBounded(3) - random.nextIntBounded(3),
                y + 1,
                z + random.nextIntBounded(3) - random.nextIntBounded(3),
            ];
            flower(writer, ids, (writer.m_world.id(name), 0), at);
        }

        _ =>
        {}
    }
}

// AetherGenUtils.generateFlower
fn flower(writer: &Writer, ids: &Ids, pick: Pick, at: [i32; 3])
{
    let random: &Random = writer.m_random;
    let [mut x, mut y, mut z] = at;

    for _ in 0..gold::FLOWER_TRIES
    {
        x += random.nextIntBounded(8) - random.nextIntBounded(8);
        y += random.nextIntBounded(4) - random.nextIntBounded(4);
        z += random.nextIntBounded(8) - random.nextIntBounded(8);
        if writer.getOffset(x, y, z) == 0 && writer.getOffset(x, y - 1, z) == ids.m_grass.0
        {
            writer.putOffset(x, y, z, pick);
        }
    }
}

// ComponentGoldenIsland.generateBaseIsland, which gives up early on finding locked angelic stone
fn island(writer: &Writer, ids: &Ids)
{
    let mut found: i32 = 0;

    for i in -gold::ISLAND_CHECK..=gold::ISLAND_CHECK
    {
        for k in (-gold::ISLAND_CHECK..=gold::ISLAND).rev()
        {
            for j in -gold::ISLAND_CHECK..=gold::ISLAND_CHECK
            {
                if writer.getOffset(i, k, j) == ids.m_lockedAngelic
                {
                    found += 1;
                    if found > gold::ISLAND_STONE_LIMIT
                    {
                        return;
                    }
                }
            }
        }
    }

    let shape: Shape = Shape
    {
        m_size: gold::ISLAND,
        m_radius: gold::ISLAND as f64,
        m_divisor: gold::WIDEN,
        m_squash: (15, 6),
        m_floor: 4,
    };
    sphere(writer, ids, [0; 3], shape, true);

    let size: i32 = gold::ISLAND + gold::CAVES / 3;

    for _ in 0..gold::CAVES
    {
        let random: &Random = writer.m_random;
        let at: [i32; 3] = [0, 0, 0].map(|_| random.nextIntBounded(gold::ISLAND) - random.nextIntBounded(gold::ISLAND));
        cave(writer, ids, at, size);
    }
}

// ComponentGoldenIsland.generateCaves, a worm of air through the island's stone, grass and dirt
fn cave(writer: &Writer, ids: &Ids, at: [i32; 3], size: i32)
{
    let random: &Random = writer.m_random;
    let [x, y, z] = at;
    let angle: f32 = random.nextFloat() * gold::PI;
    let reach = |wave: f32| (wave * size as f32 / 8.0) as f64;
    let x0: f64 = (x + 8) as f64 + reach(math::sin(angle));
    let x1: f64 = (x + 8) as f64 - reach(math::sin(angle));
    let z0: f64 = (z + 8) as f64 + reach(math::cos(angle));
    let z1: f64 = (z + 8) as f64 - reach(math::cos(angle));
    let y0: f64 = (y + random.nextIntBounded(3) + 2) as f64;
    let y1: f64 = (y + random.nextIntBounded(3) + 2) as f64;
    let carvable: [i32; 4] = [ids.m_mossy.0, ids.m_holystone.0, ids.m_grass.0, ids.m_dirt.0];

    for step in 0..=size
    {
        let along: f64 = step as f64 / size as f64;
        let cx: f64 = x0 + (x1 - x0) * along;
        let cy: f64 = y0 + (y1 - y0) * along;
        let cz: f64 = z0 + (z1 - z0) * along;
        let thickness: f64 = random.nextDouble() * size as f64 / 16.0;
        let swell: f64 = (math::sin(step as f32 * gold::PI / size as f32) as f64 + 1.0) * thickness + 1.0;
        let bounds = |center: f64| ((center - swell / 2.0) as i32, (center + swell / 2.0) as i32);
        let ((minX, maxX), (minY, maxY), (minZ, maxZ)) = (bounds(cx), bounds(cy), bounds(cz));
        for bx in minX..=maxX
        {
            let dx: f64 = (bx as f64 + 0.5 - cx) / (swell / 2.0);
            if dx * dx >= 1.0
            {
                continue;
            }

            for by in minY..=maxY
            {
                let dy: f64 = (by as f64 + 0.5 - cy) / (swell / 2.0);
                if dx * dx + dy * dy >= 1.0
                {
                    continue;
                }

                for bz in minZ..=maxZ
                {
                    let dz: f64 = (bz as f64 + 0.5 - cz) / (swell / 2.0);
                    let block: i32 = writer.getOffset(bx, by, bz);
                    if dx * dx + dy * dy + dz * dz < 1.0 && carvable.contains(&block)
                    {
                        writer.putOffset(bx, by, bz, AIR);
                    }
                }
            }
        }
    }
}

// ComponentGoldenIslandStub.generate
fn stubIsland(writer: &Writer, ids: &Ids, at: [i32; 3])
{
    let shape: Shape =
        Shape { m_size: gold::STUB_SIZE, m_radius: gold::STUB_RADIUS, m_divisor: 1.0, m_squash: (5, 2), m_floor: 1 };
    sphere(writer, ids, at, shape, false);
}

// ComponentGoldenDungeon.generate: the sun spirit's hellfire hall and the tunnel out of the island
fn hall(writer: &mut Writer, ids: &Ids, direction: i32)
{
    let radius: i32 = (gold::ISLAND as f64 * gold::HALL_SHRINK).floor() as i32;
    let wid: i32 = ((radius * radius / 2) as f64).sqrt() as i32;
    let netherrack: Pick = (writer.m_world.id("minecraft:netherrack"), 0);
    let fire: Pick = (writer.m_world.id("minecraft:fire"), 0);

    writer.setChance(ids.m_locked, ids.m_lockedLight, gold::HALL_CHANCE);

    for j in (-4..=4).rev()
    {
        let mut a: i32 = wid;

        if j >= 3 || j <= -3
        {
            a -= 1;
        }

        if j == 4 || j == -4
        {
            a -= 1;
        }

        for i in -a..=a
        {
            for k in -a..=a
            {
                let wall: bool = i == a || -i == a || k == a || -k == a;
                if j == 4 || (j > -4 && wall)
                {
                    writer.placeOffset(i, j, k);
                }
                else if j > -4
                {
                    writer.putOffset(i, j, k, AIR);
                }
                else
                {
                    writer.placeOffset(i, j, k);

                    if (i == a - 2 || -i == a - 2) && (k == a - 2 || -k == a - 2)
                    {
                        writer.putOffset(i, j + 1, k, netherrack);
                        writer.putOffset(i, j + 2, k, fire);
                    }
                }
            }
        }
    }

    tunnel(writer, ids, wid, direction);

    let boss: [i32; 3] = writer.actualOffset(0, -1, 0);
    if writer.inside(boss)
    {
        let at: [f64; 3] = boss.map(|axis| axis as f64 + 0.5);
        bosses::spawn(writer.m_world.m_world, bosses::SUN_SPIRIT, at, boss, direction);
    }
}

fn tunnel(writer: &mut Writer, ids: &Ids, wid: i32, direction: i32)
{
    let chest: Pick = (writer.m_world.id("treasure_chest"), 0);

    for i in wid..wid + gold::TUNNEL
    {
        for j in -3..2
        {
            for k in -3..4
            {
                let (mut a, mut b): (i32, i32) = if direction / 2 == 0 { (i, k) } else { (k, i) };

                if direction % 2 == 0
                {
                    a = -a;
                    b = -b;
                }

                if !good(ids, writer.getOffset(a, j, b))
                {
                    tunnelBlock(writer, ids, [a, j, b], i - wid, k);
                }

                a = -a;
                b = -b;
                writer.setChance(ids.m_locked, ids.m_lockedLight, gold::HALL_CHANCE);

                if i >= wid + gold::ROOM_DEPTH
                {
                    continue;
                }

                let edge: bool = i == wid || i == wid + 5 || k == 3 || k == -3;
                if j == -3 || j >= 1 || edge
                {
                    writer.placeOffset(a, j, b);
                }
                else if i == wid + 4 && k == 0 && j == -2
                {
                    writer.putOffset(a, j, b, chest);
                }
                else
                {
                    writer.putOffset(a, j, b, AIR);
                }
            }
        }
    }
}

// one block of the tunnel through solid island: floor, air with walls, and a hellfire mouth at the hall
fn tunnelBlock(writer: &mut Writer, ids: &Ids, at: [i32; 3], depth: i32, k: i32)
{
    let [a, j, b] = at;

    writer.setChance(ids.m_holystone, ids.m_mossy, gold::MOSS_CHANCE);

    if j == -3
    {
        writer.placeOffset(a, j, b);
    }
    else if j < 1
    {
        if depth == 0
        {
            if k < 2 && k > -2 && j < 0
            {
                writer.putOffset(a, j, b, AIR);
            }
            else
            {
                writer.setChance(ids.m_locked, ids.m_lockedLight, gold::HALL_CHANCE);
                writer.placeOffset(a, j, b);
            }
        }
        else if k != 3 && k != -3
        {
            writer.putOffset(a, j, b, AIR);
        }
        else
        {
            writer.placeOffset(a, j, b);
        }
    }
    else if depth == 0
    {
        writer.setChance(ids.m_locked, ids.m_lockedLight, gold::HALL_CHANCE);
        writer.placeOffset(a, j, b);
    }
    else
    {
        writer.setChance(ids.m_holystone, ids.m_mossy, gold::MOSS_CHANCE);
        writer.placeOffset(a, j, b);
    }
}
