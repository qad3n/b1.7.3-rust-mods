use super::{Faces, MINING_PIPE, facePlacer, registerFacing, unbreakable};
use crate::core::items::{gear, material};
use crate::core::power::{Provider, Setup};
use crate::core::stacks;
use b173_api::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// TileMiningWell: twenty five energy per block, drilling a shaft of mining pipe straight down
const NAME: &str = "mining_well";
const POWER: Setup =
    Setup { m_latency: 50, m_minReceived: 25, m_maxReceived: 25, m_minActivation: 25, m_maxStored: 1000, m_loss: None };
const DIG_COST: i32 = 25;
const HARDNESS: f32 = 1.5;
const RESISTANCE: f32 = 10.0;

const FACES: Faces = Faces
{
    m_front: "buildcraft:block/mining_well_front",
    m_top: "buildcraft:block/mining_well_top",
    m_side: "buildcraft:block/mining_well_side",
    m_bottom: Some("buildcraft:block/mining_well_back"),
    m_back: Some("buildcraft:block/mining_well_back"),
};

const ACTIVE_CHANNEL: &str = "mining_well";

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
struct ActivePacket
{
    m_world: i32,
    m_pos: BlockPos,
    m_digging: bool,
}

thread_local!
{
    // whether each mining well a client sees still digs, for the glow of the machines around it
    static ACTIVE: RefCell<HashMap<(i32, BlockPos), bool>> = RefCell::new(HashMap::new());
}

// IMachine.isActive, a mining well is active until it reaches bedrock
pub(crate) fn active(world: World, pos: BlockPos) -> bool
{
    if world.isAuthority()
    {
        let state: WellState = world.tileData(pos);
        return state.m_provider.is_none() || state.m_digging;
    }

    ACTIVE.with(|active| active.borrow().get(&(world.dimension, pos)).copied()).unwrap_or(true)
}

fn publish(world: World, pos: BlockPos, digging: bool)
{
    let packet: ActivePacket = ActivePacket { m_world: world.dimension, m_pos: pos, m_digging: digging };
    if let Err(error) = send(&format!("buildcraft:{ACTIVE_CHANNEL}"), Target::Tracking(world.dimension, pos), &packet)
    {
        warn(&format!("could not send a mining well update: {error}"));
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub(crate) struct WellState
{
    m_provider: Option<Provider>,
    pub(crate) m_digging: bool,
}

impl WellState
{
    fn provider(&mut self) -> &mut Provider { self.m_provider.get_or_insert_with(|| POWER.build()) }
}

// TileMiningWell.doWork: below the shaft the next block goes to the machine's output and the shaft grows
fn dig(world: World, pos: BlockPos, state: &mut WellState)
{
    if state.provider().useEnergy(DIG_COST, DIG_COST, true) < DIG_COST
    {
        return;
    }

    let mut depth: BlockPos = pos.offset(0, -1, 0);

    while world.is(depth, MINING_PIPE)
    {
        depth = depth.offset(0, -1, 0);
    }

    if depth.y < 0 || unbreakable(world, depth)
    {
        state.m_digging = false;
        return;
    }

    let mined: bool = !world.is(depth, crate::core::AIR);
    let drop: Option<ItemStack> = world.blockDropStack(depth);

    if let Err(error) = world.place(depth, MINING_PIPE)
    {
        warn(&format!("the mining well at {pos:?} could not extend its shaft: {error}"));
        return;
    }

    if !mined
    {
        return;
    }

    if let Some(stack) = drop
    {
        stacks::output(world, pos, stack);
    }
}

struct WellTile;

#[tile_behavior]
impl TileBehavior for WellTile
{
    fn onTick(&self, ctx: &mut BlockCtx)
    {
        if !ctx.isAuthority()
        {
            return;
        }

        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();
        let mut state: WellState = ctx.tileData();
        if state.m_provider.is_none()
        {
            state.m_digging = true;
        }

        let before: bool = state.m_digging;

        if state.provider().update(world, pos)
        {
            dig(world, pos, &mut state);
        }

        state.provider().decay(world);

        if state.m_digging != before
        {
            publish(world, pos, state.m_digging);
        }

        let _ = ctx.setTileData(&state);
    }

    fn energyDemand(&self, ctx: &mut BlockCtx, _face: i32) -> Option<EnergyDemand>
    {
        let mut state: WellState = ctx.tileData();
        state.provider().demand()
    }

    fn receiveEnergy(&self, ctx: &mut BlockCtx, _face: i32, amount: i64, simulate: bool) -> i64
    {
        let mut state: WellState = ctx.tileData();
        let taken: i64 = state.provider().take(amount, simulate);

        if !simulate
        {
            let _ = ctx.setTileData(&state);
        }

        taken
    }
}

pub(crate) struct MachineBlock
{
    pub(crate) m_faces: &'static Faces,
}

#[block_behavior]
impl BlockBehavior for MachineBlock
{
    fn textureFor(&self, face: i32, state: &StateData) -> Option<String> { self.m_faces.sprite(face, state) }

    fn onPlacedBy(&self, ctx: &mut BlockCtx, entity: Entity) { facePlacer(ctx, entity); }

    // BlockMachineRoot.getBlockBrightness, half again as bright beside a working machine
    fn brightness(&self, ctx: &mut BlockCtx, base: f32) -> Option<f32>
    {
        const GLOW: f32 = 0.5;

        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();
        let near: bool = (-1..=1)
            .any(|dx| (-1..=1).any(|dy| (-1..=1).any(|dz| crate::machines::active(world, pos.offset(dx, dy, dz)))));
        near.then_some(base + GLOW)
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    FACES.reserve();
    ctx.channels().register(ACTIVE_CHANNEL, Direction::ToClient, |packet: ActivePacket, _player|
    {
        let world: World = World { dimension: packet.m_world };
        if !world.isAuthority()
        {
            ACTIVE.with(|active| active.borrow_mut().insert((packet.m_world, packet.m_pos), packet.m_digging));
        }
    })?;
    ctx.tileEntities().register(
        NAME,
        TileEntitySpec { inventory: 0, ticks: true, sync: false, behavior: None },
        Some(Rc::new(WellTile)),
    )?;
    ctx.blocks().register(
        NAME,
        BlockSpecBuilder::template("minecraft:block")
            .material("ground")
            .sound("stone")
            .hardness(HARDNESS)
            .resistance(RESISTANCE)
            .texture(FACES.m_front)
            .tileEntity(NAME)
            .tag(crate::machines::MACHINES)
            .tag(crate::machines::CONNECTORS)
            .behavior(MachineBlock { m_faces: &FACES }),
    )?;
    registerFacing(ctx, NAME)?;

    if !super::mining()
    {
        return Ok(());
    }

    ctx.recipes()
        .shaped(NAME, ItemStack::of("buildcraft:mining_well", 1))
        .pattern(["ipi", "igi", "iPi"])
        .keyTag('p', material::REDSTONE)
        .keyTag('i', material::IRON)
        .keyTag('g', gear::IRON)
        .key('P', "minecraft:iron_pickaxe")
        .build()
}
