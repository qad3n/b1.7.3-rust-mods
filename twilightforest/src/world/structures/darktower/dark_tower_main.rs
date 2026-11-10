// draft from ComponentTFDarkTowerMain.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(random: &Random, index: i32, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::DarkTowerMain);
    init(&this, random, index, x, y, z);
    Rc::new(this)
}

pub fn new7(random: &Random, index: i32, x: i32, y: i32, z: i32, rotation: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::DarkTowerMain);
    init7(&this, random, index, x, y, z, rotation);
    Rc::new(this)
}

// ComponentTFDarkTowerMain constructor
pub fn init(this: &Piece, random: &Random, index: i32, x: i32, y: i32, z: i32)
{
    init7(this, random, index, x + 10, y, z + 10, 2);
    this.m_placedKeys.set(false);
}

// ComponentTFDarkTowerMain constructor
pub fn init7(this: &Piece, random: &Random, index: i32, x: i32, y: i32, z: i32, rotation: i32)
{
    dark_tower_wing::init(this, index, x, y, z, 19, 56 + ((random.nextIntBounded(32) / 5) * 5), rotation);
    this.m_placedKeys.set(false);
    if this.m_box.get().m_max[1] > 245
    {
        let amtToLower: i32 = (((this.m_box.get().m_max[1] - 245) / 5) * 5) + 5;
        this.m_height.set(this.m_height.get() - amtToLower);
        this.boxed(|bounds| bounds.m_max[1] -= amtToLower);
    }
    this.m_deco.set(Deco::dark_tower());
}

// ComponentTFDarkTowerMain.func_74861_a
pub fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    this.m_deco.set((parent).m_deco.get());
    if this.depth() > 0
    {
        this.addOpening(0, 1, this.m_size.get() / 2, 2);
    }
    let mut mainDir: i32 = -1;
    if this.depth() < 2
    {
        mainDir = random.nextIntBounded(4);
        for i in 0..4
        {
            if i != mainDir
            {
                let mut dest: Vec<i32> = this.getValidOpening(random, i);
                let childHeight: i32 = dark_tower_wing::validateChildHeight(this, 21 + random.nextIntBounded(10), 11);
                this.makeTowerWing(list, random, this.depth(), dest[0], dest[1], dest[2], 11, childHeight, i);
            }
        }
    }
    else
    {
        for i in 0..4
        {
            let mut dest: Vec<i32> = this.getValidOpening(random, i);
            dark_tower_main::makeBossTrapWing(this, list, random, this.depth(), dest[0], dest[1], dest[2], i);
        }
    }
    if this.depth() > 0
    {
        for i in 0..4
        {
            if i != 2
            {
                let mut dest: Vec<i32> = this.getValidOpening(random, i);
                dest[1] = 1;
                let childHeight: i32 = dark_tower_wing::validateChildHeight(this, 21 + random.nextIntBounded(10), 11);
                this.makeTowerWing(list, random, this.depth(), dest[0], dest[1], dest[2], 11, childHeight, i);
            }
        }
        this.makeABeard(&parent, list, random);
    }
    else
    {
        for i in (0..4).step_by(2)
        {
            let mut dest: Vec<i32> = this.getValidOpening(random, i);
            dest[1] = 1;
            let childHeight: i32 = dark_tower_wing::validateChildHeight(this, 10 + random.nextIntBounded(5), 9);
            this.makeEntranceTower(list, random, 5, dest[0], dest[1], dest[2], 9, childHeight, i);
        }
    }
    if mainDir > (-1)
    {
        let mut dest: Vec<i32> = this.getValidOpening(random, mainDir);
        dark_tower_main::makeNewLargeTower(this, list, random, this.depth() + 1, dest[0], dest[1], dest[2], mainDir);
    }
    this.makeARoof(&parent, list, random);
    if (!this.m_placedKeys.get()) && (this.depth() < 2)
    {
        let mut possibleKeyTowers: Pieces = Vec::new();
        let mut smallTowers: i32 = 0;
        for piece in list.clone()
        {
            if matches!(
                piece.m_kind,
                Kind::DarkTowerBossTrap | Kind::DarkTowerEntrance | Kind::DarkTowerMain | Kind::DarkTowerWing
            )
            {
                let wing: Rc<Piece> = piece;
                if (wing.m_size.get() == 9) && (wing.depth() == this.depth())
                {
                    smallTowers += 1;
                    possibleKeyTowers.push(Rc::clone(&wing));
                }
            }
        }
        for i in 0..4
        {
            if (possibleKeyTowers.len() as i32) < 1
            {
                break;
            }
            let towerNum: i32 = random.nextIntBounded((possibleKeyTowers.len() as i32));
            dark_tower_wing::setKeyTower(&Rc::clone(&possibleKeyTowers[(towerNum) as usize]), true);
            possibleKeyTowers.remove((towerNum) as usize);
        }
        this.m_placedKeys.set(true);
    }
}

// ComponentTFDarkTowerMain.makeEntranceTower
pub fn makeEntranceTower(
    this: &Piece,
    list: &mut Pieces,
    random: &Random,
    index: i32,
    x: i32,
    y: i32,
    z: i32,
    childSize: i32,
    childHeight: i32,
    rotation: i32,
) -> bool
{
    let direction: i32 = (this.mode() + rotation) % 4;
    let dx: Vec<i32> = this.offsetTower(x, y, z, 5, direction);
    let bridge: Rc<Piece> =
        dark_tower_entrance_bridge::new(index, dx[0], dx[1], dx[2], childSize, childHeight, direction);
    list.push(Rc::clone(&bridge));
    bridge.build(this, list, random);
    this.addOpening(x, y, z, rotation);
    return true;
}

// ComponentTFDarkTowerMain.makeNewLargeTower
pub fn makeNewLargeTower(
    this: &Piece,
    list: &mut Pieces,
    random: &Random,
    index: i32,
    x: i32,
    y: i32,
    z: i32,
    rotation: i32,
) -> bool
{
    let wingSize: i32 = 15;
    let wingHeight: i32 = 56;
    let direction: i32 = (this.mode() + rotation) % 4;
    let dx: Vec<i32> = this.offsetTower(x, y, z, 5, direction);
    let bridge: Rc<Piece> = dark_tower_main_bridge::new(index, dx[0], dx[1], dx[2], wingSize, wingHeight, direction);
    list.push(Rc::clone(&bridge));
    bridge.build(this, list, random);
    dark_tower_wing::addOpening5(this, x, y, z, rotation, Door::Locked);
    return true;
}

// ComponentTFDarkTowerMain.makeBossTrapWing
pub fn makeBossTrapWing(
    this: &Piece,
    list: &mut Pieces,
    random: &Random,
    index: i32,
    x: i32,
    y: i32,
    z: i32,
    rotation: i32,
) -> bool
{
    let wingSize: i32 = 11;
    let wingHeight: i32 = 9;
    let direction: i32 = (this.mode() + rotation) % 4;
    let dx: Vec<i32> = this.offsetTower(x, y, z, 5, direction);
    let bridge: Rc<Piece> = dark_tower_boss_bridge::new(index, dx[0], dx[1], dx[2], wingSize, wingHeight, direction);
    list.push(Rc::clone(&bridge));
    bridge.build(this, list, random);
    this.addOpening(x, y, z, rotation);
    return true;
}

// ComponentTFDarkTowerMain.makeARoof
pub fn makeARoof(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    if this.depth() < 2
    {
        dark_tower_wing::makeARoof(this, &parent, list, random);
    }
}

// ComponentTFDarkTowerMain.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    let decoRNG: Random = Random::withSeed(
        (world.seed() + ((this.m_box.get().m_min[0] * 321534781) as i64))
            ^ ((this.m_box.get().m_min[2] * 756839) as i64),
    );
    dark_tower_wing::makeEncasedWalls(
        this,
        world,
        random,
        sbb,
        0,
        0,
        0,
        this.m_size.get() - 1,
        this.m_height.get() - 1,
        this.m_size.get() - 1,
    );
    this.fillAir(world, sbb, 1, 1, 1, this.m_size.get() - 2, this.m_height.get() - 2, this.m_size.get() - 2);
    if this.depth() == 0
    {
        for x in 0..this.m_size.get()
        {
            for z in 0..this.m_size.get()
            {
                this.fillDown(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, x, -1, z, sbb);
            }
        }
    }
    this.darken(world);
    let totalFloors: i32 = this.m_height.get() / 5;
    let beamMaze: bool = decoRNG.nextBoolean();
    let mut centerFloors: i32 = if beamMaze { 4 } else { totalFloors / 2 };
    let bottomFloors: i32 = (totalFloors - centerFloors) / 2;
    centerFloors = totalFloors - (bottomFloors * 2);
    let topFloorsStartY: i32 = this.m_height.get() - ((bottomFloors * 5) + 1);
    dark_tower_main::addThreeQuarterFloors(this, world, &decoRNG, sbb, 0, bottomFloors * 5);
    if this.depth() < 2
    {
        dark_tower_main::addThreeQuarterFloors(this, world, &decoRNG, sbb, topFloorsStartY, this.m_height.get() - 1);
    }
    else
    {
        dark_tower_main::addThreeQuarterFloorsDecorateBoss(
            this,
            world,
            &decoRNG,
            sbb,
            topFloorsStartY,
            this.m_height.get() - 1,
        );
        dark_tower_wing::destroyTower7(this, world, &decoRNG, 12, this.m_height.get() + 4, 3, 4, sbb);
        dark_tower_wing::destroyTower7(this, world, &decoRNG, 3, this.m_height.get() + 4, 12, 4, sbb);
        dark_tower_wing::destroyTower7(this, world, &decoRNG, 3, this.m_height.get() + 4, 3, 4, sbb);
        dark_tower_wing::destroyTower7(this, world, &decoRNG, 12, this.m_height.get() + 4, 12, 4, sbb);
        dark_tower_wing::destroyTower7(this, world, &decoRNG, 8, this.m_height.get() + 4, 8, 5, sbb);
        dark_tower_main::decorateBossSpawner(this, world, &decoRNG, sbb, 0, this.m_height.get() - 6);
    }
    if beamMaze
    {
        dark_tower_main::addTimberMaze(this, world, &decoRNG, sbb, bottomFloors * 5, topFloorsStartY);
    }
    else
    {
        dark_tower_main::addBuilderPlatforms(this, world, &decoRNG, sbb, bottomFloors * 5, topFloorsStartY);
    }
    this.makeOpenings(world, sbb);
    return true;
}

// ComponentTFDarkTowerMain.addThreeQuarterFloors
pub fn addThreeQuarterFloors(
    this: &Piece,
    world: &GenWorld,
    decoRNG: &Random,
    sbb: StructureBox,
    mut bottom: i32,
    top: i32,
)
{
    let spacing: i32 = 5;
    let mut rotation: i32 = (this.m_box.get().m_min[1] + bottom) % 4;
    if bottom == 0
    {
        dark_tower_main::makeLargeStairsUp(this, world, sbb, rotation, 0);
        let var12: i32 = rotation + 3;
        rotation = var12 % 4;
        dark_tower_main::makeBottomEntrance(this, world, decoRNG, sbb, rotation, bottom);
        bottom += spacing;
    }
    {
        let mut y: i32 = bottom;
        while y < top
        {
            let isBottomFloor: bool = (y == bottom) && (bottom != spacing);
            let isTopFloor: bool = y >= (top - spacing);
            let isTowerTopFloor: bool = y >= ((this.m_height.get() - spacing) - 2);
            dark_tower_main::makeThreeQuarterFloor(this, world, sbb, rotation, y, isBottomFloor, isTowerTopFloor);
            if !isTopFloor
            {
                dark_tower_main::makeLargeStairsUp(this, world, sbb, rotation, y);
            }
            if (!isTopFloor) || isTowerTopFloor
            {
                dark_tower_main::decorateFloor(this, world, decoRNG, sbb, rotation, y, isBottomFloor, isTopFloor);
            }
            rotation += 3;
            rotation %= 4;
            y += spacing;
        }
    }
}

// ComponentTFDarkTowerMain.addThreeQuarterFloorsDecorateBoss
pub fn addThreeQuarterFloorsDecorateBoss(
    this: &Piece,
    world: &GenWorld,
    decoRNG: &Random,
    sbb: StructureBox,
    mut bottom: i32,
    top: i32,
)
{
    let spacing: i32 = 5;
    let mut rotation: i32 = (this.m_box.get().m_min[1] + bottom) % 4;
    if bottom == 0
    {
        dark_tower_main::makeLargeStairsUp(this, world, sbb, rotation, 0);
        rotation += 3;
        rotation %= 4;
        bottom += spacing;
    }
    {
        let mut y: i32 = bottom;
        while y < top
        {
            let isBottomFloor: bool = (y == bottom) && (bottom != spacing);
            let isTopFloor: bool = y >= (top - spacing);
            let isTowerTopFloor: bool = y >= ((this.m_height.get() - spacing) - 2);
            dark_tower_main::makeThreeQuarterFloor(this, world, sbb, rotation, y, isBottomFloor, isTowerTopFloor);
            if !isTopFloor
            {
                dark_tower_main::makeLargeStairsUp(this, world, sbb, rotation, y);
                dark_tower_main::decorateExperiment(this, world, decoRNG, sbb, rotation, y);
            }
            rotation += 3;
            rotation %= 4;
            y += spacing;
        }
    }
}

// ComponentTFDarkTowerMain.decorateFloor
pub fn decorateFloor(
    this: &Piece,
    world: &GenWorld,
    decoRNG: &Random,
    sbb: StructureBox,
    rotation: i32,
    y: i32,
    isBottom: bool,
    isTop: bool,
)
{
    if isTop
    {
        match decoRNG.nextIntBounded(3)
        {
            1 =>
            {
                dark_tower_main::decorateBotanical(this, world, decoRNG, sbb, rotation, y);
            }
            2 =>
            {
                dark_tower_main::decorateNetherwart(this, world, decoRNG, sbb, rotation, y, isTop);
            }
            _ =>
            {
                dark_tower_main::decorateAquarium(this, world, decoRNG, sbb, rotation, y);
            }
        }
    }
    else if isBottom
    {
        'case2:
        {
            match decoRNG.nextIntBounded(4)
            {
                0 =>
                {
                    dark_tower_main::decorateAquarium(this, world, decoRNG, sbb, rotation, y);
                }
                1 =>
                {
                    dark_tower_main::decorateBotanical(this, world, decoRNG, sbb, rotation, y);
                }
                // todo falls through to the next case
                2 =>
                {
                    if (y + this.m_box.get().m_min[1]) > 64
                    {
                        dark_tower_main::decorateNetherwart(this, world, decoRNG, sbb, rotation, y, isTop);
                        break 'case2;
                    }
                }
                3 =>
                {
                    dark_tower_main::decorateForge(this, world, decoRNG, sbb, rotation, y);
                }
                _ =>
                {}
            }
        }
    }
    else
    {
        'case2:
        {
            match decoRNG.nextIntBounded(8)
            {
                0 | 1 =>
                {
                    dark_tower_main::decorateReappearingMaze(this, world, decoRNG, sbb, rotation, y);
                }
                2 =>
                {
                    dark_tower_main::decorateUnbuilderMaze(this, world, decoRNG, sbb, rotation, y);
                }
                3 =>
                {
                    dark_tower_main::decorateAquarium(this, world, decoRNG, sbb, rotation, y);
                }
                4 =>
                {
                    dark_tower_main::decorateBotanical(this, world, decoRNG, sbb, rotation, y);
                }
                // todo falls through to the next case
                5 =>
                {
                    if (y + this.m_box.get().m_min[1]) > 64
                    {
                        dark_tower_main::decorateNetherwart(this, world, decoRNG, sbb, rotation, y, isTop);
                        break 'case2;
                    }
                }
                6 =>
                {
                    this.decorateLounge(world, decoRNG, sbb, rotation, y);
                }
                7 =>
                {
                    dark_tower_main::decorateForge(this, world, decoRNG, sbb, rotation, y);
                }
                _ =>
                {}
            }
        }
    }
}

// ComponentTFDarkTowerMain.makeThreeQuarterFloor
pub fn makeThreeQuarterFloor(
    this: &Piece,
    world: &GenWorld,
    sbb: StructureBox,
    rotation: i32,
    y: i32,
    isBottom: bool,
    isTowerTopFloor: bool,
)
{
    let half: i32 = this.m_size.get() / 2;
    this.fillTurned(
        world,
        sbb,
        half + 1,
        y,
        1,
        this.m_size.get() - 2,
        y,
        half + 1,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockMeta,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        1,
        y,
        half + 1,
        this.m_size.get() - 2,
        y,
        this.m_size.get() - 2,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockMeta,
        rotation,
    );
    let startZ: i32 = if isBottom { 1 } else { 3 };
    this.fillTurned(
        world,
        sbb,
        1,
        y,
        half,
        half,
        y,
        half,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        half,
        y,
        startZ,
        half,
        y,
        half,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        1,
        y + 1,
        half,
        half,
        y + 1,
        half,
        this.m_deco.get().m_fenceID,
        this.m_deco.get().m_fenceMeta,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        half,
        y + 1,
        startZ,
        half,
        y + 1,
        half,
        this.m_deco.get().m_fenceID,
        this.m_deco.get().m_fenceMeta,
        rotation,
    );
    if isTowerTopFloor
    {
        this.fillTurned(
            world,
            sbb,
            1,
            y + 0,
            half - 2,
            3,
            y + 0,
            half,
            this.m_deco.get().m_accentID,
            this.m_deco.get().m_accentMeta,
            rotation,
        );
        this.fillTurned(
            world,
            sbb,
            1,
            y + 1,
            half - 2,
            3,
            y + 1,
            half,
            this.m_deco.get().m_fenceID,
            this.m_deco.get().m_fenceMeta,
            rotation,
        );
        this.fillTurned(
            world,
            sbb,
            1,
            y + 0,
            half - 1,
            2,
            y + 0,
            half,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            rotation,
        );
        this.fillTurned(world, sbb, 1, y + 1, half - 1, 2, y + 1, half, blk("air"), 0, rotation);
    }
}

// ComponentTFDarkTowerMain.makeLargeStairsUp
pub fn makeLargeStairsUp(this: &Piece, world: &GenWorld, sbb: StructureBox, rotation: i32, y: i32)
{
    for i in 0..5
    {
        let z: i32 = ((this.m_size.get() / 2) - i) + 4;
        let sy: i32 = (y + i) + 1;
        this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(3 + rotation), 1, sy, z, rotation, sbb);
        this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(3 + rotation), 2, sy, z, rotation, sbb);
        this.placeTurned(
            world,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            1,
            sy,
            z - 1,
            rotation,
            sbb,
        );
        this.placeTurned(
            world,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            2,
            sy,
            z - 1,
            rotation,
            sbb,
        );
        this.placeTurned(
            world,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            3,
            sy,
            z - 1,
            rotation,
            sbb,
        );
        if (i > 0) && (i < 4)
        {
            this.placeTurned(
                world,
                this.m_deco.get().m_accentID,
                this.m_deco.get().m_accentMeta,
                3,
                sy,
                z,
                rotation,
                sbb,
            );
            this.placeTurned(
                world,
                this.m_deco.get().m_fenceID,
                this.m_deco.get().m_fenceMeta,
                3,
                sy + 1,
                z,
                rotation,
                sbb,
            );
            this.placeTurned(
                world,
                this.m_deco.get().m_fenceID,
                this.m_deco.get().m_fenceMeta,
                3,
                sy + 2,
                z,
                rotation,
                sbb,
            );
        }
        else if i == 0
        {
            this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(2 + rotation), 3, sy, z, rotation, sbb);
        }
    }
}

// ComponentTFDarkTowerMain.decorateReappearingMaze
pub fn decorateReappearingMaze(
    this: &Piece,
    world: &GenWorld,
    decoRNG: &Random,
    sbb: StructureBox,
    rotation: i32,
    y: i32,
)
{
    let mazeSize: i32 = 6;
    let mut maze: Rc<Maze> = Rc::new(Maze::new(mazeSize, mazeSize));
    maze.setSeed(
        ((world.seed() + ((this.m_box.get().m_min[0] * 90342903) as i64)) + ((y * 90342903) as i64))
            ^ (this.m_box.get().m_min[2]) as i64,
    );
    for i in 0..13
    {
        maze.putRaw(i, 0, 5);
        maze.putRaw(i, 12, 5);
        maze.putRaw(0, i, 5);
        maze.putRaw(12, i, 5);
    }
    maze.m_doorRarity.set(0.3);
    match rotation
    {
        0 =>
        {
            for x in 1..6
            {
                for z in 1..6
                {
                    maze.putRaw(x, z, 5);
                }
            }
            maze.putRaw(1, 6, 5);
            maze.putRaw(1, 7, 5);
            maze.putRaw(1, 8, 5);
            maze.putRaw(1, 9, 5);
            maze.putRaw(1, 10, 6);
            maze.putRaw(6, 1, 5);
            maze.putRaw(7, 1, 5);
            maze.putRaw(8, 1, 6);
            maze.generateRecursiveBacktracker(0, 5);
        }
        1 =>
        {
            for x in 7..12
            {
                for z in 1..6
                {
                    maze.putRaw(x, z, 5);
                }
            }
            maze.putRaw(6, 1, 5);
            maze.putRaw(5, 1, 5);
            maze.putRaw(4, 1, 5);
            maze.putRaw(3, 1, 5);
            maze.putRaw(2, 1, 6);
            maze.putRaw(11, 6, 5);
            maze.putRaw(11, 7, 5);
            maze.putRaw(11, 8, 6);
            maze.generateRecursiveBacktracker(0, 0);
        }
        2 =>
        {
            for x in 7..12
            {
                for z in 7..12
                {
                    maze.putRaw(x, z, 5);
                }
            }
            maze.putRaw(11, 6, 5);
            maze.putRaw(11, 5, 5);
            maze.putRaw(11, 4, 5);
            maze.putRaw(11, 3, 5);
            maze.putRaw(11, 2, 6);
            maze.putRaw(6, 11, 5);
            maze.putRaw(5, 11, 5);
            maze.putRaw(4, 11, 6);
            maze.generateRecursiveBacktracker(5, 0);
        }
        3 =>
        {
            for x in 1..6
            {
                for z in 7..12
                {
                    maze.putRaw(x, z, 5);
                }
            }
            maze.putRaw(6, 11, 5);
            maze.putRaw(7, 11, 5);
            maze.putRaw(8, 11, 5);
            maze.putRaw(9, 11, 5);
            maze.putRaw(10, 11, 6);
            maze.putRaw(1, 6, 5);
            maze.putRaw(1, 5, 5);
            maze.putRaw(1, 4, 6);
            maze.generateRecursiveBacktracker(5, 5);
        }
        _ =>
        {}
    }
    maze.m_wallBlockID.set(this.m_deco.get().m_blockID);
    maze.m_wallBlockMeta.set(this.m_deco.get().m_blockMeta);
    maze.m_headBlockID.set(this.m_deco.get().m_accentID);
    maze.m_headBlockMeta.set(this.m_deco.get().m_accentMeta);
    maze.m_pillarBlockID.set(this.m_deco.get().m_accentID);
    maze.m_pillarBlockMeta.set(this.m_deco.get().m_accentMeta);
    maze.m_doorBlockID.set(blk("tf:towerDevice"));
    maze.m_doorBlockMeta.set(0);
    maze.m_torchRarity.set(0.0);
    maze.m_tall.set(3);
    maze.m_head.set(1);
    maze.m_oddBias.set(2);
    maze.copyToStructure(world, 0, y + 1, 0, this, sbb);
    dark_tower_main::decorateMazeDeadEnds(this, world, decoRNG, &maze, y, rotation, sbb);
}

// ComponentTFDarkTowerMain.decorateMazeDeadEnds
pub fn decorateMazeDeadEnds(
    this: &Piece,
    world: &GenWorld,
    decoRNG: &Random,
    maze: &Maze,
    y: i32,
    rotation: i32,
    sbb: StructureBox,
)
{
    for x in 0..maze.m_width.get()
    {
        for z in 0..maze.m_depth.get()
        {
            if (((!maze.isWall(x, z, x - 1, z)) && maze.isWall(x, z, x + 1, z)) && maze.isWall(x, z, x, z - 1))
                && maze.isWall(x, z, x, z + 1)
            {
                dark_tower_main::decorateDeadEnd9(this, world, decoRNG, maze, x, y, z, 3, rotation, sbb);
            }
            if ((maze.isWall(x, z, x - 1, z) && (!maze.isWall(x, z, x + 1, z))) && maze.isWall(x, z, x, z - 1))
                && maze.isWall(x, z, x, z + 1)
            {
                dark_tower_main::decorateDeadEnd9(this, world, decoRNG, maze, x, y, z, 1, rotation, sbb);
            }
            if ((maze.isWall(x, z, x - 1, z) && maze.isWall(x, z, x + 1, z)) && (!maze.isWall(x, z, x, z - 1)))
                && maze.isWall(x, z, x, z + 1)
            {
                dark_tower_main::decorateDeadEnd9(this, world, decoRNG, maze, x, y, z, 0, rotation, sbb);
            }
            if ((maze.isWall(x, z, x - 1, z) && maze.isWall(x, z, x + 1, z)) && maze.isWall(x, z, x, z - 1))
                && (!maze.isWall(x, z, x, z + 1))
            {
                dark_tower_main::decorateDeadEnd9(this, world, decoRNG, maze, x, y, z, 2, rotation, sbb);
            }
        }
    }
}

// ComponentTFDarkTowerMain.decorateDeadEnd
pub fn decorateDeadEnd9(
    this: &Piece,
    world: &GenWorld,
    decoRNG: &Random,
    maze: &Maze,
    mx: i32,
    y: i32,
    mz: i32,
    facing: i32,
    rotation: i32,
    sbb: StructureBox,
)
{
    let x: i32 = (mx * 3) + 1;
    let z: i32 = (mz * 3) + 1;
    match facing
    {
        0 =>
        {
            this.placeBlock(
                world,
                this.m_deco.get().m_accentID,
                this.m_deco.get().m_accentMeta,
                x + 0,
                y + 1,
                z + 1,
                sbb,
            );
            this.placeBlock(
                world,
                this.m_deco.get().m_accentID,
                this.m_deco.get().m_accentMeta,
                x + 1,
                y + 1,
                z + 1,
                sbb,
            );
            this.placeBlock(world, blk("chest"), 0, x + 0, y + 2, z + 1, sbb);
            this.placeTreasure(world, decoRNG, x + 1, y + 2, z + 1, Table::DarkTowerCache, sbb);
        }
        1 =>
        {
            this.placeBlock(
                world,
                this.m_deco.get().m_accentID,
                this.m_deco.get().m_accentMeta,
                x + 0,
                y + 1,
                z + 0,
                sbb,
            );
            this.placeBlock(
                world,
                this.m_deco.get().m_accentID,
                this.m_deco.get().m_accentMeta,
                x + 0,
                y + 1,
                z + 1,
                sbb,
            );
            this.placeBlock(world, blk("chest"), rotation, x + 0, y + 2, z + 0, sbb);
            this.placeTreasure(world, decoRNG, x + 0, y + 2, z + 1, Table::DarkTowerCache, sbb);
        }
        2 =>
        {
            this.placeBlock(
                world,
                this.m_deco.get().m_accentID,
                this.m_deco.get().m_accentMeta,
                x + 0,
                y + 1,
                z + 0,
                sbb,
            );
            this.placeBlock(
                world,
                this.m_deco.get().m_accentID,
                this.m_deco.get().m_accentMeta,
                x + 1,
                y + 1,
                z + 0,
                sbb,
            );
            this.placeBlock(world, blk("chest"), rotation, x + 0, y + 2, z + 0, sbb);
            this.placeTreasure(world, decoRNG, x + 1, y + 2, z + 0, Table::DarkTowerCache, sbb);
        }
        3 =>
        {
            this.placeBlock(
                world,
                this.m_deco.get().m_accentID,
                this.m_deco.get().m_accentMeta,
                x + 1,
                y + 1,
                z + 0,
                sbb,
            );
            this.placeBlock(
                world,
                this.m_deco.get().m_accentID,
                this.m_deco.get().m_accentMeta,
                x + 1,
                y + 1,
                z + 1,
                sbb,
            );
            this.placeBlock(world, blk("chest"), rotation, x + 1, y + 2, z + 0, sbb);
            this.placeTreasure(world, decoRNG, x + 1, y + 2, z + 1, Table::DarkTowerCache, sbb);
        }
        _ =>
        {}
    }
}

// ComponentTFDarkTowerMain.decorateUnbuilderMaze
pub fn decorateUnbuilderMaze(this: &Piece, world: &GenWorld, decoRNG: &Random, sbb: StructureBox, rotation: i32, y: i32)
{
    for x in this.m_size.get() / 2..(this.m_size.get() - 1)
    {
        for z in 3..(this.m_size.get() - 1)
        {
            if ((x % 2) == 1) && ((z % 2) == 1)
            {
                for py in 1..5
                {
                    this.placeTurned(
                        world,
                        this.m_deco.get().m_pillarID,
                        this.m_deco.get().m_pillarMeta,
                        x,
                        y + py,
                        z,
                        rotation,
                        sbb,
                    );
                }
            }
            else if ((x % 2) == 1) || ((z % 2) == 1)
            {
                for py in 1..5
                {
                    this.placeTurned(
                        world,
                        this.m_deco.get().m_fenceID,
                        this.m_deco.get().m_fenceMeta,
                        x,
                        y + py,
                        z,
                        rotation,
                        sbb,
                    );
                }
                if ((x != (this.m_size.get() / 2)) && (x != (this.m_size.get() - 2))) && (z != (this.m_size.get() - 2))
                {
                    let mut ay: i32 = decoRNG.nextIntBounded(4) + 1;
                    this.placeTurned(world, blk("air"), 0, x, y + ay, z, rotation, sbb);
                    if x > (this.m_size.get() - 7)
                    {
                        ay = decoRNG.nextIntBounded(3) + 1;
                        this.placeTurned(world, blk("air"), 0, x, y + ay, z, rotation, sbb);
                    }
                }
            }
        }
    }
    this.placeTurned(world, blk("tf:towerDevice"), 9, 15, y + 2, 7, rotation, sbb);
    this.placeTurned(world, blk("tf:towerDevice"), 9, 11, y + 3, 7, rotation, sbb);
    this.placeTurned(world, blk("tf:towerDevice"), 9, 15, y + 2, 13, rotation, sbb);
    this.placeTurned(world, blk("tf:towerDevice"), 9, 11, y + 3, 13, rotation, sbb);
    this.placeTurned(world, blk("tf:towerDevice"), 9, 5, y + 3, 13, rotation, sbb);
}

// ComponentTFDarkTowerMain.decorateLounge
pub fn decorateLounge(this: &Piece, world: &GenWorld, decoRNG: &Random, sbb: StructureBox, rotation: i32, y: i32)
{
    this.fillTurned(
        world,
        sbb,
        17,
        y + 1,
        1,
        17,
        y + 4,
        6,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        12,
        y + 1,
        1,
        17,
        y + 4,
        1,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        13,
        y + 1,
        2,
        16,
        y + 1,
        5,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockMeta,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        12,
        y + 1,
        2,
        12,
        y + 1,
        6,
        this.m_deco.get().m_stairID,
        this.stairMeta(0 + rotation),
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        12,
        y + 1,
        6,
        16,
        y + 1,
        6,
        this.m_deco.get().m_stairID,
        this.stairMeta(3 + rotation),
        rotation,
    );
    dark_tower_main::makeDispenserPillar(
        this,
        world,
        this.m_deco.get(),
        13,
        y,
        1,
        this.stairMeta(3 + rotation),
        rotation,
        sbb,
    );
    dark_tower_main::makeDispenserPillar(
        this,
        world,
        this.m_deco.get(),
        15,
        y,
        1,
        this.stairMeta(3 + rotation),
        rotation,
        sbb,
    );
    dark_tower_main::makeDispenserPillar(
        this,
        world,
        this.m_deco.get(),
        17,
        y,
        3,
        this.stairMeta(0 + rotation),
        rotation,
        sbb,
    );
    dark_tower_main::makeDispenserPillar(
        this,
        world,
        this.m_deco.get(),
        17,
        y,
        5,
        this.stairMeta(0 + rotation),
        rotation,
        sbb,
    );
    dark_tower_main::makeStonePillar(
        this,
        world,
        this.m_deco.get(),
        12,
        y,
        1,
        this.stairMeta(3 + rotation),
        rotation,
        sbb,
    );
    dark_tower_main::makeStonePillar(
        this,
        world,
        this.m_deco.get(),
        17,
        y,
        6,
        this.stairMeta(0 + rotation),
        rotation,
        sbb,
    );
    this.placeTurned(world, blk("brewing_stand"), 0, 13, y + 2, 5, rotation, sbb);
    this.placeTurned(world, blk("cauldron"), 3, 15, y + 2, 3, rotation, sbb);
    this.fillTurned(
        world,
        sbb,
        10,
        y + 1,
        17,
        17,
        y + 4,
        17,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockMeta,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        17,
        y + 1,
        10,
        17,
        y + 4,
        17,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockMeta,
        rotation,
    );
    this.fillTurned(world, sbb, 11, y + 1, 17, 12, y + 4, 17, blk("bookshelf"), 0, rotation);
    this.fillTurned(world, sbb, 14, y + 1, 17, 15, y + 4, 17, blk("bookshelf"), 0, rotation);
    this.fillTurned(world, sbb, 17, y + 1, 11, 17, y + 4, 12, blk("bookshelf"), 0, rotation);
    this.fillTurned(world, sbb, 17, y + 1, 14, 17, y + 4, 15, blk("bookshelf"), 0, rotation);
    this.placeTurned(
        world,
        this.m_deco.get().m_stairID,
        this.stairMeta(0 + rotation) + 4,
        13,
        y + 1,
        14,
        rotation,
        sbb,
    );
    this.placeTurned(
        world,
        this.m_deco.get().m_stairID,
        this.stairMeta(3 + rotation) + 4,
        14,
        y + 1,
        14,
        rotation,
        sbb,
    );
    this.placeTurned(
        world,
        this.m_deco.get().m_stairID,
        this.stairMeta(2 + rotation) + 4,
        14,
        y + 1,
        13,
        rotation,
        sbb,
    );
    this.placeTurned(
        world,
        this.m_deco.get().m_stairID,
        this.stairMeta(1 + rotation) + 4,
        13,
        y + 1,
        13,
        rotation,
        sbb,
    );
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(2 + rotation), 11, y + 1, 13, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(3 + rotation), 13, y + 1, 11, rotation, sbb);
    this.placeTurned(world, blk("redstone_lamp"), 0, 8, y + 3, 8, rotation, sbb);
    this.placeTurned(world, blk("lever"), if decoRNG.nextBoolean() { 0 } else { 7 }, 8, y + 2, 8, rotation, sbb);
    dark_tower_main::placeTreePlanter(this, world, decoRNG.nextIntBounded(5), 6, y + 1, 12, rotation, sbb);
}

// ComponentTFDarkTowerMain.makeDispenserPillar
pub fn makeDispenserPillar(
    this: &Piece,
    world: &GenWorld,
    forgeDeco: Deco,
    x: i32,
    y: i32,
    z: i32,
    stairMeta: i32,
    rotation: i32,
    sbb: StructureBox,
)
{
    this.placeTurned(world, forgeDeco.m_stairID, stairMeta + 4, x, y + 2, z, rotation, sbb);
    this.placeTurned(world, blk("dispenser"), stairMeta + 4, x, y + 3, z, rotation, sbb);
    this.placeTurned(world, forgeDeco.m_stairID, stairMeta, x, y + 4, z, rotation, sbb);
}

// ComponentTFDarkTowerMain.decorateBossSpawner
pub fn decorateBossSpawner(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox, rotation: i32, y: i32)
{
    this.placeTurned(world, blk("tf:bossSpawner"), 3, 9, y + 4, 9, rotation, sbb);
}

// ComponentTFDarkTowerMain.decorateExperiment
pub fn decorateExperiment(this: &Piece, world: &GenWorld, decoRNG: &Random, sbb: StructureBox, rotation: i32, y: i32)
{
    this.fillTurned(
        world,
        sbb,
        17,
        y + 1,
        1,
        17,
        y + 4,
        6,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        12,
        y + 1,
        1,
        17,
        y + 4,
        1,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        13,
        y + 1,
        2,
        16,
        y + 1,
        5,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockMeta,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        12,
        y + 1,
        2,
        12,
        y + 1,
        6,
        this.m_deco.get().m_stairID,
        this.stairMeta(0 + rotation),
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        12,
        y + 1,
        6,
        16,
        y + 1,
        6,
        this.m_deco.get().m_stairID,
        this.stairMeta(3 + rotation),
        rotation,
    );
    dark_tower_main::makeWoodPillar(
        this,
        world,
        this.m_deco.get(),
        13,
        y,
        1,
        this.stairMeta(3 + rotation),
        rotation,
        sbb,
    );
    dark_tower_main::makeWoodPillar(
        this,
        world,
        this.m_deco.get(),
        15,
        y,
        1,
        this.stairMeta(3 + rotation),
        rotation,
        sbb,
    );
    dark_tower_main::makeWoodPillar(
        this,
        world,
        this.m_deco.get(),
        17,
        y,
        3,
        this.stairMeta(0 + rotation),
        rotation,
        sbb,
    );
    dark_tower_main::makeWoodPillar(
        this,
        world,
        this.m_deco.get(),
        17,
        y,
        5,
        this.stairMeta(0 + rotation),
        rotation,
        sbb,
    );
    dark_tower_main::makeStonePillar(
        this,
        world,
        this.m_deco.get(),
        12,
        y,
        1,
        this.stairMeta(3 + rotation),
        rotation,
        sbb,
    );
    dark_tower_main::makeStonePillar(
        this,
        world,
        this.m_deco.get(),
        17,
        y,
        6,
        this.stairMeta(0 + rotation),
        rotation,
        sbb,
    );
    this.placeTurned(world, blk("crafting_table"), 0, 14, y + 2, 4, rotation, sbb);
    if y < (this.m_height.get() - 13)
    {
        this.placeTurned(world, blk("obsidian"), 0, 13, y + 1, 13, rotation, sbb);
        this.placeTurned(world, blk("obsidian"), 0, 15, y + 1, 13, rotation, sbb);
        this.placeTurned(world, blk("obsidian"), 0, 13, y + 1, 15, rotation, sbb);
        this.placeTurned(world, blk("obsidian"), 0, 15, y + 1, 15, rotation, sbb);
        this.placeTurned(world, blk("netherrack"), 0, 13, y + 1, 14, rotation, sbb);
        this.placeTurned(world, blk("netherrack"), 0, 14, y + 1, 13, rotation, sbb);
        this.placeTurned(world, blk("netherrack"), 0, 15, y + 1, 14, rotation, sbb);
        this.placeTurned(world, blk("netherrack"), 0, 14, y + 1, 15, rotation, sbb);
        this.placeTurned(world, blk("redstone_block"), 0, 14, y + 1, 14, rotation, sbb);
        this.placeTurned(world, blk("netherrack"), 0, 13, y + 2, 13, rotation, sbb);
        this.placeTurned(world, blk("netherrack"), 0, 15, y + 2, 13, rotation, sbb);
        this.placeTurned(world, blk("netherrack"), 0, 13, y + 2, 15, rotation, sbb);
        this.placeTurned(world, blk("netherrack"), 0, 15, y + 2, 15, rotation, sbb);
        this.placeTurned(world, blk("tf:towerDevice"), 12, 14, y + 2, 14, rotation, sbb);
        this.placeTurned(world, blk("obsidian"), 0, 13, y + 3, 13, rotation, sbb);
        this.placeTurned(world, blk("obsidian"), 0, 15, y + 3, 13, rotation, sbb);
        this.placeTurned(world, blk("obsidian"), 0, 13, y + 3, 15, rotation, sbb);
        this.placeTurned(world, blk("obsidian"), 0, 15, y + 3, 15, rotation, sbb);
        this.placeTurned(world, blk("netherrack"), 0, 13, y + 3, 14, rotation, sbb);
        this.placeTurned(world, blk("netherrack"), 0, 14, y + 3, 13, rotation, sbb);
        this.placeTurned(world, blk("netherrack"), 0, 15, y + 3, 14, rotation, sbb);
        this.placeTurned(world, blk("netherrack"), 0, 14, y + 3, 15, rotation, sbb);
        this.placeTurned(world, blk("redstone_block"), 0, 14, y + 3, 14, rotation, sbb);
    }
    this.placeTurned(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, 14, y + 1, 17, rotation, sbb);
    this.placeTurned(
        world,
        blk("lever"),
        dark_tower_wing::getLeverMeta(this, rotation, 4),
        13,
        y + 1,
        17,
        rotation,
        sbb,
    );
    this.placeTurned(world, blk("piston"), 5 - this.stairMeta(3 + rotation), 14, y + 2, 17, rotation, sbb);
    this.placeTurned(world, blk("redstone_block"), 0, 14, y + 2, 16, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, 17, y + 1, 14, rotation, sbb);
    this.placeTurned(
        world,
        blk("lever"),
        dark_tower_wing::getLeverMeta(this, rotation, 2),
        17,
        y + 1,
        13,
        rotation,
        sbb,
    );
    this.placeTurned(world, blk("piston"), 5 - this.stairMeta(2 + rotation), 17, y + 2, 14, rotation, sbb);
    this.placeTurned(world, blk("redstone_block"), 0, 16, y + 2, 14, rotation, sbb);
    this.placeTurned(world, blk("redstone_block"), 0, 14, y + 2, 11, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, 14, y + 1, 11, rotation, sbb);
    this.placeTurned(
        world,
        blk("lever"),
        dark_tower_wing::getLeverMeta(this, rotation, 4) + 8,
        13,
        y + 1,
        11,
        rotation,
        sbb,
    );
    this.placeTurned(world, blk("piston"), 5 - this.stairMeta(1 + rotation), 14, y + 2, 10, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, 14, y + 1, 9, rotation, sbb);
    this.placeTurned(
        world,
        blk("lever"),
        dark_tower_wing::getLeverMeta(this, rotation, 4),
        13,
        y + 1,
        9,
        rotation,
        sbb,
    );
    this.placeTurned(world, blk("sticky_piston"), 5 - this.stairMeta(1 + rotation), 14, y + 2, 9, rotation, sbb);
    this.placeTurned(world, blk("redstone_block"), 0, 11, y + 2, 14, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, 11, y + 1, 14, rotation, sbb);
    this.placeTurned(
        world,
        blk("lever"),
        dark_tower_wing::getLeverMeta(this, rotation, 2) + 8,
        11,
        y + 1,
        13,
        rotation,
        sbb,
    );
    this.placeTurned(world, blk("piston"), 5 - this.stairMeta(0 + rotation), 10, y + 2, 14, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, 9, y + 1, 14, rotation, sbb);
    this.placeTurned(
        world,
        blk("lever"),
        dark_tower_wing::getLeverMeta(this, rotation, 2),
        9,
        y + 1,
        13,
        rotation,
        sbb,
    );
    this.placeTurned(world, blk("sticky_piston"), 5 - this.stairMeta(0 + rotation), 9, y + 2, 14, rotation, sbb);
}

// ComponentTFDarkTowerMain.makeWoodPillar
pub fn makeWoodPillar(
    this: &Piece,
    world: &GenWorld,
    forgeDeco: Deco,
    x: i32,
    y: i32,
    z: i32,
    stairMeta: i32,
    rotation: i32,
    sbb: StructureBox,
)
{
    this.placeTurned(world, blk("tf:log"), 3, x, y + 2, z, rotation, sbb);
    this.placeTurned(world, blk("tf:log"), 3, x, y + 3, z, rotation, sbb);
    this.placeTurned(world, blk("tf:log"), 3, x, y + 4, z, rotation, sbb);
}

// ComponentTFDarkTowerMain.decorateAquarium
pub fn decorateAquarium(this: &Piece, world: &GenWorld, decoRNG: &Random, sbb: StructureBox, rotation: i32, y: i32)
{
    dark_tower_wing::makePillarFrame11(this, world, sbb, this.m_deco.get(), rotation, 12, y, 3, 4, 4, 13, false);
    this.fillTurned(world, sbb, 13, y + 4, 4, 14, y + 4, 14, blk("flowing_water"), 0, rotation);
    dark_tower_wing::makePillarFrame11(this, world, sbb, this.m_deco.get(), rotation, 6, y, 12, 4, 4, 4, false);
    this.fillTurned(
        world,
        sbb,
        6,
        y + 5,
        12,
        9,
        y + 5,
        15,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        rotation,
    );
    this.fillTurned(world, sbb, 7, y + 4, 13, 8, y + 5, 14, blk("flowing_water"), 0, rotation);
}

// ComponentTFDarkTowerMain.decorateForge
pub fn decorateForge(this: &Piece, world: &GenWorld, decoRNG: &Random, sbb: StructureBox, rotation: i32, y: i32)
{
    let forgeDeco: Deco = this.m_deco.get();
    this.fillTurned(world, sbb, 17, y + 1, 1, 17, y + 4, 6, forgeDeco.m_pillarID, forgeDeco.m_pillarMeta, rotation);
    this.fillTurned(world, sbb, 12, y + 1, 1, 17, y + 4, 1, forgeDeco.m_pillarID, forgeDeco.m_pillarMeta, rotation);
    this.fillTurned(world, sbb, 12, y + 1, 17, 17, y + 4, 17, forgeDeco.m_pillarID, forgeDeco.m_pillarMeta, rotation);
    this.fillTurned(world, sbb, 17, y + 1, 12, 17, y + 4, 17, forgeDeco.m_pillarID, forgeDeco.m_pillarMeta, rotation);
    this.fillTurned(world, sbb, 13, y + 1, 2, 16, y + 1, 5, forgeDeco.m_blockID, forgeDeco.m_blockMeta, rotation);
    this.fillTurned(
        world,
        sbb,
        12,
        y + 1,
        2,
        12,
        y + 1,
        6,
        forgeDeco.m_stairID,
        this.stairMeta(0 + rotation),
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        12,
        y + 1,
        6,
        16,
        y + 1,
        6,
        forgeDeco.m_stairID,
        this.stairMeta(3 + rotation),
        rotation,
    );
    this.fillTurned(world, sbb, 13, y + 1, 13, 16, y + 1, 16, forgeDeco.m_blockID, forgeDeco.m_blockMeta, rotation);
    this.fillTurned(
        world,
        sbb,
        12,
        y + 1,
        12,
        12,
        y + 1,
        16,
        forgeDeco.m_stairID,
        this.stairMeta(0 + rotation),
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        12,
        y + 1,
        12,
        16,
        y + 1,
        12,
        forgeDeco.m_stairID,
        this.stairMeta(1 + rotation),
        rotation,
    );
    dark_tower_main::makeFurnacePillar(
        this,
        world,
        forgeDeco,
        decoRNG,
        13,
        y,
        1,
        this.stairMeta(3 + rotation),
        rotation,
        sbb,
    );
    dark_tower_main::makeFurnacePillar(
        this,
        world,
        forgeDeco,
        decoRNG,
        15,
        y,
        1,
        this.stairMeta(3 + rotation),
        rotation,
        sbb,
    );
    dark_tower_main::makeFurnacePillar(
        this,
        world,
        forgeDeco,
        decoRNG,
        17,
        y,
        3,
        this.stairMeta(0 + rotation),
        rotation,
        sbb,
    );
    dark_tower_main::makeFurnacePillar(
        this,
        world,
        forgeDeco,
        decoRNG,
        17,
        y,
        5,
        this.stairMeta(0 + rotation),
        rotation,
        sbb,
    );
    dark_tower_main::makeFurnacePillar(
        this,
        world,
        forgeDeco,
        decoRNG,
        13,
        y,
        17,
        this.stairMeta(1 + rotation),
        rotation,
        sbb,
    );
    dark_tower_main::makeFurnacePillar(
        this,
        world,
        forgeDeco,
        decoRNG,
        15,
        y,
        17,
        this.stairMeta(1 + rotation),
        rotation,
        sbb,
    );
    dark_tower_main::makeFurnacePillar(
        this,
        world,
        forgeDeco,
        decoRNG,
        17,
        y,
        13,
        this.stairMeta(0 + rotation),
        rotation,
        sbb,
    );
    dark_tower_main::makeFurnacePillar(
        this,
        world,
        forgeDeco,
        decoRNG,
        17,
        y,
        15,
        this.stairMeta(0 + rotation),
        rotation,
        sbb,
    );
    dark_tower_main::makeStonePillar(this, world, forgeDeco, 12, y, 1, this.stairMeta(3 + rotation), rotation, sbb);
    dark_tower_main::makeStonePillar(this, world, forgeDeco, 17, y, 6, this.stairMeta(0 + rotation), rotation, sbb);
    dark_tower_main::makeStonePillar(this, world, forgeDeco, 12, y, 17, this.stairMeta(1 + rotation), rotation, sbb);
    dark_tower_main::makeStonePillar(this, world, forgeDeco, 17, y, 12, this.stairMeta(0 + rotation), rotation, sbb);
    dark_tower_main::makeStonePillar(this, world, forgeDeco, 17, y, 9, this.stairMeta(0 + rotation), rotation, sbb);
    dark_tower_main::makeStonePillar(this, world, forgeDeco, 9, y, 17, this.stairMeta(1 + rotation), rotation, sbb);
    this.placeTurned(world, blk("anvil"), decoRNG.nextIntBounded(16), 13, y + 2, 5, rotation, sbb);
    this.placeTurned(world, blk("anvil"), decoRNG.nextIntBounded(16), 13, y + 2, 13, rotation, sbb);
    dark_tower_main::makeFirePit(this, world, forgeDeco, 6, y + 1, 12, rotation, sbb);
}

// ComponentTFDarkTowerMain.makeFurnacePillar
pub fn makeFurnacePillar(
    this: &Piece,
    world: &GenWorld,
    forgeDeco: Deco,
    random: &Random,
    x: i32,
    y: i32,
    z: i32,
    stairMeta: i32,
    rotation: i32,
    sbb: StructureBox,
)
{
    this.placeTurned(world, forgeDeco.m_stairID, stairMeta + 4, x, y + 2, z, rotation, sbb);
    this.placeTurned(world, blk("furnace"), stairMeta + 4, x, y + 3, z, rotation, sbb);
    let amount: i32 = if random.nextBoolean()
    {
        random.nextIntBounded(5) + 4
    }
    else
    {
        0
    };
    if amount > 0
    {
        let dx: i32 = this.xTurned(x, z, rotation);
        let dy: i32 = this.yAt(y + 3);
        let dz: i32 = this.zTurned(x, z, rotation);
        if sbb.contains(dx, dy, dz) && (world.getBlock(dx, dy, dz) == blk("furnace"))
        {
            // the furnace's fuel slot gets charcoal, coal metadata 1
            let charcoal: b173_api::ItemStack =
                b173_api::ItemStack { damage: 1, ..b173_api::ItemStack::of("minecraft:coal", amount) };
            if let Err(error) = world.m_world.setTileSlot(b173_api::BlockPos::new(dx, dy, dz), 1, Some(charcoal))
            {
                b173_api::warn(&format!("dark tower forge fuel: {}", error.0));
            }
        }
    }
    this.placeTurned(world, forgeDeco.m_stairID, stairMeta, x, y + 4, z, rotation, sbb);
}

// ComponentTFDarkTowerMain.makeStonePillar
pub fn makeStonePillar(
    this: &Piece,
    world: &GenWorld,
    forgeDeco: Deco,
    x: i32,
    y: i32,
    z: i32,
    stairMeta: i32,
    rotation: i32,
    sbb: StructureBox,
)
{
    for py in 1..=4
    {
        this.placeTurned(world, forgeDeco.m_pillarID, forgeDeco.m_pillarMeta, x, y + py, z, rotation, sbb);
    }
    let mut sx: i32 = this.xTurned(x, z, rotation);
    let sy: i32 = this.yAt(y + 1);
    let mut sz: i32 = this.zTurned(x, z, rotation);
    match stairMeta
    {
        0 =>
        {
            sx -= 1;
        }
        1 =>
        {
            sx += 1;
        }
        2 =>
        {
            sz -= 1;
        }
        3 =>
        {
            sz += 1;
        }
        _ =>
        {}
    }
    if sbb.contains(sx, sy, sz)
    {
        world.setBlock(sx, sy + 0, sz, forgeDeco.m_stairID, stairMeta, 0);
        world.setBlock(sx, sy + 3, sz, forgeDeco.m_stairID, stairMeta + 4, 0);
    }
}

// ComponentTFDarkTowerMain.makeFirePit
pub fn makeFirePit(
    this: &Piece,
    world: &GenWorld,
    myDeco: Deco,
    x: i32,
    y: i32,
    z: i32,
    rotation: i32,
    sbb: StructureBox,
)
{
    this.placeTurned(world, myDeco.m_pillarID, myDeco.m_pillarMeta, x + 1, y, z + 1, rotation, sbb);
    this.placeTurned(world, myDeco.m_pillarID, myDeco.m_pillarMeta, x + 1, y, z - 1, rotation, sbb);
    this.placeTurned(world, myDeco.m_pillarID, myDeco.m_pillarMeta, x - 1, y, z + 1, rotation, sbb);
    this.placeTurned(world, myDeco.m_pillarID, myDeco.m_pillarMeta, x - 1, y, z - 1, rotation, sbb);
    this.placeTurned(world, myDeco.m_stairID, this.stairMeta(0 + rotation), x - 1, y, z + 0, rotation, sbb);
    this.placeTurned(world, myDeco.m_stairID, this.stairMeta(2 + rotation), x + 1, y, z + 0, rotation, sbb);
    this.placeTurned(world, myDeco.m_stairID, this.stairMeta(3 + rotation), x + 0, y, z + 1, rotation, sbb);
    this.placeTurned(world, myDeco.m_stairID, this.stairMeta(1 + rotation), x + 0, y, z - 1, rotation, sbb);
    this.placeTurned(world, blk("netherrack"), 0, x, y, z, rotation, sbb);
    this.placeTurned(world, blk("fire"), 0, x, y + 1, z, rotation, sbb);
}

// ComponentTFDarkTowerMain.decorateNetherwart
pub fn decorateNetherwart(
    this: &Piece,
    world: &GenWorld,
    decoRNG: &Random,
    sbb: StructureBox,
    rotation: i32,
    y: i32,
    isTop: bool,
)
{
    let netherDeco: Deco = this.m_deco.get();
    dark_tower_wing::makePillarFrame11(this, world, sbb, netherDeco, rotation, 12, y, 9, 4, 4, 7, true);
    this.fillTurned(world, sbb, 13, y + 1, 10, 14, y + 1, 14, blk("soul_sand"), 0, rotation);
    this.fillTurned(world, sbb, 13, y + 2, 10, 14, y + 2, 14, blk("nether_wart"), 0, rotation);
    this.fillTurned(world, sbb, 13, y + 4, 10, 14, y + 4, 14, blk("soul_sand"), 0, rotation);
    dark_tower_wing::makePillarFrame11(
        this,
        world,
        sbb,
        netherDeco,
        rotation,
        5,
        y,
        12,
        3,
        if isTop { 4 } else { 9 },
        3,
        true,
    );
    this.placeTurned(world, netherDeco.m_blockID, netherDeco.m_blockMeta, 6, y + 1, 13, rotation, sbb);
    this.placeTurned(
        world,
        netherDeco.m_blockID,
        netherDeco.m_blockMeta,
        6,
        y + (if isTop { 4 } else { 9 }),
        13,
        rotation,
        sbb,
    );
    this.placeSpawnerRotated(world, 6, y + 3, 13, rotation, mob("Blaze"), sbb);
    dark_tower_wing::destroyTower7(this, world, decoRNG, 12, y, 3, 2, sbb);
}

// ComponentTFDarkTowerMain.decorateBotanical
pub fn decorateBotanical(this: &Piece, world: &GenWorld, decoRNG: &Random, sbb: StructureBox, rotation: i32, y: i32)
{
    dark_tower_wing::makePillarFrame11(this, world, sbb, this.m_deco.get(), rotation, 12, y, 12, 4, 4, 4, true);
    this.fillTurned(
        world,
        sbb,
        13,
        y + 1,
        13,
        14,
        y + 1,
        14,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockMeta,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        13,
        y + 4,
        13,
        14,
        y + 4,
        14,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockMeta,
        rotation,
    );
    dark_tower_main::placeRandomPlant(this, world, decoRNG, 13, y + 2, 13, rotation, sbb);
    dark_tower_main::placeRandomPlant(this, world, decoRNG, 13, y + 2, 14, rotation, sbb);
    dark_tower_main::placeRandomPlant(this, world, decoRNG, 14, y + 2, 13, rotation, sbb);
    dark_tower_main::placeRandomPlant(this, world, decoRNG, 14, y + 2, 14, rotation, sbb);
    for py in 1..=4
    {
        this.placeTurned(
            world,
            this.m_deco.get().m_pillarID,
            this.m_deco.get().m_pillarMeta,
            12,
            y + py,
            4,
            rotation,
            sbb,
        );
        this.placeTurned(
            world,
            this.m_deco.get().m_pillarID,
            this.m_deco.get().m_pillarMeta,
            15,
            y + py,
            4,
            rotation,
            sbb,
        );
    }
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(2 + rotation) + 4, 13, y + 1, 4, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(0 + rotation) + 4, 14, y + 1, 4, rotation, sbb);
    this.placeTreasureRotated(world, 13, y + 2, 4, rotation, Table::Basement, sbb);
    this.placeTurned(world, blk("crafting_table"), 0, 14, y + 2, 4, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(2 + rotation) + 4, 12, y + 1, 7, rotation, sbb);
    this.placeTurned(world, blk("wooden_slab"), 9, 13, y + 1, 7, rotation, sbb);
    this.placeTurned(world, blk("wooden_slab"), 9, 14, y + 1, 7, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(0 + rotation) + 4, 15, y + 1, 7, rotation, sbb);
    this.placeTurned(
        world,
        this.m_deco.get().m_stairID,
        this.stairMeta(2 + rotation) + 4,
        12,
        y + 1,
        10,
        rotation,
        sbb,
    );
    this.placeTurned(world, blk("wooden_slab"), 9, 13, y + 1, 10, rotation, sbb);
    this.placeTurned(world, blk("wooden_slab"), 9, 14, y + 1, 10, rotation, sbb);
    this.placeTurned(
        world,
        this.m_deco.get().m_stairID,
        this.stairMeta(0 + rotation) + 4,
        15,
        y + 1,
        10,
        rotation,
        sbb,
    );
    for x in 12..=15
    {
        dark_tower_main::placeRandomPlant(this, world, decoRNG, x, y + 2, 7, rotation, sbb);
        dark_tower_main::placeRandomPlant(this, world, decoRNG, x, y + 2, 10, rotation, sbb);
    }
    dark_tower_main::placeTreePlanter(this, world, decoRNG.nextIntBounded(5), 6, y + 1, 12, rotation, sbb);
}

// ComponentTFDarkTowerMain.placeTreePlanter
pub fn placeTreePlanter(
    this: &Piece,
    world: &GenWorld,
    treeNum: i32,
    x: i32,
    y: i32,
    z: i32,
    rotation: i32,
    sbb: StructureBox,
)
{
    this.placeTurned(
        world,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta,
        x + 1,
        y,
        z + 1,
        rotation,
        sbb,
    );
    this.placeTurned(
        world,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta,
        x + 1,
        y,
        z - 1,
        rotation,
        sbb,
    );
    this.placeTurned(
        world,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta,
        x - 1,
        y,
        z + 1,
        rotation,
        sbb,
    );
    this.placeTurned(
        world,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta,
        x - 1,
        y,
        z - 1,
        rotation,
        sbb,
    );
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(0 + rotation), x - 1, y, z + 0, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(2 + rotation), x + 1, y, z + 0, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(3 + rotation), x + 0, y, z + 1, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(1 + rotation), x + 0, y, z - 1, rotation, sbb);
    this.placeTurned(world, blk("dirt"), 0, x, y, z, rotation, sbb);
    let dx: i32 = this.xTurned(x, z, rotation);
    let dy: i32 = this.yAt(y + 1);
    let dz: i32 = this.zTurned(x, z, rotation);
    if sbb.contains(dx, dy, dz)
    {
        let mut var10000: Gen;
        match treeNum
        {
            1 =>
            {
                var10000 = Gen::Trees { minimum: 3, meta: 3 };
            }
            2 =>
            {
                var10000 = Gen::Birch;
            }
            3 =>
            {
                var10000 = Gen::TwilightOak;
            }
            4 =>
            {
                var10000 = Gen::Rainboak;
            }
            _ =>
            {
                var10000 = Gen::Trees { minimum: 4, meta: 0 };
            }
        }
        let treeGen: Gen = var10000;
        let mut i: i32 = 0;
        while (i < 100) && (!treeGen.generate(world, world.rand(), dx, dy, dz))
        {
            i += 1;
        }
    }
}

// ComponentTFDarkTowerMain.placeRandomPlant
pub fn placeRandomPlant(
    this: &Piece,
    world: &GenWorld,
    decoRNG: &Random,
    x: i32,
    y: i32,
    z: i32,
    rotation: i32,
    sbb: StructureBox,
)
{
    let potMeta: i32 = decoRNG.nextIntBounded(12);
    this.placeTurned(world, blk("flower_pot"), potMeta, x, y, z, rotation, sbb);
}

// ComponentTFDarkTowerMain.makeBottomEntrance
pub fn makeBottomEntrance(this: &Piece, world: &GenWorld, decoRNG: &Random, sbb: StructureBox, rotation: i32, y: i32)
{
    dark_tower_main::makeFirePit(this, world, this.m_deco.get(), 13, y + 1, 3, rotation, sbb);
    dark_tower_main::makeFirePit(this, world, this.m_deco.get(), 3, y + 1, 13, rotation, sbb);
    dark_tower_main::makeFirePit(this, world, this.m_deco.get(), 13, y + 1, 13, rotation, sbb);
    dark_tower_wing::makePillarFrame11(this, world, sbb, this.m_deco.get(), rotation, 7, y, 7, 3, 4, 3, false);
}

// ComponentTFDarkTowerMain.addTimberMaze
pub fn addTimberMaze(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox, mut bottom: i32, top: i32)
{
    let spacing: i32 = 5;
    let mut floorside: i32 = 0;
    if bottom == 0
    {
        bottom += spacing;
    }
    {
        let mut y: i32 = bottom;
        while y < top
        {
            floorside =
            {
                floorside += 1;
                floorside
            } % 4;
            dark_tower_main::makeTimberBeams(
                this,
                world,
                random,
                sbb,
                floorside,
                y,
                (y == bottom) && (bottom != spacing),
                y >= (top - spacing),
                top,
            );
            y += spacing;
        }
    }
}

// ComponentTFDarkTowerMain.makeTimberBeams
pub fn makeTimberBeams(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    sbb: StructureBox,
    rotation: i32,
    y: i32,
    isBottom: bool,
    isTop: bool,
    top: i32,
)
{
    let beamID: Blk = blk("tf:log");
    let beamMetaBase: i32 = 3;
    let beamMetaNS: i32 = if ((this.m_mode.get() + rotation) % 2) == 0
    {
        4
    }
    else
    {
        8
    };
    let beamMetaEW: i32 = if beamMetaNS == 4 { 8 } else { 4 };
    let beamMetaUD: i32 = 0;
    for z in 1..(this.m_size.get() - 1)
    {
        this.placeTurned(world, beamID, beamMetaBase + beamMetaEW, 4, y, z, rotation, sbb);
        this.placeTurned(world, beamID, beamMetaBase + beamMetaEW, 9, y, z, rotation, sbb);
        this.placeTurned(world, beamID, beamMetaBase + beamMetaEW, 14, y, z, rotation, sbb);
    }
    let mut z: i32 = dark_tower_wing::pickBetweenExcluding(this, 3, this.m_size.get() - 3, random, 4, 9, 14);
    for x in 5..9
    {
        this.placeTurned(world, beamID, beamMetaBase + beamMetaNS, x, y, z, rotation, sbb);
    }
    z = dark_tower_wing::pickBetweenExcluding(this, 3, this.m_size.get() - 3, random, 4, 9, 14);
    for x in 10..14
    {
        this.placeTurned(world, beamID, beamMetaBase + beamMetaNS, x, y, z, rotation, sbb);
    }
    let x1: i32 = 4;
    let z1: i32 = dark_tower_wing::pickFrom(this, random, 4, 9, 14);
    let x2: i32 = 9;
    let z2: i32 = dark_tower_wing::pickFrom(this, random, 4, 9, 14);
    let x3: i32 = 14;
    let z3: i32 = dark_tower_wing::pickFrom(this, random, 4, 9, 14);
    for by in 1..5
    {
        if (!isBottom) || dark_tower_wing::checkPost(this, world, x1, y - 5, z1, rotation, sbb)
        {
            this.placeTurned(world, beamID, beamMetaBase + beamMetaUD, x1, y - by, z1, rotation, sbb);
            this.placeTurned(
                world,
                blk("ladder"),
                this.ladderMetaTurned(2, rotation),
                x1 + 1,
                y - by,
                z1,
                rotation,
                sbb,
            );
        }
        if (!isBottom) || dark_tower_wing::checkPost(this, world, x2, y - 5, z2, rotation, sbb)
        {
            this.placeTurned(world, beamID, beamMetaBase + beamMetaUD, x2, y - by, z2, rotation, sbb);
        }
        if (!isBottom) || dark_tower_wing::checkPost(this, world, x3, y - 5, z3, rotation, sbb)
        {
            this.placeTurned(world, beamID, beamMetaBase + beamMetaUD, x3, y - by, z3, rotation, sbb);
            this.placeTurned(
                world,
                blk("ladder"),
                this.ladderMetaTurned(4, rotation),
                x3 - 1,
                y - by,
                z3,
                rotation,
                sbb,
            );
        }
    }
    if isTop
    {
        let topFloorRotation: i32 = ((this.m_box.get().m_min[1] + top) + 1) % 4;
        let ladderX: i32 = 4;
        let ladderZ: i32 = 10;
        let ladderMeta: i32 = 3;
        for by in 1..5
        {
            this.placeTurned(world, beamID, beamMetaBase + beamMetaUD, ladderX, y + by, 9, topFloorRotation, sbb);
            this.placeTurned(
                world,
                blk("ladder"),
                this.ladderMetaTurned(ladderMeta, topFloorRotation),
                ladderX,
                y + by,
                ladderZ,
                topFloorRotation,
                sbb,
            );
        }
        this.placeTurned(world, blk("air"), 0, ladderX, y + 6, 9, topFloorRotation, sbb);
        this.placeTurned(
            world,
            this.m_deco.get().m_fenceID,
            this.m_deco.get().m_fenceMeta,
            ladderX + 1,
            y + 5,
            ladderZ,
            topFloorRotation,
            sbb,
        );
        this.placeTurned(
            world,
            this.m_deco.get().m_fenceID,
            this.m_deco.get().m_fenceMeta,
            ladderX - 1,
            y + 5,
            ladderZ,
            topFloorRotation,
            sbb,
        );
        this.placeTurned(
            world,
            this.m_deco.get().m_fenceID,
            this.m_deco.get().m_fenceMeta,
            ladderX + 1,
            y + 6,
            ladderZ,
            topFloorRotation,
            sbb,
        );
        this.placeTurned(
            world,
            this.m_deco.get().m_fenceID,
            this.m_deco.get().m_fenceMeta,
            ladderX - 1,
            y + 6,
            ladderZ,
            topFloorRotation,
            sbb,
        );
    }
    if (!isBottom) && (!isTop)
    {
        let sx: i32 = dark_tower_wing::pickFrom(this, random, 6, 7, 11);
        let sz: i32 = dark_tower_wing::pickFrom(this, random, 6, 11, 12);
        dark_tower_main::makeMiniGhastSpawner(this, world, random, y, sx, sz, sbb);
    }
    let lx: i32 = dark_tower_wing::pickFrom(this, random, 2, 12, 16);
    let lz: i32 = 2 + random.nextIntBounded(15);
    this.placeTurned(world, blk("redstone_lamp"), 0, lx, y + 2, lz, rotation, sbb);
    this.placeTurned(world, blk("lever"), if random.nextBoolean() { 0 } else { 7 }, lx, y + 1, lz, rotation, sbb);
}

// ComponentTFDarkTowerMain.makeMiniGhastSpawner
pub fn makeMiniGhastSpawner(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    y: i32,
    sx: i32,
    sz: i32,
    sbb: StructureBox,
)
{
    // the source widens the spawn range to 16 with 2 nearby and 1 per spawn, the beta spawner keeps its defaults
    this.placeSpawner(world, random, sx, y + 2, sz, mob("Mini Ghast"), sbb);
}

// ComponentTFDarkTowerMain.addBuilderPlatforms
pub fn addBuilderPlatforms(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    sbb: StructureBox,
    mut bottom: i32,
    top: i32,
)
{
    let spacing: i32 = 5;
    let mut floorside: i32 = 0;
    if bottom == 0
    {
        bottom += spacing;
    }
    {
        let mut y: i32 = bottom;
        while y < (top - spacing)
        {
            dark_tower_main::makeBuilderPlatforms(
                this,
                world,
                random,
                sbb,
                floorside,
                y,
                (y == bottom) && (bottom != spacing),
                y >= (top - spacing),
            );
            floorside += 1 + random.nextIntBounded(3);
            floorside %= 4;
            y += spacing;
        }
    }
    dark_tower_main::makeBuilderPlatform(this, world, random, 1, bottom, 5, true, sbb);
    dark_tower_main::makeBuilderPlatform(this, world, random, 3, bottom, 5, true, sbb);
    for y in bottom - 4..bottom
    {
        this.placeTurned(world, blk("ladder"), this.ladderMetaTurned(2, 1), 1, y, 5, 1, sbb);
        this.placeTurned(world, blk("ladder"), this.ladderMetaTurned(2, 3), 1, y, 5, 3, sbb);
    }
    dark_tower_main::addTopBuilderPlatform(this, world, random, bottom, top, spacing, sbb);
}

// ComponentTFDarkTowerMain.makeBuilderPlatforms
pub fn makeBuilderPlatforms(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    sbb: StructureBox,
    rotation: i32,
    y: i32,
    bottom: bool,
    top: bool,
)
{
    let z: i32 = ((this.m_size.get() / 2) + random.nextIntBounded(5)) - random.nextIntBounded(5);
    dark_tower_main::makeBuilderPlatform(this, world, random, rotation, y, z, false, sbb);
    this.placeTurned(world, blk("ladder"), this.ladderMetaTurned(2, rotation), 1, y + 1, z, rotation, sbb);
    this.placeTurned(world, blk("ladder"), this.ladderMetaTurned(2, rotation), 1, y + 2, z, rotation, sbb);
    this.placeTurned(world, blk("ladder"), this.ladderMetaTurned(2, rotation), 1, y + 3, z, rotation, sbb);
    this.placeTurned(world, blk("ladder"), this.ladderMetaTurned(2, rotation), 1, y + 4, z, rotation, sbb);
    dark_tower_main::makeBuilderPlatform(this, world, random, rotation, y + 5, z, true, sbb);
    if (y % 2) == 1
    {
        let sx: i32 = dark_tower_wing::pickFrom(this, random, 5, 9, 13);
        let sz: i32 = if sx == 9
        {
            if random.nextBoolean() { 5 } else { 13 }
        }
        else
        {
            9
        };
        this.placeTurned(world, blk("tf:towerDevice"), 9, sx, y + 2, sz, rotation, sbb);
    }
    else
    {
        let sx: i32 = if random.nextBoolean() { 5 } else { 13 };
        let sz: i32 = if random.nextBoolean() { 5 } else { 13 };
        dark_tower_main::makeLampCluster(this, world, random, sx, y, sz, rotation, sbb);
    }
}

// ComponentTFDarkTowerMain.addTopBuilderPlatform
pub fn addTopBuilderPlatform(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    bottom: i32,
    top: i32,
    spacing: i32,
    sbb: StructureBox,
)
{
    let rotation: i32 = ((this.m_box.get().m_min[1] + top) + 1) % 4;
    this.fillTurned(
        world,
        sbb,
        5,
        top - spacing,
        9,
        7,
        top - spacing,
        11,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        6,
        top - spacing,
        9,
        6,
        top,
        9,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        6,
        (top - spacing) + 1,
        10,
        6,
        top - 1,
        10,
        blk("ladder"),
        this.ladderMetaTurned(3, rotation),
        rotation,
    );
    this.placeTurned(world, blk("air"), 0, 6, top + 1, 9, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_fenceID, this.m_deco.get().m_fenceMeta, 5, top + 0, 10, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_fenceID, this.m_deco.get().m_fenceMeta, 7, top + 0, 10, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_fenceID, this.m_deco.get().m_fenceMeta, 5, top + 1, 10, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_fenceID, this.m_deco.get().m_fenceMeta, 7, top + 1, 10, rotation, sbb);
    this.placeTurned(world, blk("tf:towerDevice"), 6, 7, top - spacing, 10, rotation, sbb);
    this.placeTurned(
        world,
        blk("lever"),
        if random.nextBoolean() { 5 } else { 6 },
        7,
        (top - spacing) + 1,
        11,
        rotation,
        sbb,
    );
}

// ComponentTFDarkTowerMain.makeBuilderPlatform
pub fn makeBuilderPlatform(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    rotation: i32,
    y: i32,
    z: i32,
    hole: bool,
    sbb: StructureBox,
)
{
    this.placeTurned(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, 1, y, z - 1, rotation, sbb);
    if !hole
    {
        this.placeTurned(
            world,
            this.m_deco.get().m_accentID,
            this.m_deco.get().m_accentMeta,
            1,
            y,
            z - 0,
            rotation,
            sbb,
        );
    }
    this.placeTurned(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, 1, y, z + 1, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, 2, y, z - 1, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, 2, y, z - 0, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, 2, y, z + 1, rotation, sbb);
    this.placeTurned(world, blk("tf:towerDevice"), 6, 2, y, if hole { z + 1 } else { z - 1 }, rotation, sbb);
    this.placeTurned(world, blk("lever"), if random.nextBoolean() { 5 } else { 6 }, 2, y + 1, z + 0, rotation, sbb);
}

// ComponentTFDarkTowerMain.makeLampCluster
pub fn makeLampCluster(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    sx: i32,
    y: i32,
    sz: i32,
    rotation: i32,
    sbb: StructureBox,
)
{
    let radius: i32 = 4;
    for i in 0..5
    {
        let mut lx: i32 = sx;
        let mut ly: i32 = y;
        let mut lz: i32 = sz;
        for r#move in 0..10
        {
            this.placeTurned(world, blk("redstone_lamp"), 0, lx, ly, lz, rotation, sbb);
            let mut direction: i32 = random.nextIntBounded(8);
            if direction > 5
            {
                direction -= 2;
            }
            lx += FACING[direction as usize][0];
            ly += FACING[direction as usize][1];
            lz += FACING[direction as usize][2];
            if (((((lx > (sx + radius)) || (lx < (sx - radius))) || (ly > (y + radius))) || (ly < (y - radius)))
                || (lz > (sz + radius)))
                || (lz < (sz - radius))
            {
                break;
            }
        }
    }
    for i in 0..5
    {
        let mut lx: i32 = sx;
        let mut ly: i32 = y;
        let mut lz: i32 = sz;
        let mut directions: Vec<i32> = vec![0; (10) as usize];
        for r#move in 0..10
        {
            directions[(r#move) as usize] = random.nextIntBounded(8);
            if directions[(r#move) as usize] > 5
            {
                directions[(r#move) as usize] -= 2;
            }
        }
        for r#move in 0..10
        {
            let mut direction: i32 = directions[(r#move) as usize];
            lx += FACING[direction as usize][0];
            ly += FACING[direction as usize][1];
            lz += FACING[direction as usize][2];
            if (((((lx > (sx + radius)) || (lx < (sx - radius))) || (ly > (y + radius))) || (ly < (y - radius)))
                || (lz > (sz + radius)))
                || (lz < (sz - radius))
            {
                break;
            }
            if this.getTurned(world, lx, ly, lz, rotation, sbb) != blk("redstone_lamp")
            {
                this.placeTurned(
                    world,
                    blk("lever"),
                    dark_tower_wing::getLeverMeta(this, rotation, direction),
                    lx,
                    ly,
                    lz,
                    rotation,
                    sbb,
                );
                break;
            }
        }
    }
}
