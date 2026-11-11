// draft from ComponentTFFinalCastleBellFoundation21.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(random: &Random, i: i32, sideTower: &Piece) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::FinalCastleBellFoundation21);
    init(&this, random, i, sideTower);
    Rc::new(this)
}

// ComponentTFFinalCastleBellFoundation21 constructor
pub(crate) fn init(this: &Piece, random: &Random, i: i32, sideTower: &Piece)
{
    final_castle_foundation13::init(this, random, i, sideTower);
    this.m_box.set(StructureBox::new(
        [sideTower.bounds().m_min[0] - 2, sideTower.bounds().m_max[1] - 1, sideTower.bounds().m_min[2] - 2],
        [sideTower.bounds().m_max[0] + 2, sideTower.bounds().m_max[1], sideTower.bounds().m_max[2] + 2],
    ));
}

// ComponentTFFinalCastleBellFoundation21.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, _random: &Random, sbb: StructureBox) -> bool
{
    if this.m_groundLevel.get() < 0
    {
        this.m_groundLevel.set(final_castle_foundation13::deadrockLevel(this, world, sbb));
    }

    let _height: i32 = this.m_box.get().m_max[1] - this.m_groundLevel.get();
    let mid: i32 = 16;
    let low: i32 = 32;
    let size: i32 = this.m_box.get().m_max[0] - this.m_box.get().m_min[0];

    for rotation in 0..4
    {
        this.fillDownTurned(world, this.m_deco.get().m_blockID, this.m_deco.get().m_blockMeta, 2, -1, 1, rotation, sbb);
        this.fillDownTurned(
            world,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            2,
            -mid,
            0,
            rotation,
            sbb,
        );
        this.fillDownTurned(world, this.m_deco.get().m_blockID, this.m_deco.get().m_blockMeta, 1, -1, 2, rotation, sbb);
        this.fillDownTurned(
            world,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            0,
            -mid,
            2,
            rotation,
            sbb,
        );
        this.fillDownTurned(
            world,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            1,
            -low,
            1,
            rotation,
            sbb,
        );
        this.fillDownTurned(
            world,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            0,
            -low,
            1,
            rotation,
            sbb,
        );
        this.fillDownTurned(
            world,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            1,
            -low,
            0,
            rotation,
            sbb,
        );
        this.fillDownTurned(
            world,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            0,
            -low,
            0,
            rotation,
            sbb,
        );

        for x in (6..(size - 3)).step_by(4)
        {
            this.fillDownTurned(
                world,
                this.m_deco.get().m_blockID,
                this.m_deco.get().m_blockMeta,
                x,
                -1,
                1,
                rotation,
                sbb,
            );
            this.fillDownTurned(
                world,
                this.m_deco.get().m_blockID,
                this.m_deco.get().m_blockMeta,
                x,
                -mid,
                0,
                rotation,
                sbb,
            );
        }
    }

    true
}
