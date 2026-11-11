// draft from ComponentTFStrongholdLeftTurn.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(i: i32, facing: i32, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::StrongholdLeftTurn);
    init(&this, i, facing, x, y, z);
    Rc::new(this)
}

// ComponentTFStrongholdLeftTurn constructor
pub(crate) fn init(this: &Piece, i: i32, facing: i32, x: i32, y: i32, z: i32)
{
    stronghold_component::init(this, i, facing, x, y, z);
}

// ComponentTFStrongholdLeftTurn.generateBoundingBox
pub(crate) fn generateBoundingBox(_this: &Piece, facing: i32, x: i32, y: i32, z: i32) -> Option<StructureBox>
{
    Some(boxFor(x, y, z, -4, -1, 0, 9, 7, 9, facing))
}

// ComponentTFStrongholdLeftTurn.func_74861_a
pub(crate) fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    stronghold_component::build(this, parent, list, random);
    stronghold_component::addDoor(this, 4, 1, 0);
    stronghold_component::addNewComponent(this, parent, list, random, 3, 9, 1, 4);
}

// ComponentTFStrongholdLeftTurn.func_74875_a
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
    this.fillAir(world, sbb, 1, 1, 1, 7, 5, 7);
    stronghold_component::placeCornerStatue(this, world, 2, 1, 6, 1, sbb);
    stronghold_component::placeDoors(this, world, random, sbb);
    true
}
