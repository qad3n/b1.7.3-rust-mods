// draft from ComponentTFHydraLair.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(world: &GenWorld, random: &Random, i: i32, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::HydraLair);
    init(&this, world, random, i, x, y, z);
    Rc::new(this)
}

// ComponentTFHydraLair constructor
pub(crate) fn init(this: &Piece, world: &GenWorld, random: &Random, i: i32, x: i32, y: i32, z: i32)
{
    hollow_hill::init(this, world, random, i, 2, x, y + 2, z);
}

// ComponentTFHydraLair.func_74861_a
pub(crate) fn build(_this: &Piece, _structurecomponent: &Piece, _list: &mut Pieces, _random: &Random) {}

// ComponentTFHydraLair.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    let stalacts: i32 = 64;
    let stalags: i32 = 8;

    for _i in 0..stalacts
    {
        let dest: Vec<i32> = hollow_hill::coordsInHill2D(this, random);
        hollow_hill::generateOreStalactite(this, world, dest[0], 1, dest[1], sbb);
    }

    for _i in 0..stalacts
    {
        let dest: Vec<i32> = hollow_hill::coordsInHill2D(this, random);
        hollow_hill::generateBlockStalactite(this, world, blk("stone"), 1.0, true, dest[0], 1, dest[1], sbb);
    }

    for _i in 0..stalags
    {
        let dest: Vec<i32> = hollow_hill::coordsInHill2D(this, random);
        hollow_hill::generateBlockStalactite(this, world, blk("stone"), 0.9, false, dest[0], 1, dest[1], sbb);
    }

    this.placeBlock(world, blk("tf:bossSpawner"), 2, 27, 3, 27, sbb);
    true
}
