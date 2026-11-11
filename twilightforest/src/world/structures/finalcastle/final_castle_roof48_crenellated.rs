// draft from ComponentTFFinalCastleRoof48Crenellated.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(random: &Random, i: i32, keep: &Piece) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::FinalCastleRoof48Crenellated);
    init(&this, random, i, keep);
    Rc::new(this)
}

// ComponentTFFinalCastleRoof48Crenellated constructor
pub fn init(this: &Piece, _random: &Random, i: i32, keep: &Piece)
{
    this.m_depth.set(i);
    let height: i32 = 5;
    this.setMode(keep.mode());
    this.m_box.set(StructureBox::new(
        [keep.bounds().m_min[0] - 2, keep.bounds().m_max[1] - 1, keep.bounds().m_min[2] - 2],
        [keep.bounds().m_max[0] + 2, (keep.bounds().m_max[1] + height) - 1, keep.bounds().m_max[2] + 2],
    ));
}

// ComponentTFFinalCastleRoof48Crenellated.func_74861_a
pub fn build(this: &Piece, parent: &Piece, _list: &mut Pieces, _random: &Random)
{
    this.m_deco.set((parent).m_deco.get());
}

// ComponentTFFinalCastleRoof48Crenellated.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, _random: &Random, sbb: StructureBox) -> bool
{
    this.fillMeta(world, sbb, 2, 2, 2, 50, 2, 50, blk("tf:castleMagic"), 3, blk("tf:castleMagic"), 3, false);
    for rotation in 0..4
    {
        this.fillTurned(
            world,
            sbb,
            3,
            1,
            1,
            45,
            3,
            1,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            rotation,
        );
        for i in (10..41).step_by(5)
        {
            this.fillTurned(
                world,
                sbb,
                i,
                1,
                0,
                i + 2,
                5,
                2,
                this.m_deco.get().m_blockID,
                this.m_deco.get().m_blockMeta,
                rotation,
            );
            this.placeTurned(
                world,
                this.m_deco.get().m_blockID,
                this.m_deco.get().m_blockMeta,
                i + 1,
                0,
                1,
                rotation,
                sbb,
            );
        }
    }
    return true;
}
