// draft from ComponentTFHollowHill.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(world: &GenWorld, random: &Random, i: i32, size: i32, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::HollowHill);
    init(&this, world, random, i, size, x, y, z);
    Rc::new(this)
}

// ComponentTFHollowHill constructor
pub fn init(this: &Piece, world: &GenWorld, random: &Random, i: i32, size: i32, x: i32, y: i32, z: i32)
{
    this.m_depth.set(i);
    this.setMode(0);
    this.m_hillSize.set(size);
    this.m_radius.set((((this.m_hillSize.get() * 2) + 1) * 8) - 6);
    this.m_box.set(boxFor(
        x,
        y,
        z,
        -this.m_radius.get(),
        -3,
        -this.m_radius.get(),
        this.m_radius.get() * 2,
        10,
        this.m_radius.get() * 2,
        0,
    ));
}

// ComponentTFHollowHill.func_74861_a
pub fn build(this: &Piece, structurecomponent: &Piece, list: &mut Pieces, random: &Random) {}

// ComponentTFHollowHill.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    let sna: Vec<i32> = vec![0, 128, 256, 512];
    let sn: i32 = sna[(this.m_hillSize.get()) as usize];
    let mga: Vec<i32> = vec![0, 1, 4, 9];
    let mg: i32 = mga[(this.m_hillSize.get()) as usize];
    let tca: Vec<i32> = vec![0, 2, 6, 12];
    let tc: i32 = tca[(this.m_hillSize.get()) as usize];
    for i in 0..mg
    {
        let dest: Vec<i32> = hollow_hill::getCoordsInHill2D(this, random);
        let mobID: &'static str = hollow_hill::getMobID(this, random);
        this.placeSpawner(world, random, dest[0], random.nextIntBounded(4), dest[1], mobID, sbb);
    }
    for i in 0..tc
    {
        let dest: Vec<i32> = hollow_hill::getCoordsInHill2D(this, random);
        hollow_hill::generateTreasureChest(this, world, dest[0], 0, dest[1], sbb);
    }
    for i in 0..sn
    {
        let dest: Vec<i32> = hollow_hill::getCoordsInHill2D(this, random);
        hollow_hill::generateOreStalactite(this, world, dest[0], 1, dest[1], sbb);
    }
    for i in 0..sn
    {
        let dest: Vec<i32> = hollow_hill::getCoordsInHill2D(this, random);
        hollow_hill::generateBlockStalactite(this, world, blk("stone"), 1.0, true, dest[0], 1, dest[1], sbb);
    }
    for i in 0..sn
    {
        let dest: Vec<i32> = hollow_hill::getCoordsInHill2D(this, random);
        hollow_hill::generateBlockStalactite(this, world, blk("stone"), 0.9, false, dest[0], 1, dest[1], sbb);
    }
    return true;
}

// ComponentTFHollowHill.generateTreasureChest
pub fn generateTreasureChest(this: &Piece, world: &GenWorld, x: i32, y: i32, z: i32, sbb: StructureBox)
{
    let chestRNG: Random = Random::withSeed(world.seed() + ((x * z) as i64));
    this.placeTreasure(
        world,
        &chestRNG,
        x,
        y,
        z,
        if this.m_hillSize.get() == 3
        {
            Table::Hill3
        }
        else
        {
            if this.m_hillSize.get() == 2
            {
                Table::Hill2
            }
            else
            {
                Table::Hill1
            }
        },
        sbb,
    );
    this.fillDown(world, blk("cobblestone"), 0, x, y - 1, z, sbb);
}

// ComponentTFHollowHill.generateOreStalactite
pub fn generateOreStalactite(this: &Piece, world: &GenWorld, x: i32, y: i32, z: i32, sbb: StructureBox)
{
    let dx: i32 = this.xAt(x, z);
    let dy: i32 = this.yAt(y);
    let dz: i32 = this.zAt(x, z);
    if sbb.contains(dx, dy, dz) && (world.getBlock(dx, dy, dz) != blk("mob_spawner"))
    {
        let stalRNG: Random = Random::withSeed(world.seed() + ((dx * dz) as i64));
        let stalag: Gen = Gen::ore(&stalRNG, this.m_hillSize.get());
        stalag.generate(world, &stalRNG, dx, dy, dz);
    }
}

// ComponentTFHollowHill.generateBlockStalactite
pub fn generateBlockStalactite(
    this: &Piece,
    world: &GenWorld,
    blockToGenerate: Blk,
    mut length: f32,
    up: bool,
    x: i32,
    y: i32,
    z: i32,
    sbb: StructureBox,
)
{
    let dx: i32 = this.xAt(x, z);
    let dy: i32 = this.yAt(y);
    let dz: i32 = this.zAt(x, z);
    if sbb.contains(dx, dy, dz) && (world.getBlock(dx, dy, dz) != blk("mob_spawner"))
    {
        let stalRNG: Random = Random::withSeed(world.seed() + ((dx * dz) as i64));
        if this.m_hillSize.get() == 1
        {
            length *= 1.9;
        }
        Gen::stalactite(blockToGenerate, length, up).generate(world, &stalRNG, dx, dy, dz);
    }
}

// ComponentTFHollowHill.isInHill
pub fn isInHill(this: &Piece, cx: i32, cz: i32) -> bool
{
    let dx: i32 = this.m_radius.get() - cx;
    let dz: i32 = this.m_radius.get() - cz;
    let dist: i32 = ((((dx * dx) + (dz * dz)) as f64).sqrt()) as i32;
    return dist < this.m_radius.get();
}

// ComponentTFHollowHill.isInHill
pub fn isInHill3(this: &Piece, mapX: i32, mapY: i32, mapZ: i32) -> bool
{
    let dx: i32 = (this.m_box.get().m_min[0] + this.m_radius.get()) - mapX;
    let dy: i32 = (this.m_box.get().m_min[1] - mapY) * 2;
    let dz: i32 = (this.m_box.get().m_min[2] + this.m_radius.get()) - mapZ;
    let dist: i32 = ((dx * dx) + (dy * dy)) + (dz * dz);
    return dist < (this.m_radius.get() * this.m_radius.get());
}

// ComponentTFHollowHill.getCoordsInHill2D
pub fn getCoordsInHill2D(this: &Piece, random: &Random) -> Vec<i32>
{
    return hollow_hill::getCoordsInHill2D2(this, random, this.m_radius.get());
}

// ComponentTFHollowHill.getCoordsInHill2D
pub fn getCoordsInHill2D2(this: &Piece, random: &Random, rad: i32) -> Vec<i32>
{
    let mut rx: i32;
    let mut rz: i32;
    loop
    {
        rx = random.nextIntBounded(2 * rad);
        rz = random.nextIntBounded(2 * rad);
        if !(!this.isInHill(rx, rz))
        {
            break;
        }
    }
    return vec![rx, rz];
}

// ComponentTFHollowHill.getMobID
pub fn getMobID(this: &Piece, random: &Random) -> &'static str
{
    return hollow_hill::getMobID2(this, random, this.m_hillSize.get());
}

// ComponentTFHollowHill.getMobID
pub fn getMobID2(this: &Piece, random: &Random, level: i32) -> &'static str
{
    let mut var10000: &'static str;
    match level
    {
        1 =>
        {
            var10000 = hollow_hill::getLevel1Mob(this, random);
        }
        2 =>
        {
            var10000 = hollow_hill::getLevel2Mob(this, random);
        }
        3 =>
        {
            var10000 = hollow_hill::getLevel3Mob(this, random);
        }
        _ =>
        {
            var10000 = "Spider";
        }
    }
    return var10000;
}

// ComponentTFHollowHill.getLevel1Mob
pub fn getLevel1Mob(this: &Piece, random: &Random) -> &'static str
{
    let mut var10000: &'static str;
    match random.nextIntBounded(10)
    {
        0 | 1 | 2 =>
        {
            var10000 = mob("Swarm Spider");
        }
        3 | 4 | 5 =>
        {
            var10000 = "Spider";
        }
        6 | 7 =>
        {
            var10000 = "Zombie";
        }
        8 =>
        {
            var10000 = "Silverfish";
        }
        9 =>
        {
            var10000 = mob("Redcap");
        }
        _ =>
        {
            var10000 = mob("Swarm Spider");
        }
    }
    return var10000;
}

// ComponentTFHollowHill.getLevel2Mob
pub fn getLevel2Mob(this: &Piece, random: &Random) -> &'static str
{
    let mut var10000: &'static str;
    match random.nextIntBounded(10)
    {
        0 | 1 | 2 =>
        {
            var10000 = mob("Redcap");
        }
        3 | 4 | 5 =>
        {
            var10000 = "Zombie";
        }
        6 | 7 =>
        {
            var10000 = "Skeleton";
        }
        8 =>
        {
            var10000 = mob("Swarm Spider");
        }
        9 =>
        {
            var10000 = "CaveSpider";
        }
        _ =>
        {
            var10000 = mob("Redcap");
        }
    }
    return var10000;
}

// ComponentTFHollowHill.getLevel3Mob
pub fn getLevel3Mob(this: &Piece, random: &Random) -> &'static str
{
    let mut var10000: &'static str;
    match random.nextIntBounded(11)
    {
        0 =>
        {
            var10000 = mob("Slime Beetle");
        }
        1 =>
        {
            var10000 = mob("Fire Beetle");
        }
        2 =>
        {
            var10000 = mob("Pinch Beetle");
        }
        3 | 4 | 5 =>
        {
            var10000 = "Skeleton";
        }
        6 | 7 | 8 =>
        {
            var10000 = "CaveSpider";
        }
        9 =>
        {
            var10000 = "Creeper";
        }
        _ =>
        {
            var10000 = mob("Twilight Wraith");
        }
    }
    return var10000;
}
