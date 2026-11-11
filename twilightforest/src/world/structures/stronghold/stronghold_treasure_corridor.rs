// draft from ComponentTFStrongholdTreasureCorridor.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, facing: i32, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::StrongholdTreasureCorridor);
    init(&this, i, facing, x, y, z);
    Rc::new(this)
}

// ComponentTFStrongholdTreasureCorridor constructor
pub fn init(this: &Piece, i: i32, facing: i32, x: i32, y: i32, z: i32)
{
    stronghold_component::init(this, i, facing, x, y, z);
}

// ComponentTFStrongholdTreasureCorridor.generateBoundingBox
pub fn generateBoundingBox(_this: &Piece, facing: i32, x: i32, y: i32, z: i32) -> Option<StructureBox>
{
    return Some(boxFor(x, y, z, -4, -1, 0, 9, 7, 27, facing));
}

// ComponentTFStrongholdTreasureCorridor.func_74861_a
pub fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    stronghold_component::build(this, &parent, list, random);
    stronghold_component::addDoor(this, 4, 1, 0);
    stronghold_component::addNewComponent(this, &parent, list, random, 0, 4, 1, 27);
}

// ComponentTFStrongholdTreasureCorridor.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
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
        26,
        random,
        this.m_deco.get().m_randomBlocks,
    );
    stronghold_component::placeWallStatue(this, world, 1, 1, 9, 1, sbb);
    stronghold_component::placeWallStatue(this, world, 1, 1, 17, 1, sbb);
    stronghold_component::placeWallStatue(this, world, 7, 1, 9, 3, sbb);
    stronghold_component::placeWallStatue(this, world, 7, 1, 17, 3, sbb);
    let rotation: i32 = if ((this.m_box.get().m_min[0] ^ this.m_box.get().m_min[2]) % 2) == 0
    {
        0
    }
    else
    {
        2
    };
    this.placeTreasureRotated(world, 8, 2, 13, rotation, Table::StrongholdCache, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(3 + rotation) + 4, 8, 3, 12, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(0 + rotation) + 4, 8, 3, 13, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(1 + rotation) + 4, 8, 3, 14, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_fenceID, this.m_deco.get().m_fenceMeta, 8, 2, 12, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_fenceID, this.m_deco.get().m_fenceMeta, 8, 2, 14, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(1 + rotation), 7, 1, 12, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(0 + rotation), 7, 1, 13, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(3 + rotation), 7, 1, 14, rotation, sbb);
    stronghold_component::placeDoors(this, world, random, sbb);
    return true;
}
