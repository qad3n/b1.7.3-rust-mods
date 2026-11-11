// draft from ComponentTFTowerRoofSlabForwards.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(i: i32, wing: &Piece) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::TowerRoofSlabForwards);
    init(&this, i, wing);
    Rc::new(this)
}

// ComponentTFTowerRoofSlabForwards constructor
pub(crate) fn init(this: &Piece, i: i32, wing: &Piece)
{
    tower_roof_slab::init(this, i, wing);
    this.setMode(wing.mode());
    this.m_size.set(wing.m_size.get() + 2);
    this.m_height.set(this.m_size.get() / 2);
    tower_roof::makeAttachedOverhangBB(this, wing);
}

// ComponentTFTowerRoofSlabForwards.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, _random: &Random, sbb: StructureBox) -> bool
{
    for y in 0..=this.m_height.get()
    {
        let min: i32 = 2 * y;
        let max: i32 = (this.m_size.get() - (2 * y)) - 1;
        for x in 0..=(max - 1)
        {
            for z in min..=max
            {
                if ((x != (max - 1)) && (z != min)) && (z != max)
                {
                    this.placeBlock(world, blk("double_wooden_slab"), 2, x, y, z, sbb);
                }
                else
                {
                    this.placeBlock(world, blk("wooden_slab"), 2, x, y, z, sbb);
                }
            }
        }
    }

    true
}
