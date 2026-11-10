// draft from ComponentTFMinotaurMaze.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub const FLOOR_LEVEL: i32 = 1;

pub fn new7(index: i32, x: i32, y: i32, z: i32, entranceX: i32, entranceZ: i32, level: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::MinotaurMaze);
    init7(&this, index, x, y, z, entranceX, entranceZ, level);
    Rc::new(this)
}

pub fn new(index: i32, x: i32, y: i32, z: i32, level: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::MinotaurMaze);
    init(&this, index, x, y, z, level);
    Rc::new(this)
}

// ComponentTFMinotaurMaze constructor
pub fn init7(this: &Piece, index: i32, x: i32, y: i32, z: i32, entranceX: i32, entranceZ: i32, level: i32)
{
    this.m_depth.set(index);
    this.setMode(0);
    this.m_level.set(level);
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
    *this.m_maze.borrow_mut() = Rc::new(Maze::new(this.getMazeSize(), this.getMazeSize()));
    minotaur_maze::setFixedMazeSeed(this);
    let nrooms: i32 = 7;
    *this.m_rcoords.borrow_mut() = vec![0; (nrooms * 2) as usize];
    minotaur_maze::addRoomsToMaze(this, entranceX, entranceZ, nrooms);
    this.m_maze.borrow().clone().generateRecursiveBacktracker(0, 0);
}

// ComponentTFMinotaurMaze constructor
pub fn init(this: &Piece, index: i32, x: i32, y: i32, z: i32, level: i32)
{
    init7(this, index, x, y, z, 11, 11, level);
}

// ComponentTFMinotaurMaze.addRoomsToMaze
pub fn addRoomsToMaze(this: &Piece, entranceX: i32, entranceZ: i32, nrooms: i32)
{
    this.m_rcoords.borrow_mut()[0] = entranceX;
    this.m_rcoords.borrow_mut()[1] = entranceZ;
    this.m_maze.borrow().clone().carveRoom1(entranceX, entranceZ);
    for i in 1..nrooms
    {
        let mut rx: i32;
        let mut rz: i32;
        loop
        {
            rx = this.m_maze.borrow().clone().m_rand.nextIntBounded(this.getMazeSize() - 2) + 1;
            rz = this.m_maze.borrow().clone().m_rand.nextIntBounded(this.getMazeSize() - 2) + 1;
            if !(minotaur_maze::isNearRoom4(this, rx, rz, &this.m_rcoords.borrow(), if i == 1 { 7 } else { 4 }))
            {
                break;
            }
        }
        this.m_maze.borrow().clone().carveRoom1(rx, rz);
        this.m_rcoords.borrow_mut()[(i * 2) as usize] = rx;
        this.m_rcoords.borrow_mut()[((i * 2) + 1) as usize] = rz;
    }
}

// ComponentTFMinotaurMaze.setFixedMazeSeed
pub fn setFixedMazeSeed(this: &Piece)
{
    this.m_maze.borrow().clone().setSeed(
        (((this.m_box.get().m_min[0] * 90342903) + (this.m_box.get().m_min[1] * 90342903)) ^ this.m_box.get().m_min[2])
            as i64,
    );
}

// ComponentTFMinotaurMaze.makeRoom
pub fn makeRoom(this: &Piece, random: &Random, i: i32, dx: i32, dz: i32) -> Rc<Piece>
{
    let mut room: Option<Rc<Piece>> = None;
    let worldX: i32 = (this.m_box.get().m_min[0] + (dx * 5)) - 4;
    let worldY: i32 = this.m_box.get().m_min[1];
    let worldZ: i32 = (this.m_box.get().m_min[2] + (dz * 5)) - 4;
    if i == 0
    {
        room = Some(maze_room::new(3 + i, random, worldX, worldY, worldZ));
    }
    else if i == 1
    {
        if this.m_level.get() == 1
        {
            room = Some(maze_room_exit::new(3 + i, random, worldX, worldY, worldZ));
        }
        else
        {
            room = Some(maze_room_boss::new(3 + i, random, worldX, worldY, worldZ));
        }
    }
    else if (i != 2) && (i != 3)
    {
        if i == 4
        {
            if this.m_level.get() == 1
            {
                room = Some(maze_room_fountain::new(3 + i, random, worldX, worldY, worldZ));
            }
            else
            {
                room = Some(maze_room_vault::new(3 + i, random, worldX, worldY, worldZ));
            }
        }
        else
        {
            room = Some(maze_room_spawner_chests::new(3 + i, random, worldX, worldY, worldZ));
        }
    }
    else if this.m_level.get() == 1
    {
        room = Some(maze_room_collapse::new(3 + i, random, worldX, worldY, worldZ));
    }
    else
    {
        room = Some(maze_mush_room::new(3 + i, random, worldX, worldY, worldZ));
    }
    return Rc::clone(room.piece());
}

// ComponentTFMinotaurMaze.decorateDeadEndsCorridors
pub fn decorateDeadEndsCorridors(this: &Piece, random: &Random, list: &mut Pieces)
{
    for x in 0..this.m_maze.borrow().clone().m_width.get()
    {
        for z in 0..this.m_maze.borrow().clone().m_depth.get()
        {
            let mut component: Option<Rc<Piece>> = None;
            if (((!this.m_maze.borrow().clone().isWall(x, z, x - 1, z))
                && this.m_maze.borrow().clone().isWall(x, z, x + 1, z))
                && this.m_maze.borrow().clone().isWall(x, z, x, z - 1))
                && this.m_maze.borrow().clone().isWall(x, z, x, z + 1)
            {
                component = Some(minotaur_maze::makeDeadEnd(this, random, x, z, 3));
            }
            if ((this.m_maze.borrow().clone().isWall(x, z, x - 1, z)
                && (!this.m_maze.borrow().clone().isWall(x, z, x + 1, z)))
                && this.m_maze.borrow().clone().isWall(x, z, x, z - 1))
                && this.m_maze.borrow().clone().isWall(x, z, x, z + 1)
            {
                component = Some(minotaur_maze::makeDeadEnd(this, random, x, z, 1));
            }
            if ((this.m_maze.borrow().clone().isWall(x, z, x - 1, z)
                && this.m_maze.borrow().clone().isWall(x, z, x + 1, z))
                && (!this.m_maze.borrow().clone().isWall(x, z, x, z - 1)))
                && this.m_maze.borrow().clone().isWall(x, z, x, z + 1)
            {
                component = Some(minotaur_maze::makeDeadEnd(this, random, x, z, 0));
            }
            if ((this.m_maze.borrow().clone().isWall(x, z, x - 1, z)
                && this.m_maze.borrow().clone().isWall(x, z, x + 1, z))
                && this.m_maze.borrow().clone().isWall(x, z, x, z - 1))
                && (!this.m_maze.borrow().clone().isWall(x, z, x, z + 1))
            {
                component = Some(minotaur_maze::makeDeadEnd(this, random, x, z, 2));
            }
            if (((((((!this.m_maze.borrow().clone().isWall(x, z, x - 1, z))
                && (!this.m_maze.borrow().clone().isWall(x, z, x + 1, z)))
                && this.m_maze.borrow().clone().isWall(x, z, x, z - 1))
                && this.m_maze.borrow().clone().isWall(x, z, x, z + 1))
                && this.m_maze.borrow().clone().isWall(x - 1, z, x - 1, z - 1))
                && this.m_maze.borrow().clone().isWall(x - 1, z, x - 1, z + 1))
                && this.m_maze.borrow().clone().isWall(x + 1, z, x + 1, z - 1))
                && this.m_maze.borrow().clone().isWall(x + 1, z, x + 1, z + 1)
            {
                component = (minotaur_maze::makeCorridor(this, random, x, z, 1));
            }
            if (((((((!this.m_maze.borrow().clone().isWall(x, z, x, z - 1))
                && (!this.m_maze.borrow().clone().isWall(x, z, x, z + 1)))
                && this.m_maze.borrow().clone().isWall(x, z, x - 1, z))
                && this.m_maze.borrow().clone().isWall(x, z, x + 1, z))
                && this.m_maze.borrow().clone().isWall(x, z - 1, x - 1, z - 1))
                && this.m_maze.borrow().clone().isWall(x, z - 1, x + 1, z - 1))
                && this.m_maze.borrow().clone().isWall(x, z + 1, x - 1, z + 1))
                && this.m_maze.borrow().clone().isWall(x, z + 1, x + 1, z + 1)
            {
                component = (minotaur_maze::makeCorridor(this, random, x, z, 0));
            }
            if component.is_some()
            {
                list.push(Rc::clone(component.piece()));
                component.piece().build(this, list, random);
            }
        }
    }
}

// ComponentTFMinotaurMaze.makeDeadEnd
pub fn makeDeadEnd(this: &Piece, random: &Random, dx: i32, dz: i32, rotation: i32) -> Rc<Piece>
{
    let worldX: i32 = (this.m_box.get().m_min[0] + (dx * 5)) + 1;
    let worldY: i32 = this.m_box.get().m_min[1];
    let worldZ: i32 = (this.m_box.get().m_min[2] + (dz * 5)) + 1;
    let decorationType: i32 = random.nextIntBounded(8);
    let mut var10000: Rc<Piece>;
    match decorationType
    {
        1 =>
        {
            var10000 = maze_dead_end_chest::new(4, worldX, worldY, worldZ, rotation);
        }
        2 =>
        {
            var10000 = maze_dead_end_trapped_chest::new(4, worldX, worldY, worldZ, rotation);
        }
        3 =>
        {
            var10000 = maze_dead_end_torches::new(4, worldX, worldY, worldZ, rotation);
        }
        4 =>
        {
            var10000 = maze_dead_end_fountain::new(4, worldX, worldY, worldZ, rotation);
        }
        5 =>
        {
            var10000 = maze_dead_end_fountain_lava::new(4, worldX, worldY, worldZ, rotation);
        }
        6 =>
        {
            var10000 = maze_dead_end_painting::new(4, worldX, worldY, worldZ, rotation);
        }
        7 =>
        {
            var10000 = if this.m_level.get() == 1
            {
                maze_dead_end_roots::new(4, worldX, worldY, worldZ, rotation)
            }
            else
            {
                maze_dead_end_shrooms::new(4, worldX, worldY, worldZ, rotation)
            };
        }
        _ =>
        {
            var10000 = maze_dead_end::new(4, worldX, worldY, worldZ, rotation);
        }
    }
    return Rc::clone(&var10000);
}

// ComponentTFMinotaurMaze.makeCorridor
pub fn makeCorridor(this: &Piece, random: &Random, dx: i32, dz: i32, rotation: i32) -> Option<Rc<Piece>>
{
    let worldX: i32 = (this.m_box.get().m_min[0] + (dx * 5)) + 1;
    let worldY: i32 = this.m_box.get().m_min[1];
    let worldZ: i32 = (this.m_box.get().m_min[2] + (dz * 5)) + 1;
    let decorationType: i32 = random.nextIntBounded(5);
    let mut var10000: Option<Rc<Piece>>;
    match decorationType
    {
        1 =>
        {
            var10000 = Some(maze_corridor::new(4, worldX, worldY, worldZ, rotation));
        }
        2 =>
        {
            var10000 = Some(maze_corridor_iron_fence::new(4, worldX, worldY, worldZ, rotation));
        }
        3 =>
        {
            var10000 = None;
        }
        4 =>
        {
            var10000 = Some(
                if this.m_level.get() == 1
                {
                    maze_corridor_roots::new(4, worldX, worldY, worldZ, rotation)
                }
                else
                {
                    maze_corridor_shrooms::new(4, worldX, worldY, worldZ, rotation)
                },
            );
        }
        _ =>
        {
            var10000 = None;
        }
    }
    return var10000;
}

// ComponentTFMinotaurMaze.func_74861_a
pub fn build(this: &Piece, structurecomponent: &Piece, list: &mut Pieces, random: &Random)
{
    if this.m_level.get() == 1
    {
        let centerX: i32 = this.m_box.get().m_min[0] + ((this.m_box.get().m_max[0] - this.m_box.get().m_min[0]) / 2);
        let centerZ: i32 = this.m_box.get().m_min[2] + ((this.m_box.get().m_max[2] - this.m_box.get().m_min[2]) / 2);
        let maze: Rc<Piece> = minotaur_maze::new7(
            1,
            centerX,
            this.m_box.get().m_min[1] - 10,
            centerZ,
            this.m_rcoords.borrow()[2],
            this.m_rcoords.borrow()[3],
            2,
        );
        list.push(Rc::clone(&maze));
        maze.build(this, list, random);
    }
    for i in 0..((this.m_rcoords.borrow().len() as i32) / 2)
    {
        let dx: i32 = this.m_rcoords.borrow()[(i * 2) as usize];
        let dz: i32 = this.m_rcoords.borrow()[((i * 2) + 1) as usize];
        let room: Rc<Piece> = minotaur_maze::makeRoom(this, random, i, dx, dz);
        list.push(Rc::clone(&room));
        room.build(this, list, random);
    }
    minotaur_maze::decorateDeadEndsCorridors(this, random, list);
}

// ComponentTFMinotaurMaze.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    if this.m_level.get() == 2
    {
        this.fill(
            world,
            sbb,
            0,
            -1,
            0,
            this.getDiameter() + 2,
            6,
            this.getDiameter() + 2,
            blk("bedrock"),
            blk("air"),
            false,
        );
    }
    this.fillAir(world, sbb, 1, 1, 1, this.getDiameter(), 4, this.getDiameter());
    this.fillMeta(
        world,
        sbb,
        1,
        5,
        1,
        this.getDiameter(),
        5,
        this.getDiameter(),
        blk("tf:mazestone"),
        0,
        blk("stone"),
        0,
        this.m_level.get() == 1,
    );
    this.fillMeta(
        world,
        sbb,
        1,
        0,
        1,
        this.getDiameter(),
        0,
        this.getDiameter(),
        blk("tf:mazestone"),
        6,
        blk("stone"),
        0,
        false,
    );
    this.m_maze.borrow().clone().m_headBlockID.set(blk("tf:mazestone"));
    this.m_maze.borrow().clone().m_headBlockMeta.set(3);
    this.m_maze.borrow().clone().m_wallBlockID.set(blk("tf:mazestone"));
    this.m_maze.borrow().clone().m_wallBlockMeta.set(1);
    this.m_maze.borrow().clone().m_rootBlockID.set(blk("tf:mazestone"));
    this.m_maze.borrow().clone().m_rootBlockMeta.set(3);
    this.m_maze.borrow().clone().m_pillarBlockID.set(blk("tf:mazestone"));
    this.m_maze.borrow().clone().m_pillarBlockMeta.set(2);
    this.m_maze.borrow().clone().m_wallVar0ID.set(blk("tf:mazestone"));
    this.m_maze.borrow().clone().m_wallVar0Meta.set(4);
    this.m_maze.borrow().clone().m_wallVarRarity.set(0.2);
    this.m_maze.borrow().clone().m_torchRarity.set(0.05);
    this.m_maze.borrow().clone().m_tall.set(2);
    this.m_maze.borrow().clone().m_head.set(1);
    this.m_maze.borrow().clone().m_roots.set(1);
    this.m_maze.borrow().clone().m_oddBias.set(4);
    this.m_maze.borrow().clone().copyToStructure(world, 1, 2, 1, this, sbb);
    return true;
}

// ComponentTFMinotaurMaze.getMazeSize
pub fn getMazeSize(this: &Piece) -> i32 { return 22; }

// ComponentTFMinotaurMaze.getRadius
pub fn getRadius(this: &Piece) -> i32 { return ((this.getMazeSize()) as f64 * 2.5) as i32; }

// ComponentTFMinotaurMaze.getDiameter
pub fn getDiameter(this: &Piece) -> i32 { return this.getMazeSize() * 5; }

// ComponentTFMinotaurMaze.isNearRoom
pub fn isNearRoom4(this: &Piece, dx: i32, dz: i32, rcoords: &[i32], range: i32) -> bool
{
    if (dx == 1) && (dz == 1)
    {
        return true;
    }
    for i in 0..((rcoords.len() as i32) / 2)
    {
        let rx: i32 = rcoords[(i * 2) as usize];
        let rz: i32 = rcoords[((i * 2) + 1) as usize];
        if (((rx != 0) || (rz != 0)) && ((dx - rx).abs() < range)) && ((dz - rz).abs() < range)
        {
            return true;
        }
    }
    return false;
}
