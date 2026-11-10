// draft from ComponentTFMushroomTowerWing.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub const RANGE: i32 = 200;
pub const FLOOR_HEIGHT: i32 = 4;
pub const MAIN_SIZE: i32 = 15;

pub fn new(i: i32, x: i32, y: i32, z: i32, pSize: i32, pHeight: i32, direction: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::MushroomTowerWing);
    init(&this, i, x, y, z, pSize, pHeight, direction);
    Rc::new(this)
}

// ComponentTFMushroomTowerWing constructor
pub fn init(this: &Piece, i: i32, x: i32, y: i32, z: i32, pSize: i32, pHeight: i32, direction: i32)
{
    tower_wing::init7(this, i, x, y, z, pSize, pHeight, direction);
    this.m_hasBase.set(false);
    this.m_isAscender.set(false);
}

// ComponentTFMushroomTowerWing.func_74861_a
pub fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    this.m_deco.set((parent).m_deco.get());
    this.addOpening(0, 1, this.m_size.get() / 2, 2);
    this.m_hasBase.set(this.m_size.get() > 3);
    if this.m_isAscender.get()
    {
        let mut dest: Vec<i32> = this.getValidOpening(random, 2);
        dest[1] = this.m_height.get() - 3;
        let childHeight: i32 = (((random.nextIntBounded(3) + random.nextIntBounded(3)) + 2) * 4) + 1;
        let madeIt: bool = mushroom_tower_wing::makeMainBridge(
            this,
            list,
            random,
            this.depth() + 1,
            dest[0],
            dest[1],
            dest[2],
            this.m_size.get() + 4,
            childHeight,
            2,
        );
        if !madeIt
        {
        }
        else
        {
        }
    }
    if (this.depth() < 5) && (this.m_size.get() > 6)
    {
        for i in 0..4
        {
            if (this.m_size.get() >= 15) || (i != 2)
            {
                let mut dest: Vec<i32> = this.getValidOpening(random, i);
                let childHeight: i32 = (((random.nextIntBounded(2) + random.nextIntBounded(2)) + 2) * 4) + 1;
                this.makeBridge(
                    list,
                    random,
                    this.depth() + 1,
                    dest[0],
                    dest[1],
                    dest[2],
                    this.m_size.get() - 4,
                    childHeight,
                    i,
                );
            }
        }
    }
    if mushroom_tower_wing::isHighest(this, this.m_box.get(), this.m_size.get(), list) || (!this.m_hasBase.get())
    {
        this.makeARoof(&parent, list, random);
    }
}

// ComponentTFMushroomTowerWing.isOutOfRange
pub fn isOutOfRange(this: &Piece, parent: &Piece, nx: i32, ny: i32, nz: i32, range: i32) -> bool
{
    return ((nx - parent.bounds().centerX()).abs() > range) || ((nz - parent.bounds().centerZ()).abs() > range);
}

// ComponentTFMushroomTowerWing.makeTowerWing
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
    let mut dx: Vec<i32> = this.offsetTower(x, y, z, wingSize, direction);
    if this.isOutOfRange(&Rc::clone(&list[(0) as usize]), dx[0], dx[1], dx[2], 200)
    {
        return false;
    }
    if wingSize > 3
    {
        dx = mushroom_tower_wing::adjustCoordinates(this, dx[0], dx[1], dx[2], wingSize, direction, list);
    }
    let mut wing: Rc<Piece> = mushroom_tower_wing::new(index, dx[0], dx[1], dx[2], wingSize, wingHeight, direction);
    let intersect: Option<Rc<Piece>> = intersecting(list, wing.bounds());
    if ((intersect.is_some()) && (!same(&intersect, &this)))
        && (!(intersect.as_ref().is_some_and(|piece| matches!(piece.m_kind, Kind::TowerRoofMushroom))))
    {
        return false;
    }
    if (matches!(this.m_kind, Kind::MushroomTowerBridge | Kind::MushroomTowerMainBridge)) && this.m_isAscender.get()
    {
        wing.m_isAscender.set(true);
    }
    list.push(Rc::clone(&wing));
    wing.build(&Rc::clone(&list[(0) as usize]), list, random);
    this.addOpening(x, y, z, rotation);
    return true;
}

// ComponentTFMushroomTowerWing.adjustCoordinates
pub fn adjustCoordinates(
    this: &Piece,
    x: i32,
    y: i32,
    z: i32,
    wingSize: i32,
    direction: i32,
    list: &mut Pieces,
) -> Vec<i32>
{
    for obj in list.clone()
    {
        if matches!(
            obj.m_kind,
            Kind::DarkTowerBalcony
                | Kind::DarkTowerBossBridge
                | Kind::DarkTowerBossTrap
                | Kind::DarkTowerBridge
                | Kind::DarkTowerEntrance
                | Kind::DarkTowerEntranceBridge
                | Kind::DarkTowerMain
                | Kind::DarkTowerMainBridge
                | Kind::DarkTowerWing
                | Kind::FinalCastleBellTower21
                | Kind::FinalCastleDamagedTower
                | Kind::FinalCastleDungeonEntrance
                | Kind::FinalCastleDungeonExit
                | Kind::FinalCastleDungeonRoom31
                | Kind::FinalCastleEntranceBottomTower
                | Kind::FinalCastleEntranceSideTower
                | Kind::FinalCastleEntranceTower
                | Kind::FinalCastleLargeTower
                | Kind::FinalCastleMazeTower13
                | Kind::FinalCastleStairTower
                | Kind::FinalCastleWreckedTower
                | Kind::IceTowerBossWing
                | Kind::IceTowerEntrance
                | Kind::IceTowerMain
                | Kind::IceTowerStairs
                | Kind::IceTowerWing
                | Kind::TowerBridge
                | Kind::TowerMain
                | Kind::TowerOutbuilding
                | Kind::TowerWing
                | Kind::MushroomTowerBridge
                | Kind::MushroomTowerMain
                | Kind::MushroomTowerMainBridge
                | Kind::MushroomTowerWing
        )
        {
            let otherWing: Rc<Piece> = Rc::clone(&obj);
            if ((!(matches!(obj.m_kind, Kind::MushroomTowerBridge | Kind::MushroomTowerMainBridge)))
                && (wingSize == otherWing.m_size.get()))
                && otherWing.bounds().intersectsXZ(x - 3, z - 3, x + 3, z + 3)
            {
                match direction
                {
                    0 =>
                    {
                        return vec![otherWing.bounds().m_min[0], y, otherWing.bounds().m_min[2]];
                    }
                    1 =>
                    {
                        return vec![otherWing.bounds().m_max[0], y, otherWing.bounds().m_min[2]];
                    }
                    2 =>
                    {
                        return vec![otherWing.bounds().m_max[0], y, otherWing.bounds().m_max[2]];
                    }
                    3 =>
                    {
                        return vec![otherWing.bounds().m_min[0], y, otherWing.bounds().m_max[2]];
                    }
                    _ =>
                    {}
                }
            }
        }
    }
    return vec![x, y, z];
}

// ComponentTFMushroomTowerWing.isHighest
pub fn isHighest(this: &Piece, boundingBox: StructureBox, size: i32, list: &mut Pieces) -> bool
{
    let mut boxAbove: StructureBox = boundingBox;
    boxAbove.m_max[1] = 256;
    for obj in list.clone()
    {
        if (!same(&this, &obj))
            && (matches!(
                obj.m_kind,
                Kind::DarkTowerBalcony
                    | Kind::DarkTowerBossBridge
                    | Kind::DarkTowerBossTrap
                    | Kind::DarkTowerBridge
                    | Kind::DarkTowerEntrance
                    | Kind::DarkTowerEntranceBridge
                    | Kind::DarkTowerMain
                    | Kind::DarkTowerMainBridge
                    | Kind::DarkTowerWing
                    | Kind::FinalCastleBellTower21
                    | Kind::FinalCastleDamagedTower
                    | Kind::FinalCastleDungeonEntrance
                    | Kind::FinalCastleDungeonExit
                    | Kind::FinalCastleDungeonRoom31
                    | Kind::FinalCastleEntranceBottomTower
                    | Kind::FinalCastleEntranceSideTower
                    | Kind::FinalCastleEntranceTower
                    | Kind::FinalCastleLargeTower
                    | Kind::FinalCastleMazeTower13
                    | Kind::FinalCastleStairTower
                    | Kind::FinalCastleWreckedTower
                    | Kind::IceTowerBossWing
                    | Kind::IceTowerEntrance
                    | Kind::IceTowerMain
                    | Kind::IceTowerStairs
                    | Kind::IceTowerWing
                    | Kind::TowerBridge
                    | Kind::TowerMain
                    | Kind::TowerOutbuilding
                    | Kind::TowerWing
                    | Kind::MushroomTowerBridge
                    | Kind::MushroomTowerMain
                    | Kind::MushroomTowerMainBridge
                    | Kind::MushroomTowerWing
            ))
        {
            let otherWing: Rc<Piece> = Rc::clone(&obj);
            if ((!(matches!(obj.m_kind, Kind::MushroomTowerBridge | Kind::MushroomTowerMainBridge)))
                && (size == otherWing.m_size.get()))
                && otherWing.bounds().intersects(&boxAbove)
            {
                return false;
            }
        }
    }
    return true;
}

// ComponentTFMushroomTowerWing.makeARoof
pub fn makeARoof(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    let mut roof: Rc<Piece> = tower_roof_mushroom::new(this.depth() + 1, this, 1.6);
    if intersecting(list, roof.bounds()).as_ref().is_some_and(|piece| matches!(piece.m_kind, Kind::TowerRoofMushroom))
    {
        list.push(Rc::clone(&roof));
        roof.build(this, list, random);
    }
    else
    {
        roof = tower_roof_mushroom::new(this.depth() + 1, this, 1.0);
        if intersecting(list, roof.bounds())
            .as_ref()
            .is_some_and(|piece| matches!(piece.m_kind, Kind::TowerRoofMushroom))
        {
            list.push(Rc::clone(&roof));
            roof.build(this, list, random);
        }
        else
        {
            roof = tower_roof_mushroom::new(this.depth() + 1, this, 0.6);
            list.push(Rc::clone(&roof));
            roof.build(this, list, random);
        }
    }
}

// ComponentTFMushroomTowerWing.makeBridge
pub fn makeBridge(
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
    return mushroom_tower_wing::makeBridge10(
        this, list, random, index, x, y, z, wingSize, wingHeight, rotation, false,
    );
}

// ComponentTFMushroomTowerWing.makeBridge
pub fn makeBridge10(
    this: &Piece,
    list: &mut Pieces,
    random: &Random,
    index: i32,
    x: i32,
    y: i32,
    z: i32,
    wingSize: i32,
    mut wingHeight: i32,
    rotation: i32,
    ascender: bool,
) -> bool
{
    let direction: i32 = (this.mode() + rotation) % 4;
    let dx: Vec<i32> = this.offsetTower(x, y, z, 3, direction);
    if wingSize == 3
    {
        wingHeight = 4;
    }
    let mut bridge: Rc<Piece> = mushroom_tower_bridge::new(index, dx[0], dx[1], dx[2], wingSize, wingHeight, direction);
    bridge.m_isAscender.set(ascender);
    let mut intersect: Option<Rc<Piece>> = intersecting(list, bridge.bounds());
    if (intersect.is_some()) && (!same(&intersect, &this))
    {
        return false;
    }
    intersect = intersecting(list, bridge.getWingBB());
    if (intersect.is_some()) && (!same(&intersect, &this))
    {
        return false;
    }
    list.push(Rc::clone(&bridge));
    bridge.build(this, list, random);
    this.addOpening(x, y, z, rotation);
    return true;
}

// ComponentTFMushroomTowerWing.makeMainBridge
pub fn makeMainBridge(
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
    let dx: Vec<i32> = this.offsetTower(x, y, z, 3, direction);
    let bridge: Rc<Piece> =
        mushroom_tower_main_bridge::new(index, dx[0], dx[1], dx[2], wingSize, wingHeight, direction);
    list.push(Rc::clone(&bridge));
    bridge.build(this, list, random);
    this.addOpening(x, y, z, rotation);
    return true;
}

// ComponentTFMushroomTowerWing.getValidOpening
pub fn getValidOpening(this: &Piece, random: &Random, direction: i32) -> Vec<i32>
{
    let wLength: i32 = (this.m_size.get() / 3).min(3);
    let offset: i32 = (this.m_size.get() - wLength) / 2;
    if (direction != 0) && (direction != 2)
    {
        if (direction != 1) && (direction != 3)
        {
            return vec![0, 0, 0];
        }
        let rx: i32 = offset + random.nextIntBounded(wLength);
        let rz: i32 = if direction == 1
        {
            this.m_size.get() - 1
        }
        else
        {
            0
        };
        let ry: i32 = this.getYByStairs(rx, random, direction);
        return vec![rx, ry, rz];
    }
    else
    {
        let rx: i32 = if direction == 0
        {
            this.m_size.get() - 1
        }
        else
        {
            0
        };
        let rz: i32 = offset + random.nextIntBounded(wLength);
        let ry: i32 = this.getYByStairs(rz, random, direction);
        return vec![rx, ry, rz];
    }
}

// ComponentTFMushroomTowerWing.getYByStairs
pub fn getYByStairs(this: &Piece, rx: i32, random: &Random, direction: i32) -> i32
{
    let floors: i32 = this.m_height.get() / 4;
    return 5 + (random.nextIntBounded(floors - 1) * 4);
}

// ComponentTFMushroomTowerWing.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    let decoRNG: Random = Random::withSeed(
        (world.seed() + ((this.m_box.get().m_min[0] * 321534781) as i64))
            ^ ((this.m_box.get().m_min[2] * 756839) as i64),
    );
    mushroom_tower_wing::makeTrunk(this, world, sbb);
    this.makeFloorsForTower(world, &decoRNG, sbb);
    this.darken(world);
    this.makeOpenings(world, sbb);
    return true;
}

// ComponentTFMushroomTowerWing.makeTrunk
pub fn makeTrunk(this: &Piece, world: &GenWorld, sbb: StructureBox)
{
    let diameter: i32 = this.m_size.get() / 2;
    let hollow: i32 = ((diameter) as f64 * 0.8) as i32;
    let cx: i32 = diameter;
    let cz: i32 = diameter;
    for dx in -diameter..=diameter
    {
        for dz in -diameter..=diameter
        {
            let ax: i32 = dx.abs();
            let az: i32 = dz.abs();
            let dist: i32 = ((ax.max(az)) as f64 + ((ax.min(az)) as f64 * 0.4)) as i32;
            if dist <= diameter
            {
                this.placeBlock(
                    world,
                    this.m_deco.get().m_floorID,
                    this.m_deco.get().m_floorMeta,
                    dx + cx,
                    0,
                    dz + cz,
                    sbb,
                );
                this.placeBlock(
                    world,
                    this.m_deco.get().m_floorID,
                    this.m_deco.get().m_floorMeta,
                    dx + cx,
                    this.m_height.get(),
                    dz + cz,
                    sbb,
                );
                if dist > hollow
                {
                    for dy in 0..=this.m_height.get()
                    {
                        this.placeBlock(
                            world,
                            this.m_deco.get().m_blockID,
                            this.m_deco.get().m_blockMeta,
                            dx + cx,
                            dy,
                            dz + cz,
                            sbb,
                        );
                    }
                }
                else
                {
                    for dy in 1..=(this.m_height.get() - 1)
                    {
                        this.placeBlock(world, blk("air"), 0, dx + cx, dy, dz + cz, sbb);
                    }
                }
                if this.m_hasBase.get()
                {
                    this.fillDown(
                        world,
                        this.m_deco.get().m_blockID,
                        this.m_deco.get().m_blockMeta,
                        dx + cx,
                        -1,
                        dz + cz,
                        sbb,
                    );
                }
            }
        }
    }
}

// ComponentTFMushroomTowerWing.makeFloorsForTower
pub fn makeFloorsForTower(this: &Piece, world: &GenWorld, decoRNG: &Random, sbb: StructureBox)
{
    let floors: i32 = this.m_height.get() / 4;
    let mut ladderDir: i32 = 3;
    let downLadderDir: i32 = -1;
    for i in 0..floors
    {
        mushroom_tower_wing::placeFloor(this, world, i * 4, sbb);
        ladderDir =
        {
            ladderDir += 1;
            ladderDir
        } % 4;
    }
}

// ComponentTFMushroomTowerWing.placeFloor
pub fn placeFloor(this: &Piece, world: &GenWorld, dy: i32, sbb: StructureBox)
{
    let diameter: i32 = this.m_size.get() / 2;
    let hollow: i32 = ((diameter) as f64 * 0.8) as i32;
    let cx: i32 = diameter;
    let cz: i32 = diameter;
    for dx in -diameter..=diameter
    {
        for dz in -diameter..=diameter
        {
            let ax: i32 = dx.abs();
            let az: i32 = dz.abs();
            let dist: i32 = ((ax.max(az)) as f64 + ((ax.min(az)) as f64 * 0.4)) as i32;
            if dist <= hollow
            {
                this.placeBlock(
                    world,
                    this.m_deco.get().m_floorID,
                    if this.m_isAscender.get()
                    {
                        3
                    }
                    else
                    {
                        this.m_deco.get().m_floorMeta
                    },
                    dx + cx,
                    dy,
                    dz + cz,
                    sbb,
                );
            }
        }
    }
}

// ComponentTFMushroomTowerWing.makeDoorOpening
pub fn makeDoorOpening(this: &Piece, world: &GenWorld, dx: i32, dy: i32, dz: i32, sbb: StructureBox)
{
    tower_wing::makeDoorOpening(this, world, dx, dy, dz, sbb);
    if this.getBlock(world, dx, dy + 2, dz, sbb) != blk("air")
    {
        this.placeBlock(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, dx, dy + 2, dz, sbb);
    }
}

// ComponentTFMushroomTowerWing.decorateFloor
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
}
