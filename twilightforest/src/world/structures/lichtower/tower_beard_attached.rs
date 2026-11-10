// draft from ComponentTFTowerBeardAttached.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, wing: &Piece) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::TowerBeardAttached);
    init(&this, i, wing);
    Rc::new(this)
}

// ComponentTFTowerBeardAttached constructor
pub fn init(this: &Piece, i: i32, wing: &Piece)
{
    tower_beard::init(this, i, &wing);
    this.m_box.set(StructureBox::new(
        [wing.bounds().m_min[0], (wing.bounds().m_min[1] - this.m_height.get()) - 1, wing.bounds().m_min[2]],
        [wing.bounds().m_max[0], wing.bounds().m_min[1] - 1, wing.bounds().m_max[2]],
    ));
}

// ComponentTFTowerBeardAttached.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    return tower_beard_attached::makeAttachedBeard(this, world, random, sbb);
}

// ComponentTFTowerBeardAttached.makeAttachedBeard
pub fn makeAttachedBeard(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    for y in 0..=this.m_height.get()
    {
        let min: i32 = y + 1;
        let max: i32 = this.m_size.get() - y;
        this.fillSelected(
            world,
            sbb,
            0,
            this.m_height.get() - y,
            min,
            max,
            this.m_height.get() - y,
            max,
            false,
            random,
            Stones::stronghold(),
        );
    }
    return true;
}
