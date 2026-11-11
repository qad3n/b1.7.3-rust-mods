// draft from ComponentTFFinalCastleRoof9Crenellated.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(random: &Random, i: i32, sideTower: &Piece) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::FinalCastleRoof9Crenellated);
    init(&this, random, i, sideTower);
    Rc::new(this)
}

// ComponentTFFinalCastleRoof9Crenellated constructor
pub fn init(this: &Piece, _random: &Random, i: i32, sideTower: &Piece)
{
    this.m_depth.set(i);
    let height: i32 = 5;
    this.setMode(sideTower.mode());
    this.m_box.set(StructureBox::new(
        [sideTower.bounds().m_min[0] - 2, sideTower.bounds().m_max[1] - 1, sideTower.bounds().m_min[2] - 2],
        [sideTower.bounds().m_max[0] + 2, (sideTower.bounds().m_max[1] + height) - 1, sideTower.bounds().m_max[2] + 2],
    ));
}

// ComponentTFFinalCastleRoof9Crenellated.func_74861_a
pub fn build(this: &Piece, parent: &Piece, _list: &mut Pieces, _random: &Random)
{
    this.m_deco.set((parent).m_deco.get());
}

// ComponentTFFinalCastleRoof9Crenellated.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, _random: &Random, sbb: StructureBox) -> bool
{
    for rotation in 0..4
    {
        this.fillTurned(
            world,
            sbb,
            0,
            -1,
            0,
            2,
            3,
            2,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            rotation,
        );
        this.placeTurned(world, this.m_deco.get().m_blockID, this.m_deco.get().m_blockMeta, 1, -2, 2, rotation, sbb);
        this.placeTurned(world, this.m_deco.get().m_blockID, this.m_deco.get().m_blockMeta, 1, -2, 1, rotation, sbb);
        this.placeTurned(world, this.m_deco.get().m_blockID, this.m_deco.get().m_blockMeta, 2, -2, 1, rotation, sbb);
        this.placeTurned(world, this.m_deco.get().m_blockID, this.m_deco.get().m_blockMeta, 3, 0, 1, rotation, sbb);
        this.placeTurned(world, this.m_deco.get().m_blockID, this.m_deco.get().m_blockMeta, 3, 1, 1, rotation, sbb);
        this.fillTurned(
            world,
            sbb,
            4,
            0,
            0,
            5,
            3,
            2,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            rotation,
        );
        this.placeTurned(world, this.m_deco.get().m_blockID, this.m_deco.get().m_blockMeta, 6, 0, 1, rotation, sbb);
        this.placeTurned(world, this.m_deco.get().m_blockID, this.m_deco.get().m_blockMeta, 6, 1, 1, rotation, sbb);
        this.fillTurned(
            world,
            sbb,
            7,
            0,
            0,
            8,
            3,
            2,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            rotation,
        );
        this.placeTurned(world, this.m_deco.get().m_blockID, this.m_deco.get().m_blockMeta, 9, 0, 1, rotation, sbb);
        this.placeTurned(world, this.m_deco.get().m_blockID, this.m_deco.get().m_blockMeta, 9, 1, 1, rotation, sbb);
    }
    return true;
}
