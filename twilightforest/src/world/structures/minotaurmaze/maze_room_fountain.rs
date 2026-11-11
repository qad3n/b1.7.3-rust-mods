// draft from ComponentTFMazeRoomFountain.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(i: i32, random: &Random, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::MazeRoomFountain);
    init(&this, i, random, x, y, z);
    Rc::new(this)
}

// ComponentTFMazeRoomFountain constructor
pub(crate) fn init(this: &Piece, i: i32, random: &Random, x: i32, y: i32, z: i32)
{
    maze_room::init(this, i, random, x, y, z);
}

// ComponentTFMazeRoomFountain.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    maze_room::add(this, world, random, sbb);
    this.fillMeta(world, sbb, 5, 1, 5, 10, 1, 10, blk("tf:mazestone"), 3, blk("air"), 0, false);
    this.fill(world, sbb, 6, 1, 6, 9, 1, 9, blk("water"), blk("air"), false);
    true
}
