// draft from ComponentTFTrollCloud.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(index: i32, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::TrollCloud);
    init(&this, index, x, y, z);
    Rc::new(this)
}

// ComponentTFTrollCloud constructor
pub fn init(this: &Piece, index: i32, x: i32, y: i32, z: i32)
{
    this.m_depth.set(index);
    this.setMode(0);
    this.m_size.set(40);
    this.m_height.set(20);
    let radius: i32 = this.m_size.get() / 2;
    this.m_box.set(boxFor(
        x,
        y,
        z,
        -radius,
        -this.m_height.get(),
        -radius,
        this.m_size.get(),
        this.m_height.get(),
        this.m_size.get(),
        0,
    ));
}

// ComponentTFTrollCloud.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    troll_cloud::placeCloud(this, world, sbb, 0, 0, 0, this.m_size.get() - 1, 6, this.m_size.get() - 1);
    return true;
}

// ComponentTFTrollCloud.placeCloud
pub fn placeCloud(
    this: &Piece,
    world: &GenWorld,
    sbb: StructureBox,
    minX: i32,
    minY: i32,
    minZ: i32,
    maxX: i32,
    maxY: i32,
    maxZ: i32,
)
{
    this.fillMeta(
        world,
        sbb,
        minX,
        minY,
        minZ,
        maxX,
        maxY,
        maxZ,
        blk("stained_glass"),
        0,
        blk("stained_glass"),
        0,
        false,
    );
    this.fillMeta(
        world,
        sbb,
        minX + 2,
        minY + 2,
        minZ + 2,
        maxX - 2,
        maxY - 1,
        maxZ - 2,
        blk("quartz_block"),
        0,
        blk("quartz_block"),
        0,
        false,
    );
}
