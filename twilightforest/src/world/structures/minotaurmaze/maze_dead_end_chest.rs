// draft from ComponentTFMazeDeadEndChest.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(i: i32, x: i32, y: i32, z: i32, rotation: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::MazeDeadEndChest);
    init(&this, i, x, y, z, rotation);
    Rc::new(this)
}

// ComponentTFMazeDeadEndChest constructor
pub(crate) fn init(this: &Piece, i: i32, x: i32, y: i32, z: i32, rotation: i32)
{
    maze_dead_end::init(this, i, x, y, z, rotation);
    this.m_spawnListIndex.set(i32::MAX);
}

// ComponentTFMazeDeadEndChest.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    this.placeBlock(world, blk("planks"), 0, 2, 1, 4, sbb);
    this.placeBlock(world, blk("planks"), 0, 3, 1, 4, sbb);
    this.placeBlock(world, blk("oak_stairs"), this.stairMeta(1), 2, 1, 3, sbb);
    this.placeBlock(world, blk("oak_stairs"), this.stairMeta(1), 3, 1, 3, sbb);
    this.placeBlock(world, blk("chest"), 0, 2, 2, 4, sbb);
    this.placeTreasure(world, random, 3, 2, 4, Table::LabyrinthDeadEnd, sbb);
    this.fillMeta(world, sbb, 1, 1, 0, 4, 3, 1, blk("tf:mazestone"), 2, blk("air"), 0, false);
    this.fillMeta(world, sbb, 1, 4, 0, 4, 4, 1, blk("tf:mazestone"), 3, blk("air"), 0, false);
    this.fill(world, sbb, 2, 1, 0, 3, 3, 1, blk("iron_bars"), blk("air"), false);
    true
}
