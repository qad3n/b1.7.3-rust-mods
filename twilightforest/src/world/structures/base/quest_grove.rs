// draft from ComponentTFQuestGrove.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(world: &GenWorld, random: &Random, i: i32, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::QuestGrove);
    init(&this, world, random, i, x, y, z);
    Rc::new(this)
}

// ComponentTFQuestGrove constructor
pub fn init(this: &Piece, _world: &GenWorld, _random: &Random, i: i32, x: i32, y: i32, z: i32)
{
    this.m_depth.set(i);
    this.m_beastPlaced.set(false);
    this.m_dispenserPlaced.set(false);
    this.setMode(0);
    this.m_box.set(boxFor(x, y, z, -13, 0, -13, 26, 10, 26, 0));
}

// ComponentTFQuestGrove.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    for i in 0..4
    {
        quest_grove::makeWallSide(this, world, random, i, sbb);
    }
    for x in 10..17
    {
        for z in 10..17
        {
            if (((x != 10) && (x != 16)) && (z != 10)) && (z != 16)
            {
                if (((x != 11) && (x != 15)) && (z != 11)) && (z != 15)
                {
                    this.placeBlock(world, blk("stonebrick"), 1, x, -1, z, sbb);
                }
                else if random.nextIntBounded(3) > 0
                {
                    this.placeBlock(world, blk("stonebrick"), 1, x, -1, z, sbb);
                }
            }
            else if random.nextIntBounded(2) > 0
            {
                this.placeBlock(world, blk("stonebrick"), 1, x, -1, z, sbb);
            }
        }
    }
    this.placeBlock(world, blk("stone_button"), 4, 13, 5, 19, sbb);
    this.placeBlock(world, blk("stonebrick"), 1, 12, 7, 20, sbb);
    this.placeBlock(world, blk("stonebrick"), 1, 13, 7, 20, sbb);
    this.placeBlock(world, blk("stonebrick"), 1, 14, 7, 20, sbb);
    this.placeBlock(world, blk("stonebrick"), 1, 12, 7, 21, sbb);
    this.placeBlock(world, blk("stonebrick"), 1, 13, 7, 21, sbb);
    this.placeBlock(world, blk("stonebrick"), 1, 14, 7, 21, sbb);
    if !this.m_dispenserPlaced.get()
    {
        let bx: i32 = this.xAt(13, 20);
        let by: i32 = this.yAt(6);
        let bz: i32 = this.zAt(13, 20);
        if sbb.contains(bx, by, bz)
        {
            this.m_dispenserPlaced.set(true);
            world.setBlock(bx, by, bz, blk("dispenser"), 2, 4);
            for i in 0..4
            {
                let wool: b173_api::ItemStack = b173_api::ItemStack
                {
                    damage: random.nextIntBounded(16),
                    ..b173_api::ItemStack::of("minecraft:wool", 1)
                };
                if let Err(error) = world.m_world.setTileSlot(b173_api::BlockPos::new(bx, by, bz), i, Some(wool))
                {
                    b173_api::warn(&format!("quest grove dispenser: {}", error.0));
                }
            }
        }
    }
    if !this.m_beastPlaced.get()
    {
        let bx: i32 = this.xAt(13, 13);
        let by: i32 = this.yAt(0);
        let bz: i32 = this.zAt(13, 13);
        if sbb.contains(bx, by, bz)
        {
            this.m_beastPlaced.set(true);
            spawnHome(world, "twilightforest:quest_ram", bx, by, bz, true);
        }
    }
    return true;
}

// ComponentTFQuestGrove.makeWallSide
pub fn makeWallSide(this: &Piece, world: &GenWorld, _random: &Random, direction: i32, sbb: StructureBox)
{
    let temp: i32 = this.mode();
    this.setMode(direction);
    quest_grove::placeOuterArch(this, world, 3, -1, sbb);
    quest_grove::placeOuterArch(this, world, 11, -1, sbb);
    quest_grove::placeOuterArch(this, world, 19, -1, sbb);
    this.placeBlock(world, blk("stonebrick"), 3, 0, 0, 0, sbb);
    this.placeBlock(world, blk("stonebrick"), 3, 0, 1, 0, sbb);
    this.placeBlock(world, blk("stonebrick"), 3, 0, 2, 0, sbb);
    this.placeBlock(world, blk("stonebrick"), 3, 0, 3, 0, sbb);
    this.placeBlock(world, blk("stonebrick"), 3, 1, 3, 0, sbb);
    this.placeBlock(world, blk("stonebrick"), 3, 2, 3, 0, sbb);
    this.placeBlock(world, blk("stonebrick"), 3, 8, 3, 0, sbb);
    this.placeBlock(world, blk("stonebrick"), 3, 9, 3, 0, sbb);
    this.placeBlock(world, blk("stonebrick"), 3, 10, 3, 0, sbb);
    this.placeBlock(world, blk("stonebrick"), 3, 16, 3, 0, sbb);
    this.placeBlock(world, blk("stonebrick"), 3, 17, 3, 0, sbb);
    this.placeBlock(world, blk("stonebrick"), 3, 18, 3, 0, sbb);
    this.placeBlock(world, blk("stonebrick"), 3, 24, 3, 0, sbb);
    this.placeBlock(world, blk("stonebrick"), 3, 25, 3, 0, sbb);
    for x in 0..9
    {
        for y in 0..9
        {
            for z in 0..2
            {
                if (((((((x == 0) || (x == 1)) || (x == 7)) || (x == 8)) || (y == 0)) || (y == 1)) || (y == 7))
                    || (y == 8)
                {
                    this.placeBlock(world, blk("stonebrick"), 1, x + 9, y - 2, z + 5, sbb);
                }
            }
        }
    }
    this.placeBlock(world, blk("stonebrick"), 3, 6, 0, 6, sbb);
    this.placeBlock(world, blk("stonebrick"), 3, 6, 1, 6, sbb);
    this.placeBlock(world, blk("stonebrick"), 3, 6, 2, 6, sbb);
    this.placeBlock(world, blk("stonebrick"), 3, 6, 3, 6, sbb);
    this.placeBlock(world, blk("stonebrick"), 3, 6, 4, 6, sbb);
    this.placeBlock(world, blk("stonebrick"), 3, 7, 4, 6, sbb);
    this.placeBlock(world, blk("stonebrick"), 3, 8, 4, 6, sbb);
    this.placeBlock(world, blk("stonebrick"), 3, 18, 4, 6, sbb);
    this.placeBlock(world, blk("stonebrick"), 3, 19, 4, 6, sbb);
    this.setMode(temp);
}

// ComponentTFQuestGrove.placeOuterArch
pub fn placeOuterArch(this: &Piece, world: &GenWorld, ox: i32, oy: i32, sbb: StructureBox)
{
    for x in 0..5
    {
        for y in 0..6
        {
            if (((x == 0) || (x == 4)) || (y == 0)) || (y == 5)
            {
                this.placeBlock(world, blk("stonebrick"), 1, x + ox, y + oy, 0, sbb);
            }
        }
    }
}
