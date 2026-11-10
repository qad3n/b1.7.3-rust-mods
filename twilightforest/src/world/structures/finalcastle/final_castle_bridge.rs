// draft from ComponentTFFinalCastleBridge.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, x: i32, y: i32, z: i32, length: i32, direction: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::FinalCastleBridge);
    init(&this, i, x, y, z, length, direction);
    Rc::new(this)
}

// ComponentTFFinalCastleBridge constructor
pub fn init(this: &Piece, i: i32, x: i32, y: i32, z: i32, length: i32, direction: i32)
{
    this.m_depth.set(i);
    this.setMode(direction);
    this.m_box.set(boxFor2(x, y, z, 0, -1, -3, length - 1, 5, 6, direction));
}

// ComponentTFFinalCastleBridge.func_74861_a
pub fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    this.m_deco.set((parent).m_deco.get());
}

// ComponentTFFinalCastleBridge.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    let length: i32 = if (this.m_mode.get() != 0) && (this.m_mode.get() != 2)
    {
        this.m_box.get().m_max[2] - this.m_box.get().m_min[2]
    }
    else
    {
        this.m_box.get().m_max[0] - this.m_box.get().m_min[0]
    };
    this.fillSelected(world, sbb, 0, 0, 0, length, 1, 6, false, random, this.m_deco.get().m_randomBlocks);
    this.fillSelected(world, sbb, 0, 1, 0, length, 2, 0, false, random, this.m_deco.get().m_randomBlocks);
    this.fillSelected(world, sbb, 0, 1, 6, length, 2, 6, false, random, this.m_deco.get().m_randomBlocks);
    let l3: i32 = length / 3;
    for i in 0..l3
    {
        let sl: i32 = l3 - ((cos((((l3 - i) as f32) / (l3) as f32) * 1.6) * (l3) as f32) as i32);
        this.fillSelected(world, sbb, i, -sl, 0, i, 0, 0, false, random, this.m_deco.get().m_randomBlocks);
        this.fillSelected(world, sbb, i, -sl, 6, i, 0, 6, false, random, this.m_deco.get().m_randomBlocks);
        this.fillSelected(
            world,
            sbb,
            length - i,
            -sl,
            0,
            length - i,
            0,
            0,
            false,
            random,
            this.m_deco.get().m_randomBlocks,
        );
        this.fillSelected(
            world,
            sbb,
            length - i,
            -sl,
            6,
            length - i,
            0,
            6,
            false,
            random,
            this.m_deco.get().m_randomBlocks,
        );
    }
    this.fillMeta(
        world,
        sbb,
        0,
        2,
        1,
        0,
        7,
        1,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta,
        false,
    );
    this.fillMeta(
        world,
        sbb,
        0,
        2,
        5,
        0,
        7,
        5,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta,
        false,
    );
    this.fillMeta(
        world,
        sbb,
        0,
        6,
        2,
        0,
        6,
        4,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        false,
    );
    this.placeBlock(world, this.m_deco.get().m_pillarID, this.m_deco.get().m_pillarMeta, 0, 7, 3, sbb);
    this.fillMeta(
        world,
        sbb,
        length,
        2,
        1,
        length,
        7,
        1,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta,
        false,
    );
    this.fillMeta(
        world,
        sbb,
        length,
        2,
        5,
        length,
        7,
        5,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta,
        false,
    );
    this.fillMeta(
        world,
        sbb,
        length,
        6,
        2,
        length,
        6,
        4,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        false,
    );
    this.placeBlock(world, this.m_deco.get().m_pillarID, this.m_deco.get().m_pillarMeta, length, 7, 3, sbb);
    return true;
}
