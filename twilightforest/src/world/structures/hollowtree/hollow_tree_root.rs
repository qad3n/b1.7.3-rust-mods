// draft from ComponentTFHollowTreeRoot.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, sx: i32, sy: i32, sz: i32, length: f64, angle: f64, tilt: f64, leafy: bool) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::HollowTreeRoot);
    init(&this, i, sx, sy, sz, length, angle, tilt, leafy);
    Rc::new(this)
}

// ComponentTFHollowTreeRoot constructor
pub fn init(this: &Piece, i: i32, sx: i32, sy: i32, sz: i32, length: f64, angle: f64, tilt: f64, leafy: bool)
{
    hollow_tree_med_branch::init(this, i, sx, sy, sz, length, angle, tilt, leafy);
    this.m_groundLevel.set(-1);
    this.m_box.set(StructureBox::new(
        [
            this.m_src.get()[0].min(this.m_dest.get()[0]),
            this.m_src.get()[1].min(this.m_dest.get()[1]),
            this.m_src.get()[2].min(this.m_dest.get()[2]),
        ],
        [
            this.m_src.get()[0].max(this.m_dest.get()[0]),
            this.m_src.get()[1].max(this.m_dest.get()[1]),
            this.m_src.get()[2].max(this.m_dest.get()[2]),
        ],
    ));
}

// ComponentTFHollowTreeRoot.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    if this.m_groundLevel.get() < 0
    {
        let rootHeight: i32 = this.m_box.get().m_max[1] - this.m_box.get().m_min[1];
        this.m_groundLevel.set(this.dirtLevel(world, sbb));
        if this.m_groundLevel.get() < 0
        {
            return true;
        }
        this.m_src.get()[1] = this.m_groundLevel.get() + 5;
    }
    let rSrc: [i32; 3] = [
        this.m_src.get()[0] - this.m_box.get().m_min[0],
        this.m_src.get()[1] - this.m_box.get().m_min[1],
        this.m_src.get()[2] - this.m_box.get().m_min[2],
    ];
    let rDest: [i32; 3] = [
        this.m_dest.get()[0] - this.m_box.get().m_min[0],
        this.m_dest.get()[1] - this.m_box.get().m_min[1],
        this.m_dest.get()[2] - this.m_box.get().m_min[2],
    ];
    hollow_tree_root::drawRootLine(
        this,
        world,
        sbb,
        rSrc[0],
        rSrc[1],
        rSrc[2],
        rDest[0],
        rDest[1],
        rDest[2],
        blk("tf:root"),
        0,
    );
    hollow_tree_root::drawRootLine(
        this,
        world,
        sbb,
        rSrc[0],
        rSrc[1] - 1,
        rSrc[2],
        rDest[0],
        rDest[1] - 1,
        rDest[2],
        blk("tf:root"),
        0,
    );
    return true;
}

// ComponentTFHollowTreeRoot.drawRootLine
pub fn drawRootLine(
    this: &Piece,
    world: &GenWorld,
    sbb: StructureBox,
    x1: i32,
    y1: i32,
    z1: i32,
    x2: i32,
    y2: i32,
    z2: i32,
    blockValue: Blk,
    metaValue: i32,
)
{
    let lineCoords: Vec<[i32; 3]> = bresenham(x1, y1, z1, x2, y2, z2);
    for coords in lineCoords.clone()
    {
        let block: Blk = this.getBlock(world, coords[0], coords[1], coords[2], sbb);
        if block.isNormalCube(world, coords[0], coords[1], coords[2])
            && ((false) || (block.material() != Material::Grass))
        {
            if (false) || (block.material() != Material::Wood)
            {
                this.placeBlock(world, blockValue, metaValue, coords[0], coords[1], coords[2], sbb);
            }
        }
        else
        {
            this.placeBlock(world, blk("tf:log"), 12, coords[0], coords[1], coords[2], sbb);
        }
    }
}
