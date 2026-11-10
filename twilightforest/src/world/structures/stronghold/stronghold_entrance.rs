// draft from ComponentTFStrongholdEntrance.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(world: &GenWorld, random: &Random, i: i32, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::StrongholdEntrance);
    init(&this, world, random, i, x, y, z);
    Rc::new(this)
}

// ComponentTFStrongholdEntrance constructor
pub fn init(this: &Piece, world: &GenWorld, random: &Random, i: i32, x: i32, y: i32, z: i32)
{
    stronghold_component::init(this, i, 0, x, y - 10, z);
    this.m_deco.set(Deco::decorator_stronghold());
}

// ComponentTFStrongholdEntrance.func_74861_a
pub fn build(this: &Piece, parent: &Piece, pieceList: &mut Pieces, random: &Random)
{
    stronghold_component::build(this, &parent, pieceList, random);
    this.m_lowerPieces.prepareStructurePieces();
    stronghold_component::addNewComponent(this, &parent, pieceList, random, 0, 4, 1, 18);
    this.m_lowerPieces.prepareStructurePieces();
    if stronghold_entrance::listContainsBossRoom(this, pieceList)
    {
        this.m_lowerPieces.markBossRoomUsed();
    }
    stronghold_component::addNewComponent(this, &parent, pieceList, random, 1, -1, 1, 13);
    this.m_lowerPieces.prepareStructurePieces();
    if stronghold_entrance::listContainsBossRoom(this, pieceList)
    {
        this.m_lowerPieces.markBossRoomUsed();
    }
    stronghold_component::addNewComponent(this, &parent, pieceList, random, 2, 13, 1, -1);
    this.m_lowerPieces.prepareStructurePieces();
    if stronghold_entrance::listContainsBossRoom(this, pieceList)
    {
        this.m_lowerPieces.markBossRoomUsed();
    }
    stronghold_component::addNewComponent(this, &parent, pieceList, random, 3, 18, 1, 4);
    if !stronghold_entrance::listContainsBossRoom(this, pieceList)
    {}
    let mut shieldBox: StructureBox = this.m_box.get();
    let mut tStairs: i32 = 0;
    let mut tCorridors: i32 = 0;
    let mut deadEnd: i32 = 0;
    let mut tRooms: i32 = 0;
    let mut bossRooms: i32 = 0;
    for component in pieceList.clone()
    {
        shieldBox.expand(&component.bounds());
        if (matches!(component.m_kind, Kind::StrongholdSmallStairs)) && ((component).m_hasTreasure.get())
        {
            tStairs += 1;
        }
        if matches!(component.m_kind, Kind::StrongholdTreasureCorridor)
        {
            tCorridors += 1;
        }
        if matches!(component.m_kind, Kind::StrongholdDeadEnd)
        {
            deadEnd += 1;
        }
        if matches!(component.m_kind, Kind::StrongholdTreasureRoom)
        {
            tRooms += 1;
        }
        if matches!(component.m_kind, Kind::StrongholdBossRoom)
        {
            bossRooms += 1;
        }
    }
    let accessChamber: Rc<Piece> = stronghold_access_chamber::new(
        2,
        this.mode(),
        this.m_box.get().m_min[0] + 8,
        this.m_box.get().m_min[1] + 7,
        this.m_box.get().m_min[2] + 4,
    );
    pieceList.push(Rc::clone(&accessChamber));
    accessChamber.build(this, pieceList, random);
}

// ComponentTFStrongholdEntrance.listContainsBossRoom
pub fn listContainsBossRoom(this: &Piece, list: &mut Pieces) -> bool
{
    for component in list.clone()
    {
        if matches!(component.m_kind, Kind::StrongholdBossRoom)
        {
            return true;
        }
    }
    return false;
}

// ComponentTFStrongholdEntrance.generateBoundingBox
pub fn generateBoundingBox(this: &Piece, facing: i32, x: i32, y: i32, z: i32) -> Option<StructureBox>
{
    return Some(vanillaBox(x, y, z, -1, -1, 0, 18, 7, 18, facing));
}

// ComponentTFStrongholdEntrance.func_74875_a
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
        6,
        17,
        random,
        this.m_deco.get().m_randomBlocks,
    );
    let var8: i32 = this.xAt(0, 0);
    let var9: i32 = this.yAt(0);
    let var10: i32 = this.zAt(0, 0);
    stronghold_component::placeCornerStatue(this, world, 5, 1, 5, 0, sbb);
    stronghold_component::placeCornerStatue(this, world, 5, 1, 12, 1, sbb);
    stronghold_component::placeCornerStatue(this, world, 12, 1, 5, 2, sbb);
    stronghold_component::placeCornerStatue(this, world, 12, 1, 12, 3, sbb);
    stronghold_component::placeWallStatue(this, world, 9, 1, 16, 0, sbb);
    stronghold_component::placeWallStatue(this, world, 1, 1, 9, 1, sbb);
    stronghold_component::placeWallStatue(this, world, 8, 1, 1, 2, sbb);
    stronghold_component::placeWallStatue(this, world, 16, 1, 8, 3, sbb);
    stronghold_component::placeDoors(this, world, random, sbb);
    return true;
}
