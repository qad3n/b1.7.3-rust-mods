use crate::core::dir;
use crate::core::items::{self, gear, material};
use crate::core::power::{self, Provider, Tracker};
use b173_api::prelude::*;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

// buildcraft.energy.Engine and its three kinds, by the block's row
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Kind
{
    Redstone,
    Steam,
    Combustion,
}

pub(crate) struct EngineRow
{
    pub(crate) m_kind: Kind,
    pub(crate) m_name: &'static str,
    pub(crate) m_base: &'static str,
    m_maxEnergy: i32,
    m_maxExtracted: i32,
    m_maxReceived: i32,
    m_explosion: f32,
    // piston speed for the blue, green, yellow and red stages
    m_speeds: [f32; 4],
    pub(crate) m_ingredient: &'static str,
    pub(crate) m_gear: &'static str,
}

pub(crate) const ENGINES: [EngineRow; 3] = [
    EngineRow
    {
        m_kind: Kind::Redstone,
        m_name: "redstone_engine",
        m_base: "buildcraft:model/engine_base_wood",
        m_maxEnergy: 1000,
        m_maxExtracted: 1,
        m_maxReceived: 50,
        m_explosion: 1.0,
        m_speeds: [0.01, 0.02, 0.04, 0.08],
        m_ingredient: material::PLANKS,
        m_gear: gear::WOOD,
    },
    EngineRow
    {
        m_kind: Kind::Steam,
        m_name: "steam_engine",
        m_base: "buildcraft:model/engine_base_stone",
        m_maxEnergy: 10000,
        m_maxExtracted: 100,
        m_maxReceived: 200,
        m_explosion: 4.0,
        m_speeds: [0.02, 0.04, 0.08, 0.16],
        m_ingredient: material::COBBLESTONE,
        m_gear: gear::STONE,
    },
    EngineRow
    {
        m_kind: Kind::Combustion,
        m_name: "combustion_engine",
        m_base: "buildcraft:model/engine_base_iron",
        m_maxEnergy: 100000,
        m_maxExtracted: 500,
        m_maxReceived: 2000,
        m_explosion: 8.0,
        m_speeds: [0.04, 0.08, 0.16, 0.32],
        m_ingredient: material::IRON,
        m_gear: gear::IRON,
    },
];

mod fuel
{
    // TileEngine.OIL_BUCKET_TIME, a bucket of oil burns this long and the chamber holds ten
    pub const OIL_BUCKET_TIME: i32 = 10000;
    pub const CHAMBER_BUCKETS: i32 = 10;
    pub const BUCKET: i32 = 1000;
    pub const STEAM_PER_TICK: i32 = 1;
    pub const COMBUSTION_PER_TICK: i32 = 2;
    pub const SCALE: i32 = 1000;
    pub const EMPTY_BUCKET: &str = "minecraft:bucket";
}

mod timing
{
    // the redstone engine gains a unit a second while powered, idle engines tell clients once a second
    pub const REDSTONE_PERIOD: i64 = 20;
    pub const IDLE_SYNC: i64 = 20;
    // TileEngine.doWork keeps 95 hundredths of what other engines push in
    pub const PASS_ON: f32 = 0.95;
    pub const HALF: f32 = 0.5;
}

// Engine.EnergyStage, a quarter of the store each, past full the engine blows up
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) enum Stage
{
    #[default]
    Blue,
    Green,
    Yellow,
    Red,
    Explosion,
}

pub(crate) const SLOT: i32 = 0;

// TileEngine.progressPart: resting, pushing out before half stroke, coming back after it
mod stroke
{
    pub const REST: i32 = 0;
    pub const OUT: i32 = 1;
    pub const BACK: i32 = 2;
}

const STAGES: [Stage; 4] = [Stage::Blue, Stage::Green, Stage::Yellow, Stage::Red];

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub(crate) struct EngineState
{
    m_started: bool,
    pub(crate) m_orientation: i32,
    pub(crate) m_progress: f32,
    pub(crate) m_progressPart: i32,
    pub(crate) m_energy: i32,
    pub(crate) m_burnTime: i32,
    pub(crate) m_totalBurnTime: i32,
    m_provider: Provider,
    m_sync: Tracker,
    // the energy a tick of the fuel now burning gives, from the job that filled the chamber
    m_rate: i32,
}

// what a client needs to draw and show an engine, sent when it changes as TileEngine's network updates
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub(crate) struct View
{
    pub(crate) m_orientation: i32,
    pub(crate) m_stage: Stage,
    pub(crate) m_progress: f32,
    pub(crate) m_progressPart: i32,
    pub(crate) m_speed: f32,
    pub(crate) m_burning: bool,
    pub(crate) m_scaledBurn: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ViewPacket
{
    m_world: i32,
    m_pos: BlockPos,
    m_view: View,
}

const VIEW_CHANNEL: &str = "engine";

thread_local!
{
    // engines as their clients see them, the single player client reads the server's state
    static VIEWS: RefCell<HashMap<(i32, BlockPos), View>> = RefCell::new(HashMap::new());
}

pub(crate) fn rowOf(world: World, pos: BlockPos) -> Option<&'static EngineRow>
{
    let block: i32 = world.block(pos);
    ENGINES.iter().find(|row| blockIs(block, row.m_name))
}

// an exploding engine still draws red
fn shownStage(row: &EngineRow, energy: i32) -> Stage
{
    match stageOf(row, energy)
    {
        Stage::Explosion => Stage::Red,
        stage => stage,
    }
}

// the burn ticks a combustion chamber holds, ten oil buckets' worth
fn chamber() -> i32 { fuel::OIL_BUCKET_TIME * fuel::CHAMBER_BUCKETS }

// what a tick of a job's fuel gives, a total spread over its duration, nothing for a job that names no energy
fn rate(job: &JobMatch) -> i32
{
    let Some(energy) = &job.energy_out
    else
    {
        return 0;
    };

    let amount: i64 = if energy.per_tick
    {
        energy.amount
    }
    else
    {
        energy.amount / i64::from(job.duration.max(1))
    };
    i32::try_from(amount).unwrap_or(i32::MAX)
}

// the job burning a fluid and the millibuckets it takes, asked with the most a chamber could hold since a job needs a
// whole bucket to match and a pipe pours less
fn fluidJob(row: &EngineRow, stack: &FluidStack) -> Option<(JobMatch, i32)>
{
    let offered: FluidStack = FluidStack { fluid: stack.fluid.clone(), amount: i32::MAX };
    let job: JobMatch = matchJob(row.m_name, &[], &[offered])?;
    let bucket: i32 = job.fluid_inputs.first()?.amount;
    (bucket > 0 && job.duration > 0).then_some((job, bucket))
}

fn stageIndex(stage: Stage) -> usize { STAGES.iter().position(|shown| *shown == stage).unwrap_or(STAGES.len() - 1) }

pub(crate) fn stageOf(row: &EngineRow, energy: i32) -> Stage
{
    const QUARTER: f64 = 25.0;

    let percent: f64 = f64::from(energy) / f64::from(row.m_maxEnergy) * QUARTER * 4.0;
    match percent
    {
        percent if percent <= QUARTER => Stage::Blue,
        percent if percent <= QUARTER * 2.0 => Stage::Green,
        percent if percent <= QUARTER * 3.0 => Stage::Yellow,
        percent if percent <= QUARTER * 4.0 => Stage::Red,
        _ => Stage::Explosion,
    }
}

fn speed(row: &EngineRow, energy: i32) -> f32
{
    match stageOf(row, energy)
    {
        Stage::Explosion => 0.0,
        stage => row.m_speeds[stageIndex(stage)],
    }
}

fn burning(row: &EngineRow, world: World, pos: BlockPos, state: &EngineState) -> bool
{
    match row.m_kind
    {
        Kind::Redstone => world.isPowered(pos),
        Kind::Steam => state.m_burnTime > 0,
        Kind::Combustion => state.m_burnTime > 0 && world.isPowered(pos),
    }
}

fn view(row: &EngineRow, world: World, pos: BlockPos, state: &EngineState) -> View
{
    let scaled: i32 = if state.m_totalBurnTime == 0
    {
        0
    }
    else
    {
        state.m_burnTime * fuel::SCALE / state.m_totalBurnTime
    };
    View
    {
        m_orientation: state.m_orientation,
        m_stage: shownStage(row, state.m_energy),
        m_progress: state.m_progress,
        m_progressPart: state.m_progressPart,
        m_speed: speed(row, state.m_energy),
        m_burning: burning(row, world, pos, state),
        m_scaledBurn: scaled,
    }
}

// the engine as this side shows it: the server's state in single player, the last packet on a client
pub(crate) fn shown(world: World, pos: BlockPos) -> Option<View>
{
    if world.isAuthority()
    {
        let row: &EngineRow = rowOf(world, pos)?;
        return Some(view(row, world, pos, &world.tileData(pos)));
    }

    VIEWS.with(|views| views.borrow().get(&(world.dimension, pos)).cloned())
}

fn publish(world: World, pos: BlockPos, view: View)
{
    let packet: ViewPacket = ViewPacket { m_world: world.dimension, m_pos: pos, m_view: view };
    if let Err(error) = send(&format!("buildcraft:{VIEW_CHANNEL}"), Target::Tracking(world.dimension, pos), &packet)
    {
        warn(&format!("could not send an engine update: {error}"));
    }
}

fn start(row: &EngineRow, state: &mut EngineState)
{
    if state.m_started
    {
        return;
    }

    state.m_started = true;
    state.m_provider = Provider::new(0, 1, row.m_maxReceived, 1, row.m_maxEnergy);
}

// Engine.addEnergy, an engine past its store explodes from its block corner as createExplosion is given
fn addEnergy(row: &EngineRow, world: World, pos: BlockPos, state: &mut EngineState, amount: i32)
{
    state.m_energy += amount;

    let corner: [f64; 3] = [f64::from(pos.x), f64::from(pos.y), f64::from(pos.z)];
    if stageOf(row, state.m_energy) == Stage::Explosion
        && let Err(error) = world.explode(corner, row.m_explosion)
    {
        warn(&format!("an engine could not explode: {error}"));
    }
}

// Engine.extractEnergy within a receiver's packet sizes, capped by what the engine gives at once
fn extractable(row: &EngineRow, state: &EngineState, min: i64, max: i64) -> i32
{
    let energy: i64 = i64::from(state.m_energy);
    if energy < min
    {
        return 0;
    }

    let cap: i64 = max.min(i64::from(row.m_maxExtracted));
    energy.min(cap) as i32
}

// isPoweredTile for any mod: the block in front takes this mod's energy or one that converts to it
fn receiver(world: World, pos: BlockPos, face: i32) -> Option<EnergyDemand>
{
    world.energyDemand(dir::step(pos, face), dir::reverse(face), &power::kind())
}

fn push(row: &EngineRow, world: World, pos: BlockPos, state: &mut EngineState)
{
    let face: i32 = state.m_orientation;
    let Some(demand) = receiver(world, pos, face)
    else
    {
        return;
    };

    let offered: i32 = extractable(row, state, demand.min, demand.max);
    if offered <= 0
    {
        return;
    }

    let taken: i64 =
        world.sendEnergy(dir::step(pos, face), dir::reverse(face), &power::kind(), i64::from(offered), false);
    state.m_energy -= taken.clamp(0, i64::from(offered)) as i32;
}

fn burnSteam(world: World, pos: BlockPos, state: &mut EngineState, powered: bool, row: &EngineRow)
{
    if state.m_burnTime > 0
    {
        state.m_burnTime -= 1;
        addEnergy(row, world, pos, state, fuel::STEAM_PER_TICK);
    }

    if state.m_burnTime != 0 || !powered
    {
        return;
    }

    let Some(stack) = world.tileSlot(pos, SLOT)
    else
    {
        return;
    };

    let ticks: i32 = machineFuel(row.m_name, &stack, true);
    state.m_burnTime = ticks;
    state.m_totalBurnTime = ticks;
    if ticks <= 0
    {
        return;
    }

    // a burnt container leaves what it drains to, as the lava bucket leaves a bucket
    let left: Option<ItemStack> = match drainItem(&stack)
    {
        Some((empty, _)) => empty,
        None => (stack.count > 1).then(|| ItemStack { count: stack.count - 1, ..stack }),
    };
    let _ = world.setTileSlot(pos, SLOT, left);
}

fn burnOil(world: World, pos: BlockPos, state: &mut EngineState, powered: bool, row: &EngineRow)
{
    if powered && state.m_burnTime > 0
    {
        state.m_burnTime -= 1;
        addEnergy(row, world, pos, state, state.m_rate);
    }

    let Some(stack) = world.tileSlot(pos, SLOT)
    else
    {
        return;
    };
    let Some(job) = matchJob(row.m_name, &[Some(stack.clone())], &[])
    else
    {
        return;
    };

    state.m_totalBurnTime = chamber();

    if state.m_burnTime + job.duration > state.m_totalBurnTime
    {
        return;
    }

    // the job's first output, the empty bucket, takes the full bucket's place
    let left: Option<ItemStack> = match job.outputs.first().and_then(|output| output.stack.clone())
    {
        Some(output) => Some(ItemStack::fromRaw(output)),
        None => (stack.count > 1).then(|| ItemStack { count: stack.count - 1, ..stack }),
    };
    if let Err(error) = world.setTileSlot(pos, SLOT, left)
    {
        warn(&format!("a combustion engine at {pos:?} could not empty its bucket: {error}"));
        return;
    }

    state.m_burnTime += job.duration;
    state.m_rate = rate(&job);
}

// TileEngine.updateEntity on the server: the piston pushes a packet at half stroke
fn work(row: &EngineRow, world: World, pos: BlockPos, state: &mut EngineState)
{
    if state.m_provider.update(world, pos)
    {
        let pushed: i32 = state.m_provider.useEnergy(1, row.m_maxReceived, true);
        addEnergy(row, world, pos, state, (pushed as f32 * timing::PASS_ON) as i32);
    }

    state.m_provider.decay(world);

    let powered: bool = world.isPowered(pos);

    // Engine.update and EngineWood.update
    if !powered && state.m_energy > 1
    {
        state.m_energy -= 1;
    }

    if row.m_kind == Kind::Redstone && powered && world.time() % timing::REDSTONE_PERIOD == 0
    {
        state.m_energy += 1;
    }

    let before: View = view(row, world, pos, state);

    if state.m_progressPart != stroke::REST
    {
        state.m_progress += speed(row, state.m_energy);

        if state.m_progress > timing::HALF && state.m_progressPart == stroke::OUT
        {
            state.m_progressPart = stroke::BACK;
            push(row, world, pos, state);
        }
        else if state.m_progress >= 1.0
        {
            state.m_progress = 0.0;
            state.m_progressPart = stroke::REST;
        }
    }
    else if powered
    {
        let ready: bool = receiver(world, pos, state.m_orientation)
            .is_some_and(|demand| extractable(row, state, demand.min, demand.max) > 0);
        if ready
        {
            state.m_progressPart = stroke::OUT;
            publish(world, pos, view(row, world, pos, state));
        }
    }
    else if state.m_sync.due(world.time(), timing::IDLE_SYNC)
    {
        publish(world, pos, view(row, world, pos, state));
    }

    match row.m_kind
    {
        Kind::Steam => burnSteam(world, pos, state, powered, row),
        Kind::Combustion => burnOil(world, pos, state, powered, row),
        Kind::Redstone =>
        {}
    }

    let after: View = view(row, world, pos, state);
    if after.m_stage != before.m_stage
        || after.m_burning != before.m_burning
        || after.m_scaledBurn != before.m_scaledBurn
    {
        publish(world, pos, after);
    }
}

// the client's piston follows the server's speed between updates
fn animate(world: World, pos: BlockPos)
{
    VIEWS.with(|views|
    {
        if let Some(view) = views.borrow_mut().get_mut(&(world.dimension, pos))
            && view.m_progressPart != stroke::REST
        {
            view.m_progress += view.m_speed;

            if view.m_progress > 1.0
            {
                view.m_progressPart = stroke::REST;
                view.m_progress = 0.0;
            }
        }
    });
}

// TileEngine.switchOrientation, the next face after the current one with a receiver
pub(crate) fn switchOrientation(world: World, pos: BlockPos, state: &mut EngineState)
{
    let current: i32 = state.m_orientation;
    if let Some(face) = (current + 1..=current + dir::COUNT)
        .map(|step| step % dir::COUNT)
        .find(|face| receiver(world, pos, *face).is_some())
    {
        state.m_orientation = face;
        world.markBlockForRender(pos);
    }
}

pub(crate) struct EngineTile;

#[tile_behavior]
impl TileBehavior for EngineTile
{
    fn onTick(&self, ctx: &mut BlockCtx)
    {
        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();
        if !ctx.isAuthority()
        {
            animate(world, pos);
            return;
        }

        let Some(row) = rowOf(world, pos)
        else
        {
            return;
        };
        let mut state: EngineState = ctx.tileData();
        start(row, &mut state);
        work(row, world, pos, &mut state);

        if let Err(error) = ctx.setTileData(&state)
        {
            warn(&format!("an engine at {pos:?} could not save: {error}"));
        }
    }

    fn onRemoved(&self, ctx: &mut BlockCtx)
    {
        VIEWS.with(|views| views.borrow_mut().remove(&(ctx.world().dimension, ctx.pos())));
    }

    fn pose(&self, ctx: &mut BlockCtx, _partial: f32, _ticks: i32) -> Pose { super::model::pose(ctx) }

    // TileEngine as IPowerReceptor, other engines push into it
    fn energyDemand(&self, ctx: &mut BlockCtx, _face: i32) -> Option<EnergyDemand>
    {
        let row: &EngineRow = rowOf(ctx.world(), ctx.pos())?;
        let mut state: EngineState = ctx.tileData();
        start(row, &mut state);
        state.m_provider.demand()
    }

    fn receiveEnergy(&self, ctx: &mut BlockCtx, _face: i32, amount: i64, simulate: bool) -> i64
    {
        let Some(row) = rowOf(ctx.world(), ctx.pos())
        else
        {
            return 0;
        };
        let mut state: EngineState = ctx.tileData();
        start(row, &mut state);

        let taken: i64 = state.m_provider.take(amount, simulate);
        if !simulate
        {
            let _ = ctx.setTileData(&state);
        }

        taken
    }

    // TileEngine.fill, only the combustion engine burns liquid, ten oil buckets' worth at most
    fn fillFluid(&self, ctx: &mut BlockCtx, _face: i32, stack: &FluidStack, simulate: bool) -> i32
    {
        let world: World = ctx.world();
        let Some(row) = rowOf(world, ctx.pos()).filter(|row| row.m_kind == Kind::Combustion)
        else
        {
            return 0;
        };
        let Some((job, bucket)) = fluidJob(row, stack)
        else
        {
            return 0;
        };

        let mut state: EngineState = ctx.tileData();
        let capacity: i32 = chamber();
        let room: i32 = (capacity - state.m_burnTime).max(0) * bucket / job.duration;
        let used: i32 = stack.amount.min(room);

        if !simulate
        {
            state.m_totalBurnTime = capacity;
            state.m_burnTime += used * job.duration / bucket;
            state.m_rate = rate(&job);

            if let Err(error) = ctx.setTileData(&state)
            {
                warn(&format!("a combustion engine at {:?} could not save: {error}", ctx.pos()));
            }
        }

        used
    }
}

fn onView(packet: ViewPacket)
{
    let world: World = World { dimension: packet.m_world };
    if world.isAuthority()
    {
        return;
    }

    VIEWS.with(|views| views.borrow_mut().insert((packet.m_world, packet.m_pos), packet.m_view));
    world.markBlockForRender(packet.m_pos);
}

// the engines as recipe viewers show them, what each burns and the energy it gives
pub(crate) fn registerPages(ctx: &mut RegisterCtx) -> Result<()>
{
    for row in &ENGINES
    {
        ctx.machines().register(row.m_name, None, &[row.m_name])?;
    }

    let [redstone, steam, combustion] = [&ENGINES[0], &ENGINES[1], &ENGINES[2]].map(|row| row.m_name);
    let chambers: String = format!("The chamber holds {} buckets", fuel::CHAMBER_BUCKETS);

    ctx.machines()
        .recipe(redstone)
        .note(&format!("1 MJ every {} ticks while redstone powered", timing::REDSTONE_PERIOD))
        .build()?;

    // every furnace fuel known while BuildCraft registers, read through the loader so burn times stay the furnace's
    let mut listed: HashSet<ItemRef> = HashSet::new();
    for stack in itemCatalog()
    {
        let ticks: i32 = machineFuel(steam, &stack, true);
        if ticks <= 0 || !listed.insert(stack.item.clone())
        {
            continue;
        }

        let stack: ItemStack = ItemStack::of(stack.item, 1);

        let per: i32 = fuel::STEAM_PER_TICK;
        ctx.machines()
            .recipe(steam)
            .input(stack.withDamage(-1))
            .note(&format!("{} MJ at {per} MJ/t", ticks * per))
            .note("Takes new fuel only while redstone powered")
            .build()?;
    }

    // TileEngine burns oil only, other mods add jobs for their fuels to the same machine
    let per: i64 = i64::from(fuel::COMBUSTION_PER_TICK);
    ctx.machines()
        .job(combustion, fuel::OIL_BUCKET_TIME)
        .inputTag(super::oil::OIL_BUCKETS, 1)
        .output(fuel::EMPTY_BUCKET, 1)
        .energyOut(power::MJ, per, true)
        .note(&chambers)
        .build()?;
    ctx.machines()
        .job(combustion, fuel::OIL_BUCKET_TIME)
        .fluidInputTag(super::oil::OIL_TAG, fuel::BUCKET)
        .energyOut(power::MJ, per, true)
        .note(&chambers)
        .build()
}

pub(crate) fn registerChannel(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.channels().register(VIEW_CHANNEL, Direction::ToClient, |packet: ViewPacket, _player| onView(packet))
}

// BlockEngine.blockActivated: a wrench turns it, the steam and combustion engines open their screens
pub(crate) fn activated(ctx: &mut BlockCtx, player: Player) -> bool
{
    let world: World = ctx.world();
    let pos: BlockPos = ctx.pos();
    let Some(row) = rowOf(world, pos)
    else
    {
        return true;
    };

    if !ctx.isAuthority()
    {
        return true;
    }

    if items::holdsWrench(player)
    {
        let mut state: EngineState = ctx.tileData();
        start(row, &mut state);
        switchOrientation(world, pos, &mut state);
        let _ = ctx.setTileData(&state);
        publish(world, pos, view(row, world, pos, &state));
        return true;
    }

    if row.m_kind != Kind::Redstone
    {
        let _ = player.openScreen(row.m_name, Some(pos), Vec::new());
    }

    true
}

// BlockEngine.onBlockPlaced faces up, then the first receiver around it
pub(crate) fn placed(ctx: &mut BlockCtx)
{
    let world: World = ctx.world();
    let pos: BlockPos = ctx.pos();
    let Some(row) = rowOf(world, pos)
    else
    {
        return;
    };
    if !ctx.isAuthority()
    {
        return;
    }

    let mut state: EngineState = ctx.tileData();
    start(row, &mut state);
    state.m_orientation = dir::UP;
    switchOrientation(world, pos, &mut state);
    let _ = ctx.setTileData(&state);
    publish(world, pos, view(row, world, pos, &state));
}

// BlockEngine.isPipeConnected, never for the redstone engine and never through its output
pub(crate) fn acceptsPipe(world: World, pos: BlockPos, face: i32) -> bool
{
    let Some(row) = rowOf(world, pos)
    else
    {
        return false;
    };
    let orientation: i32 = shown(world, pos).map_or(dir::UP, |view| view.m_orientation);
    row.m_kind != Kind::Redstone && face != orientation
}
