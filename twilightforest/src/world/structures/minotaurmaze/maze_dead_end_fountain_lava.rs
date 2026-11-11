// draft from ComponentTFMazeDeadEndFountainLava.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(i: i32, x: i32, y: i32, z: i32, rotation: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::MazeDeadEndFountainLava);
    init(&this, i, x, y, z, rotation);
    Rc::new(this)
}

// ComponentTFMazeDeadEndFountainLava constructor
pub(crate) fn init(this: &Piece, i: i32, x: i32, y: i32, z: i32, rotation: i32)
{
    maze_dead_end_fountain::init(this, i, x, y, z, rotation);
}

// ComponentTFMazeDeadEndFountainLava.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    maze_dead_end_fountain::add(this, world, random, sbb);
    this.placeBlock(world, blk("air"), 0, 2, 3, 4, sbb);
    this.placeBlock(world, blk("air"), 0, 3, 3, 4, sbb);
    this.placeBlock(world, blk("flowing_lava"), 0, 2, 3, 4, sbb);
    this.placeBlock(world, blk("flowing_lava"), 0, 3, 3, 4, sbb);
    true
}
