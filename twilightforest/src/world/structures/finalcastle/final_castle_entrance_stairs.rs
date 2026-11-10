// draft from ComponentTFFinalCastleEntranceStairs.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(index: i32, x: i32, y: i32, z: i32, direction: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::FinalCastleEntranceStairs);
    init(&this, index, x, y, z, direction);
    Rc::new(this)
}

// ComponentTFFinalCastleEntranceStairs constructor
pub fn init(this: &Piece, index: i32, x: i32, y: i32, z: i32, direction: i32)
{
    this.setMode(direction);
    this.m_box.set(boxFor2(x, y, z, 0, -1, -5, 12, 0, 12, direction));
}

// ComponentTFFinalCastleEntranceStairs.func_74861_a
pub fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    this.m_deco.set((parent).m_deco.get());
}

// ComponentTFFinalCastleEntranceStairs.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    let size: i32 = 13;
    for x in 1..size
    {
        this.placeStairs(world, sbb, x, 1 - x, 5, 2);
        for z in 0..=x
        {
            if (z > 0) && (z <= (size / 2))
            {
                this.placeStairs(world, sbb, x, 1 - x, 5 - z, 2);
                this.placeStairs(world, sbb, x, 1 - x, 5 + z, 2);
            }
            if x <= (size / 2)
            {
                this.placeStairs(world, sbb, z, 1 - x, 5 - x, 1);
                this.placeStairs(world, sbb, z, 1 - x, 5 + x, 3);
            }
        }
    }
    this.fillDown(world, this.m_deco.get().m_blockID, this.m_deco.get().m_blockMeta, 0, 0, 5, sbb);
    return true;
}

// ComponentTFFinalCastleEntranceStairs.placeStairs
pub fn placeStairs(this: &Piece, world: &GenWorld, sbb: StructureBox, x: i32, y: i32, z: i32, stairMeta: i32)
{
    if this.getBlock(world, x, y, z, sbb).isReplaceable(world, x, y, z)
    {
        this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(stairMeta), x, y, z, sbb);
        this.fillDown(world, this.m_deco.get().m_blockID, this.m_deco.get().m_blockMeta, x, y - 1, z, sbb);
    }
}
