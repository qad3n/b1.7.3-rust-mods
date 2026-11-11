// draft from ComponentTFStrongholdDeadEnd.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(i: i32, facing: i32, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::StrongholdDeadEnd);
    init(&this, i, facing, x, y, z);
    Rc::new(this)
}

// ComponentTFStrongholdDeadEnd constructor
pub(crate) fn init(this: &Piece, i: i32, facing: i32, x: i32, y: i32, z: i32)
{
    stronghold_component::init(this, i, facing, x, y, z);
}

// ComponentTFStrongholdDeadEnd.generateBoundingBox
pub(crate) fn generateBoundingBox(_this: &Piece, facing: i32, x: i32, y: i32, z: i32) -> Option<StructureBox>
{
    Some(boxFor(x, y, z, -4, -1, 0, 9, 6, 9, facing))
}

// ComponentTFStrongholdDeadEnd.func_74861_a
pub(crate) fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    stronghold_component::build(this, parent, list, random);
    stronghold_component::addDoor(this, 4, 1, 0);
    this.m_chestTrapped.set(random.nextIntBounded(3) == 0);
}

// ComponentTFStrongholdDeadEnd.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    stronghold_component::placeStrongholdWalls(
        this,
        world,
        sbb,
        0,
        0,
        0,
        8,
        6,
        8,
        random,
        this.m_deco.get().m_randomBlocks,
    );
    stronghold_component::placeWallStatue(this, world, 1, 1, 4, 1, sbb);
    stronghold_component::placeWallStatue(this, world, 7, 1, 4, 3, sbb);
    stronghold_component::placeWallStatue(this, world, 4, 1, 7, 0, sbb);
    stronghold_component::placeDoors(this, world, random, sbb);
    this.placeTreasureTrapped(world, random, 4, 1, 3, Table::StrongholdCache, this.m_chestTrapped.get(), sbb);

    if this.m_chestTrapped.get()
    {
        this.placeBlock(world, blk("tnt"), 0, 4, 0, 3, sbb);
    }

    for z in 2..5
    {
        this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(0), 3, 1, z, sbb);
        this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(2), 5, 1, z, sbb);
    }

    this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(1), 4, 1, 2, sbb);
    this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(3), 4, 1, 4, sbb);
    this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(1), 4, 2, 3, sbb);
    true
}
