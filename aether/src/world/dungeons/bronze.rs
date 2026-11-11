use super::loot;
use super::writer::{AIR, Pick, RandomTracker, Writer, shuffle};
use crate::entities::bosses;
use crate::world::GenWorld;
use b173_api::terrain::Random;
use b173_api::terrain::structure::StructureBox;
use b173_api::{BlockPos, ItemStack};

mod size
{
    // BronzeDungeon
    pub const BOSS_ROOM: [i32; 3] = [16, 12, 16];
    pub const BOSS_CHECK: [i32; 3] = [16, 18, 16];
    pub const BOSS_CHECK_DROP: i32 = 3;
    pub const VAULT: [i32; 3] = [4, 4, 4];
    pub const ENTRY_ROOM: [i32; 3] = [12, 12, 12];
    pub const ROOM: [i32; 3] = [12, 8, 12];
    pub const TUBE: [i32; 3] = [6, 6, 6];
    pub const STEP: i32 = 16;
    pub const MAX_TUNNEL: i32 = 100;
    pub const LOCKED_CHANCE: i32 = 20;
    pub const FILLER_CHANCE: i32 = 5;
    pub const TRAP_ODDS: i32 = 100;
    pub const BOSS_ODDS: [i32; 2] = [15, 40];
    pub const EXTRA_ROOMS: i32 = 4;
    pub const MIN_ROOMS: i32 = 2;
    pub const CHEST_SLOTS: i32 = 27;
    pub const MIN_LOOT: i32 = 3;
    pub const LOOT_SPREAD: i32 = 3;
}

const SIDES: [i32; 4] = [1, 2, 3, 4];

struct Palette
{
    m_locked: Pick,
    m_lockedLight: Pick,
    m_main: Pick,
    m_mainLight: Pick,
    m_filler: Pick,
    m_fillerMossy: Pick,
    m_trap: Pick,
}

const PALETTE: Palette = Palette
{
    m_locked: Pick::of("locked_carved_stone"),
    m_lockedLight: Pick::of("locked_sentry_stone"),
    m_main: Pick::of("carved_stone"),
    m_mainLight: Pick::of("sentry_stone"),
    m_filler: Pick::of("holystone"),
    m_fillerMossy: Pick::of("mossy_holystone"),
    m_trap: Pick::of("carved_trap"),
};

// BronzeDungeon: a locked boss room with the slider and its treasure, an entry room, a chain of trapped rooms and a
// tunnel back out to the open air
struct Bronze<'a>
{
    m_writer: Writer<'a>,
    m_needsCorridor: bool,
    m_roomMaximum: i32,
    m_roomCount: i32,
}

// the rooms and tubes it built, none when the spot failed its checks
pub(crate) fn generate(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32) -> Vec<StructureBox>
{
    let mut writer: Writer = Writer::dungeon(world, random);
    writer.m_replaceAir = true;
    writer.m_replaceSolid = true;

    let roomMaximum: i32 = random.nextIntBounded(size::EXTRA_ROOMS) + size::MIN_ROOMS;

    let mut dungeon: Bronze =
        Bronze { m_writer: writer, m_needsCorridor: false, m_roomMaximum: roomMaximum, m_roomCount: 0 };
    dungeon.bossRoom(x, y, z);
    dungeon.m_writer.m_rooms
}

impl Bronze<'_>
{
    fn random(&self) -> &Random { self.m_writer.m_random }

    fn bossRoom(&mut self, i: i32, j: i32, k: i32) -> bool
    {
        if !self.m_writer.boxSolid([i, j - size::BOSS_CHECK_DROP, k], size::BOSS_CHECK)
            || !self.m_writer.boxSolid([i + 20, j, k + 2], size::ENTRY_ROOM)
        {
            return false;
        }
        let mut tracker: RandomTracker = RandomTracker::default();
        if tracker.test(self.random(), size::BOSS_ODDS[0]) != 0 && tracker.test(self.random(), size::BOSS_ODDS[1]) != 0
        {
            return false;
        }

        self.m_writer.setChance(PALETTE.m_locked, PALETTE.m_lockedLight, size::LOCKED_CHANCE);
        self.m_writer.hollowBox([i, j, k], size::BOSS_ROOM);
        self.m_writer.hollowBox([i + 6, j - 2, k + 6], size::VAULT);
        bosses::spawn(
            self.m_writer.m_world.m_world,
            bosses::SLIDER,
            [(i + 8) as f64, (j + 2) as f64, (k + 8) as f64],
            [i, j, k],
            0,
        );
        self.m_writer.put(i + 7, j - 1, k + 7, Pick::of("treasure_chest"));
        self.entryRoom(i, j, k)
    }

    fn traps(&self, x: i32, y: i32, z: i32)
    {
        const SPACING: usize = 3;

        for p in (x + 2..x + 10).step_by(SPACING)
        {
            for q in (z + 2..z + 10).step_by(SPACING)
            {
                self.m_writer.put(p, y, q, PALETTE.m_trap);
            }
        }
    }

    fn entryRoom(&mut self, i: i32, j: i32, k: i32) -> bool
    {
        let (x, z, tube, angle): (i32, i32, [i32; 3], i32) = match self.random().nextIntBounded(4)
        {
            0 => (i + 20, k + 2, [i + 15, j, k + 5], 0),
            1 => (i - 16, k + 2, [i - 5, j, k + 5], 0),
            2 => (i + 2, k + 20, [i + 5, j, k + 15], 2),
            _ => (i + 2, k - 16, [i + 5, j, k - 5], 2),
        };
        if !self.m_writer.boxSolid([x, j, z], size::ENTRY_ROOM)
        {
            return true;
        }

        self.m_writer.setChance(PALETTE.m_main, PALETTE.m_mainLight, size::LOCKED_CHANCE);
        self.m_writer.hollowBox([x, j, z], size::ENTRY_ROOM);
        self.m_writer.setChance(PALETTE.m_filler, PALETTE.m_fillerMossy, size::FILLER_CHANCE);
        self.m_writer.squareTube(tube, size::TUBE, angle);
        self.traps(x, j, z);

        if !self.determineRoomPosition(x, j, z) && self.m_roomCount == 0
        {
            return false;
        }

        if self.m_needsCorridor
        {
            self.endCorridor(x, j, z);
        }

        true
    }

    fn determineRoomPosition(&mut self, x: i32, y: i32, z: i32) -> bool
    {
        if self.m_roomCount >= self.m_roomMaximum
        {
            self.m_needsCorridor = true;
            return true;
        }

        let mut sides: [i32; 4] = SIDES;
        shuffle(self.random(), &mut sides);

        for side in sides
        {
            if self.roomWithSide(x, y, z, side)
            {
                return true;
            }
        }

        self.m_needsCorridor = true;
        false
    }

    fn roomWithSide(&mut self, x: i32, y: i32, z: i32, side: i32) -> bool
    {
        let (dx, dz, direction): (i32, i32, i32) = match side
        {
            1 => (size::STEP, 0, 0),
            2 => (0, size::STEP, 1),
            3 => (-size::STEP, 0, 2),
            _ => (0, -size::STEP, 3),
        };
        self.nextRoom(x + dx, y, z + dz, direction)
    }

    fn roomChest(&self, x: i32, y: i32, z: i32)
    {
        let random: &Random = self.random();
        let kind: i32 = random.nextIntBounded(2);
        let p: i32 = x + 5 + random.nextIntBounded(2);
        let q: i32 = z + 5 + random.nextIntBounded(2);
        let world: &GenWorld = self.m_writer.m_world;

        if kind == 0
        {
            self.m_writer.put(p, y + 2, q, Pick::of("chest_mimic"));
            return;
        }

        if world.block(p, y + 2, q) != 0
        {
            return;
        }

        self.m_writer.put(p, y + 2, q, Pick::of("minecraft:chest"));
        fillChest(world, random, BlockPos::new(p, y + 2, q), loot::bronzeRoom);
    }

    fn nextRoom(&mut self, x: i32, y: i32, z: i32, direction: i32) -> bool
    {
        if !self.m_writer.boxSolid([x, y, z], size::ROOM)
        {
            return false;
        }

        self.m_writer.setChance(PALETTE.m_main, PALETTE.m_mainLight, size::LOCKED_CHANCE);
        self.m_writer.hollowBox([x, y, z], size::ROOM);

        for p in x..x + size::ROOM[0]
        {
            for q in y..y + size::ROOM[1]
            {
                for r in z..z + size::ROOM[2]
                {
                    if self.m_writer.m_world.is(p, q, r, PALETTE.m_main.m_block)
                        && self.random().nextIntBounded(size::TRAP_ODDS) == 0
                    {
                        self.m_writer.put(p, q, r, PALETTE.m_trap);
                    }
                }
            }
        }

        const CORNERS: usize = 7;

        for p in (x + 2..x + 10).step_by(CORNERS)
        {
            for q in (z + 2..z + 10).step_by(CORNERS)
            {
                self.m_writer.put(p, y, q, PALETTE.m_trap);
            }
        }

        self.m_writer.planeY(x + 4, y + 1, z + 4, 4, 4);
        self.roomChest(x, y, z);
        self.m_writer.setChance(PALETTE.m_filler, PALETTE.m_fillerMossy, size::FILLER_CHANCE);

        let (tube, angle): ([i32; 3], i32) = match direction
        {
            0 => ([x - 5, y, z + 3], 0),
            1 => ([x + 3, y, z - 5], 2),
            2 => ([x + 11, y, z + 3], 0),
            _ => ([x + 3, y, z + 11], 2),
        };
        self.m_writer.squareTube(tube, size::TUBE, angle);
        self.m_roomCount += 1;
        // the Java returns the second call when the first succeeds, so a placed room tries again
        self.determineRoomPosition(x, y, z) && self.determineRoomPosition(x, y, z)
    }

    fn endCorridor(&mut self, x: i32, y: i32, z: i32) -> bool
    {
        let mut sides: [i32; 4] = SIDES;
        shuffle(self.random(), &mut sides);
        sides.iter().any(|side| self.endCorridorSide(x, y, z, *side))
    }

    // generateEndCorridor: tunnels from the entry room until it breaks into open air or back into carved stone
    fn endCorridorSide(&mut self, x: i32, y: i32, z: i32, side: i32) -> bool
    {
        if !self.m_needsCorridor
        {
            return false;
        }

        self.m_writer.m_replaceAir = false;

        // start, step, the box ahead checked for solid stone and the slice the tunnel occupies
        let (start, step, ahead, aheadSize, slice): ([i32; 3], [i32; 3], [i32; 3], [i32; 3], [i32; 3]) = match side
        {
            1 => ([x + 11, y, z + 3], [1, 0, 0], [1, 0, 0], [2, 8, 6], [1, 8, 6]),
            2 => ([x, y, z + 3], [-1, 0, 0], [-1, 0, 0], [1, 8, 6], [1, 8, 6]),
            3 => ([x + 3, y, z + 11], [0, 0, 1], [0, 0, 1], [6, 8, 2], [6, 8, 1]),
            _ => ([x + 3, y, z], [0, 0, -1], [0, 0, -1], [6, 8, 1], [6, 8, 1]),
        };
        let shifted = |at: [i32; 3]| [at[0] + ahead[0], at[1], at[2] + ahead[2]];
        if !self.m_writer.boxSolid(shifted(start), aheadSize)
        {
            return false;
        }

        let along: usize = if step[0] != 0 { 0 } else { 2 };
        let room: [i32; 3] = [x, y, z];
        let mut at: [i32; 3] = start;
        let mut tunnelling: bool = true;
        let mut tooLong: bool = false;

        while tunnelling
        {
            if self.m_writer.boxEmpty(at, slice)
            {
                tunnelling = false;
            }

            let next: [i32; 3] = shifted(at);
            if self.m_writer.hasBlock(next, aheadSize, PALETTE.m_main.m_block)
                || self.m_writer.hasBlock(next, aheadSize, PALETTE.m_locked.m_block)
            {
                tunnelling = false;
            }

            if (at[along] - room[along]).abs() > size::MAX_TUNNEL
            {
                tooLong = true;
                tunnelling = false;
            }

            self.m_writer.setChance(PALETTE.m_filler, PALETTE.m_fillerMossy, size::FILLER_CHANCE);

            let (wall, hole): ([i32; 3], [i32; 3]) = if along == 0
            {
                ([1, 8, 6], [1, 6, 4])
            }
            else
            {
                ([6, 8, 1], [4, 6, 1])
            };
            self.m_writer.fill(at, wall);
            self.m_writer.setChance(AIR, AIR, 1);

            let inset: [i32; 3] = if along == 0
            {
                [at[0], at[1] + 1, at[2] + 1]
            }
            else
            {
                [at[0] + 1, at[1] + 1, at[2]]
            };
            self.m_writer.fill(inset, hole);
            at[along] += step[along];
        }

        if tooLong
        {
            return false;
        }

        self.m_needsCorridor = false;
        true
    }
}

// a few loot rolls into random slots of a chest, the same slot may be rolled twice
pub(crate) fn fillChest(world: &GenWorld, random: &Random, pos: BlockPos, table: fn(&Random) -> loot::Loot)
{
    // the Java loop draws a fresh bound every pass
    let mut roll: i32 = 0;
    while roll < size::MIN_LOOT + random.nextIntBounded(size::LOOT_SPREAD)
    {
        roll += 1;

        let slot: i32 = random.nextIntBounded(size::CHEST_SLOTS);
        let (name, count): loot::Loot = table(random);
        let _ = world.m_world.setTileSlot(pos, slot, Some(ItemStack::of(name, count)));
    }
}
