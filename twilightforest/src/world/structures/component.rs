use super::legacy;
use super::piece::Piece;
use crate::treasure::Table;
use crate::world::generate::GenWorld;
use b173_api::prelude::*;
use b173_api::terrain::Random;
use b173_api::terrain::structure::StructureBox;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

pub type Pieces = Vec<Rc<Piece>>;

mod limits
{
    // StructureBoundingBox's four argument constructor spans this column height
    pub const COLUMN_TOP: i32 = 512;
    // getAverageGroundLevel tests the clip at this height
    pub const SAMPLE_Y: i32 = 64;
    // getSampledDirtLevel searches down from here and answers this when it finds nothing
    pub const DIRT_TOP: i32 = 90;
    pub const NO_DIRT: i32 = 256;
    // World.setBlock flag sending the change to clients
    pub const NOTIFY: i32 = 2;
}

// a block as the structure code passes it around, a 1.7 or TF name before placing or an id read back from the world
#[derive(Clone, Copy, Debug)]
pub enum Blk
{
    Named(&'static str),
    Id(i32),
}

pub const AIR: Blk = Blk::Named("air");

pub const fn blk(name: &'static str) -> Blk { Blk::Named(name) }

thread_local!
{
    static RESOLVED: RefCell<HashMap<(&'static str, i32), (i32, i32)>> = RefCell::new(HashMap::new());
    static SELECTED: Cell<(Blk, i32)> = const { Cell::new((AIR, 0)) };
    static QUALIFIED: RefCell<HashMap<&'static str, &'static str>> = RefCell::new(HashMap::new());
}

impl Blk
{
    // the beta block id and metadata this block places as
    pub fn resolve(self, meta: i32) -> (i32, i32)
    {
        match self
        {
            Blk::Id(id) => (id, meta),
            Blk::Named(name) => RESOLVED.with(|cache|
            {
                *cache.borrow_mut().entry((name, meta)).or_insert_with(||
                {
                    let (target, meta): legacy::Resolved = legacy::resolve(name, meta);
                    let id: Option<i32> = if name.starts_with("tf:")
                    {
                        Blocks.id(target).ok()
                    }
                    else
                    {
                        Blocks.id(&format!("minecraft:{target}")).ok()
                    };
                    if id.is_none()
                    {
                        warn(&format!("structure block {name} as {target} is not registered"));
                    }
                    (id.unwrap_or(0), meta)
                })
            }),
        }
    }

    pub fn id(self) -> i32 { self.resolve(0).0 }

    // the registry name this block places as, for the generators that take a name
    pub fn qualified(self) -> &'static str
    {
        let Blk::Named(name) = self
        else
        {
            return "minecraft:air";
        };

        let (target, _): legacy::Resolved = legacy::resolve(name, 0);
        if name.starts_with("tf:")
        {
            return target;
        }
        QUALIFIED.with(|names|
        {
            *names.borrow_mut().entry(target).or_insert_with(|| String::leak(format!("minecraft:{target}")))
        })
    }
}

impl PartialEq for Blk
{
    fn eq(&self, other: &Blk) -> bool { self.id() == other.id() }
}

// pieces compared by address, as Java compares component references
pub trait PieceRef
{
    fn address(&self) -> Option<*const Piece>;
}

impl PieceRef for Piece
{
    fn address(&self) -> Option<*const Piece> { Some(self as *const Piece) }
}

impl PieceRef for &Piece
{
    fn address(&self) -> Option<*const Piece> { Some(*self as *const Piece) }
}

impl PieceRef for Rc<Piece>
{
    fn address(&self) -> Option<*const Piece> { Some(Rc::as_ptr(self)) }
}

impl PieceRef for Option<Rc<Piece>>
{
    fn address(&self) -> Option<*const Piece> { self.as_ref().map(Rc::as_ptr) }
}

pub fn same(a: &impl PieceRef, b: &impl PieceRef) -> bool { a.address() == b.address() }

// StructureComponent.findIntersecting
pub fn intersecting(list: &Pieces, check: StructureBox) -> Option<Rc<Piece>>
{
    list.iter().find(|piece| piece.m_box.get().intersects(&check)).cloned()
}

// StructureTFComponent.findIntersectingExcluding
pub fn intersectingExcept(list: &Pieces, check: StructureBox, exclude: &Piece) -> Option<Rc<Piece>>
{
    list.iter().find(|piece| !same(*piece, &exclude) && piece.m_box.get().intersects(&check)).cloned()
}

// StructureTFComponent.getComponentToAddBoundingBox
#[allow(clippy::too_many_arguments)]
pub fn boxFor(
    x: i32,
    y: i32,
    z: i32,
    minX: i32,
    minY: i32,
    minZ: i32,
    maxX: i32,
    maxY: i32,
    maxZ: i32,
    dir: i32,
) -> StructureBox
{
    match dir
    {
        1 => StructureBox::new([x - maxZ + minZ, y + minY, z + minX], [x + minZ, y + maxY + minY, z + maxX + minX]),
        2 => StructureBox::new([x - maxX - minX, y + minY, z - maxZ - minZ], [x - minX, y + maxY + minY, z - minZ]),
        3 => StructureBox::new([x + minZ, y + minY, z - maxX], [x + maxZ + minZ, y + maxY + minY, z + minX]),
        _ => StructureBox::new([x + minX, y + minY, z + minZ], [x + maxX + minX, y + maxY + minY, z + maxZ + minZ]),
    }
}

// StructureTFComponent.getComponentToAddBoundingBox2
#[allow(clippy::too_many_arguments)]
pub fn boxFor2(
    x: i32,
    y: i32,
    z: i32,
    minX: i32,
    minY: i32,
    minZ: i32,
    maxX: i32,
    maxY: i32,
    maxZ: i32,
    dir: i32,
) -> StructureBox
{
    match dir
    {
        1 => StructureBox::new([x - maxZ - minZ, y + minY, z + minX], [x - minZ, y + maxY + minY, z + maxX + minX]),
        2 => StructureBox::new([x - maxX - minX, y + minY, z - maxZ - minZ], [x - minX, y + maxY + minY, z - minZ]),
        3 => StructureBox::new([x + minZ, y + minY, z - maxX], [x + maxZ + minZ, y + maxY + minY, z - minX]),
        _ => StructureBox::new([x + minX, y + minY, z + minZ], [x + maxX + minX, y + maxY + minY, z + maxZ + minZ]),
    }
}

// StructureBoundingBox.getComponentToAddBoundingBox, the vanilla rotation the stronghold pieces use
#[allow(clippy::too_many_arguments)]
pub fn vanillaBox(
    x: i32,
    y: i32,
    z: i32,
    minX: i32,
    minY: i32,
    minZ: i32,
    sizeX: i32,
    sizeY: i32,
    sizeZ: i32,
    dir: i32,
) -> StructureBox
{
    match dir
    {
        1 => StructureBox::new(
            [x - sizeZ + 1 + minZ, y + minY, z + minX],
            [x + minZ, y + sizeY - 1 + minY, z + sizeX - 1 + minX],
        ),
        2 => StructureBox::new(
            [x + minX, y + minY, z - sizeZ + 1 + minZ],
            [x + sizeX - 1 + minX, y + sizeY - 1 + minY, z + minZ],
        ),
        3 => StructureBox::new(
            [x + minZ, y + minY, z + minX],
            [x + sizeZ - 1 + minZ, y + sizeY - 1 + minY, z + sizeX - 1 + minX],
        ),
        _ => StructureBox::new(
            [x + minX, y + minY, z + minZ],
            [x + sizeX - 1 + minX, y + sizeY - 1 + minY, z + sizeZ - 1 + minZ],
        ),
    }
}

// the StructureBoundingBox methods the SDK box lacks
pub trait BoxExt
{
    fn empty() -> StructureBox;
    fn flat(minX: i32, minZ: i32, maxX: i32, maxZ: i32) -> StructureBox;
    fn intersectsXZ(&self, minX: i32, minZ: i32, maxX: i32, maxZ: i32) -> bool;
    fn xSize(&self) -> i32;
    fn ySize(&self) -> i32;
    fn zSize(&self) -> i32;
    fn centerX(&self) -> i32;
    fn centerZ(&self) -> i32;
    fn shift(&mut self, by: [i32; 3]);
    fn expand(&mut self, other: &StructureBox);
}

impl BoxExt for StructureBox
{
    fn empty() -> StructureBox { StructureBox::new([i32::MAX; 3], [i32::MIN; 3]) }

    fn flat(minX: i32, minZ: i32, maxX: i32, maxZ: i32) -> StructureBox
    {
        StructureBox::new([minX, 1, minZ], [maxX, limits::COLUMN_TOP, maxZ])
    }

    fn intersectsXZ(&self, minX: i32, minZ: i32, maxX: i32, maxZ: i32) -> bool
    {
        self.m_max[0] >= minX && self.m_min[0] <= maxX && self.m_max[2] >= minZ && self.m_min[2] <= maxZ
    }

    fn xSize(&self) -> i32 { self.m_max[0] - self.m_min[0] + 1 }

    fn ySize(&self) -> i32 { self.m_max[1] - self.m_min[1] + 1 }

    fn zSize(&self) -> i32 { self.m_max[2] - self.m_min[2] + 1 }

    fn centerX(&self) -> i32 { self.m_min[0] + (self.m_max[0] - self.m_min[0] + 1) / 2 }

    fn centerZ(&self) -> i32 { self.m_min[2] + (self.m_max[2] - self.m_min[2] + 1) / 2 }

    fn shift(&mut self, by: [i32; 3]) { self.offset(by); }

    fn expand(&mut self, other: &StructureBox) { self.include(other); }
}

// the World methods the structure code calls, on the generation world
pub trait WorldExt
{
    fn setBlock(&self, x: i32, y: i32, z: i32, block: Blk, meta: i32, flags: i32);
    fn getBlock(&self, x: i32, y: i32, z: i32) -> Blk;
    fn clear(&self, x: i32, y: i32, z: i32);
    fn seed(&self) -> i64;
    fn liquid(&self, x: i32, y: i32, z: i32) -> bool;
    fn rand(&self) -> &'static Random;
    fn biome(&self, x: i32, z: i32) -> crate::world::biomes::Biome;
}

impl WorldExt for GenWorld
{
    fn setBlock(&self, x: i32, y: i32, z: i32, block: Blk, meta: i32, _flags: i32)
    {
        let (id, meta): (i32, i32) = block.resolve(meta);
        self.setMeta(x, y, z, id, meta);
    }

    fn getBlock(&self, x: i32, y: i32, z: i32) -> Blk { Blk::Id(self.block(x, y, z)) }

    fn clear(&self, x: i32, y: i32, z: i32) { self.set(x, y, z, 0); }

    fn seed(&self) -> i64 { self.m_world.seed() }

    fn liquid(&self, x: i32, y: i32, z: i32) -> bool { self.isLiquid(x, y, z) }

    fn rand(&self) -> &'static Random { worldRandom() }

    fn biome(&self, x: i32, z: i32) -> crate::world::biomes::Biome
    {
        crate::world::biomes::at(self.m_world.seed(), x, z)
    }
}

// StructureTFDecorator, the blocks a tower family builds with
#[derive(Clone, Copy, Debug)]
pub struct Deco
{
    pub m_blockID: Blk,
    pub m_blockMeta: i32,
    pub m_accentID: Blk,
    pub m_accentMeta: i32,
    pub m_stairID: Blk,
    pub m_fenceID: Blk,
    pub m_fenceMeta: i32,
    pub m_pillarID: Blk,
    pub m_pillarMeta: i32,
    pub m_platformID: Blk,
    pub m_platformMeta: i32,
    pub m_floorID: Blk,
    pub m_floorMeta: i32,
    pub m_roofID: Blk,
    pub m_roofMeta: i32,
    pub m_randomBlocks: Stones,
}

impl Default for Deco
{
    fn default() -> Deco
    {
        Deco
        {
            m_blockID: blk("stone"),
            m_blockMeta: 0,
            m_accentID: blk("cobblestone"),
            m_accentMeta: 0,
            m_stairID: AIR,
            m_fenceID: AIR,
            m_fenceMeta: 0,
            m_pillarID: AIR,
            m_pillarMeta: 0,
            m_platformID: AIR,
            m_platformMeta: 0,
            m_floorID: AIR,
            m_floorMeta: 0,
            m_roofID: AIR,
            m_roofMeta: 0,
            m_randomBlocks: Stones::Stronghold,
        }
    }
}

impl Deco
{
    // StructureDecoratorDarkTower
    pub fn dark_tower() -> Deco
    {
        Deco
        {
            m_blockID: blk("tf:towerWood"),
            m_accentID: blk("tf:towerWood"),
            m_accentMeta: 1,
            m_fenceID: blk("fence"),
            m_stairID: blk("spruce_stairs"),
            m_pillarID: blk("tf:towerWood"),
            m_pillarMeta: 1,
            m_platformID: blk("tf:towerWood"),
            m_platformMeta: 1,
            m_randomBlocks: Stones::TowerWoods,
            ..Deco::default()
        }
    }

    // StructureDecoratorIceTower
    pub fn ice_tower() -> Deco
    {
        Deco
        {
            m_blockID: blk("tf:auroraBlock"),
            m_accentID: blk("planks"),
            m_accentMeta: 2,
            m_fenceID: blk("fence"),
            m_stairID: blk("birch_stairs"),
            m_pillarID: blk("tf:auroraPillar"),
            m_platformID: blk("wooden_slab"),
            m_platformMeta: 2,
            m_floorID: blk("planks"),
            m_floorMeta: 2,
            m_randomBlocks: Stones::AuroraBricks,
            ..Deco::default()
        }
    }

    // StructureDecoratorMushroomTower
    pub fn mushroom_tower() -> Deco
    {
        Deco
        {
            m_blockID: blk("red_mushroom_block"),
            m_blockMeta: 10,
            m_accentID: blk("red_mushroom_block"),
            m_accentMeta: 14,
            m_fenceID: blk("fence"),
            m_stairID: blk("spruce_stairs"),
            m_pillarID: blk("red_mushroom_block"),
            m_pillarMeta: 10,
            m_floorID: blk("planks"),
            ..Deco::default()
        }
    }

    // StructureTFDecoratorStronghold
    pub fn decorator_stronghold() -> Deco
    {
        Deco
        {
            m_blockID: blk("tf:underBrick"),
            m_accentID: blk("tf:underBrick"),
            m_accentMeta: 3,
            m_fenceID: blk("cobblestone_wall"),
            m_stairID: blk("stone_brick_stairs"),
            m_pillarID: blk("stonebrick"),
            m_pillarMeta: 1,
            m_platformID: blk("stone_slab"),
            m_platformMeta: 13,
            m_randomBlocks: Stones::KnightStones,
            ..Deco::default()
        }
    }

    // StructureTFDecoratorCastle
    pub fn decorator_castle() -> Deco
    {
        Deco
        {
            m_blockID: blk("tf:castleBlock"),
            m_accentID: blk("quartz_block"),
            m_accentMeta: 1,
            m_roofID: blk("tf:castleBlock"),
            m_roofMeta: 3,
            m_pillarID: blk("quartz_block"),
            m_pillarMeta: 2,
            m_fenceID: blk("fence"),
            m_stairID: blk("quartz_stairs"),
            m_randomBlocks: Stones::CastleBlocks,
            ..Deco::default()
        }
    }
}

// the BlockSelector subclasses, the selection kept per thread as StructureComponent.BlockSelector keeps it per object
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stones
{
    Stronghold,
    TowerWoods,
    AuroraBricks,
    KnightStones,
    CastleBlocks,
}

impl Stones
{
    pub fn stronghold() -> Stones { Stones::Stronghold }

    // BlockSelector.selectBlocks
    pub fn select(self, random: &Random, x: i32, _y: i32, z: i32, wall: bool)
    {
        let picked: (Blk, i32) = if wall
        {
            self.pick(random, x, z)
        }
        else
        {
            (AIR, 0)
        };
        SELECTED.with(|selected| selected.set(picked));
    }

    pub fn block(self) -> Blk { SELECTED.with(|selected| selected.get().0) }

    pub fn meta(self) -> i32 { SELECTED.with(|selected| selected.get().1) }

    fn pick(self, random: &Random, x: i32, z: i32) -> (Blk, i32)
    {
        match self
        {
            Stones::Stronghold =>
            {
                let roll: f32 = random.nextFloat();
                match roll
                {
                    _ if roll < 0.2 => (blk("stonebrick"), 2),
                    _ if roll < 0.5 => (blk("stonebrick"), 1),
                    _ if roll < 0.55 => (blk("monster_egg"), 2),
                    _ => (blk("stonebrick"), 0),
                }
            }
            Stones::TowerWoods =>
            {
                let roll: f32 = random.nextFloat();
                let meta: i32 = match roll
                {
                    _ if roll < 0.1 => 2,
                    _ if roll < 0.2 => 3,
                    _ if roll < 0.225 => 4,
                    _ => 0,
                };
                (blk("tf:towerWood"), meta)
            }
            Stones::AuroraBricks => (blk("tf:auroraBlock"), (x + z).abs() % 16),
            Stones::KnightStones =>
            {
                let roll: f32 = random.nextFloat();
                (
                    blk("tf:underBrick"),
                    if roll < 0.2
                    {
                        2
                    }
                    else if roll < 0.5
                    {
                        1
                    }
                    else
                    {
                        0
                    },
                )
            }
            Stones::CastleBlocks =>
            {
                let roll: f32 = random.nextFloat();
                (
                    blk("tf:castleBlock"),
                    if roll < 0.1
                    {
                        1
                    }
                    else if roll < 0.2
                    {
                        2
                    }
                    else
                    {
                        0
                    },
                )
            }
        }
    }
}

pub fn mob(name: &str) -> &'static str { legacy::mob(name) }

pub fn sin(value: f32) -> f32 { b173_api::terrain::math::sin(value) }

pub fn cos(value: f32) -> f32 { b173_api::terrain::math::cos(value) }

pub fn sqrtF(value: f32) -> f32 { value.sqrt() }

pub fn floorF(value: f32) -> i32 { floor(f64::from(value)) }

pub fn floor(value: f64) -> i32 { b173_api::terrain::math::floor(value) }

// StructureComponent and StructureTFComponent, the helpers every piece builds with
#[allow(clippy::too_many_arguments)]
impl Piece
{
    pub fn bounds(&self) -> StructureBox { self.m_box.get() }

    pub fn depth(&self) -> i32 { self.m_depth.get() }

    pub fn mode(&self) -> i32 { self.m_mode.get() }

    pub fn setMode(&self, mode: i32) { self.m_mode.set(mode); }

    pub fn boxed(&self, edit: impl FnOnce(&mut StructureBox))
    {
        let mut bounds: StructureBox = self.m_box.get();
        edit(&mut bounds);
        self.m_box.set(bounds);
    }

    // StructureTFComponent.getXWithOffsetAsIfRotated, getXWithOffset when turned zero
    pub fn xTurned(&self, x: i32, z: i32, turn: i32) -> i32
    {
        let mode: i32 = self.mode();
        if mode < 0
        {
            return x;
        }
        let bounds: StructureBox = self.m_box.get();
        match (mode + turn) % 4
        {
            0 => bounds.m_min[0] + x,
            1 => bounds.m_max[0] - z,
            2 => bounds.m_max[0] - x,
            _ => bounds.m_min[0] + z,
        }
    }

    pub fn zTurned(&self, x: i32, z: i32, turn: i32) -> i32
    {
        let mode: i32 = self.mode();
        if mode < 0
        {
            return z;
        }
        let bounds: StructureBox = self.m_box.get();
        match (mode + turn) % 4
        {
            0 => bounds.m_min[2] + z,
            1 => bounds.m_min[2] + x,
            2 => bounds.m_max[2] - z,
            _ => bounds.m_max[2] - x,
        }
    }

    pub fn xAt(&self, x: i32, z: i32) -> i32 { self.xTurned(x, z, 0) }

    pub fn zAt(&self, x: i32, z: i32) -> i32 { self.zTurned(x, z, 0) }

    pub fn yAt(&self, y: i32) -> i32
    {
        if self.mode() < 0
        {
            y
        }
        else
        {
            y + self.m_box.get().m_min[1]
        }
    }

    // StructureComponent.placeBlockAtCurrentPosition
    pub fn placeBlock(&self, world: &GenWorld, block: Blk, meta: i32, x: i32, y: i32, z: i32, sbb: StructureBox)
    {
        self.placeTurned(world, block, meta, x, y, z, 0, sbb);
    }

    // StructureComponent.getBlockAtCurrentPosition
    pub fn getBlock(&self, world: &GenWorld, x: i32, y: i32, z: i32, sbb: StructureBox) -> Blk
    {
        self.getTurned(world, x, y, z, 0, sbb)
    }

    pub fn placeTurned(
        &self,
        world: &GenWorld,
        block: Blk,
        meta: i32,
        x: i32,
        y: i32,
        z: i32,
        turn: i32,
        sbb: StructureBox,
    )
    {
        let (wx, wy, wz): (i32, i32, i32) = (self.xTurned(x, z, turn), self.yAt(y), self.zTurned(x, z, turn));
        if sbb.contains(wx, wy, wz)
        {
            world.setBlock(wx, wy, wz, block, meta, limits::NOTIFY);
        }
    }

    pub fn getTurned(&self, world: &GenWorld, x: i32, y: i32, z: i32, turn: i32, sbb: StructureBox) -> Blk
    {
        let (wx, wy, wz): (i32, i32, i32) = (self.xTurned(x, z, turn), self.yAt(y), self.zTurned(x, z, turn));
        if sbb.contains(wx, wy, wz)
        {
            world.getBlock(wx, wy, wz)
        }
        else
        {
            AIR
        }
    }

    fn air(&self, world: &GenWorld, x: i32, y: i32, z: i32, sbb: StructureBox) -> bool
    {
        self.getBlock(world, x, y, z, sbb).id() == 0
    }

    // StructureComponent.fillWithMetadataBlocks
    pub fn fillMeta(
        &self,
        world: &GenWorld,
        sbb: StructureBox,
        minX: i32,
        minY: i32,
        minZ: i32,
        maxX: i32,
        maxY: i32,
        maxZ: i32,
        edge: Blk,
        edgeMeta: i32,
        inside: Blk,
        insideMeta: i32,
        always: bool,
    )
    {
        for y in minY..=maxY
        {
            for x in minX..=maxX
            {
                for z in minZ..=maxZ
                {
                    if !always && self.air(world, x, y, z, sbb)
                    {
                        continue;
                    }
                    let wall: bool = y == minY || y == maxY || x == minX || x == maxX || z == minZ || z == maxZ;
                    if wall
                    {
                        self.placeBlock(world, edge, edgeMeta, x, y, z, sbb);
                    }
                    else
                    {
                        self.placeBlock(world, inside, insideMeta, x, y, z, sbb);
                    }
                }
            }
        }
    }

    // StructureComponent.fillWithBlocks
    pub fn fill(
        &self,
        world: &GenWorld,
        sbb: StructureBox,
        minX: i32,
        minY: i32,
        minZ: i32,
        maxX: i32,
        maxY: i32,
        maxZ: i32,
        edge: Blk,
        inside: Blk,
        always: bool,
    )
    {
        self.fillMeta(world, sbb, minX, minY, minZ, maxX, maxY, maxZ, edge, 0, inside, 0, always);
    }

    // StructureComponent.fillWithAir
    pub fn fillAir(
        &self,
        world: &GenWorld,
        sbb: StructureBox,
        minX: i32,
        minY: i32,
        minZ: i32,
        maxX: i32,
        maxY: i32,
        maxZ: i32,
    )
    {
        for y in minY..=maxY
        {
            for x in minX..=maxX
            {
                for z in minZ..=maxZ
                {
                    self.placeBlock(world, AIR, 0, x, y, z, sbb);
                }
            }
        }
    }

    // StructureComponent.fillWithRandomizedBlocks
    pub fn fillSelected(
        &self,
        world: &GenWorld,
        sbb: StructureBox,
        minX: i32,
        minY: i32,
        minZ: i32,
        maxX: i32,
        maxY: i32,
        maxZ: i32,
        always: bool,
        random: &Random,
        stones: Stones,
    )
    {
        for y in minY..=maxY
        {
            for x in minX..=maxX
            {
                for z in minZ..=maxZ
                {
                    if !always && self.air(world, x, y, z, sbb)
                    {
                        continue;
                    }
                    let wall: bool = y == minY || y == maxY || x == minX || x == maxX || z == minZ || z == maxZ;
                    stones.select(random, x, y, z, wall);
                    self.placeBlock(world, stones.block(), stones.meta(), x, y, z, sbb);
                }
            }
        }
    }

    // StructureComponent.randomlyFillWithBlocks
    pub fn fillRandomly(
        &self,
        world: &GenWorld,
        sbb: StructureBox,
        random: &Random,
        chance: f32,
        minX: i32,
        minY: i32,
        minZ: i32,
        maxX: i32,
        maxY: i32,
        maxZ: i32,
        edge: Blk,
        inside: Blk,
        always: bool,
    )
    {
        for y in minY..=maxY
        {
            for x in minX..=maxX
            {
                for z in minZ..=maxZ
                {
                    if random.nextFloat() > chance || (!always && self.air(world, x, y, z, sbb))
                    {
                        continue;
                    }
                    let wall: bool = y == minY || y == maxY || x == minX || x == maxX || z == minZ || z == maxZ;
                    self.placeBlock(world, if wall { edge } else { inside }, 0, x, y, z, sbb);
                }
            }
        }
    }

    // StructureComponent.randomlyPlaceBlock
    pub fn placeRandomly(
        &self,
        world: &GenWorld,
        sbb: StructureBox,
        random: &Random,
        chance: f32,
        x: i32,
        y: i32,
        z: i32,
        block: Blk,
        meta: i32,
    )
    {
        if random.nextFloat() < chance
        {
            self.placeBlock(world, block, meta, x, y, z, sbb);
        }
    }

    // StructureComponent.fillCurrentPositionBlocksDownwards
    pub fn fillDown(&self, world: &GenWorld, block: Blk, meta: i32, x: i32, y: i32, z: i32, sbb: StructureBox)
    {
        self.fillDownTurned(world, block, meta, x, y, z, 0, sbb);
    }

    // StructureTFComponent.fillToGroundRotated
    pub fn fillDownTurned(
        &self,
        world: &GenWorld,
        block: Blk,
        meta: i32,
        x: i32,
        y: i32,
        z: i32,
        turn: i32,
        sbb: StructureBox,
    )
    {
        let (wx, mut wy, wz): (i32, i32, i32) = (self.xTurned(x, z, turn), self.yAt(y), self.zTurned(x, z, turn));
        if !sbb.contains(wx, wy, wz)
        {
            return;
        }

        while (world.isAir(wx, wy, wz) || world.liquid(wx, wy, wz)) && wy > 1
        {
            world.setBlock(wx, wy, wz, block, meta, limits::NOTIFY);
            wy -= 1;
        }
    }

    // StructureComponent.isLiquidInStructureBoundingBox
    pub fn liquidIn(&self, world: &GenWorld, sbb: StructureBox) -> bool
    {
        let bounds: StructureBox = self.m_box.get();
        let min: [i32; 3] = [
            bounds.m_min[0].max(sbb.m_min[0]) - 1,
            bounds.m_min[1].max(sbb.m_min[1]) - 1,
            bounds.m_min[2].max(sbb.m_min[2]) - 1,
        ];
        let max: [i32; 3] = [
            bounds.m_max[0].min(sbb.m_max[0]) + 1,
            bounds.m_max[1].min(sbb.m_max[1]) + 1,
            bounds.m_max[2].min(sbb.m_max[2]) + 1,
        ];
        for x in min[0]..=max[0]
        {
            for z in min[2]..=max[2]
            {
                if world.liquid(x, min[1], z) || world.liquid(x, max[1], z)
                {
                    return true;
                }
            }
        }
        for x in min[0]..=max[0]
        {
            for y in min[1]..=max[1]
            {
                if world.liquid(x, y, min[2]) || world.liquid(x, y, max[2])
                {
                    return true;
                }
            }
        }
        for z in min[2]..=max[2]
        {
            for y in min[1]..=max[1]
            {
                if world.liquid(min[0], y, z) || world.liquid(max[0], y, z)
                {
                    return true;
                }
            }
        }
        false
    }

    // StructureTFComponent.fillBlocksRotated
    pub fn fillTurned(
        &self,
        world: &GenWorld,
        sbb: StructureBox,
        minX: i32,
        minY: i32,
        minZ: i32,
        maxX: i32,
        maxY: i32,
        maxZ: i32,
        block: Blk,
        meta: i32,
        turn: i32,
    )
    {
        for y in minY..=maxY
        {
            for x in minX..=maxX
            {
                for z in minZ..=maxZ
                {
                    self.placeTurned(world, block, meta, x, y, z, turn, sbb);
                }
            }
        }
    }

    pub fn fillAirTurned(
        &self,
        world: &GenWorld,
        sbb: StructureBox,
        minX: i32,
        minY: i32,
        minZ: i32,
        maxX: i32,
        maxY: i32,
        maxZ: i32,
        turn: i32,
    )
    {
        self.fillTurned(world, sbb, minX, minY, minZ, maxX, maxY, maxZ, AIR, 0, turn);
    }

    // StructureTFComponent.randomlyFillBlocksRotated
    pub fn fillRandomlyTurned(
        &self,
        world: &GenWorld,
        sbb: StructureBox,
        random: &Random,
        chance: f32,
        minX: i32,
        minY: i32,
        minZ: i32,
        maxX: i32,
        maxY: i32,
        maxZ: i32,
        block: Blk,
        meta: i32,
        other: Blk,
        otherMeta: i32,
        turn: i32,
    )
    {
        for y in minY..=maxY
        {
            for x in minX..=maxX
            {
                for z in minZ..=maxZ
                {
                    if random.nextFloat() < chance
                    {
                        self.placeTurned(world, block, meta, x, y, z, turn, sbb);
                    }
                    else
                    {
                        self.placeTurned(world, other, otherMeta, x, y, z, turn, sbb);
                    }
                }
            }
        }
    }

    // StructureTFComponent.placeSpawnerAtCurrentPosition
    pub fn placeSpawner(&self, world: &GenWorld, _random: &Random, x: i32, y: i32, z: i32, mob: &str, sbb: StructureBox)
    {
        self.placeSpawnerRotated(world, x, y, z, 0, mob, sbb);
    }

    pub fn placeSpawnerRotated(&self, world: &GenWorld, x: i32, y: i32, z: i32, turn: i32, mob: &str, sbb: StructureBox)
    {
        let (wx, wy, wz): (i32, i32, i32) = (self.xTurned(x, z, turn), self.yAt(y), self.zTurned(x, z, turn));
        let spawner: i32 = world.id("minecraft:mob_spawner");
        if !sbb.contains(wx, wy, wz) || world.block(wx, wy, wz) == spawner
        {
            return;
        }

        if let Err(error) = world.m_world.setSpawner(BlockPos::new(wx, wy, wz), mob)
        {
            warn(&format!("structure spawner {mob}: {}", error.0));
        }
    }

    // StructureTFComponent.placeTreasureAtCurrentPosition, beta has no trapped chest so every chest is plain
    pub fn placeTreasure(
        &self,
        world: &GenWorld,
        random: &Random,
        x: i32,
        y: i32,
        z: i32,
        table: Table,
        sbb: StructureBox,
    )
    {
        self.placeTreasureTurned(world, random, x, y, z, 0, table, sbb);
    }

    pub fn placeTreasureTrapped(
        &self,
        world: &GenWorld,
        random: &Random,
        x: i32,
        y: i32,
        z: i32,
        table: Table,
        _trapped: bool,
        sbb: StructureBox,
    )
    {
        self.placeTreasureTurned(world, random, x, y, z, 0, table, sbb);
    }

    pub fn placeTreasureRotated(
        &self,
        world: &GenWorld,
        x: i32,
        y: i32,
        z: i32,
        turn: i32,
        table: Table,
        sbb: StructureBox,
    )
    {
        self.placeTreasureTurned(world, &Random::withSeed(0), x, y, z, turn, table, sbb);
    }

    pub fn placeTreasureRotatedTrapped(
        &self,
        world: &GenWorld,
        x: i32,
        y: i32,
        z: i32,
        turn: i32,
        table: Table,
        _trapped: bool,
        sbb: StructureBox,
    )
    {
        self.placeTreasureRotated(world, x, y, z, turn, table, sbb);
    }

    fn placeTreasureTurned(
        &self,
        world: &GenWorld,
        random: &Random,
        x: i32,
        y: i32,
        z: i32,
        turn: i32,
        table: Table,
        sbb: StructureBox,
    )
    {
        let (wx, wy, wz): (i32, i32, i32) = (self.xTurned(x, z, turn), self.yAt(y), self.zTurned(x, z, turn));
        if sbb.contains(wx, wy, wz) && world.block(wx, wy, wz) != world.id("minecraft:chest")
        {
            crate::treasure::place(world, random, table, wx, wy, wz);
        }
    }

    // StructureTFComponent.placeSignAtCurrentPosition, the sign stands without text since the port has no sign text call
    pub fn placeSign(&self, world: &GenWorld, x: i32, y: i32, z: i32, _first: &str, _second: &str, sbb: StructureBox)
    {
        let (wx, wy, wz): (i32, i32, i32) = (self.xAt(x, z), self.yAt(y), self.zAt(x, z));
        if sbb.contains(wx, wy, wz) && world.block(wx, wy, wz) != world.id("minecraft:standing_sign")
        {
            world.setBlock(wx, wy, wz, blk("standing_sign"), self.mode() * 2, limits::NOTIFY);
        }
    }

    // StructureTFComponent.offsetTowerCoords
    pub fn offsetTower(&self, x: i32, y: i32, z: i32, size: i32, dir: i32) -> Vec<i32>
    {
        self.offsetTowerBase(x, y, z, size, dir).to_vec()
    }

    // StructureTFComponent.offsetTowerCCoords, which the castle and troll cave pieces override
    pub fn offsetTowerAt(&self, x: i32, y: i32, z: i32, size: i32, dir: i32) -> [i32; 3]
    {
        self.offsetTowerCCoords(x, y, z, size, dir)
    }

    pub fn offsetTowerBase(&self, x: i32, y: i32, z: i32, size: i32, dir: i32) -> [i32; 3]
    {
        let (dx, dy, dz): (i32, i32, i32) = (self.xAt(x, z), self.yAt(y), self.zAt(x, z));
        match dir
        {
            0 => [dx + 1, dy - 1, dz - size / 2],
            1 => [dx + size / 2, dy - 1, dz + 1],
            2 => [dx - 1, dy - 1, dz + size / 2],
            3 => [dx - size / 2, dy - 1, dz - 1],
            _ => [x, y, z],
        }
    }

    // StructureTFComponent.getStairMeta
    pub fn stairMeta(&self, dir: i32) -> i32 { [0, 2, 1, 3][((self.mode() + dir) % 4) as usize] }

    // StructureTFComponent.getLadderMeta
    pub fn ladderMeta(&self, dir: i32) -> i32 { [4, 2, 5, 3][((self.mode() + dir) % 4) as usize] }

    pub fn ladderMetaTurned(&self, dir: i32, turn: i32) -> i32 { self.ladderMeta(dir + turn) }

    // StructureTFComponent.nullifySkyLight, beta relights generated chunks so the dark pockets come from the walls alone
    pub fn darken(&self, _world: &GenWorld) {}

    pub fn darkenAt(&self, _world: &GenWorld, _sx: i32, _sy: i32, _sz: i32, _dx: i32, _dy: i32, _dz: i32) {}

    // StructureTFComponent.getAverageGroundLevel, which the labyrinth entrance pieces override
    pub fn groundLevel(&self, world: &GenWorld, sbb: StructureBox) -> i32 { self.getAverageGroundLevel(world, sbb) }

    pub fn groundLevelBase(&self, world: &GenWorld, sbb: StructureBox) -> i32
    {
        let bounds: StructureBox = self.m_box.get();
        let sea: i32 = crate::world::terrain::SEA_LEVEL;
        let (mut total, mut count): (i32, i32) = (0, 0);
        for z in bounds.m_min[2]..=bounds.m_max[2]
        {
            for x in bounds.m_min[0]..=bounds.m_max[0]
            {
                if sbb.contains(x, limits::SAMPLE_Y, z)
                {
                    total += world.topSolid(x, z).max(sea);
                    count += 1;
                }
            }
        }
        if count == 0 { -1 } else { total / count }
    }

    // StructureTFComponent.getSampledDirtLevel, the first ground, stone or grass block down the clip's middle
    pub fn dirtLevel(&self, world: &GenWorld, sbb: StructureBox) -> i32
    {
        let (x, z): (i32, i32) = (sbb.centerX(), sbb.centerZ());
        let ground: [i32; 4] = [
            world.id("minecraft:dirt"),
            world.id("minecraft:stone"),
            world.id("minecraft:grass"),
            world.id("minecraft:gravel"),
        ];
        (1..=limits::DIRT_TOP).rev().find(|&y| ground.contains(&world.block(x, y, z))).unwrap_or(limits::NO_DIRT)
    }
}

// a value java may leave null, read where the java code has already checked it
pub trait Nullable<T>
{
    fn piece(&self) -> &T;
}

impl<T> Nullable<T> for Option<T>
{
    fn piece(&self) -> &T { self.as_ref().expect("structure value checked for null before use") }
}

// the Material classes structure code tests blocks against
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Material
{
    Air,
    Grass,
    Ground,
    Rock,
    Wood,
    Other,
}

impl Blk
{
    // Block.getMaterial for the materials the structures ask about
    pub fn material(self) -> Material
    {
        let id: i32 = self.id();
        match id
        {
            0 => Material::Air,
            _ if id == blk("grass").id() => Material::Grass,
            _ if id == blk("dirt").id() || id == blk("gravel").id() => Material::Ground,
            _ if crate::ids::stony(id) => Material::Rock,
            _ if crate::ids::holdsLeaves(id) || id == blk("planks").id() => Material::Wood,
            _ => Material::Other,
        }
    }

    // Block.isReplaceable at the spot, the block read from there
    pub fn isReplaceable(self, world: &GenWorld, x: i32, y: i32, z: i32) -> bool { world.isReplaceable(x, y, z) }

    // Block.canBeReplacedByLeaves, anything that does not fill its space
    pub fn canBeReplacedByLeaves(self, world: &GenWorld, x: i32, y: i32, z: i32) -> bool { !world.isOpaque(x, y, z) }

    // Block.isNormalCube
    pub fn isNormalCube(self, world: &GenWorld, x: i32, y: i32, z: i32) -> bool { world.isOpaque(x, y, z) }

    // Block.canPlaceBlockAt for the fireflies a trunk carries, an empty spot
    pub fn canPlaceAt(self, world: &GenWorld, x: i32, y: i32, z: i32) -> bool { world.isAir(x, y, z) }
}

thread_local!
{
    static WORLD_RANDOM: &'static Random = Box::leak(Box::new(Random::withSeed(0)));
}

// World.rand, the shared world random structures borrow for trees and loot
pub fn worldRandom() -> &'static Random { WORLD_RANDOM.with(|random| *random) }

// a mob a structure places while it generates, EntityCreature.setHomeArea kept as the boss home
pub fn spawnHome(world: &GenWorld, kind: &str, x: i32, y: i32, z: i32, home: bool)
{
    let center: [f64; 3] = [f64::from(x) + 0.5, f64::from(y), f64::from(z) + 0.5];
    let Some(spawned) = crate::entities::spawnAt(world.m_world, kind, center, 0.0)
    else
    {
        return;
    };

    if home && let Err(error) = spawned.setData(&crate::bosses::Home { m_home: Some([x, y, z]) })
    {
        warn(&format!("structure mob {kind} home: {}", error.0));
    }
}

// EnumDarkTowerDoor
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Door
{
    Vanishing,
    Reappearing,
    Locked,
}

// Facing.offsetsXForSide, offsetsYForSide and offsetsZForSide by side
pub const FACING: [[i32; 3]; 6] = [[0, -1, 0], [0, 1, 0], [0, 0, -1], [0, 0, 1], [-1, 0, 0], [1, 0, 0]];
