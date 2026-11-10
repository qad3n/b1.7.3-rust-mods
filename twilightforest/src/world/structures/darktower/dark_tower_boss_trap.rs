// draft from ComponentTFDarkTowerBossTrap.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, x: i32, y: i32, z: i32, pSize: i32, pHeight: i32, direction: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::DarkTowerBossTrap);
    init(&this, i, x, y, z, pSize, pHeight, direction);
    Rc::new(this)
}

// ComponentTFDarkTowerBossTrap constructor
pub fn init(this: &Piece, i: i32, x: i32, y: i32, z: i32, pSize: i32, pHeight: i32, direction: i32)
{
    dark_tower_wing::init(this, i, x, y, z, pSize, pHeight, direction);
    this.m_spawnListIndex.set(-1);
}

// ComponentTFDarkTowerBossTrap.func_74861_a
pub fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    this.m_deco.set((parent).m_deco.get());
    this.addOpening(0, 1, this.m_size.get() / 2, 2);
    this.makeABeard(&parent, list, random);
    for i in 0..4
    {
        if (i != 2) && (!random.nextBoolean())
        {
            let mut dest: Vec<i32> = this.getValidOpening(random, i);
            dest[1] = 1;
            dark_tower_wing::makeTowerBalcony(this, list, random, this.depth(), dest[0], dest[1], dest[2], i);
        }
    }
}

// ComponentTFDarkTowerBossTrap.makeARoof
pub fn makeARoof(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random) {}

// ComponentTFDarkTowerBossTrap.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    let decoRNG: Random = Random::withSeed(
        (world.seed() + ((this.m_box.get().m_min[0] * 321534781) as i64))
            ^ ((this.m_box.get().m_min[2] * 756839) as i64),
    );
    dark_tower_wing::makeEncasedWalls(
        this,
        world,
        random,
        sbb,
        0,
        0,
        0,
        this.m_size.get() - 1,
        this.m_height.get() - 1,
        this.m_size.get() - 1,
    );
    this.fillAir(world, sbb, 1, 1, 1, this.m_size.get() - 2, this.m_height.get() - 2, this.m_size.get() - 2);
    this.makeOpenings(world, sbb);
    dark_tower_boss_trap::addBossTrapFloors(this, world, &decoRNG, sbb, 4, this.m_height.get() - 1);
    dark_tower_wing::destroyTower7(this, world, &decoRNG, 5, this.m_height.get() + 2, 5, 4, sbb);
    dark_tower_wing::destroyTower7(this, world, &decoRNG, 0, this.m_height.get(), 0, 3, sbb);
    dark_tower_wing::destroyTower7(this, world, &decoRNG, 0, this.m_height.get(), 8, 4, sbb);
    dark_tower_wing::destroyTower7(this, world, &decoRNG, 5, 6, 5, 2, sbb);
    this.fillMeta(
        world,
        sbb,
        1,
        0,
        1,
        this.m_size.get() / 2,
        0,
        this.m_size.get() - 2,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockMeta,
        blk("air"),
        0,
        false,
    );
    this.fillMeta(
        world,
        sbb,
        1,
        1,
        1,
        this.m_size.get() / 2,
        1,
        this.m_size.get() - 2,
        blk("air"),
        0,
        blk("air"),
        0,
        false,
    );
    this.placeBlock(world, blk("tf:towerDevice"), 10, 5, 1, 5, sbb);
    this.placeBlock(world, blk("redstone_wire"), 0, 5, 1, 6, sbb);
    this.placeBlock(world, blk("redstone_wire"), 0, 5, 1, 7, sbb);
    this.placeBlock(world, blk("redstone_wire"), 0, 5, 1, 8, sbb);
    this.placeBlock(world, blk("redstone_wire"), 0, 4, 1, 8, sbb);
    this.placeBlock(world, blk("redstone_wire"), 0, 3, 1, 8, sbb);
    this.placeBlock(world, blk("wooden_pressure_plate"), 0, 2, 1, 8, sbb);
    return true;
}

// ComponentTFDarkTowerBossTrap.addBossTrapFloors
pub fn addBossTrapFloors(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox, bottom: i32, top: i32)
{
    dark_tower_wing::makeFullFloor(this, world, sbb, 3, 4, 4);
    dark_tower_wing::addStairsDown6(this, world, sbb, 3, 4, this.m_size.get() - 2, 4);
    dark_tower_wing::addStairsDown6(this, world, sbb, 3, 4, this.m_size.get() - 3, 4);
    dark_tower_wing::addStairsDown6(this, world, sbb, 1, this.m_height.get() - 1, this.m_size.get() - 2, 4);
    dark_tower_wing::addStairsDown6(this, world, sbb, 1, this.m_height.get() - 1, this.m_size.get() - 3, 4);
}
