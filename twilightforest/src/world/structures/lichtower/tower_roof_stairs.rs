// draft from ComponentTFTowerRoofStairs.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, wing: &Piece) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::TowerRoofStairs);
    init(&this, i, wing);
    Rc::new(this)
}

// ComponentTFTowerRoofStairs constructor
pub fn init(this: &Piece, i: i32, wing: &Piece)
{
    tower_roof::init(this, i, &wing);
    this.setMode(0);
    this.m_size.set(wing.m_size.get());
    this.m_height.set(this.m_size.get() / 2);
    tower_roof::makeCapBB(this, &wing);
}

// ComponentTFTowerRoofStairs.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, _random: &Random, sbb: StructureBox) -> bool
{
    for y in 0..=this.m_height.get()
    {
        let min: i32 = y;
        let max: i32 = (this.m_size.get() - y) - 1;
        for x in min..=max
        {
            for z in min..=max
            {
                if x == min
                {
                    if (z != min) && (z != max)
                    {
                        this.placeBlock(world, blk("birch_stairs"), 0, x, y, z, sbb);
                    }
                    else
                    {
                        this.placeBlock(world, blk("wooden_slab"), 2, x, y, z, sbb);
                    }
                }
                else if x == max
                {
                    if (z != min) && (z != max)
                    {
                        this.placeBlock(world, blk("birch_stairs"), 1, x, y, z, sbb);
                    }
                    else
                    {
                        this.placeBlock(world, blk("wooden_slab"), 2, x, y, z, sbb);
                    }
                }
                else if z == max
                {
                    this.placeBlock(world, blk("birch_stairs"), 3, x, y, z, sbb);
                }
                else if z == min
                {
                    this.placeBlock(world, blk("birch_stairs"), 2, x, y, z, sbb);
                }
                else
                {
                    this.placeBlock(world, blk("planks"), 2, x, y, z, sbb);
                }
            }
        }
    }
    return true;
}
