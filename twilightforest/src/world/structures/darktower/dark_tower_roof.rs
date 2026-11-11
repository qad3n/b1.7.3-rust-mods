// draft from ComponentTFDarkTowerRoof.java by tools/structgen.py
use crate::world::structures::prelude::*;

// ComponentTFDarkTowerRoof constructor
pub fn init(this: &Piece, i: i32, wing: &Piece)
{
    tower_roof::init(this, i, &wing);
    this.setMode(wing.mode());
    this.m_size.set(wing.m_size.get());
    this.m_height.set(12);
    tower_roof::makeCapBB(this, &wing);
    this.m_spawnListIndex.set(1);
}

// ComponentTFDarkTowerRoof.func_74861_a
pub fn build(this: &Piece, parent: &Piece, _list: &mut Pieces, _random: &Random)
{
    this.m_deco.set((parent).m_deco.get());
}

// ComponentTFDarkTowerRoof.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, _random: &Random, sbb: StructureBox) -> bool
{
    for x in 0..=(this.m_size.get() - 1)
    {
        for z in 0..=(this.m_size.get() - 1)
        {
            if (((x == 0) || (x == (this.m_size.get() - 1))) || (z == 0)) || (z == (this.m_size.get() - 1))
            {
                this.placeBlock(world, this.m_deco.get().m_fenceID, this.m_deco.get().m_fenceMeta, x, 1, z, sbb);
            }
        }
    }
    this.placeBlock(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, 0, 1, 0, sbb);
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        this.m_size.get() - 1,
        1,
        0,
        sbb,
    );
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        0,
        1,
        this.m_size.get() - 1,
        sbb,
    );
    this.placeBlock(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        this.m_size.get() - 1,
        1,
        this.m_size.get() - 1,
        sbb,
    );
    return true;
}
