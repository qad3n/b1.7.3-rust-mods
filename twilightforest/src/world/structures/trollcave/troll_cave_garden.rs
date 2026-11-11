// draft from ComponentTFTrollCaveGarden.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(index: i32, x: i32, y: i32, z: i32, caveSize: i32, caveHeight: i32, direction: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::TrollCaveGarden);
    init(&this, index, x, y, z, caveSize, caveHeight, direction);
    Rc::new(this)
}

// ComponentTFTrollCaveGarden constructor
pub fn init(this: &Piece, index: i32, x: i32, y: i32, z: i32, caveSize: i32, caveHeight: i32, direction: i32)
{
    troll_cave_main::init(this, index);
    this.m_myceliumBlobGen.set(Gen::Blob { block: blk("mycelium"), size: 5 });
    this.m_dirtGen.set(Gen::Blob { block: blk("dirt"), size: 5 });
    this.m_bigMushroomGen.set(Gen::BigMushroom);
    this.m_bigMushgloomGen.set(Gen::BigMushgloom);
    this.m_size.set(caveSize);
    this.m_height.set(caveHeight);
    this.setMode(direction);
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

// ComponentTFTrollCaveGarden.func_74861_a
pub fn build(_this: &Piece, _parent: &Piece, _list: &mut Pieces, _random: &Random) {}

// ComponentTFTrollCaveGarden.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    if troll_cave_main::isBoundingBoxOutOfHighlands(this, world, sbb)
    {
        return false;
    }
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
    let decoRNG: Random = Random::withSeed(
        (world.seed() + ((this.m_box.get().m_min[0] * 321534781) as i64))
            ^ ((this.m_box.get().m_min[2] * 756839) as i64),
    );
    troll_cave_main::makeTreasureCrate(this, world, random, sbb);
    for _i in 0..24
    {
        let dest: [i32; 3] = troll_cave_main::getCoordsInCave(this, &decoRNG);
        troll_cave_garden::generate(this, world, this.m_dirtGen.get(), &decoRNG, dest[0], 1, dest[2], sbb);
    }
    for _i in 0..16
    {
        let dest: [i32; 3] = troll_cave_main::getCoordsInCave(this, &decoRNG);
        troll_cave_garden::generate(this, world, this.m_myceliumBlobGen.get(), &decoRNG, dest[0], 1, dest[2], sbb);
    }
    for _i in 0..16
    {
        let dest: [i32; 3] = troll_cave_main::getCoordsInCave(this, &decoRNG);
        troll_cave_garden::generate(this, world, troll_cave_main::UBEROUS_GEN, &decoRNG, dest[0], 1, dest[2], sbb);
        troll_cave_main::generateAtSurface(
            this,
            world,
            troll_cave_main::UBEROUS_GEN,
            &decoRNG,
            dest[0],
            60,
            dest[2],
            sbb,
        );
    }
    for _i in 0..32
    {
        let dest: [i32; 3] = troll_cave_main::getCoordsInCave(this, &decoRNG);
        troll_cave_garden::generate(this, world, this.m_bigMushgloomGen.get(), &decoRNG, dest[0], 1, dest[2], sbb);
    }
    for _i in 0..64
    {
        let dest: [i32; 3] = troll_cave_main::getCoordsInCave(this, &decoRNG);
        troll_cave_garden::generate(this, world, this.m_bigMushroomGen.get(), &decoRNG, dest[0], 1, dest[2], sbb);
    }
    for _i in 0..128
    {
        let dest: [i32; 3] = troll_cave_main::getCoordsInCave(this, &decoRNG);
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
    return true;
}

// ComponentTFTrollCaveGarden.generate
pub fn generate(
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
    let dy: i32 = this.yAt(y);
    let dz: i32 = this.zAt(x, z);
    if sbb.contains(dx, dy, dz)
    {
        generator.generate(world, random, dx, dy, dz);
    }
}
