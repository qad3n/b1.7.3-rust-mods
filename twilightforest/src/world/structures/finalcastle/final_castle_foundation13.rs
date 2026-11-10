// draft from ComponentTFFinalCastleFoundation13.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(random: &Random, i: i32, sideTower: &Piece) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::FinalCastleFoundation13);
    init(&this, random, i, sideTower);
    Rc::new(this)
}

// ComponentTFFinalCastleFoundation13 constructor
pub fn init(this: &Piece, random: &Random, i: i32, sideTower: &Piece)
{
    this.m_depth.set(i);
    this.m_groundLevel.set(-1);
    this.setMode(sideTower.mode());
    this.m_box.set(StructureBox::new(
        [sideTower.bounds().m_min[0] - 2, sideTower.bounds().m_min[1] - 1, sideTower.bounds().m_min[2] - 2],
        [sideTower.bounds().m_max[0] + 2, sideTower.bounds().m_min[1], sideTower.bounds().m_max[2] + 2],
    ));
}

// ComponentTFFinalCastleFoundation13.func_74861_a
pub fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    this.m_deco.set((parent).m_deco.get());
}

// ComponentTFFinalCastleFoundation13.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    if this.m_groundLevel.get() < 0
    {
        this.m_groundLevel.set(final_castle_foundation13::getDeadrockLevel(this, world, sbb));
        if this.m_groundLevel.get() < 0
        {
            return true;
        }
    }
    let height: i32 = this.m_box.get().m_max[1] - this.m_groundLevel.get();
    let mid: i32 = height / 2;
    let size: i32 = this.m_box.get().m_max[0] - this.m_box.get().m_min[0];
    for rotation in 0..4
    {
        this.fillDownTurned(world, this.m_deco.get().m_blockID, this.m_deco.get().m_blockMeta, 1, -1, 1, rotation, sbb);
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
    return true;
}

// ComponentTFFinalCastleFoundation13.getDeadrockLevel
pub fn getDeadrockLevel(this: &Piece, world: &GenWorld, sbb: StructureBox) -> i32
{
    let mut groundLevel: i32 = 256;
    for y in (0 + 1..=150).rev()
    {
        let cx: i32 = sbb.centerX();
        let cz: i32 = sbb.centerZ();
        let block: Blk = world.getBlock(cx, y, cz);
        if block == blk("tf:deadrock")
        {
            groundLevel = y;
            break;
        }
    }
    return groundLevel;
}
