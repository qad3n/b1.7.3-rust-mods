// draft from ComponentTFTowerRoof.java by tools/structgen.py
use crate::world::structures::prelude::*;

// ComponentTFTowerRoof constructor
pub(crate) fn init(this: &Piece, i: i32, _wing: &Piece)
{
    this.m_depth.set(i);
    this.m_spawnListIndex.set(-1);
}

// ComponentTFTowerRoof.makeAttachedOverhangBB
pub(crate) fn makeAttachedOverhangBB(this: &Piece, wing: &Piece)
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
pub(crate) fn makeCapBB(this: &Piece, wing: &Piece)
{
    this.m_box.set(StructureBox::new(
        [wing.bounds().m_min[0], wing.bounds().m_max[1], wing.bounds().m_min[2]],
        [wing.bounds().m_max[0], wing.bounds().m_max[1] + this.m_height.get(), wing.bounds().m_max[2]],
    ));
}

// ComponentTFTowerRoof.makeOverhangBB
pub(crate) fn makeOverhangBB(this: &Piece, wing: &Piece)
{
    this.m_box.set(StructureBox::new(
        [wing.bounds().m_min[0] - 1, wing.bounds().m_max[1], wing.bounds().m_min[2] - 1],
        [wing.bounds().m_max[0] + 1, (wing.bounds().m_max[1] + this.m_height.get()) - 1, wing.bounds().m_max[2] + 1],
    ));
}

// ComponentTFTowerRoof.func_74875_a
pub(crate) fn add(_this: &Piece, _world: &GenWorld, _random: &Random, _sbb: StructureBox) -> bool { false }

// ComponentTFTowerRoof.fits
pub(crate) fn fits(this: &Piece, parent: &Piece, list: &mut Pieces, _random: &Random) -> bool
{
    same(&intersecting(list, this.m_box.get()), &parent)
}
