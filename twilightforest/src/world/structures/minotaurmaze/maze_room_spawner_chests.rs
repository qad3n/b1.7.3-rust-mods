// draft from ComponentTFMazeRoomSpawnerChests.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, random: &Random, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::MazeRoomSpawnerChests);
    init(&this, i, random, x, y, z);
    Rc::new(this)
}

// ComponentTFMazeRoomSpawnerChests constructor
pub fn init(this: &Piece, i: i32, random: &Random, x: i32, y: i32, z: i32)
{
    maze_room::init(this, i, random, x, y, z);
}

// ComponentTFMazeRoomSpawnerChests.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    maze_room::add(this, world, random, sbb);
    maze_room_spawner_chests::placePillarEnclosure(this, world, sbb, 3, 3);
    maze_room_spawner_chests::placePillarEnclosure(this, world, sbb, 10, 3);
    maze_room_spawner_chests::placePillarEnclosure(this, world, sbb, 3, 10);
    maze_room_spawner_chests::placePillarEnclosure(this, world, sbb, 10, 10);
    this.placeSpawner(world, random, 4, 2, 4, mob("Minotaur"), sbb);
    this.placeTreasure(world, random, 4, 2, 11, Table::LabyrinthRoom, sbb);
    this.placeTreasure(world, random, 11, 2, 4, Table::LabyrinthRoom, sbb);
    this.placeBlock(world, blk("wooden_pressure_plate"), 0, 11, 1, 11, sbb);
    this.placeBlock(world, blk("tnt"), 0, 10, 0, 11, sbb);
    this.placeBlock(world, blk("tnt"), 0, 11, 0, 10, sbb);
    this.placeBlock(world, blk("tnt"), 0, 11, 0, 12, sbb);
    this.placeBlock(world, blk("tnt"), 0, 12, 0, 11, sbb);
    return true;
}

// ComponentTFMazeRoomSpawnerChests.placePillarEnclosure
pub fn placePillarEnclosure(this: &Piece, world: &GenWorld, sbb: StructureBox, dx: i32, dz: i32)
{
    for y in 1..5
    {
        this.placeBlock(world, blk("tf:mazestone"), 2, dx + 0, y, dz + 0, sbb);
        this.placeBlock(world, blk("tf:mazestone"), 2, dx + 2, y, dz + 0, sbb);
        this.placeBlock(world, blk("tf:mazestone"), 2, dx + 0, y, dz + 2, sbb);
        this.placeBlock(world, blk("tf:mazestone"), 2, dx + 2, y, dz + 2, sbb);
    }
    this.placeBlock(world, blk("planks"), 0, dx + 1, 1, dz + 1, sbb);
    this.placeBlock(world, blk("planks"), 0, dx + 1, 4, dz + 1, sbb);
    this.placeBlock(world, blk("oak_stairs"), this.stairMeta(1), dx + 1, 1, dz + 0, sbb);
    this.placeBlock(world, blk("oak_stairs"), this.stairMeta(0), dx + 0, 1, dz + 1, sbb);
    this.placeBlock(world, blk("oak_stairs"), this.stairMeta(2), dx + 2, 1, dz + 1, sbb);
    this.placeBlock(world, blk("oak_stairs"), this.stairMeta(3), dx + 1, 1, dz + 2, sbb);
    this.placeBlock(world, blk("oak_stairs"), this.stairMeta(1) + 4, dx + 1, 4, dz + 0, sbb);
    this.placeBlock(world, blk("oak_stairs"), this.stairMeta(0) + 4, dx + 0, 4, dz + 1, sbb);
    this.placeBlock(world, blk("oak_stairs"), this.stairMeta(2) + 4, dx + 2, 4, dz + 1, sbb);
    this.placeBlock(world, blk("oak_stairs"), this.stairMeta(3) + 4, dx + 1, 4, dz + 2, sbb);
    this.placeBlock(world, blk("iron_bars"), 0, dx + 1, 2, dz + 0, sbb);
    this.placeBlock(world, blk("iron_bars"), 0, dx + 0, 2, dz + 1, sbb);
    this.placeBlock(world, blk("iron_bars"), 0, dx + 2, 2, dz + 1, sbb);
    this.placeBlock(world, blk("iron_bars"), 0, dx + 1, 2, dz + 2, sbb);
    this.placeBlock(world, blk("iron_bars"), 0, dx + 1, 3, dz + 0, sbb);
    this.placeBlock(world, blk("iron_bars"), 0, dx + 0, 3, dz + 1, sbb);
    this.placeBlock(world, blk("iron_bars"), 0, dx + 2, 3, dz + 1, sbb);
    this.placeBlock(world, blk("iron_bars"), 0, dx + 1, 3, dz + 2, sbb);
}
