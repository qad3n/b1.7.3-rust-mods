// draft from ComponentTFTowerRoofFence.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, wing: &Piece) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::TowerRoofFence);
    init(&this, i, wing);
    Rc::new(this)
}

// ComponentTFTowerRoofFence constructor
pub fn init(this: &Piece, i: i32, wing: &Piece)
{
    tower_roof::init(this, i, &wing);
    this.setMode(wing.mode());
    this.m_size.set(wing.m_size.get());
    this.m_height.set(0);
    tower_roof::makeCapBB(this, &wing);
}

// ComponentTFTowerRoofFence.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    let y: i32 = this.m_height.get() + 1;
    for x in 0..=(this.m_size.get() - 1)
    {
        for z in 0..=(this.m_size.get() - 1)
        {
            if (((x == 0) || (x == (this.m_size.get() - 1))) || (z == 0)) || (z == (this.m_size.get() - 1))
            {
                this.placeBlock(world, blk("fence"), 0, x, y, z, sbb);
            }
        }
    }
    return true;
}
