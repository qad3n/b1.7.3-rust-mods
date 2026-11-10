// draft from ComponentTFTowerRoofSlab.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, wing: &Piece) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::TowerRoofSlab);
    init(&this, i, wing);
    Rc::new(this)
}

// ComponentTFTowerRoofSlab constructor
pub fn init(this: &Piece, i: i32, wing: &Piece)
{
    tower_roof::init(this, i, &wing);
    this.setMode(wing.mode());
    this.m_size.set(wing.m_size.get());
    this.m_height.set(this.m_size.get() / 2);
    tower_roof::makeCapBB(this, &wing);
}

// ComponentTFTowerRoofSlab.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    let slabMeta: i32 = 2;
    return tower_roof_slab::makePyramidCap(this, world, slabMeta, sbb);
}

// ComponentTFTowerRoofSlab.makePyramidCap
pub fn makePyramidCap(this: &Piece, world: &GenWorld, slabMeta: i32, sbb: StructureBox) -> bool
{
    for y in 0..=this.m_height.get()
    {
        let min: i32 = 2 * y;
        let max: i32 = (this.m_size.get() - (2 * y)) - 1;
        for x in min..=max
        {
            for z in min..=max
            {
                if (((x != min) && (x != max)) && (z != min)) && (z != max)
                {
                    this.placeBlock(world, blk("planks"), slabMeta, x, y, z, sbb);
                }
                else
                {
                    this.placeBlock(world, blk("wooden_slab"), slabMeta, x, y, z, sbb);
                }
            }
        }
    }
    return true;
}

// ComponentTFTowerRoofSlab.makeConnectedCap
pub fn makeConnectedCap(this: &Piece, world: &GenWorld, slabMeta: i32, sbb: StructureBox) -> bool
{
    for y in 0..this.m_height.get()
    {
        let min: i32 = 2 * y;
        let max: i32 = (this.m_size.get() - (2 * y)) - 1;
        for x in 0..=max
        {
            for z in min..=max
            {
                if ((x != max) && (z != min)) && (z != max)
                {
                    this.placeBlock(world, blk("planks"), slabMeta, x, y, z, sbb);
                }
                else
                {
                    this.placeBlock(world, blk("wooden_slab"), slabMeta, x, y, z, sbb);
                }
            }
        }
    }
    return true;
}
