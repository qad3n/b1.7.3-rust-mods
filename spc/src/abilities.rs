use b173_api::prelude::*;
use std::cell::Cell;

const FLY_CHANNEL: &str = "fly";
const DATA_KEY: &str = "abilities";

mod flight
{
    pub const VERTICAL: f64 = 0.4;
    pub const PUSH: f64 = 0.02;
}

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
pub struct Abilities
{
    pub m_fly: bool,
    pub m_god: bool,
}

thread_local! {
    static LOCAL_FLY: Cell<bool> = const { Cell::new(false) };
}

pub fn of(player: Player) -> Abilities { player.data(DATA_KEY) }

pub fn set(player: Player, abilities: Abilities) -> Result<()>
{
    player.setData(DATA_KEY, &abilities)?;
    send(FLY_CHANNEL, Target::Player(player.id), &abilities.m_fly)
}

fn entityOf(player: Player) -> Option<Entity>
{
    player.info().map(|info| Entity { world: World { dimension: info.world }, id: player.id })
}

fn localFlight()
{
    if !LOCAL_FLY.with(Cell::get)
    {
        return;
    }
    let Some(player) = localPlayer()
    else
    {
        return;
    };
    let (Some(entity), Some(info)) = (entityOf(player), player.info())
    else
    {
        return;
    };
    let vertical: f64 = match (entity.flag(EntityField::Jumping), entity.flag(EntityField::Sneaking))
    {
        (true, false) => flight::VERTICAL,
        (false, true) => -flight::VERTICAL,
        _ => 0.0,
    };
    let forward: f64 = entity.field(EntityField::MoveForward);
    let strafe: f64 = entity.field(EntityField::MoveStrafe);
    let yaw: f64 = (info.yaw as f64).to_radians();
    let push: [f64; 2] = [strafe * yaw.cos() - forward * yaw.sin(), forward * yaw.cos() + strafe * yaw.sin()];
    let _ =
        entity.setMotion([info.motion[0] + push[0] * flight::PUSH, vertical, info.motion[2] + push[1] * flight::PUSH]);
    entity.setField(EntityField::FallDistance, 0.0);
}

fn serverFlight()
{
    for player in allPlayers()
    {
        let Some(entity) = entityOf(player).filter(|entity| entity.world.isAuthority())
        else
        {
            continue;
        };
        if of(player).m_fly
        {
            entity.setField(EntityField::FallDistance, 0.0);
        }
    }
}

fn shielded(event: &mut Event)
{
    let EventData::Attack { entity, .. } = event.data()
    else
    {
        return;
    };
    if entity.player.is_some() && of(Player { id: entity.id }).m_god
    {
        event.cancel();
    }
}

fn login(event: &mut Event)
{
    if let Some(player) = event.player()
    {
        let _ = send(FLY_CHANNEL, Target::Player(player.id), &of(player).m_fly);
    }
}

pub fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.channels().register::<bool>(FLY_CHANNEL, Direction::ToClient, |fly, _| LOCAL_FLY.set(fly))?;
    ctx.events().on::<TickPost>(Priority::Normal, |_| {
        if isClient()
        {
            localFlight();
        }
        serverFlight();
    })?;
    ctx.events().on::<LivingAttack>(Priority::High, shielded)?;
    ctx.events().on::<PlayerLogin>(Priority::Normal, login)?;
    Ok(())
}
