// draft from ComponentTFTowerBeard.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, wing: &Piece) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::TowerBeard);
    init(&this, i, wing);
    Rc::new(this)
}

// ComponentTFTowerBeard constructor
pub fn init(this: &Piece, i: i32, wing: &Piece)
{
    this.m_depth.set(i);
    this.setMode(wing.mode());
    this.m_size.set(wing.m_size.get() - 2);
    this.m_height.set(this.m_size.get() / 2);
    this.m_box.set(StructureBox::new(
        [wing.bounds().m_min[0] + 1, (wing.bounds().m_min[1] - this.m_height.get()) - 1, wing.bounds().m_min[2] + 1],
        [wing.bounds().m_max[0] - 1, wing.bounds().m_min[1] - 1, wing.bounds().m_max[2] - 1],
    ));
}

// ComponentTFTowerBeard.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    return tower_beard::makePyramidBeard(this, world, random, sbb);
}

// ComponentTFTowerBeard.makePyramidBeard
pub fn makePyramidBeard(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    for y in 0..=this.m_height.get()
    {
        let min: i32 = y;
        let max: i32 = (this.m_size.get() - y) - 1;
        this.fillSelected(
            world,
            sbb,
            min,
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
