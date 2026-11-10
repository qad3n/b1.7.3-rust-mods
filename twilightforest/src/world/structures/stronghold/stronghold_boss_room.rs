// draft from ComponentTFStrongholdBossRoom.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, facing: i32, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::StrongholdBossRoom);
    init(&this, i, facing, x, y, z);
    Rc::new(this)
}

// ComponentTFStrongholdBossRoom constructor
pub fn init(this: &Piece, i: i32, facing: i32, x: i32, y: i32, z: i32)
{
    stronghold_component::init(this, i, facing, x, y, z);
    this.m_spawnListIndex.set(i32::MAX);
}

// ComponentTFStrongholdBossRoom.generateBoundingBox
pub fn generateBoundingBox(this: &Piece, facing: i32, x: i32, y: i32, z: i32) -> Option<StructureBox>
{
    return Some(boxFor(x, y, z, -13, -1, 0, 27, 7, 27, facing));
}

// ComponentTFStrongholdBossRoom.func_74861_a
pub fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    stronghold_component::build(this, &parent, list, random);
    stronghold_component::addDoor(this, 13, 1, 0);
}

// ComponentTFStrongholdBossRoom.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    stronghold_component::placeStrongholdWalls(
        this,
        world,
        sbb,
        0,
        0,
        0,
        26,
        6,
        26,
        random,
        this.m_deco.get().m_randomBlocks,
    );
    this.fillSelected(world, sbb, 1, 1, 1, 3, 5, 25, false, random, this.m_deco.get().m_randomBlocks);
    this.fillSelected(world, sbb, 23, 1, 1, 25, 5, 25, false, random, this.m_deco.get().m_randomBlocks);
    this.fillSelected(world, sbb, 4, 1, 1, 22, 5, 3, false, random, this.m_deco.get().m_randomBlocks);
    this.fillSelected(world, sbb, 4, 1, 23, 22, 5, 25, false, random, this.m_deco.get().m_randomBlocks);
    this.fillMeta(world, sbb, 1, 1, 1, 2, 5, 25, blk("obsidian"), 0, blk("obsidian"), 0, false);
    this.fillMeta(world, sbb, 24, 1, 1, 25, 5, 25, blk("obsidian"), 0, blk("obsidian"), 0, false);
    this.fillMeta(world, sbb, 4, 1, 1, 22, 5, 2, blk("obsidian"), 0, blk("obsidian"), 0, false);
    this.fillMeta(world, sbb, 4, 1, 24, 22, 5, 25, blk("obsidian"), 0, blk("obsidian"), 0, false);
    this.fillSelected(world, sbb, 4, 1, 4, 4, 5, 7, false, random, this.m_deco.get().m_randomBlocks);
    this.fillSelected(world, sbb, 5, 1, 4, 5, 5, 5, false, random, this.m_deco.get().m_randomBlocks);
    this.fillSelected(world, sbb, 6, 1, 4, 7, 5, 4, false, random, this.m_deco.get().m_randomBlocks);
    this.fillSelected(world, sbb, 4, 1, 19, 4, 5, 22, false, random, this.m_deco.get().m_randomBlocks);
    this.fillSelected(world, sbb, 5, 1, 21, 5, 5, 22, false, random, this.m_deco.get().m_randomBlocks);
    this.fillSelected(world, sbb, 6, 1, 22, 7, 5, 22, false, random, this.m_deco.get().m_randomBlocks);
    this.fillSelected(world, sbb, 22, 1, 4, 22, 5, 7, false, random, this.m_deco.get().m_randomBlocks);
    this.fillSelected(world, sbb, 21, 1, 4, 21, 5, 5, false, random, this.m_deco.get().m_randomBlocks);
    this.fillSelected(world, sbb, 19, 1, 4, 20, 5, 4, false, random, this.m_deco.get().m_randomBlocks);
    this.fillSelected(world, sbb, 22, 1, 19, 22, 5, 22, false, random, this.m_deco.get().m_randomBlocks);
    this.fillSelected(world, sbb, 21, 1, 21, 21, 5, 22, false, random, this.m_deco.get().m_randomBlocks);
    this.fillSelected(world, sbb, 19, 1, 22, 20, 5, 22, false, random, this.m_deco.get().m_randomBlocks);
    stronghold_boss_room::placePillarDecorations(this, world, sbb, 0);
    stronghold_boss_room::placePillarDecorations(this, world, sbb, 1);
    stronghold_boss_room::placePillarDecorations(this, world, sbb, 2);
    stronghold_boss_room::placePillarDecorations(this, world, sbb, 3);
    stronghold_boss_room::placeSarcophagus(this, world, sbb, 8, 1, 8, 0);
    stronghold_boss_room::placeSarcophagus(this, world, sbb, 13, 1, 8, 0);
    stronghold_boss_room::placeSarcophagus(this, world, sbb, 18, 1, 8, 0);
    stronghold_boss_room::placeSarcophagus(this, world, sbb, 8, 1, 15, 0);
    stronghold_boss_room::placeSarcophagus(this, world, sbb, 13, 1, 15, 0);
    stronghold_boss_room::placeSarcophagus(this, world, sbb, 18, 1, 15, 0);
    this.fillAir(world, sbb, 12, 1, 1, 14, 4, 2);
    this.fill(world, sbb, 12, 1, 3, 14, 4, 3, blk("iron_bars"), blk("iron_bars"), false);
    let var8: i32 = this.xAt(0, 0);
    let var9: i32 = this.yAt(0);
    let var10: i32 = this.zAt(0, 0);
    this.placeBlock(world, blk("tf:bossSpawner"), 4, 13, 2, 13, sbb);
    stronghold_component::placeDoors(this, world, random, sbb);
    return true;
}

// ComponentTFStrongholdBossRoom.placeSarcophagus
pub fn placeSarcophagus(this: &Piece, world: &GenWorld, sbb: StructureBox, x: i32, y: i32, z: i32, rotation: i32)
{
    this.placeTurned(
        world,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta,
        x + 1,
        y,
        z + 0,
        rotation,
        sbb,
    );
    this.placeTurned(
        world,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta,
        x - 1,
        y,
        z + 0,
        rotation,
        sbb,
    );
    this.placeTurned(
        world,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta,
        x + 1,
        y,
        z + 3,
        rotation,
        sbb,
    );
    this.placeTurned(
        world,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta,
        x - 1,
        y,
        z + 3,
        rotation,
        sbb,
    );
    if world.rand().nextIntBounded(7) == 0
    {
        this.placeTurned(world, blk("torch"), 0, x + 1, y + 1, z + 0, rotation, sbb);
    }
    else
    {
        this.placeTurned(
            world,
            this.m_deco.get().m_fenceID,
            this.m_deco.get().m_fenceMeta,
            x + 1,
            y + 1,
            z + 0,
            rotation,
            sbb,
        );
    }
    if world.rand().nextIntBounded(7) == 0
    {
        this.placeTurned(world, blk("torch"), 0, x - 1, y + 1, z + 0, rotation, sbb);
    }
    else
    {
        this.placeTurned(
            world,
            this.m_deco.get().m_fenceID,
            this.m_deco.get().m_fenceMeta,
            x - 1,
            y + 1,
            z + 0,
            rotation,
            sbb,
        );
    }
    if world.rand().nextIntBounded(7) == 0
    {
        this.placeTurned(world, blk("torch"), 0, x + 1, y + 1, z + 3, rotation, sbb);
    }
    else
    {
        this.placeTurned(
            world,
            this.m_deco.get().m_fenceID,
            this.m_deco.get().m_fenceMeta,
            x + 1,
            y + 1,
            z + 3,
            rotation,
            sbb,
        );
    }
    if world.rand().nextIntBounded(7) == 0
    {
        this.placeTurned(world, blk("torch"), 0, x - 1, y + 1, z + 3, rotation, sbb);
    }
    else
    {
        this.placeTurned(
            world,
            this.m_deco.get().m_fenceID,
            this.m_deco.get().m_fenceMeta,
            x - 1,
            y + 1,
            z + 3,
            rotation,
            sbb,
        );
    }
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(1 + rotation), x + 0, y, z + 0, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(3 + rotation), x + 0, y, z + 3, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(2 + rotation), x + 1, y, z + 1, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(2 + rotation), x + 1, y, z + 2, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(0 + rotation), x - 1, y, z + 1, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(0 + rotation), x - 1, y, z + 2, rotation, sbb);
    this.placeTurned(world, blk("stone_slab"), 0, x + 0, y + 1, z + 1, rotation, sbb);
    this.placeTurned(world, blk("stone_slab"), 0, x + 0, y + 1, z + 2, rotation, sbb);
}

// ComponentTFStrongholdBossRoom.placePillarDecorations
pub fn placePillarDecorations(this: &Piece, world: &GenWorld, sbb: StructureBox, rotation: i32)
{
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(3 + rotation), 4, 1, 8, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(2 + rotation), 8, 1, 4, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(3 + rotation) + 4, 4, 5, 8, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(2 + rotation) + 4, 8, 5, 4, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(3 + rotation), 5, 1, 6, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(2 + rotation), 6, 1, 6, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(2 + rotation), 6, 1, 5, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(3 + rotation) + 4, 5, 5, 6, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(2 + rotation) + 4, 6, 5, 6, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(2 + rotation) + 4, 6, 5, 5, rotation, sbb);
}

// ComponentTFStrongholdBossRoom.placeDoorwayAt
pub fn placeDoorwayAt(this: &Piece, world: &GenWorld, random: &Random, x: i32, y: i32, z: i32, sbb: StructureBox)
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

// ComponentTFStrongholdBossRoom.isValidBreakInPoint
pub fn isValidBreakInPoint(this: &Piece, wx: i32, wy: i32, wz: i32) -> bool { return false; }
