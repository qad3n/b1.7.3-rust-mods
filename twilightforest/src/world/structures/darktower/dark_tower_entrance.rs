// draft from ComponentTFDarkTowerEntrance.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(i: i32, x: i32, y: i32, z: i32, pSize: i32, pHeight: i32, direction: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::DarkTowerEntrance);
    init(&this, i, x, y, z, pSize, pHeight, direction);
    Rc::new(this)
}

// ComponentTFDarkTowerEntrance constructor
pub(crate) fn init(this: &Piece, i: i32, x: i32, y: i32, z: i32, pSize: i32, pHeight: i32, direction: i32)
{
    dark_tower_wing::init(this, i, x, y, z, pSize, pHeight, direction);
}

// ComponentTFDarkTowerEntrance.func_74861_a
pub(crate) fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    dark_tower_wing::build(this, parent, list, random);
    dark_tower_wing::addOpening5(this, this.m_size.get() / 2, 1, 0, 1, Door::Reappearing);
    dark_tower_wing::addOpening5(this, this.m_size.get() / 2, 1, this.m_size.get() - 1, 3, Door::Reappearing);
}

// ComponentTFDarkTowerEntrance.makeABeard
pub(crate) fn makeABeard(_this: &Piece, _parent: &Piece, _list: &mut Pieces, _random: &Random) {}

// ComponentTFDarkTowerEntrance.makeARoof
pub(crate) fn makeARoof(_this: &Piece, _parent: &Piece, _list: &mut Pieces, _random: &Random) {}

// ComponentTFDarkTowerEntrance.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    dark_tower_wing::makeEncasedWalls(
        this,
        world,
        random,
        sbb,
        0,
        0,
        0,
        this.m_size.get() - 1,
        this.m_height.get() - 1,
        this.m_size.get() - 1,
    );

    for x in 0..this.m_size.get()
    {
        for z in 0..this.m_size.get()
        {
            this.fillDown(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, x, -1, z, sbb);
        }
    }

    this.fillAir(world, sbb, 1, 1, 1, this.m_size.get() - 2, this.m_height.get() - 2, this.m_size.get() - 2);
    this.darken(world);
    this.makeOpenings(world, sbb);
    true
}
