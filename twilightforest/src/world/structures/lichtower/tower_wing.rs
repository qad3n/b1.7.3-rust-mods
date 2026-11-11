// draft from ComponentTFTowerWing.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new7(i: i32, x: i32, y: i32, z: i32, pSize: i32, pHeight: i32, direction: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::TowerWing);
    init7(&this, i, x, y, z, pSize, pHeight, direction);
    Rc::new(this)
}

// ComponentTFTowerWing constructor
pub fn init(this: &Piece, i: i32)
{
    this.m_depth.set(i);
    *this.m_openings.borrow_mut() = Vec::new();
    *this.m_openingTowards.borrow_mut() = vec![false, false, true, false];
    this.m_highestOpening.set(0);
}

// ComponentTFTowerWing constructor
pub fn init7(this: &Piece, i: i32, x: i32, y: i32, z: i32, pSize: i32, pHeight: i32, direction: i32)
{
    this.m_depth.set(i);
    *this.m_openings.borrow_mut() = Vec::new();
    *this.m_openingTowards.borrow_mut() = vec![false, false, true, false];
    this.m_size.set(pSize);
    this.m_height.set(pHeight);
    this.setMode(direction);
    this.m_highestOpening.set(0);
    this.m_box.set(boxFor(
        x,
        y,
        z,
        0,
        0,
        0,
        this.m_size.get() - 1,
        this.m_height.get() - 1,
        this.m_size.get() - 1,
        direction,
    ));
}

// ComponentTFTowerWing.func_74861_a
pub fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    this.addOpening(0, 1, this.m_size.get() / 2, 2);
    this.makeARoof(&parent, list, random);
    this.makeABeard(&parent, list, random);
    if this.m_size.get() > 4
    {
        for i in 0..4
        {
            if i != 2
            {
                let dest: Vec<i32> = this.getValidOpening(random, i);
                if ((!this.makeTowerWing(
                    list,
                    random,
                    1,
                    dest[0],
                    dest[1],
                    dest[2],
                    this.m_size.get() - 2,
                    this.m_height.get() - 4,
                    i,
                )) && (this.m_size.get() > 8))
                    && (!this.makeTowerWing(
                        list,
                        random,
                        1,
                        dest[0],
                        dest[1],
                        dest[2],
                        this.m_size.get() - 4,
                        this.m_height.get() - 6,
                        i,
                    ))
                {
                    this.makeTowerWing(
                        list,
                        random,
                        1,
                        dest[0],
                        dest[1],
                        dest[2],
                        this.m_size.get() - 6,
                        this.m_height.get() - 12,
                        i,
                    );
                }
            }
        }
    }
}

// ComponentTFTowerWing.makeTowerWing
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
    if wingHeight < 6
    {
        return false;
    }
    else
    {
        let direction: i32 = (this.mode() + rotation) % 4;
        let dx: Vec<i32> = this.offsetTower(x, y, z, wingSize, direction);
        if random.nextIntBounded(6) == 0
        {
            return this.makeBridge(list, random, index, x, y, z, wingSize, wingHeight, rotation);
        }
        else
        {
            let wing: Rc<Piece> = tower_wing::new7(index, dx[0], dx[1], dx[2], wingSize, wingHeight, direction);
            let intersect: Option<Rc<Piece>> = intersecting(list, wing.m_box.get());
            if (intersect.is_none()) || (same(&intersect, &this))
            {
                list.push(Rc::clone(&wing));
                wing.build(this, list, random);
                this.addOpening(x, y, z, rotation);
                return true;
            }
            else
            {
                return if random.nextIntBounded(3) > 0
                {
                    this.makeBridge(list, random, index, x, y, z, wingSize, wingHeight, rotation)
                }
                else
                {
                    false
                };
            }
        }
    }
}

// ComponentTFTowerWing.makeBridge
pub fn makeBridge(
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
) -> bool
{
    let direction: i32 = (this.mode() + rotation) % 4;
    let dx: Vec<i32> = this.offsetTower(x, y, z, 3, direction);
    if (wingSize == 3) && (wingHeight > 10)
    {
        wingHeight = 6 + random.nextIntBounded(5);
    }
    let bridge: Rc<Piece> = tower_bridge::new(index, dx[0], dx[1], dx[2], wingSize, wingHeight, direction);
    let mut intersect: Option<Rc<Piece>> = intersecting(list, bridge.m_box.get());
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

// ComponentTFTowerWing.addOpening
pub fn addOpening(this: &Piece, dx: i32, dy: i32, dz: i32, direction: i32)
{
    this.m_openingTowards.borrow_mut()[(direction) as usize] = true;
    if dy > this.m_highestOpening.get()
    {
        this.m_highestOpening.set(dy);
    }
    this.m_openings.borrow_mut().push([dx, dy, dz]);
}

// ComponentTFTowerWing.makeABeard
pub fn makeABeard(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    let attached: bool = parent.bounds().m_min[1] < this.m_box.get().m_min[1];
    let index: i32 = this.depth();
    let beard: Rc<Piece>;
    if attached
    {
        beard = tower_beard_attached::new(index + 1, this);
    }
    else
    {
        beard = tower_beard::new(index + 1, this);
    }
    list.push(Rc::clone(&beard));
    beard.build(this, list, random);
}

// ComponentTFTowerWing.makeARoof
pub fn makeARoof(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    let attached: bool = parent.bounds().m_max[1] > this.m_box.get().m_max[1];
    if attached
    {
        this.makeAttachedRoof(list, random);
    }
    else
    {
        tower_wing::makeFreestandingRoof(this, list, random);
    }
}

// ComponentTFTowerWing.makeAttachedRoof
pub fn makeAttachedRoof(this: &Piece, list: &mut Pieces, random: &Random)
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

// ComponentTFTowerWing.tryToFitRoof
pub fn tryToFitRoof(this: &Piece, list: &mut Pieces, random: &Random, roof: &Rc<Piece>)
{
    if tower_roof::fits(roof, this, list, random)
    {
        list.push(Rc::clone(roof));
        roof.build(this, list, random);
        this.m_roofType.set(Some(roof.m_kind));
    }
}

// ComponentTFTowerWing.makeFreestandingRoof
pub fn makeFreestandingRoof(this: &Piece, list: &mut Pieces, random: &Random)
{
    let index: i32 = this.depth();
    if this.m_roofType.get().is_none() && (random.nextIntBounded(8) != 0)
    {
        let roof: Rc<Piece> = tower_roof_pointy_overhang::new(index + 1, this);
        tower_wing::tryToFitRoof(this, list, random, &roof);
    }
    if this.m_roofType.get().is_none()
    {
        let roof: Rc<Piece> = tower_roof_stairs_overhang::new(index + 1, this);
        tower_wing::tryToFitRoof(this, list, random, &roof);
    }
    if this.m_roofType.get().is_none()
    {
        let roof: Rc<Piece> = tower_roof_stairs::new(index + 1, this);
        tower_wing::tryToFitRoof(this, list, random, &roof);
    }
    if this.m_roofType.get().is_none() && (random.nextIntBounded(53) != 0)
    {
        let roof: Rc<Piece> = tower_roof_slab::new(index + 1, this);
        tower_wing::tryToFitRoof(this, list, random, &roof);
    }
    if this.m_roofType.get().is_none()
    {
        let roof: Rc<Piece> = tower_roof_fence::new(index + 1, this);
        tower_wing::tryToFitRoof(this, list, random, &roof);
    }
}

// ComponentTFTowerWing.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
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
        Stones::stronghold(),
    );
    this.fillAir(world, sbb, 1, 1, 1, this.m_size.get() - 2, this.m_height.get() - 2, this.m_size.get() - 2);
    this.darken(world);
    if this.m_highestOpening.get() > 1
    {
        tower_wing::makeStairs(this, world, random, sbb);
    }
    tower_wing::decorateThisTower(this, world, random, sbb);
    tower_wing::makeWindows(this, world, random, sbb, this.m_size.get() < 4);
    this.makeOpenings(world, sbb);
    return true;
}

// ComponentTFTowerWing.decorateThisTower
pub fn decorateThisTower(this: &Piece, world: &GenWorld, _random: &Random, sbb: StructureBox)
{
    let decoRNG: Random = Random::withSeed(
        world.seed() + ((((this.m_box.get().m_min[0] * 321534781) * this.m_box.get().m_min[2]) * 756839) as i64),
    );
    if this.m_size.get() > 3
    {
        if this.isDeadEnd()
        {
            tower_wing::decorateDeadEnd(this, world, &decoRNG, sbb);
        }
        else
        {
            tower_wing::decorateStairTower(this, world, &decoRNG, sbb);
        }
    }
}

// ComponentTFTowerWing.decorateDeadEnd
pub fn decorateDeadEnd(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox)
{
    let floors: i32 = (this.m_height.get() - 1) / 5;
    let floorHeight: i32 = this.m_height.get() / floors;
    for i in 1..floors
    {
        for x in 1..(this.m_size.get() - 1)
        {
            for z in 1..(this.m_size.get() - 1)
            {
                this.placeBlock(world, blk("planks"), 2, x, i * floorHeight, z, sbb);
            }
        }
    }
    if floors > 1
    {
        let mut ladderDir: i32 = 3;
        this.decorateFloor8(world, random, 0, 1, floorHeight, ladderDir, -1, sbb);
        for i in 1..(floors - 1)
        {
            let bottom: i32 = 1 + (floorHeight * i);
            let top: i32 = floorHeight * (i + 1);
            let downLadderDir: i32 = ladderDir;
            ladderDir += 1;
            ladderDir %= 4;
            this.decorateFloor8(world, random, i, bottom, top, ladderDir, downLadderDir, sbb);
        }
        this.decorateFloor8(
            world,
            random,
            floors,
            1 + (floorHeight * (floors - 1)),
            this.m_height.get() - 1,
            -1,
            ladderDir,
            sbb,
        );
    }
    else
    {
        this.decorateFloor8(world, random, 0, 1, this.m_height.get() - 1, -1, -1, sbb);
    }
}

// ComponentTFTowerWing.decorateFloor
pub fn decorateFloor8(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    _floor: i32,
    bottom: i32,
    top: i32,
    ladderUpDir: i32,
    ladderDownDir: i32,
    sbb: StructureBox,
)
{
    if ladderUpDir > (-1)
    {
        let meta: i32 = this.ladderMeta(ladderUpDir);
        let dx: i32 = tower_wing::getLadderX(this, ladderUpDir);
        let dz: i32 = tower_wing::getLadderZ(this, ladderUpDir);
        for dy in bottom..top
        {
            this.placeBlock(world, blk("ladder"), meta, dx, dy, dz, sbb);
        }
    }
    if ladderDownDir > (-1)
    {
        let meta: i32 = this.ladderMeta(ladderDownDir);
        let dx: i32 = tower_wing::getLadderX(this, ladderDownDir);
        let dz: i32 = tower_wing::getLadderZ(this, ladderDownDir);
        for dy in bottom - 1..(bottom + 2)
        {
            this.placeBlock(world, blk("ladder"), meta, dx, dy, dz, sbb);
        }
    }
    if (random.nextIntBounded(7) == 0) && (ladderDownDir == (-1))
    {
        tower_wing::decorateWell(this, world, random, bottom, top, ladderUpDir, ladderDownDir, sbb);
    }
    else if (random.nextIntBounded(7) == 0) && (ladderDownDir == (-1))
    {
        tower_wing::decorateSkeletonRoom(this, world, random, bottom, top, ladderUpDir, ladderDownDir, sbb);
    }
    else if (random.nextIntBounded(6) == 0) && (ladderDownDir == (-1))
    {
        tower_wing::decorateZombieRoom(this, world, random, bottom, top, ladderUpDir, ladderDownDir, sbb);
    }
    else if (random.nextIntBounded(5) == 0) && (ladderDownDir == (-1))
    {
        tower_wing::decorateCactusRoom(this, world, random, bottom, top, ladderUpDir, ladderDownDir, sbb);
    }
    else if (random.nextIntBounded(4) == 0) && (ladderDownDir > (-1))
    {
        tower_wing::decorateTreasureChest(this, world, random, bottom, top, ladderUpDir, ladderDownDir, sbb);
    }
    else if random.nextIntBounded(5) == 0
    {
        tower_wing::decorateSpiderWebs(this, world, random, bottom, top, ladderUpDir, ladderDownDir, sbb);
    }
    else if (random.nextIntBounded(12) == 0) && (ladderDownDir > (-1))
    {
        tower_wing::decorateSolidRock(this, world, random, bottom, top, ladderUpDir, ladderDownDir, sbb);
    }
    else if random.nextIntBounded(3) == 0
    {
        tower_wing::decorateFullLibrary(this, world, random, bottom, top, ladderUpDir, ladderDownDir, sbb);
    }
    else
    {
        tower_wing::decorateLibrary7(this, world, random, bottom, top, ladderUpDir, ladderDownDir, sbb);
    }
}

// ComponentTFTowerWing.decorateWell
pub fn decorateWell(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    bottom: i32,
    _top: i32,
    _ladderUpDir: i32,
    _ladderDownDir: i32,
    sbb: StructureBox,
)
{
    let cx: i32 = this.m_size.get() / 2;
    let cz: i32 = cx;
    let cy: i32 = bottom;
    let waterOrLava: Blk = if random.nextIntBounded(4) == 0
    {
        blk("lava")
    }
    else
    {
        blk("water")
    };
    if this.m_size.get() > 5
    {
        this.placeBlock(world, blk("stonebrick"), 0, cx - 1, cy + 0, cz - 1, sbb);
        this.placeBlock(world, blk("stone_slab"), 5, cx - 1, cy + 1, cz - 1, sbb);
        this.placeBlock(world, blk("stonebrick"), 0, cx + 0, cy + 0, cz - 1, sbb);
        this.placeBlock(world, blk("stonebrick"), 0, cx + 1, cy + 0, cz - 1, sbb);
        this.placeBlock(world, blk("stone_slab"), 5, cx + 1, cy + 1, cz - 1, sbb);
        this.placeBlock(world, blk("stonebrick"), 0, cx - 1, cy + 0, cz + 0, sbb);
        this.placeBlock(world, waterOrLava, 0, cx + 0, cy + 0, cz + 0, sbb);
        this.placeBlock(world, blk("stonebrick"), 0, cx + 1, cy + 0, cz + 0, sbb);
        this.placeBlock(world, blk("stonebrick"), 0, cx - 1, cy + 0, cz + 1, sbb);
        this.placeBlock(world, blk("stone_slab"), 5, cx - 1, cy + 1, cz + 1, sbb);
        this.placeBlock(world, blk("stonebrick"), 0, cx + 0, cy + 0, cz + 1, sbb);
        this.placeBlock(world, blk("stonebrick"), 0, cx + 1, cy + 0, cz + 1, sbb);
        this.placeBlock(world, blk("stone_slab"), 5, cx + 1, cy + 1, cz + 1, sbb);
    }
    this.placeBlock(world, waterOrLava, 0, cx + 0, cy - 1, cz + 0, sbb);
}

// ComponentTFTowerWing.decorateSkeletonRoom
pub fn decorateSkeletonRoom(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    bottom: i32,
    top: i32,
    ladderUpDir: i32,
    ladderDownDir: i32,
    sbb: StructureBox,
)
{
    this.placeSpawner(world, random, this.m_size.get() / 2, bottom + 2, this.m_size.get() / 2, "Skeleton", sbb);
    let mut chainList: Vec<[i32; 3]> = Vec::new();
    chainList.push([this.m_size.get() / 2, bottom + 2, this.m_size.get() / 2]);
    for _i in 0..(this.m_size.get() + 2)
    {
        let chain: [i32; 3] = [
            2 + random.nextIntBounded(this.m_size.get() - 4),
            this.m_height.get() - 2,
            2 + random.nextIntBounded(this.m_size.get() - 4),
        ];
        if !tower_wing::chainCollides(this, chain, &chainList)
        {
            for dy in bottom..top
            {
                this.placeBlock(world, blk("iron_bars"), 0, chain[0], dy, chain[2], sbb);
            }
            chainList.push(chain);
        }
    }
    for dx in 1..=(this.m_size.get() - 2)
    {
        for dz in 1..=(this.m_size.get() - 2)
        {
            if (((((dx == 1) || (dx == (this.m_size.get() - 2))) || (dz == 1)) || (dz == (this.m_size.get() - 2)))
                && (!tower_wing::isWindowPos(this, dx, dz)))
                && (!tower_wing::isLadderPos(this, dx, dz, ladderUpDir, ladderDownDir))
            {
                this.placeBlock(world, blk("web"), 0, dx, top - 1, dz, sbb);
            }
        }
    }
}

// ComponentTFTowerWing.decorateZombieRoom
pub fn decorateZombieRoom(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    bottom: i32,
    _top: i32,
    ladderUpDir: i32,
    ladderDownDir: i32,
    sbb: StructureBox,
)
{
    this.placeSpawner(world, random, this.m_size.get() / 2, bottom + 2, this.m_size.get() / 2, "Zombie", sbb);
    for dx in 1..=(this.m_size.get() - 2)
    {
        for dz in 1..=(this.m_size.get() - 2)
        {
            if ((!tower_wing::isWindowPos(this, dx, dz))
                && (!tower_wing::isLadderPos(this, dx, dz, ladderUpDir, ladderDownDir)))
                && (random.nextIntBounded(5) == 0)
            {
                this.placeBlock(world, blk("brown_mushroom"), 0, dx, bottom, dz, sbb);
            }
        }
    }
    let mut slabList: Vec<[i32; 3]> = Vec::new();
    slabList.push([this.m_size.get() / 2, bottom + 2, this.m_size.get() / 2]);
    for _i in 0..(this.m_size.get() - 1)
    {
        let slab: [i32; 3] = [
            2 + random.nextIntBounded(this.m_size.get() - 4),
            this.m_height.get() - 2,
            2 + random.nextIntBounded(this.m_size.get() - 4),
        ];
        if !tower_wing::chainCollides(this, slab, &slabList)
        {
            this.placeBlock(world, blk("iron_bars"), 0, slab[0], bottom + 0, slab[2], sbb);
            this.placeBlock(world, blk("wooden_slab"), 2, slab[0], bottom + 1, slab[2], sbb);
            this.placeBlock(world, blk("soul_sand"), 0, slab[0], bottom + 2, slab[2], sbb);
            slabList.push(slab);
        }
    }
}

// ComponentTFTowerWing.decorateCactusRoom
pub fn decorateCactusRoom(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    bottom: i32,
    top: i32,
    ladderUpDir: i32,
    ladderDownDir: i32,
    sbb: StructureBox,
)
{
    for dx in 1..=(this.m_size.get() - 2)
    {
        for dz in 1..=(this.m_size.get() - 2)
        {
            this.placeBlock(world, blk("sand"), 0, dx, bottom - 1, dz, sbb);
            if ((!tower_wing::isWindowPos(this, dx, dz))
                && (!tower_wing::isLadderPos(this, dx, dz, ladderUpDir, ladderDownDir)))
                && (random.nextIntBounded(4) == 0)
            {
                this.placeBlock(world, blk("deadbush"), 0, dx, bottom, dz, sbb);
            }
        }
    }
    let mut cactusList: Vec<[i32; 3]> = Vec::new();
    cactusList.push([this.m_size.get() / 2, bottom + 2, this.m_size.get() / 2]);
    for _i in 0..(this.m_size.get() + 12)
    {
        let cactus: [i32; 3] = [
            2 + random.nextIntBounded(this.m_size.get() - 4),
            this.m_height.get() - 2,
            2 + random.nextIntBounded(this.m_size.get() - 4),
        ];
        if !tower_wing::chainCollides(this, cactus, &cactusList)
        {
            for dy in bottom..top
            {
                this.placeBlock(world, blk("cactus"), 0, cactus[0], dy, cactus[2], sbb);
            }
            cactusList.push(cactus);
        }
    }
}

// ComponentTFTowerWing.decorateTreasureChest
pub fn decorateTreasureChest(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    bottom: i32,
    top: i32,
    _ladderUpDir: i32,
    _ladderDownDir: i32,
    sbb: StructureBox,
)
{
    let cx: i32 = this.m_size.get() / 2;
    let cz: i32 = cx;
    this.placeBlock(world, blk("stone_brick_stairs"), this.stairMeta(1), cx + 0, bottom, cz - 1, sbb);
    this.placeBlock(world, blk("stone_brick_stairs"), this.stairMeta(0), cx - 1, bottom, cz + 0, sbb);
    this.placeBlock(world, blk("stone_brick_stairs"), this.stairMeta(2), cx + 1, bottom, cz + 0, sbb);
    this.placeBlock(world, blk("stone_brick_stairs"), this.stairMeta(3), cx + 0, bottom, cz + 1, sbb);
    this.placeBlock(world, blk("stonebrick"), 0, cx + 0, bottom, cz + 0, sbb);
    if this.m_size.get() > 5
    {
        this.placeBlock(world, blk("stonebrick"), 0, cx - 1, bottom, cz - 1, sbb);
        this.placeBlock(world, blk("stonebrick"), 0, cx + 1, bottom, cz - 1, sbb);
        this.placeBlock(world, blk("stonebrick"), 0, cx - 1, bottom, cz + 1, sbb);
        this.placeBlock(world, blk("stonebrick"), 0, cx + 1, bottom, cz + 1, sbb);
    }
    this.placeBlock(world, blk("stone_brick_stairs"), this.stairMeta(1) + 4, cx + 0, top - 1, cz - 1, sbb);
    this.placeBlock(world, blk("stone_brick_stairs"), this.stairMeta(0) + 4, cx - 1, top - 1, cz + 0, sbb);
    this.placeBlock(world, blk("stone_brick_stairs"), this.stairMeta(2) + 4, cx + 1, top - 1, cz + 0, sbb);
    this.placeBlock(world, blk("stone_brick_stairs"), this.stairMeta(3) + 4, cx + 0, top - 1, cz + 1, sbb);
    this.placeBlock(world, blk("stonebrick"), 0, cx + 0, top - 1, cz + 0, sbb);
    if this.m_size.get() > 5
    {
        this.placeBlock(world, blk("stonebrick"), 0, cx - 1, top - 1, cz - 1, sbb);
        this.placeBlock(world, blk("stonebrick"), 0, cx + 1, top - 1, cz - 1, sbb);
        this.placeBlock(world, blk("stonebrick"), 0, cx - 1, top - 1, cz + 1, sbb);
        this.placeBlock(world, blk("stonebrick"), 0, cx + 1, top - 1, cz + 1, sbb);
    }
    if this.m_size.get() > 5
    {
        for cy in bottom + 1..(top - 1)
        {
            this.placeBlock(world, blk("stonebrick"), 5, cx - 1, cy, cz - 1, sbb);
            this.placeBlock(world, blk("stonebrick"), 5, cx + 1, cy, cz - 1, sbb);
            this.placeBlock(world, blk("stonebrick"), 5, cx - 1, cy, cz + 1, sbb);
            this.placeBlock(world, blk("stonebrick"), 5, cx + 1, cy, cz + 1, sbb);
        }
    }
    this.placeTreasure(world, random, cx + 0, bottom + 1, cz + 0, Table::TowerRoom, sbb);
    let mut i: i32 = 0;
    while i < 4
    {
        i += 1;
    }
}

// ComponentTFTowerWing.decorateSpiderWebs
pub fn decorateSpiderWebs(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    bottom: i32,
    top: i32,
    ladderUpDir: i32,
    ladderDownDir: i32,
    sbb: StructureBox,
)
{
    for dy in bottom..top
    {
        let chance: i32 = (top - dy) + 2;
        for dx in 1..=(this.m_size.get() - 2)
        {
            for dz in 1..=(this.m_size.get() - 2)
            {
                if (!tower_wing::isLadderPos(this, dx, dz, ladderUpDir, ladderDownDir))
                    && (random.nextIntBounded(chance) == 0)
                {
                    this.placeBlock(world, blk("web"), 0, dx, dy, dz, sbb);
                }
            }
        }
    }
    if random.nextIntBounded(5) == 0
    {
        let var10000: &'static str;
        match random.nextIntBounded(4)
        {
            1 =>
            {
                var10000 = mob("Hedge Spider");
            }
            2 =>
            {
                var10000 = mob("Swarm Spider");
            }
            3 =>
            {
                var10000 = "CaveSpider";
            }
            _ =>
            {
                var10000 = "Spider";
            }
        }
        let spiderName: &'static str = var10000;
        this.placeSpawner(world, random, this.m_size.get() / 2, bottom + 2, this.m_size.get() / 2, spiderName, sbb);
    }
    else
    {
        tower_wing::decorateFurniture(this, world, random, bottom, this.m_size.get() - 2, sbb);
    }
}

// ComponentTFTowerWing.decorateFurniture
pub fn decorateFurniture(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    bottom: i32,
    freeSpace: i32,
    sbb: StructureBox,
)
{
    if random.nextIntBounded(3) > 0
    {
        this.placeBlock(world, blk("fence"), 0, this.m_size.get() / 2, bottom, this.m_size.get() / 2, sbb);
        this.placeBlock(
            world,
            blk("wooden_pressure_plate"),
            0,
            this.m_size.get() / 2,
            bottom + 1,
            this.m_size.get() / 2,
            sbb,
        );
    }
    if (random.nextIntBounded(3) == 0) && (freeSpace > 1)
    {
        this.placeBlock(
            world,
            blk("spruce_stairs"),
            this.stairMeta(0),
            (this.m_size.get() / 2) + 1,
            bottom,
            this.m_size.get() / 2,
            sbb,
        );
    }
    if (random.nextIntBounded(3) == 0) && (freeSpace > 1)
    {
        this.placeBlock(
            world,
            blk("spruce_stairs"),
            this.stairMeta(1),
            this.m_size.get() / 2,
            bottom,
            (this.m_size.get() / 2) + 1,
            sbb,
        );
    }
    if (random.nextIntBounded(3) == 0) && (freeSpace > 1)
    {
        this.placeBlock(
            world,
            blk("spruce_stairs"),
            this.stairMeta(2),
            (this.m_size.get() / 2) - 1,
            bottom,
            this.m_size.get() / 2,
            sbb,
        );
    }
    if (random.nextIntBounded(3) == 0) && (freeSpace > 1)
    {
        this.placeBlock(
            world,
            blk("spruce_stairs"),
            this.stairMeta(3),
            this.m_size.get() / 2,
            bottom,
            (this.m_size.get() / 2) - 1,
            sbb,
        );
    }
}

// ComponentTFTowerWing.decorateSolidRock
pub fn decorateSolidRock(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    bottom: i32,
    top: i32,
    ladderUpDir: i32,
    ladderDownDir: i32,
    sbb: StructureBox,
)
{
    for dy in bottom..top
    {
        for dx in 1..=(this.m_size.get() - 2)
        {
            for dz in 1..=(this.m_size.get() - 2)
            {
                if (!tower_wing::isLadderPos(this, dx, dz, ladderUpDir, ladderDownDir))
                    && (random.nextIntBounded(9) != 0)
                {
                    this.placeBlock(world, blk("stone"), 0, dx, dy, dz, sbb);
                }
            }
        }
    }
}

// ComponentTFTowerWing.decorateLibrary
pub fn decorateLibrary7(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    bottom: i32,
    top: i32,
    ladderUpDir: i32,
    ladderDownDir: i32,
    sbb: StructureBox,
)
{
    for dx in 1..=(this.m_size.get() - 2)
    {
        for dz in 1..=(this.m_size.get() - 2)
        {
            for dy in bottom..(top - 1)
            {
                if (((((dx == 1) || (dx == (this.m_size.get() - 2))) || (dz == 1)) || (dz == (this.m_size.get() - 2)))
                    && (!tower_wing::isWindowPos(this, dx, dz)))
                    && (!tower_wing::isLadderPos(this, dx, dz, ladderUpDir, ladderDownDir))
                {
                    this.placeBlock(world, blk("bookshelf"), 0, dx, dy, dz, sbb);
                }
            }
        }
    }
    if (random.nextIntBounded(2) == 0) && (this.m_size.get() > 5)
    {
        tower_wing::decorateLibraryTreasure(this, world, random, bottom, top, ladderUpDir, ladderDownDir, sbb);
    }
    if (random.nextIntBounded(2) == 0) && (this.m_size.get() > 5)
    {
        tower_wing::decorateFurniture(this, world, random, bottom, this.m_size.get() - 2, sbb);
    }
}

// ComponentTFTowerWing.decorateLibraryTreasure
pub fn decorateLibraryTreasure(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    _bottom: i32,
    top: i32,
    ladderUpDir: i32,
    ladderDownDir: i32,
    sbb: StructureBox,
)
{
    'case1:
    {
        match random.nextIntBounded(4)
        {
            // todo falls through to the next case
            0 =>
            {
                if !tower_wing::isLadderPos(this, 2, 1, ladderUpDir, ladderDownDir)
                {
                    this.placeTreasure(world, random, 2, top - 2, 1, Table::TowerLibrary, sbb);
                    break 'case1;
                }
            }
            // todo falls through to the next case
            1 =>
            {
                if !tower_wing::isLadderPos(this, this.m_size.get() - 2, 2, ladderUpDir, ladderDownDir)
                {
                    this.placeTreasure(world, random, this.m_size.get() - 2, top - 2, 2, Table::TowerLibrary, sbb);
                    break 'case1;
                }
            }
            // todo falls through to the next case
            2 =>
            {
                if !tower_wing::isLadderPos(
                    this,
                    this.m_size.get() - 3,
                    this.m_size.get() - 2,
                    ladderUpDir,
                    ladderDownDir,
                )
                {
                    this.placeTreasure(
                        world,
                        random,
                        this.m_size.get() - 3,
                        top - 2,
                        this.m_size.get() - 2,
                        Table::TowerLibrary,
                        sbb,
                    );
                    break 'case1;
                }
            }
            3 =>
            {
                if !tower_wing::isLadderPos(this, 1, this.m_size.get() - 3, ladderUpDir, ladderDownDir)
                {
                    this.placeTreasure(world, random, 1, top - 2, this.m_size.get() - 3, Table::TowerLibrary, sbb);
                }
            }
            _ =>
            {}
        }
    }
}

// ComponentTFTowerWing.decorateFullLibrary
pub fn decorateFullLibrary(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    bottom: i32,
    top: i32,
    ladderUpDir: i32,
    ladderDownDir: i32,
    sbb: StructureBox,
)
{
    for dx in 1..=(this.m_size.get() - 2)
    {
        for dz in 1..=(this.m_size.get() - 2)
        {
            for dy in bottom..top
            {
                if ((((((dx % 2) != 0)
                    && (((dz >= dx) && (dz <= ((this.m_size.get() - dx) - 1)))
                        || ((dz >= ((this.m_size.get() - dx) - 1)) && (dz <= dx))))
                    || (((dz % 2) != 0)
                        && (((dx >= dz) && (dx <= ((this.m_size.get() - dz) - 1)))
                            || ((dx >= ((this.m_size.get() - dz) - 1)) && (dx <= dz)))))
                    && (!tower_wing::isWindowPos3(this, dx, dy, dz)))
                    && (!tower_wing::isOpeningPos(this, dx, dy, dz)))
                    && (!tower_wing::isLadderPos(this, dx, dz, ladderUpDir, ladderDownDir))
                {
                    this.placeBlock(world, blk("bookshelf"), 0, dx, dy, dz, sbb);
                }
            }
        }
    }
    if (random.nextIntBounded(2) == 0) && (this.m_size.get() > 5)
    {
        tower_wing::decorateLibraryTreasure(this, world, random, bottom, top, ladderUpDir, ladderDownDir, sbb);
    }
}

// ComponentTFTowerWing.isWindowPos
pub fn isWindowPos(this: &Piece, x: i32, z: i32) -> bool
{
    if (x == 1) && (z == (this.m_size.get() / 2))
    {
        return true;
    }
    else if (x == (this.m_size.get() - 2)) && (z == (this.m_size.get() / 2))
    {
        return true;
    }
    else
    {
        return if (x == (this.m_size.get() / 2)) && (z == 1)
        {
            true
        }
        else
        {
            (x == (this.m_size.get() / 2)) && (z == (this.m_size.get() - 2))
        };
    }
}

// ComponentTFTowerWing.isWindowPos
pub fn isWindowPos3(this: &Piece, x: i32, y: i32, z: i32) -> bool
{
    let mut checkYDir: i32 = -1;
    if (x == 1) && (z == (this.m_size.get() / 2))
    {
        checkYDir = 2;
    }
    else if (x == (this.m_size.get() - 2)) && (z == (this.m_size.get() / 2))
    {
        checkYDir = 0;
    }
    else if (x == (this.m_size.get() / 2)) && (z == 1)
    {
        checkYDir = 3;
    }
    else if (x == (this.m_size.get() / 2)) && (z == (this.m_size.get() - 2))
    {
        checkYDir = 1;
    }
    return if checkYDir <= (-1)
    {
        false
    }
    else
    {
        (!this.m_openingTowards.borrow()[(checkYDir) as usize])
            && (((y == 2) || (y == 3))
                || ((this.m_height.get() > 8)
                    && ((y == (this.m_height.get() - 3)) || (y == (this.m_height.get() - 4)))))
    };
}

// ComponentTFTowerWing.isOpeningPos
pub fn isOpeningPos(this: &Piece, x: i32, y: i32, z: i32) -> bool
{
    for door in this.m_openings.borrow().clone()
    {
        let mut inside: [i32; 3] = door;
        if inside[0] == 0
        {
            inside[0] += 1;
        }
        else if inside[0] == (this.m_size.get() - 1)
        {
            inside[0] -= 1;
        }
        else if inside[2] == 0
        {
            inside[2] += 1;
        }
        else if inside[2] == (this.m_size.get() - 1)
        {
            inside[2] -= 1;
        }
        if ((inside[0] == x) && (inside[2] == z)) && ((inside[1] == y) || ((inside[1] + 1) == y))
        {
            return true;
        }
    }
    return false;
}

// ComponentTFTowerWing.isLadderPos
pub fn isLadderPos(this: &Piece, x: i32, z: i32, ladderUpDir: i32, ladderDownDir: i32) -> bool
{
    return if (x == tower_wing::getLadderX(this, ladderUpDir)) && (z == tower_wing::getLadderZ(this, ladderUpDir))
    {
        true
    }
    else
    {
        (x == tower_wing::getLadderX(this, ladderDownDir)) && (z == tower_wing::getLadderZ(this, ladderDownDir))
    };
}

// ComponentTFTowerWing.getLadderX
pub fn getLadderX(this: &Piece, ladderDir: i32) -> i32
{
    let var10000: i32;
    match ladderDir
    {
        0 =>
        {
            var10000 = this.m_size.get() - 2;
        }
        1 =>
        {
            var10000 = (this.m_size.get() / 2) + 1;
        }
        2 =>
        {
            var10000 = 1;
        }
        3 =>
        {
            var10000 = (this.m_size.get() / 2) - 1;
        }
        _ =>
        {
            var10000 = this.m_size.get() / 2;
        }
    }
    return var10000;
}

// ComponentTFTowerWing.getLadderZ
pub fn getLadderZ(this: &Piece, ladderDir: i32) -> i32
{
    let var10000: i32;
    match ladderDir
    {
        0 =>
        {
            var10000 = (this.m_size.get() / 2) - 1;
        }
        1 =>
        {
            var10000 = this.m_size.get() - 2;
        }
        2 =>
        {
            var10000 = (this.m_size.get() / 2) + 1;
        }
        3 =>
        {
            var10000 = 1;
        }
        _ =>
        {
            var10000 = this.m_size.get() / 2;
        }
    }
    return var10000;
}

// ComponentTFTowerWing.decorateStairTower
pub fn decorateStairTower(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox)
{
    if (this.m_height.get() - this.m_highestOpening.get()) > 8
    {
        let base: i32 = this.m_highestOpening.get() + 3;
        let floors: i32 = (this.m_height.get() - base) / 5;
        let floorHeight: i32 = (this.m_height.get() - base) / floors;
        for i in 0..floors
        {
            for x in 1..(this.m_size.get() - 1)
            {
                for z in 1..(this.m_size.get() - 1)
                {
                    this.placeBlock(world, blk("planks"), 2, x, (i * floorHeight) + base, z, sbb);
                }
            }
        }
        let mut ladderDir: i32 = 3;
        let meta: i32 = this.ladderMeta(ladderDir);
        let dx: i32 = tower_wing::getLadderX(this, ladderDir);
        let dz: i32 = tower_wing::getLadderZ(this, ladderDir);
        for dy in 1..3
        {
            this.placeBlock(world, blk("ladder"), meta, dx, base - dy, dz, sbb);
        }
        for i in 0..(floors - 1)
        {
            let bottom: i32 = (base + 1) + (floorHeight * i);
            let top: i32 = base + (floorHeight * (i + 1));
            let downLadderDir: i32 = ladderDir;
            ladderDir += 1;
            ladderDir %= 4;
            this.decorateFloor8(world, random, i, bottom, top, ladderDir, downLadderDir, sbb);
        }
        this.decorateFloor8(
            world,
            random,
            floors,
            (base + 1) + (floorHeight * (floors - 1)),
            this.m_height.get() - 1,
            -1,
            ladderDir,
            sbb,
        );
        if base > 8
        {
            match random.nextIntBounded(4)
            {
                0 =>
                {
                    tower_wing::decorateChandelier(this, world, random, base + 1, sbb);
                }
                1 =>
                {
                    tower_wing::decorateHangingChains(this, world, random, base + 1, sbb);
                }
                2 =>
                {
                    tower_wing::decorateFloatingBooks(this, world, random, base + 1, sbb);
                }
                3 =>
                {
                    tower_wing::decorateFloatingVines(this, world, random, base + 1, sbb);
                }
                _ =>
                {}
            }
        }
    }
    else if this.m_size.get() > 5
    {
        match random.nextIntBounded(4)
        {
            0 =>
            {
                tower_wing::decorateChandelier(this, world, random, this.m_height.get(), sbb);
            }
            1 =>
            {
                tower_wing::decorateHangingChains(this, world, random, this.m_height.get(), sbb);
            }
            2 =>
            {
                tower_wing::decorateFloatingBooks(this, world, random, this.m_height.get(), sbb);
            }
            3 =>
            {
                tower_wing::decorateFloatingVines(this, world, random, this.m_height.get(), sbb);
            }
            _ =>
            {}
        }
    }
    else if this.m_size.get() > 3
    {
        match random.nextIntBounded(3)
        {
            0 =>
            {
                tower_wing::decorateHangingChains(this, world, random, this.m_height.get(), sbb);
            }
            1 =>
            {
                tower_wing::decorateFloatingBooks(this, world, random, this.m_height.get(), sbb);
            }
            2 =>
            {
                tower_wing::decorateFloatingVines(this, world, random, this.m_height.get(), sbb);
            }
            _ =>
            {}
        }
    }
    tower_wing::decorateStairFloor(this, world, random, sbb);
}

// ComponentTFTowerWing.decorateStairFloor
pub fn decorateStairFloor(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox)
{
    if this.m_size.get() > 5
    {
        if random.nextIntBounded(3) == 0
        {
            tower_wing::decorateStairWell(this, world, random, sbb);
        }
        else if (random.nextIntBounded(3) > 0) || (this.m_size.get() >= 15)
        {
            tower_wing::decoratePlanter(this, world, random, sbb);
        }
    }
}

// ComponentTFTowerWing.decorateChandelier
pub fn decorateChandelier(this: &Piece, world: &GenWorld, random: &Random, decoTop: i32, sbb: StructureBox)
{
    if (decoTop >= 8) && (this.m_size.get() >= 8)
    {
        let cx: i32 = this.m_size.get() / 2;
        let cy: i32 = (decoTop - random.nextIntBounded(decoTop - 7)) - 2;
        let cz: i32 = this.m_size.get() / 2;
        this.placeBlock(world, blk("fence"), 0, cx + 0, cy + 0, cz + 0, sbb);
        this.placeBlock(world, blk("fence"), 0, cx - 1, cy + 0, cz + 0, sbb);
        this.placeBlock(world, blk("fence"), 0, cx + 1, cy + 0, cz + 0, sbb);
        this.placeBlock(world, blk("fence"), 0, cx + 0, cy + 0, cz - 1, sbb);
        this.placeBlock(world, blk("fence"), 0, cx + 0, cy + 0, cz + 1, sbb);
        this.placeBlock(world, blk("fence"), 0, cx + 0, cy + 1, cz + 0, sbb);
        this.placeBlock(world, blk("torch"), 0, cx - 1, cy + 1, cz + 0, sbb);
        this.placeBlock(world, blk("torch"), 0, cx + 1, cy + 1, cz + 0, sbb);
        this.placeBlock(world, blk("torch"), 0, cx + 0, cy + 1, cz - 1, sbb);
        this.placeBlock(world, blk("torch"), 0, cx + 0, cy + 1, cz + 1, sbb);
        for y in cy..(decoTop - 1)
        {
            this.placeBlock(world, blk("fence"), 0, cx + 0, y, cz + 0, sbb);
        }
    }
}

// ComponentTFTowerWing.decorateHangingChains
pub fn decorateHangingChains(this: &Piece, world: &GenWorld, random: &Random, decoTop: i32, sbb: StructureBox)
{
    let mut chainList: Vec<[i32; 3]> = Vec::new();
    for _i in 0..(this.m_size.get() + 2)
    {
        let filled: i32 = if this.m_size.get() < 15 { 2 } else { 4 };
        let chain: [i32; 3] = [
            filled + random.nextIntBounded(this.m_size.get() - (filled * 2)),
            decoTop - 2,
            filled + random.nextIntBounded(this.m_size.get() - (filled * 2)),
        ];
        if !tower_wing::chainCollides(this, chain, &chainList)
        {
            let length: i32 = 1 + random.nextIntBounded(decoTop - 7);
            tower_wing::decorateOneChain(this, world, random, chain[0], decoTop, length, chain[2], sbb);
            chainList.push(chain);
        }
    }
}

// ComponentTFTowerWing.chainCollides
pub fn chainCollides(_this: &Piece, coords: [i32; 3], list: &[[i32; 3]]) -> bool
{
    for existing in list
    {
        if (coords[2] == existing[2]) && ((coords[0] - existing[0]).abs() <= 1)
        {
            return true;
        }
        if (coords[0] == existing[0]) && ((coords[2] - existing[2]).abs() <= 1)
        {
            return true;
        }
    }
    return false;
}

// ComponentTFTowerWing.decorateOneChain
pub fn decorateOneChain(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    dx: i32,
    decoTop: i32,
    length: i32,
    dz: i32,
    sbb: StructureBox,
)
{
    for y in 1..=length
    {
        this.placeBlock(world, blk("iron_bars"), 0, dx, (decoTop - y) - 1, dz, sbb);
    }
    let ballMeta: i32;
    let ballBlock: Blk;
    match random.nextIntBounded(10)
    {
        0 =>
        {
            ballBlock = blk("iron_block");
            ballMeta = 0;
        }
        1 =>
        {
            ballBlock = blk("bookshelf");
            ballMeta = 0;
        }
        2 =>
        {
            ballBlock = blk("netherrack");
            ballMeta = 0;
        }
        3 =>
        {
            ballBlock = blk("soul_sand");
            ballMeta = 0;
        }
        4 =>
        {
            ballBlock = blk("glass");
            ballMeta = 0;
        }
        5 =>
        {
            ballBlock = blk("lapis_block");
            ballMeta = 0;
        }
        6 =>
        {
            ballBlock = blk("monster_egg");
            ballMeta = 2;
        }
        _ =>
        {
            ballBlock = blk("glowstone");
            ballMeta = 0;
        }
    }
    this.placeBlock(world, ballBlock, ballMeta, dx, (decoTop - length) - 2, dz, sbb);
}

// ComponentTFTowerWing.decorateFloatingBooks
pub fn decorateFloatingBooks(this: &Piece, world: &GenWorld, random: &Random, decoTop: i32, sbb: StructureBox)
{
    let mut shelfList: Vec<[i32; 3]> = Vec::new();
    for _i in 0..(this.m_size.get() + 2)
    {
        let filled: i32 = if this.m_size.get() < 15 { 2 } else { 4 };
        let shelf: [i32; 3] = [
            filled + random.nextIntBounded(this.m_size.get() - (filled * 2)),
            decoTop - 2,
            filled + random.nextIntBounded(this.m_size.get() - (filled * 2)),
        ];
        if !tower_wing::chainCollides(this, shelf, &shelfList)
        {
            let bottom: i32 = 2 + random.nextIntBounded(decoTop - 7);
            let top: i32 = random.nextIntBounded(bottom - 1) + 2;
            for y in top..=bottom
            {
                this.placeBlock(world, blk("bookshelf"), 0, shelf[0], decoTop - y, shelf[2], sbb);
            }
            shelfList.push(shelf);
        }
    }
}

// ComponentTFTowerWing.decorateFloatingVines
pub fn decorateFloatingVines(this: &Piece, world: &GenWorld, random: &Random, decoTop: i32, sbb: StructureBox)
{
    let mut mossList: Vec<[i32; 3]> = Vec::new();
    for _i in 0..(this.m_size.get() + 2)
    {
        let filled: i32 = if this.m_size.get() < 15 { 2 } else { 4 };
        let moss: [i32; 3] = [
            filled + random.nextIntBounded(this.m_size.get() - (filled * 2)),
            decoTop - 2,
            filled + random.nextIntBounded(this.m_size.get() - (filled * 2)),
        ];
        if !tower_wing::chainCollides(this, moss, &mossList)
        {
            let bottom: i32 = 2 + random.nextIntBounded(decoTop - 7);
            let top: i32 = random.nextIntBounded(bottom - 1) + 2;
            for y in top..=bottom
            {
                this.placeBlock(world, blk("mossy_cobblestone"), 0, moss[0], decoTop - y, moss[2], sbb);
                this.placeBlock(
                    world,
                    blk("vine"),
                    tower_wing::getVineMeta(this, 2),
                    moss[0] + 1,
                    decoTop - y,
                    moss[2] + 0,
                    sbb,
                );
                this.placeBlock(
                    world,
                    blk("vine"),
                    tower_wing::getVineMeta(this, 0),
                    moss[0] - 1,
                    decoTop - y,
                    moss[2] + 0,
                    sbb,
                );
                this.placeBlock(
                    world,
                    blk("vine"),
                    tower_wing::getVineMeta(this, 3),
                    moss[0] + 0,
                    decoTop - y,
                    moss[2] + 1,
                    sbb,
                );
                this.placeBlock(
                    world,
                    blk("vine"),
                    tower_wing::getVineMeta(this, 1),
                    moss[0] + 0,
                    decoTop - y,
                    moss[2] - 1,
                    sbb,
                );
            }
            mossList.push(moss);
        }
    }
    for y in this.m_highestOpening.get() + 3..(decoTop - 1)
    {
        for x in 1..(this.m_size.get() - 1)
        {
            if random.nextIntBounded(3) == 0
            {
                this.placeBlock(world, blk("vine"), tower_wing::getVineMeta(this, 3), x, y, 1, sbb);
            }
            if random.nextIntBounded(3) == 0
            {
                this.placeBlock(world, blk("vine"), tower_wing::getVineMeta(this, 1), x, y, this.m_size.get() - 2, sbb);
            }
        }
        for z in 1..(this.m_size.get() - 1)
        {
            if random.nextIntBounded(3) == 0
            {
                this.placeBlock(world, blk("vine"), tower_wing::getVineMeta(this, 2), 1, y, z, sbb);
            }
            if random.nextIntBounded(3) == 0
            {
                this.placeBlock(world, blk("vine"), tower_wing::getVineMeta(this, 0), this.m_size.get() - 2, y, z, sbb);
            }
        }
    }
}

// ComponentTFTowerWing.getVineMeta
pub fn getVineMeta(this: &Piece, vineDir: i32) -> i32
{
    let var10000: i32;
    match (this.mode() + vineDir) % 4
    {
        0 =>
        {
            var10000 = 8;
        }
        1 =>
        {
            var10000 = 1;
        }
        2 =>
        {
            var10000 = 2;
        }
        3 =>
        {
            var10000 = 4;
        }
        _ =>
        {
            var10000 = -1;
        }
    }
    return var10000;
}

// ComponentTFTowerWing.decoratePlanter
pub fn decoratePlanter(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox)
{
    let cx: i32 = this.m_size.get() / 2;
    let cz: i32 = cx;
    this.placeBlock(world, blk("stone_slab"), 0, cx + 0, 1, cz + 1, sbb);
    this.placeBlock(world, blk("stone_slab"), 0, cx + 0, 1, cz - 1, sbb);
    this.placeBlock(world, blk("stone_slab"), 0, cx + 1, 1, cz + 0, sbb);
    this.placeBlock(world, blk("stone_slab"), 0, cx - 1, 1, cz + 0, sbb);
    if this.m_size.get() > 7
    {
        this.placeBlock(world, blk("double_stone_slab"), 0, cx - 1, 1, cz - 1, sbb);
        this.placeBlock(world, blk("double_stone_slab"), 0, cx + 1, 1, cz - 1, sbb);
        this.placeBlock(world, blk("double_stone_slab"), 0, cx + 1, 1, cz + 1, sbb);
        this.placeBlock(world, blk("double_stone_slab"), 0, cx - 1, 1, cz + 1, sbb);
    }
    this.placeBlock(world, blk("grass"), 0, cx + 0, 1, cz + 0, sbb);
    let planterBlock: Blk;
    let planterMeta: i32;
    match random.nextIntBounded(6)
    {
        0 =>
        {
            planterBlock = blk("sapling");
            planterMeta = 0;
        }
        1 =>
        {
            planterBlock = blk("sapling");
            planterMeta = 1;
        }
        2 =>
        {
            planterBlock = blk("sapling");
            planterMeta = 2;
        }
        3 =>
        {
            planterBlock = blk("sapling");
            planterMeta = 3;
        }
        4 =>
        {
            planterBlock = blk("brown_mushroom");
            planterMeta = 0;
        }
        _ =>
        {
            planterBlock = blk("red_mushroom");
            planterMeta = 0;
        }
    }
    this.placeBlock(world, planterBlock, planterMeta, cx + 0, 2, cz + 0, sbb);
    if planterBlock == blk("sapling")
    {
        let wx: i32 = this.xAt(cx, cz);
        let wy: i32 = this.yAt(2);
        let wz: i32 = this.zAt(cx, cz);
        Gen::Trees { minimum: 4, meta: planterMeta }.generate(world, world.rand(), wx, wy, wz);
    }
    if (planterBlock == blk("brown_mushroom")) || (planterBlock == blk("red_mushroom"))
    {
        let wx: i32 = this.xAt(cx, cz);
        let wy: i32 = this.yAt(2);
        let wz: i32 = this.zAt(cx, cz);
        if let Err(error) = world.m_world.randomTick(b173_api::BlockPos::new(wx, wy, wz))
        {
            b173_api::warn(&format!("tower planter mushroom: {}", error.0));
        }
    }
    let whatHappened: Blk = this.getBlock(world, cx + 0, 2, cz + 0, sbb);
    if (whatHappened == planterBlock) || (whatHappened == blk("air"))
    {
        let potMeta: i32 = 0;
        this.placeBlock(world, blk("flower_pot"), potMeta, cx + 0, 2, cz + 0, sbb);
    }
}

// ComponentTFTowerWing.decorateStairWell
pub fn decorateStairWell(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox)
{
    let cx: i32 = this.m_size.get() / 2;
    let cz: i32 = cx;
    let cy: i32 = 1;
    let waterOrLava: Blk = if random.nextIntBounded(4) == 0
    {
        blk("lava")
    }
    else
    {
        blk("water")
    };
    if this.m_size.get() > 7
    {
        this.placeBlock(world, blk("stonebrick"), 0, cx - 1, cy + 0, cz - 1, sbb);
        this.placeBlock(world, blk("stone_slab"), 5, cx - 1, cy + 1, cz - 1, sbb);
        this.placeBlock(world, blk("stonebrick"), 0, cx + 0, cy + 0, cz - 1, sbb);
        this.placeBlock(world, blk("stonebrick"), 0, cx + 1, cy + 0, cz - 1, sbb);
        this.placeBlock(world, blk("stone_slab"), 5, cx + 1, cy + 1, cz - 1, sbb);
        this.placeBlock(world, blk("stonebrick"), 0, cx - 1, cy + 0, cz + 0, sbb);
        this.placeBlock(world, waterOrLava, 0, cx + 0, cy + 0, cz + 0, sbb);
        this.placeBlock(world, blk("stonebrick"), 0, cx + 1, cy + 0, cz + 0, sbb);
        this.placeBlock(world, blk("stonebrick"), 0, cx - 1, cy + 0, cz + 1, sbb);
        this.placeBlock(world, blk("stone_slab"), 5, cx - 1, cy + 1, cz + 1, sbb);
        this.placeBlock(world, blk("stonebrick"), 0, cx + 0, cy + 0, cz + 1, sbb);
        this.placeBlock(world, blk("stonebrick"), 0, cx + 1, cy + 0, cz + 1, sbb);
        this.placeBlock(world, blk("stone_slab"), 5, cx + 1, cy + 1, cz + 1, sbb);
    }
    this.placeBlock(world, waterOrLava, 0, cx + 0, cy - 1, cz + 0, sbb);
}

// ComponentTFTowerWing.isDeadEnd
pub fn isDeadEnd(this: &Piece) -> bool { return (this.m_openings.borrow().len() as i32) == 1; }

// ComponentTFTowerWing.makeOpenings
pub fn makeOpenings(this: &Piece, world: &GenWorld, sbb: StructureBox)
{
    for door in this.m_openings.borrow().clone()
    {
        this.makeDoorOpening(world, door[0], door[1], door[2], sbb);
    }
}

// ComponentTFTowerWing.makeDoorOpening
pub fn makeDoorOpening(this: &Piece, world: &GenWorld, dx: i32, dy: i32, dz: i32, sbb: StructureBox)
{
    this.placeBlock(world, blk("air"), 0, dx, dy + 0, dz, sbb);
    this.placeBlock(world, blk("air"), 0, dx, dy + 1, dz, sbb);
    if this.getBlock(world, dx, dy + 2, dz, sbb) != blk("air")
    {
        this.placeBlock(world, blk("double_stone_slab"), 0, dx, dy + 2, dz, sbb);
    }
    if dx == 0
    {
        tower_wing::updateLight(this, world, dx - 1, dy + 0, dz);
        tower_wing::updateLight(this, world, dx - 1, dy + 1, dz);
    }
    if dx == (this.m_size.get() - 1)
    {
        tower_wing::updateLight(this, world, dx + 1, dy + 0, dz);
        tower_wing::updateLight(this, world, dx + 1, dy + 1, dz);
    }
    if dz == 0
    {
        tower_wing::updateLight(this, world, dx, dy + 0, dz - 1);
        tower_wing::updateLight(this, world, dx, dy + 1, dz - 1);
    }
    if dz == (this.m_size.get() - 1)
    {
        tower_wing::updateLight(this, world, dx, dy + 0, dz + 1);
        tower_wing::updateLight(this, world, dx, dy + 1, dz + 1);
    }
}

// ComponentTFTowerWing.updateLight
pub fn updateLight(_this: &Piece, _world: &GenWorld, _dx: i32, _dy: i32, _dz: i32) {}

// ComponentTFTowerWing.getValidOpening
pub fn getValidOpening(this: &Piece, random: &Random, direction: i32) -> Vec<i32>
{
    let mut wLength: i32 = this.m_size.get() - 2;
    let mut offset: i32 = 1;
    if this.m_size.get() == 15
    {
        wLength = 11;
        offset = 2;
    }
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

// ComponentTFTowerWing.getYByStairs
pub fn getYByStairs(this: &Piece, rx: i32, random: &Random, direction: i32) -> i32
{
    let mut rise: i32 = 1;
    let mut base: i32 = 0;
    if this.m_size.get() == 15
    {
        rise = 10;
        base = if (direction != 0) && (direction != 2)
        {
            28
        }
        else
        {
            23
        };
    }
    if this.m_size.get() == 9
    {
        rise = 6;
        base = if (direction != 0) && (direction != 2)
        {
            5
        }
        else
        {
            2
        };
    }
    if this.m_size.get() == 7
    {
        rise = 4;
        base = if (direction != 0) && (direction != 2)
        {
            4
        }
        else
        {
            2
        };
    }
    if this.m_size.get() == 5
    {
        rise = 4;
        let var10000: i32;
        match direction
        {
            0 =>
            {
                var10000 = 3;
            }
            1 =>
            {
                var10000 = 2;
            }
            2 =>
            {
                var10000 = 5;
            }
            3 =>
            {
                var10000 = 4;
            }
            _ =>
            {
                var10000 = base;
            }
        }
        base = var10000;
    }
    let flights: i32 = (((this.m_height.get() - 6) - base) / rise) + 1;
    if (base > 0) && (flights > 0)
    {
        let flightChosen: i32 = random.nextIntBounded(flights);
        let mut dy: i32 = (flightChosen * rise) + base;
        if this.m_size.get() == 15
        {
            dy -= if (direction != 0) && (direction != 3)
            {
                ((this.m_size.get() - rx) - 3) / 2
            }
            else
            {
                (rx - 2) / 2
            };
        }
        else
        {
            dy -= if (direction != 0) && (direction != 3)
            {
                ((this.m_size.get() - rx) - 2) / 2
            }
            else
            {
                (rx - 1) / 2
            };
        }
        if dy < 1
        {
            dy = 1;
        }
        return dy;
    }
    else
    {
        return 0;
    }
}

// ComponentTFTowerWing.makeWindows
pub fn makeWindows(this: &Piece, world: &GenWorld, _random: &Random, sbb: StructureBox, real: bool)
{
    for i in 0..4
    {
        let realWindows: bool = real && (!this.m_openingTowards.borrow()[(i) as usize]);
        tower_wing::makeWindowBlock(this, world, this.m_size.get() - 1, 2, this.m_size.get() / 2, i, sbb, realWindows);
        tower_wing::makeWindowBlock(this, world, this.m_size.get() - 1, 3, this.m_size.get() / 2, i, sbb, realWindows);
        tower_wing::makeWindowBase(this, world, this.m_size.get() - 1, 1, this.m_size.get() / 2, i, sbb);
        if this.m_height.get() > 8
        {
            tower_wing::makeWindowBlock(
                this,
                world,
                this.m_size.get() - 1,
                this.m_height.get() - 3,
                this.m_size.get() / 2,
                i,
                sbb,
                realWindows,
            );
            tower_wing::makeWindowBlock(
                this,
                world,
                this.m_size.get() - 1,
                this.m_height.get() - 4,
                this.m_size.get() / 2,
                i,
                sbb,
                realWindows,
            );
            tower_wing::makeWindowBase(
                this,
                world,
                this.m_size.get() - 1,
                this.m_height.get() - 5,
                this.m_size.get() / 2,
                i,
                sbb,
            );
        }
    }
}

// ComponentTFTowerWing.makeWindowBlock
pub fn makeWindowBlock(
    this: &Piece,
    world: &GenWorld,
    x: i32,
    y: i32,
    z: i32,
    rotation: i32,
    sbb: StructureBox,
    realWindows: bool,
)
{
    let temp: i32 = this.mode();
    this.setMode((this.mode() + rotation) % 4);
    let outside: Blk = this.getBlock(world, x + 1, y, z, sbb);
    let inside: Blk = this.getBlock(world, x - 1, y, z, sbb);
    if (realWindows && (inside == blk("air"))) && (outside == blk("air"))
    {
        this.placeBlock(world, blk("glass_pane"), 0, x, y, z, sbb);
    }
    else
    {
        this.placeBlock(world, blk("cobblestone"), 0, x, y, z, sbb);
    }
    this.setMode(temp);
}

// ComponentTFTowerWing.makeWindowBase
pub fn makeWindowBase(this: &Piece, world: &GenWorld, x: i32, y: i32, z: i32, rotation: i32, sbb: StructureBox)
{
    let temp: i32 = this.mode();
    this.setMode((this.mode() + rotation) % 4);
    this.placeBlock(world, blk("double_stone_slab"), 0, x, y, z, sbb);
    this.setMode(temp);
}

// ComponentTFTowerWing.makeStairs
pub fn makeStairs(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    if this.m_size.get() == 15
    {
        return tower_wing::makeStairs15(this, world, random, sbb);
    }
    else if this.m_size.get() == 9
    {
        return tower_wing::makeStairs9(this, world, random, sbb);
    }
    else if this.m_size.get() == 7
    {
        return tower_wing::makeStairs7(this, world, random, sbb);
    }
    else
    {
        return if this.m_size.get() == 5
        {
            tower_wing::makeStairs5(this, world, random, sbb)
        }
        else
        {
            false
        };
    }
}

// ComponentTFTowerWing.makeStairs5
pub fn makeStairs5(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    let rise: i32 = 1;
    let numFlights: i32 = this.m_highestOpening.get() / rise;
    for i in 0..numFlights
    {
        tower_wing::makeStairs5flight(this, world, random, sbb, i * rise, 0 + (i * 3), 2);
    }
    return true;
}

// ComponentTFTowerWing.makeStairs5flight
pub fn makeStairs5flight(
    this: &Piece,
    world: &GenWorld,
    _random: &Random,
    sbb: StructureBox,
    height: i32,
    rotation: i32,
    meta: i32,
)
{
    let temp: i32 = this.mode();
    this.setMode((this.mode() + rotation) % 4);
    let singleSlabBlock: Blk = if meta == 0
    {
        blk("stone_slab")
    }
    else
    {
        blk("wooden_slab")
    };
    let doubleSlabBlock: Blk = if meta == 0
    {
        blk("double_stone_slab")
    }
    else
    {
        blk("planks")
    };
    this.placeBlock(world, singleSlabBlock, meta, 2, 1 + height, 3, sbb);
    this.placeBlock(world, doubleSlabBlock, meta, 3, 1 + height, 3, sbb);
    this.setMode(temp);
}

// ComponentTFTowerWing.makeStairs7
pub fn makeStairs7(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    this.placeBlock(world, blk("wooden_slab"), 2, 1, 1, 4, sbb);
    this.placeBlock(world, blk("planks"), 2, 1, 1, 5, sbb);
    this.placeBlock(world, blk("stone_slab"), 0, 5, 1, 2, sbb);
    this.placeBlock(world, blk("double_stone_slab"), 0, 5, 1, 1, sbb);
    let rise: i32 = 2;
    let numFlights: i32 = this.m_highestOpening.get() / rise;
    for i in 0..numFlights
    {
        tower_wing::makeStairs7flight(this, world, random, sbb, 1 + (i * rise), 0 + (i * 3), 2);
        tower_wing::makeStairs7flight(this, world, random, sbb, 1 + (i * rise), 2 + (i * 3), 0);
    }
    return true;
}

// ComponentTFTowerWing.makeStairs7flight
pub fn makeStairs7flight(
    this: &Piece,
    world: &GenWorld,
    _random: &Random,
    sbb: StructureBox,
    height: i32,
    rotation: i32,
    meta: i32,
)
{
    let temp: i32 = this.mode();
    this.setMode((this.mode() + rotation) % 4);
    let singleSlabBlock: Blk = if meta == 0
    {
        blk("stone_slab")
    }
    else
    {
        blk("wooden_slab")
    };
    let doubleSlabBlock: Blk = if meta == 0
    {
        blk("double_stone_slab")
    }
    else
    {
        blk("planks")
    };
    this.placeBlock(world, singleSlabBlock, meta, 2, 1 + height, 5, sbb);
    this.placeBlock(world, doubleSlabBlock, meta, 3, 1 + height, 5, sbb);
    this.placeBlock(world, singleSlabBlock, meta, 4, 2 + height, 5, sbb);
    this.placeBlock(world, doubleSlabBlock, meta, 5, 2 + height, 5, sbb);
    this.setMode(temp);
}

// ComponentTFTowerWing.makeStairs9
pub fn makeStairs9(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    this.placeBlock(world, blk("wooden_slab"), 2, 1, 1, 6, sbb);
    this.placeBlock(world, blk("planks"), 2, 1, 1, 7, sbb);
    this.placeBlock(world, blk("stone_slab"), 0, 7, 1, 2, sbb);
    this.placeBlock(world, blk("double_stone_slab"), 0, 7, 1, 1, sbb);
    let rise: i32 = 3;
    let numFlights: i32 = this.m_highestOpening.get() / rise;
    for i in 0..numFlights
    {
        tower_wing::makeStairs9flight(this, world, random, sbb, 1 + (i * rise), 0 + (i * 3), 2);
        tower_wing::makeStairs9flight(this, world, random, sbb, 1 + (i * rise), 2 + (i * 3), 0);
    }
    return true;
}

// ComponentTFTowerWing.makeStairs9flight
pub fn makeStairs9flight(
    this: &Piece,
    world: &GenWorld,
    _random: &Random,
    sbb: StructureBox,
    height: i32,
    rotation: i32,
    meta: i32,
)
{
    let temp: i32 = this.mode();
    this.setMode((this.mode() + rotation) % 4);
    let singleSlabBlock: Blk = if meta == 0
    {
        blk("stone_slab")
    }
    else
    {
        blk("wooden_slab")
    };
    let doubleSlabBlock: Blk = if meta == 0
    {
        blk("double_stone_slab")
    }
    else
    {
        blk("planks")
    };
    this.placeBlock(world, singleSlabBlock, meta, 2, 1 + height, 7, sbb);
    this.placeBlock(world, doubleSlabBlock, meta, 3, 1 + height, 7, sbb);
    this.placeBlock(world, singleSlabBlock, meta, 4, 2 + height, 7, sbb);
    this.placeBlock(world, doubleSlabBlock, meta, 5, 2 + height, 7, sbb);
    this.placeBlock(world, singleSlabBlock, meta, 6, 3 + height, 7, sbb);
    this.placeBlock(world, doubleSlabBlock, meta, 7, 3 + height, 7, sbb);
    this.setMode(temp);
}

// ComponentTFTowerWing.makeStairs15
pub fn makeStairs15(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    this.placeBlock(world, blk("wooden_slab"), 2, 1, 1, 9, sbb);
    this.placeBlock(world, blk("wooden_slab"), 2, 2, 1, 9, sbb);
    this.placeBlock(world, blk("planks"), 2, 1, 1, 10, sbb);
    this.placeBlock(world, blk("planks"), 2, 2, 1, 10, sbb);
    this.placeBlock(world, blk("wooden_slab"), 2, 1, 2, 11, sbb);
    this.placeBlock(world, blk("wooden_slab"), 2, 2, 2, 11, sbb);
    this.placeBlock(world, blk("planks"), 2, 1, 2, 12, sbb);
    this.placeBlock(world, blk("planks"), 2, 2, 2, 12, sbb);
    this.placeBlock(world, blk("planks"), 2, 1, 2, 13, sbb);
    this.placeBlock(world, blk("planks"), 2, 2, 2, 13, sbb);
    this.placeBlock(world, blk("planks"), 2, 3, 2, 11, sbb);
    this.placeBlock(world, blk("fence"), 0, 3, 3, 11, sbb);
    this.placeBlock(world, blk("fence"), 0, 3, 4, 11, sbb);
    this.placeBlock(world, blk("planks"), 2, 3, 1, 10, sbb);
    this.placeBlock(world, blk("fence"), 0, 3, 2, 10, sbb);
    this.placeBlock(world, blk("fence"), 0, 3, 3, 10, sbb);
    this.placeBlock(world, blk("planks"), 2, 3, 1, 9, sbb);
    this.placeBlock(world, blk("fence"), 0, 3, 2, 9, sbb);
    this.placeBlock(world, blk("stone_slab"), 0, 13, 1, 5, sbb);
    this.placeBlock(world, blk("stone_slab"), 0, 12, 1, 5, sbb);
    this.placeBlock(world, blk("double_stone_slab"), 0, 13, 1, 4, sbb);
    this.placeBlock(world, blk("double_stone_slab"), 0, 12, 1, 4, sbb);
    this.placeBlock(world, blk("stone_slab"), 0, 13, 2, 3, sbb);
    this.placeBlock(world, blk("stone_slab"), 0, 12, 2, 3, sbb);
    this.placeBlock(world, blk("double_stone_slab"), 0, 13, 2, 2, sbb);
    this.placeBlock(world, blk("double_stone_slab"), 0, 12, 2, 2, sbb);
    this.placeBlock(world, blk("double_stone_slab"), 0, 13, 2, 1, sbb);
    this.placeBlock(world, blk("double_stone_slab"), 0, 12, 2, 1, sbb);
    this.placeBlock(world, blk("double_stone_slab"), 0, 11, 2, 3, sbb);
    this.placeBlock(world, blk("fence"), 0, 11, 3, 3, sbb);
    this.placeBlock(world, blk("fence"), 0, 11, 4, 3, sbb);
    this.placeBlock(world, blk("double_stone_slab"), 0, 11, 1, 4, sbb);
    this.placeBlock(world, blk("fence"), 0, 11, 2, 4, sbb);
    this.placeBlock(world, blk("fence"), 0, 11, 3, 4, sbb);
    this.placeBlock(world, blk("double_stone_slab"), 0, 11, 1, 5, sbb);
    this.placeBlock(world, blk("fence"), 0, 11, 2, 5, sbb);
    let rise: i32 = 5;
    let numFlights: i32 = this.m_highestOpening.get() / rise;
    for i in 0..numFlights
    {
        tower_wing::makeStairs15flight(this, world, random, sbb, 2 + (i * rise), 0 + (i * 3), 2);
        tower_wing::makeStairs15flight(this, world, random, sbb, 2 + (i * rise), 2 + (i * 3), 0);
    }
    return true;
}

// ComponentTFTowerWing.makeStairs15flight
pub fn makeStairs15flight(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    sbb: StructureBox,
    height: i32,
    rotation: i32,
    meta: i32,
)
{
    let temp: i32 = this.mode();
    this.setMode((this.mode() + rotation) % 4);
    let singleSlabBlock: Blk = if meta == 0
    {
        blk("stone_slab")
    }
    else
    {
        blk("wooden_slab")
    };
    let doubleSlabBlock: Blk = if meta == 0
    {
        blk("double_stone_slab")
    }
    else
    {
        blk("planks")
    };
    this.placeBlock(world, singleSlabBlock, meta, 3, 1 + height, 13, sbb);
    this.placeRandomly(world, sbb, random, 0.9, 4, 1 + height, 13, doubleSlabBlock, meta);
    this.placeBlock(world, singleSlabBlock, meta, 5, 2 + height, 13, sbb);
    this.placeBlock(world, doubleSlabBlock, meta, 6, 2 + height, 13, sbb);
    this.placeBlock(world, singleSlabBlock, meta, 7, 3 + height, 13, sbb);
    this.placeBlock(world, doubleSlabBlock, meta, 8, 3 + height, 13, sbb);
    this.placeBlock(world, singleSlabBlock, meta, 9, 4 + height, 13, sbb);
    this.placeRandomly(world, sbb, random, 0.9, 10, 4 + height, 13, doubleSlabBlock, meta);
    this.placeRandomly(world, sbb, random, 0.9, 11, 5 + height, 13, singleSlabBlock, meta);
    this.placeBlock(world, doubleSlabBlock, meta, 12, 5 + height, 13, sbb);
    this.placeBlock(world, doubleSlabBlock, meta, 13, 5 + height, 13, sbb);
    this.placeRandomly(world, sbb, random, 0.9, 3, 1 + height, 12, singleSlabBlock, meta);
    this.placeBlock(world, doubleSlabBlock, meta, 4, 1 + height, 12, sbb);
    this.placeBlock(world, singleSlabBlock, meta, 5, 2 + height, 12, sbb);
    this.placeBlock(world, doubleSlabBlock, meta, 6, 2 + height, 12, sbb);
    this.placeRandomly(world, sbb, random, 0.9, 7, 3 + height, 12, singleSlabBlock, meta);
    this.placeBlock(world, doubleSlabBlock, meta, 8, 3 + height, 12, sbb);
    this.placeBlock(world, singleSlabBlock, meta, 9, 4 + height, 12, sbb);
    this.placeRandomly(world, sbb, random, 0.9, 10, 4 + height, 12, doubleSlabBlock, meta);
    this.placeBlock(world, singleSlabBlock, meta, 11, 5 + height, 12, sbb);
    this.placeBlock(world, doubleSlabBlock, meta, 12, 5 + height, 12, sbb);
    this.placeBlock(world, doubleSlabBlock, meta, 13, 5 + height, 12, sbb);
    this.placeBlock(world, doubleSlabBlock, meta, 4, 1 + height, 11, sbb);
    this.placeBlock(world, doubleSlabBlock, meta, 5, 2 + height, 11, sbb);
    this.placeRandomly(world, sbb, random, 0.9, 6, 2 + height, 11, doubleSlabBlock, meta);
    this.placeBlock(world, doubleSlabBlock, meta, 7, 3 + height, 11, sbb);
    this.placeRandomly(world, sbb, random, 0.9, 8, 3 + height, 11, doubleSlabBlock, meta);
    this.placeBlock(world, doubleSlabBlock, meta, 9, 4 + height, 11, sbb);
    this.placeBlock(world, doubleSlabBlock, meta, 10, 4 + height, 11, sbb);
    this.placeBlock(world, doubleSlabBlock, meta, 11, 5 + height, 11, sbb);
    this.placeBlock(world, blk("fence"), 0, 4, 2 + height, 11, sbb);
    this.placeBlock(world, blk("fence"), 0, 5, 3 + height, 11, sbb);
    this.placeBlock(world, blk("fence"), 0, 6, 3 + height, 11, sbb);
    this.placeBlock(world, blk("fence"), 0, 7, 4 + height, 11, sbb);
    this.placeBlock(world, blk("fence"), 0, 8, 4 + height, 11, sbb);
    this.placeBlock(world, blk("fence"), 0, 9, 5 + height, 11, sbb);
    this.placeBlock(world, blk("fence"), 0, 10, 5 + height, 11, sbb);
    this.placeBlock(world, blk("fence"), 0, 11, 6 + height, 11, sbb);
    this.placeBlock(world, blk("fence"), 0, 4, 3 + height, 11, sbb);
    this.placeBlock(world, blk("fence"), 0, 6, 4 + height, 11, sbb);
    this.placeBlock(world, blk("fence"), 0, 8, 5 + height, 11, sbb);
    this.placeBlock(world, blk("fence"), 0, 10, 6 + height, 11, sbb);
    this.placeBlock(world, blk("fence"), 0, 11, 7 + height, 11, sbb);
    this.setMode(temp);
}

// EnumArt sizes in their 1.7 order, the picks draw from the random as the source does
const ART: [(i32, i32); 26] = [
    (16, 16),
    (16, 16),
    (16, 16),
    (16, 16),
    (16, 16),
    (16, 16),
    (16, 16),
    (32, 16),
    (32, 16),
    (32, 16),
    (32, 16),
    (32, 16),
    (16, 32),
    (16, 32),
    (32, 32),
    (32, 32),
    (32, 32),
    (32, 32),
    (32, 32),
    (32, 32),
    (64, 32),
    (64, 64),
    (64, 64),
    (64, 64),
    (64, 48),
    (64, 48),
];

// ComponentTFTowerWing.generatePaintingsOnWall, the spots and arts are drawn but the loader cannot hang a painting yet
pub fn generatePaintingsOnWall(
    this: &Piece,
    random: &Random,
    howMany: i32,
    floorLevel: i32,
    direction: i32,
    minSize: i32,
    sbb: StructureBox,
)
{
    for _ in 0..howMany
    {
        if tower_wing::getRandomWallSpot(this, random, floorLevel, direction, sbb).is_some()
        {
            tower_wing::getPaintingOfSize(random, minSize);
        }
    }
}

// ComponentTFTowerWing.getPaintingOfSize, an index into ART
pub fn getPaintingOfSize(random: &Random, minSize: i32) -> Option<usize>
{
    let valid: Vec<usize> = (0..ART.len()).filter(|&art| ART[art].0 >= minSize || ART[art].1 >= minSize).collect();
    if valid.is_empty()
    {
        None
    }
    else
    {
        Some(valid[random.nextIntBounded(valid.len() as i32) as usize])
    }
}

// ComponentTFTowerWing.getRandomWallSpot
pub fn getRandomWallSpot(
    this: &Piece,
    random: &Random,
    floorLevel: i32,
    direction: i32,
    sbb: StructureBox,
) -> Option<[i32; 3]>
{
    let mut minX: i32 = this.m_box.get().m_min[0] + 2;
    let mut maxX: i32 = this.m_box.get().m_max[0] - 2;
    let minY: i32 = (this.m_box.get().m_min[1] + floorLevel) + 2;
    let maxY: i32 = this.m_box.get().m_max[1] - 2;
    let mut minZ: i32 = this.m_box.get().m_min[2] + 2;
    let mut maxZ: i32 = this.m_box.get().m_max[2] - 2;
    match direction
    {
        0 =>
        {
            minZ = this.m_box.get().m_min[2];
            maxZ = this.m_box.get().m_min[2];
        }
        1 =>
        {
            maxX = this.m_box.get().m_max[0];
            minX = this.m_box.get().m_max[0];
        }
        2 =>
        {
            maxZ = this.m_box.get().m_max[2];
            minZ = this.m_box.get().m_max[2];
        }
        3 =>
        {
            minX = this.m_box.get().m_min[0];
            maxX = this.m_box.get().m_min[0];
        }
        _ =>
        {}
    }
    for _i in 0..30
    {
        let cx: i32 = minX
            + (if maxX > minX
            {
                random.nextIntBounded(maxX - minX)
            }
            else
            {
                0
            });
        let cy: i32 = minY
            + (if maxY > minY
            {
                random.nextIntBounded(maxY - minY)
            }
            else
            {
                0
            });
        let cz: i32 = minZ
            + (if maxZ > minZ
            {
                random.nextIntBounded(maxZ - minZ)
            }
            else
            {
                0
            });
        if sbb.contains(cx, cy, cz)
        {
            return Some([cx, cy, cz]);
        }
    }
    return None;
}

// ComponentTFTowerWing.makeGlyphBranches
pub fn makeGlyphBranches(this: &Piece, world: &GenWorld, random: &Random, meta: i32, sbb: StructureBox)
{
    if !((this.m_height.get() == 0) | (this.m_size.get() == 0))
    {
        let rotation: i32 = random.nextIntBounded(4);
        let startHeight: i32 = random.nextIntBounded(((this.m_height.get()) as f32 * 0.66) as i32);
        let startZ: i32 = 3 + random.nextIntBounded(this.m_size.get() - 6);
        let dx: i32 = this.xTurned(0, startZ, rotation);
        let dz: i32 = this.zTurned(0, startZ, rotation);
        if sbb.contains(dx, this.m_box.get().m_min[1] + 1, dz)
        {
            {
                let mut dy: i32 = this.yAt(startHeight);
                while (dy > 0) && (world.getBlock(dx, dy, dz) == blk("tf:castleBlock"))
                {
                    world.setBlock(dx, dy, dz, blk("tf:castleMagic"), meta, 2);
                    dy -= 1;
                }
            }
        }
        let leftOffset: i32 = startZ - (1 + random.nextIntBounded(3));
        let leftHeight: i32 = random.nextIntBounded(this.m_height.get() - startHeight);
        if leftOffset >= 0
        {
            for z in (leftOffset + 1..=startZ).rev()
            {
                this.placeTurned(world, blk("tf:castleMagic"), meta, 0, startHeight, z, rotation, sbb);
            }
            for y in startHeight..(startHeight + leftHeight)
            {
                this.placeTurned(world, blk("tf:castleMagic"), meta, 0, y, leftOffset, rotation, sbb);
            }
        }
        let rightOffset: i32 = (startZ + 1) + random.nextIntBounded(3);
        let rightHeight: i32 = random.nextIntBounded(this.m_height.get() - startHeight);
        if rightOffset < (this.m_size.get() - 1)
        {
            for z in startZ..rightOffset
            {
                this.placeTurned(world, blk("tf:castleMagic"), meta, 0, startHeight, z, rotation, sbb);
            }
            for y in startHeight..(startHeight + rightHeight)
            {
                this.placeTurned(world, blk("tf:castleMagic"), meta, 0, y, rightOffset, rotation, sbb);
            }
        }
    }
}
