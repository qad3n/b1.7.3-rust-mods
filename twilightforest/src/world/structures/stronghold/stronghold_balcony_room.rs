// draft from ComponentTFStrongholdBalconyRoom.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(i: i32, facing: i32, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::StrongholdBalconyRoom);
    init(&this, i, facing, x, y, z);
    Rc::new(this)
}

// ComponentTFStrongholdBalconyRoom constructor
pub(crate) fn init(this: &Piece, i: i32, facing: i32, x: i32, y: i32, z: i32)
{
    stronghold_component::init(this, i, facing, x, y, z);
}

// ComponentTFStrongholdBalconyRoom.generateBoundingBox
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
            boxFor(x, y, z, -4, -1, 0, 18, 14, 27, facing)
        }
        else
        {
            boxFor(x, y, z, -13, -8, 0, 18, 14, 27, facing)
        },
    )
}

// ComponentTFStrongholdBalconyRoom.func_74861_a
pub(crate) fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    stronghold_component::build(this, parent, list, random);
    stronghold_component::addNewComponent(this, parent, list, random, 0, 13, 1, 27);
    stronghold_component::addNewComponent(this, parent, list, random, 1, -1, 1, 13);
    stronghold_component::addNewComponent(this, parent, list, random, 3, 18, 1, 13);
    stronghold_component::addNewComponent(this, parent, list, random, 0, 4, 8, 27);
    stronghold_component::addNewComponent(this, parent, list, random, 1, -1, 8, 4);
    stronghold_component::addNewComponent(this, parent, list, random, 3, 18, 8, 22);

    if this.m_enterBottom.get()
    {
        stronghold_component::addDoor(this, 4, 1, 0);
        stronghold_component::addNewComponent(this, parent, list, random, 2, 13, 8, -1);
    }
    else
    {
        stronghold_component::addDoor(this, 13, 8, 0);
        stronghold_component::addNewComponent(this, parent, list, random, 2, 4, 1, -1);
    }
}

// ComponentTFStrongholdBalconyRoom.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    stronghold_component::placeStrongholdWalls(
        this,
        world,
        sbb,
        0,
        0,
        0,
        17,
        13,
        26,
        random,
        this.m_deco.get().m_randomBlocks,
    );
    this.fillSelected(world, sbb, 1, 6, 1, 16, 7, 25, false, random, this.m_deco.get().m_randomBlocks);
    this.fill(world, sbb, 4, 8, 4, 13, 8, 22, this.m_deco.get().m_fenceID, blk("air"), false);
    this.fillAir(world, sbb, 5, 6, 5, 12, 8, 21);
    stronghold_balcony_room::placeStairsAndPillars(this, world, sbb, 0);
    stronghold_balcony_room::placeStairsAndPillars(this, world, sbb, 2);
    stronghold_component::placeDoors(this, world, random, sbb);
    true
}

// ComponentTFStrongholdBalconyRoom.placeStairsAndPillars
pub(crate) fn placeStairsAndPillars(this: &Piece, world: &GenWorld, sbb: StructureBox, rotation: i32)
{
    this.fillTurned(
        world,
        sbb,
        4,
        1,
        4,
        4,
        12,
        4,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta,
        rotation,
    );
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(3 + rotation), 4, 1, 5, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(2 + rotation), 5, 1, 4, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(3 + rotation) + 4, 4, 5, 5, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(2 + rotation) + 4, 5, 5, 4, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(3 + rotation) + 4, 4, 12, 5, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(2 + rotation) + 4, 5, 12, 4, rotation, sbb);
    this.fillTurned(
        world,
        sbb,
        13,
        1,
        4,
        13,
        12,
        4,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta,
        rotation,
    );
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(3 + rotation), 13, 1, 5, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(0 + rotation), 12, 1, 4, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(3 + rotation) + 4, 13, 5, 5, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(0 + rotation) + 4, 12, 5, 4, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(3 + rotation) + 4, 13, 12, 5, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(0 + rotation) + 4, 12, 12, 4, rotation, sbb);
    this.fillTurned(
        world,
        sbb,
        13,
        1,
        8,
        13,
        12,
        8,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta,
        rotation,
    );
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(3 + rotation), 13, 1, 9, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(1 + rotation), 13, 1, 7, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(0 + rotation), 12, 1, 8, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(3 + rotation) + 4, 13, 5, 9, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(1 + rotation) + 4, 13, 5, 7, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(3 + rotation) + 4, 13, 12, 9, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(1 + rotation) + 4, 13, 12, 7, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(0 + rotation) + 4, 12, 12, 8, rotation, sbb);

    for y in 1..8
    {
        for z in 5..8
        {
            this.placeTurned(world, blk("air"), 0, y + 6, y + 1, z, rotation, sbb);
            this.placeTurned(
                world,
                this.m_deco.get().m_stairID,
                this.stairMeta(0 + rotation),
                y + 6,
                y,
                z,
                rotation,
                sbb,
            );
            this.placeTurned(
                world,
                this.m_deco.get().m_blockID,
                this.m_deco.get().m_blockMeta,
                y + 6,
                y - 1,
                z,
                rotation,
                sbb,
            );
        }
    }
}
