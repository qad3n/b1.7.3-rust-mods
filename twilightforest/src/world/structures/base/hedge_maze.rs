// draft from ComponentTFHedgeMaze.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub const MSIZE: i32 = 16;
pub const RADIUS: i32 = ((MSIZE / 2) * 3) + 1;
pub const DIAMETER: i32 = 2 * RADIUS;
pub const FLOOR_LEVEL: i32 = 3;

pub fn new(world: &GenWorld, random: &Random, i: i32, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::HedgeMaze);
    init(&this, world, random, i, x, y, z);
    Rc::new(this)
}

// ComponentTFHedgeMaze constructor
pub fn init(this: &Piece, _world: &GenWorld, _random: &Random, i: i32, x: i32, y: i32, z: i32)
{
    this.m_depth.set(i);
    this.setMode(0);
    this.m_box.set(boxFor(x, y, z, -RADIUS, -3, -RADIUS, RADIUS * 2, 10, RADIUS * 2, 0));
}

// ComponentTFHedgeMaze.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, _random: &Random, sbb: StructureBox) -> bool
{
    let maze: Rc<Maze> = Rc::new(Maze::new(MSIZE, MSIZE));
    maze.m_oddBias.set(2);
    maze.m_torchBlockID.set(blk("tf:firefly"));
    maze.m_wallBlockID.set(blk("tf:hedge"));
    maze.m_wallBlockMeta.set(0);
    maze.m_type.set(4);
    maze.m_tall.set(3);
    maze.m_roots.set(3);
    maze.setSeed(world.seed() + ((this.m_box.get().m_min[0] * this.m_box.get().m_min[2]) as i64));
    for fx in 0..=DIAMETER
    {
        for fz in 0..=DIAMETER
        {
            this.placeBlock(world, blk("grass"), 0, fx, FLOOR_LEVEL - 1, fz, sbb);
        }
    }
    this.placeBlock(world, blk("lit_pumpkin"), 1, 0, FLOOR_LEVEL, 24, sbb);
    this.placeBlock(world, blk("lit_pumpkin"), 1, 0, FLOOR_LEVEL, 29, sbb);
    this.placeBlock(world, blk("lit_pumpkin"), 3, 50, FLOOR_LEVEL, 24, sbb);
    this.placeBlock(world, blk("lit_pumpkin"), 3, 50, FLOOR_LEVEL, 29, sbb);
    this.placeBlock(world, blk("lit_pumpkin"), 2, 24, FLOOR_LEVEL, 0, sbb);
    this.placeBlock(world, blk("lit_pumpkin"), 2, 29, FLOOR_LEVEL, 0, sbb);
    this.placeBlock(world, blk("lit_pumpkin"), 0, 24, FLOOR_LEVEL, 50, sbb);
    this.placeBlock(world, blk("lit_pumpkin"), 0, 29, FLOOR_LEVEL, 50, sbb);
    let nrooms: i32 = MSIZE / 3;
    let mut rcoords: Vec<i32> = vec![0; (nrooms * 2) as usize];
    for i in 0..nrooms
    {
        let mut rx: i32;
        let mut rz: i32;
        loop
        {
            rx = maze.m_rand.nextIntBounded(MSIZE - 2) + 1;
            rz = maze.m_rand.nextIntBounded(MSIZE - 2) + 1;
            if !(this.isNearRoom(rx, rz, &rcoords))
            {
                break;
            }
        }
        maze.carveRoom1(rx, rz);
        rcoords[(i * 2) as usize] = rx;
        rcoords[((i * 2) + 1) as usize] = rz;
    }
    maze.generateRecursiveBacktracker(0, 0);
    maze.add4Exits();
    maze.copyToStructure(world, 1, FLOOR_LEVEL, 1, this, sbb);
    this.decorate3x3Rooms(world, &rcoords, sbb);
    return true;
}

// ComponentTFHedgeMaze.isNearRoom
pub fn isNearRoom(_this: &Piece, dx: i32, dz: i32, rcoords: &[i32]) -> bool
{
    if (dx == 1) && (dz == 1)
    {
        return true;
    }
    for i in 0..((rcoords.len() as i32) / 2)
    {
        let rx: i32 = rcoords[(i * 2) as usize];
        let rz: i32 = rcoords[((i * 2) + 1) as usize];
        if (((rx != 0) || (rz != 0)) && ((dx - rx).abs() < 3)) && ((dz - rz).abs() < 3)
        {
            return true;
        }
    }
    return false;
}

// ComponentTFHedgeMaze.decorate3x3Rooms
pub fn decorate3x3Rooms(this: &Piece, world: &GenWorld, rcoords: &[i32], sbb: StructureBox)
{
    for i in 0..((rcoords.len() as i32) / 2)
    {
        let mut dx: i32 = rcoords[(i * 2) as usize];
        let mut dz: i32 = rcoords[((i * 2) + 1) as usize];
        dx = (dx * 3) + 3;
        dz = (dz * 3) + 3;
        this.decorate3x3Room(world, dx, dz, sbb);
    }
}

// ComponentTFHedgeMaze.decorate3x3Room
pub fn decorate3x3Room(this: &Piece, world: &GenWorld, x: i32, z: i32, sbb: StructureBox)
{
    let roomRNG: Random = Random::withSeed(world.seed() ^ ((x + z) as i64));
    hedge_maze::roomJackO(this, world, &roomRNG, x, z, 8, sbb);
    if roomRNG.nextIntBounded(4) == 0
    {
        hedge_maze::roomJackO(this, world, &roomRNG, x, z, 8, sbb);
    }
    this.roomSpawner(world, &roomRNG, x, z, 8, sbb);
    this.roomTreasure(world, &roomRNG, x, z, 8, sbb);
    if roomRNG.nextIntBounded(4) == 0
    {
        this.roomTreasure(world, &roomRNG, x, z, 8, sbb);
    }
}

// ComponentTFHedgeMaze.roomSpawner
pub fn roomSpawner(this: &Piece, world: &GenWorld, random: &Random, x: i32, z: i32, diameter: i32, sbb: StructureBox)
{
    let rx: i32 = (x + random.nextIntBounded(diameter)) - (diameter / 2);
    let rz: i32 = (z + random.nextIntBounded(diameter)) - (diameter / 2);
    let var10000: &'static str;
    match random.nextIntBounded(3)
    {
        1 =>
        {
            var10000 = mob("Swarm Spider");
        }
        2 =>
        {
            var10000 = mob("Hostile Wolf");
        }
        _ =>
        {
            var10000 = mob("Hedge Spider");
        }
    }
    let mobID: &'static str = var10000;
    this.placeSpawner(world, random, rx, FLOOR_LEVEL, rz, mobID, sbb);
}

// ComponentTFHedgeMaze.roomTreasure
pub fn roomTreasure(this: &Piece, world: &GenWorld, random: &Random, x: i32, z: i32, diameter: i32, sbb: StructureBox)
{
    let rx: i32 = (x + random.nextIntBounded(diameter)) - (diameter / 2);
    let rz: i32 = (z + random.nextIntBounded(diameter)) - (diameter / 2);
    this.placeTreasure(world, random, rx, FLOOR_LEVEL, rz, Table::HedgeMaze, sbb);
}

// ComponentTFHedgeMaze.roomJackO
pub fn roomJackO(this: &Piece, world: &GenWorld, random: &Random, x: i32, z: i32, diameter: i32, sbb: StructureBox)
{
    let rx: i32 = (x + random.nextIntBounded(diameter)) - (diameter / 2);
    let rz: i32 = (z + random.nextIntBounded(diameter)) - (diameter / 2);
    this.placeBlock(world, blk("lit_pumpkin"), random.nextIntBounded(4), rx, FLOOR_LEVEL, rz, sbb);
}
