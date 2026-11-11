// draft from ComponentTFStrongholdUpperAscender.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(i: i32, facing: i32, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::StrongholdUpperAscender);
    init(&this, i, facing, x, y, z);
    Rc::new(this)
}

// ComponentTFStrongholdUpperAscender constructor
pub(crate) fn init(this: &Piece, i: i32, facing: i32, x: i32, y: i32, z: i32)
{
    stronghold_component::init(this, i, facing, x, y, z);
}

// ComponentTFStrongholdUpperAscender.generateBoundingBox
pub(crate) fn generateBoundingBox(this: &Piece, facing: i32, x: i32, y: i32, z: i32) -> Option<StructureBox>
{
    if y < 36
    {
        this.m_exitTop.set(true);
        Some(vanillaBox(x, y, z, -2, -1, 0, 5, 10, 10, facing))
    }
    else
    {
        this.m_exitTop.set(false);
        Some(vanillaBox(x, y, z, -2, -6, 0, 5, 10, 10, facing))
    }
}

// ComponentTFStrongholdUpperAscender.func_74861_a
pub(crate) fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    stronghold_component::build(this, parent, list, random);
    stronghold_component::addNewUpperComponent(
        this,
        parent,
        list,
        random,
        0,
        2,
        if this.m_exitTop.get() { 6 } else { 1 },
        10,
    );
}

// ComponentTFStrongholdUpperAscender.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    if this.liquidIn(world, sbb)
    {
        return false;
    }

    stronghold_component::placeUpperStrongholdWalls(
        this,
        world,
        sbb,
        0,
        0,
        0,
        4,
        9,
        9,
        random,
        this.m_deco.get().m_randomBlocks,
    );
    stronghold_component::placeSmallDoorwayAt(
        this,
        world,
        random,
        2,
        2,
        if this.m_exitTop.get() { 1 } else { 6 },
        0,
        sbb,
    );
    stronghold_component::placeSmallDoorwayAt(
        this,
        world,
        random,
        0,
        2,
        if this.m_exitTop.get() { 6 } else { 1 },
        9,
        sbb,
    );

    if this.m_exitTop.get()
    {
        stronghold_upper_ascender::makeStairsAt(this, world, 1, 3, 1, sbb);
        stronghold_upper_ascender::makeStairsAt(this, world, 2, 4, 1, sbb);
        stronghold_upper_ascender::makeStairsAt(this, world, 3, 5, 1, sbb);
        stronghold_upper_ascender::makeStairsAt(this, world, 4, 6, 1, sbb);
        stronghold_upper_ascender::makeStairsAt(this, world, 5, 7, 1, sbb);
        stronghold_upper_ascender::makePlatformAt(this, world, 5, 8, sbb);
    }
    else
    {
        stronghold_upper_ascender::makeStairsAt(this, world, 1, 6, 3, sbb);
        stronghold_upper_ascender::makeStairsAt(this, world, 2, 5, 3, sbb);
        stronghold_upper_ascender::makeStairsAt(this, world, 3, 4, 3, sbb);
        stronghold_upper_ascender::makeStairsAt(this, world, 4, 3, 3, sbb);
        stronghold_upper_ascender::makeStairsAt(this, world, 5, 2, 3, sbb);
        stronghold_upper_ascender::makePlatformAt(this, world, 5, 1, sbb);
    }
    true
}

// ComponentTFStrongholdUpperAscender.makeStairsAt
pub(crate) fn makeStairsAt(this: &Piece, world: &GenWorld, y: i32, z: i32, facing: i32, sbb: StructureBox)
{
    if (this.getBlock(world, 0, y, z, sbb) != blk("air")) || (this.getBlock(world, 4, y, z, sbb) != blk("air"))
    {
        for x in 1..4
        {
            this.placeBlock(world, blk("stone_brick_stairs"), this.stairMeta(facing), x, y, z, sbb);
        }
    }
}

// ComponentTFStrongholdUpperAscender.makePlatformAt
pub(crate) fn makePlatformAt(this: &Piece, world: &GenWorld, y: i32, z: i32, sbb: StructureBox)
{
    if (this.getBlock(world, 0, y, z, sbb) != blk("air")) || (this.getBlock(world, 4, y, z, sbb) != blk("air"))
    {
        for x in 1..4
        {
            this.placeBlock(world, blk("stonebrick"), 0, x, y, z, sbb);
        }
    }
}

// ComponentTFStrongholdUpperAscender.isComponentProtected
pub(crate) fn isComponentProtected(_this: &Piece) -> bool { false }
