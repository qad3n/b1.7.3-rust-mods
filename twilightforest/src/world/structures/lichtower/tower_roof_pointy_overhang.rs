// draft from ComponentTFTowerRoofPointyOverhang.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(i: i32, wing: &Piece) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::TowerRoofPointyOverhang);
    init(&this, i, wing);
    Rc::new(this)
}

// ComponentTFTowerRoofPointyOverhang constructor
pub(crate) fn init(this: &Piece, i: i32, wing: &Piece)
{
    tower_roof_pointy::init(this, i, wing);
    this.setMode(wing.mode());
    this.m_size.set(wing.m_size.get() + 2);
    this.m_height.set(this.m_size.get());
    tower_roof::makeOverhangBB(this, wing);
}
