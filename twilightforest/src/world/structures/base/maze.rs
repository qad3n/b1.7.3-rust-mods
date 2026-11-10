// draft from TFMaze.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub const OUT_OF_BOUNDS: i32 = i32::MIN;
pub const OOB: i32 = i32::MIN;
pub const ROOM: i32 = 5;
pub const DOOR: i32 = 6;

pub struct Maze
{
    pub m_width: Cell<i32>,
    pub m_depth: Cell<i32>,
    pub m_oddBias: Cell<i32>,
    pub m_evenBias: Cell<i32>,
    pub m_tall: Cell<i32>,
    pub m_head: Cell<i32>,
    pub m_roots: Cell<i32>,
    pub m_worldX: Cell<i32>,
    pub m_worldY: Cell<i32>,
    pub m_worldZ: Cell<i32>,
    pub m_type: Cell<i32>,
    pub m_wallBlockID: Cell<Blk>,
    pub m_wallBlockMeta: Cell<i32>,
    pub m_wallVar0ID: Cell<Blk>,
    pub m_wallVar0Meta: Cell<i32>,
    pub m_wallVarRarity: Cell<f32>,
    pub m_headBlockID: Cell<Blk>,
    pub m_headBlockMeta: Cell<i32>,
    pub m_rootBlockID: Cell<Blk>,
    pub m_rootBlockMeta: Cell<i32>,
    pub m_pillarBlockID: Cell<Blk>,
    pub m_pillarBlockMeta: Cell<i32>,
    pub m_doorBlockID: Cell<Blk>,
    pub m_doorBlockMeta: Cell<i32>,
    pub m_doorRarity: Cell<f32>,
    pub m_torchBlockID: Cell<Blk>,
    pub m_torchBlockMeta: Cell<i32>,
    pub m_torchRarity: Cell<f32>,
    pub m_rawWidth: Cell<i32>,
    pub m_rawDepth: Cell<i32>,
    pub m_storage: RefCell<Vec<i32>>,
    pub m_rand: Random,
}

impl Maze
{
    pub fn empty() -> Maze
    {
        Maze
        {
            m_width: Cell::new(0),
            m_depth: Cell::new(0),
            m_oddBias: Cell::new(0),
            m_evenBias: Cell::new(0),
            m_tall: Cell::new(0),
            m_head: Cell::new(0),
            m_roots: Cell::new(0),
            m_worldX: Cell::new(0),
            m_worldY: Cell::new(0),
            m_worldZ: Cell::new(0),
            m_type: Cell::new(0),
            m_wallBlockID: Cell::new(AIR),
            m_wallBlockMeta: Cell::new(0),
            m_wallVar0ID: Cell::new(AIR),
            m_wallVar0Meta: Cell::new(0),
            m_wallVarRarity: Cell::new(0.0),
            m_headBlockID: Cell::new(AIR),
            m_headBlockMeta: Cell::new(0),
            m_rootBlockID: Cell::new(AIR),
            m_rootBlockMeta: Cell::new(0),
            m_pillarBlockID: Cell::new(AIR),
            m_pillarBlockMeta: Cell::new(0),
            m_doorBlockID: Cell::new(AIR),
            m_doorBlockMeta: Cell::new(0),
            m_doorRarity: Cell::new(0.0),
            m_torchBlockID: Cell::new(AIR),
            m_torchBlockMeta: Cell::new(0),
            m_torchRarity: Cell::new(0.0),
            m_rawWidth: Cell::new(0),
            m_rawDepth: Cell::new(0),
            m_storage: RefCell::new(Vec::new()),
            m_rand: Random::withSeed(0),
        }
    }

    pub fn new(cellsWidth: i32, cellsDepth: i32) -> Maze
    {
        let maze: Maze = Maze::empty();
        maze.init(cellsWidth, cellsDepth);
        maze
    }

    // TFMaze constructor
    pub fn init(&self, cellsWidth: i32, cellsDepth: i32)
    {
        self.m_oddBias.set(3);
        self.m_evenBias.set(1);
        self.m_tall.set(3);
        self.m_head.set(0);
        self.m_roots.set(0);
        self.m_wallBlockID.set(blk("tf:mazestone"));
        self.m_wallBlockMeta.set(2);
        self.m_rootBlockID.set(blk("tf:mazestone"));
        self.m_rootBlockMeta.set(0);
        self.m_torchBlockID.set(blk("torch"));
        self.m_pillarBlockID.set(AIR);
        self.m_torchBlockMeta.set(0);
        self.m_torchRarity.set(0.75);
        self.m_doorRarity.set(0.0);
        self.m_width.set(cellsWidth);
        self.m_depth.set(cellsDepth);
        self.m_rawWidth.set((self.m_width.get() * 2) + 1);
        self.m_rawDepth.set((self.m_depth.get() * 2) + 1);
        *self.m_storage.borrow_mut() = vec![0; (self.m_rawWidth.get() * self.m_rawDepth.get()) as usize];
        self.m_rand.setSeed(0);
    }

    // TFMaze.getCell
    pub fn getCell(&self, x: i32, z: i32) -> i32 { return self.getRaw((x * 2) + 1, (z * 2) + 1); }

    // TFMaze.putCell
    pub fn putCell(&self, x: i32, z: i32, value: i32) { self.putRaw((x * 2) + 1, (z * 2) + 1, value); }

    // TFMaze.cellEquals
    pub fn cellEquals(&self, x: i32, z: i32, value: i32) -> bool { return self.getCell(x, z) == value; }

    // TFMaze.getWall
    pub fn getWall(&self, sx: i32, sz: i32, dx: i32, dz: i32) -> i32
    {
        if (dx == (sx + 1)) && (dz == sz)
        {
            return self.getRaw((sx * 2) + 2, (sz * 2) + 1);
        }
        if (dx == (sx - 1)) && (dz == sz)
        {
            return self.getRaw((sx * 2) + 0, (sz * 2) + 1);
        }
        if (dx == sx) && (dz == (sz + 1))
        {
            return self.getRaw((sx * 2) + 1, (sz * 2) + 2);
        }
        if (dx == sx) && (dz == (sz - 1))
        {
            return self.getRaw((sx * 2) + 1, (sz * 2) + 0);
        }
        return i32::MIN;
    }

    // TFMaze.putWall
    pub fn putWall(&self, sx: i32, sz: i32, dx: i32, dz: i32, value: i32)
    {
        if (dx == (sx + 1)) && (dz == sz)
        {
            self.putRaw((sx * 2) + 2, (sz * 2) + 1, value);
        }
        else if (dx == (sx - 1)) && (dz == sz)
        {
            self.putRaw((sx * 2) + 0, (sz * 2) + 1, value);
        }
        else if (dx == sx) && (dz == (sz + 1))
        {
            self.putRaw((sx * 2) + 1, (sz * 2) + 2, value);
        }
        else if (dx == sx) && (dz == (sz - 1))
        {
            self.putRaw((sx * 2) + 1, (sz * 2) + 0, value);
        }
    }

    // TFMaze.isWall
    pub fn isWall(&self, sx: i32, sz: i32, dx: i32, dz: i32) -> bool { return self.getWall(sx, sz, dx, dz) == 0; }

    // TFMaze.putRaw
    pub fn putRaw(&self, rawx: i32, rawz: i32, value: i32)
    {
        if (((rawx >= 0) && (rawx < self.m_rawWidth.get())) && (rawz >= 0)) && (rawz < self.m_rawDepth.get())
        {
            self.m_storage.borrow_mut()[((rawz * self.m_rawWidth.get()) + rawx) as usize] = value;
        }
    }

    // TFMaze.getRaw
    pub fn getRaw(&self, rawx: i32, rawz: i32) -> i32
    {
        return if (((rawx >= 0) && (rawx < self.m_rawWidth.get())) && (rawz >= 0)) && (rawz < self.m_rawDepth.get())
        {
            self.m_storage.borrow()[((rawz * self.m_rawWidth.get()) + rawx) as usize]
        }
        else
        {
            i32::MIN
        };
    }

    // TFMaze.setSeed
    pub fn setSeed(&self, newSeed: i64) { self.m_rand.setSeed(newSeed); }

    // TFMaze.copyToWorld
    pub fn copyToWorld(&self, world: &GenWorld, dx: i32, dy: i32, dz: i32)
    {
        self.m_worldX.set(dx);
        self.m_worldY.set(dy);
        self.m_worldZ.set(dz);
        for x in 0..self.m_rawWidth.get()
        {
            for z in 0..self.m_rawDepth.get()
            {
                if self.getRaw(x, z) == 0
                {
                    let mdx: i32 = dx + ((x / 2) * (self.m_evenBias.get() + self.m_oddBias.get()));
                    let mdz: i32 = dz + ((z / 2) * (self.m_evenBias.get() + self.m_oddBias.get()));
                    if self.isEven(x) && self.isEven(z)
                    {
                        if (self.m_type.get() == 4) && self.shouldTree(x, z)
                        {
                            Gen::Canopy.generate(world, &self.m_rand, mdx, dy, mdz);
                        }
                        else
                        {
                            for y in 0..self.m_head.get()
                            {
                                self.putHeadBlock(world, mdx, (dy + self.m_tall.get()) + y, mdz);
                            }
                            for y in 0..self.m_tall.get()
                            {
                                self.putWallBlock(world, mdx, dy + y, mdz);
                            }
                            for y in 1..=self.m_roots.get()
                            {
                                self.putRootBlock(world, mdx, dy - y, mdz);
                            }
                        }
                    }
                    if self.isEven(x) && (!self.isEven(z))
                    {
                        for even in 0..self.m_evenBias.get()
                        {
                            for odd in 1..=self.m_oddBias.get()
                            {
                                for y in 0..self.m_head.get()
                                {
                                    self.putHeadBlock(world, mdx + even, (dy + self.m_tall.get()) + y, mdz + odd);
                                }
                                for y in 0..self.m_tall.get()
                                {
                                    self.putWallBlock(world, mdx + even, dy + y, mdz + odd);
                                }
                                for y in 1..=self.m_roots.get()
                                {
                                    self.putRootBlock(world, mdx + even, dy - y, mdz + odd);
                                }
                            }
                        }
                    }
                    if (!self.isEven(x)) && self.isEven(z)
                    {
                        for even in 0..self.m_evenBias.get()
                        {
                            for odd in 1..=self.m_oddBias.get()
                            {
                                for y in 0..self.m_head.get()
                                {
                                    self.putHeadBlock(world, mdx + odd, (dy + self.m_tall.get()) + y, mdz + even);
                                }
                                for y in 0..self.m_tall.get()
                                {
                                    self.putWallBlock(world, mdx + odd, dy + y, mdz + even);
                                }
                                for y in 1..=self.m_roots.get()
                                {
                                    self.putRootBlock(world, mdx + odd, dy - y, mdz + even);
                                }
                            }
                        }
                    }
                }
            }
        }
        self.placeTorches(world);
    }

    // TFMaze.carveToWorld
    pub fn carveToWorld(&self, world: &GenWorld, dx: i32, dy: i32, dz: i32)
    {
        self.m_worldX.set(dx);
        self.m_worldY.set(dy);
        self.m_worldZ.set(dz);
        for x in 0..self.m_rawWidth.get()
        {
            for z in 0..self.m_rawDepth.get()
            {
                if self.getRaw(x, z) != 0
                {
                    let mdx: i32 = dx + ((x / 2) * (self.m_evenBias.get() + self.m_oddBias.get()));
                    let mdz: i32 = dz + ((z / 2) * (self.m_evenBias.get() + self.m_oddBias.get()));
                    if self.isEven(x) && self.isEven(z)
                    {
                        for y in 0..self.m_tall.get()
                        {
                            self.carveBlock(world, mdx, dy + y, mdz);
                        }
                    }
                    else if self.isEven(x) && (!self.isEven(z))
                    {
                        for i in 1..=self.m_oddBias.get()
                        {
                            for y in 0..self.m_tall.get()
                            {
                                self.carveBlock(world, mdx, dy + y, mdz + i);
                            }
                        }
                    }
                    else if (!self.isEven(x)) && self.isEven(z)
                    {
                        for i in 1..=self.m_oddBias.get()
                        {
                            for y in 0..self.m_tall.get()
                            {
                                self.carveBlock(world, mdx + i, dy + y, mdz);
                            }
                        }
                    }
                    else if (!self.isEven(x)) && (!self.isEven(z))
                    {
                        for mx in 1..=self.m_oddBias.get()
                        {
                            for mz in 1..=self.m_oddBias.get()
                            {
                                for y in 0..self.m_tall.get()
                                {
                                    self.carveBlock(world, mdx + mx, dy + y, mdz + mz);
                                }
                            }
                        }
                    }
                }
            }
        }
        self.placeTorches(world);
    }

    // TFMaze.copyToStructure
    pub fn copyToStructure(&self, world: &GenWorld, dx: i32, dy: i32, dz: i32, component: &Piece, sbb: StructureBox)
    {
        for x in 0..self.m_rawWidth.get()
        {
            for z in 0..self.m_rawDepth.get()
            {
                if self.getRaw(x, z) == 0
                {
                    let mut mdx: i32 = dx + ((x / 2) * (self.m_evenBias.get() + self.m_oddBias.get()));
                    let mut mdz: i32 = dz + ((z / 2) * (self.m_evenBias.get() + self.m_oddBias.get()));
                    if self.m_evenBias.get() > 1
                    {
                        mdx -= 1;
                        mdz -= 1;
                    }
                    if self.isEven(x) && self.isEven(z)
                    {
                        if (self.m_type.get() == 4) && self.shouldTree(x, z)
                        {
                            self.putCanopyTree(world, mdx, dy, mdz, &component, sbb);
                        }
                        else
                        {
                            for even in 0..self.m_evenBias.get()
                            {
                                for even2 in 0..self.m_evenBias.get()
                                {
                                    for y in 0..self.m_head.get()
                                    {
                                        self.putHeadBlock6(
                                            world,
                                            mdx + even,
                                            (dy + self.m_tall.get()) + y,
                                            mdz + even2,
                                            &component,
                                            sbb,
                                        );
                                    }
                                    for y in 0..self.m_tall.get()
                                    {
                                        if self.shouldPillar(x, z)
                                        {
                                            self.putPillarBlock(
                                                world,
                                                mdx + even,
                                                dy + y,
                                                mdz + even2,
                                                &component,
                                                sbb,
                                            );
                                        }
                                        else
                                        {
                                            self.putWallBlock6(world, mdx + even, dy + y, mdz + even2, &component, sbb);
                                        }
                                    }
                                    for y in 1..=self.m_roots.get()
                                    {
                                        self.putRootBlock6(world, mdx + even, dy - y, mdz + even2, &component, sbb);
                                    }
                                }
                            }
                        }
                    }
                    if self.isEven(x) && (!self.isEven(z))
                    {
                        for even in 0..self.m_evenBias.get()
                        {
                            for odd in 1..=self.m_oddBias.get()
                            {
                                self.makeWallThing(world, dy, &component, sbb, mdx, mdz, even, odd);
                            }
                        }
                    }
                    if (!self.isEven(x)) && self.isEven(z)
                    {
                        for even in 0..self.m_evenBias.get()
                        {
                            for odd in 1..=self.m_oddBias.get()
                            {
                                self.makeWallThing(world, dy, &component, sbb, mdx, mdz, odd, even);
                            }
                        }
                    }
                }
                else if self.getRaw(x, z) == 6
                {
                    let mut mdx: i32 = dx + ((x / 2) * (self.m_evenBias.get() + self.m_oddBias.get()));
                    let mut mdz: i32 = dz + ((z / 2) * (self.m_evenBias.get() + self.m_oddBias.get()));
                    if self.m_evenBias.get() > 1
                    {
                        mdx -= 1;
                        mdz -= 1;
                    }
                    if self.isEven(x) && (!self.isEven(z))
                    {
                        for even in 0..self.m_evenBias.get()
                        {
                            for odd in 1..=self.m_oddBias.get()
                            {
                                for y in 0..self.m_head.get()
                                {
                                    self.putHeadBlock6(
                                        world,
                                        mdx + even,
                                        (dy + self.m_tall.get()) + y,
                                        mdz + odd,
                                        &component,
                                        sbb,
                                    );
                                }
                                for y in 0..self.m_tall.get()
                                {
                                    self.putDoorBlock(world, mdx + even, dy + y, mdz + odd, &component, sbb);
                                }
                                for y in 1..=self.m_roots.get()
                                {
                                    self.putRootBlock6(world, mdx + even, dy - y, mdz + odd, &component, sbb);
                                }
                            }
                        }
                    }
                    if (!self.isEven(x)) && self.isEven(z)
                    {
                        for even in 0..self.m_evenBias.get()
                        {
                            for odd in 1..=self.m_oddBias.get()
                            {
                                for y in 0..self.m_head.get()
                                {
                                    self.putHeadBlock6(
                                        world,
                                        mdx + odd,
                                        (dy + self.m_tall.get()) + y,
                                        mdz + even,
                                        &component,
                                        sbb,
                                    );
                                }
                                for y in 0..self.m_tall.get()
                                {
                                    self.putDoorBlock(world, mdx + odd, dy + y, mdz + even, &component, sbb);
                                }
                                for y in 1..=self.m_roots.get()
                                {
                                    self.putRootBlock6(world, mdx + odd, dy - y, mdz + even, &component, sbb);
                                }
                            }
                        }
                    }
                }
            }
        }
        for x in 0..self.m_rawWidth.get()
        {
            for z in 0..self.m_rawDepth.get()
            {
                if self.getRaw(x, z) == 0
                {
                    let mut mdx: i32 = dx + ((x / 2) * (self.m_evenBias.get() + self.m_oddBias.get()));
                    let mdy: i32 = dy + 1;
                    let mut mdz: i32 = dz + ((z / 2) * (self.m_evenBias.get() + self.m_oddBias.get()));
                    if ((self.isEven(x) && self.isEven(z)) && self.shouldTorch(x, z))
                        && (component.getBlock(world, mdx, mdy, mdz, sbb) == self.m_wallBlockID.get())
                    {
                        component.placeBlock(
                            world,
                            self.m_torchBlockID.get(),
                            self.m_torchBlockMeta.get(),
                            mdx,
                            mdy,
                            mdz,
                            sbb,
                        );
                    }
                }
            }
        }
    }

    // TFMaze.makeWallThing
    pub fn makeWallThing(
        &self,
        world: &GenWorld,
        dy: i32,
        component: &Piece,
        sbb: StructureBox,
        mdx: i32,
        mdz: i32,
        even: i32,
        odd: i32,
    )
    {
        for y in 0..self.m_head.get()
        {
            self.putHeadBlock6(world, mdx + even, (dy + self.m_tall.get()) + y, mdz + odd, &component, sbb);
        }
        for y in 0..self.m_tall.get()
        {
            self.putWallBlock6(world, mdx + even, dy + y, mdz + odd, &component, sbb);
        }
        for y in 1..=self.m_roots.get()
        {
            self.putRootBlock6(world, mdx + even, dy - y, mdz + odd, &component, sbb);
        }
    }

    // TFMaze.putPillarBlock
    pub fn putPillarBlock(&self, world: &GenWorld, x: i32, y: i32, z: i32, component: &Piece, sbb: StructureBox)
    {
        component.placeBlock(world, self.m_pillarBlockID.get(), self.m_pillarBlockMeta.get(), x, y, z, sbb);
    }

    // TFMaze.putWallBlock
    pub fn putWallBlock(&self, world: &GenWorld, x: i32, y: i32, z: i32)
    {
        world.setBlock(x, y, z, self.m_wallBlockID.get(), self.m_wallBlockMeta.get(), 2);
    }

    // TFMaze.putWallBlock
    pub fn putWallBlock6(&self, world: &GenWorld, x: i32, y: i32, z: i32, component: &Piece, sbb: StructureBox)
    {
        if (self.m_wallVarRarity.get() > 0.0) && (self.m_rand.nextFloat() < self.m_wallVarRarity.get())
        {
            component.placeBlock(world, self.m_wallVar0ID.get(), self.m_wallVar0Meta.get(), x, y, z, sbb);
        }
        else
        {
            component.placeBlock(world, self.m_wallBlockID.get(), self.m_wallBlockMeta.get(), x, y, z, sbb);
        }
    }

    // TFMaze.putDoorBlock
    pub fn putDoorBlock(&self, world: &GenWorld, x: i32, y: i32, z: i32, component: &Piece, sbb: StructureBox)
    {
        component.placeBlock(world, self.m_doorBlockID.get(), self.m_doorBlockMeta.get(), x, y, z, sbb);
    }

    // TFMaze.carveBlock
    pub fn carveBlock(&self, world: &GenWorld, x: i32, y: i32, z: i32) { world.setBlock(x, y, z, blk("air"), 0, 2); }

    // TFMaze.putHeadBlock
    pub fn putHeadBlock(&self, world: &GenWorld, x: i32, y: i32, z: i32)
    {
        world.setBlock(x, y, z, self.m_headBlockID.get(), self.m_headBlockMeta.get(), 2);
    }

    // TFMaze.putHeadBlock
    pub fn putHeadBlock6(&self, world: &GenWorld, x: i32, y: i32, z: i32, component: &Piece, sbb: StructureBox)
    {
        component.placeBlock(world, self.m_headBlockID.get(), self.m_headBlockMeta.get(), x, y, z, sbb);
    }

    // TFMaze.putRootBlock
    pub fn putRootBlock(&self, world: &GenWorld, x: i32, y: i32, z: i32)
    {
        world.setBlock(x, y, z, self.m_rootBlockID.get(), self.m_rootBlockMeta.get(), 2);
    }

    // TFMaze.putRootBlock
    pub fn putRootBlock6(&self, world: &GenWorld, x: i32, y: i32, z: i32, component: &Piece, sbb: StructureBox)
    {
        component.placeBlock(world, self.m_rootBlockID.get(), self.m_rootBlockMeta.get(), x, y, z, sbb);
    }

    // TFMaze.putCanopyTree
    pub fn putCanopyTree(&self, world: &GenWorld, x: i32, y: i32, z: i32, component: &Piece, sbb: StructureBox)
    {
        let wx: i32 = component.xAt(x, z);
        let wy: i32 = component.yAt(y);
        let wz: i32 = component.zAt(x, z);
        if sbb.contains(wx, wy, wz)
        {
            Gen::Canopy.generate(world, &self.m_rand, wx, wy, wz);
        }
    }

    // TFMaze.isEven
    pub fn isEven(&self, n: i32) -> bool { return (n % 2) == 0; }

    // TFMaze.placeTorches
    pub fn placeTorches(&self, world: &GenWorld)
    {
        let torchHeight: i32 = 1;
        for x in 0..self.m_rawWidth.get()
        {
            for z in 0..self.m_rawDepth.get()
            {
                if self.getRaw(x, z) == 0
                {
                    let mdx: i32 = self.m_worldX.get() + ((x / 2) * (self.m_evenBias.get() + self.m_oddBias.get()));
                    let mdy: i32 = self.m_worldY.get() + torchHeight;
                    let mdz: i32 = self.m_worldZ.get() + ((z / 2) * (self.m_evenBias.get() + self.m_oddBias.get()));
                    if ((self.isEven(x) && self.isEven(z)) && self.shouldTorch(x, z))
                        && (world.getBlock(mdx, mdy, mdz) == self.m_wallBlockID.get())
                    {
                        world.setBlock(mdx, mdy, mdz, self.m_torchBlockID.get(), self.m_torchBlockMeta.get(), 2);
                    }
                }
            }
        }
    }

    // TFMaze.shouldTorch
    pub fn shouldTorch(&self, rx: i32, rz: i32) -> bool
    {
        if (((self.getRaw(rx + 1, rz) != i32::MIN) && (self.getRaw(rx - 1, rz) != i32::MIN))
            && (self.getRaw(rx, rz + 1) != i32::MIN))
            && (self.getRaw(rx, rz - 1) != i32::MIN)
        {
            return if ((self.getRaw(rx + 1, rz) != 0) || (self.getRaw(rx - 1, rz) != 0))
                && ((self.getRaw(rx, rz + 1) != 0) || (self.getRaw(rx, rz - 1) != 0))
            {
                self.m_rand.nextFloat() <= self.m_torchRarity.get()
            }
            else
            {
                false
            };
        }
        else
        {
            return false;
        }
    }

    // TFMaze.shouldPillar
    pub fn shouldPillar(&self, rx: i32, rz: i32) -> bool
    {
        if self.m_pillarBlockID.get().id() == 0
        {
            return false;
        }
        else
        {
            return if (((self.getRaw(rx + 1, rz) != i32::MIN) && (self.getRaw(rx - 1, rz) != i32::MIN))
                && (self.getRaw(rx, rz + 1) != i32::MIN))
                && (self.getRaw(rx, rz - 1) != i32::MIN)
            {
                ((self.getRaw(rx + 1, rz) != 0) || (self.getRaw(rx - 1, rz) != 0))
                    && ((self.getRaw(rx, rz + 1) != 0) || (self.getRaw(rx, rz - 1) != 0))
            }
            else
            {
                false
            };
        }
    }

    // TFMaze.shouldTree
    pub fn shouldTree(&self, rx: i32, rz: i32) -> bool
    {
        if ((rx != 0) && (rx != (self.m_rawWidth.get() - 1)))
            || ((self.getRaw(rx, rz + 1) == 0) && (self.getRaw(rx, rz - 1) == 0))
        {
            return if ((rz != 0) && (rz != (self.m_rawDepth.get() - 1)))
                || ((self.getRaw(rx + 1, rz) == 0) && (self.getRaw(rx - 1, rz) == 0))
            {
                self.m_rand.nextIntBounded(50) == 0
            }
            else
            {
                true
            };
        }
        else
        {
            return true;
        }
    }

    // TFMaze.getWorldX
    pub fn getWorldX(&self, x: i32) -> i32
    {
        return (self.m_worldX.get() + (x * (self.m_evenBias.get() + self.m_oddBias.get()))) + 1;
    }

    // TFMaze.getWorldZ
    pub fn getWorldZ(&self, z: i32) -> i32
    {
        return (self.m_worldZ.get() + (z * (self.m_evenBias.get() + self.m_oddBias.get()))) + 1;
    }

    // TFMaze.carveRoom0
    pub fn carveRoom0(&self, cx: i32, cz: i32)
    {
        self.putCell(cx, cz, 5);
        self.putCell(cx + 1, cz, 5);
        self.putWall(cx, cz, cx + 1, cz, 5);
        self.putCell(cx - 1, cz, 5);
        self.putWall(cx, cz, cx - 1, cz, 5);
        self.putCell(cx, cz + 1, 5);
        self.putWall(cx, cz, cx, cz + 1, 5);
        self.putCell(cx, cz - 1, 5);
        self.putWall(cx, cz, cx, cz - 1, 5);
    }

    // TFMaze.carveRoom1
    pub fn carveRoom1(&self, cx: i32, cz: i32)
    {
        let rx: i32 = (cx * 2) + 1;
        let rz: i32 = (cz * 2) + 1;
        for i in -2..=2
        {
            for j in -2..=2
            {
                self.putRaw(rx + i, rz + j, 5);
            }
        }
        self.putCell(rx, rz + 1, 0);
        self.putCell(rx, rz - 1, 0);
        self.putCell(rx + 1, rz, 0);
        self.putCell(rx - 1, rz, 0);
        if self.getRaw(rx, rz + 4) != i32::MIN
        {
            self.putRaw(rx, rz + 3, 5);
        }
        if self.getRaw(rx, rz - 4) != i32::MIN
        {
            self.putRaw(rx, rz - 3, 5);
        }
        if self.getRaw(rx + 4, rz) != i32::MIN
        {
            self.putRaw(rx + 3, rz, 5);
        }
        if self.getRaw(rx - 4, rz) != i32::MIN
        {
            self.putRaw(rx - 3, rz, 5);
        }
    }

    // TFMaze.add4Exits
    pub fn add4Exits(&self)
    {
        let hx: i32 = (self.m_rawWidth.get() / 2) + 1;
        let hz: i32 = (self.m_rawDepth.get() / 2) + 1;
        self.putRaw(hx, 0, 5);
        self.putRaw(hx, self.m_rawDepth.get() - 1, 5);
        self.putRaw(0, hz, 5);
        self.putRaw(self.m_rawWidth.get() - 1, hz, 5);
    }

    // TFMaze.generateRecursiveBacktracker
    pub fn generateRecursiveBacktracker(&self, sx: i32, sz: i32) { self.rbGen(sx, sz); }

    // TFMaze.rbGen
    pub fn rbGen(&self, sx: i32, sz: i32)
    {
        self.putCell(sx, sz, 1);
        let mut unvisited: i32 = 0;
        if self.cellEquals(sx + 1, sz, 0)
        {
            unvisited += 1;
        }
        if self.cellEquals(sx - 1, sz, 0)
        {
            unvisited += 1;
        }
        if self.cellEquals(sx, sz + 1, 0)
        {
            unvisited += 1;
        }
        if self.cellEquals(sx, sz - 1, 0)
        {
            unvisited += 1;
        }
        if unvisited != 0
        {
            let mut rn: i32 = self.m_rand.nextIntBounded(unvisited);
            let mut dz: i32 = 0;
            let mut dx: i32 = 0;
            if self.cellEquals(sx + 1, sz, 0)
            {
                if rn == 0
                {
                    dx = sx + 1;
                    dz = sz;
                }
                rn -= 1;
            }
            if self.cellEquals(sx - 1, sz, 0)
            {
                if rn == 0
                {
                    dx = sx - 1;
                    dz = sz;
                }
                rn -= 1;
            }
            if self.cellEquals(sx, sz + 1, 0)
            {
                if rn == 0
                {
                    dx = sx;
                    dz = sz + 1;
                }
                rn -= 1;
            }
            if self.cellEquals(sx, sz - 1, 0) && (rn == 0)
            {
                dx = sx;
                dz = sz - 1;
            }
            if self.m_rand.nextFloat() <= self.m_doorRarity.get()
            {
                self.putWall(sx, sz, dx, dz, 6);
            }
            else
            {
                self.putWall(sx, sz, dx, dz, 2);
            }
            self.rbGen(dx, dz);
            self.rbGen(sx, sz);
            self.rbGen(sx, sz);
        }
    }
}
