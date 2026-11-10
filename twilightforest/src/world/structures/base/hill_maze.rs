// draft from ComponentTFHillMaze.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub const FLOOR_LEVEL: i32 = 1;

pub fn new(i: i32, x: i32, y: i32, z: i32, hsize: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::HillMaze);
    init(&this, i, x, y, z, hsize);
    Rc::new(this)
}

// ComponentTFHillMaze constructor
pub fn init(this: &Piece, i: i32, x: i32, y: i32, z: i32, hsize: i32)
{
    this.m_depth.set(i);
    this.m_hillSize.set(hsize);
    this.setMode(0);
    this.m_box.set(boxFor(
        x,
        y,
        z,
        -this.getRadius(),
        0,
        -this.getRadius(),
        this.getRadius() * 2,
        5,
        this.getRadius() * 2,
        0,
    ));
}

// ComponentTFHillMaze.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    this.fill(world, sbb, 0, 1, 0, this.getDiameter(), 3, this.getDiameter(), blk("air"), blk("air"), false);
    this.fill(
        world,
        sbb,
        0,
        0,
        0,
        this.getDiameter(),
        0,
        this.getDiameter(),
        blk("tf:mazestone"),
        blk("tf:mazestone"),
        false,
    );
    this.fill(
        world,
        sbb,
        0,
        4,
        0,
        this.getDiameter(),
        4,
        this.getDiameter(),
        blk("tf:mazestone"),
        blk("tf:mazestone"),
        true,
    );
    let mut maze: Rc<Maze> = Rc::new(Maze::new(this.getMazeSize(), this.getMazeSize()));
    maze.m_wallBlockID.set(blk("tf:mazestone"));
    maze.m_wallBlockMeta.set(3);
    maze.m_torchRarity.set(0.05);
    maze.setSeed(world.seed() + ((this.m_box.get().m_min[0] * this.m_box.get().m_min[2]) as i64));
    let nrooms: i32 = this.getMazeSize() / 3;
    let mut rcoords: Vec<i32> = vec![0; (nrooms * 2) as usize];
    for i in 0..nrooms
    {
        let mut rx: i32;
        let mut rz: i32;
        loop
        {
            rx = maze.m_rand.nextIntBounded(this.getMazeSize() - 2) + 1;
            rz = maze.m_rand.nextIntBounded(this.getMazeSize() - 2) + 1;
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
    maze.copyToStructure(world, 0, 1, 0, this, sbb);
    this.decorate3x3Rooms(world, &rcoords, sbb);
    return true;
}

// ComponentTFHillMaze.getMazeSize
pub fn getMazeSize(this: &Piece) -> i32
{
    return if this.m_hillSize.get() == 1
    {
        11
    }
    else
    {
        if this.m_hillSize.get() == 2 { 19 } else { 27 }
    };
}

// ComponentTFHillMaze.getRadius
pub fn getRadius(this: &Piece) -> i32 { return this.getMazeSize() * 2; }

// ComponentTFHillMaze.getDiameter
pub fn getDiameter(this: &Piece) -> i32 { return this.getMazeSize() * 4; }

// ComponentTFHillMaze.isNearRoom
pub fn isNearRoom(this: &Piece, dx: i32, dz: i32, rcoords: &[i32]) -> bool
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

// ComponentTFHillMaze.decorate3x3Rooms
pub fn decorate3x3Rooms(this: &Piece, world: &GenWorld, rcoords: &[i32], sbb: StructureBox)
{
    for i in 0..((rcoords.len() as i32) / 2)
    {
        let mut dx: i32 = rcoords[(i * 2) as usize];
        let mut dz: i32 = rcoords[((i * 2) + 1) as usize];
        dx = (dx * 4) + 2;
        dz = (dz * 4) + 2;
        this.decorate3x3Room(world, dx, dz, sbb);
    }
}

// ComponentTFHillMaze.decorate3x3Room
pub fn decorate3x3Room(this: &Piece, world: &GenWorld, x: i32, z: i32, sbb: StructureBox)
{
    let roomRNG: Random = Random::withSeed(world.seed() ^ ((x + z) as i64));
    this.roomSpawner(world, &roomRNG, x, z, 8, sbb);
    this.roomTreasure(world, &roomRNG, x, z, 8, sbb);
    if roomRNG.nextIntBounded(4) == 0
    {
        this.roomTreasure(world, &roomRNG, x, z, 8, sbb);
    }
}

// ComponentTFHillMaze.roomSpawner
pub fn roomSpawner(this: &Piece, world: &GenWorld, random: &Random, x: i32, z: i32, diameter: i32, sbb: StructureBox)
{
    let rx: i32 = (x + random.nextIntBounded(diameter)) - (diameter / 2);
    let rz: i32 = (z + random.nextIntBounded(diameter)) - (diameter / 2);
    let mut var10000: &'static str;
    match random.nextIntBounded(3)
    {
        1 =>
        {
            var10000 = "Skeleton";
        }
        2 =>
        {
            var10000 = "Zombie";
        }
        _ =>
        {
            var10000 = "Spider";
        }
    }
    let mobID: &'static str = var10000;
    this.placeSpawner(world, random, rx, 1, rz, mobID, sbb);
}

// ComponentTFHillMaze.roomTreasure
pub fn roomTreasure(this: &Piece, world: &GenWorld, random: &Random, x: i32, z: i32, diameter: i32, sbb: StructureBox)
{
    let rx: i32 = (x + random.nextIntBounded(diameter)) - (diameter / 2);
    let rz: i32 = (z + random.nextIntBounded(diameter)) - (diameter / 2);
    this.placeTreasure(world, random, rx, 1, rz, Table::LabyrinthRoom, sbb);
}
