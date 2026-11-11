// draft from ComponentTFStrongholdFoundry.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, facing: i32, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::StrongholdFoundry);
    init(&this, i, facing, x, y, z);
    Rc::new(this)
}

// ComponentTFStrongholdFoundry constructor
pub fn init(this: &Piece, i: i32, facing: i32, x: i32, y: i32, z: i32)
{
    stronghold_component::init(this, i, facing, x, y, z);
}

// ComponentTFStrongholdFoundry.generateBoundingBox
pub fn generateBoundingBox(this: &Piece, facing: i32, x: i32, y: i32, z: i32) -> Option<StructureBox>
{
    if y > 17
    {
        this.m_entranceLevel.set(3);
        return Some(boxFor(x, y, z, -4, -20, 0, 18, 25, 18, facing));
    }
    else if y < 11
    {
        this.m_entranceLevel.set(1);
        return Some(boxFor(x, y, z, -4, -6, 0, 18, 25, 18, facing));
    }
    else
    {
        this.m_entranceLevel.set(2);
        return Some(boxFor(x, y, z, -4, -13, 0, 18, 25, 18, facing));
    }
}

// ComponentTFStrongholdFoundry.func_74861_a
pub fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    stronghold_component::build(this, &parent, list, random);
    match this.m_entranceLevel.get()
    {
        1 =>
        {
            stronghold_component::addDoor(this, 4, 6, 0);
            stronghold_component::addNewComponent(this, &parent, list, random, 1, -1, 13, 13);
            stronghold_component::addNewComponent(this, &parent, list, random, 3, 18, 13, 4);
            stronghold_component::addNewComponent(this, &parent, list, random, 0, 13, 20, 18);
        }
        2 =>
        {
            stronghold_component::addDoor(this, 4, 13, 0);
            stronghold_component::addNewComponent(this, &parent, list, random, 1, -1, 6, 13);
            stronghold_component::addNewComponent(this, &parent, list, random, 3, 18, 20, 4);
            stronghold_component::addNewComponent(this, &parent, list, random, 0, 13, 13, 18);
        }
        3 =>
        {
            stronghold_component::addDoor(this, 4, 20, 0);
            stronghold_component::addNewComponent(this, &parent, list, random, 0, 13, 6, 18);
            stronghold_component::addNewComponent(this, &parent, list, random, 1, -1, 13, 13);
            stronghold_component::addNewComponent(this, &parent, list, random, 3, 18, 13, 4);
        }
        _ =>
        {}
    }
}

// ComponentTFStrongholdFoundry.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    stronghold_component::placeStrongholdWalls(
        this,
        world,
        sbb,
        0,
        0,
        0,
        17,
        25,
        17,
        random,
        this.m_deco.get().m_randomBlocks,
    );
    this.fill(world, sbb, 1, 0, 1, 16, 4, 16, blk("lava"), blk("lava"), false);
    this.fillSelected(world, sbb, 1, 19, 1, 16, 19, 16, false, random, this.m_deco.get().m_randomBlocks);
    this.fillAir(world, sbb, 2, 19, 2, 15, 19, 15);
    this.fillSelected(world, sbb, 1, 12, 1, 16, 12, 16, false, random, this.m_deco.get().m_randomBlocks);
    this.fillAir(world, sbb, 2, 12, 2, 15, 12, 15);
    this.fillSelected(world, sbb, 1, 5, 1, 16, 5, 16, false, random, this.m_deco.get().m_randomBlocks);
    this.fillAir(world, sbb, 2, 5, 2, 15, 5, 15);
    this.fillSelected(world, sbb, 1, 1, 1, 1, 24, 2, false, random, this.m_deco.get().m_randomBlocks);
    this.fillSelected(world, sbb, 2, 1, 1, 2, 24, 1, false, random, this.m_deco.get().m_randomBlocks);
    this.fillSelected(world, sbb, 16, 1, 1, 16, 24, 2, false, random, this.m_deco.get().m_randomBlocks);
    this.fillSelected(world, sbb, 15, 1, 1, 15, 24, 1, false, random, this.m_deco.get().m_randomBlocks);
    this.fillSelected(world, sbb, 1, 1, 15, 1, 24, 16, false, random, this.m_deco.get().m_randomBlocks);
    this.fillSelected(world, sbb, 2, 1, 16, 2, 24, 16, false, random, this.m_deco.get().m_randomBlocks);
    this.fillSelected(world, sbb, 16, 1, 15, 16, 24, 16, false, random, this.m_deco.get().m_randomBlocks);
    this.fillSelected(world, sbb, 15, 1, 16, 15, 24, 16, false, random, this.m_deco.get().m_randomBlocks);
    let massRandom: Random = Random::withSeed(random.nextLong());
    for x in 4..14
    {
        for z in 4..14
        {
            for y in 8..23
            {
                let c: f32 = (((x) as f32 - 8.5).abs() + ((z) as f32 - 8.5).abs()) + ((y) as f32 - 18.0).abs();
                let r: f32 = 5.5 + ((massRandom.nextFloat() - massRandom.nextFloat()) * 3.5);
                if c < r
                {
                    this.placeBlock(world, blk("stone"), 0, x, y, z, sbb);
                }
            }
        }
    }
    for _i in 0..400
    {
        let dx: i32 = massRandom.nextIntBounded(9) + 5;
        let dz: i32 = massRandom.nextIntBounded(9) + 5;
        let dy: i32 = massRandom.nextIntBounded(13) + 10;
        if this.getBlock(world, dx, dy, dz, sbb) != blk("air")
        {
            for y in 0..3
            {
                this.placeBlock(world, blk("stone"), 0, dx, dy - y, dz, sbb);
            }
        }
    }
    for _i in 0..8
    {
        let blockID: Blk = blk("redstone_ore");
        let blockMeta: i32 = 0;
        stronghold_foundry::addOreToMass(this, world, sbb, &massRandom, blockID, blockMeta);
    }
    for _i in 0..8
    {
        let blockID: Blk = blk("iron_ore");
        let blockMeta: i32 = 0;
        stronghold_foundry::addOreToMass(this, world, sbb, &massRandom, blockID, blockMeta);
    }
    for _i in 0..6
    {
        let blockID: Blk = blk("gold_ore");
        let blockMeta: i32 = 0;
        stronghold_foundry::addOreToMass(this, world, sbb, &massRandom, blockID, blockMeta);
    }
    for _i in 0..2
    {
        let blockID: Blk = blk("glowstone");
        let blockMeta: i32 = 0;
        stronghold_foundry::addOreToMass(this, world, sbb, &massRandom, blockID, blockMeta);
    }
    for _i in 0..2
    {
        let blockID: Blk = blk("emerald_ore");
        let blockMeta: i32 = 0;
        stronghold_foundry::addOreToMass(this, world, sbb, &massRandom, blockID, blockMeta);
    }
    for _i in 0..4
    {
        let blockID: Blk = blk("diamond_ore");
        let blockMeta: i32 = 0;
        stronghold_foundry::addOreToMass(this, world, sbb, &massRandom, blockID, blockMeta);
    }
    stronghold_component::placeDoors(this, world, random, sbb);
    return true;
}

// ComponentTFStrongholdFoundry.addOreToMass
pub fn addOreToMass(
    this: &Piece,
    world: &GenWorld,
    sbb: StructureBox,
    massRandom: &Random,
    blockID: Blk,
    blockMeta: i32,
)
{
    for _i in 0..10
    {
        let dx: i32 = massRandom.nextIntBounded(9) + 5;
        let dz: i32 = massRandom.nextIntBounded(9) + 5;
        let dy: i32 = massRandom.nextIntBounded(13) + 10;
        if this.getBlock(world, dx, dy, dz, sbb) != blk("air")
        {
            this.placeBlock(world, blockID, blockMeta, dx, dy, dz, sbb);
            break;
        }
    }
}
