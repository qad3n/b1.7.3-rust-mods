use crate::{ether, labels, screens};
use b173_api::prelude::*;
use b173_api::world::{BlockCtx, Player, World};
use std::collections::BTreeMap;
use std::rc::Rc;

pub(crate) const RECEIVER: &str = "receiver";
pub(crate) const TRANSMITTER: &str = "transmitter";
pub(crate) const CLOCKER: &str = "clocker";
pub(crate) const FREQUENCIES: i32 = 10000;
pub(crate) const SIDES: usize = 6;

mod clock
{
    pub const DEFAULT: i32 = 1000;
    pub const MIN: i32 = 200;
    pub const MAX: i32 = 2000000000;
    pub const TICK_MS: i64 = 50;
}

// the on state is metadata bit 0, as the source kept it
const POWERED: &str = "powered";
const STATES: [&str; 2] = ["false", "true"];

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind
{
    Receiver,
    Transmitter,
    Clocker,
}

// TileEntityRedstoneWireless and the clocker's extra fields, synced for the screens and labels
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
pub(crate) struct Tile
{
    pub m_freq: i32,
    pub m_direct: [bool; SIDES],
    pub m_indirect: [bool; SIDES],
    pub m_period: i32,
    pub m_running: bool,
}

impl Tile
{
    pub(crate) fn new() -> Self
    {
        Tile
        {
            m_freq: 0,
            m_direct: [true; SIDES],
            m_indirect: [true; SIDES],
            m_period: clock::DEFAULT,
            m_running: false,
        }
    }

    pub(crate) fn setFreq(&mut self, freq: i32) { self.m_freq = freq.clamp(0, FREQUENCIES - 1); }

    pub(crate) fn setPeriod(&mut self, period: i32) { self.m_period = period.clamp(clock::MIN, clock::MAX); }

    fn direct(&self, side: i32) -> bool
    {
        usize::try_from(side).ok().and_then(|side| self.m_direct.get(side)) == Some(&true)
    }

    fn indirect(&self, side: i32) -> bool
    {
        usize::try_from(side).ok().and_then(|side| self.m_indirect.get(side)) == Some(&true)
    }

    pub(crate) fn flipDirect(&mut self, side: usize)
    {
        if self.m_indirect[side] && self.m_direct[side]
        {
            self.flipIndirect(side);
        }

        self.m_direct[side] = !self.m_direct[side];
    }

    pub(crate) fn flipIndirect(&mut self, side: usize)
    {
        if !self.m_direct[side] && !self.m_indirect[side]
        {
            self.flipDirect(side);
        }

        self.m_indirect[side] = !self.m_indirect[side];
    }
}

impl Default for Tile
{
    fn default() -> Self { Self::new() }
}

fn powered(meta: i32) -> bool { meta & 1 == 1 }

pub(crate) fn kindAt(world: World, pos: BlockPos) -> Option<Kind>
{
    let state: StateData = world.blockState(pos)?;

    if state.block.namespace() != modId()
    {
        return None;
    }

    ROWS.iter().find(|row| row.m_name == state.block.path()).map(|row| row.m_kind)
}

// setBlockMetadataWithNotify redraws and syncs only blocks that notify themselves, so the source marked it after
fn setMeta(world: World, pos: BlockPos, on: bool)
{
    if let Err(error) = world.setMeta(pos, i32::from(on))
    {
        warn(&format!("wireless block state at {pos:?}: {error:?}"));
    }

    world.markBlockForRender(pos);
}

// BlockRedstoneWirelessR.notifyNeighbors, every block around and the receiver itself
pub(crate) fn notifyAround(world: World, pos: BlockPos)
{
    let offsets: [[i32; 3]; 7] = [[0, 0, 0], [-1, 0, 0], [1, 0, 0], [0, -1, 0], [0, 1, 0], [0, 0, -1], [0, 0, 1]];

    for [x, y, z] in offsets
    {
        let at: BlockPos = BlockPos { x: pos.x + x, y: pos.y + y, z: pos.z + z };

        if let Err(error) = world.notifyNeighbors(at, 0)
        {
            warn(&format!("receiver neighbour update at {at:?}: {error:?}"));
        }
    }
}

// BlockRedstoneWirelessR.updateTick, follows its frequency's state
pub(crate) fn updateReceiver(world: World, pos: BlockPos)
{
    if kindAt(world, pos) != Some(Kind::Receiver)
    {
        return;
    }

    let freq: i32 = world.tileData::<Tile>(pos).m_freq;
    let now: bool = ether::state(world, freq);

    if now == powered(world.meta(pos))
    {
        return;
    }

    setMeta(world, pos, now);
    notifyAround(world, pos);
}

fn setTransmitter(world: World, pos: BlockPos, freq: i32, on: bool)
{
    ether::setTransmitterState(world, pos, freq, on);

    if on != powered(world.meta(pos))
    {
        setMeta(world, pos, on);
    }
}

// the source's changeFreq run from updateEntity, joins the ether again whenever the frequency moved
pub(crate) fn join(world: World, pos: BlockPos, kind: Kind)
{
    let freq: i32 = world.tileData::<Tile>(pos).m_freq;
    let joined: Option<i32> = ether::joined(world, pos);

    if joined == Some(freq)
    {
        return;
    }

    match kind
    {
        Kind::Receiver =>
        {
            if let Some(old) = joined
            {
                ether::removeReceiver(world, pos, old);
            }

            ether::addReceiver(world, pos, freq);
            updateReceiver(world, pos);
        }

        Kind::Transmitter | Kind::Clocker =>
        {
            if let Some(old) = joined
            {
                ether::removeTransmitter(world, pos, old);
            }

            ether::addTransmitter(world, pos, freq);
            ether::setTransmitterState(world, pos, freq, powered(world.meta(pos)));
        }
    }
}

fn notify(ctx: &BlockCtx)
{
    if let Err(error) = ctx.world().notifyNeighbors(ctx.pos(), ctx.block())
    {
        warn(&format!("receiver neighbour update at {:?}: {error:?}", ctx.pos()));
    }
}

fn leave(ctx: &BlockCtx, kind: Kind)
{
    let freq: i32 = ctx.tileData::<Tile>().m_freq;

    match kind
    {
        Kind::Receiver => ether::removeReceiver(ctx.world(), ctx.pos(), freq),
        Kind::Transmitter | Kind::Clocker => ether::removeTransmitter(ctx.world(), ctx.pos(), freq),
    }
}

// onNeighborBlockChange of the transmitter and clocker: on when a real block powers it, off once unpowered
fn input(ctx: &BlockCtx, real: bool) -> Option<bool>
{
    let powered: bool = ctx.world().isPowered(ctx.pos());

    if real && powered
    {
        return Some(true);
    }

    (!powered).then_some(false)
}

fn setRunning(ctx: &BlockCtx, on: bool)
{
    let mut tile: Tile = ctx.tileData();

    if tile.m_running == on
    {
        return;
    }

    tile.m_running = on;

    if let Err(error) = ctx.setTileData(&tile)
    {
        warn(&format!("clocker state at {:?}: {error:?}", ctx.pos()));
    }
}

fn neighborChanged(ctx: &BlockCtx, kind: Kind, real: bool)
{
    if kind == Kind::Receiver
    {
        updateReceiver(ctx.world(), ctx.pos());
        return;
    }

    match (kind, input(ctx, real))
    {
        (Kind::Transmitter, Some(on)) => setTransmitter(ctx.world(), ctx.pos(), ctx.tileData::<Tile>().m_freq, on),
        (Kind::Clocker, Some(on)) => setRunning(ctx, on),
        _ =>
        {}
    }
}

// ThreadWirelessClocker ran every 10 ms over a global millisecond counter, a tick is 50 ms of it
fn clockTick(ctx: &BlockCtx)
{
    let tile: Tile = ctx.tileData();
    let on: bool = powered(i32::from(ctx.meta()));

    if !tile.m_running
    {
        if on
        {
            setTransmitter(ctx.world(), ctx.pos(), tile.m_freq, false);
        }

        return;
    }

    let now: i64 = ctx.world().time() * clock::TICK_MS;
    let period: i64 = i64::from(tile.m_period);

    if now / period != (now - clock::TICK_MS) / period
    {
        setTransmitter(ctx.world(), ctx.pos(), tile.m_freq, !on);
    }
}

struct WirelessBlock
{
    m_kind: Kind,
    m_screen: &'static str,
}

#[block_behavior]
impl BlockBehavior for WirelessBlock
{
    fn onActivated(&self, ctx: &mut BlockCtx, player: Player) -> bool
    {
        let configuring: bool = self.m_kind == Kind::Receiver
            && player.held().is_some_and(|stack| stack.item.is(crate::items::CONFIGURATOR));

        if configuring
        {
            return false;
        }

        if ctx.isAuthority()
            && let Err(error) = player.openScreen(self.m_screen, Some(ctx.pos()), Vec::new())
        {
            warn(&format!("open {} screen: {error:?}", self.m_screen));
        }

        true
    }

    fn onAdded(&self, ctx: &mut BlockCtx)
    {
        if !ctx.isAuthority()
        {
            return;
        }

        join(ctx.world(), ctx.pos(), self.m_kind);

        if self.m_kind == Kind::Receiver
        {
            notify(ctx);
        }

        // the source passes redstone wire as the changed neighbour, a real block
        neighborChanged(ctx, self.m_kind, true);
    }

    fn onRemoved(&self, ctx: &mut BlockCtx)
    {
        if !ctx.isAuthority()
        {
            return;
        }

        leave(ctx, self.m_kind);

        if self.m_kind == Kind::Receiver
        {
            notify(ctx);
        }
    }

    fn onNeighborChanged(&self, ctx: &mut BlockCtx, neighbor: i32)
    {
        if ctx.isAuthority()
        {
            neighborChanged(ctx, self.m_kind, neighbor > 0);
        }
    }

    fn poweringTo(&self, ctx: &mut BlockCtx, direction: i32) -> bool
    {
        self.m_kind == Kind::Receiver && powered(i32::from(ctx.meta())) && ctx.tileData::<Tile>().direct(direction)
    }

    fn indirectlyPoweringTo(&self, ctx: &mut BlockCtx, direction: i32) -> bool
    {
        ctx.tileData::<Tile>().indirect(direction) && self.poweringTo(ctx, direction)
    }

    fn canProvidePower(&self) -> bool { self.m_kind == Kind::Receiver }
}

struct WirelessTile
{
    m_kind: Kind,
}

#[tile_behavior]
impl TileBehavior for WirelessTile
{
    fn onTick(&self, ctx: &mut BlockCtx)
    {
        if !ctx.isAuthority()
        {
            return;
        }

        join(ctx.world(), ctx.pos(), self.m_kind);

        if self.m_kind == Kind::Clocker
        {
            clockTick(ctx);
        }
    }

    fn pose(&self, ctx: &mut BlockCtx, _partial: f32, _ticks: i32) -> Pose
    {
        labels::pose(ctx.tileData::<Tile>().m_freq)
    }
}

struct Row
{
    m_name: &'static str,
    m_kind: Kind,
    m_side: &'static str,
}

const ROWS: [Row; 3] = [
    Row { m_name: RECEIVER, m_kind: Kind::Receiver, m_side: "receiver" },
    Row { m_name: TRANSMITTER, m_kind: Kind::Transmitter, m_side: "transmitter" },
    Row { m_name: CLOCKER, m_kind: Kind::Clocker, m_side: "clocker" },
];

fn faces(side: &str, state: &str) -> FaceTextures
{
    let top: Option<String> = Some(format!("wirelessredstone:block/top_{state}"));
    let side: Option<String> = Some(format!("wirelessredstone:block/{side}_{state}"));
    FaceTextures { top: top.clone(), bottom: top, side, front: None }
}

fn registerBlock(ctx: &mut RegisterCtx, row: &Row) -> Result<()>
{
    let spec: TileEntitySpec = TileEntitySpec { inventory: 0, ticks: true, sync: true, behavior: None };
    ctx.tileEntities().register(row.m_name, spec, Some(Rc::new(WirelessTile { m_kind: row.m_kind })))?;
    ctx.tileEntities().model(row.m_name, EntityModelSpec::default())?;
    ctx.blocks().register(
        row.m_name,
        BlockSpecBuilder::template("minecraft:block")
            .material("circuits")
            .hardness(1.0)
            .sound("metal")
            .opacity(0)
            .faces(faces(row.m_side, "off"))
            .tileEntity(row.m_name)
            .behavior(WirelessBlock { m_kind: row.m_kind, m_screen: screens::screenFor(row.m_kind) }),
    )?;

    let property: PropertySpec = PropertySpec { name: POWERED.to_owned(), values: STATES.map(str::to_owned).to_vec() };
    let on: StateVariantSpec = StateVariantSpec
    {
        when: BTreeMap::from([(POWERED.to_owned(), STATES[1].to_owned())]),
        texture: None,
        textures: Some(faces(row.m_side, "on")),
    };
    ctx.blocks().states(
        row.m_name,
        BlockStatesSpec { properties: vec![property], variants: vec![on], ..BlockStatesSpec::default() },
    )
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for row in &ROWS
    {
        registerBlock(ctx, row)?;
    }

    Ok(())
}
