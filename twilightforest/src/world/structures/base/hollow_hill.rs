// draft from ComponentTFHollowHill.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(world: &GenWorld, random: &Random, i: i32, size: i32, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::HollowHill);
    init(&this, world, random, i, size, x, y, z);
    Rc::new(this)
}

// ComponentTFHollowHill constructor
pub(crate) fn init(this: &Piece, _world: &GenWorld, _random: &Random, i: i32, size: i32, x: i32, y: i32, z: i32)
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
pub(crate) fn build(_this: &Piece, _structurecomponent: &Piece, _list: &mut Pieces, _random: &Random) {}

// ComponentTFHollowHill.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    let sna: Vec<i32> = vec![0, 128, 256, 512];
    let sn: i32 = sna[(this.m_hillSize.get()) as usize];
    let mga: Vec<i32> = vec![0, 1, 4, 9];
    let mg: i32 = mga[(this.m_hillSize.get()) as usize];
    let tca: Vec<i32> = vec![0, 2, 6, 12];
    let tc: i32 = tca[(this.m_hillSize.get()) as usize];

    for _i in 0..mg
    {
        let dest: Vec<i32> = hollow_hill::coordsInHill2D(this, random);
        let mobID: &'static str = hollow_hill::getMobID(this, random);
        this.placeSpawner(world, random, dest[0], random.nextIntBounded(4), dest[1], mobID, sbb);
    }

    for _i in 0..tc
    {
        let dest: Vec<i32> = hollow_hill::coordsInHill2D(this, random);
        hollow_hill::generateTreasureChest(this, world, dest[0], 0, dest[1], sbb);
    }

    for _i in 0..sn
    {
        let dest: Vec<i32> = hollow_hill::coordsInHill2D(this, random);
        hollow_hill::generateOreStalactite(this, world, dest[0], 1, dest[1], sbb);
    }

    for _i in 0..sn
    {
        let dest: Vec<i32> = hollow_hill::coordsInHill2D(this, random);
        hollow_hill::generateBlockStalactite(this, world, blk("stone"), 1.0, true, dest[0], 1, dest[1], sbb);
    }

    for _i in 0..sn
    {
        let dest: Vec<i32> = hollow_hill::coordsInHill2D(this, random);
        hollow_hill::generateBlockStalactite(this, world, blk("stone"), 0.9, false, dest[0], 1, dest[1], sbb);
    }

    true
}

// ComponentTFHollowHill.generateTreasureChest
pub(crate) fn generateTreasureChest(this: &Piece, world: &GenWorld, x: i32, y: i32, z: i32, sbb: StructureBox)
{
    let chestRNG: Random = Random::withSeed(world.seed() + ((x * z) as i64));

    let table: Table = if this.m_hillSize.get() == 3
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
    };
    this.placeTreasure(world, &chestRNG, x, y, z, table, sbb);
    this.fillDown(world, blk("cobblestone"), 0, x, y - 1, z, sbb);
}

// ComponentTFHollowHill.generateOreStalactite
pub(crate) fn generateOreStalactite(this: &Piece, world: &GenWorld, x: i32, y: i32, z: i32, sbb: StructureBox)
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
pub(crate) fn generateBlockStalactite(
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
pub(crate) fn isInHill(this: &Piece, cx: i32, cz: i32) -> bool
{
    let dx: i32 = this.m_radius.get() - cx;
    let dz: i32 = this.m_radius.get() - cz;
    let dist: i32 = ((((dx * dx) + (dz * dz)) as f64).sqrt()) as i32;
    dist < this.m_radius.get()
}

// ComponentTFHollowHill.getCoordsInHill2D
pub(crate) fn coordsInHill2D(this: &Piece, random: &Random) -> Vec<i32>
{
    hollow_hill::coordsInHill2D2(this, random, this.m_radius.get())
}

// ComponentTFHollowHill.getCoordsInHill2D
pub(crate) fn coordsInHill2D2(this: &Piece, random: &Random, rad: i32) -> Vec<i32>
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

    vec![rx, rz]
}

// ComponentTFHollowHill.getMobID
pub(crate) fn getMobID(this: &Piece, random: &Random) -> &'static str
{
    hollow_hill::mobID2(this, random, this.m_hillSize.get())
}

// ComponentTFHollowHill.getMobID
pub(crate) fn mobID2(this: &Piece, random: &Random, level: i32) -> &'static str
{
    let var10000: &'static str = match level
    {
        1 => hollow_hill::level1Mob(this, random),
        2 => hollow_hill::level2Mob(this, random),
        3 => hollow_hill::level3Mob(this, random),
        _ => "Spider",
    };
    var10000
}

// ComponentTFHollowHill.getLevel1Mob
pub(crate) fn level1Mob(_this: &Piece, random: &Random) -> &'static str
{
    let var10000: &'static str = match random.nextIntBounded(10)
    {
        0 | 1 | 2 => mob("Swarm Spider"),
        3 | 4 | 5 => "Spider",
        6 | 7 => "Zombie",
        8 => "Silverfish",
        9 => mob("Redcap"),
        _ => mob("Swarm Spider"),
    };
    var10000
}

// ComponentTFHollowHill.getLevel2Mob
pub(crate) fn level2Mob(_this: &Piece, random: &Random) -> &'static str
{
    let var10000: &'static str = match random.nextIntBounded(10)
    {
        0 | 1 | 2 => mob("Redcap"),
        3 | 4 | 5 => "Zombie",
        6 | 7 => "Skeleton",
        8 => mob("Swarm Spider"),
        9 => "CaveSpider",
        _ => mob("Redcap"),
    };
    var10000
}

// ComponentTFHollowHill.getLevel3Mob
pub(crate) fn level3Mob(_this: &Piece, random: &Random) -> &'static str
{
    let var10000: &'static str = match random.nextIntBounded(11)
    {
        0 => mob("Slime Beetle"),
        1 => mob("Fire Beetle"),
        2 => mob("Pinch Beetle"),
        3 | 4 | 5 => "Skeleton",
        6 | 7 | 8 => "CaveSpider",
        9 => "Creeper",
        _ => mob("Twilight Wraith"),
    };
    var10000
}
