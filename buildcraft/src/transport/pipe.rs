use super::liquids::{FLOW_RATE, Liquid};
use super::travel::{self, Traveler};
use super::{EXCLUSION, Kind, joins, kindAt, property};
use crate::core::power::{Provider, Setup};
use crate::core::stacks::{self, Stored};
use crate::core::{dir, items, state};
use b173_api::prelude::*;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;

const ITEM_CHANNEL: &str = "pipe_item";
const LIQUID_CHANNEL: &str = "pipe_liquid";
// TilePipe sends an item's place at most once a second and its liquid levels every 50 ticks
const ITEM_SYNC: i64 = 20;
const LIQUID_SYNC: i64 = 50;
// TileObsidianPipe remembers the last 32 items it dropped so it never sucks them back in
const DROPPED_MEMORY: usize = 32;
// TileWoodenPipe only pulls another bucket of liquid while less than one is waiting
const BUCKET: i32 = 1000;

// a pipe's tile, saved on the server; a multiplayer client keeps its own copy fed by the server's item packets
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub(crate) struct PipeState
{
    pub(crate) m_items: Vec<Traveler>,
    pub(crate) m_liquid: Liquid,
    m_started: bool,
    m_neighborChange: bool,
    m_sync: crate::core::power::Tracker,
    pub(crate) m_power: Option<Provider>,
    m_liquidToExtract: i32,
    m_lastPower: bool,
    m_dropped: Vec<i32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ItemPacket
{
    m_world: i32,
    m_pos: BlockPos,
    m_item: Traveler,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct LiquidPacket
{
    m_world: i32,
    m_pos: BlockPos,
    m_liquid: Liquid,
}

thread_local!
{
    // a multiplayer client's pipes, filled by item packets and moved by its own ticks between them
    static CLIENT: RefCell<HashMap<(i32, BlockPos), PipeState>> = RefCell::new(HashMap::new());
    // EntityPassiveItem ids, unique on this side for the session
    static NEXT_ID: Cell<u32> = const { Cell::new(1) };
}

pub(crate) fn load(world: World, pos: BlockPos) -> PipeState
{
    if world.isAuthority()
    {
        return world.tileData(pos);
    }

    CLIENT.with(|pipes| pipes.borrow().get(&(world.dimension, pos)).cloned()).unwrap_or_default()
}

fn save(world: World, pos: BlockPos, state: &PipeState)
{
    if !world.isAuthority()
    {
        CLIENT.with(|pipes| pipes.borrow_mut().insert((world.dimension, pos), state.clone()));
        return;
    }

    if let Err(error) = world.setTileData(pos, state)
    {
        warn(&format!("a pipe at {pos:?} could not save: {error}"));
    }
}

fn nextId() -> u32 { NEXT_ID.with(|next| next.replace(next.get().wrapping_add(1))) }

// TileWoodenPipe and TileObsidianPipe configure their providers in their constructors
const WOODEN_POWER: Setup = Setup
{
    m_latency: 50,
    m_minReceived: 1,
    m_maxReceived: 64,
    m_minActivation: 1,
    m_maxStored: 64,
    m_loss: Some((64, 1)),
};
const OBSIDIAN_POWER: Setup = Setup
{
    m_latency: 25,
    m_minReceived: 1,
    m_maxReceived: 64,
    m_minActivation: 1,
    m_maxStored: 256,
    m_loss: Some((1, 1)),
};

fn provider(kind: Kind) -> Option<Provider>
{
    match kind
    {
        Kind::Wooden => Some(WOODEN_POWER.build()),
        Kind::Obsidian => Some(OBSIDIAN_POWER.build()),
        _ => None,
    }
}

fn started(kind: Kind, world: World, pos: BlockPos, state: &mut PipeState)
{
    if state.m_started
    {
        return;
    }

    state.m_started = true;
    state.m_liquid = Liquid::new();
    state.m_power = provider(kind);
    // TileWoodenPipe.initialize looks for a source, TileIronPipe.initialize remembers the power it started with
    state.m_neighborChange = kind == Kind::Wooden;
    state.m_lastPower = world.isPowered(pos);
}

// TilePipe.entityEntering, with the server telling clients so they draw the item moving
fn enter(kind: Kind, world: World, pos: BlockPos, state: &mut PipeState, mut item: Traveler, face: i32)
{
    item.m_speed = travel::adjustSpeed(kind, world, pos, item.m_speed);
    item.m_dir = face;

    if face != dir::UP && face != dir::DOWN
    {
        item.m_pos[1] = pos.y as f64 + travel::FLOOR;
    }

    // TilePipe.entityEntering, createItemPacket stirs the item's numbers each time it is sent
    if travel::dedicated(world) && item.m_sync.due(world.time(), ITEM_SYNC)
    {
        item.m_spin += world.random(dir::COUNT);
        broadcast(world, pos, &item);
    }

    match state.m_items.iter_mut().find(|held| held.m_id == item.m_id)
    {
        // a pipe already carrying the entity keeps its way, a new one heads it to the center as TilePipe.EntityData
        Some(held) => *held = Traveler { m_dir: held.m_dir, m_toCenter: held.m_toCenter, ..item },
        None => state.m_items.push(Traveler { m_toCenter: true, ..item }),
    }
}

fn broadcast(world: World, pos: BlockPos, item: &Traveler)
{
    let packet: ItemPacket = ItemPacket { m_world: world.dimension, m_pos: pos, m_item: item.clone() };

    if let Err(error) = send(&channel(ITEM_CHANNEL), Target::Tracking(world.dimension, pos), &packet)
    {
        warn(&format!("could not send a pipe item: {error}"));
    }
}

fn channel(name: &str) -> String { format!("buildcraft:{name}") }

// an item handed to the pipe at pos from outside it, by another pipe or a machine
pub(crate) fn receive(world: World, pos: BlockPos, item: Traveler, face: i32)
{
    let Some(kind) = kindAt(world, pos)
    else
    {
        return;
    };
    let mut state: PipeState = load(world, pos);

    started(kind, world, pos, &mut state);
    enter(kind, world, pos, &mut state, item, face);
    save(world, pos, &state);
}

// a stack entering from a face, placed at that face's edge the way Utils.addToRandomPipeEntry and the wooden pipe do
fn traveler(pos: BlockPos, face: i32, stack: &ItemStack) -> Option<Traveler>
{
    const TO_EDGE: f64 = 0.5;

    let edge: [f64; 3] = dir::forward(travel::center(pos), face, TO_EDGE);
    Some(Traveler
    {
        m_id: nextId(),
        m_stack: Stored::of(stack)?,
        m_pos: edge,
        m_speed: travel::NORMAL_SPEED,
        m_dir: dir::reverse(face),
        m_toCenter: true,
        m_spin: 0,
        m_sync: crate::core::power::Tracker::default(),
    })
}

// Utils.addToRandomPipeEntry: a random neighbouring pipe takes the whole stack
pub(crate) fn toRandomPipe(world: World, pos: BlockPos, from: i32, stack: &ItemStack) -> bool
{
    let pipes: Vec<i32> = dir::all()
        .filter(|face| *face != dir::reverse(from))
        .filter(|face| kindAt(world, dir::step(pos, *face)).is_some())
        .collect();
    if pipes.is_empty()
    {
        return false;
    }

    let face: i32 = pipes[world.random(pipes.len() as i32) as usize];
    let target: BlockPos = dir::step(pos, face);
    world.insertItem(target, dir::reverse(face), stack.clone(), false).is_ok_and(|left| left == 0)
}

// what happens to an item at the end of a pipe: the next pipe, an inventory, or the floor
fn deliver(world: World, pos: BlockPos, state: &mut PipeState, item: Traveler)
{
    let target: BlockPos = dir::step(pos, item.m_dir);
    if kindAt(world, target).is_some()
    {
        let face: i32 = item.m_dir;
        receive(world, target, item, face);
        return;
    }

    if !world.isAuthority()
    {
        return;
    }

    let stack: ItemStack = item.m_stack.stack();
    let left: i32 = stacks::insert(world, target, dir::reverse(item.m_dir), &stack, false);
    if left <= 0
    {
        return;
    }

    let mut rest: Traveler = item;
    rest.m_stack.m_count = left;
    dropped(state, travel::toEntity(world, &rest));
}

fn dropped(state: &mut PipeState, entity: Option<i32>)
{
    let Some(entity) = entity
    else
    {
        return;
    };

    state.m_dropped.push(entity);

    if state.m_dropped.len() > DROPPED_MEMORY
    {
        state.m_dropped.remove(0);
    }
}

// TilePipe.moveSolids
fn moveItems(kind: Kind, world: World, pos: BlockPos, state: &mut PipeState)
{
    let mut kept: Vec<Traveler> = Vec::new();

    for mut item in std::mem::take(&mut state.m_items)
    {
        item.m_pos = dir::forward(item.m_pos, item.m_dir, f64::from(item.m_speed));

        let turning: bool = (item.m_toCenter && travel::middleReached(pos, &item)) || travel::outOfBounds(pos, &item);
        if !turning
        {
            if !item.m_toCenter && travel::endReached(pos, &item)
            {
                deliver(world, pos, state, item);
            }
            else
            {
                kept.push(item);
            }

            continue;
        }

        item.m_toCenter = false;
        item.m_pos = travel::center(pos);
        let next: i32 = travel::destination(kind, world, pos, &item);
        if next == dir::UNKNOWN
        {
            dropped(state, travel::toEntity(world, &item));
            continue;
        }

        item.m_dir = next;
        kept.push(item);
    }

    // items handed in while this pipe moved its own keep their place after them
    kept.append(&mut state.m_items);
    state.m_items = kept;
}

// the config names blocks by full name, by path in any namespace, or by tag as #namespace:path
fn excluded(world: World, pos: BlockPos) -> bool
{
    let list: String = configValue(EXCLUSION);
    let block: i32 = world.block(pos);

    list.split(',').map(str::trim).filter(|entry| !entry.is_empty()).any(|entry| match entry.strip_prefix('#')
    {
        Some(tag) => blockTagged(block, tag),
        None if entry.contains(':') => blockIs(block, entry),
        None => nameOf(RegistryKind::Block, block).is_some_and(|name| name.path() == entry),
    })
}

// the face a wooden pipe pulls from, None while it has no source
fn source(world: World, pos: BlockPos) -> Option<i32> { state::faceAt(world, pos, property::SOURCE) }

// TileWoodenPipe.doWork: pull a stack the stored energy pays for, or a bucket of liquid
fn woodenWork(world: World, pos: BlockPos, state: &mut PipeState, power: &mut Provider)
{
    if power.m_stored <= 0
    {
        return;
    }

    let Some(source) = source(world, pos)
    else
    {
        return;
    };

    let from: BlockPos = dir::step(pos, source);
    let face: i32 = dir::reverse(source);
    let items: bool = inventory(world, from, face);
    if !items && world.capabilities(from, face) & capability::FLUIDS == 0 || excluded(world, from)
    {
        return;
    }

    if !items
    {
        if state.m_liquidToExtract <= BUCKET
        {
            state.m_liquidToExtract += power.useEnergy(1, 1, true) * BUCKET;
        }

        return;
    }

    let Some(taken) = stacks::extract(world, from, face, |available| power.useEnergy(1, available, true))
        .filter(|taken| taken.count > 0)
    else
    {
        power.useEnergy(1, 1, false);
        return;
    };

    if let Some(item) = traveler(pos, source, &taken)
    {
        enter(Kind::Wooden, world, pos, state, item, face);
    }
}

// TileWoodenPipe.updateEntity: the liquid it paid for comes in a flow rate at a time
fn woodenLiquid(world: World, pos: BlockPos, state: &mut PipeState)
{
    if state.m_liquidToExtract <= 0
    {
        return;
    }

    let Some(source) = source(world, pos)
    else
    {
        return;
    };

    let from: BlockPos = dir::step(pos, source);
    let face: i32 = dir::reverse(source);
    let Some(offer) = world.drainFluid(from, face, state.m_liquidToExtract.min(FLOW_RATE), true)
    else
    {
        return;
    };

    let taken: i32 = state.m_liquid.fill(Kind::Wooden, world, pos, source, &offer, false);
    if taken > 0
    {
        world.drainFluid(from, face, taken, false);
    }

    state.m_liquidToExtract -= taken;
}

// instanceof IInventory without pipes: BuildCraft let a wooden pipe pick a diamond pipe's filter slots as its source
// and then pull nothing, so pipes never count
fn inventory(world: World, pos: BlockPos, face: i32) -> bool
{
    kindAt(world, pos).is_none() && world.capabilities(pos, face) & capability::ITEMS != 0
}

// TileWoodenPipe.switchSource, the next face after the current one with an inventory or tank that is no pipe
pub(crate) fn switchSource(world: World, pos: BlockPos)
{
    let current: Option<i32> = source(world, pos);
    let start: i32 = current.unwrap_or(dir::UNKNOWN);
    let pick: Option<i32> = (start + 1..=start + dir::COUNT).map(|step| step % dir::COUNT).find(|face|
    {
        let from: BlockPos = dir::step(pos, *face);
        let tank: bool =
            kindAt(world, from).is_none() && world.capabilities(from, dir::reverse(*face)) & capability::FLUIDS != 0;
        (inventory(world, from, dir::reverse(*face)) || tank) && joins(world, pos, *face) && !excluded(world, from)
    });
    if pick == current
    {
        return;
    }

    state::set(world, pos, property::SOURCE, pick.map_or(property::NONE, dir::name));
    world.markBlockForRender(pos);
}

// TileIronPipe.switchPosition, the next face with a pipe that is not wooden or an inventory
pub(crate) fn switchOutput(world: World, pos: BlockPos)
{
    let current: i32 = super::output(world, pos).unwrap_or(dir::DOWN);
    let pick: Option<i32> = (1..=dir::COUNT).map(|step| (current + step) % dir::COUNT).find(|face|
    {
        let target: BlockPos = dir::step(pos, *face);
        match kindAt(world, target)
        {
            Some(kind) => kind != Kind::Wooden,
            None => world.capabilities(target, dir::reverse(*face)) & capability::ITEMS != 0,
        }
    });

    if let Some(face) = pick
    {
        state::set(world, pos, property::OUTPUT, dir::name(face));
        world.markBlockForRender(pos);
    }
}

fn neighborChange(kind: Kind, world: World, pos: BlockPos, state: &mut PipeState)
{
    state.m_liquid.neighborChanged(kind, world, pos);

    if kind != Kind::Wooden
    {
        return;
    }

    // TileWoodenPipe.neighborChange keeps a source that is still an inventory
    let kept: bool = source(world, pos).is_some_and(|face| inventory(world, dir::step(pos, face), dir::reverse(face)));
    if !kept
    {
        switchSource(world, pos);
    }
}

fn sendLiquid(world: World, pos: BlockPos, state: &mut PipeState)
{
    if !state.m_sync.due(world.time(), LIQUID_SYNC)
    {
        return;
    }

    let packet: LiquidPacket = LiquidPacket { m_world: world.dimension, m_pos: pos, m_liquid: state.m_liquid.clone() };
    if let Err(error) = send(&channel(LIQUID_CHANNEL), Target::Tracking(world.dimension, pos), &packet)
    {
        warn(&format!("could not send pipe liquid: {error}"));
    }
}

fn tick(kind: Kind, ctx: &mut BlockCtx)
{
    let world: World = ctx.world();
    let pos: BlockPos = ctx.pos();
    let mut state: PipeState = load(world, pos);

    started(kind, world, pos, &mut state);

    if world.isAuthority()
        && let Some(mut power) = state.m_power.take()
    {
        if power.update(world, pos)
        {
            match kind
            {
                Kind::Wooden => woodenWork(world, pos, &mut state, &mut power),
                Kind::Obsidian => super::obsidian::work(world, pos, &mut state, &mut power),
                _ =>
                {}
            }
        }

        power.decay(world);
        state.m_power = Some(power);
    }

    moveItems(kind, world, pos, &mut state);

    if world.isAuthority()
    {
        state.m_liquid.tick(kind, world, pos);

        if kind == Kind::Wooden
        {
            woodenLiquid(world, pos, &mut state);
        }

        if state.m_neighborChange
        {
            state.m_neighborChange = false;
            neighborChange(kind, world, pos, &mut state);
        }

        sendLiquid(world, pos, &mut state);
    }

    save(world, pos, &state);
}

pub(crate) struct PipeTile
{
    pub(crate) m_kind: Kind,
}

#[tile_behavior]
impl TileBehavior for PipeTile
{
    fn onTick(&self, ctx: &mut BlockCtx) { tick(self.m_kind, ctx); }

    // TilePipe.destroy throws out every item still travelling
    fn onRemoved(&self, ctx: &mut BlockCtx)
    {
        let world: World = ctx.world();
        let state: PipeState = load(world, ctx.pos());

        for item in &state.m_items
        {
            travel::toEntity(world, item);
        }

        CLIENT.with(|pipes| pipes.borrow_mut().remove(&(world.dimension, ctx.pos())));
    }

    fn pose(&self, ctx: &mut BlockCtx, _partial: f32, _ticks: i32) -> Pose { super::render::pose(ctx) }

    // IPipeEntry for every mod: a pipe takes any stack offered from any face
    fn insertItem(&self, ctx: &mut BlockCtx, face: i32, stack: ItemStack, simulate: bool) -> i32
    {
        if simulate
        {
            return 0;
        }

        let Some(item) = traveler(ctx.pos(), face, &stack)
        else
        {
            return stack.count;
        };

        receive(ctx.world(), ctx.pos(), item, dir::reverse(face));
        0
    }

    fn fillFluid(&self, ctx: &mut BlockCtx, face: i32, stack: &FluidStack, simulate: bool) -> i32
    {
        let world: World = ctx.world();
        let mut state: PipeState = load(world, ctx.pos());
        started(self.m_kind, world, ctx.pos(), &mut state);

        let taken: i32 = state.m_liquid.fill(self.m_kind, world, ctx.pos(), face, stack, simulate);
        if !simulate
        {
            save(world, ctx.pos(), &state);
        }

        taken
    }

    fn energyDemand(&self, ctx: &mut BlockCtx, _face: i32) -> Option<EnergyDemand>
    {
        let mut state: PipeState = load(ctx.world(), ctx.pos());
        started(self.m_kind, ctx.world(), ctx.pos(), &mut state);
        state.m_power.and_then(|power| power.demand())
    }

    fn receiveEnergy(&self, ctx: &mut BlockCtx, _face: i32, amount: i64, simulate: bool) -> i64
    {
        let world: World = ctx.world();
        let mut state: PipeState = load(world, ctx.pos());
        started(self.m_kind, world, ctx.pos(), &mut state);

        let Some(mut power) = state.m_power
        else
        {
            return 0;
        };

        let taken: i64 = power.take(amount, simulate);
        state.m_power = Some(power);

        if !simulate
        {
            save(world, ctx.pos(), &state);
        }

        taken
    }
}

// BlockWoodenPipe and BlockIronPipe turn with a wrench, the diamond pipe opens its filters
pub(crate) fn activated(kind: Kind, ctx: &mut BlockCtx, player: Player) -> bool
{
    let world: World = ctx.world();
    let pos: BlockPos = ctx.pos();

    match kind
    {
        Kind::Wooden | Kind::Iron if items::holdsWrench(player) =>
        {
            if ctx.isAuthority()
            {
                if kind == Kind::Wooden
                {
                    switchSource(world, pos)
                }
                else
                {
                    switchOutput(world, pos)
                }
            }

            true
        }

        Kind::Diamond =>
        {
            if ctx.isAuthority()
            {
                let _ = player.openScreen(super::diamond::SCREEN, Some(pos), Vec::new());
            }

            true
        }

        _ => false,
    }
}

// BlockIronPipe.onBlockPlaced points up first, then at the first neighbour that takes items
pub(crate) fn placed(kind: Kind, ctx: &mut BlockCtx)
{
    if !ctx.isAuthority() || kind != Kind::Iron
    {
        return;
    }

    state::set(ctx.world(), ctx.pos(), property::OUTPUT, dir::name(dir::UP));
    switchOutput(ctx.world(), ctx.pos());
}

// BlockPipe.onNeighborBlockChange schedules a check, the iron pipe turns on a redstone edge, the gold pipe shows power
pub(crate) fn neighborChanged(kind: Kind, ctx: &mut BlockCtx)
{
    let world: World = ctx.world();
    let pos: BlockPos = ctx.pos();
    if !ctx.isAuthority()
    {
        return;
    }

    let mut state: PipeState = load(world, pos);
    started(kind, world, pos, &mut state);

    match kind
    {
        Kind::Iron =>
        {
            let powered: bool = world.isPowered(pos);
            if powered != state.m_lastPower
            {
                switchOutput(world, pos);
                state.m_lastPower = powered;
            }
        }

        Kind::Golden =>
        {
            let lit: bool = world.isPowered(pos);
            if ctx.stateAt().is_some_and(|shown| shown.flag(property::POWERED)) != lit
            {
                state::set(world, pos, property::POWERED, &lit.to_string());
            }

            state.m_neighborChange = true;
        }

        _ => state.m_neighborChange = true,
    }

    save(world, pos, &state);
}

pub(crate) fn collided(ctx: &mut BlockCtx, entity: Entity)
{
    if !ctx.isAuthority()
    {
        return;
    }

    let world: World = ctx.world();
    let pos: BlockPos = ctx.pos();
    let mut state: PipeState = load(world, pos);
    started(Kind::Obsidian, world, pos, &mut state);

    super::obsidian::touched(world, pos, &mut state, entity);
    save(world, pos, &state);
}

pub(crate) fn enterFromSuction(world: World, pos: BlockPos, state: &mut PipeState, stack: &ItemStack, face: i32)
{
    let Some(stored) = Stored::of(stack)
    else
    {
        return;
    };
    let item: Traveler = Traveler
    {
        m_id: nextId(),
        m_stack: stored,
        m_pos: travel::center(pos),
        m_speed: travel::NORMAL_SPEED,
        m_dir: face,
        m_toCenter: true,
        m_spin: 0,
        m_sync: crate::core::power::Tracker::default(),
    };

    enter(Kind::Obsidian, world, pos, state, item, face);
}

pub(crate) fn droppedIds(state: &PipeState) -> &[i32] { &state.m_dropped }

fn onItemPacket(packet: ItemPacket)
{
    let world: World = World { dimension: packet.m_world };
    if world.isAuthority()
    {
        return;
    }

    let mut state: PipeState = load(world, packet.m_pos);
    let item: Traveler = packet.m_item;

    match state.m_items.iter_mut().find(|held| held.m_id == item.m_id)
    {
        Some(held) => *held = item,
        None => state.m_items.push(item),
    }

    save(world, packet.m_pos, &state);
}

fn onLiquidPacket(packet: LiquidPacket)
{
    let world: World = World { dimension: packet.m_world };
    if world.isAuthority()
    {
        return;
    }

    let mut state: PipeState = load(world, packet.m_pos);
    state.m_liquid = packet.m_liquid;
    save(world, packet.m_pos, &state);
}

pub(crate) fn registerChannels(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.channels().register(ITEM_CHANNEL, Direction::ToClient, |packet: ItemPacket, _player| onItemPacket(packet))?;
    ctx.channels().register(LIQUID_CHANNEL, Direction::ToClient, |packet: LiquidPacket, _player| onLiquidPacket(packet))
}
