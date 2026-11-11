// draft from ComponentTFTrollCaveMain.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) const UBEROUS_GEN: Gen = Gen::Blob { block: blk("tf:uberousSoil"), size: 4 };

pub(crate) fn new6(world: &GenWorld, random: &Random, i: i32, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::TrollCaveMain);
    init6(&this, world, random, i, x, y, z);
    Rc::new(this)
}

// ComponentTFTrollCaveMain constructor
pub(crate) fn init(this: &Piece, index: i32) { this.m_depth.set(index); }

// ComponentTFTrollCaveMain constructor
pub(crate) fn init6(this: &Piece, _world: &GenWorld, _random: &Random, _i: i32, x: i32, mut y: i32, z: i32)
{
    this.setMode(0);
    y += 10;
    this.m_size.set(30);
    this.m_height.set(20);

    let radius: i32 = this.m_size.get() / 2;
    this.m_box.set(boxFor(
        x,
        y,
        z,
        -radius,
        -this.m_height.get(),
        -radius,
        this.m_size.get(),
        this.m_height.get(),
        this.m_size.get(),
        0,
    ));
}

// ComponentTFTrollCaveMain.func_74861_a
pub(crate) fn build(this: &Piece, _parent: &Piece, list: &mut Pieces, random: &Random)
{
    for i in 0..4
    {
        let dest: Option<[i32; 3]> = troll_cave_main::validOpening3(this, random, 5, i);
        this.makeSmallerCave(
            list,
            random,
            this.depth() + 1,
            dest.piece()[0],
            dest.piece()[1],
            dest.piece()[2],
            18,
            15,
            i,
        );
    }

    let castle: Rc<Piece> = cloud_castle::new(
        this.depth() + 1,
        this.m_box.get().m_min[0] + ((this.m_box.get().m_max[0] - this.m_box.get().m_min[0]) / 2),
        168,
        this.m_box.get().m_min[2] + ((this.m_box.get().m_max[2] - this.m_box.get().m_min[2]) / 2),
    );
    list.push(Rc::clone(&castle));
    castle.build(this, list, random);

    let vault: Rc<Piece> = troll_vault::new(
        this.depth() + 1,
        this.m_box.get().m_min[0] + ((this.m_box.get().m_max[0] - this.m_box.get().m_min[0]) / 2),
        this.m_box.get().m_min[1],
        this.m_box.get().m_min[2] + ((this.m_box.get().m_max[2] - this.m_box.get().m_min[2]) / 2),
    );
    list.push(Rc::clone(&vault));
    vault.build(this, list, random);
}

// ComponentTFTrollCaveMain.makeSmallerCave
pub(crate) fn makeSmallerCave(
    this: &Piece,
    list: &mut Pieces,
    random: &Random,
    index: i32,
    x: i32,
    y: i32,
    z: i32,
    caveSize: i32,
    caveHeight: i32,
    rotation: i32,
) -> bool
{
    let direction: i32 = (this.mode() + rotation) % 4;
    let dest: [i32; 3] = this.offsetTowerAt(x, y, z, caveSize, direction);
    let cave: Rc<Piece> = troll_cave_connect::new(index, dest[0], dest[1], dest[2], caveSize, caveHeight, direction);
    let intersect: Option<Rc<Piece>> = intersecting(list, cave.bounds());
    if (intersect.is_some()) && (!same(&intersect, &this))
    {
        return false;
    }

    list.push(Rc::clone(&cave));
    cave.build(&Rc::clone(&list[0usize]), list, random);
    true
}

// ComponentTFTrollCaveMain.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    let decoRNG: Random = Random::withSeed(
        (world.seed() + ((this.m_box.get().m_min[0] * 321534781) as i64))
            ^ ((this.m_box.get().m_min[2] * 756839) as i64),
    );
    troll_cave_main::hollowCaveMiddle(
        this,
        world,
        sbb,
        random,
        0,
        0,
        0,
        this.m_size.get() - 1,
        this.m_height.get() - 1,
        this.m_size.get() - 1,
    );

    for _i in 0..128
    {
        let dest: [i32; 3] = troll_cave_main::coordsInCave(this, &decoRNG);
        troll_cave_main::generateBlockStalactite9(
            this,
            world,
            &decoRNG,
            blk("stone"),
            0.7,
            true,
            dest[0],
            3,
            dest[2],
            sbb,
        );
    }

    for _i in 0..32
    {
        let dest: [i32; 3] = troll_cave_main::coordsInCave(this, &decoRNG);
        troll_cave_main::generateBlockStalactite9(
            this,
            world,
            &decoRNG,
            blk("stone"),
            0.5,
            false,
            dest[0],
            3,
            dest[2],
            sbb,
        );
    }

    for _i in 0..32
    {
        let dest: [i32; 3] = troll_cave_main::coordsInCave(this, &decoRNG);
        troll_cave_main::generateAtSurface(this, world, UBEROUS_GEN, &decoRNG, dest[0], 60, dest[2], sbb);
    }

    true
}

// ComponentTFTrollCaveMain.getCoordsInCave
pub(crate) fn coordsInCave(this: &Piece, random: &Random) -> [i32; 3]
{
    [
        random.nextIntBounded(this.m_size.get() - 1),
        random.nextIntBounded(this.m_height.get() - 1),
        random.nextIntBounded(this.m_size.get() - 1),
    ]
}

// ComponentTFTrollCaveMain.hollowCaveMiddle
pub(crate) fn hollowCaveMiddle(
    this: &Piece,
    world: &GenWorld,
    sbb: StructureBox,
    random: &Random,
    minX: i32,
    minY: i32,
    minZ: i32,
    maxX: i32,
    maxY: i32,
    maxZ: i32,
)
{
    let threshold: i32 = this.m_size.get() / 5;

    for y in minY..=maxY
    {
        for x in minX..=maxX
        {
            for z in minZ..=maxZ
            {
                let ex: i32 = (x - minX).min(maxX - x);
                let ey: i32 = ((y - minY) * 2).min(maxY - y);
                let ez: i32 = (z - minZ).min(maxZ - z);
                let dist: f64 = (((ex * ey) * ez) as f64).sqrt();
                if dist > (threshold) as f64
                {
                    this.placeBlock(world, blk("air"), 0, x, y, z, sbb);
                }
                else if ((dist == (threshold) as f64) && (random.nextIntBounded(4) == 0))
                    && (this.getBlock(world, x, y, z, sbb) == blk("stone"))
                {
                    this.placeBlock(world, blk("tf:trollSteinn"), 0, x, y, z, sbb);
                }
            }
        }
    }
}

// ComponentTFTrollCaveMain.getValidOpening
pub(crate) fn validOpening3(this: &Piece, random: &Random, _caveHeight: i32, direction: i32) -> Option<[i32; 3]>
{
    let offset: i32 = this.m_size.get() / 4;
    let wLength: i32 = this.m_size.get() - (offset * 2);

    if (direction != 0) && (direction != 2)
    {
        if (direction != 1) && (direction != 3)
        {
            return None;
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
        let ry: i32 = random.nextIntBounded(offset) - random.nextIntBounded(offset);
        Some([rx, ry, rz])
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
        let ry: i32 = random.nextIntBounded(offset) - random.nextIntBounded(offset);
        Some([rx, ry, rz])
    }
}

// ComponentTFTrollCaveMain.offsetTowerCCoords
pub(crate) fn offsetTowerCCoords(this: &Piece, x: i32, y: i32, z: i32, towerSize: i32, direction: i32) -> [i32; 3]
{
    let dx: i32 = this.xAt(x, z);
    let dy: i32 = this.yAt(y);
    let dz: i32 = this.zAt(x, z);

    let var10000: [i32; 3] = match direction
    {
        0 => [dx - 1, dy - 1, dz - (towerSize / 2)],
        1 => [dx + (towerSize / 2), dy - 1, dz - 1],
        2 => [dx + 1, dy - 1, dz + (towerSize / 2)],
        3 => [dx - (towerSize / 2), dy - 1, dz + 1],
        _ => [x, y, z],
    };
    var10000
}

// ComponentTFTrollCaveMain.isBoundingBoxOutOfHighlands
pub(crate) fn boundingBoxOutOfHighlands(this: &Piece, world: &GenWorld, _sbb: StructureBox) -> bool
{
    let minX: i32 = this.m_box.get().m_min[0] - 1;
    let minZ: i32 = this.m_box.get().m_min[2] - 1;
    let maxX: i32 = this.m_box.get().m_max[0] + 1;
    let maxZ: i32 = this.m_box.get().m_max[2] + 1;

    for x in minX..=maxX
    {
        for z in minZ..=maxZ
        {
            if world.biome(x, z) != Biome::Highlands
            {
                return true;
            }
        }
    }

    false
}

// ComponentTFTrollCaveMain.generateBlockStalactite
pub(crate) fn generateBlockStalactite9(
    this: &Piece,
    world: &GenWorld,
    random: &Random,
    blockToGenerate: Blk,
    length: f32,
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
    if sbb.contains(dx, dy, dz)
    {
        Gen::stalactite(blockToGenerate, length, up).generate(world, random, dx, dy, dz);
    }
}

// ComponentTFTrollCaveMain.generateAtSurface
pub(crate) fn generateAtSurface(
    this: &Piece,
    world: &GenWorld,
    generator: Gen,
    random: &Random,
    x: i32,
    y: i32,
    z: i32,
    sbb: StructureBox,
)
{
    let dx: i32 = this.xAt(x, z);
    let mut dy: i32 = y;
    let dz: i32 = this.zAt(x, z);
    if sbb.contains(dx, dy, dz)
    {
        dy = y;
        while (dy < (y + 32)) && (!world.isAir(dx, dy, dz))
        {
            dy += 1;
        }

        generator.generate(world, random, dx, dy, dz);
    }
}

// ComponentTFTrollCaveMain.makeTreasureCrate
pub(crate) fn makeTreasureCrate(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox)
{
    let mid: i32 = this.m_size.get() / 2;
    this.fill(world, sbb, mid - 2, 0, mid - 2, mid + 1, 3, mid + 1, blk("obsidian"), blk("obsidian"), false);
    this.fillAir(world, sbb, mid - 1, 1, mid - 1, mid + 0, 2, mid + 0);
    this.placeTreasureTrapped(world, random, mid, 1, mid, Table::TrollGarden, false, sbb);
}
