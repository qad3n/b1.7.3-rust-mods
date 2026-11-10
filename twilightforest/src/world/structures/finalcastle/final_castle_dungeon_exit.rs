// draft from ComponentTFFinalCastleDungeonExit.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(random: &Random, i: i32, x: i32, y: i32, z: i32, direction: i32, level: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::FinalCastleDungeonExit);
    init(&this, random, i, x, y, z, direction, level);
    Rc::new(this)
}

// ComponentTFFinalCastleDungeonExit constructor
pub fn init(this: &Piece, random: &Random, i: i32, x: i32, y: i32, z: i32, direction: i32, level: i32)
{
    final_castle_dungeon_room31::init(this, random, i, x, y, z, direction, level);
}

// ComponentTFFinalCastleDungeonExit.func_74861_a
pub fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    this.m_deco.set((parent).m_deco.get());
    let bestDir: i32 =
        final_castle_dungeon_exit::findStairDirectionTowards(this, parent.bounds().m_min[0], parent.bounds().m_min[2]);
    let steps0: Rc<Piece> = final_castle_dungeon_steps::new(
        random,
        5,
        this.m_box.get().m_min[0] + 15,
        this.m_box.get().m_min[1] + 0,
        this.m_box.get().m_min[2] + 15,
        bestDir,
    );
    list.push(Rc::clone(&steps0));
    steps0.build(this, list, random);
    if this.m_level.get() == 1
    {
        final_castle_dungeon_steps::buildLevelUnder(&steps0, &parent, list, random, this.m_level.get() + 1);
    }
    else
    {
        final_castle_dungeon_steps::buildBossRoomUnder(&steps0, &parent, list, random);
    }
}

// ComponentTFFinalCastleDungeonExit.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    final_castle_dungeon_room31::add(this, world, random, sbb);
    this.fillMeta(world, sbb, 7, 0, 16, 7, 3, 18, blk("tf:castleDoor"), 2, blk("air"), 0, false);
    this.fillMeta(
        world,
        sbb,
        7,
        4,
        16,
        7,
        4,
        18,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockMeta,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockMeta,
        false,
    );
    return true;
}

// ComponentTFFinalCastleDungeonExit.findStairDirectionTowards
pub fn findStairDirectionTowards(this: &Piece, x: i32, z: i32) -> i32
{
    let cx: i32 = this.m_box.get().centerX();
    let cz: i32 = this.m_box.get().centerZ();
    let dx: i32 = cx - x;
    let dz: i32 = cz - z;
    let mut absoluteDir: i32;
    if dz.abs() >= dx.abs()
    {
        absoluteDir = if dz >= 0 { 2 } else { 0 };
    }
    else
    {
        absoluteDir = if dx >= 0 { 3 } else { 1 };
    }
    return absoluteDir;
}

// ComponentTFFinalCastleDungeonExit.getForceFieldMeta
pub fn getForceFieldMeta(this: &Piece, decoRNG: &Random) -> i32 { return 1; }

// ComponentTFFinalCastleDungeonExit.getRuneMeta
pub fn getRuneMeta(this: &Piece, fieldMeta: i32) -> i32 { return 0; }
