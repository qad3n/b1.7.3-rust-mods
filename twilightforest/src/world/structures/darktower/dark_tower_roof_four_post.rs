// draft from ComponentTFDarkTowerRoofFourPost.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(i: i32, wing: &Piece) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::DarkTowerRoofFourPost);
    init(&this, i, wing);
    Rc::new(this)
}

// ComponentTFDarkTowerRoofFourPost constructor
pub(crate) fn init(this: &Piece, i: i32, wing: &Piece) { dark_tower_roof::init(this, i, wing); }

// ComponentTFDarkTowerRoofFourPost.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    dark_tower_roof::add(this, world, random, sbb);
    dark_tower_roof_four_post::makeSmallAntenna(this, world, sbb, 4, this.m_size.get() - 2, this.m_size.get() - 2);
    dark_tower_roof_four_post::makeSmallAntenna(this, world, sbb, 5, 1, this.m_size.get() - 2);
    dark_tower_roof_four_post::makeSmallAntenna(this, world, sbb, 6, this.m_size.get() - 2, 1);
    dark_tower_roof_four_post::makeSmallAntenna(this, world, sbb, 7, 1, 1);
    true
}

// ComponentTFDarkTowerRoofFourPost.makeSmallAntenna
pub(crate) fn makeSmallAntenna(this: &Piece, world: &GenWorld, sbb: StructureBox, height: i32, x: i32, z: i32)
{
    for y in 1..height
    {
        this.placeBlock(world, this.m_deco.get().m_blockID, this.m_deco.get().m_blockMeta, x, y, z, sbb);
    }

    this.placeBlock(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, x, height + 0, z, sbb);
    this.placeBlock(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, x, height + 1, z, sbb);
    this.placeBlock(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, x + 1, height + 1, z, sbb);
    this.placeBlock(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, x - 1, height + 1, z, sbb);
    this.placeBlock(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, x, height + 1, z + 1, sbb);
    this.placeBlock(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, x, height + 1, z - 1, sbb);
    this.placeBlock(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, x, height + 2, z, sbb);
}
