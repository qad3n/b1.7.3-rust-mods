// draft from ComponentTFIceTowerRoof.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(i: i32, wing: &Piece) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::IceTowerRoof);
    init(&this, i, wing);
    Rc::new(this)
}

// ComponentTFIceTowerRoof constructor
pub(crate) fn init(this: &Piece, i: i32, wing: &Piece)
{
    tower_roof::init(this, i, wing);
    this.setMode(wing.mode());
    this.m_size.set(wing.m_size.get());
    this.m_height.set(12);
    this.m_deco.set(wing.m_deco.get());
    tower_roof::makeCapBB(this, wing);
}

// ComponentTFIceTowerRoof.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    tower_roof::add(this, world, random, sbb);

    for x in 0..this.m_size.get()
    {
        for z in 0..this.m_size.get()
        {
            let rHeight: i32 = sqrtF(((x * x) + (z * z)) as f32).round() as i32;
            for y in 0..rHeight
            {
                this.placeBlock(world, this.m_deco.get().m_blockID, this.m_deco.get().m_blockMeta, x, y, z, sbb);
            }
        }
    }

    true
}
