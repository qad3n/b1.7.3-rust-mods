use crate::world::GenWorld;
use b173_api::terrain::Random;
use b173_api::terrain::structure::StructureBox;

// a block and its metadata
pub(crate) type Pick = (i32, i32);
pub(crate) const AIR: Pick = (0, 0);

// AetherStructure and AetherDungeon: builders that fill lines, planes and boxes with a block, sometimes swapped for
// an extra one. A structure works in its component box clipped to the chunk being populated, a dungeon in plain
// world coordinates
pub(crate) struct Writer<'a>
{
    pub m_world: &'a GenWorld,
    pub m_random: &'a Random,
    m_clip: Option<StructureBox>,
    m_origin: [i32; 3],
    m_offset: [i32; 3],
    pub m_replaceAir: bool,
    pub m_replaceSolid: bool,
    m_block: Pick,
    m_extra: Pick,
    pub m_chance: i32,
    // every room and tube built, so a finished dungeon can be recorded piece by piece
    pub m_rooms: Vec<StructureBox>,
}

impl<'a> Writer<'a>
{
    pub(crate) fn structure(
        world: &'a GenWorld,
        random: &'a Random,
        component: &StructureBox,
        clip: StructureBox,
    ) -> Self
    {
        Writer { m_clip: Some(clip), m_origin: component.m_min, ..Writer::dungeon(world, random) }
    }

    pub(crate) fn dungeon(world: &'a GenWorld, random: &'a Random) -> Self
    {
        Writer
        {
            m_world: world,
            m_random: random,
            m_clip: None,
            m_origin: [0; 3],
            m_offset: [0; 3],
            m_replaceAir: false,
            m_replaceSolid: false,
            m_block: AIR,
            m_extra: AIR,
            m_chance: 0,
            m_rooms: Vec::new(),
        }
    }

    pub(crate) fn setOffset(&mut self, offset: [i32; 3]) { self.m_offset = offset; }

    pub(crate) fn setChance(&mut self, block: Pick, extra: Pick, chance: i32)
    {
        self.m_block = block;
        self.m_extra = extra;
        self.m_chance = chance.max(1);
    }

    pub(crate) fn actual(&self, x: i32, y: i32, z: i32) -> [i32; 3]
    {
        [self.m_origin[0] + x, self.m_origin[1] + y, self.m_origin[2] + z]
    }

    pub(crate) fn inside(&self, at: [i32; 3]) -> bool
    {
        self.m_clip.is_none_or(|clip| clip.contains(at[0], at[1], at[2]))
    }

    // StructureComponent.getBlockAtCurrentPosition, air outside the clip
    pub(crate) fn get(&self, x: i32, y: i32, z: i32) -> i32
    {
        let at: [i32; 3] = self.actual(x, y, z);
        if self.inside(at)
        {
            self.m_world.block(at[0], at[1], at[2])
        }
        else
        {
            0
        }
    }

    pub(crate) fn getOffset(&self, x: i32, y: i32, z: i32) -> i32
    {
        self.get(x + self.m_offset[0], y + self.m_offset[1], z + self.m_offset[2])
    }

    pub(crate) fn put(&self, x: i32, y: i32, z: i32, pick: Pick)
    {
        let at: [i32; 3] = self.actual(x, y, z);
        if self.inside(at)
        {
            self.m_world.setWithMeta(at[0], at[1], at[2], pick.0, pick.1);
        }
    }

    pub(crate) fn putOffset(&self, x: i32, y: i32, z: i32, pick: Pick)
    {
        self.put(x + self.m_offset[0], y + self.m_offset[1], z + self.m_offset[2], pick);
    }

    // AetherStructure.setBlock: the current block, or the extra one on a roll of the chance
    pub(crate) fn place(&self, x: i32, y: i32, z: i32)
    {
        let pick: Pick = if self.m_chance != 0 && self.m_random.nextIntBounded(self.m_chance) == 0
        {
            self.m_extra
        }
        else
        {
            self.m_block
        };
        self.put(x, y, z, pick);
    }

    pub(crate) fn placeOffset(&self, x: i32, y: i32, z: i32)
    {
        self.place(x + self.m_offset[0], y + self.m_offset[1], z + self.m_offset[2]);
    }

    fn allowed(&self, block: i32) -> bool { (self.m_replaceAir || block != 0) && (self.m_replaceSolid || block == 0) }

    // every line, plane and box fill walks x, then y, then z
    pub(crate) fn fill(&self, from: [i32; 3], size: [i32; 3])
    {
        for x in from[0]..from[0] + size[0]
        {
            for y in from[1]..from[1] + size[1]
            {
                for z in from[2]..from[2] + size[2]
                {
                    if !self.allowed(self.getOffset(x, y, z))
                    {
                        continue;
                    }

                    self.placeOffset(x, y, z);

                    // AetherDungeon's fills stop after their first block whenever no chance is set
                    if self.m_clip.is_none() && self.m_chance == 0
                    {
                        return;
                    }
                }
            }
        }
    }

    pub(crate) fn lineX(&self, x: i32, y: i32, z: i32, length: i32) { self.fill([x, y, z], [length, 1, 1]); }

    pub(crate) fn lineY(&self, x: i32, y: i32, z: i32, length: i32) { self.fill([x, y, z], [1, length, 1]); }

    pub(crate) fn lineZ(&self, x: i32, y: i32, z: i32, length: i32) { self.fill([x, y, z], [1, 1, length]); }

    pub(crate) fn planeX(&self, x: i32, y: i32, z: i32, height: i32, depth: i32)
    {
        self.fill([x, y, z], [1, height, depth]);
    }

    pub(crate) fn planeY(&self, x: i32, y: i32, z: i32, width: i32, depth: i32)
    {
        self.fill([x, y, z], [width, 1, depth]);
    }

    pub(crate) fn planeZ(&self, x: i32, y: i32, z: i32, width: i32, height: i32)
    {
        self.fill([x, y, z], [width, height, 1]);
    }

    pub(crate) fn solidBox(&self, from: [i32; 3], size: [i32; 3]) { self.fill(from, size); }

    // addHollowBox and addSquareTube clear the box with air at the current chance first, which leaves the chance at least one
    fn clear(&mut self, from: [i32; 3], size: [i32; 3]) -> (Pick, Pick)
    {
        let kept: (Pick, Pick) = (self.m_block, self.m_extra);

        self.setChance(AIR, AIR, self.m_chance);
        self.fill(from, size);
        self.setChance(kept.0, kept.1, self.m_chance);
        kept
    }

    fn addRoom(&mut self, from: [i32; 3], size: [i32; 3])
    {
        let min: [i32; 3] = self.actualOffset(from[0], from[1], from[2]);
        let max: [i32; 3] = self.actualOffset(from[0] + size[0] - 1, from[1] + size[1] - 1, from[2] + size[2] - 1);
        self.m_rooms.push(StructureBox::new(min, max));
    }

    pub(crate) fn hollowBox(&mut self, from: [i32; 3], size: [i32; 3])
    {
        self.addRoom(from, size);
        self.clear(from, size);

        let [x, y, z] = from;
        let [w, h, d] = size;
        self.planeY(x, y, z, w, d);
        self.planeY(x, y + h - 1, z, w, d);
        self.planeX(x, y, z, h, d);
        self.planeX(x + w - 1, y, z, h, d);
        self.planeZ(x, y, z, w, h);
        self.planeZ(x, y, z + d - 1, w, h);
    }

    // walls on two sides: 0 floor and ends, 1 sides and ends, 2 floor and sides
    pub(crate) fn squareTube(&mut self, from: [i32; 3], size: [i32; 3], angle: i32)
    {
        self.addRoom(from, size);
        self.clear(from, size);

        let [x, y, z] = from;
        let [w, h, d] = size;

        if angle == 0 || angle == 2
        {
            self.planeY(x, y, z, w, d);
            self.planeY(x, y + h - 1, z, w, d);
        }

        if angle == 1 || angle == 2
        {
            self.planeX(x, y, z, h, d);
            self.planeX(x + w - 1, y, z, h, d);
        }

        if angle == 0 || angle == 1
        {
            self.planeZ(x, y, z, w, h);
            self.planeZ(x, y, z + d - 1, w, h);
        }
    }

    fn every(&self, from: [i32; 3], size: [i32; 3], test: impl Fn(i32) -> bool) -> bool
    {
        (from[0]..from[0] + size[0]).all(|x|
        {
            (from[1]..from[1] + size[1]).all(|y| (from[2]..from[2] + size[2]).all(|z| test(self.getOffset(x, y, z))))
        })
    }

    pub(crate) fn boxSolid(&self, from: [i32; 3], size: [i32; 3]) -> bool { self.every(from, size, |block| block != 0) }

    pub(crate) fn boxEmpty(&self, from: [i32; 3], size: [i32; 3]) -> bool { self.every(from, size, |block| block == 0) }

    pub(crate) fn hasBlock(&self, from: [i32; 3], size: [i32; 3], wanted: i32) -> bool
    {
        !self.every(from, size, |block| block != wanted)
    }

    // the world position of an offset structure position, AetherStructure.getActualX and friends
    pub(crate) fn actualOffset(&self, x: i32, y: i32, z: i32) -> [i32; 3]
    {
        self.actual(x + self.m_offset[0], y + self.m_offset[1], z + self.m_offset[2])
    }
}

// java.util.Collections.shuffle over a list, swapping from the end with nextInt
pub(crate) fn shuffle<T>(random: &Random, list: &mut [T])
{
    for i in (2..=list.len()).rev()
    {
        list.swap(i - 1, random.nextIntBounded(i as i32) as usize);
    }
}

// RandomTracker.testRandom: a draw that repeats the previous one is drawn again and reported as -1
#[derive(Default)]
pub(crate) struct RandomTracker
{
    m_last: Option<i32>,
}

impl RandomTracker
{
    pub(crate) fn test(&mut self, random: &Random, bound: i32) -> i32
    {
        let drawn: i32 = random.nextIntBounded(bound);
        if self.m_last != Some(drawn)
        {
            self.m_last = Some(drawn);
            return drawn;
        }

        self.test(random, bound);
        -1
    }
}
