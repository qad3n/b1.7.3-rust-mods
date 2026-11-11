// draft from ComponentTFMazeDeadEndTrappedChest.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(i: i32, x: i32, y: i32, z: i32, rotation: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::MazeDeadEndTrappedChest);
    init(&this, i, x, y, z, rotation);
    Rc::new(this)
}

// ComponentTFMazeDeadEndTrappedChest constructor
pub(crate) fn init(this: &Piece, i: i32, x: i32, y: i32, z: i32, rotation: i32)
{
    maze_dead_end_chest::init(this, i, x, y, z, rotation);
}

// ComponentTFMazeDeadEndTrappedChest.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    maze_dead_end_chest::add(this, world, random, sbb);
    this.placeBlock(world, blk("tripwire_hook"), maze_dead_end_trapped_chest::hookMeta(this, 3), 1, 1, 2, sbb);
    this.placeBlock(world, blk("tripwire_hook"), maze_dead_end_trapped_chest::hookMeta(this, 1), 4, 1, 2, sbb);
    this.placeBlock(world, blk("tripwire"), 0, 2, 1, 2, sbb);
    this.placeBlock(world, blk("tripwire"), 0, 3, 1, 2, sbb);
    this.placeBlock(world, blk("tnt"), 0, 0, 0, 2, sbb);
    this.placeBlock(world, blk("air"), 0, 0, -1, 2, sbb);
    this.placeBlock(world, blk("air"), 0, 1, -1, 2, sbb);
    this.placeBlock(world, blk("tnt"), 0, 2, 0, 4, sbb);
    this.placeBlock(world, blk("tnt"), 0, 3, 0, 4, sbb);
    this.placeBlock(world, blk("tnt"), 0, 2, 0, 3, sbb);
    this.placeBlock(world, blk("tnt"), 0, 3, 0, 3, sbb);
    true
}

// ComponentTFMazeDeadEndTrappedChest.getHookMeta
pub(crate) fn hookMeta(this: &Piece, dir: i32) -> i32 { (this.mode() + dir) % 4 }
