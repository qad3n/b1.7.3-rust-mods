// draft from ComponentTFStrongholdSmallHallway.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(i: i32, facing: i32, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::StrongholdSmallHallway);
    init(&this, i, facing, x, y, z);
    Rc::new(this)
}

// ComponentTFStrongholdSmallHallway constructor
pub(crate) fn init(this: &Piece, i: i32, facing: i32, x: i32, y: i32, z: i32)
{
    stronghold_component::init(this, i, facing, x, y, z);
}

// ComponentTFStrongholdSmallHallway.generateBoundingBox
pub(crate) fn generateBoundingBox(_this: &Piece, facing: i32, x: i32, y: i32, z: i32) -> Option<StructureBox>
{
    Some(boxFor(x, y, z, -4, -1, 0, 9, 7, 18, facing))
}

// ComponentTFStrongholdSmallHallway.func_74861_a
pub(crate) fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    stronghold_component::build(this, parent, list, random);
    stronghold_component::addDoor(this, 4, 1, 0);
    stronghold_component::addNewComponent(this, parent, list, random, 0, 4, 1, 18);
}

// ComponentTFStrongholdSmallHallway.func_74875_a
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
        17,
        random,
        this.m_deco.get().m_randomBlocks,
    );
    stronghold_component::placeWallStatue(this, world, 1, 1, 9, 1, sbb);
    stronghold_component::placeWallStatue(this, world, 7, 1, 9, 3, sbb);
    stronghold_component::placeDoors(this, world, random, sbb);
    true
}
