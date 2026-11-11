// draft from ComponentTFIceTowerBeard.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, wing: &Piece) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::IceTowerBeard);
    init(&this, i, wing);
    Rc::new(this)
}

// ComponentTFIceTowerBeard constructor
pub fn init(this: &Piece, i: i32, wing: &Piece)
{
    this.m_depth.set(i);
    this.setMode(wing.mode());
    this.m_size.set(wing.m_size.get());
    this.m_height.set(((this.m_size.get()) as f32 * 1.414).round() as i32);
    this.m_deco.set(wing.m_deco.get());
    this.m_box.set(StructureBox::new(
        [wing.bounds().m_min[0], wing.bounds().m_min[1] - this.m_height.get(), wing.bounds().m_min[2]],
        [wing.bounds().m_max[0], wing.bounds().m_min[1], wing.bounds().m_max[2]],
    ));
}

// ComponentTFIceTowerBeard.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, _random: &Random, sbb: StructureBox) -> bool
{
    for x in 0..this.m_size.get()
    {
        for z in 0..this.m_size.get()
        {
            let rHeight: i32 = sqrtF(((x * x) + (z * z)) as f32).round() as i32;
            for y in 0..rHeight
            {
                this.placeBlock(
                    world,
                    this.m_deco.get().m_blockID,
                    this.m_deco.get().m_blockMeta,
                    x,
                    this.m_height.get() - y,
                    z,
                    sbb,
                );
            }
        }
    }
    return true;
}
