use super::bronze::fillChest;
use super::writer::{AIR, Pick, RandomTracker, Writer};
use super::{START_BOX, START_LIFT, START_OFFSET, loot};
use crate::entities::bosses;
use crate::world::GenWorld;
use b173_api::BlockPos;
use b173_api::terrain::Random;
use b173_api::terrain::structure::StructureBox;

mod silver
{
    // MapGenSilverDungeon and ComponentSilverDungeon
    pub const ODDS: [i32; 2] = [110, 150];
    // the roof tops out 45 above the box floor
    pub const BASE: [i32; 2] = [50, 82];
    pub const GRID: i32 = 6;
    pub const CLOUD_TRIES: i32 = 100;
    pub const CLOUD_SIZE: i32 = 10;
    pub const CLOUD_SPREAD: [i32; 2] = [77, 50];
    pub const CLOUD_OFFSET: [i32; 3] = [21, 17, 20];
    pub const OFFSET: [i32; 3] = [31, 24, 30];
    pub const MOSS_CHANCE: i32 = 30;
    pub const LOCKED_CHANCE: i32 = 20;
    pub const TRAP_CHANCE: i32 = 30;
    pub const STAIR_CHANCE: i32 = 15;
    pub const COLUMN_HEIGHT: i32 = 14;
    pub const ROOF_STEPS: i32 = 7;
    pub const CHANDELIER_HEIGHT: i32 = 8;
    pub const CHANDELIERS: [[i32; 2]; 4] = [[28, 10], [43, 10], [43, 19], [28, 19]];
    pub const ELLIPSE: i32 = 21;
    pub const BLUE_WOOL: i32 = 11;
    pub const CHEST_TYPES: i32 = 3;
    pub const CORE_CHANCE: i32 = 5;
    pub const SAPLING_CHANCE: i32 = 2;
}

// MapGenSilverDungeon.Start: the stair rows, cloud drift and box height drawn from the world seed
pub(crate) struct Start
{
    m_stairs: [i32; 3],
    m_tendency: [i32; 2],
    pub m_box: StructureBox,
}

pub(crate) fn canSpawn(random: &Random, chunkX: i32, chunkZ: i32) -> bool
{
    let mut tracker: RandomTracker = RandomTracker::default();
    let rare: bool = tracker.test(random, silver::ODDS[0]) != 0 && tracker.test(random, silver::ODDS[1]) != 0;
    !rare && chunkX % silver::GRID == 0 && chunkZ % silver::GRID == 0
}

pub(crate) fn start(seed: i64, chunkX: i32, chunkZ: i32) -> Start
{
    let random: Random = super::startRandom(seed, chunkX, chunkZ);
    let min: [i32; 3] = [(chunkX << 4) + START_OFFSET, START_BOX[0], (chunkZ << 4) + START_OFFSET];
    let mut component: StructureBox =
        StructureBox::new(min, [min[0] + START_BOX[2], START_BOX[1], min[2] + START_BOX[2]]);
    let stairs: [i32; 3] = [random.nextIntBounded(3), random.nextIntBounded(3), random.nextIntBounded(3)];
    let tendency: [i32; 2] = [random.nextIntBounded(3) - 1, random.nextIntBounded(3) - 1];

    super::fitLift(&mut component, random.nextIntBounded(START_LIFT), silver::BASE[0], silver::BASE[1]);
    Start { m_stairs: stairs, m_tendency: tendency, m_box: component }
}

struct Blocks
{
    m_locked: Pick,
    m_lockedLight: Pick,
    m_trap: Pick,
    m_pillar: Pick,
    m_pillarTop: Pick,
    m_torch: Pick,
    m_slab: Pick,
    m_doubleSlab: Pick,
    m_glowstone: Pick,
    m_fence: Pick,
}

fn blocks(world: &GenWorld) -> Blocks
{
    let pick = |name: &'static str| (world.id(name), 0);

    Blocks
    {
        m_locked: pick("locked_angelic_stone"),
        m_lockedLight: pick("locked_light_angelic_stone"),
        m_trap: pick("angelic_trap"),
        m_pillar: pick("pillar"),
        m_pillarTop: pick("pillar_top"),
        m_torch: pick("ambrosium_torch"),
        m_slab: pick("minecraft:stone_slab"),
        m_doubleSlab: pick("minecraft:double_stone_slab"),
        m_glowstone: pick("minecraft:glowstone"),
        m_fence: pick("minecraft:fence"),
    }
}

// ComponentSilverDungeon.generate
pub(crate) fn generate(world: &GenWorld, random: &Random, start: &Start, clip: StructureBox)
{
    let mut writer: Writer = Writer::structure(world, random, &start.m_box, clip);
    writer.m_replaceAir = true;

    let mut dungeon: Silver =
        Silver { m_writer: writer, m_blocks: blocks(world), m_rooms: [[[0; 3]; 3]; 3], m_stairs: start.m_stairs };
    dungeon.clouds(start.m_tendency);
    dungeon.build();
}

struct Silver<'a>
{
    m_writer: Writer<'a>,
    m_blocks: Blocks,
    m_rooms: [[[i32; 3]; 3]; 3],
    m_stairs: [i32; 3],
}

impl Silver<'_>
{
    fn random(&self) -> &Random { self.m_writer.m_random }

    fn locked(&mut self, chance: i32)
    {
        self.m_writer.setChance(self.m_blocks.m_locked, self.m_blocks.m_lockedLight, chance);
    }

    // AetherGenUtils.generateClouds, its loop bounds draw again on every pass and it tests air without the offset
    fn clouds(&mut self, tendency: [i32; 2])
    {
        self.m_writer.setOffset(silver::CLOUD_OFFSET);

        let cloud: Pick = (self.m_writer.m_world.id("cold_aercloud"), 0);

        for _ in 0..silver::CLOUD_TRIES
        {
            let random: &Random = self.random();
            let [mut x, mut y, mut z]: [i32; 3] =
                [random.nextIntBounded(silver::CLOUD_SPREAD[0]), 0, random.nextIntBounded(silver::CLOUD_SPREAD[1])];

            for _ in 0..silver::CLOUD_SIZE
            {
                x += random.nextIntBounded(3) - 1 + tendency[0];

                if random.nextBoolean()
                {
                    y += random.nextIntBounded(3) - 1;
                }

                z += random.nextIntBounded(3) - 1 + tendency[1];

                let mut x1: i32 = x;
                while x1 < x + random.nextIntBounded(4) + 3
                {
                    let mut y1: i32 = y;
                    while y1 < y + random.nextIntBounded(1) + 2
                    {
                        let mut z1: i32 = z;
                        while z1 < z + random.nextIntBounded(4) + 3
                        {
                            if self.m_writer.get(x1, y1, z1) == 0
                                && (x1 - x).abs() + (y1 - y).abs() + (z1 - z).abs() < 4 + random.nextIntBounded(2)
                            {
                                self.m_writer.putOffset(x1, y1, z1, cloud);
                            }

                            z1 += 1;
                        }

                        y1 += 1;
                    }

                    x1 += 1;
                }
            }
        }
    }

    fn column(&mut self, x: i32, y: i32, z: i32, height: i32)
    {
        self.locked(silver::LOCKED_CHANCE);
        self.m_writer.planeY(x, y, z, 3, 3);
        self.m_writer.planeY(x, y + height, z, 3, 3);
        self.m_writer.setChance(self.m_blocks.m_pillar, self.m_blocks.m_pillar, 1);
        self.m_writer.lineY(x + 1, y, z + 1, height - 1);
        self.m_writer.putOffset(x + 1, y + height - 1, z + 1, self.m_blocks.m_pillarTop);
    }

    fn staircase(&mut self, x: i32, y: i32, z: i32, height: i32)
    {
        const STEPS: [[i32; 3]; 10] = [
            [1, 0, 1],
            [2, 0, 1],
            [3, 1, 1],
            [4, 1, 1],
            [4, 2, 2],
            [4, 2, 3],
            [4, 3, 4],
            [3, 3, 4],
            [2, 4, 4],
            [1, 4, 4],
        ];
        const UPPER: [[i32; 3]; 10] = [
            [1, 5, 3],
            [1, 5, 2],
            [1, 6, 1],
            [2, 6, 1],
            [3, 7, 1],
            [4, 7, 1],
            [4, 8, 2],
            [4, 8, 3],
            [4, 9, 4],
            [3, 9, 4],
        ];
        const SHORT: i32 = 5;
        self.m_writer.setChance(AIR, AIR, 1);
        self.m_writer.solidBox([x + 1, y, z + 1], [4, height, 4]);
        self.locked(silver::CORE_CHANCE);
        self.m_writer.solidBox([x + 2, y, z + 2], [2, height + 4, 2]);

        let upper: &[[i32; 3]] = if height != SHORT { &UPPER } else { &[] };
        for (index, [dx, dy, dz]) in STEPS.iter().chain(upper).enumerate()
        {
            let pick: Pick = if index % 2 == 0
            {
                self.m_blocks.m_slab
            }
            else
            {
                self.m_blocks.m_doubleSlab
            };
            self.m_writer.putOffset(x + dx, y + dy, z + dz, pick);
        }
    }

    fn doorX(&self, x: i32, y: i32, z: i32)
    {
        for (dy, dz) in [(0, 0), (0, 1), (1, 0), (1, 1)]
        {
            self.m_writer.putOffset(x, y + dy, z + dz, AIR);
        }
    }

    fn doorZ(&self, z: i32, x: i32, y: i32)
    {
        for (dx, dy) in [(0, 0), (0, 1), (1, 0), (1, 1)]
        {
            self.m_writer.putOffset(x + dx, y + dy, z, AIR);
        }
    }

    fn shell(&mut self)
    {
        self.m_writer.setOffset(silver::OFFSET);
        self.m_writer.m_replaceSolid = true;

        let holystone: Pick = (self.m_writer.m_world.id("holystone"), 0);
        let mossy: Pick = (self.m_writer.m_world.id("mossy_holystone"), 0);
        self.m_writer.setChance(holystone, mossy, silver::MOSS_CHANCE);
        self.m_writer.solidBox([0, -5, 0], [55, 5, 30]);

        for x in (0..55).step_by(4)
        {
            self.column(x, 0, 0, silver::COLUMN_HEIGHT);
            self.column(x, 0, 27, silver::COLUMN_HEIGHT);
        }

        for z in (0..12).step_by(4).chain((19..30).step_by(4))
        {
            self.column(0, 0, z, silver::COLUMN_HEIGHT);
            self.column(52, 0, z, silver::COLUMN_HEIGHT);
        }

        self.locked(silver::LOCKED_CHANCE);
        self.m_writer.hollowBox([4, -1, 4], [47, 16, 22]);

        for x in [11, 18, 25]
        {
            self.m_writer.planeX(x, 0, 5, 15, 20);
        }

        self.m_writer.planeZ(5, 0, 11, 20, 15);
        self.m_writer.planeZ(5, 0, 18, 20, 15);
        self.m_writer.setChance(self.m_blocks.m_locked, self.m_blocks.m_trap, silver::TRAP_CHANCE);
        self.m_writer.planeY(5, 4, 5, 20, 20);
        self.m_writer.planeY(5, 9, 5, 20, 20);

        for y in 0..2
        {
            for z in 14..16
            {
                self.m_writer.putOffset(4, y, z, AIR);
            }
        }

        self.m_writer.setChance(AIR, AIR, 1);

        for step in 0..4
        {
            self.m_writer.solidBox([step, -4 + step, 14], [1, 4 - step, 2]);
        }

        self.locked(silver::STAIR_CHANCE);

        for y in 0..silver::ROOF_STEPS
        {
            self.m_writer.planeY(-1, 15 + y, -1 + 2 * y, 57, 32 - 4 * y);
        }
    }

    fn stairs(&mut self)
    {
        let [first, second, last] = self.m_stairs;
        self.staircase(19, 0, 5 + last * 7, 10);
        self.m_rooms[2][0][last as usize] = 2;
        self.m_rooms[2][1][last as usize] = 2;
        self.m_rooms[2][2][last as usize] = 1;

        for y in 0..2
        {
            for z in 7 + 7 * last..9 + 7 * last
            {
                self.m_writer.putOffset(25, y, z, AIR);
            }
        }

        self.staircase(12, 0, 5 + first * 7, 5);
        self.m_rooms[1][0][first as usize] = 1;
        self.m_rooms[1][1][first as usize] = 1;
        self.staircase(5, 5, 5 + second * 7, 5);
        self.m_rooms[0][1][second as usize] = 1;
        self.m_rooms[0][2][second as usize] = 1;
    }

    // the doors toward the neighbouring rows of the floor, each row number gets its own set
    fn sideDoors(&self, r: i32, x: i32, y: i32, low: i32)
    {
        match r
        {
            0 => self.doorZ(low, x, y),

            1 =>
            {
                self.doorZ(4 + 7 * r, x, y);
                self.doorZ(11 + 7 * r, x, y);
            }

            _ => self.doorZ(4 + 7 * r, x, y),
        }
    }

    fn rooms(&mut self)
    {
        let [first, second, last] = self.m_stairs;

        for p in 0..3
        {
            for q in 0..3
            {
                for r in 0..3
                {
                    let (x, y, z) = (7 + 7 * p, 5 * q, 7 + 7 * r);

                    if p == 0 && q != 0 && second == r
                    {
                        self.doorX(11 + 7 * p, y, z);
                        // the first row's door sits at -3, outside the room, as in the Java
                        self.sideDoors(r, x, y, -3 + 7 * r);
                    }
                    else if p == 1 && q != 2 && first == r
                    {
                        if first != last
                        {
                            self.doorX(11 + 7 * p, y, z);
                        }

                        self.sideDoors(r, x, y, 11 + 7 * r);
                        self.doorX(4 + 7 * p, y, z);
                    }
                    else if p != 2 || last != r
                    {
                        self.plainRoom(p as usize, q as usize, r as usize);
                    }
                    else if q == 0
                    {
                        self.doorX(11 + 7 * p, y, z);
                    }
                    else if q == 2
                    {
                        self.doorX(4 + 7 * p, y, z);
                        self.sideDoors(r, x, y, 11 + 7 * r);
                    }
                }
            }
        }
    }

    // a room off the stairs: doors toward rooms it may join, then a trap with a chest or mimics
    fn plainRoom(&mut self, p: usize, q: usize, r: usize)
    {
        let (pi, qi, ri) = (p as i32, q as i32, r as i32);
        let opens = |kind: i32, other: i32| other != 2 && (other != 1 || kind != 1);
        let mut kind: i32 = self.m_rooms[p][q][r];
        if p + 1 < 3 && opens(kind, self.m_rooms[p + 1][q][r])
        {
            kind = 3;
            self.m_rooms[p][q][r] = kind;
            self.doorX(11 + 7 * pi, 5 * qi, 7 + 7 * ri);
        }

        if p >= 2 && opens(kind, self.m_rooms[p - 1][q][r])
        {
            kind = 4;
            self.m_rooms[p][q][r] = kind;
            self.doorX(4 + 7 * pi, 5 * qi, 7 + 7 * ri);
        }

        if r + 1 < 3 && opens(kind, self.m_rooms[p][q][r + 1])
        {
            kind = 5;
            self.m_rooms[p][q][r] = kind;
            self.doorZ(11 + 7 * ri, 7 + 7 * pi, 5 * qi);
        }

        if r >= 2 && opens(kind, self.m_rooms[p][q][r - 1])
        {
            kind = 6;
            self.m_rooms[p][q][r] = kind;
            self.doorZ(4 + 7 * ri, 7 + 7 * pi, 5 * qi);
        }

        let contents: i32 = self.random().nextIntBounded(silver::CHEST_TYPES);

        if kind < 3
        {
            return;
        }

        let (x, y, z) = (7 + 7 * pi, 5 * qi, 7 + 7 * ri);
        self.m_writer.putOffset(x, y - 1, z, self.m_blocks.m_trap);

        match contents
        {
            1 =>
            {
                self.m_writer.planeY(x, y, z, 2, 2);

                let u: i32 = x + self.random().nextIntBounded(2);
                let v: i32 = z + self.random().nextIntBounded(2);
                if self.m_writer.get(u, y + 1, v) != 0
                {
                    return;
                }

                self.m_writer.putOffset(u, y + 1, v, (self.m_writer.m_world.id("minecraft:chest"), 0));

                let at: [i32; 3] = self.m_writer.actualOffset(u, y + 1, v);
                if self.m_writer.inside(at)
                {
                    fillChest(
                        self.m_writer.m_world,
                        self.random(),
                        BlockPos::new(at[0], at[1], at[2]),
                        loot::silverRoom,
                    );
                }
            }

            2 =>
            {
                self.m_writer.planeY(x, y, z, 2, 2);

                let mimic: Pick = (self.m_writer.m_world.id("chest_mimic"), 0);
                let [u, v] = [x + self.random().nextIntBounded(2), z + self.random().nextIntBounded(2)];
                self.m_writer.putOffset(u, y + 1, v, mimic);

                if self.random().nextBoolean()
                {
                    let [u, v] = [x + self.random().nextIntBounded(2), z + self.random().nextIntBounded(2)];
                    self.m_writer.putOffset(u, y + 1, v, mimic);
                }
            }

            _ =>
            {}
        }
    }

    // the queen's hall: an ellipse of light stone, her throne, pools, chandeliers and saplings
    fn hall(&mut self)
    {
        for x in 0..24
        {
            for z in 0..20
            {
                let distance: i32 = (((x * x + (z - 7) * (z - 7)) as f64).sqrt()
                    + ((x * x + (z - 12) * (z - 12)) as f64).sqrt()) as i32;

                match distance.cmp(&silver::ELLIPSE)
                {
                    std::cmp::Ordering::Equal => self.m_writer.putOffset(26 + x, 0, 5 + z, self.m_blocks.m_lockedLight),
                    std::cmp::Ordering::Greater => self.m_writer.putOffset(26 + x, 0, 5 + z, self.m_blocks.m_locked),
                    std::cmp::Ordering::Less =>
                    {}
                }
            }
        }

        self.locked(silver::LOCKED_CHANCE);
        self.m_writer.planeY(44, 1, 11, 6, 8);
        self.m_writer.solidBox([46, 2, 13], [4, 2, 4]);
        self.m_writer.lineX(46, 4, 13, 4);
        self.m_writer.lineX(46, 4, 16, 4);
        self.m_writer.planeX(49, 4, 13, 4, 4);

        let wool: Pick = (self.m_writer.m_world.id("minecraft:wool"), silver::BLUE_WOOL);
        self.m_writer.setChance(wool, wool, silver::LOCKED_CHANCE);
        self.m_writer.planeY(47, 3, 14, 2, 2);

        for x in 0..2
        {
            for z in 0..2
            {
                self.m_writer.putOffset(44 + x * 5, 2, 11 + z * 7, self.m_blocks.m_torch);
            }
        }

        self.locked(silver::LOCKED_CHANCE);
        self.m_writer.planeY(35, 1, 5, 6, 3);
        self.m_writer.planeY(35, 1, 22, 6, 3);
        self.m_writer.lineZ(34, 1, 5, 2);
        self.m_writer.lineZ(41, 1, 5, 2);
        self.m_writer.lineX(36, 1, 8, 4);
        self.m_writer.lineZ(34, 1, 23, 2);
        self.m_writer.lineZ(41, 1, 23, 2);
        self.m_writer.lineX(36, 1, 21, 4);

        let water: Pick = (self.m_writer.m_world.id("minecraft:water"), 0);
        self.m_writer.setChance(water, water, 1);
        self.m_writer.planeY(35, 1, 5, 6, 3);
        self.m_writer.planeY(35, 1, 22, 6, 3);

        for [x, z] in [[35, 7], [40, 7], [35, 22], [40, 22]]
        {
            self.m_writer.putOffset(x, 1, z, self.m_blocks.m_locked);
        }

        for x in [36, 39]
        {
            for z in [8, 21]
            {
                self.m_writer.putOffset(x, 2, z, self.m_blocks.m_torch);
            }
        }

        for [x, z] in silver::CHANDELIERS
        {
            self.chandelier(x, 0, z, silver::CHANDELIER_HEIGHT);
        }

        self.sapling(45, 1, 6);
        self.sapling(45, 1, 21);
    }

    fn chandelier(&self, x: i32, y: i32, z: i32, height: i32)
    {
        for lineY in y + height + 3..y + height + 6
        {
            self.m_writer.putOffset(x, lineY, z, self.m_blocks.m_fence);
        }

        for lineX in x - 1..x + 2
        {
            self.m_writer.putOffset(lineX, y + height + 1, z, self.m_blocks.m_glowstone);
        }

        for lineY in y + height..y + height + 3
        {
            self.m_writer.putOffset(x, lineY, z, self.m_blocks.m_glowstone);
        }

        for lineZ in z - 1..z + 2
        {
            self.m_writer.putOffset(x, y + height + 1, lineZ, self.m_blocks.m_glowstone);
        }
    }

    fn sapling(&mut self, x: i32, y: i32, z: i32)
    {
        self.locked(silver::SAPLING_CHANCE);
        self.m_writer.planeY(x, y, z, 3, 3);
        self.m_writer.putOffset(x + 1, y, z + 1, (self.m_writer.m_world.id("aether_dirt"), 0));
        self.m_writer.putOffset(x + 1, y + 1, z + 1, (self.m_writer.m_world.id("golden_oak_sapling"), 0));

        for lineX in [x, x + 2]
        {
            for lineZ in [z, z + 2]
            {
                self.m_writer.putOffset(lineX, y + 1, lineZ, self.m_blocks.m_torch);
            }
        }
    }

    fn build(&mut self)
    {
        self.shell();
        self.stairs();
        self.rooms();
        self.hall();

        let home: [i32; 3] = self.m_writer.actualOffset(26, 0, 24);
        let queen: [i32; 3] = self.m_writer.actualOffset(40, 1, 15);
        if self.m_writer.inside(queen)
        {
            let at: [f64; 3] = queen.map(|axis| axis as f64 + 0.5);
            bosses::spawn(self.m_writer.m_world.m_world, bosses::VALKYRIE_QUEEN, at, home, 0);
        }

        self.locked(silver::LOCKED_CHANCE);
        self.m_writer.hollowBox([41, -2, 13], [4, 4, 4]);

        let x: i32 = 42 + self.random().nextIntBounded(2);
        let z: i32 = 14 + self.random().nextIntBounded(2);
        self.m_writer.putOffset(x, -1, z, (self.m_writer.m_world.id("treasure_chest"), 0));
    }
}
