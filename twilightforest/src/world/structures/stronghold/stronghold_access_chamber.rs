// draft from ComponentTFStrongholdAccessChamber.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, facing: i32, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::StrongholdAccessChamber);
    init(&this, i, facing, x, y, z);
    Rc::new(this)
}

// ComponentTFStrongholdAccessChamber constructor
pub fn init(this: &Piece, i: i32, facing: i32, x: i32, y: i32, z: i32)
{
    stronghold_component::init(this, i, facing, x, y, z);
}

// ComponentTFStrongholdAccessChamber.generateBoundingBox
pub fn generateBoundingBox(this: &Piece, facing: i32, x: i32, y: i32, z: i32) -> Option<StructureBox>
{
    return Some(vanillaBox(x, y, z, -4, 1, 0, 9, 5, 9, facing));
}

// ComponentTFStrongholdAccessChamber.func_74861_a
pub fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    stronghold_component::build(this, &parent, list, random);
    stronghold_component::addNewUpperComponent(this, &parent, list, random, 0, 4, 1, 9);
    stronghold_component::addNewUpperComponent(this, &parent, list, random, 1, -1, 1, 4);
    stronghold_component::addNewUpperComponent(this, &parent, list, random, 2, 4, 1, -1);
    stronghold_component::addNewUpperComponent(this, &parent, list, random, 3, 9, 1, 4);
}

// ComponentTFStrongholdAccessChamber.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    this.fillSelected(world, sbb, 0, 0, 0, 8, 4, 8, true, random, this.m_deco.get().m_randomBlocks);
    stronghold_component::placeSmallDoorwayAt(this, world, random, 0, 4, 1, 8, sbb);
    stronghold_component::placeSmallDoorwayAt(this, world, random, 1, 0, 1, 4, sbb);
    stronghold_component::placeSmallDoorwayAt(this, world, random, 2, 4, 1, 0, sbb);
    stronghold_component::placeSmallDoorwayAt(this, world, random, 3, 8, 1, 4, sbb);
    this.fillMeta(world, sbb, 2, -2, 2, 6, 0, 6, blk("stonebrick"), 1, blk("air"), 0, false);
    this.fillAir(world, sbb, 3, -2, 3, 5, 2, 5);
    this.fillMeta(world, sbb, 2, 0, 3, 2, 0, 6, this.m_deco.get().m_stairID, this.stairMeta(2), blk("air"), 0, false);
    this.fillMeta(world, sbb, 6, 0, 2, 6, 0, 6, this.m_deco.get().m_stairID, this.stairMeta(0), blk("air"), 0, false);
    this.fillMeta(world, sbb, 3, 0, 2, 5, 0, 2, this.m_deco.get().m_stairID, this.stairMeta(3), blk("air"), 0, false);
    this.fillMeta(world, sbb, 3, 0, 6, 5, 0, 6, this.m_deco.get().m_stairID, this.stairMeta(1), blk("air"), 0, false);
    this.placeBlock(world, this.m_deco.get().m_pillarID, this.m_deco.get().m_pillarMeta, 2, 0, 2, sbb);
    this.placeBlock(world, blk("tf:trophyPedestal"), 15, 2, 1, 2, sbb);
    this.fillMeta(world, sbb, 2, -1, 2, 6, -1, 6, blk("tf:shield"), 15, blk("air"), 0, false);
    return true;
}

// ComponentTFStrongholdAccessChamber.isComponentProtected
pub fn isComponentProtected(this: &Piece) -> bool { return false; }
