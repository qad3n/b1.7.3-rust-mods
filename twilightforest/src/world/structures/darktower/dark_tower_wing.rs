// draft from ComponentTFDarkTowerWing.java by tools/structgen.py
use crate::world::structures::prelude::*;

// a single chest holds this many stacks
const CHEST_SLOTS: i32 = 27;

pub(crate) fn new(i: i32, x: i32, y: i32, z: i32, pSize: i32, pHeight: i32, direction: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::DarkTowerWing);
    init(&this, i, x, y, z, pSize, pHeight, direction);
    Rc::new(this)
}

// ComponentTFDarkTowerWing constructor
pub(crate) fn init(this: &Piece, i: i32, x: i32, y: i32, z: i32, pSize: i32, pHeight: i32, direction: i32)
{
    tower_wing::init7(this, i, x, y, z, pSize, pHeight, direction);
    this.m_keyTower.set(false);
    *this.m_openingTypes.borrow_mut() = Vec::new();
}

// ComponentTFDarkTowerWing.func_74861_a
pub(crate) fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    this.m_deco.set((parent).m_deco.get());
    this.addOpening(0, 1, this.m_size.get() / 2, 2);
    this.makeARoof(parent, list, random);
    this.makeABeard(parent, list, random);

    if this.m_size.get() > 10
    {
        for direction in 0..4
        {
            let dest: Vec<i32> = this.getValidOpening(random, direction);
            let childSize: i32 = this.m_size.get() - 2;
            let childHeight: i32 = dark_tower_wing::validateChildHeight(
                this,
                ((this.m_height.get() - 4) + random.nextIntBounded(10)) - random.nextIntBounded(10),
                childSize,
            );

            let madeWing: bool = this.makeTowerWing(
                list,
                random,
                this.depth(),
                dest[0],
                dest[1],
                dest[2],
                this.m_size.get() - 2,
                childHeight,
                direction,
            );

            if (!madeWing) && ((direction == 2) || random.nextBoolean())
            {
                dark_tower_wing::makeTowerBalcony(
                    this,
                    list,
                    random,
                    this.depth(),
                    dest[0],
                    dest[1],
                    dest[2],
                    direction,
                );
            }
        }
    }
    else if random.nextIntBounded(4) == 0
    {
        let direction: i32 = random.nextIntBounded(4);
        let dest: Vec<i32> = this.getValidOpening(random, direction);
        dark_tower_wing::makeTowerBalcony(this, list, random, this.depth(), dest[0], dest[1], dest[2], direction);
    }
}

// ComponentTFDarkTowerWing.validateChildHeight
pub(crate) fn validateChildHeight(_this: &Piece, childHeight: i32, _childSize: i32) -> i32
{
    ((childHeight / 4) * 4) + 1
}

// ComponentTFDarkTowerWing.makeARoof
pub(crate) fn makeARoof(this: &Piece, _parent: &Piece, list: &mut Pieces, random: &Random)
{
    let index: i32 = this.depth();

    let var10000: Rc<Piece> = match random.nextIntBounded(5)
    {
        2 => dark_tower_roof_cactus::new(index, this),
        3 => dark_tower_roof_rings::new(index, this),
        4 => dark_tower_roof_four_post::new(index, this),
        _ => dark_tower_roof_antenna::new(index, this),
    };
    let roof: Rc<Piece> = Rc::clone(&var10000);
    list.push(Rc::clone(&roof));
    roof.build(this, list, random);
    this.m_roofType.set(Some(roof.m_kind));
}

// ComponentTFDarkTowerWing.makeAttachedRoof
pub(crate) fn makeAttachedRoof(this: &Piece, list: &mut Pieces, random: &Random)
{
    let index: i32 = this.depth();

    if this.m_roofType.get().is_none() && (random.nextIntBounded(32) != 0)
    {
        tower_wing::tryToFitRoof(this, list, random, &tower_roof_gable_forwards::new(index + 1, this));
    }

    if this.m_roofType.get().is_none() && (random.nextIntBounded(8) != 0)
    {
        tower_wing::tryToFitRoof(this, list, random, &tower_roof_slab_forwards::new(index + 1, this));
    }

    if this.m_roofType.get().is_none() && (random.nextIntBounded(32) != 0)
    {
        let roof: Rc<Piece> = tower_roof_attached_slab::new(index + 1, this);
        tower_wing::tryToFitRoof(this, list, random, &roof);
    }

    if this.m_roofType.get().is_none()
    {
        let roof: Rc<Piece> = tower_roof_fence::new(index + 1, this);
        tower_wing::tryToFitRoof(this, list, random, &roof);
    }
}

// ComponentTFDarkTowerWing.makeABeard
pub(crate) fn makeABeard(this: &Piece, _parent: &Piece, list: &mut Pieces, random: &Random)
{
    let beard: Rc<Piece> = dark_tower_beard::new(this.depth() + 1, this);
    list.push(Rc::clone(&beard));
    beard.build(this, list, random);
}

// ComponentTFDarkTowerWing.makeTowerWing
pub(crate) fn makeTowerWing(
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
    if wingHeight < 8
    {
        return false;
    }
    let direction: i32 = (this.mode() + rotation) % 4;
    let dx: Vec<i32> = this.offsetTower(x, y, z, 5, direction);
    if (dx[1] + wingHeight) > 250
    {
        return false;
    }
    let bridge: Rc<Piece> = dark_tower_bridge::new(index, dx[0], dx[1], dx[2], wingSize, wingHeight, direction);
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
    true
}

// ComponentTFDarkTowerWing.makeTowerBalcony
pub(crate) fn makeTowerBalcony(
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
    let direction: i32 = (this.mode() + rotation) % 4;
    let dx: Vec<i32> = this.offsetTower(x, y, z, 5, direction);
    let balcony: Rc<Piece> = dark_tower_balcony::new(index, dx[0], dx[1], dx[2], direction);
    let intersect: Option<Rc<Piece>> = intersecting(list, balcony.bounds());
    if (intersect.is_some()) && (!same(&intersect, &this))
    {
        return false;
    }

    list.push(Rc::clone(&balcony));
    balcony.build(this, list, random);
    dark_tower_wing::addOpening5(this, x, y, z, rotation, Door::Reappearing);
    true
}

// ComponentTFDarkTowerWing.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
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
    this.darken(world);

    if this.m_size.get() > 9
    {
        dark_tower_wing::addHalfFloors(this, world, &decoRNG, sbb, 4, this.m_height.get() - 1);
    }
    else if decoRNG.nextIntBounded(3) == 0
    {
        dark_tower_wing::addSmallTimberBeams(this, world, &decoRNG, sbb, 4, this.m_height.get() - 1);
    }
    else
    {
        dark_tower_wing::addHalfFloors(this, world, &decoRNG, sbb, 4, this.m_height.get() - 1);
    }
    this.makeOpenings(world, sbb);

    if (decoRNG.nextBoolean() && (!dark_tower_wing::isKeyTower(this))) && (this.m_height.get() > 8)
    {
        let mut blobs: i32 = 1;

        if (this.m_size.get() > 9) && decoRNG.nextBoolean()
        {
            blobs += 1;
        }

        for _i in 0..blobs
        {
            let x: i32 = decoRNG.nextIntBounded(this.m_size.get());
            let y: i32 = decoRNG.nextIntBounded(this.m_height.get() - 7) + 2;
            let z: i32 = decoRNG.nextIntBounded(this.m_size.get());
            dark_tower_wing::destroyTower7(this, world, &decoRNG, x, y, z, 3, sbb);
        }
    }

    true
}

// ComponentTFDarkTowerWing.destroyTower
pub(crate) fn destroyTower7(
    this: &Piece,
    world: &GenWorld,
    decoRNG: &Random,
    x: i32,
    y: i32,
    z: i32,
    amount: i32,
    sbb: StructureBox,
)
{
    let initialRadius: i32 = decoRNG.nextIntBounded(amount) + amount;
    dark_tower_wing::drawBlob(this, world, x, y, z, initialRadius, blk("air"), 0, sbb);

    for _i in 0..3
    {
        let dx: i32 = x + ((initialRadius - 1) * (if decoRNG.nextBoolean() { 1 } else { -1 }));
        let dy: i32 = y + ((initialRadius - 1) * (if decoRNG.nextBoolean() { 1 } else { -1 }));
        let dz: i32 = z + ((initialRadius - 1) * (if decoRNG.nextBoolean() { 1 } else { -1 }));
        dark_tower_wing::netherTransformBlob(this, world, decoRNG, dx, dy, dz, initialRadius - 1, sbb);
        dark_tower_wing::drawBlob(this, world, dx, dy, dz, initialRadius - 2, blk("air"), 0, sbb);
    }
}

// ComponentTFDarkTowerWing.netherTransformBlob
pub(crate) fn netherTransformBlob(
    this: &Piece,
    world: &GenWorld,
    inRand: &Random,
    sx: i32,
    sy: i32,
    sz: i32,
    rad: i32,
    sbb: StructureBox,
)
{
    let rand: Random = Random::withSeed(inRand.nextLong());

    for dx in 0..=rad
    {
        for dy in 0..=rad
        {
            for dz in 0..=rad
            {
                let dist: i32 = if (dx >= dy) && (dx >= dz)
                {
                    dx + ((((dy.max(dz)) as f64 * 0.5) + ((dy.min(dz)) as f64 * 0.25)) as i32)
                }
                else if (dy >= dx) && (dy >= dz)
                {
                    dy + ((((dx.max(dz)) as f64 * 0.5) + ((dx.min(dz)) as f64 * 0.25)) as i32)
                }
                else
                {
                    dz + ((((dx.max(dy)) as f64 * 0.5) + ((dx.min(dy)) as f64 * 0.25)) as i32)
                };

                if dist <= rad
                {
                    dark_tower_wing::testAndChangeToNetherrack(this, world, &rand, sx + dx, sy + dy, sz + dz, sbb);
                    dark_tower_wing::testAndChangeToNetherrack(this, world, &rand, sx + dx, sy + dy, sz + dz, sbb);
                    dark_tower_wing::testAndChangeToNetherrack(this, world, &rand, sx + dx, sy + dy, sz - dz, sbb);
                    dark_tower_wing::testAndChangeToNetherrack(this, world, &rand, sx - dx, sy + dy, sz + dz, sbb);
                    dark_tower_wing::testAndChangeToNetherrack(this, world, &rand, sx - dx, sy + dy, sz - dz, sbb);
                    dark_tower_wing::testAndChangeToNetherrack(this, world, &rand, sx + dx, sy - dy, sz + dz, sbb);
                    dark_tower_wing::testAndChangeToNetherrack(this, world, &rand, sx + dx, sy - dy, sz - dz, sbb);
                    dark_tower_wing::testAndChangeToNetherrack(this, world, &rand, sx - dx, sy - dy, sz + dz, sbb);
                    dark_tower_wing::testAndChangeToNetherrack(this, world, &rand, sx - dx, sy - dy, sz - dz, sbb);
                }
            }
        }
    }
}

// ComponentTFDarkTowerWing.testAndChangeToNetherrack
pub(crate) fn testAndChangeToNetherrack(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    x: i32,
    y: i32,
    z: i32,
    sbb: StructureBox,
)
{
    if this.getBlock(world, x, y, z, sbb) != blk("air")
    {
        this.placeBlock(world, blk("netherrack"), 0, x, y, z, sbb);

        if (this.getBlock(world, x, y + 1, z, sbb) == blk("air")) && random.nextBoolean()
        {
            this.placeBlock(world, blk("fire"), 0, x, y + 1, z, sbb);
        }
    }
}

// ComponentTFDarkTowerWing.drawBlob
pub(crate) fn drawBlob(
    this: &Piece,
    world: &GenWorld,
    sx: i32,
    sy: i32,
    sz: i32,
    rad: i32,
    blockValue: Blk,
    metaValue: i32,
    sbb: StructureBox,
)
{
    for dx in 0..=rad
    {
        for dy in 0..=rad
        {
            for dz in 0..=rad
            {
                let dist: i32 = if (dx >= dy) && (dx >= dz)
                {
                    dx + ((((dy.max(dz)) as f64 * 0.5) + ((dy.min(dz)) as f64 * 0.25)) as i32)
                }
                else if (dy >= dx) && (dy >= dz)
                {
                    dy + ((((dx.max(dz)) as f64 * 0.5) + ((dx.min(dz)) as f64 * 0.25)) as i32)
                }
                else
                {
                    dz + ((((dx.max(dy)) as f64 * 0.5) + ((dx.min(dy)) as f64 * 0.25)) as i32)
                };

                if dist <= rad
                {
                    this.placeBlock(world, blockValue, metaValue, sx + dx, sy + dy, sz + dz, sbb);
                    this.placeBlock(world, blockValue, metaValue, sx + dx, sy + dy, sz - dz, sbb);
                    this.placeBlock(world, blockValue, metaValue, sx - dx, sy + dy, sz + dz, sbb);
                    this.placeBlock(world, blockValue, metaValue, sx - dx, sy + dy, sz - dz, sbb);
                    this.placeBlock(world, blockValue, metaValue, sx + dx, sy - dy, sz + dz, sbb);
                    this.placeBlock(world, blockValue, metaValue, sx + dx, sy - dy, sz - dz, sbb);
                    this.placeBlock(world, blockValue, metaValue, sx - dx, sy - dy, sz + dz, sbb);
                    this.placeBlock(world, blockValue, metaValue, sx - dx, sy - dy, sz - dz, sbb);
                }
            }
        }
    }
}

// ComponentTFDarkTowerWing.addHalfFloors
pub(crate) fn addHalfFloors(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    sbb: StructureBox,
    mut bottom: i32,
    top: i32,
)
{
    let spacing: i32 = 4;
    let mut rotation: i32 = (this.m_box.get().m_min[1] + bottom) % 3;

    if bottom == 0
    {
        bottom += spacing;
    }

    {
        let mut y: i32 = bottom;
        while y < top
        {
            rotation += 2;
            rotation %= 4;

            if y >= (top - spacing)
            {
                dark_tower_wing::makeFullFloor(this, world, sbb, rotation, y, spacing);

                if this.isDeadEnd()
                {
                    dark_tower_wing::decorateTreasureRoom(this, world, sbb, rotation, y, 4, this.m_deco.get());
                }
            }
            else
            {
                dark_tower_wing::makeHalfFloor(this, world, sbb, rotation, y, spacing);

                'case4:
                {
                    match random.nextIntBounded(8)
                    {
                        // todo falls through to the next case
                        0 =>
                        {
                            if this.m_size.get() < 11
                            {
                                dark_tower_wing::decorateReappearingFloor(this, world, random, sbb, rotation, y);
                                break 'case4;
                            }
                        }

                        1 =>
                        {
                            dark_tower_wing::decorateSpawner(this, world, random, sbb, rotation, y);
                        }

                        2 =>
                        {
                            this.decorateLounge(world, random, sbb, rotation, y);
                        }

                        3 =>
                        {
                            dark_tower_wing::decorateLibrary(this, world, random, sbb, rotation, y);
                        }

                        4 =>
                        {
                            dark_tower_wing::decorateExperimentPulser(this, world, random, sbb, rotation, y);
                        }

                        5 =>
                        {
                            dark_tower_wing::decorateExperimentLamp(this, world, random, sbb, rotation, y);
                        }

                        6 =>
                        {
                            dark_tower_wing::decoratePuzzleChest(this, world, random, sbb, rotation, y);
                        }

                        _ =>
                        {}
                    }
                }
            }
            dark_tower_wing::addStairsDown6(this, world, sbb, rotation, y, this.m_size.get() - 2, spacing);

            if this.m_size.get() > 9
            {
                dark_tower_wing::addStairsDown6(this, world, sbb, rotation, y, this.m_size.get() - 3, spacing);
            }

            y += spacing;
        }
    }
    rotation += 2;
    rotation %= 4;
    dark_tower_wing::addStairsDown6(
        this,
        world,
        sbb,
        rotation,
        this.m_height.get() - 1,
        this.m_size.get() - 2,
        spacing,
    );
}

// ComponentTFDarkTowerWing.makeHalfFloor
pub(crate) fn makeHalfFloor(this: &Piece, world: &GenWorld, sbb: StructureBox, rotation: i32, y: i32, _spacing: i32)
{
    this.fillTurned(
        world,
        sbb,
        this.m_size.get() / 2,
        y,
        1,
        this.m_size.get() - 2,
        y,
        this.m_size.get() - 2,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockMeta,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        (this.m_size.get() / 2) - 1,
        y,
        1,
        (this.m_size.get() / 2) - 1,
        y,
        this.m_size.get() - 2,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        rotation,
    );
}

// ComponentTFDarkTowerWing.makeFullFloor
pub(crate) fn makeFullFloor(this: &Piece, world: &GenWorld, sbb: StructureBox, _rotation: i32, y: i32, _spacing: i32)
{
    this.fillMeta(
        world,
        sbb,
        1,
        y,
        1,
        this.m_size.get() - 2,
        y,
        this.m_size.get() - 2,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockMeta,
        blk("air"),
        0,
        false,
    );
    this.fillMeta(
        world,
        sbb,
        this.m_size.get() / 2,
        y,
        1,
        this.m_size.get() / 2,
        y,
        this.m_size.get() - 2,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        blk("air"),
        0,
        true,
    );
}

// ComponentTFDarkTowerWing.decorateTreasureRoom
pub(crate) fn decorateTreasureRoom(
    this: &Piece,
    world: &GenWorld,
    sbb: StructureBox,
    rotation: i32,
    y: i32,
    spacing: i32,
    myDeco: Deco,
)
{
    let x: i32 = this.m_size.get() / 2;
    let z: i32 = this.m_size.get() / 2;

    for dy in 1..spacing
    {
        this.placeTurned(world, myDeco.m_pillarID, myDeco.m_pillarMeta, x - 1, y + dy, z - 1, rotation, sbb);
        this.placeTurned(world, myDeco.m_pillarID, myDeco.m_pillarMeta, x + 1, y + dy, z - 1, rotation, sbb);
        this.placeTurned(world, myDeco.m_pillarID, myDeco.m_pillarMeta, x - 1, y + dy, z + 1, rotation, sbb);
        this.placeTurned(world, myDeco.m_pillarID, myDeco.m_pillarMeta, x + 1, y + dy, z + 1, rotation, sbb);
    }

    this.placeTurned(world, myDeco.m_stairID, this.stairMeta(1 + rotation), x + 0, y + 1, z - 1, rotation, sbb);
    this.placeTurned(world, myDeco.m_stairID, this.stairMeta(0 + rotation), x - 1, y + 1, z + 0, rotation, sbb);
    this.placeTurned(world, myDeco.m_stairID, this.stairMeta(2 + rotation), x + 1, y + 1, z + 0, rotation, sbb);
    this.placeTurned(world, myDeco.m_stairID, this.stairMeta(3 + rotation), x + 0, y + 1, z + 1, rotation, sbb);

    for dy in 2..(spacing - 1)
    {
        this.placeTurned(world, myDeco.m_fenceID, myDeco.m_fenceMeta, x + 0, y + dy, z - 1, rotation, sbb);
        this.placeTurned(world, myDeco.m_fenceID, myDeco.m_fenceMeta, x - 1, y + dy, z + 0, rotation, sbb);
        this.placeTurned(world, myDeco.m_fenceID, myDeco.m_fenceMeta, x + 1, y + dy, z + 0, rotation, sbb);
        this.placeTurned(world, myDeco.m_fenceID, myDeco.m_fenceMeta, x + 0, y + dy, z + 1, rotation, sbb);
    }

    this.placeTurned(
        world,
        myDeco.m_stairID,
        this.stairMeta(1 + rotation) + 4,
        x + 0,
        (y + spacing) - 1,
        z - 1,
        rotation,
        sbb,
    );
    this.placeTurned(
        world,
        myDeco.m_stairID,
        this.stairMeta(0 + rotation) + 4,
        x - 1,
        (y + spacing) - 1,
        z + 0,
        rotation,
        sbb,
    );
    this.placeTurned(
        world,
        myDeco.m_stairID,
        this.stairMeta(2 + rotation) + 4,
        x + 1,
        (y + spacing) - 1,
        z + 0,
        rotation,
        sbb,
    );
    this.placeTurned(
        world,
        myDeco.m_stairID,
        this.stairMeta(3 + rotation) + 4,
        x + 0,
        (y + spacing) - 1,
        z + 1,
        rotation,
        sbb,
    );
    this.placeTurned(world, myDeco.m_platformID, myDeco.m_platformMeta, x, y + 1, z, rotation, sbb);
    this.placeTreasure(
        world,
        world.rand(),
        x,
        y + 2,
        z,
        if dark_tower_wing::isKeyTower(this)
        {
            Table::DarkTowerKey
        }
        else
        {
            Table::DarkTowerCache
        },
        sbb,
    );

    if dark_tower_wing::isKeyTower(this)
    {
        dark_tower_wing::putItemInTreasure(this, world, x, y + 2, z, crate::names::item::TOWER_KEY, sbb);
    }
}

// ComponentTFDarkTowerWing.decorateSpawner
pub(crate) fn decorateSpawner(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox, rotation: i32, y: i32)
{
    let x: i32 = if this.m_size.get() > 9 { 4 } else { 3 };
    let z: i32 = if this.m_size.get() > 9 { 5 } else { 4 };

    let mobID: &'static str = if this.m_size.get() > 9
    {
        if random.nextBoolean()
        {
            mob("Tower Golem")
        }
        else
        {
            mob("Redscale Broodling")
        }
    }
    else
    {
        mob("Redscale Broodling")
    };
    dark_tower_wing::makePillarFrame(this, world, sbb, this.m_deco.get(), rotation, x, y, z, true);
    this.placeSpawnerRotated(world, x + 1, y + 2, z + 1, rotation, mobID, sbb);
}

// ComponentTFDarkTowerWing.decorateLounge
pub(crate) fn decorateLounge(this: &Piece, world: &GenWorld, _random: &Random, sbb: StructureBox, rotation: i32, y: i32)
{
    let mut cx: i32 = if this.m_size.get() > 9 { 9 } else { 7 };
    let cz: i32 = if this.m_size.get() > 9 { 4 } else { 3 };
    this.placeTurned(
        world,
        this.m_deco.get().m_stairID,
        this.stairMeta(3 + rotation),
        cx,
        y + 1,
        cz + 0,
        rotation,
        sbb,
    );
    this.placeTurned(
        world,
        this.m_deco.get().m_stairID,
        this.stairMeta(0 + rotation),
        cx,
        y + 1,
        cz + 1,
        rotation,
        sbb,
    );
    this.placeTurned(
        world,
        this.m_deco.get().m_stairID,
        this.stairMeta(1 + rotation),
        cx,
        y + 1,
        cz + 2,
        rotation,
        sbb,
    );
    cx = if this.m_size.get() > 9 { 5 } else { 3 };
    this.placeTurned(
        world,
        this.m_deco.get().m_stairID,
        this.stairMeta(3 + rotation) + 4,
        cx,
        y + 1,
        cz + 0,
        rotation,
        sbb,
    );
    this.placeTurned(world, blk("wooden_slab"), 9, cx, y + 1, cz + 1, rotation, sbb);
    this.placeTurned(
        world,
        this.m_deco.get().m_stairID,
        this.stairMeta(1 + rotation) + 4,
        cx,
        y + 1,
        cz + 2,
        rotation,
        sbb,
    );
}

// ComponentTFDarkTowerWing.decorateReappearingFloor
pub(crate) fn decorateReappearingFloor(
    this: &Piece,
    world: &GenWorld,
    _random: &Random,
    sbb: StructureBox,
    rotation: i32,
    y: i32,
)
{
    this.fillTurned(world, sbb, 4, y, 3, 7, y, 5, blk("tf:towerDevice"), 0, rotation);
    this.fillTurned(world, sbb, 4, y + 1, 2, 7, y + 1, 2, blk("wooden_pressure_plate"), 0, rotation);
    this.fillTurned(world, sbb, 4, y + 1, 6, 7, y + 1, 6, blk("wooden_pressure_plate"), 0, rotation);
}

// ComponentTFDarkTowerWing.decorateExperimentLamp
pub(crate) fn decorateExperimentLamp(
    this: &Piece,
    world: &GenWorld,
    _random: &Random,
    sbb: StructureBox,
    rotation: i32,
    y: i32,
)
{
    let cx: i32 = if this.m_size.get() > 9 { 5 } else { 3 };
    let cz: i32 = if this.m_size.get() > 9 { 5 } else { 4 };
    this.placeTurned(world, blk("sticky_piston"), 1, cx, y + 1, cz, rotation, sbb);
    this.placeTurned(world, blk("redstone_lamp"), 0, cx, y + 2, cz, rotation, sbb);
    this.placeTurned(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        cx,
        y + 1,
        cz + 1,
        rotation,
        sbb,
    );
    this.placeTurned(
        world,
        blk("lever"),
        dark_tower_wing::leverMeta(this, rotation, 3),
        cx,
        y + 1,
        cz + 2,
        rotation,
        sbb,
    );
    this.placeTurned(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        cx,
        y + 3,
        cz - 1,
        rotation,
        sbb,
    );
    this.placeTurned(
        world,
        blk("lever"),
        dark_tower_wing::leverMeta(this, rotation, 2) + 8,
        cx,
        y + 3,
        cz - 2,
        rotation,
        sbb,
    );
}

// ComponentTFDarkTowerWing.decorateExperimentPulser
pub(crate) fn decorateExperimentPulser(
    this: &Piece,
    world: &GenWorld,
    _random: &Random,
    sbb: StructureBox,
    rotation: i32,
    y: i32,
)
{
    let cx: i32 = if this.m_size.get() > 9 { 6 } else { 5 };
    let cz: i32 = if this.m_size.get() > 9 { 4 } else { 3 };
    this.placeTurned(world, blk("sticky_piston"), 5 - this.stairMeta(3 + rotation), cx, y + 1, cz + 1, rotation, sbb);
    this.placeTurned(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, cx, y + 1, cz, rotation, sbb);
    this.placeTurned(world, blk("redstone_wire"), 0, cx + 1, y + 1, cz, rotation, sbb);
    this.placeTurned(world, blk("wooden_pressure_plate"), 0, cx + 2, y + 1, cz, rotation, sbb);
    this.placeTurned(world, blk("unpowered_repeater"), ((rotation + 1) % 4) + 4, cx - 1, y + 1, cz, rotation, sbb);
    this.placeTurned(world, blk("redstone_wire"), 0, cx - 2, y + 1, cz, rotation, sbb);
    this.placeTurned(world, blk("redstone_wire"), 0, cx - 2, y + 1, cz + 1, rotation, sbb);
    this.placeTurned(world, blk("redstone_wire"), 0, cx - 1, y + 1, cz + 1, rotation, sbb);
}

// ComponentTFDarkTowerWing.decorateLibrary
pub(crate) fn decorateLibrary(
    this: &Piece,
    world: &GenWorld,
    _random: &Random,
    sbb: StructureBox,
    rotation: i32,
    y: i32,
)
{
    let mut bx: i32 = if this.m_size.get() > 9 { 4 } else { 3 };
    let mut bz: i32 = if this.m_size.get() > 9 { 3 } else { 2 };
    dark_tower_wing::makeSmallBookshelf(this, world, sbb, rotation, y, bx, bz);
    bx = if this.m_size.get() > 9 { 9 } else { 7 };
    bz = if this.m_size.get() > 9 { 3 } else { 2 };
    dark_tower_wing::makeSmallBookshelf(this, world, sbb, rotation, y, bx, bz);
}

// ComponentTFDarkTowerWing.makeSmallBookshelf
pub(crate) fn makeSmallBookshelf(
    this: &Piece,
    world: &GenWorld,
    sbb: StructureBox,
    rotation: i32,
    y: i32,
    bx: i32,
    bz: i32,
)
{
    this.placeTurned(
        world,
        this.m_deco.get().m_stairID,
        this.stairMeta(1 + rotation) + 0,
        bx,
        y + 1,
        bz + 0,
        rotation,
        sbb,
    );
    this.placeTurned(
        world,
        this.m_deco.get().m_stairID,
        this.stairMeta(1 + rotation) + 4,
        bx,
        y + 2,
        bz + 0,
        rotation,
        sbb,
    );
    this.placeTurned(world, blk("bookshelf"), 0, bx, y + 1, bz + 1, rotation, sbb);
    this.placeTurned(world, blk("bookshelf"), 0, bx, y + 2, bz + 1, rotation, sbb);
    this.placeTurned(world, blk("bookshelf"), 0, bx, y + 1, bz + 2, rotation, sbb);
    this.placeTurned(world, blk("bookshelf"), 0, bx, y + 2, bz + 2, rotation, sbb);
    this.placeTurned(
        world,
        this.m_deco.get().m_stairID,
        this.stairMeta(3 + rotation) + 0,
        bx,
        y + 1,
        bz + 3,
        rotation,
        sbb,
    );
    this.placeTurned(
        world,
        this.m_deco.get().m_stairID,
        this.stairMeta(3 + rotation) + 4,
        bx,
        y + 2,
        bz + 3,
        rotation,
        sbb,
    );
}

// ComponentTFDarkTowerWing.decoratePuzzleChest
pub(crate) fn decoratePuzzleChest(
    this: &Piece,
    world: &GenWorld,
    _random: &Random,
    sbb: StructureBox,
    rotation: i32,
    y: i32,
)
{
    let x: i32 = if this.m_size.get() > 9 { 4 } else { 3 };
    let z: i32 = if this.m_size.get() > 9 { 5 } else { 4 };
    dark_tower_wing::makePillarFrame(this, world, sbb, this.m_deco.get(), rotation, x, y, z, true);
    this.placeTurned(
        world,
        this.m_deco.get().m_platformID,
        this.m_deco.get().m_platformMeta,
        x + 1,
        y + 1,
        z + 1,
        rotation,
        sbb,
    );
    this.placeTurned(
        world,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockMeta,
        x + 2,
        y + 1,
        z + 1,
        rotation,
        sbb,
    );
    this.placeTurned(
        world,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockMeta,
        x + 0,
        y + 1,
        z + 1,
        rotation,
        sbb,
    );
    this.placeTurned(
        world,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockMeta,
        x + 1,
        y + 1,
        z + 2,
        rotation,
        sbb,
    );
    this.placeTurned(
        world,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockMeta,
        x + 1,
        y + 1,
        z + 0,
        rotation,
        sbb,
    );
    this.placeTurned(
        world,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockMeta,
        x + 2,
        y + 3,
        z + 1,
        rotation,
        sbb,
    );
    this.placeTurned(
        world,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockMeta,
        x + 0,
        y + 3,
        z + 1,
        rotation,
        sbb,
    );
    this.placeTurned(
        world,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockMeta,
        x + 1,
        y + 3,
        z + 2,
        rotation,
        sbb,
    );
    this.placeTurned(world, blk("air"), 0, x + 1, y + 3, z + 0, rotation, sbb);
    this.placeTurned(
        world,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockMeta,
        x + 1,
        y + 3,
        z + 1,
        rotation,
        sbb,
    );
    this.placeTurned(world, blk("sticky_piston"), 5 - this.stairMeta(1 + rotation), x + 1, y + 3, z - 1, rotation, sbb);
    this.placeTurned(
        world,
        this.m_deco.get().m_accentID,
        this.m_deco.get().m_accentMeta,
        x + 1,
        y + 3,
        z - 2,
        rotation,
        sbb,
    );
    this.placeTurned(
        world,
        blk("lever"),
        dark_tower_wing::leverMeta(this, rotation, 5),
        x + 2,
        y + 3,
        z - 2,
        rotation,
        sbb,
    );
    this.placeTreasureRotated(world, x + 1, y + 2, z + 1, rotation, Table::DarkTowerCache, sbb);
}

// ComponentTFDarkTowerWing.makePillarFrame
pub(crate) fn makePillarFrame(
    this: &Piece,
    world: &GenWorld,
    sbb: StructureBox,
    myDeco: Deco,
    rotation: i32,
    x: i32,
    y: i32,
    z: i32,
    fenced: bool,
)
{
    dark_tower_wing::makePillarFrame11(this, world, sbb, myDeco, rotation, x, y, z, 3, 3, 3, fenced);
}

// ComponentTFDarkTowerWing.makePillarFrame
pub(crate) fn makePillarFrame11(
    this: &Piece,
    world: &GenWorld,
    sbb: StructureBox,
    myDeco: Deco,
    rotation: i32,
    x: i32,
    y: i32,
    z: i32,
    width: i32,
    height: i32,
    length: i32,
    fenced: bool,
)
{
    for dx in 0..width
    {
        for dz in 0..length
        {
            if (((dx % 3) == 0) || (dx == (width - 1))) && (((dz % 3) == 0) || (dz == (length - 1)))
            {
                for py in 1..=height
                {
                    this.placeTurned(
                        world,
                        myDeco.m_pillarID,
                        myDeco.m_pillarMeta,
                        x + dx,
                        y + py,
                        z + dz,
                        rotation,
                        sbb,
                    );
                }
            }
            else
            {
                if dx == 0
                {
                    this.placeTurned(
                        world,
                        myDeco.m_stairID,
                        this.stairMeta(0 + rotation),
                        x + dx,
                        y + 1,
                        z + dz,
                        rotation,
                        sbb,
                    );
                    this.placeTurned(
                        world,
                        myDeco.m_stairID,
                        this.stairMeta(0 + rotation) + 4,
                        x + dx,
                        y + height,
                        z + dz,
                        rotation,
                        sbb,
                    );
                }
                else if dx == (width - 1)
                {
                    this.placeTurned(
                        world,
                        myDeco.m_stairID,
                        this.stairMeta(2 + rotation),
                        x + dx,
                        y + 1,
                        z + dz,
                        rotation,
                        sbb,
                    );
                    this.placeTurned(
                        world,
                        myDeco.m_stairID,
                        this.stairMeta(2 + rotation) + 4,
                        x + dx,
                        y + height,
                        z + dz,
                        rotation,
                        sbb,
                    );
                }
                else if dz == 0
                {
                    this.placeTurned(
                        world,
                        myDeco.m_stairID,
                        this.stairMeta(1 + rotation),
                        x + dx,
                        y + 1,
                        z + dz,
                        rotation,
                        sbb,
                    );
                    this.placeTurned(
                        world,
                        myDeco.m_stairID,
                        this.stairMeta(1 + rotation) + 4,
                        x + dx,
                        y + height,
                        z + dz,
                        rotation,
                        sbb,
                    );
                }
                else if dz == (length - 1)
                {
                    this.placeTurned(
                        world,
                        myDeco.m_stairID,
                        this.stairMeta(3 + rotation),
                        x + dx,
                        y + 1,
                        z + dz,
                        rotation,
                        sbb,
                    );
                    this.placeTurned(
                        world,
                        myDeco.m_stairID,
                        this.stairMeta(3 + rotation) + 4,
                        x + dx,
                        y + height,
                        z + dz,
                        rotation,
                        sbb,
                    );
                }

                if fenced && ((((dx == 0) || (dx == (width - 1))) || (dz == 0)) || (dz == (length - 1)))
                {
                    for fy in 2..=(height - 1)
                    {
                        this.placeTurned(
                            world,
                            myDeco.m_fenceID,
                            myDeco.m_fenceMeta,
                            x + dx,
                            y + fy,
                            z + dz,
                            rotation,
                            sbb,
                        );
                    }
                }
            }
        }
    }
}

// ComponentTFDarkTowerWing.putItemInTreasure, into a random empty slot of the chest unless it already holds one
pub(crate) fn putItemInTreasure(
    this: &Piece,
    world: &GenWorld,
    x: i32,
    y: i32,
    z: i32,
    item: &'static str,
    sbb: StructureBox,
)
{
    let (dx, dy, dz): (i32, i32, i32) = (this.xAt(x, z), this.yAt(y), this.zAt(x, z));
    if !sbb.contains(dx, dy, dz)
    {
        return;
    }

    let pos: b173_api::BlockPos = b173_api::BlockPos::new(dx, dy, dz);
    let slots: Vec<Option<b173_api::ItemStack>> =
        (0..CHEST_SLOTS).map(|slot| world.m_world.tileSlot(pos, slot)).collect();

    if slots.iter().flatten().any(|stack| stack.item.is(item))
    {
        return;
    }

    let empty: Vec<i32> = (0..CHEST_SLOTS).filter(|&slot| slots[slot as usize].is_none()).collect();
    if empty.is_empty()
    {
        return;
    }

    let slot: i32 = empty[world.rand().nextIntBounded(empty.len() as i32) as usize];
    if let Err(error) = world.m_world.setTileSlot(pos, slot, Some(b173_api::ItemStack::of(item, 1)))
    {
        b173_api::warn(&format!("dark tower key: {}", error.0));
    }
}

// ComponentTFDarkTowerWing.addStairsDown
pub(crate) fn addStairsDown6(
    this: &Piece,
    world: &GenWorld,
    sbb: StructureBox,
    rotation: i32,
    y: i32,
    sz: i32,
    spacing: i32,
)
{
    for i in 0..spacing
    {
        let sx: i32 = (this.m_size.get() - 3) - i;
        this.placeTurned(
            world,
            this.m_deco.get().m_stairID,
            this.stairMeta(0 + rotation),
            sx,
            y - i,
            sz,
            rotation,
            sbb,
        );
        this.placeTurned(
            world,
            this.m_deco.get().m_accentID,
            this.m_deco.get().m_accentMeta,
            sx,
            (y - 1) - i,
            sz,
            rotation,
            sbb,
        );
        this.placeTurned(world, blk("air"), 0, sx, (y + 1) - i, sz, rotation, sbb);
        this.placeTurned(world, blk("air"), 0, sx, (y + 2) - i, sz, rotation, sbb);
        this.placeTurned(world, blk("air"), 0, sx - 1, (y + 2) - i, sz, rotation, sbb);
        this.placeTurned(world, blk("air"), 0, sx, (y + 3) - i, sz, rotation, sbb);
        this.placeTurned(world, blk("air"), 0, sx - 1, (y + 3) - i, sz, rotation, sbb);
    }
}

// ComponentTFDarkTowerWing.addSmallTimberBeams
pub(crate) fn addSmallTimberBeams(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    sbb: StructureBox,
    mut bottom: i32,
    top: i32,
)
{
    let spacing: i32 = 4;
    let mut rotation: i32 = 0;

    if bottom == 0
    {
        bottom += spacing;
    }

    {
        let mut y: i32 = bottom;
        while y < top
        {
            rotation =
            {
                rotation += 1;
                rotation
            } % 4;

            if (y >= (top - spacing)) && this.isDeadEnd()
            {
                dark_tower_wing::makeTimberFloor(this, world, random, sbb, rotation, y, spacing);

                let mut logDeco: Deco = Deco::dark_tower();
                logDeco.m_pillarID = blk("tf:log");
                logDeco.m_pillarMeta = 3;
                logDeco.m_platformID = blk("tf:log");
                logDeco.m_pillarMeta = 3;
                dark_tower_wing::decorateTreasureRoom(this, world, sbb, rotation, y, 4, logDeco);
            }
            else
            {
                dark_tower_wing::makeSmallTimberBeams(
                    this,
                    world,
                    random,
                    sbb,
                    rotation,
                    y,
                    (y == bottom) && (bottom != spacing),
                    y >= (top - spacing),
                );
            }
            y += spacing;
        }
    }
}

// ComponentTFDarkTowerWing.makeTimberFloor
pub(crate) fn makeTimberFloor(
    this: &Piece,
    world: &GenWorld,
    _random: &Random,
    sbb: StructureBox,
    rotation: i32,
    y: i32,
    _spacing: i32,
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
        for x in 1..(this.m_size.get() - 1)
        {
            if x < z
            {
                this.placeTurned(world, beamID, beamMetaBase + beamMetaNS, x, y, z, rotation, sbb);
            }
            else
            {
                this.placeTurned(world, beamID, beamMetaBase + beamMetaEW, x, y, z, rotation, sbb);
            }
        }
    }

    for by in 1..4
    {
        this.placeTurned(world, beamID, beamMetaBase + beamMetaUD, 2, y - by, 2, rotation, sbb);
        this.placeTurned(world, blk("ladder"), this.ladderMeta(2 + rotation), 3, y - by, 2, rotation, sbb);
        this.placeTurned(world, beamID, beamMetaBase + beamMetaUD, 6, y - by, 6, rotation, sbb);
        this.placeTurned(world, blk("ladder"), this.ladderMeta(4 + rotation), 5, y - by, 6, rotation, sbb);
    }

    this.placeTurned(world, blk("air"), 0, 3, y, 2, rotation, sbb);
    this.placeTurned(world, blk("air"), 0, 5, y, 6, rotation, sbb);
}

// ComponentTFDarkTowerWing.makeSmallTimberBeams
pub(crate) fn makeSmallTimberBeams(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    sbb: StructureBox,
    rotation: i32,
    y: i32,
    bottom: bool,
    _top: bool,
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
        this.placeTurned(world, beamID, beamMetaBase + beamMetaEW, 2, y, z, rotation, sbb);
        this.placeTurned(world, beamID, beamMetaBase + beamMetaEW, 6, y, z, rotation, sbb);
    }

    let z: i32 = dark_tower_wing::pickBetweenExcluding(this, 3, this.m_size.get() - 3, random, 2, 2, 6);

    for x in 3..6
    {
        this.placeTurned(world, beamID, beamMetaBase + beamMetaNS, x, y, z, rotation, sbb);
    }

    let x1: i32 = 2;
    let z1: i32 = if random.nextBoolean() { 2 } else { 6 };
    let x3: i32 = 6;
    let z3: i32 = if random.nextBoolean() { 2 } else { 6 };

    for by in 1..4
    {
        if (!bottom) || dark_tower_wing::checkPost(this, world, x1, y - 4, z1, rotation, sbb)
        {
            this.placeTurned(world, beamID, beamMetaBase + beamMetaUD, x1, y - by, z1, rotation, sbb);
            this.placeTurned(world, blk("ladder"), this.ladderMeta(2 + rotation), x1 + 1, y - by, z1, rotation, sbb);
        }

        if (!bottom) || dark_tower_wing::checkPost(this, world, x3, y - 4, z3, rotation, sbb)
        {
            this.placeTurned(world, beamID, beamMetaBase + beamMetaUD, x3, y - by, z3, rotation, sbb);
            this.placeTurned(world, blk("ladder"), this.ladderMeta(4 + rotation), x3 - 1, y - by, z3, rotation, sbb);
        }
    }
}

// ComponentTFDarkTowerWing.pickBetweenExcluding
pub(crate) fn pickBetweenExcluding(_this: &Piece, low: i32, high: i32, random: &Random, k: i32, l: i32, m: i32) -> i32
{
    let mut result: i32;

    loop
    {
        result = random.nextIntBounded(high - low) + low;
        if !(((result == k) || (result == l)) || (result == m))
        {
            break;
        }
    }

    result
}

// ComponentTFDarkTowerWing.pickFrom
pub(crate) fn pickFrom(_this: &Piece, random: &Random, i: i32, j: i32, k: i32) -> i32
{
    let var10000: i32 = match random.nextIntBounded(3)
    {
        1 => j,
        2 => k,
        _ => i,
    };
    var10000
}

// ComponentTFDarkTowerWing.checkPost
pub(crate) fn checkPost(
    this: &Piece,
    world: &GenWorld,
    x: i32,
    y: i32,
    z: i32,
    rotation: i32,
    sbb: StructureBox,
) -> bool
{
    let worldX: i32 = this.xTurned(x, z, rotation);
    let worldY: i32 = this.yAt(y);
    let worldZ: i32 = this.zTurned(x, z, rotation);
    let blockID: Blk = if sbb.contains(worldX, worldY, worldZ)
    {
        world.getBlock(worldX, worldY, worldZ)
    }
    else
    {
        blk("air")
    };
    (blockID != blk("air"))
        && ((blockID != this.m_deco.get().m_accentID)
            || (world.meta(worldX, worldY, worldZ) != this.m_deco.get().m_accentMeta))
}

// ComponentTFDarkTowerWing.makeEncasedWalls
pub(crate) fn makeEncasedWalls(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    sbb: StructureBox,
    minX: i32,
    minY: i32,
    minZ: i32,
    maxX: i32,
    maxY: i32,
    maxZ: i32,
)
{
    for x in minX..=maxX
    {
        for y in minY..=maxY
        {
            for z in minZ..=maxZ
            {
                if (((((x == minX) || (x == maxX)) || (y == minY)) || (y == maxY)) || (z == minZ)) || (z == maxZ)
                {
                    if ((((x != minY) && (x != maxX))
                        || ((((y != minY) && (y != maxY)) && (z != minZ)) && (z != maxZ)))
                        && (((y != minY) && (y != maxY))
                            || ((((x != minY) && (x != maxX)) && (z != minZ)) && (z != maxZ))))
                        && (((z != minZ) && (z != maxZ))
                            || ((((x != minY) && (x != maxX)) && (y != minY)) && (y != maxY)))
                    {
                        let blocker: Stones = this.m_deco.get().m_randomBlocks;
                        blocker.select(random, x, y, z, true);
                        this.placeBlock(world, blocker.block(), blocker.meta(), x, y, z, sbb);
                    }
                    else
                    {
                        this.placeBlock(
                            world,
                            this.m_deco.get().m_accentID,
                            this.m_deco.get().m_accentMeta,
                            x,
                            y,
                            z,
                            sbb,
                        );
                    }
                }
            }
        }
    }

    this.placeBlock(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, minX + 1, minY + 1, minZ, sbb);
    this.placeBlock(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, minX + 1, minY + 1, maxZ, sbb);
    this.placeBlock(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, maxX - 1, minY + 1, minZ, sbb);
    this.placeBlock(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, maxX - 1, minY + 1, maxZ, sbb);
    this.placeBlock(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, minX + 1, maxY - 1, minZ, sbb);
    this.placeBlock(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, minX + 1, maxY - 1, maxZ, sbb);
    this.placeBlock(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, maxX - 1, maxY - 1, minZ, sbb);
    this.placeBlock(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, maxX - 1, maxY - 1, maxZ, sbb);
    this.placeBlock(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, minX, minY + 1, minZ + 1, sbb);
    this.placeBlock(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, minX, minY + 1, maxZ - 1, sbb);
    this.placeBlock(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, maxX, minY + 1, minZ + 1, sbb);
    this.placeBlock(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, maxX, minY + 1, maxZ - 1, sbb);
    this.placeBlock(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, minX, maxY - 1, minZ + 1, sbb);
    this.placeBlock(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, minX, maxY - 1, maxZ - 1, sbb);
    this.placeBlock(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, maxX, maxY - 1, minZ + 1, sbb);
    this.placeBlock(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, maxX, maxY - 1, maxZ - 1, sbb);
    this.placeBlock(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, minX + 1, minY, minZ + 1, sbb);
    this.placeBlock(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, minX + 1, minY, maxZ - 1, sbb);
    this.placeBlock(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, maxX - 1, minY, minZ + 1, sbb);
    this.placeBlock(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, maxX - 1, minY, maxZ - 1, sbb);
    this.placeBlock(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, minX + 1, maxY, minZ + 1, sbb);
    this.placeBlock(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, minX + 1, maxY, maxZ - 1, sbb);
    this.placeBlock(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, maxX - 1, maxY, minZ + 1, sbb);
    this.placeBlock(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, maxX - 1, maxY, maxZ - 1, sbb);
}

// ComponentTFDarkTowerWing.getValidOpening
pub(crate) fn getValidOpening(this: &Piece, _random: &Random, direction: i32) -> Vec<i32>
{
    let verticalOffset: i32 = if this.m_size.get() == 19 { 5 } else { 4 };

    if (direction != 0) && (direction != 2)
    {
        if (direction != 1) && (direction != 3)
        {
            return vec![0, 0, 0];
        }

        let rx: i32 = this.m_size.get() / 2;

        let rz: i32 = if direction == 1
        {
            this.m_size.get() - 1
        }
        else
        {
            0
        };
        let ry: i32 = this.m_height.get() - verticalOffset;
        vec![rx, ry, rz]
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
        let rz: i32 = this.m_size.get() / 2;
        let ry: i32 = this.m_height.get() - verticalOffset;
        vec![rx, ry, rz]
    }
}

// ComponentTFDarkTowerWing.addOpening
pub(crate) fn addOpening(this: &Piece, dx: i32, dy: i32, dz: i32, direction: i32)
{
    dark_tower_wing::addOpening5(this, dx, dy, dz, direction, Door::Vanishing);
}

// ComponentTFDarkTowerWing.addOpening
pub(crate) fn addOpening5(this: &Piece, dx: i32, dy: i32, dz: i32, direction: i32, r#type: Door)
{
    tower_wing::addOpening(this, dx, dy, dz, direction);
    this.m_openingTypes.borrow_mut().insert(
        (this.m_openings.borrow().iter().position(|item| *item == [dx, dy, dz]).map_or(-1, |index| index as i32))
            as usize,
        r#type,
    );
}

// ComponentTFDarkTowerWing.makeOpenings
pub(crate) fn makeOpenings(this: &Piece, world: &GenWorld, sbb: StructureBox)
{
    for i in 0..(this.m_openings.borrow().len() as i32)
    {
        let doorCoords: [i32; 3] = this.m_openings.borrow()[(i) as usize];

        let doorType: Door = if (this.m_openingTypes.borrow().len() as i32) > i
        {
            this.m_openingTypes.borrow()[(i) as usize]
        }
        else
        {
            Door::Vanishing
        };

        match doorType
        {
            Door::Reappearing =>
            {
                dark_tower_wing::makeReappearingDoorOpening(
                    this,
                    world,
                    doorCoords[0],
                    doorCoords[1],
                    doorCoords[2],
                    sbb,
                );
            }

            Door::Locked =>
            {
                dark_tower_wing::makeLockedDoorOpening(this, world, doorCoords[0], doorCoords[1], doorCoords[2], sbb);
            }

            _ =>
            {
                this.makeDoorOpening(world, doorCoords[0], doorCoords[1], doorCoords[2], sbb);
            }
        }
    }
}

// ComponentTFDarkTowerWing.makeDoorOpening
pub(crate) fn makeDoorOpening(this: &Piece, world: &GenWorld, dx: i32, dy: i32, dz: i32, sbb: StructureBox)
{
    this.darkenAt(world, dx - 3, dy - 1, dz - 3, dx + 3, dy + 3, dz + 3);

    if (dx == 0) || (dx == (this.m_size.get() - 1))
    {
        this.fillMeta(
            world,
            sbb,
            dx,
            dy - 1,
            dz - 2,
            dx,
            dy + 3,
            dz + 2,
            this.m_deco.get().m_accentID,
            this.m_deco.get().m_accentMeta,
            blk("air"),
            0,
            false,
        );
        this.fillMeta(world, sbb, dx, dy, dz - 1, dx, dy + 2, dz + 1, blk("tf:towerDevice"), 2, blk("air"), 0, false);
    }

    if (dz == 0) || (dz == (this.m_size.get() - 1))
    {
        this.fillMeta(
            world,
            sbb,
            dx - 2,
            dy - 1,
            dz,
            dx + 2,
            dy + 3,
            dz,
            this.m_deco.get().m_accentID,
            this.m_deco.get().m_accentMeta,
            blk("air"),
            0,
            false,
        );
        this.fillMeta(world, sbb, dx - 1, dy, dz, dx + 1, dy + 2, dz, blk("tf:towerDevice"), 2, blk("air"), 0, false);
    }
}

// ComponentTFDarkTowerWing.makeReappearingDoorOpening
pub(crate) fn makeReappearingDoorOpening(this: &Piece, world: &GenWorld, dx: i32, dy: i32, dz: i32, sbb: StructureBox)
{
    this.darkenAt(world, dx - 3, dy - 1, dz - 3, dx + 3, dy + 3, dz + 3);

    if (dx == 0) || (dx == (this.m_size.get() - 1))
    {
        this.fillMeta(
            world,
            sbb,
            dx,
            dy - 1,
            dz - 2,
            dx,
            dy + 3,
            dz + 2,
            this.m_deco.get().m_accentID,
            this.m_deco.get().m_accentMeta,
            blk("air"),
            0,
            false,
        );
        this.fillMeta(world, sbb, dx, dy, dz - 1, dx, dy + 2, dz + 1, blk("tf:towerDevice"), 0, blk("air"), 0, false);
    }

    if (dz == 0) || (dz == (this.m_size.get() - 1))
    {
        this.fillMeta(
            world,
            sbb,
            dx - 2,
            dy - 1,
            dz,
            dx + 2,
            dy + 3,
            dz,
            this.m_deco.get().m_accentID,
            this.m_deco.get().m_accentMeta,
            blk("air"),
            0,
            false,
        );
        this.fillMeta(world, sbb, dx - 1, dy, dz, dx + 1, dy + 2, dz, blk("tf:towerDevice"), 0, blk("air"), 0, false);
    }
}

// ComponentTFDarkTowerWing.makeLockedDoorOpening
pub(crate) fn makeLockedDoorOpening(this: &Piece, world: &GenWorld, dx: i32, dy: i32, dz: i32, sbb: StructureBox)
{
    this.darkenAt(world, dx - 3, dy - 1, dz - 3, dx + 3, dy + 3, dz + 3);

    if (dx == 0) || (dx == (this.m_size.get() - 1))
    {
        this.fillMeta(
            world,
            sbb,
            dx,
            dy - 1,
            dz - 2,
            dx,
            dy + 3,
            dz + 2,
            this.m_deco.get().m_accentID,
            this.m_deco.get().m_accentMeta,
            blk("air"),
            0,
            false,
        );
        this.fillMeta(world, sbb, dx, dy, dz - 1, dx, dy + 2, dz + 1, blk("tf:towerDevice"), 2, blk("air"), 0, false);
        this.placeBlock(world, blk("tf:towerDevice"), 4, dx, dy + 0, dz + 1, sbb);
        this.placeBlock(world, blk("tf:towerDevice"), 4, dx, dy + 0, dz - 1, sbb);
        this.placeBlock(world, blk("tf:towerDevice"), 4, dx, dy + 2, dz + 1, sbb);
        this.placeBlock(world, blk("tf:towerDevice"), 4, dx, dy + 2, dz - 1, sbb);
    }

    if (dz == 0) || (dz == (this.m_size.get() - 1))
    {
        this.fillMeta(
            world,
            sbb,
            dx - 2,
            dy - 1,
            dz,
            dx + 2,
            dy + 3,
            dz,
            this.m_deco.get().m_accentID,
            this.m_deco.get().m_accentMeta,
            blk("air"),
            0,
            false,
        );
        this.fillMeta(world, sbb, dx - 1, dy, dz, dx + 1, dy + 2, dz, blk("tf:towerDevice"), 2, blk("air"), 0, false);
        this.placeBlock(world, blk("tf:towerDevice"), 4, dx + 1, dy + 0, dz, sbb);
        this.placeBlock(world, blk("tf:towerDevice"), 4, dx - 1, dy + 0, dz, sbb);
        this.placeBlock(world, blk("tf:towerDevice"), 4, dx + 1, dy + 2, dz, sbb);
        this.placeBlock(world, blk("tf:towerDevice"), 4, dx - 1, dy + 2, dz, sbb);
    }
}

// ComponentTFDarkTowerWing.isDeadEnd
pub(crate) fn isDeadEnd(this: &Piece) -> bool
{
    let mut nonBalconies: i32 = 0;

    for r#type in this.m_openingTypes.borrow().clone()
    {
        if r#type != Door::Reappearing
        {
            nonBalconies += 1;
        }
    }

    nonBalconies <= 1
}

// ComponentTFDarkTowerWing.isKeyTower
pub(crate) fn isKeyTower(this: &Piece) -> bool { this.m_keyTower.get() }

// ComponentTFDarkTowerWing.setKeyTower
pub(crate) fn setKeyTower(this: &Piece, keyTower: bool) { this.m_keyTower.set(keyTower); }

// ComponentTFDarkTowerWing.getLeverMeta
pub(crate) fn leverMeta(this: &Piece, mut rotation: i32, direction: i32) -> i32
{
    if direction == 0
    {
        return 0;
    }
    if direction == 1
    {
        return 5;
    }

    rotation += this.mode();
    rotation %= 4;
    if rotation == 0
    {
        match direction
        {
            2 =>
            {
                return 4;
            }

            3 =>
            {
                return 3;
            }

            4 =>
            {
                return 2;
            }

            5 =>
            {
                return 1;
            }

            _ =>
            {}
        }
    }
    else if rotation == 1
    {
        match direction
        {
            2 =>
            {
                return 1;
            }

            3 =>
            {
                return 2;
            }

            4 =>
            {
                return 4;
            }

            5 =>
            {
                return 3;
            }

            _ =>
            {}
        }
    }
    else if rotation == 2
    {
        match direction
        {
            2 =>
            {
                return 3;
            }

            3 =>
            {
                return 4;
            }

            4 =>
            {
                return 1;
            }

            5 =>
            {
                return 2;
            }

            _ =>
            {}
        }
    }
    else if rotation == 3
    {
        match direction
        {
            2 =>
            {
                return 2;
            }

            3 =>
            {
                return 1;
            }

            4 =>
            {
                return 3;
            }

            5 =>
            {
                return 4;
            }

            _ =>
            {}
        }
    }
    -1
}
