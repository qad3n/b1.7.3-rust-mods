use crate::blocks::{self, Kind, SIDES, Tile};
use crate::{ether, items, screens};
use b173_api::prelude::*;
use b173_api::world::{Player, World};
use std::cell::RefCell;
use wirelessredstone_api::FREQUENCIES;

pub(crate) const FREQ: &str = "freq";
pub(crate) const PERIOD: &str = "period";
pub(crate) const ROUTE: &str = "route";
pub(crate) const ITEM_FREQ: &str = "item_freq";
pub(crate) const PULSE: &str = "pulse";
pub(crate) const SNIFF: &str = "sniff";
pub(crate) const ACTIVE: &str = "active";
pub(crate) const LOCATE: &str = "locate";
pub(crate) const CLOSEST: &str = "closest";

// the remote's pulse lasted 1000 ms, its thread slept that long
const PULSE_TICKS: i32 = 20;
// Container.canInteractWith in the source, the square of 8 blocks from the block center
const REACH_SQUARED: f64 = 64.0;

#[derive(Serialize, Deserialize)]
pub(crate) struct Step
{
    pub m_pos: Option<BlockPos>,
    pub m_step: i32,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Route
{
    pub m_pos: BlockPos,
    pub m_side: u8,
    pub m_indirect: bool,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Pulse
{
    pub m_pos: BlockPos,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Locate
{
    pub m_freq: i32,
}

#[derive(Serialize, Deserialize, Default)]
pub(crate) struct Active
{
    pub m_freqs: Vec<u16>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Closest
{
    pub m_pos: Option<[i32; 2]>,
}

struct Pending
{
    m_world: World,
    m_pos: BlockPos,
    m_freq: i32,
    m_ticks: i32,
}

thread_local!
{
    static PULSES: RefCell<Vec<Pending>> = const { RefCell::new(Vec::new()) };
}

fn world(player: Player) -> Option<World> { player.info().map(|info| World { dimension: info.world }) }

// the player's world when the player stands within reach of the block, as Container.canInteractWith
fn reach(player: Player, pos: BlockPos) -> Option<World>
{
    let info: EntityInfo = player.info()?;
    let center: [f64; 3] = [pos.x as f64 + 0.5, pos.y as f64 + 0.5, pos.z as f64 + 0.5];
    let distance: f64 = info.pos.iter().zip(center).map(|(a, b)| (a - b) * (a - b)).sum();
    (distance <= REACH_SQUARED).then_some(World { dimension: info.world })
}

fn saveTile(world: World, pos: BlockPos, tile: &Tile)
{
    if let Err(error) = world.setTileData(pos, tile)
    {
        warn(&format!("wireless tile at {pos:?}: {error:?}"));
    }
}

// GuiRedstoneWireless.actionPerformed, frequencies wrap around 0 and 9999
fn stepFreq(message: Step, player: Player)
{
    let Some(pos) = message.m_pos
    else
    {
        return;
    };

    let Some(world) = reach(player, pos)
    else
    {
        return;
    };

    let Some(kind) = blocks::kindAt(world, pos)
    else
    {
        return;
    };

    if !screens::FREQ_STEPS.iter().any(|button| button.m_step == message.m_step)
    {
        return;
    }

    let mut tile: Tile = world.tileData(pos);
    tile.setFreq((tile.m_freq + message.m_step).rem_euclid(FREQUENCIES));
    saveTile(world, pos, &tile);
    blocks::join(world, pos, kind);
}

// GuiRedstoneWirelessClocker.actionPerformed, the period stays between 200 ms and 2000000000 ms
fn stepPeriod(message: Step, player: Player)
{
    let Some(pos) = message.m_pos
    else
    {
        return;
    };

    let Some(world) = reach(player, pos)
    else
    {
        return;
    };

    let valid: bool = blocks::kindAt(world, pos) == Some(Kind::Clocker)
        && screens::CLOCK_STEPS.iter().any(|button| button.m_step == message.m_step);

    if !valid
    {
        return;
    }

    let mut tile: Tile = world.tileData(pos);
    tile.setPeriod(tile.m_period.saturating_add(message.m_step));
    saveTile(world, pos, &tile);
}

// GuiRedstoneWirelessPowerDirector.actionPerformed
fn flipRoute(message: Route, player: Player)
{
    let Some(world) = reach(player, message.m_pos)
    else
    {
        return;
    };

    let side: usize = usize::from(message.m_side);
    let valid: bool = side < SIDES && blocks::kindAt(world, message.m_pos) == Some(Kind::Receiver);

    if !valid
    {
        return;
    }

    let mut tile: Tile = world.tileData(message.m_pos);

    if message.m_indirect
    {
        tile.flipIndirect(side);
    }
    else
    {
        tile.flipDirect(side);
    }

    saveTile(world, message.m_pos, &tile);
    blocks::notifyAround(world, message.m_pos);
}

// the remote and triangulator guis, the frequency lives on the held stack
fn stepItemFreq(message: Step, player: Player)
{
    let held: bool =
        player.held().is_some_and(|stack| stack.item.is(items::REMOTE) || stack.item.is(items::TRIANGULATOR));
    let valid: bool = held && screens::FREQ_STEPS.iter().any(|button| button.m_step == message.m_step);

    if !valid
    {
        return;
    }

    let at: StackRef = StackRef::Held { player: player.id };
    let freq: i32 = (stackComponent::<i32>(at, items::FREQ) + message.m_step).rem_euclid(FREQUENCIES);

    if let Err(error) = setStackComponent(at, items::FREQ, &freq)
    {
        warn(&format!("item frequency: {error:?}"));
    }
}

// ThreadWirelessRemote, a transmitter that stays on for a second where the remote was used
fn pulse(message: Pulse, player: Player)
{
    let remote: bool = player.held().is_some_and(|stack| stack.item.is(items::REMOTE));

    let Some(world) = reach(player, message.m_pos).filter(|_| remote)
    else
    {
        return;
    };

    let freq: i32 = stackComponent::<i32>(StackRef::Held { player: player.id }, items::FREQ);
    ether::addTransmitter(world, message.m_pos, freq);
    ether::setTransmitterState(world, message.m_pos, freq, true);

    let pending: Pending = Pending { m_world: world, m_pos: message.m_pos, m_freq: freq, m_ticks: PULSE_TICKS };
    PULSES.with(|pulses| pulses.borrow_mut().push(pending));
}

pub(crate) fn tickPulses()
{
    let ended: Vec<Pending> = PULSES.with(|pulses|
    {
        let mut pulses = pulses.borrow_mut();

        for pending in pulses.iter_mut()
        {
            pending.m_ticks -= 1;
        }

        let (ended, running): (Vec<Pending>, Vec<Pending>) = pulses.drain(..).partition(|pending| pending.m_ticks <= 0);
        *pulses = running;
        ended
    });

    for pending in ended
    {
        ether::removeTransmitter(pending.m_world, pending.m_pos, pending.m_freq);
    }
}

fn reply<T: Serialize>(channel: &str, player: Player, message: &T)
{
    if let Err(error) = send(channel, Target::Player(player.id), message)
    {
        warn(&format!("{channel} reply: {error:?}"));
    }
}

fn sniff(_: (), player: Player)
{
    let Some(world) = world(player)
    else
    {
        return;
    };

    reply(ACTIVE, player, &Active { m_freqs: ether::activeFrequencies(world) });
}

fn locate(message: Locate, player: Player)
{
    let Some(info) = player.info()
    else
    {
        return;
    };

    let world: World = World { dimension: info.world };
    let [x, _, z] = info.pos;
    let pos: Option<[i32; 2]> = ether::closestActive(world, x as i32, z as i32, message.m_freq);
    reply(CLOSEST, player, &Closest { m_pos: pos });
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.channels().register::<Step>(FREQ, Direction::ToServer, stepFreq)?;
    ctx.channels().register::<Step>(PERIOD, Direction::ToServer, stepPeriod)?;
    ctx.channels().register::<Route>(ROUTE, Direction::ToServer, flipRoute)?;
    ctx.channels().register::<Step>(ITEM_FREQ, Direction::ToServer, stepItemFreq)?;
    ctx.channels().register::<Pulse>(PULSE, Direction::ToServer, pulse)?;
    ctx.channels().register::<()>(SNIFF, Direction::ToServer, sniff)?;
    ctx.channels().register::<Locate>(LOCATE, Direction::ToServer, locate)?;
    ctx.channels().register::<Active>(ACTIVE, Direction::ToClient, |message, _| screens::sniffed(message.m_freqs))?;
    ctx.channels()
        .register::<Closest>(CLOSEST, Direction::ToClient, |message, _| crate::triangulator::located(message.m_pos))
}
