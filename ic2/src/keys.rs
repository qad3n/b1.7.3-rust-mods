use b173_api::prelude::*;

const CHANNEL: &str = "keys";

// mod_IC2's keybindings and their default LWJGL codes, the server learns their state through the channel
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Key
{
    Suit,
    Hover,
    Laser,
    // the jump and forward keys, which the server cannot see for a remote player
    Jump,
    Forward,
}

struct Binding
{
    m_key: Key,
    m_name: &'static str,
    m_code: i32,
}

const BINDINGS: [Binding; 3] = [
    Binding { m_key: Key::Suit, m_name: "ic2.key.suit", m_code: 29 },
    Binding { m_key: Key::Hover, m_name: "ic2.key.hover", m_code: 35 },
    Binding { m_key: Key::Laser, m_name: "ic2.key.laser", m_code: 50 },
];

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
struct Press
{
    m_key: Key,
    m_down: bool,
}

static HELD: EntityState<Vec<Key>> = EntityState::new("held_keys");

pub(crate) fn entityOf(player: Player) -> Option<Entity>
{
    player.info().map(|info| Entity { world: World { dimension: info.world }, id: player.id })
}

// Keyboard.isKeyDown for one of IC2's keys, as the player last reported it
pub(crate) fn held(player: Player, key: Key) -> bool
{
    entityOf(player).is_some_and(|entity| HELD.get(entity).contains(&key))
}

fn record(press: Press, player: Player)
{
    let Some(entity) = entityOf(player)
    else
    {
        return;
    };

    let changed: Result<()> = HELD.update(entity, |keys|
    {
        keys.retain(|key| *key != press.m_key);
        if press.m_down
        {
            keys.push(press.m_key);
        }
    });
    if let Err(error) = changed
    {
        warn(&format!("could not record an IC2 key: {error}"));
    }
}

fn report(key: Key, down: bool)
{
    let press: Press = Press { m_key: key, m_down: down };
    if let Some(player) = localPlayer()
    {
        record(press, player);
    }

    if let Err(error) = send(&format!("ic2:{CHANNEL}"), Target::Server, &press)
    {
        warn(&format!("could not report an IC2 key: {error}"));
    }
}

// the local player's jump and forward keys, sent when they change
fn reportMovement()
{
    let Some((player, info)) = localPlayer().and_then(|player| player.info().map(|info| (player, info)))
    else
    {
        return;
    };
    let entity: Entity = Entity { world: World { dimension: info.world }, id: player.id };

    for (key, down) in
        [(Key::Jump, entity.flag(EntityField::Jumping)), (Key::Forward, entity.field(EntityField::MoveForward) > 0.0)]
    {
        if held(player, key) != down
        {
            report(key, down);
        }
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.channels().register(CHANNEL, Direction::ToServer, record)?;

    for binding in &BINDINGS
    {
        let key: Key = binding.m_key;
        ctx.keybinds().register(binding.m_name, binding.m_code, move |down| report(key, down))?;
    }

    ctx.events().on::<TickPost>(Priority::High, |_|
    {
        if isClient()
        {
            reportMovement();
        }
    })
}
