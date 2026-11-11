// draft from ComponentTFMazeDeadEndTorches.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(i: i32, x: i32, y: i32, z: i32, rotation: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::MazeDeadEndTorches);
    init(&this, i, x, y, z, rotation);
    Rc::new(this)
}

// ComponentTFMazeDeadEndTorches constructor
pub(crate) fn init(this: &Piece, i: i32, x: i32, y: i32, z: i32, rotation: i32)
{
    maze_dead_end::init(this, i, x, y, z, rotation);
}

// ComponentTFMazeDeadEndTorches.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    maze_dead_end::add(this, world, random, sbb);
    this.fillMeta(world, sbb, 2, 1, 4, 3, 4, 4, blk("torch"), 0, blk("air"), 0, false);
    this.fillMeta(world, sbb, 1, 1, 1, 1, 4, 4, blk("torch"), 0, blk("air"), 0, false);
    this.fillMeta(world, sbb, 4, 1, 1, 4, 4, 4, blk("torch"), 0, blk("air"), 0, false);
    true
}
