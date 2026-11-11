// draft from ComponentTFMazeRoomExit.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(i: i32, random: &Random, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::MazeRoomExit);
    init(&this, i, random, x, y, z);
    Rc::new(this)
}

// ComponentTFMazeRoomExit constructor
pub(crate) fn init(this: &Piece, i: i32, random: &Random, x: i32, y: i32, z: i32)
{
    maze_room::init(this, i, random, x, y, z);
}

// ComponentTFMazeRoomExit.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    maze_room::add(this, world, random, sbb);
    this.fillMeta(world, sbb, 5, -5, 5, 10, 0, 10, blk("tf:mazestone"), 1, blk("air"), 0, false);
    this.fillMeta(world, sbb, 5, 1, 5, 10, 1, 10, blk("tf:mazestone"), 3, blk("air"), 0, false);
    this.fillMeta(world, sbb, 5, 2, 5, 10, 3, 10, blk("iron_bars"), 0, blk("air"), 0, false);
    this.fillMeta(world, sbb, 5, 4, 5, 10, 4, 10, blk("tf:mazestone"), 3, blk("air"), 0, false);
    this.fillAir(world, sbb, 6, -5, 6, 9, 4, 9);
    true
}
