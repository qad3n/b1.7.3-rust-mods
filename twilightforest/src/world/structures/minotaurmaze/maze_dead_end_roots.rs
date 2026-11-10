// draft from ComponentTFMazeDeadEndRoots.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, x: i32, y: i32, z: i32, rotation: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::MazeDeadEndRoots);
    init(&this, i, x, y, z, rotation);
    Rc::new(this)
}

// ComponentTFMazeDeadEndRoots constructor
pub fn init(this: &Piece, i: i32, x: i32, y: i32, z: i32, rotation: i32)
{
    maze_dead_end::init(this, i, x, y, z, rotation);
}

// ComponentTFMazeDeadEndRoots.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    for x in 1..5
    {
        for z in 0..5
        {
            if random.nextIntBounded(z + 2) > 0
            {
                let length: i32 = random.nextIntBounded(6);
                this.placeBlock(world, blk("dirt"), 0, x, 6, z, sbb);
                for y in 6 - length..6
                {
                    this.placeBlock(world, blk("tf:plant"), 14, x, y, z, sbb);
                }
                if random.nextIntBounded(z + 1) > 1
                {
                    this.placeBlock(world, blk("gravel"), 0, x, 1, z, sbb);
                    if random.nextIntBounded(z + 1) > 1
                    {
                        this.placeBlock(world, blk("gravel"), 0, x, 2, z, sbb);
                    }
                }
            }
        }
    }
    return true;
}
