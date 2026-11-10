// draft from ComponentTFFinalCastleRoof13Conical.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(random: &Random, i: i32, sideTower: &Piece) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::FinalCastleRoof13Conical);
    init(&this, random, i, sideTower);
    Rc::new(this)
}

// ComponentTFFinalCastleRoof13Conical constructor
pub fn init(this: &Piece, random: &Random, i: i32, sideTower: &Piece)
{
    this.m_depth.set(i);
    this.m_slope.set((2 + random.nextIntBounded(3)) + random.nextIntBounded(3));
    let height: i32 = this.m_slope.get() * 4;
    this.setMode(sideTower.mode());
    this.m_box.set(StructureBox::new(
        [sideTower.bounds().m_min[0] - 2, sideTower.bounds().m_max[1] - 1, sideTower.bounds().m_min[2] - 2],
        [sideTower.bounds().m_max[0] + 2, (sideTower.bounds().m_max[1] + height) - 1, sideTower.bounds().m_max[2] + 2],
    ));
}

// ComponentTFFinalCastleRoof13Conical.func_74861_a
pub fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    this.m_deco.set((parent).m_deco.get());
}

// ComponentTFFinalCastleRoof13Conical.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    for rotation in 0..4
    {
        this.fillTurned(
            world,
            sbb,
            0,
            -1,
            0,
            3,
            2,
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
            12,
            1,
            1,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            rotation,
        );
        for i in (3..13).step_by(2)
        {
            this.fillTurned(
                world,
                sbb,
                i,
                -1,
                1,
                i,
                2,
                1,
                this.m_deco.get().m_blockID,
                this.m_deco.get().m_blockMeta,
                rotation,
            );
        }
        for i in 2..9
        {
            let base: i32 = 2 - this.m_slope.get();
            if i < 7
            {
                this.fillTurned(
                    world,
                    sbb,
                    i - 1,
                    ((i - 1) * this.m_slope.get()) + base,
                    i - 1,
                    i,
                    ((i * this.m_slope.get()) + base) - 1,
                    i,
                    this.m_deco.get().m_blockID,
                    this.m_deco.get().m_blockMeta,
                    rotation,
                );
            }
            else
            {
                this.fillTurned(
                    world,
                    sbb,
                    16 - i,
                    ((i - 1) * this.m_slope.get()) + base,
                    i,
                    16 - i,
                    ((i * this.m_slope.get()) + base) - 1,
                    i,
                    this.m_deco.get().m_roofID,
                    this.m_deco.get().m_roofMeta,
                    rotation,
                );
            }
            this.fillTurned(
                world,
                sbb,
                i + 1,
                ((i - 1) * this.m_slope.get()) + base,
                i,
                15 - i,
                ((i * this.m_slope.get()) + base) - 1,
                i,
                this.m_deco.get().m_roofID,
                this.m_deco.get().m_roofMeta,
                rotation,
            );
        }
        this.fillTurned(
            world,
            sbb,
            8,
            (this.m_slope.get() * 6) + 2,
            8,
            8,
            (this.m_slope.get() * 7) + 2,
            8,
            this.m_deco.get().m_roofID,
            this.m_deco.get().m_roofMeta,
            rotation,
        );
    }
    return true;
}
