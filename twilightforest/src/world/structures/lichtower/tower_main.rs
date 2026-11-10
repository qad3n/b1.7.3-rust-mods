// draft from ComponentTFTowerMain.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(world: &GenWorld, random: &Random, index: i32, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::TowerMain);
    init(&this, world, random, index, x, y, z);
    Rc::new(this)
}

// ComponentTFTowerMain constructor
pub fn init(this: &Piece, world: &GenWorld, random: &Random, index: i32, x: i32, y: i32, z: i32)
{
    tower_wing::init7(this, index, x, y, z, 15, 55 + random.nextIntBounded(32), 0);
}

// ComponentTFTowerMain.func_74861_a
pub fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    this.makeARoof(&parent, list, random);
    for i in 0..4
    {
        let mut dest: Vec<i32> = this.getValidOpening(random, i);
        if dest[1] < (this.m_height.get() / 2)
        {
            dest[1] += 20;
        }
        let childHeight: i32 = (21 + random.nextIntBounded(10)).min((this.m_height.get() - dest[1]) - 3);
        if !this.makeTowerWing(list, random, 1, dest[0], dest[1], dest[2], 9, childHeight, i)
        {
            this.makeTowerWing(list, random, 1, dest[0], dest[1], dest[2], 7, childHeight, i);
        }
    }
    for i in 0..4
    {
        let mut dest: Vec<i32> = this.getValidOpening(random, i);
        if dest[1] < (this.m_height.get() / 2)
        {
            dest[1] += 10;
        }
        let childHeight: i32 = (21 + random.nextIntBounded(10)).min((this.m_height.get() - dest[1]) - 3);
        if !this.makeTowerWing(list, random, 1, dest[0], dest[1], dest[2], 9, childHeight, i)
        {
            this.makeTowerWing(list, random, 1, dest[0], dest[1], dest[2], 7, childHeight, i);
        }
    }
    for i in 0..4
    {
        let mut dest: Vec<i32> = this.getValidOpening(random, i);
        let childHeight: i32 = (7 + random.nextIntBounded(6)).min((this.m_height.get() - dest[1]) - 3);
        if !this.makeTowerWing(list, random, 1, dest[0], dest[1], dest[2], 5, childHeight, i)
        {
            this.makeTowerWing(list, random, 1, dest[0], dest[1], dest[2], 3, childHeight, i);
        }
    }
    for i in 0..4
    {
        let mut dest: Vec<i32> = tower_main::getOutbuildingOpening(this, random, i);
        let childHeight: i32 = 11 + random.nextIntBounded(10);
        let childSize: i32 = 7 + (random.nextIntBounded(2) * 2);
        tower_main::makeTowerOutbuilding(this, list, random, 1, dest[0], dest[1], dest[2], childSize, childHeight, i);
    }
    for i in 0..16
    {
        let mut dest: Vec<i32> = this.getValidOpening(random, i % 4);
        let childHeight: i32 = 6 + random.nextIntBounded(5);
        if (random.nextIntBounded(3) == 0)
            || (!this.makeTowerWing(list, random, 1, dest[0], dest[1], dest[2], 5, childHeight, i % 4))
        {
            this.makeTowerWing(list, random, 1, dest[0], dest[1], dest[2], 3, childHeight, i % 4);
        }
    }
}

// ComponentTFTowerMain.getOutbuildingOpening
pub fn getOutbuildingOpening(this: &Piece, random: &Random, rotation: i32) -> Vec<i32>
{
    let mut rx: i32 = 0;
    let ry: i32 = 1;
    let mut rz: i32 = 0;
    match rotation
    {
        0 =>
        {
            rx = this.m_size.get() - 1;
            rz = 6 + random.nextIntBounded(8);
        }
        1 =>
        {
            rx = 1 + random.nextIntBounded(11);
            rz = this.m_size.get() - 1;
        }
        2 =>
        {
            rx = 0;
            rz = 1 + random.nextIntBounded(8);
        }
        3 =>
        {
            rx = 3 + random.nextIntBounded(11);
            rz = 0;
        }
        _ =>
        {}
    }
    return vec![rx, ry, rz];
}

// ComponentTFTowerMain.makeTowerOutbuilding
pub fn makeTowerOutbuilding(
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
    let outbuilding: Rc<Piece> = tower_outbuilding::new(index, dx[0], dx[1], dx[2], wingSize, wingHeight, direction);
    let intersect: Option<Rc<Piece>> = intersecting(list, outbuilding.bounds());
    if (intersect.is_some()) && (!same(&intersect, &this))
    {
        return false;
    }
    list.push(Rc::clone(&outbuilding));
    outbuilding.build(this, list, random);
    this.addOpening(x, y, z, rotation);
    return true;
}

// ComponentTFTowerMain.func_74875_a
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
    for x in 0..this.m_size.get()
    {
        for z in 0..this.m_size.get()
        {
            this.fillDown(world, blk("cobblestone"), 0, x, -1, z, sbb);
        }
    }
    this.darken(world);
    if (this.m_height.get() - this.m_highestOpening.get()) > 15
    {
        this.m_highestOpening.set(this.m_height.get() - 15);
    }
    tower_wing::makeStairs(this, world, random, sbb);
    this.makeOpenings(world, sbb);
    tower_wing::decorateStairFloor(this, world, random, sbb);
    tower_main::makeStairwayCrossings(this, world, random, sbb);
    tower_main::makeLichRoom(this, world, random, sbb);
    tower_main::makeTowerPaintings(this, world, random, sbb);
    return true;
}

// ComponentTFTowerMain.makeStairwayCrossings
pub fn makeStairwayCrossings(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox)
{
    let flights: i32 = (this.m_highestOpening.get() / 5) - 2;
    {
        let mut i: i32 = 2 + random.nextIntBounded(2);
        while i < flights
        {
            tower_main::makeStairCrossing(this, world, random, i, sbb);
            i += 1 + random.nextIntBounded(5);
        }
    }
}

// ComponentTFTowerMain.makeStairCrossing
pub fn makeStairCrossing(this: &Piece, world: &GenWorld, random: &Random, flight: i32, sbb: StructureBox)
{
    let temp: i32 = this.mode();
    if (flight % 2) == 0
    {
        this.setMode((this.mode() + 1) % 4);
    }
    let floorMeta: i32 = if random.nextIntBounded(2) == 0 { 0 } else { 2 };
    let mut floorLevel: i32 = 0 + (flight * 5);
    for dx in 6..=8
    {
        for dz in 4..=10
        {
            this.placeBlock(world, blk("double_stone_slab"), floorMeta, dx, floorLevel, dz, sbb);
        }
    }
    floorLevel += 1;
    let mut dx: i32 = 6;
    for dz in 3..=11
    {
        this.placeBlock(world, blk("fence"), 0, dx, floorLevel, dz, sbb);
    }
    dx += 1;
    for dz in 3..=11
    {
        this.placeBlock(world, blk("air"), 0, dx, floorLevel, dz, sbb);
    }
    dx += 1;
    for dz in 3..=11
    {
        this.placeBlock(world, blk("fence"), 0, dx, floorLevel, dz, sbb);
    }
    this.placeBlock(world, blk("double_stone_slab"), floorMeta, 6, floorLevel - 1, 11, sbb);
    this.placeBlock(world, blk("double_stone_slab"), floorMeta, 8, floorLevel - 1, 3, sbb);
    this.placeBlock(world, blk("fence"), 0, 5, floorLevel, 11, sbb);
    this.placeBlock(world, blk("fence"), 0, 9, floorLevel, 3, sbb);
    let mut var10000: &'static str;
    match random.nextIntBounded(4)
    {
        0 | 1 =>
        {
            var10000 = "Skeleton";
        }
        2 =>
        {
            var10000 = "Zombie";
        }
        3 =>
        {
            var10000 = mob("Swarm Spider");
        }
        _ =>
        {
            var10000 = "Skeleton";
        }
    }
    let mobID: &'static str = var10000;
    this.placeSpawner(world, random, 7, floorLevel + 2, 7, mobID, sbb);
    this.placeBlock(world, blk("fence"), 0, 6, floorLevel + 1, 7, sbb);
    this.placeBlock(world, blk("fence"), 0, 8, floorLevel + 1, 7, sbb);
    this.placeBlock(world, blk("fence"), 0, 6, floorLevel + 2, 7, sbb);
    this.placeBlock(world, blk("fence"), 0, 8, floorLevel + 2, 7, sbb);
    this.setMode(temp);
}

// ComponentTFTowerMain.makeLichRoom
pub fn makeLichRoom(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox)
{
    let floorLevel: i32 = 2 + ((this.m_highestOpening.get() / 5) * 5);
    tower_main::makeLichFloor(this, world, floorLevel, (this.m_highestOpening.get() / 5) % 2, sbb);
    tower_main::decorateLichChandelier(this, world, floorLevel, sbb);
    tower_main::decoratePaintings(this, world, random, floorLevel, sbb);
    tower_main::decorateTorches(this, world, random, floorLevel, sbb);
    this.placeBlock(world, blk("tf:bossSpawner"), 1, this.m_size.get() / 2, floorLevel + 2, this.m_size.get() / 2, sbb);
}

// ComponentTFTowerMain.makeTowerPaintings
pub fn makeTowerPaintings(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox)
{
    let howMany: i32 = 10;
    tower_wing::generatePaintingsOnWall(this, random, howMany, 0, 0, 48, sbb);
    tower_wing::generatePaintingsOnWall(this, random, howMany, 0, 0, 32, sbb);
    tower_wing::generatePaintingsOnWall(this, random, howMany, 0, 0, 0, sbb);
    tower_wing::generatePaintingsOnWall(this, random, howMany, 0, 1, 48, sbb);
    tower_wing::generatePaintingsOnWall(this, random, howMany, 0, 1, 32, sbb);
    tower_wing::generatePaintingsOnWall(this, random, howMany, 0, 1, 0, sbb);
    tower_wing::generatePaintingsOnWall(this, random, howMany, 0, 2, 48, sbb);
    tower_wing::generatePaintingsOnWall(this, random, howMany, 0, 2, 32, sbb);
    tower_wing::generatePaintingsOnWall(this, random, howMany, 0, 2, 0, sbb);
    tower_wing::generatePaintingsOnWall(this, random, howMany, 0, 3, 48, sbb);
    tower_wing::generatePaintingsOnWall(this, random, howMany, 0, 3, 32, sbb);
    tower_wing::generatePaintingsOnWall(this, random, howMany, 0, 3, 0, sbb);
}

// ComponentTFTowerMain.makeLichFloor
pub fn makeLichFloor(this: &Piece, world: &GenWorld, floorLevel: i32, rotation: i32, sbb: StructureBox)
{
    let temp: i32 = this.mode();
    this.setMode((this.mode() + rotation) % 4);
    for fx in 1..14
    {
        for fz in 1..14
        {
            if (((fx == 1) || (fx == 2)) && (fz >= 6)) && (fz <= 12)
            {
                if fz == 6
                {
                    this.placeBlock(world, blk("wooden_slab"), 10, fx, floorLevel, fz, sbb);
                }
            }
            else if (((fx == 12) || (fx == 13)) && (fz >= 3)) && (fz <= 8)
            {
                if fz == 8
                {
                    this.placeBlock(world, blk("wooden_slab"), 10, fx, floorLevel, fz, sbb);
                }
            }
            else if (((fx >= 4) && (fx <= 10)) && (fz >= 4)) && (fz <= 10)
            {
                if ((fx != 4) || (fz != 4)) && ((fx != 10) || (fz != 10))
                {
                    this.placeBlock(world, blk("glass"), 0, fx, floorLevel, fz, sbb);
                }
                else
                {
                    this.placeBlock(world, blk("planks"), 2, fx, floorLevel, fz, sbb);
                }
            }
            else if ((fx != 2) && (fx != 3)) || ((fz != 2) && (fz != 3))
            {
                if ((fx != 11) && (fx != 12)) || ((fz != 11) && (fz != 12))
                {
                    this.placeBlock(world, blk("planks"), 2, fx, floorLevel, fz, sbb);
                }
                else
                {
                    this.placeBlock(world, blk("glass"), 0, fx, floorLevel, fz, sbb);
                }
            }
            else
            {
                this.placeBlock(world, blk("glass"), 0, fx, floorLevel, fz, sbb);
            }
        }
    }
    this.placeBlock(world, blk("air"), 0, 3, floorLevel + 1, 11, sbb);
    this.placeBlock(world, blk("air"), 0, 3, floorLevel + 1, 10, sbb);
    this.placeBlock(world, blk("air"), 0, 3, floorLevel + 2, 11, sbb);
    this.placeBlock(world, blk("air"), 0, 11, floorLevel + 1, 3, sbb);
    this.placeBlock(world, blk("air"), 0, 11, floorLevel + 1, 4, sbb);
    this.placeBlock(world, blk("air"), 0, 11, floorLevel + 2, 3, sbb);
    this.setMode(temp);
}

// ComponentTFTowerMain.decorateLichChandelier
pub fn decorateLichChandelier(this: &Piece, world: &GenWorld, floorLevel: i32, sbb: StructureBox)
{
    let cx: i32 = this.m_size.get() / 2;
    let mut cy: i32 = floorLevel + 4;
    let cz: i32 = this.m_size.get() / 2;
    this.placeBlock(world, blk("fence"), 0, cx + 1, cy, cz + 0, sbb);
    this.placeBlock(world, blk("fence"), 0, cx + 2, cy, cz + 0, sbb);
    this.placeBlock(world, blk("fence"), 0, cx + 1, cy, cz + 1, sbb);
    this.placeBlock(world, blk("fence"), 0, cx + 0, cy, cz + 1, sbb);
    this.placeBlock(world, blk("fence"), 0, cx + 0, cy, cz + 2, sbb);
    this.placeBlock(world, blk("fence"), 0, cx - 1, cy, cz + 1, sbb);
    this.placeBlock(world, blk("fence"), 0, cx - 1, cy, cz + 0, sbb);
    this.placeBlock(world, blk("fence"), 0, cx - 2, cy, cz + 0, sbb);
    this.placeBlock(world, blk("fence"), 0, cx - 1, cy, cz - 1, sbb);
    this.placeBlock(world, blk("fence"), 0, cx + 0, cy, cz - 1, sbb);
    this.placeBlock(world, blk("fence"), 0, cx + 0, cy, cz - 2, sbb);
    this.placeBlock(world, blk("fence"), 0, cx + 1, cy, cz - 1, sbb);
    this.placeBlock(
        world,
        blk("fence"),
        0,
        cx + 1,
        {
            cy += 1;
            cy
        },
        cz + 0,
        sbb,
    );
    this.placeBlock(world, blk("torch"), 0, cx + 2, cy, cz + 0, sbb);
    this.placeBlock(world, blk("torch"), 0, cx + 1, cy, cz + 1, sbb);
    this.placeBlock(world, blk("fence"), 0, cx + 0, cy, cz + 1, sbb);
    this.placeBlock(world, blk("torch"), 0, cx + 0, cy, cz + 2, sbb);
    this.placeBlock(world, blk("torch"), 0, cx - 1, cy, cz + 1, sbb);
    this.placeBlock(world, blk("fence"), 0, cx - 1, cy, cz + 0, sbb);
    this.placeBlock(world, blk("torch"), 0, cx - 2, cy, cz + 0, sbb);
    this.placeBlock(world, blk("torch"), 0, cx - 1, cy, cz - 1, sbb);
    this.placeBlock(world, blk("fence"), 0, cx + 0, cy, cz - 1, sbb);
    this.placeBlock(world, blk("torch"), 0, cx + 0, cy, cz - 2, sbb);
    this.placeBlock(world, blk("torch"), 0, cx + 1, cy, cz - 1, sbb);
    this.placeBlock(
        world,
        blk("torch"),
        0,
        cx + 1,
        {
            cy += 1;
            cy
        },
        cz + 0,
        sbb,
    );
    this.placeBlock(world, blk("torch"), 0, cx + 0, cy, cz + 1, sbb);
    this.placeBlock(world, blk("torch"), 0, cx - 1, cy, cz + 0, sbb);
    this.placeBlock(world, blk("torch"), 0, cx + 0, cy, cz - 1, sbb);
    for y in floorLevel + 5..(this.m_height.get() - 1)
    {
        this.placeBlock(world, blk("fence"), 0, cx + 0, y, cz + 0, sbb);
    }
}

// ComponentTFTowerMain.decoratePaintings
pub fn decoratePaintings(this: &Piece, world: &GenWorld, random: &Random, floorLevel: i32, sbb: StructureBox)
{
    let howMany: i32 = 100;
    tower_wing::generatePaintingsOnWall(this, random, howMany, floorLevel, 0, 48, sbb);
    tower_wing::generatePaintingsOnWall(this, random, howMany, floorLevel, 0, 32, sbb);
    tower_wing::generatePaintingsOnWall(this, random, howMany, floorLevel, 0, 0, sbb);
    tower_wing::generatePaintingsOnWall(this, random, howMany, floorLevel, 1, 48, sbb);
    tower_wing::generatePaintingsOnWall(this, random, howMany, floorLevel, 1, 32, sbb);
    tower_wing::generatePaintingsOnWall(this, random, howMany, floorLevel, 1, 0, sbb);
    tower_wing::generatePaintingsOnWall(this, random, howMany, floorLevel, 2, 48, sbb);
    tower_wing::generatePaintingsOnWall(this, random, howMany, floorLevel, 2, 32, sbb);
    tower_wing::generatePaintingsOnWall(this, random, howMany, floorLevel, 2, 0, sbb);
    tower_wing::generatePaintingsOnWall(this, random, howMany, floorLevel, 3, 48, sbb);
    tower_wing::generatePaintingsOnWall(this, random, howMany, floorLevel, 3, 32, sbb);
    tower_wing::generatePaintingsOnWall(this, random, howMany, floorLevel, 3, 0, sbb);
}

// ComponentTFTowerMain.decorateTorches
pub fn decorateTorches(this: &Piece, world: &GenWorld, random: &Random, floorLevel: i32, sbb: StructureBox)
{
    tower_main::generateTorchesOnWall(this, world, random, floorLevel, 0, sbb);
    tower_main::generateTorchesOnWall(this, world, random, floorLevel, 1, sbb);
    tower_main::generateTorchesOnWall(this, world, random, floorLevel, 2, sbb);
    tower_main::generateTorchesOnWall(this, world, random, floorLevel, 3, sbb);
}

// ComponentTFTowerMain.generateTorchesOnWall
pub fn generateTorchesOnWall(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    floorLevel: i32,
    direction: i32,
    sbb: StructureBox,
)
{
    for i in 0..5
    {
        let Some(mut tCoords) = tower_wing::getRandomWallSpot(this, random, floorLevel, direction, sbb)
        else
        {
            continue;
        };

        match direction
        {
            0 =>
            {
                tCoords[2] += 1;
            }
            1 =>
            {
                tCoords[0] -= 1;
            }
            2 =>
            {
                tCoords[2] -= 1;
            }
            3 =>
            {
                tCoords[0] += 1;
            }
            _ =>
            {}
        }
        if ((world.getBlock(tCoords[0], tCoords[1], tCoords[2]) == blk("air"))
            && (world.getBlock(tCoords[0], tCoords[1] + 1, tCoords[2]) == blk("air")))
        {
            world.setBlock(tCoords[0], tCoords[1], tCoords[2], blk("fence"), 0, 2);
            world.setBlock(tCoords[0], tCoords[1] + 1, tCoords[2], blk("torch"), 5, 2);
        }
    }
}
