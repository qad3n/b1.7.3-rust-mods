// draft from ComponentTFMazeDeadEndFountain.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, x: i32, y: i32, z: i32, rotation: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::MazeDeadEndFountain);
    init(&this, i, x, y, z, rotation);
    Rc::new(this)
}

// ComponentTFMazeDeadEndFountain constructor
pub fn init(this: &Piece, i: i32, x: i32, y: i32, z: i32, rotation: i32)
{
    maze_dead_end::init(this, i, x, y, z, rotation);
}

// ComponentTFMazeDeadEndFountain.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    maze_dead_end::add(this, world, random, sbb);
    this.fillMeta(world, sbb, 1, 1, 4, 4, 4, 4, blk("tf:mazestone"), 1, blk("air"), 0, false);
    this.placeBlock(world, blk("flowing_water"), 0, 2, 3, 4, sbb);
    this.placeBlock(world, blk("flowing_water"), 0, 3, 3, 4, sbb);
    this.placeBlock(world, blk("air"), 0, 2, 0, 3, sbb);
    this.placeBlock(world, blk("air"), 0, 3, 0, 3, sbb);
    return true;
}
