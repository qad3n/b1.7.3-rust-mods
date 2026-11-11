// draft from ComponentTFStrongholdEntrance.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(world: &GenWorld, random: &Random, i: i32, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::StrongholdEntrance);
    init(&this, world, random, i, x, y, z);
    Rc::new(this)
}

// ComponentTFStrongholdEntrance constructor
pub(crate) fn init(this: &Piece, _world: &GenWorld, _random: &Random, i: i32, x: i32, y: i32, z: i32)
{
    stronghold_component::init(this, i, 0, x, y - 10, z);
    this.m_deco.set(Deco::decorator_stronghold());
}

// ComponentTFStrongholdEntrance.func_74861_a
pub(crate) fn build(this: &Piece, parent: &Piece, pieceList: &mut Pieces, random: &Random)
{
    stronghold_component::build(this, parent, pieceList, random);
    this.m_lowerPieces.prepareStructurePieces();
    stronghold_component::addNewComponent(this, parent, pieceList, random, 0, 4, 1, 18);
    this.m_lowerPieces.prepareStructurePieces();

    if stronghold_entrance::listContainsBossRoom(this, pieceList)
    {
        this.m_lowerPieces.markBossRoomUsed();
    }

    stronghold_component::addNewComponent(this, parent, pieceList, random, 1, -1, 1, 13);
    this.m_lowerPieces.prepareStructurePieces();

    if stronghold_entrance::listContainsBossRoom(this, pieceList)
    {
        this.m_lowerPieces.markBossRoomUsed();
    }

    stronghold_component::addNewComponent(this, parent, pieceList, random, 2, 13, 1, -1);
    this.m_lowerPieces.prepareStructurePieces();

    if stronghold_entrance::listContainsBossRoom(this, pieceList)
    {
        this.m_lowerPieces.markBossRoomUsed();
    }

    stronghold_component::addNewComponent(this, parent, pieceList, random, 3, 18, 1, 4);
    stronghold_entrance::listContainsBossRoom(this, pieceList);

    let mut shieldBox: StructureBox = this.m_box.get();

    for component in pieceList.clone()
    {
        shieldBox.expand(&component.bounds());
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
pub(crate) fn listContainsBossRoom(_this: &Piece, list: &mut Pieces) -> bool
{
    for component in list.clone()
    {
        if matches!(component.m_kind, Kind::StrongholdBossRoom)
        {
            return true;
        }
    }

    false
}

// ComponentTFStrongholdEntrance.generateBoundingBox
pub(crate) fn generateBoundingBox(_this: &Piece, facing: i32, x: i32, y: i32, z: i32) -> Option<StructureBox>
{
    Some(vanillaBox(x, y, z, -1, -1, 0, 18, 7, 18, facing))
}

// ComponentTFStrongholdEntrance.func_74875_a
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
        6,
        17,
        random,
        this.m_deco.get().m_randomBlocks,
    );

    let _var8: i32 = this.xAt(0, 0);
    let _var9: i32 = this.yAt(0);
    let _var10: i32 = this.zAt(0, 0);

    stronghold_component::placeCornerStatue(this, world, 5, 1, 5, 0, sbb);
    stronghold_component::placeCornerStatue(this, world, 5, 1, 12, 1, sbb);
    stronghold_component::placeCornerStatue(this, world, 12, 1, 5, 2, sbb);
    stronghold_component::placeCornerStatue(this, world, 12, 1, 12, 3, sbb);
    stronghold_component::placeWallStatue(this, world, 9, 1, 16, 0, sbb);
    stronghold_component::placeWallStatue(this, world, 1, 1, 9, 1, sbb);
    stronghold_component::placeWallStatue(this, world, 8, 1, 1, 2, sbb);
    stronghold_component::placeWallStatue(this, world, 16, 1, 8, 3, sbb);
    stronghold_component::placeDoors(this, world, random, sbb);
    true
}
