// draft from ComponentTFMazeDeadEndShrooms.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, x: i32, y: i32, z: i32, rotation: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::MazeDeadEndShrooms);
    init(&this, i, x, y, z, rotation);
    Rc::new(this)
}

// ComponentTFMazeDeadEndShrooms constructor
pub fn init(this: &Piece, i: i32, x: i32, y: i32, z: i32, rotation: i32)
{
    maze_dead_end_roots::init(this, i, x, y, z, rotation);
}

// ComponentTFMazeDeadEndShrooms.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    for x in 1..5
    {
        for z in 0..5
        {
            if random.nextIntBounded(z + 2) > 0
            {
                this.placeBlock(world, blk("mycelium"), 0, x, 0, z, sbb);
            }
            if random.nextIntBounded(z + 2) > 0
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
    let mut mushType: Blk = if random.nextBoolean()
    {
        blk("red_mushroom_block")
    }
    else
    {
        blk("brown_mushroom_block")
    };
    let mut mushY: i32 = random.nextIntBounded(4) + 1;
    let mut mushZ: i32 = random.nextIntBounded(3) + 1;
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
    mushZ = random.nextIntBounded(3) + 1;
    this.fillMeta(world, sbb, 4, 1, mushZ, 4, mushY, mushZ, mushType, 10, blk("air"), 0, false);
    this.fillMeta(world, sbb, 3, mushY, mushZ - 1, 4, mushY, mushZ + 1, mushType, 14, blk("air"), 0, false);
    mushType = if random.nextBoolean()
    {
        blk("red_mushroom_block")
    }
    else
    {
        blk("brown_mushroom_block")
    };
    mushY = random.nextIntBounded(4) + 1;
    let mushX: i32 = random.nextIntBounded(3) + 2;
    this.fillMeta(world, sbb, mushX, 1, 4, mushX, mushY, 4, mushType, 10, blk("air"), 0, false);
    this.fillMeta(world, sbb, mushX - 1, mushY, 3, mushX + 1, mushY, 4, mushType, 14, blk("air"), 0, false);
    return true;
}
