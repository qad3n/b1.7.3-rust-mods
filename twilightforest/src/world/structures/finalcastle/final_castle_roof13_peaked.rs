// draft from ComponentTFFinalCastleRoof13Peaked.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(random: &Random, i: i32, sideTower: &Piece) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::FinalCastleRoof13Peaked);
    init(&this, random, i, sideTower);
    Rc::new(this)
}

// ComponentTFFinalCastleRoof13Peaked constructor
pub fn init(this: &Piece, random: &Random, i: i32, sideTower: &Piece)
{
    this.m_depth.set(i);
    let height: i32 = 18;
    this.setMode(sideTower.mode());
    this.m_box.set(StructureBox::new(
        [sideTower.bounds().m_min[0] - 2, sideTower.bounds().m_max[1] - 1, sideTower.bounds().m_min[2] - 2],
        [sideTower.bounds().m_max[0] + 2, (sideTower.bounds().m_max[1] + height) - 1, sideTower.bounds().m_max[2] + 2],
    ));
}

// ComponentTFFinalCastleRoof13Peaked.func_74861_a
pub fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    this.m_deco.set((parent).m_deco.get());
}

// ComponentTFFinalCastleRoof13Peaked.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    for i in 0..3
    {
        this.fillMeta(
            world,
            sbb,
            1,
            i,
            i,
            15,
            i,
            i,
            this.m_deco.get().m_roofID,
            this.m_deco.get().m_roofMeta,
            this.m_deco.get().m_roofID,
            this.m_deco.get().m_roofMeta,
            false,
        );
        this.fillMeta(
            world,
            sbb,
            1,
            i,
            16 - i,
            15,
            i,
            16 - i,
            this.m_deco.get().m_roofID,
            this.m_deco.get().m_roofMeta,
            this.m_deco.get().m_roofID,
            this.m_deco.get().m_roofMeta,
            false,
        );
    }
    for i in 0..3
    {
        let mut dz: i32 = 3 + i;
        this.fillMeta(
            world,
            sbb,
            2,
            5 + ((i - 1) * 2),
            dz,
            14,
            4 + (i * 2),
            dz,
            this.m_deco.get().m_roofID,
            this.m_deco.get().m_roofMeta,
            this.m_deco.get().m_roofID,
            this.m_deco.get().m_roofMeta,
            false,
        );
        this.fillMeta(
            world,
            sbb,
            1,
            1,
            dz,
            1,
            5 + ((i - 1) * 2),
            dz,
            this.m_deco.get().m_roofID,
            this.m_deco.get().m_roofMeta,
            this.m_deco.get().m_roofID,
            this.m_deco.get().m_roofMeta,
            false,
        );
        this.fillMeta(
            world,
            sbb,
            1,
            5 + ((i - 1) * 2),
            dz - 1,
            1,
            4 + (i * 2),
            dz,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            false,
        );
        this.fillMeta(
            world,
            sbb,
            15,
            1,
            dz,
            15,
            5 + ((i - 1) * 2),
            dz,
            this.m_deco.get().m_roofID,
            this.m_deco.get().m_roofMeta,
            this.m_deco.get().m_roofID,
            this.m_deco.get().m_roofMeta,
            false,
        );
        this.fillMeta(
            world,
            sbb,
            15,
            5 + ((i - 1) * 2),
            dz - 1,
            15,
            4 + (i * 2),
            dz,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            false,
        );
        dz = 13 - i;
        this.fillMeta(
            world,
            sbb,
            2,
            5 + ((i - 1) * 2),
            dz,
            14,
            4 + (i * 2),
            dz,
            this.m_deco.get().m_roofID,
            this.m_deco.get().m_roofMeta,
            this.m_deco.get().m_roofID,
            this.m_deco.get().m_roofMeta,
            false,
        );
        this.fillMeta(
            world,
            sbb,
            1,
            1,
            dz,
            1,
            5 + ((i - 1) * 2),
            dz,
            this.m_deco.get().m_roofID,
            this.m_deco.get().m_roofMeta,
            this.m_deco.get().m_roofID,
            this.m_deco.get().m_roofMeta,
            false,
        );
        this.fillMeta(
            world,
            sbb,
            1,
            5 + ((i - 1) * 2),
            dz,
            1,
            4 + (i * 2),
            dz + 1,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            false,
        );
        this.fillMeta(
            world,
            sbb,
            15,
            1,
            dz,
            15,
            5 + ((i - 1) * 2),
            dz,
            this.m_deco.get().m_roofID,
            this.m_deco.get().m_roofMeta,
            this.m_deco.get().m_roofID,
            this.m_deco.get().m_roofMeta,
            false,
        );
        this.fillMeta(
            world,
            sbb,
            15,
            5 + ((i - 1) * 2),
            dz,
            15,
            4 + (i * 2),
            dz + 1,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            false,
        );
    }
    for i in 0..3
    {
        let mut dz: i32 = 6 + i;
        this.fillMeta(
            world,
            sbb,
            2,
            12 + ((i - 1) * 3),
            dz,
            14,
            11 + (i * 3),
            dz,
            this.m_deco.get().m_roofID,
            this.m_deco.get().m_roofMeta,
            this.m_deco.get().m_roofID,
            this.m_deco.get().m_roofMeta,
            false,
        );
        this.fillMeta(
            world,
            sbb,
            1,
            1,
            dz,
            1,
            12 + ((i - 1) * 3),
            dz,
            this.m_deco.get().m_roofID,
            this.m_deco.get().m_roofMeta,
            this.m_deco.get().m_roofID,
            this.m_deco.get().m_roofMeta,
            false,
        );
        this.fillMeta(
            world,
            sbb,
            1,
            12 + ((i - 1) * 3),
            dz - 1,
            1,
            11 + (i * 3),
            dz,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            false,
        );
        this.fillMeta(
            world,
            sbb,
            15,
            1,
            dz,
            15,
            12 + ((i - 1) * 3),
            dz,
            this.m_deco.get().m_roofID,
            this.m_deco.get().m_roofMeta,
            this.m_deco.get().m_roofID,
            this.m_deco.get().m_roofMeta,
            false,
        );
        this.fillMeta(
            world,
            sbb,
            15,
            12 + ((i - 1) * 3),
            dz - 1,
            15,
            11 + (i * 3),
            dz,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            false,
        );
        dz = 10 - i;
        this.fillMeta(
            world,
            sbb,
            2,
            12 + ((i - 1) * 3),
            dz,
            14,
            11 + (i * 3),
            dz,
            this.m_deco.get().m_roofID,
            this.m_deco.get().m_roofMeta,
            this.m_deco.get().m_roofID,
            this.m_deco.get().m_roofMeta,
            false,
        );
        this.fillMeta(
            world,
            sbb,
            1,
            1,
            dz,
            1,
            12 + ((i - 1) * 3),
            dz,
            this.m_deco.get().m_roofID,
            this.m_deco.get().m_roofMeta,
            this.m_deco.get().m_roofID,
            this.m_deco.get().m_roofMeta,
            false,
        );
        this.fillMeta(
            world,
            sbb,
            1,
            12 + ((i - 1) * 3),
            dz,
            1,
            11 + (i * 3),
            dz + 1,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            false,
        );
        this.fillMeta(
            world,
            sbb,
            15,
            1,
            dz,
            15,
            12 + ((i - 1) * 3),
            dz,
            this.m_deco.get().m_roofID,
            this.m_deco.get().m_roofMeta,
            this.m_deco.get().m_roofID,
            this.m_deco.get().m_roofMeta,
            false,
        );
        this.fillMeta(
            world,
            sbb,
            15,
            12 + ((i - 1) * 3),
            dz,
            15,
            11 + (i * 3),
            dz + 1,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            false,
        );
    }
    this.fillMeta(
        world,
        sbb,
        1,
        18,
        8,
        5,
        18,
        8,
        this.m_deco.get().m_roofID,
        this.m_deco.get().m_roofMeta,
        this.m_deco.get().m_roofID,
        this.m_deco.get().m_roofMeta,
        false,
    );
    this.fillMeta(
        world,
        sbb,
        11,
        18,
        8,
        14,
        18,
        8,
        this.m_deco.get().m_roofID,
        this.m_deco.get().m_roofMeta,
        this.m_deco.get().m_roofID,
        this.m_deco.get().m_roofMeta,
        false,
    );
    this.fillMeta(
        world,
        sbb,
        0,
        17,
        8,
        1,
        19,
        8,
        this.m_deco.get().m_roofID,
        this.m_deco.get().m_roofMeta,
        this.m_deco.get().m_roofID,
        this.m_deco.get().m_roofMeta,
        false,
    );
    this.fillMeta(
        world,
        sbb,
        15,
        17,
        8,
        16,
        19,
        8,
        this.m_deco.get().m_roofID,
        this.m_deco.get().m_roofMeta,
        this.m_deco.get().m_roofID,
        this.m_deco.get().m_roofMeta,
        false,
    );
    for rotation in (1..4).step_by(2)
    {
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
    }
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
    }
    return true;
}
