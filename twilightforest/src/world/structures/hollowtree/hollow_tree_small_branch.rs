// draft from ComponentTFHollowTreeSmallBranch.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, sx: i32, sy: i32, sz: i32, length: f64, angle: f64, tilt: f64, leafy: bool) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::HollowTreeSmallBranch);
    init(&this, i, sx, sy, sz, length, angle, tilt, leafy);
    Rc::new(this)
}

// ComponentTFHollowTreeSmallBranch constructor
pub fn init(this: &Piece, i: i32, sx: i32, sy: i32, sz: i32, length: f64, angle: f64, tilt: f64, leafy: bool)
{
    hollow_tree_med_branch::init(this, i, sx, sy, sz, length, angle, tilt, leafy);
}

// ComponentTFHollowTreeSmallBranch.func_74861_a
pub fn build(_this: &Piece, _structurecomponent: &Piece, _list: &mut Pieces, _random: &Random) {}

// ComponentTFHollowTreeSmallBranch.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
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
    hollow_tree_med_branch::drawBresehnam(
        this,
        world,
        sbb,
        rSrc[0],
        rSrc[1],
        rSrc[2],
        rDest[0],
        rDest[1],
        rDest[2],
        blk("tf:log"),
        12,
    );
    if this.m_leafy.get()
    {
        let leafRad: i32 = random.nextIntBounded(2) + 1;
        hollow_tree_med_branch::makeLeafBlob(this, world, sbb, rDest[0], rDest[1], rDest[2], leafRad);
    }
    return true;
}
