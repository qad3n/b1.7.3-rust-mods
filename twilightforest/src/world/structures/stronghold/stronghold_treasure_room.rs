// draft from ComponentTFStrongholdTreasureRoom.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, facing: i32, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::StrongholdTreasureRoom);
    init(&this, i, facing, x, y, z);
    Rc::new(this)
}

// ComponentTFStrongholdTreasureRoom constructor
pub fn init(this: &Piece, i: i32, facing: i32, x: i32, y: i32, z: i32)
{
    stronghold_component::init(this, i, facing, x, y, z);
}

// ComponentTFStrongholdTreasureRoom.generateBoundingBox
pub fn generateBoundingBox(_this: &Piece, facing: i32, x: i32, y: i32, z: i32) -> Option<StructureBox>
{
    return Some(vanillaBox(x, y, z, -4, -1, 0, 9, 7, 18, facing));
}

// ComponentTFStrongholdTreasureRoom.func_74861_a
pub fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    stronghold_component::build(this, &parent, list, random);
    stronghold_component::addDoor(this, 4, 1, 0);
}

// ComponentTFStrongholdTreasureRoom.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    stronghold_component::placeStrongholdWalls(
        this,
        world,
        sbb,
        0,
        0,
        0,
        8,
        6,
        17,
        random,
        this.m_deco.get().m_randomBlocks,
    );
    stronghold_component::placeWallStatue(this, world, 1, 1, 4, 1, sbb);
    stronghold_component::placeWallStatue(this, world, 1, 1, 13, 1, sbb);
    stronghold_component::placeWallStatue(this, world, 7, 1, 4, 3, sbb);
    stronghold_component::placeWallStatue(this, world, 7, 1, 13, 3, sbb);
    stronghold_component::placeWallStatue(this, world, 4, 1, 16, 0, sbb);
    this.fillSelected(world, sbb, 1, 1, 8, 7, 5, 9, false, random, this.m_deco.get().m_randomBlocks);
    this.fill(world, sbb, 3, 1, 8, 5, 4, 9, blk("iron_bars"), blk("iron_bars"), false);
    this.placeSpawner(world, random, 4, 1, 4, mob("Helmet Crab"), sbb);
    this.placeSpawner(world, random, 4, 4, 15, mob("Helmet Crab"), sbb);
    this.placeTreasure(world, random, 2, 4, 13, Table::StrongholdRoom, sbb);
    this.placeTreasure(world, random, 6, 4, 13, Table::StrongholdRoom, sbb);
    stronghold_component::placeDoors(this, world, random, sbb);
    return true;
}

// ComponentTFStrongholdTreasureRoom.placeDoorwayAt
pub fn placeDoorwayAt(this: &Piece, world: &GenWorld, _random: &Random, x: i32, y: i32, z: i32, sbb: StructureBox)
{
    if (x != 0) && (x != stronghold_component::getXSize(this))
    {
        this.fillMeta(world, sbb, x - 1, y, z, x + 1, y + 3, z, blk("iron_bars"), 0, blk("air"), 0, false);
    }
    else
    {
        this.fillMeta(world, sbb, x, y, z - 1, x, y + 3, z + 1, blk("iron_bars"), 0, blk("air"), 0, false);
    }
}
