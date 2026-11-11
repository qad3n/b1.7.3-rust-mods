use crate::core::power::{Provider, Setup};
use crate::core::render::{self, Scene};
use crate::core::{AIR, dir};
use b173_api::prelude::*;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::rc::Rc;

// TilePump: lowers a tube to the oil below it and pumps the top layer of the pool first, a bucket at a time
const NAME: &str = "pump";
const POWER: Setup =
    Setup { m_latency: 20, m_minReceived: 10, m_maxReceived: 10, m_minActivation: 10, m_maxStored: 100, m_loss: None };
const PUMP_COST: i32 = 10;
const BUCKET: i32 = 1000;
const FLOW_RATE: i32 = 20;
const TUBE_SPEED: f64 = 0.01;
const RESCAN: i64 = 100;
const HARDNESS: f32 = 5.0;
const OIL: &str = "buildcraft:oil";
const VIEW_CHANNEL: &str = "pump";
const TOP: &str = "buildcraft:block/pump_top";
const BOTTOM: &str = "buildcraft:block/pump_bottom";
const SIDE: &str = "buildcraft:block/pump_side";
const TUBE: &str = "buildcraft:block/pump_tube";
const TUBE_WIDTH: f64 = 0.5;
const TUBE_INSET: f64 = 0.25;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
struct PumpState
{
    m_started: bool,
    m_internal: i32,
    m_tubeY: f64,
    m_aimY: i32,
    m_provider: Option<Provider>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
struct ViewPacket
{
    m_world: i32,
    m_pos: BlockPos,
    m_tubeY: f64,
}

// the pumpable oil under one pump by layer height, the top layer pumped first
type Pool = BTreeMap<i32, VecDeque<BlockPos>>;

thread_local!
{
    // TilePump.blocksToPump, rebuilt from the world every few seconds so never saved
    static LAYERS: RefCell<HashMap<(i32, BlockPos), Pool>> = RefCell::new(HashMap::new());
    static TUBES: RefCell<HashMap<(i32, BlockPos), f64>> = RefCell::new(HashMap::new());
}

fn isOil(world: World, pos: BlockPos) -> bool { world.tagged(pos, crate::energy::OIL_TAG) }

fn pumpable(world: World, pos: BlockPos) -> bool
{
    isOil(world, pos) && world.fluidAt(pos).is_some_and(|fluid| fluid.source)
}

// getNextIndexToPump: the highest layer that still has oil, its last entry to look at, its first to take
fn next(world: World, pump: BlockPos, take: bool) -> Option<BlockPos>
{
    LAYERS.with(|layers|
    {
        let mut layers = layers.borrow_mut();
        let pool: &mut Pool = layers.get_mut(&(world.dimension, pump))?;
        let (&height, layer) = pool.iter_mut().rev().find(|(height, layer)| **height > 0 && !layer.is_empty())?;

        if !take
        {
            return layer.back().copied();
        }

        let found: Option<BlockPos> = layer.pop_front();
        if layer.is_empty()
        {
            pool.remove(&height);
        }

        found
    })
}

// initializePumpFromPosition: every connected oil block, each pumpable one listed under its layer
// TilePump.initializePumpFromPosition: the frontier is a TreeSet walked in x, y, z order; BuildCraft filed sideways
// neighbours under the previous block's upper layer, here each block goes to its own layer so the top drains first
fn survey(world: World, pump: BlockPos, start: BlockPos)
{
    type Index = (i32, i32, i32);

    let mut marked: BTreeSet<Index> = BTreeSet::new();
    let mut found: BTreeSet<Index> = BTreeSet::new();
    let mut pool: Pool = LAYERS.with(|layers| layers.borrow_mut().remove(&(world.dimension, pump))).unwrap_or_default();

    let mut visit = |index: Index, found: &mut BTreeSet<Index>, pool: &mut Pool|
    {
        if !marked.insert(index)
        {
            return;
        }

        let pos: BlockPos = BlockPos::new(index.0, index.1, index.2);
        if pumpable(world, pos)
        {
            pool.entry(pos.y).or_default().push_front(pos);
        }

        if isOil(world, pos)
        {
            found.insert(index);
        }
    };

    visit((start.x, start.y, start.z), &mut found, &mut pool);

    while !found.is_empty()
    {
        for (x, y, z) in std::mem::take(&mut found)
        {
            for index in [(x + 1, y, z), (x - 1, y, z), (x, y, z + 1), (x, y, z - 1), (x, y + 1, z)]
            {
                visit(index, &mut found, &mut pool);
            }
        }
    }

    LAYERS.with(|layers| layers.borrow_mut().insert((world.dimension, pump), pool));
}

fn publish(world: World, pos: BlockPos, tubeY: f64)
{
    let packet: ViewPacket = ViewPacket { m_world: world.dimension, m_pos: pos, m_tubeY: tubeY };
    if let Err(error) = send(&format!("buildcraft:{VIEW_CHANNEL}"), Target::Tracking(world.dimension, pos), &packet)
    {
        warn(&format!("could not send a pump update: {error}"));
    }
}

// TilePump.updateEntity on the server
fn tick(world: World, pos: BlockPos, state: &mut PumpState)
{
    let mut provider: Provider = state.m_provider.take().unwrap_or_else(|| POWER.build());
    // doWork does nothing for a pump, so the update ends with its power loss
    provider.update(world, pos);
    provider.decay(world);

    if state.m_tubeY - f64::from(state.m_aimY) > TUBE_SPEED
    {
        state.m_tubeY -= TUBE_SPEED;
        publish(world, pos, state.m_tubeY);
        state.m_provider = Some(provider);
        return;
    }

    if state.m_internal <= FLOW_RATE
    {
        pumpOrSearch(world, pos, state, &mut provider);
    }

    state.m_provider = Some(provider);
    push(world, pos, state);
}

fn pumpOrSearch(world: World, pos: BlockPos, state: &mut PumpState, provider: &mut Provider)
{
    if next(world, pos, false).is_some_and(|index| pumpable(world, index))
    {
        if provider.useEnergy(PUMP_COST, PUMP_COST, true) != PUMP_COST
        {
            return;
        }

        if let Some(index) = next(world, pos, true)
            && world.clear(index).is_ok()
        {
            state.m_internal += BUCKET;
        }

        return;
    }

    if world.time() % RESCAN != 0
    {
        return;
    }

    survey(world, pos, BlockPos::new(pos.x, state.m_aimY, pos.z));

    if next(world, pos, false).is_some()
    {
        return;
    }

    // the tube looks further down through air for more oil
    for y in (1..pos.y).rev()
    {
        let below: BlockPos = BlockPos::new(pos.x, y, pos.z);
        if isOil(world, below)
        {
            state.m_aimY = y;
            return;
        }

        if !world.is(below, AIR)
        {
            return;
        }
    }
}

// TilePump fills its TilePipe neighbours; other mods' fluid blocks take oil too, BuildCraft's own tanks and engines
// only through a pipe as in BuildCraft
fn push(world: World, pos: BlockPos, state: &mut PumpState)
{
    let Ok(oil) = Identifier::parse(OIL)
    else
    {
        return;
    };

    for face in dir::all()
    {
        if state.m_internal < FLOW_RATE
        {
            return;
        }

        let target: BlockPos = dir::step(pos, face);
        let ours: bool = world.blockName(target).is_some_and(|name| name.namespace() == "buildcraft");
        let open: bool = crate::transport::isPipe(world, target)
            || (!ours && world.capabilities(target, dir::reverse(face)) & capability::FLUIDS != 0);
        if !open
        {
            continue;
        }

        let offered: FluidStack = FluidStack { fluid: oil.clone(), amount: FLOW_RATE };
        state.m_internal -= world.fillFluid(target, dir::reverse(face), &offered, false);
    }
}

fn start(pos: BlockPos, state: &mut PumpState)
{
    if state.m_started
    {
        return;
    }

    state.m_started = true;
    state.m_tubeY = f64::from(pos.y);
    state.m_aimY = pos.y;
}

struct PumpTile;

#[tile_behavior]
impl TileBehavior for PumpTile
{
    fn onTick(&self, ctx: &mut BlockCtx)
    {
        if !ctx.isAuthority()
        {
            return;
        }

        let mut state: PumpState = ctx.tileData();
        start(ctx.pos(), &mut state);
        tick(ctx.world(), ctx.pos(), &mut state);
        let _ = ctx.setTileData(&state);
    }

    fn onRemoved(&self, ctx: &mut BlockCtx)
    {
        let key: (i32, BlockPos) = (ctx.world().dimension, ctx.pos());
        LAYERS.with(|layers| layers.borrow_mut().remove(&key));
        TUBES.with(|tubes| tubes.borrow_mut().remove(&key));
    }

    // the tube from the pump down to its head, as the EntityBlock TilePump keeps
    fn pose(&self, ctx: &mut BlockCtx, _partial: f32, _ticks: i32) -> Pose
    {
        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();
        let tubeY: f64 = if world.isAuthority()
        {
            let state: PumpState = world.tileData(pos);
            if state.m_started
            {
                state.m_tubeY
            }
            else
            {
                f64::from(pos.y)
            }
        }
        else
        {
            TUBES.with(|tubes| tubes.borrow().get(&(world.dimension, pos)).copied()).unwrap_or(f64::from(pos.y))
        };
        let length: f64 = f64::from(pos.y) - tubeY;
        if length <= 0.0
        {
            return Pose::default();
        }

        let mut draw: Scene = Scene::new();
        let origin: [f64; 3] = render::relative(pos, [pos.x as f64 + TUBE_INSET, tubeY, pos.z as f64 + TUBE_INSET]);
        draw.stretched(origin, [TUBE_WIDTH, length, TUBE_WIDTH], render::uv(TUBE), render::lightAt(world, pos));
        draw.finish()
    }

    fn energyDemand(&self, ctx: &mut BlockCtx, _face: i32) -> Option<EnergyDemand>
    {
        let state: PumpState = ctx.tileData();
        state.m_provider.unwrap_or_else(|| POWER.build()).demand()
    }

    fn receiveEnergy(&self, ctx: &mut BlockCtx, _face: i32, amount: i64, simulate: bool) -> i64
    {
        let mut state: PumpState = ctx.tileData();
        let mut provider: Provider = state.m_provider.unwrap_or_else(|| POWER.build());
        let taken: i64 = provider.take(amount, simulate);

        if !simulate
        {
            state.m_provider = Some(provider);
            let _ = ctx.setTileData(&state);
        }

        taken
    }
}

struct PumpBlock;

#[block_behavior]
impl BlockBehavior for PumpBlock
{
    fn texture(&self, face: i32, _meta: i32) -> i32
    {
        render::slot(match face
        {
            0 => BOTTOM,
            1 => TOP,
            _ => SIDE,
        })
    }
}

fn onView(packet: ViewPacket)
{
    let world: World = World { dimension: packet.m_world };
    if !world.isAuthority()
    {
        TUBES.with(|tubes| tubes.borrow_mut().insert((packet.m_world, packet.m_pos), packet.m_tubeY));
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for sprite in [TOP, BOTTOM, SIDE, TUBE]
    {
        render::slot(sprite);
    }

    ctx.tileEntities().register(
        NAME,
        TileEntitySpec { inventory: 0, ticks: true, sync: false, behavior: None },
        Some(Rc::new(PumpTile)),
    )?;
    ctx.tileEntities().model(NAME, EntityModelSpec::default())?;
    ctx.blocks().register(
        NAME,
        BlockSpecBuilder::template("minecraft:block")
            .material("iron")
            .sound("metal")
            .hardness(HARDNESS)
            .texture(SIDE)
            .tileEntity(NAME)
            .tag(crate::machines::MACHINES)
            .behavior(PumpBlock),
    )?;
    ctx.channels().register(VIEW_CHANNEL, Direction::ToClient, |packet: ViewPacket, _player| onView(packet))?;

    // a tank over a mining well
    ctx.recipes()
        .shaped(NAME, ItemStack::of("buildcraft:pump", 1))
        .pattern(["T ", "W "])
        .key('T', "buildcraft:tank")
        .key('W', "buildcraft:mining_well")
        .build()
}
