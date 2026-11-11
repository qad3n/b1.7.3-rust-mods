// draft from ComponentTFFinalCastleDungeonSteps.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(random: &Random, i: i32, x: i32, y: i32, z: i32, rotation: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::FinalCastleDungeonSteps);
    init(&this, random, i, x, y, z, rotation);
    Rc::new(this)
}

// ComponentTFFinalCastleDungeonSteps constructor
pub(crate) fn init(this: &Piece, _random: &Random, _i: i32, x: i32, y: i32, z: i32, rotation: i32)
{
    this.m_spawnListIndex.set(2);
    this.setMode(rotation);
    this.m_box.set(boxFor2(x, y, z, -2, -15, -3, 5, 15, 20, rotation));
}

// ComponentTFFinalCastleDungeonSteps.func_74861_a
pub(crate) fn build(this: &Piece, parent: &Piece, _list: &mut Pieces, _random: &Random)
{
    this.m_deco.set((parent).m_deco.get());
}

// ComponentTFFinalCastleDungeonSteps.buildMoreStepsTowards
pub(crate) fn buildMoreStepsTowards(
    this: &Piece,
    _parent: &Piece,
    list: &mut Pieces,
    random: &Random,
    rotation: i32,
) -> Rc<Piece>
{
    let direction: i32 = (rotation + this.m_mode.get()) % 4;
    let mut sx: i32 = 2;
    let sy: i32 = 0;
    let mut sz: i32 = 17;

    match rotation
    {
        0 =>
        {
            sz -= 5;
        }

        1 =>
        {
            sx -= 5;
        }

        2 =>
        {
            sz += 5;
        }

        3 =>
        {
            sx += 6;
        }

        _ =>
        {}
    }

    let dx: i32 = this.xAt(sx, sz);
    let dy: i32 = this.yAt(sy);
    let dz: i32 = this.zAt(sx, sz);
    let steps: Rc<Piece> = final_castle_dungeon_steps::new(random, this.m_depth.get() + 1, dx, dy, dz, direction);
    list.push(Rc::clone(&steps));
    steps.build(this, list, random);
    Rc::clone(&steps)
}

// ComponentTFFinalCastleDungeonSteps.buildLevelUnder
pub(crate) fn buildLevelUnder(
    this: &Piece,
    _parent: &Piece,
    list: &mut Pieces,
    random: &Random,
    level: i32,
) -> Rc<Piece>
{
    let dx: i32 = this.xAt(2, 19);
    let dy: i32 = this.yAt(-7);
    let dz: i32 = this.zAt(2, 19);
    let room: Rc<Piece> = final_castle_dungeon_entrance::new(random, 8, dx, dy, dz, this.m_mode.get(), level);
    list.push(Rc::clone(&room));
    room.build(this, list, random);
    Rc::clone(&room)
}

// ComponentTFFinalCastleDungeonSteps.buildBossRoomUnder
pub(crate) fn buildBossRoomUnder(this: &Piece, _parent: &Piece, list: &mut Pieces, random: &Random) -> Rc<Piece>
{
    let dx: i32 = this.xAt(2, 19);
    let dy: i32 = this.yAt(-31);
    let dz: i32 = this.zAt(2, 19);
    let room: Rc<Piece> = final_castle_dungeon_forge_room::new(random, 8, dx, dy, dz, this.m_mode.get());
    list.push(Rc::clone(&room));
    room.build(this, list, random);
    Rc::clone(&room)
}

// ComponentTFFinalCastleDungeonSteps.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, _random: &Random, sbb: StructureBox) -> bool
{
    for z in 0..15
    {
        let y: i32 = 14 - z;
        this.fillMeta(
            world,
            sbb,
            0,
            y,
            z,
            4,
            y,
            z,
            this.m_deco.get().m_stairID,
            this.stairMeta(3),
            this.m_deco.get().m_stairID,
            this.stairMeta(3),
            false,
        );
        this.fillAir(world, sbb, 0, y + 1, z, 4, y + 6, z);
    }

    this.fillAir(world, sbb, 0, 0, 15, 4, 5, 19);
    true
}
