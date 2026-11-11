// draft from ComponentTFDarkTowerRoofCactus.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(i: i32, wing: &Piece) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::DarkTowerRoofCactus);
    init(&this, i, wing);
    Rc::new(this)
}

// ComponentTFDarkTowerRoofCactus constructor
pub(crate) fn init(this: &Piece, i: i32, wing: &Piece) { dark_tower_roof::init(this, i, wing); }

// ComponentTFDarkTowerRoofCactus.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    dark_tower_roof::add(this, world, random, sbb);
    dark_tower_roof::mast(this, world, sbb);
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        (this.m_size.get() / 2) + 1,
        7,
        this.m_size.get() / 2,
        sbb,
    );
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        (this.m_size.get() / 2) + 2,
        7,
        this.m_size.get() / 2,
        sbb,
    );
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        (this.m_size.get() / 2) + 2,
        8,
        this.m_size.get() / 2,
        sbb,
    );
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        (this.m_size.get() / 2) + 2,
        9,
        this.m_size.get() / 2,
        sbb,
    );
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        (this.m_size.get() / 2) + 3,
        9,
        this.m_size.get() / 2,
        sbb,
    );
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        this.m_size.get() / 2,
        6,
        (this.m_size.get() / 2) + 1,
        sbb,
    );
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        this.m_size.get() / 2,
        6,
        (this.m_size.get() / 2) + 2,
        sbb,
    );
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        this.m_size.get() / 2,
        7,
        (this.m_size.get() / 2) + 2,
        sbb,
    );
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        this.m_size.get() / 2,
        8,
        (this.m_size.get() / 2) + 2,
        sbb,
    );
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        this.m_size.get() / 2,
        8,
        (this.m_size.get() / 2) + 3,
        sbb,
    );
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        (this.m_size.get() / 2) - 1,
        5,
        this.m_size.get() / 2,
        sbb,
    );
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        (this.m_size.get() / 2) - 2,
        5,
        this.m_size.get() / 2,
        sbb,
    );
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        (this.m_size.get() / 2) - 2,
        6,
        this.m_size.get() / 2,
        sbb,
    );
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        (this.m_size.get() / 2) - 2,
        7,
        this.m_size.get() / 2,
        sbb,
    );
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        (this.m_size.get() / 2) - 3,
        7,
        this.m_size.get() / 2,
        sbb,
    );
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        this.m_size.get() / 2,
        4,
        (this.m_size.get() / 2) - 1,
        sbb,
    );
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        this.m_size.get() / 2,
        4,
        (this.m_size.get() / 2) - 2,
        sbb,
    );
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        this.m_size.get() / 2,
        5,
        (this.m_size.get() / 2) - 2,
        sbb,
    );
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        this.m_size.get() / 2,
        6,
        (this.m_size.get() / 2) - 2,
        sbb,
    );
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        this.m_size.get() / 2,
        6,
        (this.m_size.get() / 2) - 3,
        sbb,
    );
    true
}
