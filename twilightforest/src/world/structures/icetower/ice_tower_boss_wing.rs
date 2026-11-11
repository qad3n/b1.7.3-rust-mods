// draft from ComponentTFIceTowerBossWing.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(index: i32, x: i32, y: i32, z: i32, wingSize: i32, wingHeight: i32, direction: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::IceTowerBossWing);
    init(&this, index, x, y, z, wingSize, wingHeight, direction);
    Rc::new(this)
}

// ComponentTFIceTowerBossWing constructor
pub fn init(this: &Piece, index: i32, x: i32, y: i32, z: i32, wingSize: i32, wingHeight: i32, direction: i32)
{
    ice_tower_wing::init(this, index, x, y, z, wingSize, wingHeight, direction);
    this.m_spawnListIndex.set(-1);
}

// ComponentTFIceTowerBossWing.shouldHaveBase
pub fn shouldHaveBase(_this: &Piece, _random: &Random) -> bool { return false; }

// ComponentTFIceTowerBossWing.placeFloor
pub fn placeFloor5(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox, floorHeight: i32, floor: i32)
{
    for x in 1..(this.m_size.get() - 1)
    {
        for z in 1..(this.m_size.get() - 1)
        {
            let ice: Blk = if random.nextIntBounded(4) == 0
            {
                blk("ice")
            }
            else
            {
                blk("packed_ice")
            };
            let thickness: i32 = ((1 + random.nextIntBounded(2)) + random.nextIntBounded(2)) + random.nextIntBounded(2);
            for y in 0..thickness
            {
                this.placeBlock(world, ice, 0, x, ((floor * floorHeight) + floorHeight) - y, z, sbb);
            }
        }
    }
}

// ComponentTFIceTowerBossWing.decorateFloor
pub fn decorateFloor8(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    _floor: i32,
    bottom: i32,
    _top: i32,
    _ladderUpDir: i32,
    ladderDownDir: i32,
    sbb: StructureBox,
)
{
    for y in 0..3
    {
        let rotation: i32 = (ladderDownDir + y) % 4;
        let rotation2: i32 = ((ladderDownDir + y) + 2) % 4;
        ice_tower_boss_wing::placeIceStairs(this, world, sbb, random, bottom + (y * 3), rotation);
        ice_tower_boss_wing::placeIceStairs(this, world, sbb, random, bottom + (y * 3), rotation2);
    }
}

// ComponentTFIceTowerBossWing.placeIceStairs
pub fn placeIceStairs(this: &Piece, world: &GenWorld, sbb: StructureBox, random: &Random, y: i32, rotation: i32)
{
    this.fillTurned(world, sbb, 8, y + 1, 1, 10, y + 1, 3, blk("packed_ice"), 0, rotation);
    if y > 1
    {
        this.fillRandomlyTurned(
            world,
            sbb,
            random,
            0.5,
            8,
            y + 0,
            1,
            10,
            y + 0,
            3,
            blk("packed_ice"),
            0,
            blk("air"),
            0,
            rotation,
        );
    }
    this.fillTurned(world, sbb, 11, y + 2, 1, 13, y + 2, 3, blk("packed_ice"), 0, rotation);
    this.fillRandomlyTurned(
        world,
        sbb,
        random,
        0.5,
        11,
        y + 1,
        1,
        13,
        y + 1,
        3,
        blk("packed_ice"),
        0,
        blk("air"),
        0,
        rotation,
    );
    this.fillTurned(world, sbb, 11, y + 3, 4, 13, y + 3, 6, blk("packed_ice"), 0, rotation);
    this.fillRandomlyTurned(
        world,
        sbb,
        random,
        0.5,
        11,
        y + 2,
        4,
        13,
        y + 2,
        6,
        blk("packed_ice"),
        0,
        blk("air"),
        0,
        rotation,
    );
}

// ComponentTFIceTowerBossWing.decorateTopFloor
pub fn decorateTopFloor(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    _floor: i32,
    _bottom: i32,
    top: i32,
    _ladderUpDir: i32,
    _ladderDownDir: i32,
    sbb: StructureBox,
)
{
    for x in 1..(this.m_size.get() - 1)
    {
        for z in 1..(this.m_size.get() - 1)
        {
            let ice: Blk = if random.nextIntBounded(10) == 0
            {
                blk("glowstone")
            }
            else
            {
                blk("packed_ice")
            };
            let thickness: i32 = random.nextIntBounded(2) + random.nextIntBounded(2);
            for y in 0..thickness
            {
                this.placeBlock(world, ice, 0, x, (top - 1) - y, z, sbb);
            }
        }
    }
    this.placeTurned(world, blk("tf:bossSpawner"), 5, 7, top - 6, 7, 0, sbb);
    this.fillAir(world, sbb, 5, top - 3, 5, 9, top - 1, 9);
}
