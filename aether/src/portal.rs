use crate::names::{AIR, Pick, block, handle};
use crate::world::DIMENSION_ID;
use b173_api::math::{Face, blockOf, floor, offset};
use b173_api::prelude::*;

const GLOWSTONE: &str = "minecraft:glowstone";
const FIRE: &str = "minecraft:fire";
const WATER_BUCKET: &str = "minecraft:water_bucket";
const BUCKET: &str = "minecraft:bucket";
const LAVA_BUCKET: &str = "minecraft:lava_bucket";
const AEROGEL: &str = "aerogel";
const BED: &str = "minecraft:bed";
// AetherEventHandler.checkBlockBannedEvent: what only smokes when used in the Aether
const BANNED: [&str; 2] = ["minecraft:flint_and_steel", "minecraft:torch"];
const BAN_PUFFS: i32 = 10;
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

// ItemAetherPortalFrame.createPortalFrame: glowstone four wide and five tall around a two by three portal
mod kit
{
    pub const WIDTH: i32 = 4;
    pub const ROWS: std::ops::Range<i32> = -1..4;
    pub const INNER_COLUMNS: std::ops::Range<i32> = 1..3;
    pub const INNER_ROWS: std::ops::Range<i32> = 0..3;
    // the base row sits on the clicked block, or in its place when that block gives way
    pub const ON_TOP: i32 = 2;
    pub const IN_PLACE: i32 = 1;
    pub const QUARTERS: f64 = 4.0;
    pub const TURN: f64 = 360.0;
}

// Direction.offsetX and offsetZ for the BlockPortal axis tables, left then right
const AXES: [[[i32; 2]; 2]; 2] = [[[1, 0], [-1, 0]], [[0, -1], [0, 1]]];

fn isEmpty(world: World, pos: BlockPos) -> bool
{
    world.isAir(pos) || world.is(pos, FIRE) || world.is(pos, block::AETHER_PORTAL)
}

fn isFrame(world: World, pos: BlockPos) -> bool { world.is(pos, GLOWSTONE) }

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
    fn measure(world: World, pos: BlockPos, axis: usize) -> Option<Self>
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

        let mut size: Self =
            Self { m_right: right, m_bottomLeft: bottomLeft, m_width: width, m_height: 0, m_portals: 0 };
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

                if world.is(at, block::AETHER_PORTAL)
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
        let portal: Pick = Pick::of(block::AETHER_PORTAL);

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
fn placeQuiet(world: World, pos: BlockPos, pick: Pick)
{
    if let Err(error) = handle(pick).place(world, pos, false)
    {
        warn(&format!("portal block {} not placed at {pos:?}: {error}", pick.m_block));
        return;
    }

    world.markBlockForRender(pos);
}

// BlockAetherPortal.trySpawnPortal
pub(crate) fn trySpawn(world: World, pos: BlockPos) -> bool
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

// the nearest portal this mod built or lit in the target dimension, TeleporterAether.placeInExistingPortal
fn knownPortal(dimension: i32, x: i32, z: i32) -> Option<[f64; 3]>
{
    let log: PortalLog = worldData(PORTALS_KEY);

    let distance = |point: &[f64; 3]|
    {
        let pos: BlockPos = blockOf(*point);
        ((pos.x - x).abs(), (pos.z - z).abs())
    };
    log.m_entries
        .iter()
        .filter(|(entry, point)|
        {
            let (dx, dz): (i32, i32) = distance(point);
            *entry == dimension && dx <= travel::SEARCH_RADIUS && dz <= travel::SEARCH_RADIUS
        })
        .min_by_key(|(_, point)|
        {
            let (dx, dz): (i32, i32) = distance(point);
            dx * dx + dz * dz
        })
        .map(|(_, point)| *point)
}

#[derive(Serialize, Deserialize, Default)]
struct Visitor
{
    m_ticks: i32,
    m_lastTick: i64,
    // a portal in another dimension starts a fresh visit
    m_dimension: i32,
    // a player must step out of a portal before it can send them again
    m_armed: bool,
}

thread_local!
{
    // ticks of this side's game loop; world clocks are shared between dimensions and step more than once a tick
    static TICKS: std::cell::Cell<i64> = const { std::cell::Cell::new(0) };
}

// a player's portal visit, and the known portal a sent player arrives at; both outlive the trip between worlds
static VISITORS: EntityState<Visitor> = EntityState::new("portal_visit");
static ARRIVALS: EntityState<Option<Option<[f64; 3]>>> = EntityState::new("portal_arrival");

pub(crate) fn aetherDimension() -> Option<i32>
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
            *self = Self { m_ticks: 0, m_lastTick: now, m_dimension: dimension, m_armed: true };
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

    fn sent(&mut self, target: i32) { *self = Self { m_ticks: 0, m_lastTick: 0, m_dimension: target, m_armed: false }; }
}

pub(crate) fn inPortal(info: &EntityInfo) -> bool
{
    let world: World = World { dimension: info.world };
    let feet: BlockPos = blockOf(info.pos);
    world.is(feet, block::AETHER_PORTAL) || world.is(feet.offset(0, 1, 0), block::AETHER_PORTAL)
}

// a sent player arms again only once seen standing outside every portal, as vanilla waits for the player to leave
fn rearm()
{
    for (entity, info) in allPlayers().into_iter().filter_map(crate::abilities::entityOf)
    {
        let Some(mut visitor) = VISITORS.find(entity).filter(|visitor| !visitor.m_armed)
        else
        {
            continue;
        };

        if !inPortal(&info)
        {
            visitor.m_armed = true;
            crate::entities::store(&VISITORS, entity, &visitor);
        }
    }
}

// PlayerAether.onUpdate portal counter, the player is sent once it stood in the portal long enough
fn enter(world: World, player: Player)
{
    let now: i64 = TICKS.with(std::cell::Cell::get);

    let entity: Entity = Entity { world, id: player.id };
    let fresh: Visitor = Visitor { m_ticks: 0, m_lastTick: now, m_dimension: world.dimension, m_armed: true };
    let mut visitor: Visitor = VISITORS.find(entity).unwrap_or(fresh);
    let ready: bool = visitor.tick(world.dimension, now);
    crate::entities::store(&VISITORS, entity, &visitor);
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
    let (x, z): (i32, i32) = (floor(info.pos[0]), floor(info.pos[2]));
    let known: Option<[f64; 3]> = knownPortal(target, x, z);
    let hold: [f64; 3] = known.unwrap_or([x as f64 + 0.5, travel::HOLD_Y, z as f64 + 0.5]);

    let mut visitor: Visitor = VISITORS.get(entity);
    visitor.sent(target);
    crate::entities::store(&VISITORS, entity, &visitor);
    crate::entities::store(&ARRIVALS, entity, &Some(known));

    let _ = player.teleport(target, hold);
}

fn setFrame(world: World, pos: BlockPos, frameBlock: Pick, portal: Pick, inside: bool)
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
    let glowstone: Pick = Pick::of(GLOWSTONE);
    let portal: Pick = Pick::of(block::AETHER_PORTAL);
    let floor: Pick = if aetherDimension() == Some(world.dimension)
    {
        Pick::of(block::HOLYSTONE)
    }
    else
    {
        Pick::of(COBBLESTONE)
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
                    placeQuiet(world, at, AIR);
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
    let arriving: Entity = Entity { world: World { dimension: to }, id: player.id };
    let Some(known) = ARRIVALS.get(arriving)
    else
    {
        return;
    };

    crate::entities::clear(&ARRIVALS, arriving);

    let world: World = World { dimension: to };
    let standing: Option<[f64; 3]> = known.filter(|point| world.is(blockOf(*point), block::AETHER_PORTAL));

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

            buildPortal(world, floor(info.pos[0]), floor(info.pos[2]))
        }
    };
    let _ = player.teleport(to, spot);
}

fn replaceable(world: World, pos: BlockPos) -> bool
{
    world.isAir(pos) || World::blockInfo(world.block(pos)).replaceable
}

// ItemAetherPortalFrame.createPortalFrame: nothing is built unless the whole frame has room
fn buildFrame(world: World, clicked: BlockPos, yaw: f32) -> bool
{
    let facing: i32 = floor(f64::from(yaw) * kit::QUARTERS / kit::TURN + 0.5) & 3;
    let alongX: bool = facing == 0 || facing == 2;
    let lift: i32 = if replaceable(world, clicked)
    {
        kit::IN_PLACE
    }
    else
    {
        kit::ON_TOP
    };
    let base: BlockPos = clicked.offset(0, lift, 0);
    let cell = |along: i32, up: i32| -> BlockPos
    {
        if alongX
        {
            base.offset(along - 1, up, 0)
        }
        else
        {
            base.offset(0, up, along - 1)
        }
    };
    let cells = |columns: std::ops::Range<i32>, rows: std::ops::Range<i32>| -> Vec<BlockPos>
    {
        columns.flat_map(|along| rows.clone().map(move |up| (along, up))).map(|(along, up)| cell(along, up)).collect()
    };

    let frame: Vec<BlockPos> = cells(0..kit::WIDTH, kit::ROWS);
    if !frame.iter().all(|at| replaceable(world, *at))
    {
        return false;
    }

    for at in frame
    {
        placeQuiet(world, at, Pick::of(GLOWSTONE));
    }

    for at in cells(kit::INNER_COLUMNS, kit::INNER_ROWS)
    {
        placeQuiet(world, at, AIR);
    }

    trySpawn(world, cell(kit::INNER_COLUMNS.start, kit::INNER_ROWS.start))
}

// ItemAetherPortalFrame: one use builds a whole lit portal on the block it is used on
pub(crate) struct FrameKit;

#[item_behavior]
impl ItemBehavior for FrameKit
{
    fn onUseOnBlock(&self, world: World, player: Player, stack: ItemStack, pos: BlockPos, _face: i32) -> Use
    {
        let built: bool = world.isAuthority() && player.info().is_some_and(|info| buildFrame(world, pos, info.yaw));
        if !built
        {
            return Use::Done(Some(stack));
        }

        Use::Done((stack.count > 1).then(|| ItemStack { count: stack.count - 1, ..stack }))
    }
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

    fn onRandomDisplayTick(&self, ctx: &mut BlockCtx) { crate::sparkle::portalTick(ctx, block::AETHER_PORTAL) }

    // BlockAetherPortal.onNeighborBlockChange, a broken frame drops the portal
    fn onNeighborChanged(&self, ctx: &mut BlockCtx, _neighbor: i32)
    {
        let world: World = ctx.world();
        let intact: bool =
            (0..AXES.len()).any(|axis| PortalSize::measure(world, ctx.pos(), axis).is_some_and(|size| size.isFull()));

        if !intact && ctx.isAuthority()
        {
            let _ = world.clear(ctx.pos());
        }
    }
}

const PORTAL_HARDNESS: f32 = -1.0;
const PORTAL_RESISTANCE: f32 = 900000.0;
const PORTAL_LIGHT: f32 = 0.75;

pub(crate) fn registerBlock(ctx: &mut RegisterCtx) -> Result<()>
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

fn inAether(world: World) -> bool { aetherDimension() == Some(world.dimension) }

// AetherEventHandler.checkBlockBannedEvent: flint and steel and torches only smoke in the Aether
fn smother(event: &mut Event, world: World, pos: BlockPos)
{
    for _ in 0..BAN_PUFFS
    {
        world.particle("smoke", [f64::from(pos.x), f64::from(pos.y), f64::from(pos.z)], [0.0; 3]);
    }

    event.cancel();
}

// AetherEventHandler.onFillBucket: water in a glowstone frame lights the portal, lava poured in the Aether sets
// into aerogel
fn bucketUsed(event: &mut Event)
{
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

    if inAether(world) && BANNED.iter().any(|name| stack.item.is(name))
    {
        smother(event, world, pos);
        return;
    }

    let Some(face) = Face::fromIndex(face)
    else
    {
        return;
    };

    // the block next to the clicked face as ItemBucket offsets its target
    let target: BlockPos = offset(pos, face);
    let travels: bool = inAether(world) || world.dimension == travel::OVERWORLD;
    if stack.item.is(WATER_BUCKET) && travels && trySpawn(world, target)
    {
        event.cancel();

        if world.isAuthority()
        {
            let _ = player.setHeld(Some(ItemStack::of(BUCKET, 1)));
        }

        return;
    }

    if !stack.item.is(LAVA_BUCKET) || !inAether(world)
    {
        return;
    }

    event.cancel();

    if world.isAir(target) && world.isAuthority()
    {
        let _ = world.place(target, AEROGEL);
        let _ = player.setHeld(Some(ItemStack::of(BUCKET, 1)));
    }
}

// AetherEventHandler.checkBlockBannedEvent: an empty hand never makes a vanilla bed in the Aether
fn bedUsed(event: &mut Event)
{
    let EventData::Block { world, block, player: Some(player), .. } = event.data().clone()
    else
    {
        return;
    };

    if inAether(World { dimension: world }) && blockIs(block, BED) && (Player { id: player }).held().is_none()
    {
        event.cancel();
    }
}

pub(crate) fn registerEvents(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.events().on::<ItemUseOnBlock>(Priority::Normal, bucketUsed)?;
    ctx.events().on::<BlockActivate>(Priority::Normal, bedUsed)?;

    ctx.events().on::<TickPost>(Priority::Normal, |_|
    {
        TICKS.with(|ticks| ticks.set(ticks.get() + 1));
        rearm();
    })?;

    ctx.events().on::<PlayerChangeDimension>(Priority::Normal, |event|
    {
        if let (EventData::Dimension { to, .. }, Some(player)) = (event.data().clone(), event.player())
        {
            arrive(player, to);
        }
    })?;

    Ok(())
}
