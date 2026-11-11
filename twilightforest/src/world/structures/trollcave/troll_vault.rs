// draft from ComponentTFTrollVault.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(index: i32, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::TrollVault);
    init(&this, index, x, y, z);
    Rc::new(this)
}

// ComponentTFTrollVault constructor
pub(crate) fn init(this: &Piece, index: i32, mut x: i32, mut y: i32, mut z: i32)
{
    this.m_depth.set(index);
    this.setMode(0);
    x = (x >> 2) << 2;
    y = (y / 4) * 4;
    z = (z >> 2) << 2;
    this.m_spawnListIndex.set(-1);
    this.m_box.set(boxFor(x, y, z, -8, 0, -8, 12, 12, 12, 0));
}

// ComponentTFTrollVault.func_74861_a
pub(crate) fn build(_this: &Piece, _parent: &Piece, _list: &mut Pieces, _random: &Random) {}

// ComponentTFTrollVault.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    this.fillMeta(world, sbb, 0, 0, 0, 11, 11, 11, blk("tf:giantObsidian"), 0, blk("tf:giantObsidian"), 0, false);
    this.fillAir(world, sbb, 4, 4, 4, 7, 7, 7);
    this.fillMeta(world, sbb, 5, 5, 5, 6, 5, 6, blk("cobblestone"), 0, blk("cobblestone"), 0, false);
    this.placeBlock(world, blk("chest"), 0, 5, 6, 5, sbb);
    this.placeTreasureTrapped(world, random, 5, 6, 6, Table::TrollVault, false, sbb);
    this.placeTreasureTrapped(world, random, 6, 6, 5, Table::TrollGarden, true, sbb);
    this.placeBlock(world, blk("trapped_chest"), 0, 6, 6, 6, sbb);
    true
}
