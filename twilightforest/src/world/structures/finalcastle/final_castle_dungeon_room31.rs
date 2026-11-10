// draft from ComponentTFFinalCastleDungeonRoom31.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(random: &Random, i: i32, x: i32, y: i32, z: i32, direction: i32, level: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::FinalCastleDungeonRoom31);
    init(&this, random, i, x, y, z, direction, level);
    Rc::new(this)
}

// ComponentTFFinalCastleDungeonRoom31 constructor
pub fn init(this: &Piece, random: &Random, i: i32, x: i32, y: i32, z: i32, direction: i32, level: i32)
{
    tower_wing::init(this, i);
    this.setMode(direction);
    this.m_spawnListIndex.set(2);
    this.m_size.set(31);
    this.m_height.set(7);
    this.m_level.set(level);
    this.m_box.set(boxFor(
        x,
        y,
        z,
        -15,
        0,
        -15,
        this.m_size.get() - 1,
        this.m_height.get() - 1,
        this.m_size.get() - 1,
        0,
    ));
}

// ComponentTFFinalCastleDungeonRoom31.func_74861_a
pub fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    this.m_deco.set((parent).m_deco.get());
    let mySpread: i32 = this.depth() - parent.depth();
    let maxSpread: i32 = if this.m_level.get() == 1 { 2 } else { 3 };
    if (mySpread == maxSpread) && (!final_castle_dungeon_room31::isExitBuildForLevel(this, &parent))
    {
        let mut direction: i32 = random.nextIntBounded(4);
        {
            let mut i: i32 = 0;
            while (i < 8) && (!final_castle_dungeon_room31::isExitBuildForLevel(this, &parent))
            {
                direction = (direction + i) % 4;
                if final_castle_dungeon_room31::addDungeonExit(this, &parent, list, random, direction)
                {
                    final_castle_dungeon_room31::setExitBuiltForLevel(this, &parent, true);
                }
                i += 1;
            }
        }
    }
    if mySpread < maxSpread
    {
        let mut direction: i32 = random.nextIntBounded(4);
        for i in 0..12
        {
            direction = (direction + i) % 4;
            final_castle_dungeon_room31::addDungeonRoom(this, &parent, list, random, direction, this.m_level.get());
        }
    }
}

// ComponentTFFinalCastleDungeonRoom31.isExitBuildForLevel
pub fn isExitBuildForLevel(this: &Piece, parent: &Piece) -> bool
{
    return if matches!(parent.m_kind, Kind::FinalCastleDungeonEntrance)
    {
        (parent).m_hasExit.get()
    }
    else
    {
        false
    };
}

// ComponentTFFinalCastleDungeonRoom31.setExitBuiltForLevel
pub fn setExitBuiltForLevel(this: &Piece, parent: &Piece, exit: bool)
{
    if matches!(parent.m_kind, Kind::FinalCastleDungeonEntrance)
    {
        parent.m_hasExit.set(exit);
    }
}

// ComponentTFFinalCastleDungeonRoom31.addDungeonRoom
pub fn addDungeonRoom(
    this: &Piece,
    parent: &Piece,
    list: &mut Pieces,
    random: &Random,
    mut rotation: i32,
    level: i32,
) -> bool
{
    rotation = (this.m_mode.get() + rotation) % 4;
    let rc: [i32; 3] = final_castle_dungeon_room31::getNewRoomCoords(this, random, rotation);
    let dRoom: Rc<Piece> =
        final_castle_dungeon_room31::new(random, this.m_depth.get() + 1, rc[0], rc[1], rc[2], rotation, level);
    let mut largerBB: StructureBox = dRoom.bounds();
    let expand: i32 = 0;
    largerBB.m_min[0] -= expand;
    largerBB.m_min[2] -= expand;
    largerBB.m_max[0] += expand;
    largerBB.m_max[2] += expand;
    let intersect: Option<Rc<Piece>> = intersectingExcept(list, largerBB, this);
    if intersect.is_none()
    {
        list.push(Rc::clone(&dRoom));
        dRoom.build(&parent, list, random);
        return true;
    }
    else
    {
        return false;
    }
}

// ComponentTFFinalCastleDungeonRoom31.addDungeonExit
pub fn addDungeonExit(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random, mut rotation: i32) -> bool
{
    rotation = (this.m_mode.get() + rotation) % 4;
    let rc: [i32; 3] = final_castle_dungeon_room31::getNewRoomCoords(this, random, rotation);
    let dRoom: Rc<Piece> = final_castle_dungeon_exit::new(
        random,
        this.m_depth.get() + 1,
        rc[0],
        rc[1],
        rc[2],
        rotation,
        this.m_level.get(),
    );
    let intersect: Option<Rc<Piece>> = intersectingExcept(list, dRoom.bounds(), this);
    if intersect.is_none()
    {
        list.push(Rc::clone(&dRoom));
        dRoom.build(this, list, random);
        return true;
    }
    else
    {
        return false;
    }
}

// ComponentTFFinalCastleDungeonRoom31.getNewRoomCoords
pub fn getNewRoomCoords(this: &Piece, random: &Random, rotation: i32) -> [i32; 3]
{
    let mut offset: i32 = random.nextIntBounded(15) - 9;
    if random.nextBoolean()
    {
        offset += this.m_size.get();
    }
    let mut var10000: [i32; 3];
    match rotation
    {
        1 =>
        {
            var10000 = [this.m_box.get().m_min[0] + offset, this.m_box.get().m_min[1], this.m_box.get().m_max[2] + 9];
        }
        2 =>
        {
            var10000 = [this.m_box.get().m_min[0] - 9, this.m_box.get().m_min[1], this.m_box.get().m_min[2] + offset];
        }
        3 =>
        {
            var10000 = [this.m_box.get().m_min[0] + offset, this.m_box.get().m_min[1], this.m_box.get().m_min[2] - 9];
        }
        _ =>
        {
            var10000 = [this.m_box.get().m_max[0] + 9, this.m_box.get().m_min[1], this.m_box.get().m_min[2] + offset];
        }
    }
    return var10000;
}

// ComponentTFFinalCastleDungeonRoom31.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    if final_castle_dungeon_room31::isBoundingBoxOutOfPlateau(this, world, sbb)
    {
        return false;
    }
    let decoRNG: Random = Random::withSeed(
        (world.seed() + ((this.m_box.get().m_min[0] * 321534781) as i64))
            ^ ((this.m_box.get().m_min[2] * 756839) as i64),
    );
    this.fillAir(world, sbb, 0, 0, 0, this.m_size.get() - 1, this.m_height.get() - 1, this.m_size.get() - 1);
    let forceFieldMeta: i32 = this.getForceFieldMeta(&decoRNG);
    let runeMeta: i32 = this.getRuneMeta(forceFieldMeta);
    for rotation in 0..4
    {
        let cs: i32 = 7;
        this.fillTurned(
            world,
            sbb,
            cs,
            0,
            cs + 1,
            cs,
            this.m_height.get() - 1,
            (this.m_size.get() - 2) - cs,
            blk("tf:forceField"),
            forceFieldMeta,
            rotation,
        );
        for z in (cs..((this.m_size.get() - 1) - cs)).step_by(4)
        {
            this.fillTurned(
                world,
                sbb,
                cs,
                0,
                z,
                cs,
                this.m_height.get() - 1,
                z,
                blk("tf:castleMagic"),
                runeMeta,
                rotation,
            );
            let y: i32 = if ((z - cs) % 8) == 0
            {
                decoRNG.nextIntBounded(3) + 0
            }
            else
            {
                decoRNG.nextIntBounded(3) + 4
            };
            this.fillTurned(world, sbb, cs, y, z + 1, cs, y, z + 3, blk("tf:castleMagic"), runeMeta, rotation);
        }
    }
    return true;
}

// ComponentTFFinalCastleDungeonRoom31.isBoundingBoxOutOfPlateau
pub fn isBoundingBoxOutOfPlateau(this: &Piece, world: &GenWorld, sbb: StructureBox) -> bool
{
    let minX: i32 = this.m_box.get().m_min[0] - 1;
    let minZ: i32 = this.m_box.get().m_min[2] - 1;
    let maxX: i32 = this.m_box.get().m_max[0] + 1;
    let maxZ: i32 = this.m_box.get().m_max[2] + 1;
    for x in minX..=maxX
    {
        for z in minZ..=maxZ
        {
            if (world.biome(x, z) != Biome::FinalPlateau) && (world.biome(x, z) != Biome::Thornlands)
            {
                return true;
            }
        }
    }
    return false;
}

// ComponentTFFinalCastleDungeonRoom31.getRuneMeta
pub fn getRuneMeta(this: &Piece, forceFieldMeta: i32) -> i32 { return if forceFieldMeta == 4 { 1 } else { 2 }; }

// ComponentTFFinalCastleDungeonRoom31.getForceFieldMeta
pub fn getForceFieldMeta(this: &Piece, decoRNG: &Random) -> i32 { return decoRNG.nextIntBounded(2) + 3; }
