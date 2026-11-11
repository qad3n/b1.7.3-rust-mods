// draft from ComponentTFStrongholdSmallStairs.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(i: i32, facing: i32, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::StrongholdSmallStairs);
    init(&this, i, facing, x, y, z);
    Rc::new(this)
}

// ComponentTFStrongholdSmallStairs constructor
pub(crate) fn init(this: &Piece, i: i32, facing: i32, x: i32, y: i32, z: i32)
{
    stronghold_component::init(this, i, facing, x, y, z);
}

// ComponentTFStrongholdSmallStairs.generateBoundingBox
pub(crate) fn generateBoundingBox(this: &Piece, facing: i32, x: i32, y: i32, z: i32) -> Option<StructureBox>
{
    if y > 17
    {
        this.m_enterBottom.set(false);
    }
    else if y < 11
    {
        this.m_enterBottom.set(true);
    }
    else
    {
        this.m_enterBottom.set((z & 1) == 0);
    }
    Some(
        if this.m_enterBottom.get()
        {
            vanillaBox(x, y, z, -4, -1, 0, 9, 14, 9, facing)
        }
        else
        {
            vanillaBox(x, y, z, -4, -8, 0, 9, 14, 9, facing)
        },
    )
}

// ComponentTFStrongholdSmallStairs.func_74861_a
pub(crate) fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    stronghold_component::build(this, parent, list, random);

    if this.m_enterBottom.get()
    {
        stronghold_component::addDoor(this, 4, 1, 0);
        stronghold_component::addNewComponent(this, parent, list, random, 0, 4, 8, 9);
    }
    else
    {
        stronghold_component::addDoor(this, 4, 8, 0);
        stronghold_component::addNewComponent(this, parent, list, random, 0, 4, 1, 9);
    }
    this.m_hasTreasure.set(random.nextBoolean());
    this.m_chestTrapped.set(random.nextIntBounded(3) == 0);
}

// ComponentTFStrongholdSmallStairs.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    stronghold_component::placeStrongholdWalls(
        this,
        world,
        sbb,
        0,
        0,
        0,
        8,
        13,
        8,
        random,
        this.m_deco.get().m_randomBlocks,
    );
    this.fillMeta(
        world,
        sbb,
        1,
        7,
        1,
        7,
        7,
        7,
        this.m_deco.get().m_platformID,
        this.m_deco.get().m_platformMeta,
        blk("air"),
        0,
        false,
    );
    this.fillAir(world, sbb, 2, 7, 2, 6, 7, 6);

    let rotation: i32 = if this.m_enterBottom.get() { 0 } else { 2 };

    for y in 1..8
    {
        for x in 3..6
        {
            this.placeTurned(world, blk("air"), 0, x, y + 1, y, rotation, sbb);
            this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(1 + rotation), x, y, y, rotation, sbb);
            this.placeTurned(
                world,
                this.m_deco.get().m_blockID,
                this.m_deco.get().m_blockMeta,
                x,
                y - 1,
                y,
                rotation,
                sbb,
            );
        }
    }

    if this.m_hasTreasure.get()
    {
        this.placeTreasureRotatedTrapped(
            world,
            4,
            1,
            6,
            rotation,
            Table::StrongholdCache,
            this.m_chestTrapped.get(),
            sbb,
        );

        if this.m_chestTrapped.get()
        {
            this.placeTurned(world, blk("tnt"), 0, 4, 0, 6, rotation, sbb);
        }

        for z in 5..8
        {
            this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(0 + rotation), 3, 1, z, rotation, sbb);
            this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(2 + rotation), 5, 1, z, rotation, sbb);
        }

        this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(1 + rotation), 4, 1, 5, rotation, sbb);
        this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(3 + rotation), 4, 1, 7, rotation, sbb);
        this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(1 + rotation), 4, 2, 6, rotation, sbb);
    }

    if this.m_enterBottom.get()
    {
        stronghold_component::placeWallStatue(this, world, 4, 8, 1, 2, sbb);
    }
    else
    {
        stronghold_component::placeWallStatue(this, world, 4, 8, 7, 0, sbb);
    }
    stronghold_component::placeDoors(this, world, random, sbb);
    true
}
