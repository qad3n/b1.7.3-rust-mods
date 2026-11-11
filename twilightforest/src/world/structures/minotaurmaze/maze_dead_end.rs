// draft from ComponentTFMazeDeadEnd.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, x: i32, y: i32, z: i32, rotation: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::MazeDeadEnd);
    init(&this, i, x, y, z, rotation);
    Rc::new(this)
}

// ComponentTFMazeDeadEnd constructor
pub fn init(this: &Piece, i: i32, x: i32, y: i32, z: i32, rotation: i32)
{
    this.m_depth.set(i);
    this.m_mode.set(rotation);
    this.m_box.set(StructureBox::new([x, y, z], [x + 5, y + 5, z + 5]));
}

// ComponentTFMazeDeadEnd.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, _random: &Random, sbb: StructureBox) -> bool
{
    this.fillMeta(world, sbb, 1, 1, 0, 4, 4, 0, blk("fence"), 0, blk("air"), 0, false);
    this.fillAir(world, sbb, 2, 1, 0, 3, 3, 0);
    return true;
}
