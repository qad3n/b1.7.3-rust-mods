// draft from ComponentTFYetiCave.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(world: &GenWorld, random: &Random, i: i32, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::YetiCave);
    init(&this, world, random, i, x, y, z);
    Rc::new(this)
}

// ComponentTFYetiCave constructor
pub(crate) fn init(this: &Piece, world: &GenWorld, random: &Random, i: i32, x: i32, y: i32, z: i32)
{
    hollow_hill::init(this, world, random, i, 2, x, y + 2, z);
}

// ComponentTFYetiCave.isInHill
pub(crate) fn isInHill(this: &Piece, cx: i32, cz: i32) -> bool
{
    (((cx < (this.m_radius.get() * 2)) && (cx > 0)) && (cz < (this.m_radius.get() * 2))) && (cz > 0)
}

// ComponentTFYetiCave.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    let sn: i32 = 128;
    for _i in 0..sn
    {
        let dest: Vec<i32> = hollow_hill::coordsInHill2D(this, random);
        hollow_hill::generateBlockStalactite(this, world, blk("stone"), 1.0, true, dest[0], 1, dest[1], sbb);
    }

    for _i in 0..sn
    {
        let dest: Vec<i32> = hollow_hill::coordsInHill2D(this, random);
        hollow_hill::generateBlockStalactite(this, world, blk("ice"), 1.0, true, dest[0], 1, dest[1], sbb);
    }

    for _i in 0..sn
    {
        let dest: Vec<i32> = hollow_hill::coordsInHill2D(this, random);
        hollow_hill::generateBlockStalactite(this, world, blk("packed_ice"), 0.9, true, dest[0], 1, dest[1], sbb);
    }

    if !this.m_yetiPlaced.get()
    {
        let bx: i32 = this.xAt(this.m_radius.get(), this.m_radius.get());
        let by: i32 = this.yAt(0);
        let bz: i32 = this.zAt(this.m_radius.get(), this.m_radius.get());
        if sbb.contains(bx, by, bz)
        {
            this.m_yetiPlaced.set(true);
            spawnHome(world, "twilightforest:yeti_alpha", bx, by, bz, true);
        }
    }

    true
}
