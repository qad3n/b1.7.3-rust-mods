// draft from ComponentTFMazeRoomBoss.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(i: i32, random: &Random, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::MazeRoomBoss);
    init(&this, i, random, x, y, z);
    Rc::new(this)
}

// ComponentTFMazeRoomBoss constructor
pub(crate) fn init(this: &Piece, i: i32, random: &Random, x: i32, y: i32, z: i32)
{
    maze_room::init(this, i, random, x, y, z);
    this.m_taurPlaced.set(false);
}

// ComponentTFMazeRoomBoss.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    if this.getBlock(world, 7, 1, 0, sbb) == blk("air")
    {
        this.fill(world, sbb, 6, 1, 0, 9, 4, 0, blk("fence"), blk("air"), false);
    }

    if this.getBlock(world, 7, 1, 15, sbb) == blk("air")
    {
        this.fill(world, sbb, 6, 1, 15, 9, 4, 15, blk("fence"), blk("air"), false);
    }

    if this.getBlock(world, 0, 1, 7, sbb) == blk("air")
    {
        this.fill(world, sbb, 0, 1, 6, 0, 4, 9, blk("fence"), blk("air"), false);
    }

    if this.getBlock(world, 15, 1, 7, sbb) == blk("air")
    {
        this.fill(world, sbb, 15, 1, 6, 15, 4, 9, blk("fence"), blk("air"), false);
    }

    for x in 1..14
    {
        for z in 1..14
        {
            let dist: i32 = (7.0
                / (((7.5 - (x) as f64) * (7.5 - (x) as f64)) + ((7.5 - (z) as f64) * (7.5 - (z) as f64))).sqrt())
            .round() as i32;

            let mycelium: bool = random.nextIntBounded(dist + 1) > 0;
            let mushroom: bool = random.nextIntBounded(dist) > 0;
            let mushRed: bool = random.nextBoolean();

            if mycelium
            {
                this.placeBlock(world, blk("mycelium"), 0, x, 0, z, sbb);
            }

            if mushroom
            {
                let block: Blk = if mushRed
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

    this.fillMeta(world, sbb, 1, 1, 1, 3, 1, 3, blk("red_mushroom_block"), 14, blk("air"), 0, false);
    this.fillMeta(world, sbb, 1, 2, 1, 1, 3, 4, blk("red_mushroom_block"), 14, blk("air"), 0, false);
    this.fillMeta(world, sbb, 2, 2, 1, 4, 3, 1, blk("red_mushroom_block"), 14, blk("air"), 0, false);
    this.fillMeta(world, sbb, 1, 4, 1, 3, 4, 3, blk("red_mushroom_block"), 14, blk("air"), 0, false);
    this.placeTreasure(world, random, 3, 2, 3, Table::LabyrinthRoom, sbb);
    this.fillMeta(world, sbb, 12, 1, 12, 14, 1, 14, blk("red_mushroom_block"), 14, blk("air"), 0, false);
    this.fillMeta(world, sbb, 14, 2, 11, 14, 3, 14, blk("red_mushroom_block"), 14, blk("air"), 0, false);
    this.fillMeta(world, sbb, 11, 2, 14, 14, 3, 14, blk("red_mushroom_block"), 14, blk("air"), 0, false);
    this.fillMeta(world, sbb, 12, 4, 12, 14, 4, 14, blk("red_mushroom_block"), 14, blk("air"), 0, false);
    this.placeTreasure(world, random, 12, 2, 12, Table::LabyrinthRoom, sbb);
    this.fillMeta(world, sbb, 1, 1, 12, 3, 1, 14, blk("brown_mushroom_block"), 14, blk("air"), 0, false);
    this.fillMeta(world, sbb, 1, 2, 11, 1, 3, 14, blk("brown_mushroom_block"), 14, blk("air"), 0, false);
    this.fillMeta(world, sbb, 2, 2, 14, 4, 3, 14, blk("brown_mushroom_block"), 14, blk("air"), 0, false);
    this.fillMeta(world, sbb, 1, 4, 12, 3, 4, 14, blk("brown_mushroom_block"), 14, blk("air"), 0, false);
    this.placeTreasure(world, random, 3, 2, 12, Table::LabyrinthRoom, sbb);
    this.fillMeta(world, sbb, 12, 1, 1, 14, 1, 3, blk("brown_mushroom_block"), 14, blk("air"), 0, false);
    this.fillMeta(world, sbb, 11, 2, 1, 14, 3, 1, blk("brown_mushroom_block"), 14, blk("air"), 0, false);
    this.fillMeta(world, sbb, 14, 2, 2, 14, 3, 4, blk("brown_mushroom_block"), 14, blk("air"), 0, false);
    this.fillMeta(world, sbb, 12, 4, 1, 14, 4, 3, blk("brown_mushroom_block"), 14, blk("air"), 0, false);
    this.placeTreasure(world, random, 12, 2, 3, Table::LabyrinthRoom, sbb);
    this.fillMeta(world, sbb, 5, 4, 5, 7, 5, 7, blk("brown_mushroom_block"), 14, blk("air"), 0, false);
    this.fillMeta(world, sbb, 8, 4, 8, 10, 5, 10, blk("red_mushroom_block"), 14, blk("air"), 0, false);

    if !this.m_taurPlaced.get()
    {
        let bx: i32 = this.xAt(7, 7);
        let by: i32 = this.yAt(1);
        let bz: i32 = this.zAt(7, 7);
        if sbb.contains(bx, by, bz)
        {
            this.m_taurPlaced.set(true);
            spawnHome(world, "twilightforest:minoshroom", bx, by, bz, true);
        }
    }

    true
}
