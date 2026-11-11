// draft from ComponentTFStrongholdUpperLeftTurn.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, facing: i32, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::StrongholdUpperLeftTurn);
    init(&this, i, facing, x, y, z);
    Rc::new(this)
}

// ComponentTFStrongholdUpperLeftTurn constructor
pub fn init(this: &Piece, i: i32, facing: i32, x: i32, y: i32, z: i32)
{
    stronghold_component::init(this, i, facing, x, y, z);
}

// ComponentTFStrongholdUpperLeftTurn.generateBoundingBox
pub fn generateBoundingBox(_this: &Piece, facing: i32, x: i32, y: i32, z: i32) -> Option<StructureBox>
{
    return Some(vanillaBox(x, y, z, -2, -1, 0, 5, 5, 5, facing));
}

// ComponentTFStrongholdUpperLeftTurn.func_74861_a
pub fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    stronghold_component::build(this, &parent, list, random);
    stronghold_component::addNewUpperComponent(this, &parent, list, random, 3, 5, 1, 2);
}

// ComponentTFStrongholdUpperLeftTurn.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    if this.liquidIn(world, sbb)
    {
        return false;
    }
    stronghold_component::placeUpperStrongholdWalls(
        this,
        world,
        sbb,
        0,
        0,
        0,
        4,
        4,
        4,
        random,
        this.m_deco.get().m_randomBlocks,
    );
    stronghold_component::placeSmallDoorwayAt(this, world, random, 2, 2, 1, 0, sbb);
    stronghold_component::placeSmallDoorwayAt(this, world, random, 3, 4, 1, 2, sbb);
    return true;
}

// ComponentTFStrongholdUpperLeftTurn.isComponentProtected
pub fn isComponentProtected(_this: &Piece) -> bool { return false; }
