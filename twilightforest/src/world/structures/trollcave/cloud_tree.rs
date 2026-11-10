// draft from ComponentTFCloudTree.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(index: i32, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::CloudTree);
    init(&this, index, x, y, z);
    Rc::new(this)
}

// ComponentTFCloudTree constructor
pub fn init(this: &Piece, index: i32, mut x: i32, mut y: i32, mut z: i32)
{
    this.setMode(0);
    x = (x >> 2) << 2;
    y = (y >> 2) << 2;
    z = (z >> 2) << 2;
    this.m_box.set(boxFor(x, y, z, -8, 0, -8, 20, 28, 20, 0));
    this.m_spawnListIndex.set(1);
}

// ComponentTFCloudTree.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    this.fillMeta(world, sbb, 0, 12, 0, 19, 19, 19, blk("tf:giantLeaves"), 0, blk("tf:giantLeaves"), 0, false);
    this.fillMeta(world, sbb, 4, 20, 4, 15, 23, 15, blk("tf:giantLeaves"), 0, blk("tf:giantLeaves"), 0, false);
    this.fillMeta(world, sbb, 8, 24, 4, 11, 27, 15, blk("tf:giantLeaves"), 0, blk("tf:giantLeaves"), 0, false);
    this.fillMeta(world, sbb, 4, 24, 8, 15, 27, 11, blk("tf:giantLeaves"), 0, blk("tf:giantLeaves"), 0, false);
    this.fillMeta(world, sbb, 8, 0, 8, 11, 23, 11, blk("tf:giantLog"), 0, blk("tf:giantLog"), 0, false);
    this.fillMeta(world, sbb, 8, -4, 8, 11, -1, 11, blk("tf:fluffyCloud"), 0, blk("tf:fluffyCloud"), 0, false);
    return true;
}
