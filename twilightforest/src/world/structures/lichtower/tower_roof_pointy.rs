// draft from ComponentTFTowerRoofPointy.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, wing: &Piece) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::TowerRoofPointy);
    init(&this, i, wing);
    Rc::new(this)
}

// ComponentTFTowerRoofPointy constructor
pub fn init(this: &Piece, i: i32, wing: &Piece)
{
    tower_roof::init(this, i, &wing);
    this.setMode(wing.mode());
    this.m_size.set(wing.m_size.get());
    this.m_height.set(this.m_size.get());
    tower_roof::makeCapBB(this, &wing);
}

// ComponentTFTowerRoofPointy.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    let slabMeta: i32 = 2;
    for y in 0..=this.m_height.get()
    {
        let slopeChange: i32 = this.slopeChangeForSize(this.m_size.get());
        let mut min: i32;
        let mut max: i32;
        if y < slopeChange
        {
            min = y;
            max = (this.m_size.get() - y) - 1;
        }
        else
        {
            min = (y + slopeChange) / 2;
            max = (this.m_size.get() - ((y + slopeChange) / 2)) - 1;
        }
        let mid: i32 = min + ((max - min) / 2);
        for x in min..=max
        {
            for z in min..=max
            {
                this.placeBlock(world, blk("planks"), slabMeta, x, y, z, sbb);
                if ((x == min) && ((z == min) || (z == max))) || ((x == max) && ((z == min) || (z == max)))
                {
                    this.placeBlock(world, blk("wooden_slab"), slabMeta, x, y + 1, z, sbb);
                }
                if (((((x == min) || (x == max)) && (z == mid)) && ((x % 2) == 0))
                    || ((((z == min) || (z == max)) && (x == mid)) && ((z % 2) == 0)))
                    && (mid != (min + 1))
                {
                    this.placeBlock(world, blk("wooden_slab"), slabMeta, x, y + 1, z, sbb);
                }
            }
        }
    }
    return true;
}

// ComponentTFTowerRoofPointy.slopeChangeForSize
pub fn slopeChangeForSize(this: &Piece, pSize: i32) -> i32
{
    if this.m_size.get() > 10
    {
        return 3;
    }
    else
    {
        return if this.m_size.get() > 6 { 2 } else { 1 };
    }
}
