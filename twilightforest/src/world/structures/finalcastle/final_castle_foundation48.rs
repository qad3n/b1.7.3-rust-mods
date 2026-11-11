// draft from ComponentTFFinalCastleFoundation48.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(random: &Random, i: i32, sideTower: &Piece) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::FinalCastleFoundation48);
    init(&this, random, i, sideTower);
    Rc::new(this)
}

// ComponentTFFinalCastleFoundation48 constructor
pub(crate) fn init(this: &Piece, _random: &Random, i: i32, sideTower: &Piece)
{
    this.m_depth.set(i);
    this.m_groundLevel.set(-1);
    this.setMode(sideTower.mode());
    this.m_box.set(StructureBox::new(
        [sideTower.bounds().m_min[0], sideTower.bounds().m_min[1], sideTower.bounds().m_min[2]],
        [sideTower.bounds().m_max[0], sideTower.bounds().m_min[1] - 1, sideTower.bounds().m_max[2]],
    ));
}

// ComponentTFFinalCastleFoundation48.func_74861_a
pub(crate) fn build(this: &Piece, parent: &Piece, _list: &mut Pieces, _random: &Random)
{
    this.m_deco.set((parent).m_deco.get());
}

// ComponentTFFinalCastleFoundation48.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, _random: &Random, sbb: StructureBox) -> bool
{
    for x in 4..45
    {
        for z in 4..45
        {
            this.fillDown(world, this.m_deco.get().m_blockID, this.m_deco.get().m_blockMeta, x, -1, z, sbb);
        }
    }

    let mid: i32 = 16;

    for rotation in 0..4
    {
        this.fillDownTurned(world, this.m_deco.get().m_blockID, this.m_deco.get().m_blockMeta, 3, -2, 3, rotation, sbb);
        this.fillTurned(
            world,
            sbb,
            2,
            -2,
            1,
            46,
            -1,
            1,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            rotation,
        );
        this.fillTurned(
            world,
            sbb,
            2,
            -4,
            2,
            45,
            -1,
            2,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            rotation,
        );
        this.fillTurned(
            world,
            sbb,
            4,
            -6,
            3,
            44,
            -1,
            3,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            rotation,
        );

        for i in (9..45).step_by(6)
        {
            final_castle_foundation48::makePiling(this, world, sbb, mid, rotation, i);
        }

        final_castle_foundation48::makePiling(this, world, sbb, mid, rotation, 4);
        final_castle_foundation48::makePiling(this, world, sbb, mid, rotation, 44);
    }

    this.fillDownTurned(world, this.m_deco.get().m_blockID, this.m_deco.get().m_blockMeta, 21, -2, 0, 1, sbb);
    this.fillDownTurned(world, this.m_deco.get().m_blockID, this.m_deco.get().m_blockMeta, 21, -4, 1, 1, sbb);
    this.fillDownTurned(world, this.m_deco.get().m_blockID, this.m_deco.get().m_blockMeta, 21, -6, 2, 1, sbb);
    this.fillDownTurned(world, this.m_deco.get().m_blockID, this.m_deco.get().m_blockMeta, 27, -2, 0, 1, sbb);
    this.fillDownTurned(world, this.m_deco.get().m_blockID, this.m_deco.get().m_blockMeta, 27, -4, 1, 1, sbb);
    this.fillDownTurned(world, this.m_deco.get().m_blockID, this.m_deco.get().m_blockMeta, 27, -6, 2, 1, sbb);
    true
}

// ComponentTFFinalCastleFoundation48.makePiling
pub(crate) fn makePiling(this: &Piece, world: &GenWorld, sbb: StructureBox, mid: i32, rotation: i32, i: i32)
{
    this.fillDownTurned(world, this.m_deco.get().m_blockID, this.m_deco.get().m_blockMeta, i, -7, 3, rotation, sbb);
    this.fillDownTurned(world, this.m_deco.get().m_blockID, this.m_deco.get().m_blockMeta, i, -mid, 2, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_blockID, this.m_deco.get().m_blockMeta, i, -1, 0, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_blockID, this.m_deco.get().m_blockMeta, i, -3, 1, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_blockID, this.m_deco.get().m_blockMeta, i, -5, 2, rotation, sbb);
}
