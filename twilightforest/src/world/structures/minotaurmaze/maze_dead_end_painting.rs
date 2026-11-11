// draft from ComponentTFMazeDeadEndPainting.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(i: i32, x: i32, y: i32, z: i32, rotation: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::MazeDeadEndPainting);
    init(&this, i, x, y, z, rotation);
    Rc::new(this)
}

// ComponentTFMazeDeadEndPainting constructor
pub(crate) fn init(this: &Piece, i: i32, x: i32, y: i32, z: i32, rotation: i32)
{
    maze_dead_end::init(this, i, x, y, z, rotation);
}

// ComponentTFMazeDeadEndPainting.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    maze_dead_end::add(this, world, random, sbb);
    this.placeBlock(world, blk("torch"), 0, 1, 3, 3, sbb);
    this.placeBlock(world, blk("torch"), 0, 4, 3, 3, sbb);
    true
}
