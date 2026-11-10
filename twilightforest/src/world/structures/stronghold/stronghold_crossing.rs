// draft from ComponentTFStrongholdCrossing.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, facing: i32, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::StrongholdCrossing);
    init(&this, i, facing, x, y, z);
    Rc::new(this)
}

// ComponentTFStrongholdCrossing constructor
pub fn init(this: &Piece, i: i32, facing: i32, x: i32, y: i32, z: i32)
{
    stronghold_component::init(this, i, facing, x, y, z);
}

// ComponentTFStrongholdCrossing.generateBoundingBox
pub fn generateBoundingBox(this: &Piece, facing: i32, x: i32, y: i32, z: i32) -> Option<StructureBox>
{
    return Some(boxFor(x, y, z, -13, -1, 0, 18, 7, 18, facing));
}

// ComponentTFStrongholdCrossing.func_74861_a
pub fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    stronghold_component::build(this, &parent, list, random);
    stronghold_component::addDoor(this, 13, 1, 0);
    stronghold_component::addNewComponent(this, &parent, list, random, 0, 4, 1, 18);
    stronghold_component::addNewComponent(this, &parent, list, random, 1, -1, 1, 13);
    stronghold_component::addNewComponent(this, &parent, list, random, 3, 18, 1, 4);
}

// ComponentTFStrongholdCrossing.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    stronghold_component::placeStrongholdWalls(
        this,
        world,
        sbb,
        0,
        0,
        0,
        17,
        6,
        17,
        random,
        this.m_deco.get().m_randomBlocks,
    );
    stronghold_component::placeCornerStatue(this, world, 2, 1, 2, 0, sbb);
    stronghold_component::placeCornerStatue(this, world, 15, 1, 15, 3, sbb);
    this.fillTurned(world, sbb, 8, 1, 8, 9, 5, 9, this.m_deco.get().m_pillarID, this.m_deco.get().m_pillarMeta, 0);
    stronghold_component::placeWallStatue(this, world, 8, 1, 7, 0, sbb);
    stronghold_component::placeWallStatue(this, world, 7, 1, 9, 3, sbb);
    stronghold_component::placeWallStatue(this, world, 9, 1, 10, 2, sbb);
    stronghold_component::placeWallStatue(this, world, 10, 1, 8, 1, sbb);
    stronghold_crossing::placeTableAndChairs(this, world, sbb, 0);
    stronghold_crossing::placeTableAndChairs(this, world, sbb, 1);
    stronghold_crossing::placeTableAndChairs(this, world, sbb, 2);
    stronghold_crossing::placeTableAndChairs(this, world, sbb, 3);
    stronghold_component::placeDoors(this, world, random, sbb);
    return true;
}

// ComponentTFStrongholdCrossing.placeTableAndChairs
pub fn placeTableAndChairs(this: &Piece, world: &GenWorld, sbb: StructureBox, rotation: i32)
{
    this.placeTurned(world, blk("oak_stairs"), this.stairMeta(0 + rotation) + 4, 5, 1, 3, rotation, sbb);
    this.placeTurned(world, blk("oak_stairs"), this.stairMeta(3 + rotation) + 4, 5, 1, 4, rotation, sbb);
    this.placeTurned(world, blk("oak_stairs"), this.stairMeta(1 + rotation) + 4, 6, 1, 3, rotation, sbb);
    this.placeTurned(world, blk("oak_stairs"), this.stairMeta(2 + rotation) + 4, 6, 1, 4, rotation, sbb);
    this.placeTurned(world, blk("spruce_stairs"), this.stairMeta(3 + rotation), 5, 1, 2, rotation, sbb);
    this.placeTurned(world, blk("spruce_stairs"), this.stairMeta(0 + rotation), 7, 1, 3, rotation, sbb);
    this.placeTurned(world, blk("spruce_stairs"), this.stairMeta(1 + rotation), 6, 1, 5, rotation, sbb);
    this.placeTurned(world, blk("spruce_stairs"), this.stairMeta(2 + rotation), 4, 1, 4, rotation, sbb);
}
