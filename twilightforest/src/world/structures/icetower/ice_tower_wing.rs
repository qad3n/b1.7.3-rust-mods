// draft from ComponentTFIceTowerWing.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub const SIZE: i32 = 11;
pub const RANGE: i32 = 17;

pub fn new(i: i32, x: i32, y: i32, z: i32, pSize: i32, pHeight: i32, direction: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::IceTowerWing);
    init(&this, i, x, y, z, pSize, pHeight, direction);
    Rc::new(this)
}

// ComponentTFIceTowerWing constructor
pub fn init(this: &Piece, i: i32, x: i32, y: i32, z: i32, pSize: i32, pHeight: i32, direction: i32)
{
    tower_wing::init7(this, i, x, y, z, pSize, pHeight, direction);
    this.m_hasBase.set(false);
    this.m_treasureFloor.set(-1);
}

// ComponentTFIceTowerWing.func_74861_a
pub fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    this.m_deco.set((parent).m_deco.get());
    this.addOpening(0, 1, this.m_size.get() / 2, 2);
    this.m_hasBase.set(this.shouldHaveBase(random));
    if this.depth() < 5
    {
        let dirOffset: i32 = random.nextIntBounded(4);
        for i in 0..4
        {
            let dir: i32 = (dirOffset + i) % 4;
            let dest: Vec<i32> = this.getValidOpening(random, dir);
            if ((this.depth() == 4) && (matches!(parent.m_kind, Kind::IceTowerMain))) && (!(parent).m_hasBossWing.get())
            {
                let hasBoss: bool = ice_tower_wing::makeBossTowerWing(
                    this,
                    list,
                    random,
                    this.depth() + 1,
                    dest[0],
                    dest[1],
                    dest[2],
                    15,
                    41,
                    dir,
                );
                parent.m_hasBossWing.set(hasBoss);
            }
            else
            {
                let childHeight: i32 = (((random.nextIntBounded(2) + random.nextIntBounded(2)) + 2) * 10) + 1;
                this.makeTowerWing(list, random, this.depth() + 1, dest[0], dest[1], dest[2], 11, childHeight, dir);
            }
        }
    }
    let floors: i32 = this.m_height.get() / 10;
    if (this.m_treasureFloor.get() == (-1)) && (floors > 1)
    {
        this.m_treasureFloor.set(random.nextIntBounded(floors - 1));
    }
    this.makeARoof(&parent, list, random);
    if !this.m_hasBase.get()
    {
        this.makeABeard(&parent, list, random);
    }
}

// ComponentTFIceTowerWing.shouldHaveBase
pub fn shouldHaveBase(this: &Piece, random: &Random) -> bool { return (this.depth() == 0) || random.nextBoolean(); }

// ComponentTFIceTowerWing.isOutOfRange
pub fn isOutOfRange(this: &Piece, parent: &Piece, nx: i32, ny: i32, nz: i32, range: i32) -> bool
{
    return ((nx - parent.bounds().centerX()).abs() > range) || ((nz - parent.bounds().centerZ()).abs() > range);
}

// ComponentTFIceTowerWing.makeTowerWing
pub fn makeTowerWing(
    this: &Piece,
    list: &mut Pieces,
    random: &Random,
    index: i32,
    x: i32,
    y: i32,
    z: i32,
    wingSize: i32,
    wingHeight: i32,
    rotation: i32,
) -> bool
{
    let direction: i32 = (this.mode() + rotation) % 4;
    let dx: Vec<i32> = this.offsetTower(x, y, z, wingSize, direction);
    if this.isOutOfRange(&Rc::clone(&list[(0) as usize]), dx[0], dx[1], dx[2], 17)
    {
        return false;
    }
    let wing: Rc<Piece> = ice_tower_wing::new(index, dx[0], dx[1], dx[2], wingSize, wingHeight, direction);
    let intersect: Option<Rc<Piece>> = intersecting(list, wing.bounds());
    if (intersect.is_some()) && (!same(&intersect, &this))
    {
        return false;
    }
    list.push(Rc::clone(&wing));
    wing.build(&Rc::clone(&list[(0) as usize]), list, random);
    this.addOpening(x, y, z, rotation);
    return true;
}

// ComponentTFIceTowerWing.makeBossTowerWing
pub fn makeBossTowerWing(
    this: &Piece,
    list: &mut Pieces,
    random: &Random,
    index: i32,
    x: i32,
    y: i32,
    z: i32,
    wingSize: i32,
    wingHeight: i32,
    rotation: i32,
) -> bool
{
    let direction: i32 = (this.mode() + rotation) % 4;
    let dx: Vec<i32> = this.offsetTower(x, y, z, wingSize, direction);
    let wing: Rc<Piece> = ice_tower_boss_wing::new(index, dx[0], dx[1], dx[2], wingSize, wingHeight, direction);
    let intersect: Option<Rc<Piece>> = intersecting(list, wing.bounds());
    if (intersect.is_some()) && (!same(&intersect, &this))
    {
        return false;
    }
    list.push(Rc::clone(&wing));
    wing.build(&Rc::clone(&list[(0) as usize]), list, random);
    this.addOpening(x, y, z, rotation);
    return true;
}

// ComponentTFIceTowerWing.getYByStairs
pub fn getYByStairs(this: &Piece, rx: i32, random: &Random, direction: i32) -> i32
{
    let floors: i32 = this.m_height.get() / 10;
    return 11 + (random.nextIntBounded(floors - 1) * 10);
}

// ComponentTFIceTowerWing.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    let decoRNG: Random = Random::withSeed(
        (world.seed() + ((this.m_box.get().m_min[0] * 321534781) as i64))
            ^ ((this.m_box.get().m_min[2] * 756839) as i64),
    );
    this.fillSelected(
        world,
        sbb,
        0,
        0,
        0,
        this.m_size.get() - 1,
        this.m_height.get() - 1,
        this.m_size.get() - 1,
        false,
        random,
        this.m_deco.get().m_randomBlocks,
    );
    this.fillAir(world, sbb, 1, 1, 1, this.m_size.get() - 2, this.m_height.get() - 2, this.m_size.get() - 2);
    if this.m_hasBase.get()
    {
        for x in 0..this.m_size.get()
        {
            for z in 0..this.m_size.get()
            {
                this.fillDown(world, this.m_deco.get().m_blockID, this.m_deco.get().m_blockMeta, x, -1, z, sbb);
            }
        }
    }
    this.darken(world);
    this.makeFloorsForTower(world, &decoRNG, sbb);
    this.makeOpenings(world, sbb);
    return true;
}

// ComponentTFIceTowerWing.nullifySkyLightForBoundingBox
pub fn nullifySkyLightForBoundingBox(this: &Piece, world: &GenWorld)
{
    this.darkenWorld(
        world,
        this.m_box.get().m_min[0] + 1,
        this.m_box.get().m_min[1] + 1,
        this.m_box.get().m_min[2] + 1,
        this.m_box.get().m_max[0] - 1,
        this.m_box.get().m_max[1] - 1,
        this.m_box.get().m_max[2] - 1,
    );
}

// ComponentTFIceTowerWing.makeFloorsForTower
pub fn makeFloorsForTower(this: &Piece, world: &GenWorld, decoRNG: &Random, sbb: StructureBox)
{
    let floors: i32 = this.m_height.get() / 10;
    let mut ladderDir: i32 = 3;
    let mut downLadderDir: i32 = -1;
    let floorHeight: i32 = 10;
    for i in 0..(floors - 1)
    {
        this.placeFloor5(world, decoRNG, sbb, floorHeight, i);
        downLadderDir =
        {
            let old = ladderDir;
            ladderDir += 1;
            old
        };
        ladderDir %= 4;
        this.decorateFloor8(
            world,
            decoRNG,
            i,
            i * floorHeight,
            (i * floorHeight) + floorHeight,
            ladderDir,
            downLadderDir,
            sbb,
        );
    }
    let topFloor: i32 = floors - 1;
    this.decorateTopFloor(
        world,
        decoRNG,
        topFloor,
        topFloor * floorHeight,
        (topFloor * floorHeight) + floorHeight,
        ladderDir,
        downLadderDir,
        sbb,
    );
}

// ComponentTFIceTowerWing.placeFloor
pub fn placeFloor5(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox, floorHeight: i32, floor: i32)
{
    for x in 1..(this.m_size.get() - 1)
    {
        for z in 1..(this.m_size.get() - 1)
        {
            this.placeBlock(
                world,
                this.m_deco.get().m_floorID,
                this.m_deco.get().m_floorMeta,
                x,
                (floor * floorHeight) + floorHeight,
                z,
                sbb,
            );
        }
    }
}

// ComponentTFIceTowerWing.makeDoorOpening
pub fn makeDoorOpening(this: &Piece, world: &GenWorld, dx: i32, dy: i32, dz: i32, sbb: StructureBox)
{
    tower_wing::makeDoorOpening(this, world, dx, dy, dz, sbb);
    if this.getBlock(world, dx, dy + 2, dz, sbb) != blk("air")
    {
        this.placeBlock(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, dx, dy + 2, dz, sbb);
    }
}

// ComponentTFIceTowerWing.decorateFloor
pub fn decorateFloor8(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    floor: i32,
    bottom: i32,
    top: i32,
    ladderUpDir: i32,
    ladderDownDir: i32,
    sbb: StructureBox,
)
{
    let hasTreasure: bool = this.m_treasureFloor.get() == floor;
    'case1:
    {
        match random.nextIntBounded(8)
        {
            // todo falls through to the next case
            0 =>
            {
                if ice_tower_wing::isNoDoorAreaRotated(this, 9, bottom + 5, 1, 10, top + 1, 7, ladderUpDir)
                {
                    ice_tower_wing::decorateWraparoundWallSteps(
                        this,
                        world,
                        random,
                        bottom,
                        top,
                        ladderUpDir,
                        ladderDownDir,
                        hasTreasure,
                        sbb,
                    );
                    break 'case1;
                }
            }
            // todo falls through to the next case
            1 =>
            {
                if ice_tower_wing::isNoDoorAreaRotated(this, 7, bottom, 0, 10, top + 1, 10, ladderUpDir)
                {
                    ice_tower_wing::decorateFarWallSteps(
                        this,
                        world,
                        random,
                        bottom,
                        top,
                        ladderUpDir,
                        ladderDownDir,
                        hasTreasure,
                        sbb,
                    );
                    break 'case1;
                }
            }
            // todo falls through to the next case
            2 =>
            {
                if ice_tower_wing::isNoDoorAreaRotated(this, 9, bottom + 5, 1, 10, top + 1, 7, ladderUpDir)
                {
                    ice_tower_wing::decorateWraparoundWallStepsPillars(
                        this,
                        world,
                        random,
                        bottom,
                        top,
                        ladderUpDir,
                        ladderDownDir,
                        hasTreasure,
                        sbb,
                    );
                    break 'case1;
                }
            }
            3 =>
            {
                ice_tower_wing::decoratePlatform(
                    this,
                    world,
                    random,
                    bottom,
                    top,
                    ladderUpDir,
                    ladderDownDir,
                    hasTreasure,
                    sbb,
                );
            }
            4 =>
            {
                ice_tower_wing::decoratePillarParkour(
                    this,
                    world,
                    random,
                    bottom,
                    top,
                    ladderUpDir,
                    ladderDownDir,
                    hasTreasure,
                    sbb,
                );
            }
            5 =>
            {
                ice_tower_wing::decoratePillarPlatforms(
                    this,
                    world,
                    random,
                    bottom,
                    top,
                    ladderUpDir,
                    ladderDownDir,
                    hasTreasure,
                    sbb,
                );
            }
            6 =>
            {
                ice_tower_wing::decoratePillarPlatformsOutside(
                    this,
                    world,
                    random,
                    bottom,
                    top,
                    ladderUpDir,
                    ladderDownDir,
                    hasTreasure,
                    sbb,
                );
            }
            7 =>
            {
                ice_tower_wing::decorateQuadPillarStairs(
                    this,
                    world,
                    random,
                    bottom,
                    top,
                    ladderUpDir,
                    ladderDownDir,
                    hasTreasure,
                    sbb,
                );
            }
            _ =>
            {}
        }
    }
}

// ComponentTFIceTowerWing.isNoDoorAreaRotated
pub fn isNoDoorAreaRotated(
    this: &Piece,
    minX: i32,
    minY: i32,
    minZ: i32,
    maxX: i32,
    maxY: i32,
    maxZ: i32,
    rotation: i32,
) -> bool
{
    let mut isClear: bool = true;
    let mut var10000: StructureBox;
    match rotation
    {
        1 =>
        {
            var10000 = StructureBox::new(
                [(this.m_size.get() - 1) - maxZ, minY, minX],
                [(this.m_size.get() - 1) - minZ, maxY, maxX],
            );
        }
        2 =>
        {
            var10000 = StructureBox::new(
                [(this.m_size.get() - 1) - maxX, minY, (this.m_size.get() - 1) - maxZ],
                [(this.m_size.get() - 1) - minX, maxY, (this.m_size.get() - 1) - minZ],
            );
        }
        3 =>
        {
            var10000 = StructureBox::new(
                [minZ, minY, (this.m_size.get() - 1) - maxX],
                [maxZ, maxY, (this.m_size.get() - 1) - minX],
            );
        }
        _ =>
        {
            var10000 = StructureBox::new([minX, minY, minZ], [maxX, maxY, maxZ]);
        }
    }
    let exclusionBox: StructureBox = var10000;
    for door in this.m_openings.borrow().clone()
    {
        if exclusionBox.contains(door[0], door[1], door[2])
        {
            isClear = false;
        }
    }
    return isClear;
}

// ComponentTFIceTowerWing.decorateTopFloor
pub fn decorateTopFloor(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    floor: i32,
    bottom: i32,
    top: i32,
    ladderUpDir: i32,
    ladderDownDir: i32,
    sbb: StructureBox,
)
{
    if random.nextBoolean()
    {
        ice_tower_wing::decoratePillarsCorners(this, world, random, bottom, top, ladderDownDir, sbb);
    }
    else
    {
        ice_tower_wing::decoratePillarsGrid(this, world, random, bottom, top, ladderDownDir, sbb);
    }
    if this.isDeadEnd()
    {
        ice_tower_wing::decorateTopFloorTreasure(this, world, random, bottom, top, ladderDownDir, sbb);
    }
}

// ComponentTFIceTowerWing.decorateTopFloorTreasure
pub fn decorateTopFloorTreasure(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    bottom: i32,
    top: i32,
    rotation: i32,
    sbb: StructureBox,
)
{
    this.fillTurned(
        world,
        sbb,
        5,
        bottom + 1,
        5,
        5,
        bottom + 4,
        5,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta,
        rotation,
    );
    this.placeTreasure(world, random, 5, bottom + 5, 5, Table::AuroraRoom, sbb);
}

// ComponentTFIceTowerWing.decoratePillars
pub fn decoratePillars(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    bottom: i32,
    top: i32,
    rotation: i32,
    sbb: StructureBox,
)
{
    this.fillTurned(
        world,
        sbb,
        3,
        bottom + 1,
        3,
        3,
        top - 1,
        3,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        7,
        bottom + 1,
        3,
        7,
        top - 1,
        3,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        3,
        bottom + 1,
        7,
        3,
        top - 1,
        7,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        7,
        bottom + 1,
        7,
        7,
        top - 1,
        7,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta,
        rotation,
    );
}

// ComponentTFIceTowerWing.decoratePillarsGrid
pub fn decoratePillarsGrid(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    bottom: i32,
    top: i32,
    rotation: i32,
    sbb: StructureBox,
)
{
    let beamMetaNS: i32 = if ((this.m_mode.get() + rotation) % 2) == 0
    {
        4
    }
    else
    {
        8
    };
    let beamMetaEW: i32 = if beamMetaNS == 4 { 8 } else { 4 };
    this.fillTurned(
        world,
        sbb,
        3,
        bottom + 5,
        1,
        3,
        bottom + 5,
        9,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta + beamMetaEW,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        7,
        bottom + 5,
        1,
        7,
        bottom + 5,
        9,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta + beamMetaEW,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        1,
        bottom + 5,
        3,
        9,
        bottom + 5,
        3,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta + beamMetaNS,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        1,
        bottom + 5,
        7,
        9,
        bottom + 5,
        7,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta + beamMetaNS,
        rotation,
    );
    ice_tower_wing::decoratePillars(this, world, random, bottom, top, rotation, sbb);
}

// ComponentTFIceTowerWing.decoratePillarsCorners
pub fn decoratePillarsCorners(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    bottom: i32,
    top: i32,
    rotation: i32,
    sbb: StructureBox,
)
{
    let beamMetaNS: i32 = if ((this.m_mode.get() + rotation) % 2) == 0
    {
        4
    }
    else
    {
        8
    };
    let beamMetaEW: i32 = if beamMetaNS == 4 { 8 } else { 4 };
    this.fillTurned(
        world,
        sbb,
        3,
        bottom + 5,
        1,
        3,
        bottom + 5,
        9,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta + beamMetaEW,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        7,
        bottom + 5,
        1,
        7,
        bottom + 5,
        9,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta + beamMetaEW,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        1,
        bottom + 5,
        3,
        9,
        bottom + 5,
        3,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta + beamMetaNS,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        1,
        bottom + 5,
        7,
        9,
        bottom + 5,
        7,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta + beamMetaNS,
        rotation,
    );
    this.fillAirTurned(world, sbb, 3, bottom + 5, 3, 7, bottom + 5, 7, rotation);
    ice_tower_wing::decoratePillars(this, world, random, bottom, top, rotation, sbb);
}

// ComponentTFIceTowerWing.decorateFarWallSteps
pub fn decorateFarWallSteps(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    bottom: i32,
    top: i32,
    ladderUpDir: i32,
    ladderDownDir: i32,
    hasTreasure: bool,
    sbb: StructureBox,
)
{
    for z in 1..10
    {
        let y: i32 = (bottom + 10) - (z / 2);
        this.placeTurned(
            world,
            if (z % 2) == 0
            {
                this.m_deco.get().m_pillarID
            }
            else
            {
                this.m_deco.get().m_platformID
            },
            if (z % 2) == 0
            {
                this.m_deco.get().m_pillarMeta
            }
            else
            {
                this.m_deco.get().m_platformMeta
            },
            9,
            y,
            z,
            ladderUpDir,
            sbb,
        );
        for by in bottom + 1..y
        {
            this.placeTurned(
                world,
                this.m_deco.get().m_pillarID,
                this.m_deco.get().m_pillarMeta,
                9,
                by,
                z,
                ladderUpDir,
                sbb,
            );
        }
    }
    for z in 1..10
    {
        let y: i32 = (bottom + 1) + (z / 2);
        this.placeTurned(
            world,
            if (z % 2) == 0
            {
                this.m_deco.get().m_platformID
            }
            else
            {
                this.m_deco.get().m_pillarID
            },
            if (z % 2) == 0
            {
                this.m_deco.get().m_platformMeta
            }
            else
            {
                this.m_deco.get().m_pillarMeta
            },
            8,
            y,
            z,
            ladderUpDir,
            sbb,
        );
        for by in bottom + 1..y
        {
            this.placeTurned(
                world,
                this.m_deco.get().m_pillarID,
                this.m_deco.get().m_pillarMeta,
                8,
                by,
                z,
                ladderUpDir,
                sbb,
            );
        }
    }
    this.placeTurned(
        world,
        this.m_deco.get().m_platformID,
        this.m_deco.get().m_platformMeta,
        7,
        bottom + 1,
        1,
        ladderUpDir,
        sbb,
    );
    for z in 2..7
    {
        this.placeTurned(world, blk("air"), 0, 9, top, z, ladderUpDir, sbb);
    }
    if hasTreasure
    {
        this.placeTreasureRotatedTrapped(world, 1, bottom + 8, 5, ladderUpDir, Table::AuroraCache, false, sbb);
        let beamMetaNS: i32 = if ((this.m_mode.get() + ladderUpDir) % 2) == 0
        {
            4
        }
        else
        {
            8
        };
        this.placeTurned(
            world,
            this.m_deco.get().m_pillarID,
            this.m_deco.get().m_pillarMeta + beamMetaNS,
            1,
            bottom + 7,
            5,
            ladderUpDir,
            sbb,
        );
    }
}

// ComponentTFIceTowerWing.decorateWraparoundWallSteps
pub fn decorateWraparoundWallSteps(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    bottom: i32,
    top: i32,
    ladderUpDir: i32,
    ladderDownDir: i32,
    hasTreasure: bool,
    sbb: StructureBox,
)
{
    for z in 1..10
    {
        let y: i32 = (bottom + 10) - (z / 2);
        this.placeTurned(
            world,
            this.m_deco.get().m_platformID,
            this.m_deco.get().m_platformMeta + (if (z % 2) == 0 { 8 } else { 0 }),
            9,
            y,
            z,
            ladderUpDir,
            sbb,
        );
    }
    for x in 1..9
    {
        let y: i32 = (bottom + 2) + ((x - 1) / 2);
        this.placeTurned(
            world,
            this.m_deco.get().m_platformID,
            this.m_deco.get().m_platformMeta + (if (x % 2) == 0 { 8 } else { 0 }),
            x,
            y,
            9,
            ladderUpDir,
            sbb,
        );
    }
    this.placeTurned(
        world,
        this.m_deco.get().m_platformID,
        this.m_deco.get().m_platformMeta + 8,
        1,
        bottom + 1,
        8,
        ladderUpDir,
        sbb,
    );
    this.placeTurned(
        world,
        this.m_deco.get().m_platformID,
        this.m_deco.get().m_platformMeta,
        1,
        bottom + 1,
        7,
        ladderUpDir,
        sbb,
    );
    for z in 2..7
    {
        this.placeTurned(world, blk("air"), 0, 9, top, z, ladderUpDir, sbb);
    }
    if hasTreasure
    {
        this.placeTreasureRotatedTrapped(world, 1, bottom + 5, 5, ladderUpDir, Table::AuroraCache, false, sbb);
        let beamMetaNS: i32 = if ((this.m_mode.get() + ladderUpDir) % 2) == 0
        {
            4
        }
        else
        {
            8
        };
        this.placeTurned(
            world,
            this.m_deco.get().m_pillarID,
            this.m_deco.get().m_pillarMeta + beamMetaNS,
            1,
            bottom + 4,
            5,
            ladderUpDir,
            sbb,
        );
    }
}

// ComponentTFIceTowerWing.decorateWraparoundWallStepsPillars
pub fn decorateWraparoundWallStepsPillars(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    bottom: i32,
    top: i32,
    ladderUpDir: i32,
    ladderDownDir: i32,
    hasTreasure: bool,
    sbb: StructureBox,
)
{
    let rotation: i32 = ladderDownDir;
    let beamMetaNS: i32 = if ((this.m_mode.get() + rotation) % 2) == 0
    {
        4
    }
    else
    {
        8
    };
    let beamMetaEW: i32 = if beamMetaNS == 4 { 8 } else { 4 };
    ice_tower_wing::decorateWraparoundWallSteps(
        this,
        world,
        random,
        bottom,
        top,
        ladderUpDir,
        ladderDownDir,
        false,
        sbb,
    );
    ice_tower_wing::decoratePillars(this, world, random, bottom, top, rotation, sbb);
    this.fillTurned(
        world,
        sbb,
        3,
        bottom + 5,
        1,
        3,
        bottom + 5,
        2,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta + beamMetaEW,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        7,
        bottom + 5,
        1,
        7,
        bottom + 5,
        2,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta + beamMetaEW,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        8,
        bottom + 5,
        3,
        9,
        bottom + 5,
        3,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta + beamMetaNS,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        8,
        bottom + 5,
        7,
        9,
        bottom + 5,
        7,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta + beamMetaNS,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        1,
        bottom + 2,
        3,
        2,
        bottom + 2,
        3,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta + beamMetaNS,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        1,
        bottom + 6,
        3,
        2,
        bottom + 6,
        3,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta + beamMetaNS,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        1,
        bottom + 4,
        7,
        2,
        bottom + 4,
        7,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta + beamMetaNS,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        1,
        bottom + 8,
        7,
        2,
        bottom + 8,
        7,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta + beamMetaNS,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        3,
        bottom + 6,
        8,
        3,
        bottom + 6,
        9,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta + beamMetaEW,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        7,
        bottom + 8,
        8,
        7,
        bottom + 8,
        9,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta + beamMetaEW,
        rotation,
    );
    if hasTreasure
    {
        this.placeTreasureRotatedTrapped(world, 7, bottom + 6, 1, ladderUpDir, Table::AuroraCache, false, sbb);
    }
}

// ComponentTFIceTowerWing.decoratePlatform
pub fn decoratePlatform(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    bottom: i32,
    top: i32,
    ladderUpDir: i32,
    ladderDownDir: i32,
    hasTreasure: bool,
    sbb: StructureBox,
)
{
    ice_tower_wing::decoratePillars(this, world, random, bottom, top, ladderDownDir, sbb);
    this.fillTurned(
        world,
        sbb,
        3,
        bottom + 5,
        3,
        7,
        bottom + 5,
        7,
        this.m_deco.get().m_floorID,
        this.m_deco.get().m_floorMeta,
        ladderDownDir,
    );
    for z in 6..10
    {
        let y: i32 = (bottom - 2) + (z / 2);
        this.placeTurned(
            world,
            this.m_deco.get().m_platformID,
            this.m_deco.get().m_platformMeta + (if (z % 2) == 1 { 8 } else { 0 }),
            1,
            y,
            z,
            ladderDownDir,
            sbb,
        );
    }
    for x in 2..6
    {
        let y: i32 = (bottom + 2) + (x / 2);
        this.placeTurned(
            world,
            this.m_deco.get().m_platformID,
            this.m_deco.get().m_platformMeta + (if (x % 2) == 1 { 8 } else { 0 }),
            x,
            y,
            9,
            ladderDownDir,
            sbb,
        );
    }
    this.placeTurned(
        world,
        this.m_deco.get().m_platformID,
        this.m_deco.get().m_platformMeta,
        5,
        bottom + 5,
        8,
        ladderDownDir,
        sbb,
    );
    this.placeTurned(
        world,
        this.m_deco.get().m_platformID,
        this.m_deco.get().m_platformMeta,
        5,
        bottom + 6,
        2,
        ladderUpDir,
        sbb,
    );
    for x in 5..10
    {
        let y: i32 = (bottom + 4) + (x / 2);
        this.placeTurned(
            world,
            this.m_deco.get().m_platformID,
            this.m_deco.get().m_platformMeta + (if (x % 2) == 1 { 8 } else { 0 }),
            x,
            y,
            1,
            ladderUpDir,
            sbb,
        );
        if x > 6
        {
            this.placeTurned(world, blk("air"), 0, x, top, 1, ladderUpDir, sbb);
        }
    }
    for z in 2..5
    {
        let y: i32 = (bottom + 8) + (z / 2);
        this.placeTurned(world, blk("air"), 0, 9, top, z, ladderUpDir, sbb);
        this.placeTurned(
            world,
            this.m_deco.get().m_platformID,
            this.m_deco.get().m_platformMeta + (if (z % 2) == 1 { 8 } else { 0 }),
            9,
            y,
            z,
            ladderUpDir,
            sbb,
        );
    }
    if hasTreasure
    {
        this.placeTreasureRotatedTrapped(world, 3, bottom + 6, 5, ladderDownDir, Table::AuroraCache, false, sbb);
    }
}

// ComponentTFIceTowerWing.decorateQuadPillarStairs
pub fn decorateQuadPillarStairs(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    bottom: i32,
    top: i32,
    ladderUpDir: i32,
    ladderDownDir: i32,
    hasTreasure: bool,
    sbb: StructureBox,
)
{
    ice_tower_wing::decoratePillars(this, world, random, bottom, top, ladderDownDir, sbb);
    for z in 6..9
    {
        let y: i32 = (bottom - 2) + (z / 2);
        this.placeTurned(
            world,
            this.m_deco.get().m_platformID,
            this.m_deco.get().m_platformMeta + (if (z % 2) == 1 { 8 } else { 0 }),
            2,
            y,
            z,
            ladderDownDir,
            sbb,
        );
    }
    for x in 3..9
    {
        let y: i32 = (bottom + 1) + (x / 2);
        this.placeTurned(
            world,
            this.m_deco.get().m_platformID,
            this.m_deco.get().m_platformMeta + (if (x % 2) == 1 { 8 } else { 0 }),
            x,
            y,
            8,
            ladderDownDir,
            sbb,
        );
    }
    for z in (1 + 1..=7).rev()
    {
        let y: i32 = (top - 2) - ((z - 1) / 2);
        if z < 4
        {
            this.placeTurned(world, blk("air"), 0, 8, top, z, ladderDownDir, sbb);
        }
        this.placeTurned(
            world,
            this.m_deco.get().m_platformID,
            this.m_deco.get().m_platformMeta + (if (z % 2) == 1 { 8 } else { 0 }),
            8,
            y,
            z,
            ladderDownDir,
            sbb,
        );
    }
    for x in (3 + 1..=7).rev()
    {
        let y: i32 = (top + 1) - ((x - 1) / 2);
        this.placeTurned(world, blk("air"), 0, x, top, 2, ladderDownDir, sbb);
        this.placeTurned(
            world,
            this.m_deco.get().m_platformID,
            this.m_deco.get().m_platformMeta + (if (x % 2) == 1 { 8 } else { 0 }),
            x,
            y,
            2,
            ladderDownDir,
            sbb,
        );
    }
    if hasTreasure
    {
        this.placeTreasureRotatedTrapped(world, 3, bottom + 7, 7, ladderUpDir, Table::AuroraCache, false, sbb);
    }
}

// ComponentTFIceTowerWing.decoratePillarPlatforms
pub fn decoratePillarPlatforms(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    bottom: i32,
    top: i32,
    ladderUpDir: i32,
    ladderDownDir: i32,
    hasTreasure: bool,
    sbb: StructureBox,
)
{
    for i in 1..10
    {
        let rotation: i32 = (ladderUpDir + i) % 4;
        this.fillTurned(
            world,
            sbb,
            2,
            bottom + i,
            2,
            4,
            bottom + i,
            4,
            this.m_deco.get().m_floorID,
            this.m_deco.get().m_floorMeta,
            rotation,
        );
    }
    this.fillAirTurned(world, sbb, 2, top, 2, 8, top, 4, ladderUpDir);
    this.fillAirTurned(world, sbb, 2, top, 2, 4, top, 6, ladderUpDir);
    this.placeTurned(world, this.m_deco.get().m_pillarID, this.m_deco.get().m_pillarMeta, 7, top, 3, ladderUpDir, sbb);
    this.placeTurned(world, this.m_deco.get().m_pillarID, this.m_deco.get().m_pillarMeta, 3, top, 3, ladderUpDir, sbb);
    ice_tower_wing::decoratePillars(this, world, random, bottom, top, ladderUpDir, sbb);
    if hasTreasure
    {
        this.placeTreasureRotatedTrapped(world, 3, bottom + 5, 2, ladderUpDir, Table::AuroraCache, false, sbb);
    }
}

// ComponentTFIceTowerWing.decoratePillarPlatformsOutside
pub fn decoratePillarPlatformsOutside(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    bottom: i32,
    top: i32,
    ladderUpDir: i32,
    ladderDownDir: i32,
    hasTreasure: bool,
    sbb: StructureBox,
)
{
    for i in 1..8
    {
        let rotation: i32 = (ladderUpDir + i) % 4;
        this.fillTurned(
            world,
            sbb,
            1,
            bottom + i,
            1,
            3,
            bottom + i,
            3,
            this.m_deco.get().m_platformID,
            this.m_deco.get().m_platformMeta,
            rotation,
        );
        this.fillTurned(
            world,
            sbb,
            4,
            bottom + i,
            1,
            6,
            bottom + i,
            3,
            this.m_deco.get().m_floorID,
            this.m_deco.get().m_floorMeta,
            rotation,
        );
    }
    let rotation: i32 = (ladderUpDir + 2) % 4;
    this.fillAirTurned(world, sbb, 5, top, 8, 9, top, 9, rotation);
    this.fillAirTurned(world, sbb, 8, top, 6, 9, top, 9, rotation);
    this.fillTurned(
        world,
        sbb,
        8,
        top - 2,
        7,
        9,
        top - 2,
        7,
        this.m_deco.get().m_platformID,
        this.m_deco.get().m_platformMeta,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        8,
        top - 2,
        8,
        9,
        top - 2,
        9,
        this.m_deco.get().m_floorID,
        this.m_deco.get().m_floorMeta,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        7,
        top - 1,
        8,
        7,
        top - 1,
        9,
        this.m_deco.get().m_platformID,
        this.m_deco.get().m_platformMeta,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        6,
        top - 1,
        8,
        6,
        top - 1,
        9,
        this.m_deco.get().m_platformID,
        this.m_deco.get().m_platformMeta | 8,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        5,
        top - 0,
        8,
        5,
        top - 0,
        9,
        this.m_deco.get().m_platformID,
        this.m_deco.get().m_platformMeta,
        rotation,
    );
    ice_tower_wing::decoratePillars(this, world, random, bottom, top, ladderUpDir, sbb);
    if hasTreasure
    {
        this.placeTreasureRotatedTrapped(world, 3, bottom + 5, 2, ladderUpDir, Table::AuroraCache, false, sbb);
    }
}

// ComponentTFIceTowerWing.decoratePillarParkour
pub fn decoratePillarParkour(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    bottom: i32,
    top: i32,
    ladderUpDir: i32,
    ladderDownDir: i32,
    hasTreasure: bool,
    sbb: StructureBox,
)
{
    let rotation: i32 = ladderDownDir;
    let beamMetaNS: i32 = if ((this.m_mode.get() + rotation) % 2) == 0
    {
        4
    }
    else
    {
        8
    };
    let beamMetaEW: i32 = if beamMetaNS == 4 { 8 } else { 4 };
    ice_tower_wing::decoratePillars(this, world, random, bottom, top, rotation, sbb);
    this.placeTurned(
        world,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta,
        5,
        bottom + 1,
        5,
        rotation,
        sbb,
    );
    this.fillTurned(
        world,
        sbb,
        5,
        bottom + 2,
        7,
        5,
        bottom + 2,
        9,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta + beamMetaEW,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        1,
        bottom + 3,
        7,
        2,
        bottom + 3,
        7,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta + beamMetaNS,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        3,
        bottom + 3,
        8,
        3,
        bottom + 3,
        9,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta + beamMetaEW,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        1,
        bottom + 7,
        7,
        2,
        bottom + 7,
        7,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta + beamMetaNS,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        3,
        bottom + 7,
        8,
        3,
        bottom + 7,
        9,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta + beamMetaEW,
        rotation,
    );
    this.fillAirTurned(world, sbb, 3, bottom + 4, 7, 3, bottom + 6, 7, rotation);
    this.fillTurned(
        world,
        sbb,
        1,
        bottom + 4,
        5,
        2,
        bottom + 4,
        5,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta + beamMetaNS,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        3,
        bottom + 5,
        1,
        3,
        bottom + 5,
        2,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta + beamMetaEW,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        1,
        bottom + 5,
        3,
        2,
        bottom + 5,
        3,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta + beamMetaNS,
        rotation,
    );
    this.fillAirTurned(world, sbb, 3, bottom + 6, 3, 3, bottom + 8, 3, rotation);
    this.fillTurned(
        world,
        sbb,
        5,
        bottom + 6,
        1,
        5,
        bottom + 6,
        2,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta + beamMetaEW,
        rotation,
    );
    this.fillAirTurned(world, sbb, 7, bottom + 8, 3, 7, bottom + 10, 3, rotation);
    this.fillTurned(
        world,
        sbb,
        7,
        bottom + 7,
        1,
        7,
        bottom + 7,
        2,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta + beamMetaEW,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        8,
        bottom + 7,
        3,
        9,
        bottom + 7,
        3,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta + beamMetaNS,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        8,
        bottom + 8,
        5,
        9,
        bottom + 8,
        5,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta + beamMetaNS,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        8,
        bottom + 9,
        7,
        9,
        bottom + 9,
        7,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta + beamMetaNS,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        7,
        bottom + 9,
        8,
        7,
        bottom + 9,
        9,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta + beamMetaEW,
        rotation,
    );
    this.fillAirTurned(world, sbb, 2, top, 2, 8, top, 4, ladderUpDir);
    this.fillAirTurned(world, sbb, 2, top, 2, 4, top, 6, ladderUpDir);
    if hasTreasure
    {
        this.placeTreasureRotatedTrapped(world, 8, bottom + 8, 7, ladderUpDir, Table::AuroraCache, false, sbb);
    }
}

// ComponentTFIceTowerWing.makeARoof
pub fn makeARoof(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    let index: i32 = this.depth();
    tower_wing::tryToFitRoof(this, list, random, &ice_tower_roof::new(index + 1, this));
}

// ComponentTFIceTowerWing.makeABeard
pub fn makeABeard(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    let index: i32 = this.depth();
    let beard: Rc<Piece> = ice_tower_beard::new(index + 1, this);
    list.push(Rc::clone(&beard));
    beard.build(this, list, random);
}
