// draft from ComponentTFMazeRoomVault.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, random: &Random, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::MazeRoomVault);
    init(&this, i, random, x, y, z);
    Rc::new(this)
}

// ComponentTFMazeRoomVault constructor
pub fn init(this: &Piece, i: i32, random: &Random, x: i32, y: i32, z: i32)
{
    maze_room::init(this, i, random, x, y, z);
    this.m_spawnListIndex.set(i32::MAX);
}

// ComponentTFMazeRoomVault.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    this.fillMeta(world, sbb, 0, 1, 0, 15, 4, 15, blk("tf:mazestone"), 3, blk("air"), 0, false);
    this.fillMeta(world, sbb, 0, 2, 0, 15, 3, 15, blk("tf:mazestone"), 1, blk("air"), 0, false);
    this.fillAir(world, sbb, 6, 2, 6, 9, 3, 9);
    this.fillMeta(world, sbb, 6, 2, 5, 9, 2, 5, blk("wooden_pressure_plate"), 0, blk("air"), 0, false);
    this.fillMeta(world, sbb, 6, 2, 10, 9, 2, 10, blk("wooden_pressure_plate"), 0, blk("air"), 0, false);
    this.fillMeta(world, sbb, 5, 2, 6, 5, 2, 9, blk("wooden_pressure_plate"), 0, blk("air"), 0, false);
    this.fillMeta(world, sbb, 10, 2, 6, 10, 2, 9, blk("wooden_pressure_plate"), 0, blk("air"), 0, false);
    this.fillMeta(world, sbb, 6, 4, 5, 9, 4, 5, blk("sand"), 0, blk("air"), 0, false);
    this.fillMeta(world, sbb, 6, 4, 10, 9, 4, 10, blk("sand"), 0, blk("air"), 0, false);
    this.fillMeta(world, sbb, 5, 4, 6, 5, 4, 9, blk("sand"), 0, blk("air"), 0, false);
    this.fillMeta(world, sbb, 10, 4, 6, 10, 4, 9, blk("sand"), 0, blk("air"), 0, false);
    this.fillMeta(world, sbb, 6, 0, 5, 9, 0, 5, blk("tnt"), 0, blk("air"), 0, false);
    this.fillMeta(world, sbb, 6, 0, 10, 9, 0, 10, blk("tnt"), 0, blk("air"), 0, false);
    this.fillMeta(world, sbb, 5, 0, 6, 5, 0, 9, blk("tnt"), 0, blk("air"), 0, false);
    this.fillMeta(world, sbb, 10, 0, 6, 10, 0, 9, blk("tnt"), 0, blk("air"), 0, false);
    this.placeBlock(world, blk("chest"), 0, 7, 2, 6, sbb);
    this.placeTreasure(world, random, 8, 2, 6, Table::LabyrinthVault, sbb);
    this.placeBlock(world, blk("chest"), 0, 8, 2, 9, sbb);
    this.placeTreasure(world, random, 7, 2, 9, Table::LabyrinthVault, sbb);
    this.placeBlock(world, blk("chest"), 0, 6, 2, 7, sbb);
    this.placeTreasure(world, random, 6, 2, 8, Table::LabyrinthVault, sbb);
    this.placeBlock(world, blk("chest"), 0, 9, 2, 8, sbb);
    this.placeTreasure(world, random, 9, 2, 7, Table::LabyrinthVault, sbb);
    return true;
}
