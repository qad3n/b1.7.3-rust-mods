// draft from ComponentTFFinalCastleDungeonEntrance.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(random: &Random, i: i32, x: i32, y: i32, z: i32, direction: i32, level: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::FinalCastleDungeonEntrance);
    init(&this, random, i, x, y, z, direction, level);
    Rc::new(this)
}

// ComponentTFFinalCastleDungeonEntrance constructor
pub fn init(this: &Piece, random: &Random, i: i32, x: i32, y: i32, z: i32, direction: i32, level: i32)
{
    final_castle_dungeon_room31::init(this, random, i, x, y, z, direction, level);
    this.m_hasExit.set(false);
}

// ComponentTFFinalCastleDungeonEntrance.func_74861_a
pub fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    this.m_deco.set(Deco::decorator_castle());
    this.m_deco.get().m_blockID = blk("tf:castleMagic");
    this.m_deco.get().m_blockMeta = 2;
    this.m_deco.get().m_fenceID = blk("tf:forceField");
    this.m_deco.get().m_fenceMeta = 1;
    final_castle_dungeon_room31::build(this, this, list, random);
}

// ComponentTFFinalCastleDungeonEntrance.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    final_castle_dungeon_room31::add(this, world, random, sbb);
    for y in 0..=this.m_height.get()
    {
        let x: i32 = (this.m_size.get() / 2) - 2;
        let z: i32 = ((this.m_size.get() / 2) - y) + 2;
        this.fillMeta(
            world,
            sbb,
            x,
            y,
            z,
            x + 4,
            y,
            z,
            this.m_deco.get().m_stairID,
            this.stairMeta(3),
            this.m_deco.get().m_stairID,
            this.stairMeta(3),
            false,
        );
        this.fillMeta(world, sbb, x, 0, z, x + 4, y - 1, z, blk("tf:deadrock"), 0, blk("tf:deadrock"), 0, false);
    }
    this.fillMeta(world, sbb, 23, 0, 12, 23, 3, 14, blk("tf:castleDoor"), 2, blk("air"), 0, false);
    this.fillMeta(
        world,
        sbb,
        23,
        4,
        12,
        23,
        4,
        14,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockMeta,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockMeta,
        false,
    );
    return true;
}

// ComponentTFFinalCastleDungeonEntrance.getForceFieldMeta
pub fn getForceFieldMeta(this: &Piece, decoRNG: &Random) -> i32 { return 1; }

// ComponentTFFinalCastleDungeonEntrance.getRuneMeta
pub fn getRuneMeta(this: &Piece, fieldMeta: i32) -> i32 { return 0; }
