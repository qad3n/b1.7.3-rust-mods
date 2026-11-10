// draft from ComponentTFFinalCastleMain.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(world: &GenWorld, random: &Random, i: i32, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::FinalCastleMain);
    init(&this, world, random, i, x, y, z);
    Rc::new(this)
}

// ComponentTFFinalCastleMain constructor
pub fn init(this: &Piece, world: &GenWorld, random: &Random, i: i32, mut x: i32, y: i32, mut z: i32)
{
    this.setMode(0);
    this.m_spawnListIndex.set(1);
    x = ((x + 127) >> 8) << 8;
    z = ((z + 127) >> 8) << 8;
    this.m_box.set(boxFor(x, y, z, -24, 120, -24, 48, 40, 48, 0));
    let cx: i32 = (x >> 8) << 8;
    let cz: i32 = (z >> 8) << 8;
    this.m_deco.set(Deco::decorator_castle());
}

// ComponentTFFinalCastleMain.func_74861_a
pub fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    let foundation: Rc<Piece> = final_castle_foundation48::new(random, 4, this);
    list.push(Rc::clone(&foundation));
    foundation.build(this, list, random);
    let roof: Rc<Piece> = final_castle_roof48_crenellated::new(random, 4, this);
    list.push(Rc::clone(&roof));
    roof.build(this, list, random);
    let gazebo: Rc<Piece> = final_castle_boss_gazebo::new(random, 5, this);
    list.push(Rc::clone(&gazebo));
    gazebo.build(this, list, random);
    let tower0: Rc<Piece> = final_castle_stair_tower::new(
        random,
        3,
        this.m_box.get().m_min[0],
        this.m_box.get().m_min[1] + 3,
        this.m_box.get().m_min[2],
        2,
    );
    list.push(Rc::clone(&tower0));
    tower0.build(this, list, random);
    let tower1: Rc<Piece> = final_castle_large_tower::new(
        random,
        3,
        this.m_box.get().m_max[0],
        this.m_box.get().m_min[1] + 3,
        this.m_box.get().m_min[2],
        3,
    );
    list.push(Rc::clone(&tower1));
    tower1.build(this, list, random);
    let tower2: Rc<Piece> = final_castle_stair_tower::new(
        random,
        3,
        this.m_box.get().m_min[0],
        this.m_box.get().m_min[1] + 3,
        this.m_box.get().m_max[2],
        1,
    );
    list.push(Rc::clone(&tower2));
    tower2.build(this, list, random);
    let tower3: Rc<Piece> = final_castle_stair_tower::new(
        random,
        3,
        this.m_box.get().m_max[0],
        this.m_box.get().m_min[1] + 3,
        this.m_box.get().m_max[2],
        0,
    );
    list.push(Rc::clone(&tower3));
    tower3.build(this, list, random);
    let mut dest: [i32; 3] = [this.m_box.get().m_min[0] - 4, this.m_box.get().m_max[1], this.m_box.get().m_min[2] - 24];
    final_castle_main::buildTowerMaze(this, list, random, 48, 0, 24, 60, 0, 0, dest);
    dest = [this.m_box.get().m_max[0] + 4, this.m_box.get().m_min[1], this.m_box.get().m_max[2] + 24];
    final_castle_main::buildTowerMaze(this, list, random, 0, 30, 24, 60, 2, 1, dest);
    let steps0: Rc<Piece> = final_castle_dungeon_steps::new(
        random,
        5,
        this.m_box.get().m_min[0] + 18,
        this.m_box.get().m_min[1] + 1,
        this.m_box.get().m_min[2] + 18,
        0,
    );
    list.push(Rc::clone(&steps0));
    steps0.build(this, list, random);
    let steps1: Rc<Piece> = final_castle_dungeon_steps::buildMoreStepsTowards(&steps0, &parent, list, random, 3);
    let steps2: Rc<Piece> = final_castle_dungeon_steps::buildMoreStepsTowards(&steps1, &parent, list, random, 3);
    let steps3: Rc<Piece> = final_castle_dungeon_steps::buildMoreStepsTowards(&steps2, &parent, list, random, 3);
    let dRoom: Rc<Piece> = final_castle_dungeon_steps::buildLevelUnder(&steps3, &parent, list, random, 1);
    let mc: [i32; 3] = this.offsetTowerAt(48, 23, 25, 1, 0);
    let ComponentTFFinalCastleMural0: Rc<Piece> = final_castle_mural::new(random, 7, mc[0], mc[1], mc[2], 35, 30, 0);
    list.push(Rc::clone(&ComponentTFFinalCastleMural0));
    ComponentTFFinalCastleMural0.build(this, list, random);
    let mc1: [i32; 3] = this.offsetTowerAt(48, 33, 24, -1, 0);
    let ComponentTFFinalCastleMural1: Rc<Piece> = final_castle_mural::new(random, 7, mc1[0], mc1[1], mc[2], 19, 12, 2);
    list.push(Rc::clone(&ComponentTFFinalCastleMural1));
    ComponentTFFinalCastleMural1.build(this, list, random);
}

// ComponentTFFinalCastleMain.buildTowerMaze
pub fn buildTowerMaze(
    this: &Piece,
    list: &mut Pieces,
    random: &Random,
    x: i32,
    y: i32,
    z: i32,
    howFar: i32,
    direction: i32,
    r#type: i32,
    dest: [i32; 3],
)
{
    // the source retries by recursion, a loop keeps the wasm stack small
    loop
    {
        let before: Pieces = list.clone();
        let tc: [i32; 3] = this.offsetTowerAt(x, y, z, howFar, direction);
        let sTower: Rc<Piece> = final_castle_maze_tower13::new(random, 3, tc[0], tc[1], tc[2], r#type, direction);
        let bc: [i32; 3] = this.offsetTowerAt(x, y, z, 1, direction);
        let bridge: Rc<Piece> = final_castle_bridge::new(this.depth() + 1, bc[0], bc[1], bc[2], howFar - 7, direction);
        list.push(Rc::clone(&bridge));
        bridge.build(this, list, random);
        list.push(Rc::clone(&sTower));
        final_castle_maze_tower13::buildTowards(&sTower, this, list, random, dest);
        if final_castle_main::isMazeComplete(this, list, r#type)
        {
            return;
        }

        *list = before;
    }
}

// ComponentTFFinalCastleMain.isMazeComplete
pub fn isMazeComplete(this: &Piece, list: &mut Pieces, r#type: i32) -> bool
{
    for structurecomponent in list.clone()
    {
        if ((matches!(structurecomponent.m_kind, Kind::FinalCastleEntranceTower)) && (r#type == 0))
            || ((matches!(structurecomponent.m_kind, Kind::FinalCastleBellTower21)) && (r#type == 1))
        {
            return true;
        }
    }
    return false;
}

// ComponentTFFinalCastleMain.offsetTowerCCoords
pub fn offsetTowerCCoords(this: &Piece, x: i32, y: i32, z: i32, howFar: i32, direction: i32) -> [i32; 3]
{
    let mut dx: i32 = this.xAt(x, z);
    let dy: i32 = this.yAt(y);
    let mut dz: i32 = this.zAt(x, z);
    match direction
    {
        0 =>
        {
            dx += howFar;
        }
        1 =>
        {
            dz += howFar;
        }
        2 =>
        {
            dx -= howFar;
        }
        3 =>
        {
            dz -= howFar;
        }
        _ =>
        {}
    }
    return [dx, dy, dz];
}

// ComponentTFFinalCastleMain.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    this.fillSelected(world, sbb, 0, 0, 0, 48, 40, 48, false, random, this.m_deco.get().m_randomBlocks);
    this.fillSelected(world, sbb, 13, 30, 1, 47, 30, 12, false, random, this.m_deco.get().m_randomBlocks);
    this.fill(world, sbb, 13, 31, 12, 36, 31, 12, this.m_deco.get().m_fenceID, this.m_deco.get().m_fenceID, false);
    this.fillSelected(world, sbb, 13, 30, 36, 47, 30, 47, false, random, this.m_deco.get().m_randomBlocks);
    this.fill(world, sbb, 13, 31, 36, 36, 31, 36, this.m_deco.get().m_fenceID, this.m_deco.get().m_fenceID, false);
    this.fillSelected(world, sbb, 1, 30, 1, 12, 30, 47, false, random, this.m_deco.get().m_randomBlocks);
    this.fill(world, sbb, 12, 31, 12, 12, 31, 36, this.m_deco.get().m_fenceID, this.m_deco.get().m_fenceID, false);
    this.fillSelected(world, sbb, 38, 25, 13, 47, 25, 35, false, random, this.m_deco.get().m_randomBlocks);
    for i in 0..5
    {
        let mut y: i32 = 30 - i;
        final_castle_main::makeMezzTopStairs(this, world, sbb, y, 10 + i, 3);
        final_castle_main::makeMezzTopStairs(this, world, sbb, y, 38 - i, 1);
        y = 25 - i;
        let mut x: i32 = 37 - i;
        this.fillMeta(
            world,
            sbb,
            x,
            y,
            14,
            x,
            y,
            22,
            this.m_deco.get().m_stairID,
            this.stairMeta(0),
            this.m_deco.get().m_stairID,
            this.stairMeta(0),
            false,
        );
        this.fillMeta(
            world,
            sbb,
            x,
            y - 1,
            14,
            x,
            y - 1,
            22,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            false,
        );
        this.fillMeta(
            world,
            sbb,
            x,
            y,
            26,
            x,
            y,
            34,
            this.m_deco.get().m_stairID,
            this.stairMeta(0),
            this.m_deco.get().m_stairID,
            this.stairMeta(0),
            false,
        );
        this.fillMeta(
            world,
            sbb,
            x,
            y - 1,
            26,
            x,
            y - 1,
            34,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            false,
        );
    }
    for x in (11..47).step_by(12)
    {
        for z in (11..47).step_by(12)
        {
            this.fillMeta(
                world,
                sbb,
                x,
                1,
                z,
                x + 2,
                40,
                z + 2,
                this.m_deco.get().m_pillarID,
                this.m_deco.get().m_pillarMeta,
                this.m_deco.get().m_blockID,
                this.m_deco.get().m_blockMeta,
                false,
            );
            final_castle_main::makePillarBase(this, world, sbb, x, z, 1, 0);
            final_castle_main::makePillarBase(this, world, sbb, x, z, 19, 4);
            final_castle_main::makePillarBase(this, world, sbb, x, z, 21, 0);
            final_castle_main::makePillarBase(this, world, sbb, x, z, 39, 4);
        }
    }
    for rotation in 0..4
    {
        for z in (11..47).step_by(12)
        {
            if (z != 23) || ((rotation != 0) && (rotation != 2))
            {
                this.fillTurned(
                    world,
                    sbb,
                    1,
                    1,
                    z,
                    1,
                    40,
                    z + 2,
                    this.m_deco.get().m_pillarID,
                    this.m_deco.get().m_pillarMeta,
                    rotation,
                );
                final_castle_main::makeHalfPillarBase(this, world, sbb, rotation, 1, z, 0);
                final_castle_main::makeHalfPillarBase(this, world, sbb, rotation, 19, z, 4);
                final_castle_main::makeHalfPillarBase(this, world, sbb, rotation, 21, z, 0);
                final_castle_main::makeHalfPillarBase(this, world, sbb, rotation, 39, z, 4);
            }
        }
    }
    this.fillSelected(world, sbb, 1, 20, 1, 47, 20, 47, false, random, this.m_deco.get().m_randomBlocks);
    let fieldBlock: Blk = blk("tf:forceField");
    let fieldMeta: i32 = 6;
    this.fillMeta(world, sbb, 12, 1, 12, 24, 10, 12, fieldBlock, fieldMeta, fieldBlock, fieldMeta, false);
    this.fillMeta(world, sbb, 12, 1, 12, 12, 10, 24, fieldBlock, fieldMeta, fieldBlock, fieldMeta, false);
    this.fillMeta(world, sbb, 24, 1, 12, 24, 10, 24, fieldBlock, fieldMeta, fieldBlock, fieldMeta, false);
    this.fillMeta(world, sbb, 12, 1, 24, 24, 10, 24, fieldBlock, fieldMeta, fieldBlock, fieldMeta, false);
    this.fillMeta(world, sbb, 12, 10, 12, 24, 10, 24, fieldBlock, fieldMeta, fieldBlock, fieldMeta, false);
    this.fillMeta(world, sbb, 17, 1, 12, 19, 4, 12, blk("tf:castleDoor"), 2, blk("air"), 0, false);
    this.fillMeta(world, sbb, 17, 1, 24, 19, 4, 24, blk("tf:castleDoor"), 2, blk("air"), 0, false);
    final_castle_main::makeSmallTowerStairs(this, world, sbb, 0);
    final_castle_main::makeSmallTowerStairs(this, world, sbb, 1);
    final_castle_main::makeSmallTowerStairs(this, world, sbb, 3);
    final_castle_main::makeComponentTFFinalCastleLargeTowerStairs(this, world, sbb, 2);
    this.fillMeta(world, sbb, 48, 1, 23, 48, 4, 25, blk("tf:castleDoor"), 0, blk("air"), 0, false);
    this.fillMeta(world, sbb, 0, 31, 23, 0, 34, 25, blk("tf:castleDoor"), 1, blk("air"), 0, false);
    return true;
}

// ComponentTFFinalCastleMain.makeSmallTowerStairs
pub fn makeSmallTowerStairs(this: &Piece, world: &GenWorld, sbb: StructureBox, rotation: i32)
{
    for y in 1..4
    {
        let z: i32 = 40 + y;
        this.fillTurned(
            world,
            sbb,
            1,
            1,
            z,
            4,
            y,
            z,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            rotation,
        );
        this.fillTurned(
            world,
            sbb,
            2,
            y,
            z,
            3,
            y,
            z,
            this.m_deco.get().m_stairID,
            this.stairMeta(1 + rotation),
            rotation,
        );
    }
}

// ComponentTFFinalCastleMain.makeComponentTFFinalCastleLargeTowerStairs
pub fn makeComponentTFFinalCastleLargeTowerStairs(this: &Piece, world: &GenWorld, sbb: StructureBox, rotation: i32)
{
    for y in 1..4
    {
        let z: i32 = 38 + y;
        this.fillTurned(
            world,
            sbb,
            2,
            1,
            z,
            6,
            y,
            z,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            rotation,
        );
        this.fillTurned(
            world,
            sbb,
            3,
            y,
            z,
            5,
            y,
            z,
            this.m_deco.get().m_stairID,
            this.stairMeta(1 + rotation),
            rotation,
        );
    }
}

// ComponentTFFinalCastleMain.makeMezzTopStairs
pub fn makeMezzTopStairs(this: &Piece, world: &GenWorld, sbb: StructureBox, y: i32, z: i32, stairMeta: i32)
{
    this.fillMeta(
        world,
        sbb,
        38,
        y,
        z,
        46,
        y,
        z,
        this.m_deco.get().m_stairID,
        this.stairMeta(stairMeta),
        this.m_deco.get().m_stairID,
        this.stairMeta(stairMeta),
        false,
    );
    this.fillMeta(
        world,
        sbb,
        38,
        y - 1,
        z,
        46,
        y - 1,
        z,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockMeta,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockMeta,
        false,
    );
    this.fillAir(world, sbb, 38, y + 1, z, 46, y + 3, z);
}

// ComponentTFFinalCastleMain.makeHalfPillarBase
pub fn makeHalfPillarBase(
    this: &Piece,
    world: &GenWorld,
    sbb: StructureBox,
    rotation: i32,
    y: i32,
    z: i32,
    metaBit: i32,
)
{
    this.fillTurned(
        world,
        sbb,
        2,
        y,
        z - 1,
        2,
        y,
        z + 3,
        this.m_deco.get().m_stairID,
        this.stairMeta(2 + rotation) | metaBit,
        rotation,
    );
    this.placeTurned(
        world,
        this.m_deco.get().m_stairID,
        this.stairMeta(1 + rotation) | metaBit,
        1,
        y,
        z - 1,
        rotation,
        sbb,
    );
    this.placeTurned(
        world,
        this.m_deco.get().m_stairID,
        this.stairMeta(3 + rotation) | metaBit,
        1,
        y,
        z + 3,
        rotation,
        sbb,
    );
}

// ComponentTFFinalCastleMain.makePillarBase
pub fn makePillarBase(this: &Piece, world: &GenWorld, sbb: StructureBox, x: i32, z: i32, y: i32, metaBit: i32)
{
    this.fillMeta(
        world,
        sbb,
        x + 0,
        y,
        z + 3,
        x + 3,
        y,
        z + 3,
        this.m_deco.get().m_stairID,
        this.stairMeta(3) | metaBit,
        blk("air"),
        0,
        false,
    );
    this.fillMeta(
        world,
        sbb,
        x - 1,
        y,
        z - 1,
        x + 2,
        y,
        z - 1,
        this.m_deco.get().m_stairID,
        this.stairMeta(1) | metaBit,
        blk("air"),
        0,
        false,
    );
    this.fillMeta(
        world,
        sbb,
        x + 3,
        y,
        z - 1,
        x + 3,
        y,
        z + 2,
        this.m_deco.get().m_stairID,
        this.stairMeta(2) | metaBit,
        blk("air"),
        0,
        false,
    );
    this.fillMeta(
        world,
        sbb,
        x - 1,
        y,
        z + 0,
        x - 1,
        y,
        z + 3,
        this.m_deco.get().m_stairID,
        this.stairMeta(0) | metaBit,
        blk("air"),
        0,
        false,
    );
}
