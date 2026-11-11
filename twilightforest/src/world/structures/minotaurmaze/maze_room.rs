// draft from ComponentTFMazeRoom.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(i: i32, random: &Random, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::MazeRoom);
    init(&this, i, random, x, y, z);
    Rc::new(this)
}

// ComponentTFMazeRoom constructor
pub(crate) fn init(this: &Piece, i: i32, random: &Random, x: i32, y: i32, z: i32)
{
    this.m_depth.set(i);
    this.m_mode.set(random.nextIntBounded(4));
    this.m_box.set(StructureBox::new([x, y, z], [x + 15, y + 4, z + 15]));
}

// ComponentTFMazeRoom.func_74861_a
pub(crate) fn build(_this: &Piece, _structurecomponent: &Piece, _list: &mut Pieces, _random: &Random) {}

// ComponentTFMazeRoom.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, _random: &Random, sbb: StructureBox) -> bool
{
    this.fillMeta(world, sbb, 1, 0, 1, 14, 0, 14, blk("tf:mazestone"), 7, blk("air"), 0, true);
    this.fillMeta(world, sbb, 2, 0, 2, 13, 0, 13, blk("tf:mazestone"), 6, blk("air"), 0, true);

    if this.getBlock(world, 7, 1, 0, sbb) == blk("air")
    {
        this.fill(world, sbb, 6, 1, 0, 9, 4, 0, blk("fence"), blk("air"), false);
        this.fillAir(world, sbb, 7, 1, 0, 8, 3, 0);
    }

    if this.getBlock(world, 7, 1, 15, sbb) == blk("air")
    {
        this.fill(world, sbb, 6, 1, 15, 9, 4, 15, blk("fence"), blk("air"), false);
        this.fillAir(world, sbb, 7, 1, 15, 8, 3, 15);
    }

    if this.getBlock(world, 0, 1, 7, sbb) == blk("air")
    {
        this.fill(world, sbb, 0, 1, 6, 0, 4, 9, blk("fence"), blk("air"), false);
        this.fillAir(world, sbb, 0, 1, 7, 0, 3, 8);
    }

    if this.getBlock(world, 15, 1, 7, sbb) == blk("air")
    {
        this.fill(world, sbb, 15, 1, 6, 15, 4, 9, blk("fence"), blk("air"), false);
        this.fillAir(world, sbb, 15, 1, 7, 15, 3, 8);
    }

    true
}
