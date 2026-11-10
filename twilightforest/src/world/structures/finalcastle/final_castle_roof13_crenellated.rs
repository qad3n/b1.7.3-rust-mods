// draft from ComponentTFFinalCastleRoof13Crenellated.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(random: &Random, i: i32, sideTower: &Piece) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::FinalCastleRoof13Crenellated);
    init(&this, random, i, sideTower);
    Rc::new(this)
}

// ComponentTFFinalCastleRoof13Crenellated constructor
pub fn init(this: &Piece, random: &Random, i: i32, sideTower: &Piece)
{
    this.m_depth.set(i);
    let height: i32 = 5;
    this.setMode(sideTower.mode());
    this.m_box.set(StructureBox::new(
        [sideTower.bounds().m_min[0] - 2, sideTower.bounds().m_max[1] - 1, sideTower.bounds().m_min[2] - 2],
        [sideTower.bounds().m_max[0] + 2, (sideTower.bounds().m_max[1] + height) - 1, sideTower.bounds().m_max[2] + 2],
    ));
}

// ComponentTFFinalCastleRoof13Crenellated.func_74861_a
pub fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    this.m_deco.set((parent).m_deco.get());
}

// ComponentTFFinalCastleRoof13Crenellated.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    let size: i32 = this.m_box.get().m_max[0] - this.m_box.get().m_min[0];
    for rotation in 0..4
    {
        this.fillTurned(
            world,
            sbb,
            0,
            -1,
            0,
            3,
            3,
            3,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            rotation,
        );
        this.placeTurned(world, this.m_deco.get().m_blockID, this.m_deco.get().m_blockMeta, 1, -2, 2, rotation, sbb);
        this.placeTurned(world, this.m_deco.get().m_blockID, this.m_deco.get().m_blockMeta, 1, -2, 1, rotation, sbb);
        this.placeTurned(world, this.m_deco.get().m_blockID, this.m_deco.get().m_blockMeta, 2, -2, 1, rotation, sbb);
        this.fillTurned(
            world,
            sbb,
            4,
            0,
            1,
            size - 4,
            1,
            1,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            rotation,
        );
        for x in (5..(size - 5)).step_by(4)
        {
            this.fillTurned(
                world,
                sbb,
                x,
                0,
                0,
                x + 2,
                3,
                2,
                this.m_deco.get().m_blockID,
                this.m_deco.get().m_blockMeta,
                rotation,
            );
            this.placeTurned(
                world,
                this.m_deco.get().m_blockID,
                this.m_deco.get().m_blockMeta,
                x + 1,
                -1,
                1,
                rotation,
                sbb,
            );
            this.placeTurned(
                world,
                this.m_deco.get().m_blockID,
                this.m_deco.get().m_blockMeta,
                x + 1,
                -2,
                1,
                rotation,
                sbb,
            );
        }
    }
    return true;
}
