// draft from ComponentTFTowerRoofGableForwards.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, wing: &Piece) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::TowerRoofGableForwards);
    init(&this, i, wing);
    Rc::new(this)
}

// ComponentTFTowerRoofGableForwards constructor
pub fn init(this: &Piece, i: i32, wing: &Piece)
{
    tower_roof::init(this, i, &wing);
    this.setMode(wing.mode());
    this.m_size.set(wing.m_size.get() + 2);
    this.m_height.set(this.m_size.get());
    tower_roof::makeAttachedOverhangBB(this, &wing);
}

// ComponentTFTowerRoofGableForwards.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    let slabMeta: i32 = 2;
    let slopeChange: i32 = this.slopeChangeForSize(this.m_size.get());
    for y in 0..=this.m_height.get()
    {
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
        for x in 0..=(this.m_size.get() - 2)
        {
            for z in min..=max
            {
                if (z == min) || (z == max)
                {
                    this.placeBlock(world, blk("planks"), slabMeta, x, y, z, sbb);
                }
                else if x < (this.m_size.get() - 2)
                {
                    this.placeBlock(world, blk("planks"), slabMeta, x, y, z, sbb);
                }
            }
        }
    }
    let top: i32 = (this.m_size.get() + 1) - slopeChange;
    let zMid: i32 = this.m_size.get() / 2;
    this.placeBlock(world, blk("wooden_slab"), slabMeta | 10, this.m_size.get() - 1, top - 1, zMid, sbb);
    this.placeBlock(world, blk("wooden_slab"), slabMeta, 0, top, zMid, sbb);
    this.placeBlock(world, blk("wooden_slab"), slabMeta, this.m_size.get() - 3, top, zMid, sbb);
    this.placeBlock(world, blk("planks"), slabMeta, this.m_size.get() - 2, top, zMid, sbb);
    this.placeBlock(world, blk("planks"), slabMeta, this.m_size.get() - 1, top, zMid, sbb);
    this.placeBlock(world, blk("planks"), slabMeta, this.m_size.get() - 1, top + 1, zMid, sbb);
    return true;
}

// ComponentTFTowerRoofGableForwards.slopeChangeForSize
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
