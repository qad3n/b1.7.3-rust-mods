// draft from ComponentTFDarkTowerRoofCactus.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, wing: &Piece) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::DarkTowerRoofCactus);
    init(&this, i, wing);
    Rc::new(this)
}

// ComponentTFDarkTowerRoofCactus constructor
pub fn init(this: &Piece, i: i32, wing: &Piece) { dark_tower_roof::init(this, i, &wing); }

// ComponentTFDarkTowerRoofCactus.func_74875_a
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
    return true;
}
