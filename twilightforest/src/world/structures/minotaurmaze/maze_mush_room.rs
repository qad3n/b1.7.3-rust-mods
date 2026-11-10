// draft from ComponentTFMazeMushRoom.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, random: &Random, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::MazeMushRoom);
    init(&this, i, random, x, y, z);
    Rc::new(this)
}

// ComponentTFMazeMushRoom constructor
pub fn init(this: &Piece, i: i32, random: &Random, x: i32, y: i32, z: i32)
{
    maze_room::init(this, i, random, x, y, z);
    this.m_mode.set(0);
}

// ComponentTFMazeMushRoom.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    maze_room::add(this, world, random, sbb);
    for x in 1..14
    {
        for z in 1..14
        {
            let dist: i32 = (7.0
                / (((7.5 - (x) as f64) * (7.5 - (x) as f64)) + ((7.5 - (z) as f64) * (7.5 - (z) as f64))).sqrt())
            .round() as i32;
            if random.nextIntBounded(dist + 1) > 0
            {
                this.placeBlock(world, blk("mycelium"), 0, x, 0, z, sbb);
            }
            if random.nextIntBounded(dist) > 0
            {
                this.placeBlock(
                    world,
                    if random.nextBoolean()
                    {
                        blk("red_mushroom")
                    }
                    else
                    {
                        blk("brown_mushroom")
                    },
                    0,
                    x,
                    1,
                    z,
                    sbb,
                );
            }
        }
    }
    maze_mush_room::makeMediumMushroom(this, world, sbb, 5, 2, 9, blk("red_mushroom_block"));
    maze_mush_room::makeMediumMushroom(this, world, sbb, 5, 3, 9, blk("red_mushroom_block"));
    maze_mush_room::makeMediumMushroom(this, world, sbb, 9, 2, 5, blk("red_mushroom_block"));
    maze_mush_room::makeMediumMushroom(this, world, sbb, 6, 3, 4, blk("brown_mushroom_block"));
    maze_mush_room::makeMediumMushroom(this, world, sbb, 10, 1, 9, blk("brown_mushroom_block"));
    this.placeBlock(world, blk("red_mushroom_block"), 15, 1, 2, 1, sbb);
    this.placeBlock(world, blk("red_mushroom_block"), 5, 1, 3, 1, sbb);
    this.placeBlock(world, blk("red_mushroom_block"), 9, 2, 3, 1, sbb);
    this.placeBlock(world, blk("red_mushroom_block"), 9, 1, 3, 2, sbb);
    this.placeBlock(world, blk("brown_mushroom_block"), 15, 14, 3, 1, sbb);
    this.placeBlock(world, blk("brown_mushroom_block"), 5, 14, 4, 1, sbb);
    this.placeBlock(world, blk("brown_mushroom_block"), 7, 13, 4, 1, sbb);
    this.placeBlock(world, blk("brown_mushroom_block"), 7, 14, 4, 2, sbb);
    this.placeBlock(world, blk("brown_mushroom_block"), 15, 1, 1, 14, sbb);
    this.placeBlock(world, blk("brown_mushroom_block"), 5, 1, 2, 14, sbb);
    this.placeBlock(world, blk("brown_mushroom_block"), 3, 2, 2, 14, sbb);
    this.placeBlock(world, blk("brown_mushroom_block"), 3, 1, 2, 13, sbb);
    this.placeBlock(world, blk("brown_mushroom_block"), 5, 14, 1, 14, sbb);
    this.placeBlock(world, blk("brown_mushroom_block"), 1, 13, 1, 14, sbb);
    this.placeBlock(world, blk("brown_mushroom_block"), 1, 14, 1, 13, sbb);
    return true;
}

// ComponentTFMazeMushRoom.makeMediumMushroom
pub fn makeMediumMushroom(
    this: &Piece,
    world: &GenWorld,
    sbb: StructureBox,
    mx: i32,
    my: i32,
    mz: i32,
    redMushroomBlock: Blk,
)
{
    this.placeBlock(world, redMushroomBlock, 5, mx + 0, my, mz + 0, sbb);
    this.placeBlock(world, redMushroomBlock, 6, mx + 1, my, mz + 0, sbb);
    this.placeBlock(world, redMushroomBlock, 9, mx + 1, my, mz + 1, sbb);
    this.placeBlock(world, redMushroomBlock, 8, mx + 0, my, mz + 1, sbb);
    this.placeBlock(world, redMushroomBlock, 7, mx - 1, my, mz + 1, sbb);
    this.placeBlock(world, redMushroomBlock, 4, mx - 1, my, mz + 0, sbb);
    this.placeBlock(world, redMushroomBlock, 1, mx - 1, my, mz - 1, sbb);
    this.placeBlock(world, redMushroomBlock, 2, mx + 0, my, mz - 1, sbb);
    this.placeBlock(world, redMushroomBlock, 3, mx + 1, my, mz - 1, sbb);
    for y in 1..my
    {
        this.placeBlock(world, redMushroomBlock, 10, mx + 0, y, mz + 0, sbb);
    }
}
