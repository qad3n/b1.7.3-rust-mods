use crate::ids::{Palette, block, blockId};
use crate::world::DIMENSION_ID;
use b173_api::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

const GLOWSTONE: &str = "minecraft:glowstone";
const FIRE: &str = "minecraft:fire";
const WATER_BUCKET: &str = "minecraft:water_bucket";
const BUCKET: &str = "minecraft:bucket";
const COBBLESTONE: &str = "minecraft:cobblestone";
const PORTALS_KEY: &str = "portals";

mod frame
{
    pub const MAX_SIZE: i32 = 21;
    pub const MIN_WIDTH: i32 = 2;
    pub const MIN_HEIGHT: i32 = 3;
    pub const EDGE_SCAN: i32 = 22;
}

mod travel
{
    // PlayerAether portal counter limit before the player is sent
    pub const TICKS: i32 = 80;
    pub const SEARCH_RADIUS: i32 = 128;
    // a fresh arrival waits above the world for one tick while the portal is built under it
    pub const HOLD_Y: f64 = 120.0;
    pub const FALLBACK_Y: i32 = 64;
    pub const OVERWORLD: i32 = 0;
}

// interior of a built portal, two wide and three tall, framed by glowstone
mod build
{
    pub const WIDTH: i32 = 2;
    pub const HEIGHT: i32 = 3;
}

// Direction.offsetX and offsetZ for the BlockPortal axis tables, left then right
const AXES: [[[i32; 2]; 2]; 2] = [[[1, 0], [-1, 0]], [[0, -1], [0, 1]]];

fn isEmpty(world: World, pos: BlockPos) -> bool
{
    let id: i32 = world.block(pos);
    id == 0 || id == blockId(FIRE) || id == blockId(block::AETHER_PORTAL)
}

fn isFrame(world: World, pos: BlockPos) -> bool { world.block(pos) == blockId(GLOWSTONE) }

fn step(pos: BlockPos, direction: [i32; 2], amount: i32) -> BlockPos
{
    pos.offset(direction[0] * amount, 0, direction[1] * amount)
}

// AetherPortalSize, a glowstone frame measured along one axis
struct PortalSize
{
    m_right: [i32; 2],
    m_bottomLeft: BlockPos,
    m_width: i32,
    m_height: i32,
    m_portals: i32,
}

impl PortalSize
{
    fn measure(world: World, pos: BlockPos, axis: usize) -> Option<PortalSize>
    {
        let [left, right] = AXES[axis];
        let mut pos: BlockPos = pos;
        let lowest: i32 = pos.y - frame::MAX_SIZE;
        while pos.y > lowest && pos.y > 0 && isEmpty(world, pos.offset(0, -1, 0))
        {
            pos = pos.offset(0, -1, 0);
        }
        let toEdge: i32 = Self::distanceToEdge(world, pos, left) - 1;
        if toEdge < 0
        {
            return None;
        }
        let bottomLeft: BlockPos = step(pos, left, toEdge);
        let width: i32 = Self::distanceToEdge(world, bottomLeft, right);
        if !(frame::MIN_WIDTH..=frame::MAX_SIZE).contains(&width)
        {
            return None;
        }
        let mut size: PortalSize =
            PortalSize { m_right: right, m_bottomLeft: bottomLeft, m_width: width, m_height: 0, m_portals: 0 };
        size.measureHeight(world, left);
        (frame::MIN_HEIGHT..=frame::MAX_SIZE).contains(&size.m_height).then_some(size)
    }

    fn distanceToEdge(world: World, pos: BlockPos, direction: [i32; 2]) -> i32
    {
        let mut distance: i32 = 0;
        while distance < frame::EDGE_SCAN
        {
            let at: BlockPos = step(pos, direction, distance);
            if !isEmpty(world, at) || !isFrame(world, at.offset(0, -1, 0))
            {
                break;
            }
            distance += 1;
        }
        if isFrame(world, step(pos, direction, distance))
        {
            distance
        }
        else
        {
            0
        }
    }

    fn measureHeight(&mut self, world: World, left: [i32; 2])
    {
        'rows: while self.m_height < frame::MAX_SIZE
        {
            for i in 0..self.m_width
            {
                let at: BlockPos = step(self.m_bottomLeft, self.m_right, i).offset(0, self.m_height, 0);
                if !isEmpty(world, at)
                {
                    break 'rows;
                }
                if world.block(at) == blockId(block::AETHER_PORTAL)
                {
                    self.m_portals += 1;
                }
                let side: Option<BlockPos> = if i == 0
                {
                    Some(step(at, left, 1))
                }
                else if i == self.m_width - 1
                {
                    Some(step(at, self.m_right, 1))
                }
                else
                {
                    None
                };
                if side.is_some_and(|side| !isFrame(world, side))
                {
                    break 'rows;
                }
            }
            self.m_height += 1;
        }
        let topClosed: bool = (0..self.m_width)
            .all(|i| isFrame(world, step(self.m_bottomLeft, self.m_right, i).offset(0, self.m_height, 0)));
        if !topClosed
        {
            self.m_height = 0;
        }
    }

    fn isFull(&self) -> bool { self.m_portals >= self.m_width * self.m_height }

    fn place(&self, world: World)
    {
        let portal: i32 = blockId(block::AETHER_PORTAL);
        for i in 0..self.m_width
        {
            for l in 0..self.m_height
            {
                placeQuiet(world, step(self.m_bottomLeft, self.m_right, i).offset(0, l, 0), portal);
            }
        }
        remember(world.dimension, self.center());
    }

    // where a player stands in the middle of the portal's bottom row
    fn center(&self) -> [f64; 3]
    {
        let half: f64 = (self.m_width - 1) as f64 / 2.0;
        [
            self.m_bottomLeft.x as f64 + self.m_right[0] as f64 * half + 0.5,
            self.m_bottomLeft.y as f64,
            self.m_bottomLeft.z as f64 + self.m_right[1] as f64 * half + 0.5,
        ]
    }
}

// placed without neighbor updates so a half built portal never breaks itself, yet marked so clients and the
// renderer see it, as the original's setBlock flag 2
fn placeQuiet(world: World, pos: BlockPos, block: i32)
{
    if let Err(error) = world.setBlockQuiet(pos, block, 0)
    {
        warn(&format!("portal block {block} not placed at {pos:?}: {error}"));
        return;
    }
    world.markBlockForRender(pos);
}

// BlockAetherPortal.trySpawnPortal
fn trySpawn(world: World, pos: BlockPos) -> bool
{
    for axis in 0..AXES.len()
    {
        if let Some(size) = PortalSize::measure(world, pos, axis)
            && size.m_portals == 0
        {
            if world.isAuthority()
            {
                size.place(world);
            }
            return true;
        }
    }
    false
}

#[derive(Serialize, Deserialize, Default)]
struct PortalLog
{
    m_entries: Vec<(i32, [f64; 3])>,
}

fn remember(dimension: i32, center: [f64; 3])
{
    let mut log: PortalLog = worldData(PORTALS_KEY);
    if !log.m_entries.contains(&(dimension, center))
    {
        log.m_entries.push((dimension, center));
        setWorldData(PORTALS_KEY, &log);
    }
}

fn blockAt(point: [f64; 3]) -> BlockPos
{
    BlockPos::new(point[0].floor() as i32, point[1].floor() as i32, point[2].floor() as i32)
}

// the nearest portal this mod built or lit in the target dimension, TeleporterAether.placeInExistingPortal
fn knownPortal(dimension: i32, x: i32, z: i32) -> Option<[f64; 3]>
{
    let log: PortalLog = worldData(PORTALS_KEY);
    let distance = |point: &[f64; 3]| {
        let pos: BlockPos = blockAt(*point);
        ((pos.x - x).abs(), (pos.z - z).abs())
    };
    log.m_entries
        .iter()
        .filter(|(entry, point)| {
            let (dx, dz): (i32, i32) = distance(point);
            *entry == dimension && dx <= travel::SEARCH_RADIUS && dz <= travel::SEARCH_RADIUS
        })
        .min_by_key(|(_, point)| {
            let (dx, dz): (i32, i32) = distance(point);
            dx * dx + dz * dz
        })
        .map(|(_, point)| *point)
}

#[derive(Default)]
struct Visitor
{
    m_ticks: i32,
    m_lastTick: i64,
    // a portal in another dimension starts a fresh visit
    m_dimension: i32,
    // a player must step out of a portal before it can send them again
    m_armed: bool,
}

thread_local! {
    // ticks of this side's game loop; world clocks are shared between dimensions and step more than once a tick
    static TICKS: std::cell::Cell<i64> = const { std::cell::Cell::new(0) };
    static VISITORS: RefCell<HashMap<i32, Visitor>> = RefCell::new(HashMap::new());
    static ARRIVALS: RefCell<HashMap<i32, Option<[f64; 3]>>> = RefCell::new(HashMap::new());
}

pub fn aetherDimension() -> Option<i32>
{
    Identifier::parse(DIMENSION_ID).ok().and_then(|id| resolve(RegistryKind::Dimension, &id).ok())
}

impl Visitor
{
    // one portal tick for this visitor, true once they stood in the portal long enough to be sent
    fn tick(&mut self, dimension: i32, now: i64) -> bool
    {
        if self.m_dimension != dimension
        {
            *self = Visitor { m_ticks: 0, m_lastTick: now, m_dimension: dimension, m_armed: true };
        }
        // collisions arrive several times a tick, the counter moves once per tick
        if now == self.m_lastTick || !self.m_armed
        {
            return false;
        }
        if now - self.m_lastTick > 1
        {
            self.m_ticks = 0;
        }
        self.m_lastTick = now;
        self.m_ticks += 1;
        self.m_ticks >= travel::TICKS
    }

    fn sent(&mut self, target: i32)
    {
        *self = Visitor { m_ticks: 0, m_lastTick: 0, m_dimension: target, m_armed: false };
    }
}

pub fn inPortal(info: &EntityInfo) -> bool
{
    let world: World = World { dimension: info.world };
    let portal: i32 = blockId(block::AETHER_PORTAL);
    let feet: BlockPos =
        BlockPos::new(info.pos[0].floor() as i32, info.pos[1].floor() as i32, info.pos[2].floor() as i32);
    world.block(feet) == portal || world.block(feet.offset(0, 1, 0)) == portal
}

// a sent player arms again only once seen standing outside every portal, as vanilla waits for the player to leave
fn rearm()
{
    let waiting: Vec<i32> = VISITORS
        .with(|visitors| visitors.borrow().iter().filter(|(_, visitor)| !visitor.m_armed).map(|(id, _)| *id).collect());
    for id in waiting
    {
        let outside: bool = Player { id }.info().is_some_and(|info| !inPortal(&info));
        if outside
        {
            VISITORS.with(|visitors| {
                if let Some(visitor) = visitors.borrow_mut().get_mut(&id)
                {
                    visitor.m_armed = true;
                }
            });
        }
    }
}

// PlayerAether.onUpdate portal counter, the player is sent once it stood in the portal long enough
fn enter(world: World, player: Player)
{
    let now: i64 = TICKS.with(std::cell::Cell::get);
    let ready: bool = VISITORS.with(|visitors| {
        let mut visitors = visitors.borrow_mut();
        let visitor: &mut Visitor = visitors.entry(player.id).or_insert(Visitor {
            m_ticks: 0,
            m_lastTick: now,
            m_dimension: world.dimension,
            m_armed: true,
        });
        visitor.tick(world.dimension, now)
    });
    if !ready
    {
        return;
    }
    let (Some(aether), Some(info)) = (aetherDimension(), player.info())
    else
    {
        return;
    };
    let target: i32 = if world.dimension == aether
    {
        travel::OVERWORLD
    }
    else
    {
        aether
    };
    let (x, z): (i32, i32) = (info.pos[0].floor() as i32, info.pos[2].floor() as i32);
    let known: Option<[f64; 3]> = knownPortal(target, x, z);
    let hold: [f64; 3] = known.unwrap_or([x as f64 + 0.5, travel::HOLD_Y, z as f64 + 0.5]);
    ARRIVALS.with(|arrivals| arrivals.borrow_mut().insert(player.id, known));
    VISITORS.with(|visitors| visitors.borrow_mut().entry(player.id).or_default().sent(target));
    let _ = player.teleport(target, hold);
}

fn setFrame(world: World, pos: BlockPos, frameBlock: i32, portal: i32, inside: bool)
{
    placeQuiet(world, pos, if inside { portal } else { frameBlock });
}

// TeleporterAether.makePortal, a two by three glowstone portal on the ground with a platform when it floats
fn buildPortal(world: World, x: i32, z: i32) -> [f64; 3]
{
    let ground: i32 = world.height(x, z);
    let y: i32 = if ground > 0
    {
        ground
    }
    else
    {
        travel::FALLBACK_Y
    };
    let glowstone: i32 = blockId(GLOWSTONE);
    let portal: i32 = blockId(block::AETHER_PORTAL);
    let palette: Palette = Palette::load();
    let floor: i32 = if aetherDimension() == Some(world.dimension)
    {
        palette.m_holystone
    }
    else
    {
        blockId(COBBLESTONE)
    };
    for dx in -1..=build::WIDTH
    {
        for dz in -1..=1
        {
            let below: BlockPos = BlockPos::new(x + dx, y - 1, z + dz);
            if world.isAir(below)
            {
                placeQuiet(world, below, floor);
            }
            for dy in 0..=build::HEIGHT + 1
            {
                let at: BlockPos = BlockPos::new(x + dx, y + dy, z + dz);
                if dz != 0
                {
                    placeQuiet(world, at, 0);
                    continue;
                }
                let edge: bool = dx == -1 || dx == build::WIDTH || dy == 0 || dy == build::HEIGHT + 1;
                setFrame(world, at, glowstone, portal, !edge);
            }
        }
    }
    let center: [f64; 3] = [x as f64 + build::WIDTH as f64 / 2.0, (y + 1) as f64, z as f64 + 0.5];
    remember(world.dimension, center);
    center
}

// runs once the player stands in the target dimension, finds or builds the portal they arrive through
fn arrive(player: Player, to: i32)
{
    let Some(known) = ARRIVALS.with(|arrivals| arrivals.borrow_mut().remove(&player.id))
    else
    {
        return;
    };
    let world: World = World { dimension: to };
    let standing: Option<[f64; 3]> =
        known.filter(|point| world.block(blockAt(*point)) == blockId(block::AETHER_PORTAL));
    let spot: [f64; 3] = match standing
    {
        Some(point) => point,
        None =>
        {
            let Some(info) = player.info()
            else
            {
                return;
            };
            buildPortal(world, info.pos[0].floor() as i32, info.pos[2].floor() as i32)
        }
    };
    let _ = player.teleport(to, spot);
}

// BlockAetherPortal: glowstone framed portal blocks that send players between the Aether and the overworld
struct Portal;

#[block_behavior]
impl BlockBehavior for Portal
{
    fn onEntityCollided(&self, ctx: &mut BlockCtx, entity: Entity)
    {
        if !ctx.isAuthority()
        {
            return;
        }
        if let Some(info) = entity.info()
            && info.player.is_some()
        {
            enter(ctx.world(), Player { id: entity.id });
        }
    }

    fn onRandomDisplayTick(&self, ctx: &mut BlockCtx) { crate::sparkle::portalTick(ctx, blockId(block::AETHER_PORTAL)) }

    // BlockAetherPortal.onNeighborBlockChange, a broken frame drops the portal
    fn onNeighborChanged(&self, ctx: &mut BlockCtx, _neighbor: i32)
    {
        let world: World = ctx.world();
        let intact: bool =
            (0..AXES.len()).any(|axis| PortalSize::measure(world, ctx.pos(), axis).is_some_and(|size| size.isFull()));
        if !intact && ctx.isAuthority()
        {
            let _ = world.setBlock(ctx.pos(), 0, 0);
        }
    }
}

const PORTAL_HARDNESS: f32 = -1.0;
const PORTAL_RESISTANCE: f32 = 900000.0;
const PORTAL_LIGHT: f32 = 0.75;

pub fn registerBlock(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.blocks().register(
        block::AETHER_PORTAL,
        BlockSpecBuilder::template("minecraft:portal")
            .texture("aether:block/aether_portal")
            .hardness(PORTAL_HARDNESS)
            .resistance(PORTAL_RESISTANCE)
            .light(PORTAL_LIGHT)
            .sound("glass")
            .behavior(Portal),
    )?;
    Ok(())
}

// AetherEventHandler.onFillBucket, water poured into a glowstone frame lights the portal on both sides
pub fn registerEvents(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.events().on::<ItemUseOnBlock>(Priority::Normal, |event| {
        let (Some(stack), Some(player), Some(world)) = (event.stack(), event.player(), event.world())
        else
        {
            return;
        };
        let EventData::ItemUse { pos: Some(pos), face, .. } = event.data().clone()
        else
        {
            return;
        };
        if !stack.item.is(WATER_BUCKET)
        {
            return;
        }
        if trySpawn(world, faceOffset(pos, face))
        {
            event.cancel();
            if world.isAuthority()
            {
                let _ = player.setHeld(Some(ItemStack::of(BUCKET, 1)));
            }
        }
    })?;
    ctx.events().on::<TickPost>(Priority::Normal, |_| {
        TICKS.with(|ticks| ticks.set(ticks.get() + 1));
        rearm();
    })?;
    ctx.events().on::<PlayerChangeDimension>(Priority::Normal, |event| {
        if let (EventData::Dimension { to, .. }, Some(player)) = (event.data().clone(), event.player())
        {
            arrive(player, to);
        }
    })?;
    Ok(())
}

// the block next to the clicked face, as ItemBucket offsets its target
fn faceOffset(pos: BlockPos, face: i32) -> BlockPos
{
    match face
    {
        0 => pos.offset(0, -1, 0),
        1 => pos.offset(0, 1, 0),
        2 => pos.offset(0, 0, -1),
        3 => pos.offset(0, 0, 1),
        4 => pos.offset(-1, 0, 0),
        _ => pos.offset(1, 0, 0),
    }
}
