// draft from ComponentTFTowerRoofAttachedSlab.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, wing: &Piece) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::TowerRoofAttachedSlab);
    init(&this, i, wing);
    Rc::new(this)
}

// ComponentTFTowerRoofAttachedSlab constructor
pub fn init(this: &Piece, i: i32, wing: &Piece) { tower_roof_slab::init(this, i, &wing); }

// ComponentTFTowerRoofAttachedSlab.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    let slabMeta: i32 = 2;
    return tower_roof_slab::makeConnectedCap(this, world, slabMeta, sbb);
}
