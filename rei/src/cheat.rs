use crate::catalog::{self, Entry, Key};
use b173_api::prelude::*;
use serde::de::DeserializeOwned;
use std::cell::Cell;

mod channel
{
    pub const TOGGLE: &str = "cheat";
    pub const STATE: &str = "cheat_state";
    pub const GRAB: &str = "grab";
    pub const TRASH: &str = "trash";
}

mod text
{
    pub const NO_PERMISSION: &str = "rei.text.rei.no_permission_cheat";
    pub const FAILED: &str = "rei.text.rei.failed_cheat_items";
}

// the mode button reads R, or a red G while a click gives items, and its tooltip names the mode
const MODES: [(&str, &str); 2] = [("R", "Recipe Mode"), ("G", "Give Mode")];

#[derive(Serialize, Deserialize)]
struct Grab
{
    m_id: i32,
    m_damage: i32,
    m_count: i32,
}

thread_local! {
    // granted by the server, which checks op on every request again
    static ENABLED: Cell<bool> = const { Cell::new(false) };
}

pub fn enabled() -> bool { ENABLED.with(Cell::get) }

pub fn button() -> &'static str { MODES[usize::from(enabled())].0 }

pub fn mode() -> &'static str { MODES[usize::from(enabled())].1 }

fn post<T: Serialize>(channel: &str, target: Target, message: &T)
{
    if let Err(error) = send(channel, target, message)
    {
        warn(&format!("{channel} not sent: {error}"));
    }
}

pub fn toggle() { post(channel::TOGGLE, Target::Server, &!enabled()); }

pub fn grab(key: Key, count: i32)
{
    post(channel::GRAB, Target::Server, &Grab { m_id: key.m_id, m_damage: key.m_damage, m_count: count });
}

pub fn trash() { post(channel::TRASH, Target::Server, &()); }

// a player who lost op since cheating was granted is told and switched back off
fn permitted(player: Player) -> bool
{
    if player.isOp()
    {
        return true;
    }
    player.chat(&translate(text::NO_PERMISSION));
    post(channel::STATE, Target::Player(player.id), &false);
    false
}

fn onToggle(wanted: bool, player: Player)
{
    let granted: bool = wanted && permitted(player);
    post(channel::STATE, Target::Player(player.id), &granted);
}

// the stack goes straight into the inventory and what does not fit drops at the player
fn onGrab(request: Grab, player: Player)
{
    if !permitted(player)
    {
        return;
    }

    let key: Key = Key { m_id: request.m_id, m_damage: request.m_damage };
    let Some(max) = catalog::get().entry(key).map(|entry: &Entry| entry.m_max)
    else
    {
        warn(&format!("grab of unlisted item {key:?} refused"));
        return;
    };

    let count: i32 = request.m_count.clamp(1, max);
    if let Err(error) = player.give(key.stack(count))
    {
        warn(&format!("grab failed: {error}"));
        player.chat(&translate(text::FAILED));
    }
}

fn onTrash(_: (), player: Player)
{
    if permitted(player)
        && let Err(error) = player.setCursor(None)
    {
        warn(&format!("trash failed: {error}"));
    }
}

fn login(event: &mut Event)
{
    if let Some(player) = event.player()
    {
        post(channel::STATE, Target::Player(player.id), &false);
    }
}

fn toServer<T: DeserializeOwned + 'static>(
    ctx: &mut RegisterCtx,
    name: &str,
    handler: impl Fn(T, Player) + 'static,
) -> Result<()>
{
    ctx.channels().register::<T>(name, Direction::ToServer, handler)
}

pub fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.channels().register::<bool>(channel::STATE, Direction::ToClient, |granted, _| ENABLED.set(granted))?;
    toServer::<bool>(ctx, channel::TOGGLE, onToggle)?;
    toServer::<Grab>(ctx, channel::GRAB, onGrab)?;
    toServer::<()>(ctx, channel::TRASH, onTrash)?;
    ctx.events().on::<PlayerLogin>(Priority::Normal, login)
}
