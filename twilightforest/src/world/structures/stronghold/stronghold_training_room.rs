// draft from ComponentTFStrongholdTrainingRoom.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(i: i32, facing: i32, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::StrongholdTrainingRoom);
    init(&this, i, facing, x, y, z);
    Rc::new(this)
}

// ComponentTFStrongholdTrainingRoom constructor
pub(crate) fn init(this: &Piece, i: i32, facing: i32, x: i32, y: i32, z: i32)
{
    stronghold_component::init(this, i, facing, x, y, z);
}

// ComponentTFStrongholdTrainingRoom.generateBoundingBox
pub(crate) fn generateBoundingBox(_this: &Piece, facing: i32, x: i32, y: i32, z: i32) -> Option<StructureBox>
{
    Some(boxFor(x, y, z, -13, -1, 0, 18, 7, 18, facing))
}

// ComponentTFStrongholdTrainingRoom.func_74861_a
pub(crate) fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    stronghold_component::build(this, parent, list, random);
    stronghold_component::addDoor(this, 13, 1, 0);
    stronghold_component::addNewComponent(this, parent, list, random, 0, 4, 1, 18);
}

// ComponentTFStrongholdTrainingRoom.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
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
    this.fillRandomly(world, sbb, random, 0.7, 4, 0, 4, 8, 0, 8, blk("sand"), blk("sand"), false);
    this.fillRandomly(world, sbb, random, 0.7, 9, 0, 4, 13, 0, 8, blk("sand"), blk("sand"), false);
    this.fillRandomly(world, sbb, random, 0.7, 9, 0, 9, 13, 0, 13, blk("sand"), blk("sand"), false);
    stronghold_training_room::placeTrainingDummy(this, world, sbb, 0);
    stronghold_training_room::placeTrainingDummy(this, world, sbb, 1);
    stronghold_training_room::placeTrainingDummy(this, world, sbb, 2);
    this.fill(world, sbb, 5, 0, 10, 7, 0, 12, blk("cobblestone"), blk("cobblestone"), false);
    this.placeBlock(world, this.m_deco.get().m_pillarID, this.m_deco.get().m_pillarMeta, 5, 1, 12, sbb);
    this.placeBlock(world, this.m_deco.get().m_pillarID, this.m_deco.get().m_pillarMeta, 5, 2, 12, sbb);
    this.placeBlock(world, this.m_deco.get().m_pillarID, this.m_deco.get().m_pillarMeta, 6, 1, 12, sbb);
    this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(2), 6, 2, 12, sbb);
    this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(2), 7, 1, 12, sbb);
    this.placeBlock(world, this.m_deco.get().m_pillarID, this.m_deco.get().m_pillarMeta, 5, 1, 11, sbb);
    this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(1), 5, 2, 11, sbb);
    this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(1), 5, 1, 10, sbb);
    this.placeBlock(world, blk("anvil"), 0, 6, 1, 11, sbb);
    stronghold_component::placeDoors(this, world, random, sbb);
    true
}

// ComponentTFStrongholdTrainingRoom.placeTrainingDummy
pub(crate) fn placeTrainingDummy(this: &Piece, world: &GenWorld, sbb: StructureBox, rotation: i32)
{
    this.fillTurned(world, sbb, 5, 0, 5, 7, 0, 7, blk("sand"), 0, rotation);
    this.placeTurned(world, this.m_deco.get().m_fenceID, this.m_deco.get().m_fenceMeta, 6, 1, 6, rotation, sbb);
    this.placeTurned(world, blk("planks"), 2, 6, 2, 6, rotation, sbb);
    this.placeTurned(world, blk("fence"), 0, 5, 2, 6, rotation, sbb);
    this.placeTurned(world, blk("fence"), 0, 7, 2, 6, rotation, sbb);
    this.placeTurned(world, blk("pumpkin"), this.stairMeta(0 + rotation), 6, 3, 6, rotation, sbb);
}
