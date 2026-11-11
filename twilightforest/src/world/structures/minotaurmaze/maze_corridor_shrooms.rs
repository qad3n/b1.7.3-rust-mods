// draft from ComponentTFMazeCorridorShrooms.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(i: i32, x: i32, y: i32, z: i32, rotation: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::MazeCorridorShrooms);
    init(&this, i, x, y, z, rotation);
    Rc::new(this)
}

// ComponentTFMazeCorridorShrooms constructor
pub(crate) fn init(this: &Piece, i: i32, x: i32, y: i32, z: i32, rotation: i32)
{
    maze_corridor::init(this, i, x, y, z, rotation);
}

// ComponentTFMazeCorridorShrooms.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    for x in 1..5
    {
        for z in 0..5
        {
            if random.nextIntBounded(2) > 0
            {
                this.placeBlock(world, blk("mycelium"), 0, x, 0, z, sbb);
            }

            if random.nextIntBounded(2) > 0
            {
                let block: Blk = if random.nextBoolean()
                {
                    blk("red_mushroom")
                }
                else
                {
                    blk("brown_mushroom")
                };
                this.placeBlock(world, block, 0, x, 1, z, sbb);
            }
        }
    }

    let mut mushType: Blk = if random.nextBoolean()
    {
        blk("red_mushroom_block")
    }
    else
    {
        blk("brown_mushroom_block")
    };
    let mut mushY: i32 = random.nextIntBounded(4) + 1;
    let mut mushZ: i32 = random.nextIntBounded(4) + 1;
    this.placeBlock(world, mushType, 15, 1, mushY - 1, mushZ, sbb);
    this.fillMeta(world, sbb, 1, 1, mushZ, 1, mushY, mushZ, mushType, 10, blk("air"), 0, false);
    this.fillMeta(world, sbb, 1, mushY, mushZ - 1, 2, mushY, mushZ + 1, mushType, 14, blk("air"), 0, false);

    mushType = if mushType == blk("brown_mushroom_block")
    {
        blk("red_mushroom_block")
    }
    else
    {
        blk("brown_mushroom_block")
    };
    mushY = random.nextIntBounded(4) + 1;
    mushZ = random.nextIntBounded(4) + 1;
    this.fillMeta(world, sbb, 4, 1, mushZ, 4, mushY, mushZ, mushType, 10, blk("air"), 0, false);
    this.fillMeta(world, sbb, 3, mushY, mushZ - 1, 4, mushY, mushZ + 1, mushType, 14, blk("air"), 0, false);
    true
}
