// draft from ComponentTFTowerBridge.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, x: i32, y: i32, z: i32, pSize: i32, pHeight: i32, direction: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::TowerBridge);
    init(&this, i, x, y, z, pSize, pHeight, direction);
    Rc::new(this)
}

// ComponentTFTowerBridge constructor
pub fn init(this: &Piece, i: i32, x: i32, y: i32, z: i32, pSize: i32, pHeight: i32, direction: i32)
{
    tower_wing::init7(this, i, x, y, z, 3, 3, direction);
    this.m_dSize.set(pSize);
    this.m_dHeight.set(pHeight);
}

// ComponentTFTowerBridge.func_74861_a
pub fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    let dest: Vec<i32> = vec![2, 1, 1];
    this.makeTowerWing(list, random, 1, dest[0], dest[1], dest[2], this.m_dSize.get(), this.m_dHeight.get(), 0);
}

// ComponentTFTowerBridge.getWingBB
pub fn getWingBB(this: &Piece) -> StructureBox
{
    let dest: Vec<i32> = this.offsetTower(2, 1, 1, this.m_dSize.get(), this.mode());
    return boxFor(
        dest[0],
        dest[1],
        dest[2],
        0,
        0,
        0,
        this.m_dSize.get() - 1,
        this.m_dHeight.get() - 1,
        this.m_dSize.get() - 1,
        this.mode(),
    );
}

// ComponentTFTowerBridge.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    for x in 0..3
    {
        this.placeBlock(world, blk("fence"), 0, x, 2, 0, sbb);
        this.placeBlock(world, blk("fence"), 0, x, 2, 2, sbb);
        this.placeBlock(world, blk("stonebrick"), 0, x, 1, 0, sbb);
        this.placeBlock(world, blk("stonebrick"), 0, x, 1, 2, sbb);
        this.placeBlock(world, blk("stonebrick"), 0, x, 0, 0, sbb);
        this.placeBlock(world, blk("stonebrick"), 0, x, 0, 1, sbb);
        this.placeBlock(world, blk("stonebrick"), 0, x, 0, 2, sbb);
        this.placeBlock(world, blk("stonebrick"), 0, x, -1, 1, sbb);
    }
    this.placeBlock(world, blk("stonebrick"), 0, -1, -1, 1, sbb);
    this.placeBlock(world, blk("stonebrick"), 0, 3, -1, 1, sbb);
    this.fillAir(world, sbb, 0, 1, 1, 2, 2, 1);
    return true;
}
