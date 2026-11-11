// draft from ComponentTFStrongholdAtrium.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(i: i32, facing: i32, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::StrongholdAtrium);
    init(&this, i, facing, x, y, z);
    Rc::new(this)
}

// ComponentTFStrongholdAtrium constructor
pub(crate) fn init(this: &Piece, i: i32, facing: i32, x: i32, y: i32, z: i32)
{
    stronghold_component::init(this, i, facing, x, y, z);
}

// ComponentTFStrongholdAtrium.generateBoundingBox
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
            boxFor(x, y, z, -4, -1, 0, 18, 14, 18, facing)
        }
        else
        {
            boxFor(x, y, z, -13, -8, 0, 18, 14, 18, facing)
        },
    )
}

// ComponentTFStrongholdAtrium.func_74861_a
pub(crate) fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    stronghold_component::build(this, parent, list, random);

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
    stronghold_component::addNewComponent(this, parent, list, random, 0, 13, 1, 18);
    stronghold_component::addNewComponent(this, parent, list, random, 0, 4, 8, 18);
}

// ComponentTFStrongholdAtrium.func_74875_a
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
        17,
        random,
        this.m_deco.get().m_randomBlocks,
    );
    this.fillSelected(world, sbb, 1, 6, 1, 16, 7, 16, false, random, this.m_deco.get().m_randomBlocks);
    this.fill(world, sbb, 5, 8, 5, 12, 8, 12, this.m_deco.get().m_fenceID, blk("air"), false);
    this.fillAir(world, sbb, 6, 6, 6, 11, 8, 11);
    stronghold_atrium::placeBalconyPillar(this, world, sbb, 0);
    stronghold_atrium::placeBalconyPillar(this, world, sbb, 1);
    stronghold_atrium::placeBalconyPillar(this, world, sbb, 2);
    stronghold_atrium::placeBalconyPillar(this, world, sbb, 3);
    this.fillSelected(world, sbb, 1, 1, 1, 1, 12, 2, false, random, this.m_deco.get().m_randomBlocks);
    this.fillSelected(world, sbb, 2, 1, 1, 2, 12, 1, false, random, this.m_deco.get().m_randomBlocks);
    this.fillSelected(world, sbb, 16, 1, 1, 16, 12, 2, false, random, this.m_deco.get().m_randomBlocks);
    this.fillSelected(world, sbb, 15, 1, 1, 15, 12, 1, false, random, this.m_deco.get().m_randomBlocks);
    this.fillSelected(world, sbb, 1, 1, 15, 1, 12, 16, false, random, this.m_deco.get().m_randomBlocks);
    this.fillSelected(world, sbb, 2, 1, 16, 2, 12, 16, false, random, this.m_deco.get().m_randomBlocks);
    this.fillSelected(world, sbb, 16, 1, 15, 16, 12, 16, false, random, this.m_deco.get().m_randomBlocks);
    this.fillSelected(world, sbb, 15, 1, 16, 15, 12, 16, false, random, this.m_deco.get().m_randomBlocks);
    this.fillRandomly(world, sbb, random, 0.5, 6, 0, 6, 11, 0, 11, blk("grass"), blk("grass"), false);
    this.fill(world, sbb, 7, 0, 7, 10, 0, 10, blk("grass"), blk("air"), false);
    stronghold_atrium::spawnATree(this, world, random.nextIntBounded(5), 8, 1, 8, sbb);
    stronghold_component::placeCornerStatue(this, world, 2, 8, 2, 0, sbb);
    stronghold_component::placeCornerStatue(this, world, 2, 1, 15, 1, sbb);
    stronghold_component::placeCornerStatue(this, world, 15, 1, 2, 2, sbb);
    stronghold_component::placeCornerStatue(this, world, 15, 8, 15, 3, sbb);
    stronghold_component::placeDoors(this, world, random, sbb);
    true
}

// ComponentTFStrongholdAtrium.spawnATree
pub(crate) fn spawnATree(this: &Piece, world: &GenWorld, treeNum: i32, x: i32, y: i32, z: i32, sbb: StructureBox)
{
    let dx: i32 = this.xAt(x, z);
    let dy: i32 = this.yAt(y);
    let dz: i32 = this.zAt(x, z);
    if sbb.contains(dx, dy, dz)
    {
        let minHeight: i32 = 8;

        let var10000: Gen = match treeNum
        {
            1 => Gen::Trees { minimum: minHeight, meta: 3 },
            2 => Gen::Trees { minimum: minHeight, meta: 2 },
            3 => Gen::TwilightOak,
            4 => Gen::Rainboak,
            _ => Gen::Trees { minimum: minHeight, meta: 0 },
        };
        let treeGen: Gen = var10000;
        let mut i: i32 = 0;
        while (i < 100) && (!treeGen.generate(world, world.rand(), dx, dy, dz))
        {
            i += 1;
        }
    }
}

// ComponentTFStrongholdAtrium.placeBalconyPillar
pub(crate) fn placeBalconyPillar(this: &Piece, world: &GenWorld, sbb: StructureBox, rotation: i32)
{
    this.fillTurned(
        world,
        sbb,
        5,
        1,
        5,
        5,
        12,
        5,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta,
        rotation,
    );
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(3 + rotation), 5, 1, 6, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(2 + rotation), 6, 1, 5, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(3 + rotation) + 4, 5, 5, 6, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(2 + rotation) + 4, 6, 5, 5, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(3 + rotation) + 4, 5, 12, 6, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(2 + rotation) + 4, 6, 12, 5, rotation, sbb);
}
