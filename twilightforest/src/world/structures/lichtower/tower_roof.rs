// draft from ComponentTFTowerRoof.java by tools/structgen.py
use crate::world::structures::prelude::*;

// ComponentTFTowerRoof constructor
pub fn init(this: &Piece, i: i32, wing: &Piece)
{
    this.m_depth.set(i);
    this.m_spawnListIndex.set(-1);
}

// ComponentTFTowerRoof.makeAttachedOverhangBB
pub fn makeAttachedOverhangBB(this: &Piece, wing: &Piece)
{
    match this.mode()
    {
        0 =>
        {
            this.m_box.set(StructureBox::new(
                [wing.bounds().m_min[0], wing.bounds().m_max[1], wing.bounds().m_min[2] - 1],
                [
                    wing.bounds().m_max[0] + 1,
                    (wing.bounds().m_max[1] + this.m_height.get()) - 1,
                    wing.bounds().m_max[2] + 1,
                ],
            ));
        }
        1 =>
        {
            this.m_box.set(StructureBox::new(
                [wing.bounds().m_min[0] - 1, wing.bounds().m_max[1], wing.bounds().m_min[2]],
                [
                    wing.bounds().m_max[0] + 1,
                    (wing.bounds().m_max[1] + this.m_height.get()) - 1,
                    wing.bounds().m_max[2] + 1,
                ],
            ));
        }
        2 =>
        {
            this.m_box.set(StructureBox::new(
                [wing.bounds().m_min[0] - 1, wing.bounds().m_max[1], wing.bounds().m_min[2] - 1],
                [
                    wing.bounds().m_max[0],
                    (wing.bounds().m_max[1] + this.m_height.get()) - 1,
                    wing.bounds().m_max[2] + 1,
                ],
            ));
        }
        3 =>
        {
            this.m_box.set(StructureBox::new(
                [wing.bounds().m_min[0] - 1, wing.bounds().m_max[1], wing.bounds().m_min[2] - 1],
                [
                    wing.bounds().m_max[0] + 1,
                    (wing.bounds().m_max[1] + this.m_height.get()) - 1,
                    wing.bounds().m_max[2],
                ],
            ));
        }
        _ =>
        {}
    }
}

// ComponentTFTowerRoof.makeCapBB
pub fn makeCapBB(this: &Piece, wing: &Piece)
{
    this.m_box.set(StructureBox::new(
        [wing.bounds().m_min[0], wing.bounds().m_max[1], wing.bounds().m_min[2]],
        [wing.bounds().m_max[0], wing.bounds().m_max[1] + this.m_height.get(), wing.bounds().m_max[2]],
    ));
}

// ComponentTFTowerRoof.makeOverhangBB
pub fn makeOverhangBB(this: &Piece, wing: &Piece)
{
    this.m_box.set(StructureBox::new(
        [wing.bounds().m_min[0] - 1, wing.bounds().m_max[1], wing.bounds().m_min[2] - 1],
        [wing.bounds().m_max[0] + 1, (wing.bounds().m_max[1] + this.m_height.get()) - 1, wing.bounds().m_max[2] + 1],
    ));
}

// ComponentTFTowerRoof.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool { return false; }

// ComponentTFTowerRoof.fits
pub fn fits(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random) -> bool
{
    return same(&intersecting(list, this.m_box.get()), &parent);
}
