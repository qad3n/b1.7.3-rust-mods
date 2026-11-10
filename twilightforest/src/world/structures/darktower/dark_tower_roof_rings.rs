// draft from ComponentTFDarkTowerRoofRings.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, wing: &Piece) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::DarkTowerRoofRings);
    init(&this, i, wing);
    Rc::new(this)
}

// ComponentTFDarkTowerRoofRings constructor
pub fn init(this: &Piece, i: i32, wing: &Piece) { dark_tower_roof::init(this, i, &wing); }

// ComponentTFDarkTowerRoofRings.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    dark_tower_roof::add(this, world, random, sbb);
    for y in 1..10
    {
        this.placeBlock(
            world,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            this.m_size.get() / 2,
            y,
            this.m_size.get() / 2,
            sbb,
        );
    }
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        this.m_size.get() / 2,
        10,
        this.m_size.get() / 2,
        sbb,
    );
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        (this.m_size.get() / 2) - 1,
        1,
        this.m_size.get() / 2,
        sbb,
    );
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        (this.m_size.get() / 2) + 1,
        1,
        this.m_size.get() / 2,
        sbb,
    );
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        this.m_size.get() / 2,
        1,
        (this.m_size.get() / 2) - 1,
        sbb,
    );
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        this.m_size.get() / 2,
        1,
        (this.m_size.get() / 2) + 1,
        sbb,
    );
    dark_tower_roof_rings::makeARing(this, world, 6, sbb);
    dark_tower_roof_rings::makeARing(this, world, 8, sbb);
    return true;
}

// ComponentTFDarkTowerRoofRings.makeARing
pub fn makeARing(this: &Piece, world: &GenWorld, y: i32, sbb: StructureBox)
{
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        (this.m_size.get() / 2) - 2,
        y,
        (this.m_size.get() / 2) + 1,
        sbb,
    );
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        (this.m_size.get() / 2) - 2,
        y,
        (this.m_size.get() / 2) + 0,
        sbb,
    );
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        (this.m_size.get() / 2) - 2,
        y,
        (this.m_size.get() / 2) - 1,
        sbb,
    );
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        (this.m_size.get() / 2) + 2,
        y,
        (this.m_size.get() / 2) + 1,
        sbb,
    );
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        (this.m_size.get() / 2) + 2,
        y,
        (this.m_size.get() / 2) + 0,
        sbb,
    );
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        (this.m_size.get() / 2) + 2,
        y,
        (this.m_size.get() / 2) - 1,
        sbb,
    );
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        (this.m_size.get() / 2) + 1,
        y,
        (this.m_size.get() / 2) - 2,
        sbb,
    );
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        (this.m_size.get() / 2) + 0,
        y,
        (this.m_size.get() / 2) - 2,
        sbb,
    );
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        (this.m_size.get() / 2) - 1,
        y,
        (this.m_size.get() / 2) - 2,
        sbb,
    );
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        (this.m_size.get() / 2) + 1,
        y,
        (this.m_size.get() / 2) + 2,
        sbb,
    );
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        (this.m_size.get() / 2) + 0,
        y,
        (this.m_size.get() / 2) + 2,
        sbb,
    );
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        (this.m_size.get() / 2) - 1,
        y,
        (this.m_size.get() / 2) + 2,
        sbb,
    );
}
