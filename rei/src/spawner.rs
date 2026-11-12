use crate::catalog::Key;
use b173_api::prelude::*;

const SPAWNER: &str = "minecraft:mob_spawner";
const SUFFIX: &str = "Spawner";

fn spawner() -> Result<i32> { ItemRef::from(SPAWNER).id() }

// a registry path such as pig_zombie read as Pig Zombie
pub(crate) fn humanize(path: &str) -> String
{
    let words: Vec<String> = path
        .split('_')
        .filter(|word| !word.is_empty())
        .map(|word|
        {
            let mut chars: std::str::Chars = word.chars();
            chars.next().map_or_else(String::new, |first| first.to_uppercase().chain(chars).collect())
        })
        .collect();
    words.join(" ")
}

// one spawner per registered mob with the entity id as its damage, the way old item viewers carried the mob
pub(crate) fn stacks() -> Vec<(ItemStack, String, Identifier)>
{
    if spawner().is_err()
    {
        warn("mob spawner is not registered, no spawners listed");
        return Vec::new();
    };

    mobList()
        .into_iter()
        .filter_map(|mob|
        {
            let numeric: i32 = resolve(RegistryKind::Entity, &mob).ok()?;
            let name: String = format!("{} {SUFFIX}", humanize(mob.path()));
            Some((ItemStack::of(SPAWNER, 1).withDamage(numeric), name, mob))
        })
        .collect()
}

// the listed spawner of a mob
pub(crate) fn key(mob: &Identifier) -> Option<Key>
{
    let numeric: i32 = resolve(RegistryKind::Entity, mob).ok()?;
    spawner().ok().map(|spawner| Key { m_id: spawner, m_damage: numeric })
}

// a spawner stack that carries a mob
pub(crate) fn held(key: &Key) -> bool { key.m_damage > 0 && spawner().ok() == Some(key.m_id) }

fn mob(damage: i32) -> Option<Identifier>
{
    let mob: Identifier = nameOf(RegistryKind::Entity, damage)?;
    mobList().contains(&mob).then_some(mob)
}

// the authority sets the mob before vanilla would place a plain pig spawner, a mirroring client keeps its prediction
fn place(event: &mut Event)
{
    let EventData::Block { block: placed, .. } = *event.data()
    else
    {
        return;
    };

    let (Some(world), Some(pos), Some(player)) = (event.world(), event.pos(), event.player())
    else
    {
        return;
    };

    let held: Option<ItemStack> = player.held().filter(|held| held.damage > 0);
    let Some((held, mob)) = held.and_then(|held| mob(held.damage).map(|mob| (held, mob)))
    else
    {
        return;
    };

    if !blockIs(placed, SPAWNER) || !world.isAuthority()
    {
        return;
    }

    if let Err(error) = world.setSpawner(pos, &mob.to_string())
    {
        warn(&format!("{mob} spawner not placed: {error}"));
        return;
    }

    event.cancel();

    let left: Option<ItemStack> = (held.count > 1).then(|| ItemStack { count: held.count - 1, ..held });
    if let Err(error) = player.setHeld(left)
    {
        warn(&format!("spawner not taken from the hand: {error}"));
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()> { ctx.events().on::<BlockPlace>(Priority::Normal, place) }
