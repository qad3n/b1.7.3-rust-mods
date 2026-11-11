// draft from ComponentTFMazeCorridorIronFence.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, x: i32, y: i32, z: i32, rotation: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::MazeCorridorIronFence);
    init(&this, i, x, y, z, rotation);
    Rc::new(this)
}

// ComponentTFMazeCorridorIronFence constructor
pub fn init(this: &Piece, i: i32, x: i32, y: i32, z: i32, rotation: i32)
{
    maze_corridor::init(this, i, x, y, z, rotation);
}

// ComponentTFMazeCorridorIronFence.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, _random: &Random, sbb: StructureBox) -> bool
{
    this.fillMeta(world, sbb, 1, 4, 2, 4, 4, 3, blk("tf:mazestone"), 3, blk("air"), 0, false);
    this.fillMeta(world, sbb, 1, 1, 2, 4, 3, 3, blk("tf:mazestone"), 2, blk("air"), 0, false);
    this.fill(world, sbb, 2, 1, 2, 3, 3, 3, blk("iron_bars"), blk("air"), false);
    return true;
}
