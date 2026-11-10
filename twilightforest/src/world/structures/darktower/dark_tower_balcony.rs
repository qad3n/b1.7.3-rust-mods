// draft from ComponentTFDarkTowerBalcony.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, x: i32, y: i32, z: i32, direction: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::DarkTowerBalcony);
    init(&this, i, x, y, z, direction);
    Rc::new(this)
}

// ComponentTFDarkTowerBalcony constructor
pub fn init(this: &Piece, i: i32, x: i32, y: i32, z: i32, direction: i32)
{
    tower_wing::init7(this, i, x, y, z, 5, 5, direction);
}

// ComponentTFDarkTowerBalcony.func_74861_a
pub fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    this.m_deco.set((parent).m_deco.get());
}

// ComponentTFDarkTowerBalcony.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    this.fillMeta(
        world,
        sbb,
        0,
        0,
        0,
        2,
        0,
        4,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        blk("air"),
        0,
        false,
    );
    this.fillMeta(
        world,
        sbb,
        0,
        0,
        1,
        1,
        0,
        3,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockMeta,
        blk("air"),
        0,
        false,
    );
    this.fillMeta(
        world,
        sbb,
        0,
        1,
        0,
        2,
        1,
        4,
        this.m_deco.get().m_fenceID,
        this.m_deco.get().m_fenceMeta,
        blk("air"),
        0,
        false,
    );
    this.placeBlock(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, 2, 1, 0, sbb);
    this.placeBlock(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, 2, 1, 4, sbb);
    this.fillAir(world, sbb, 0, 1, 1, 1, 1, 3);
    return true;
}
