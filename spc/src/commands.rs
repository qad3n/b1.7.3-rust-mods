use crate::abilities::{self, Abilities};
use b173_api::math::blockOf;
use b173_api::prelude::*;

mod limits
{
    pub const STACK: i32 = 64;
    pub const GIVE: i32 = 64 * 36;
    pub const SUMMON: i32 = 32;
    pub const MAIN_INVENTORY: i32 = 36;
    // chunks searched around the player, the loader answers up to 128
    pub const LOCATE: i32 = 64;
    pub const EFFECT_SECONDS: i32 = 30;
    pub const TICKS_PER_SECOND: i32 = 20;
}

mod clock
{
    pub const DAY_LENGTH: i64 = 24000;
    pub const MORNING: i64 = 1000;
    pub const NOON: i64 = 6000;
    pub const NIGHT: i64 = 13000;
    pub const MIDNIGHT: i64 = 18000;
}

struct Spec
{
    m_name: &'static str,
    m_usage: &'static str,
    m_run: fn(&[String], Player) -> Result<String>,
    // a dedicated server already has this command, only single player gets ours
    m_vanilla: bool,
}

const COMMANDS: &[Spec] = &[
    Spec { m_name: "help", m_usage: "", m_run: help, m_vanilla: false },
    Spec { m_name: "give", m_usage: "<item> [count] [damage]", m_run: give, m_vanilla: true },
    Spec { m_name: "fly", m_usage: "", m_run: fly, m_vanilla: false },
    Spec { m_name: "god", m_usage: "", m_run: god, m_vanilla: false },
    Spec { m_name: "heal", m_usage: "", m_run: heal, m_vanilla: false },
    Spec { m_name: "repair", m_usage: "", m_run: repair, m_vanilla: false },
    Spec { m_name: "clear", m_usage: "", m_run: clear, m_vanilla: false },
    Spec { m_name: "time", m_usage: "<day|noon|night|midnight|ticks>", m_run: time, m_vanilla: true },
    Spec { m_name: "tp", m_usage: "<x> <y> <z>", m_run: teleport, m_vanilla: true },
    Spec { m_name: "summon", m_usage: "<entity> [count]", m_run: summon, m_vanilla: false },
    Spec { m_name: "locate", m_usage: "<structure> [radius]", m_run: locate, m_vanilla: false },
    Spec { m_name: "effect", m_usage: "<effect> [level] [seconds] | remove <effect>", m_run: effect, m_vanilla: false },
];

fn vanillaId(name: &str) -> Result<Identifier>
{
    Identifier::parseIn(&name.to_lowercase(), "minecraft").map_err(|error| Error(error.to_string()))
}

fn number<T: std::str::FromStr>(args: &[String], index: usize, fallback: T) -> Result<T>
{
    match arg(args, index)
    {
        Some(text) => text.parse().map_err(|_| Error(format!("not a number: {text}"))),
        None => Ok(fallback),
    }
}

fn arg(args: &[String], index: usize) -> Option<&str> { args.get(index).map(String::as_str) }

fn entityOf(player: Player) -> Result<(Entity, EntityInfo)>
{
    let info: EntityInfo = player.info().ok_or("player not loaded")?;
    Ok((Entity { world: World { dimension: info.world }, id: player.id }, info))
}

// a dedicated server keeps its own give, time and tp
fn available() -> impl Iterator<Item = &'static Spec>
{
    let server: bool = !isClient();
    COMMANDS.iter().filter(move |spec| !(server && spec.m_vanilla))
}

fn help(_: &[String], _: Player) -> Result<String>
{
    Ok(available().map(|spec| format!("/{} {}", spec.m_name, spec.m_usage)).collect::<Vec<String>>().join("\n"))
}

fn give(args: &[String], player: Player) -> Result<String>
{
    let name: &str = arg(args, 0).ok_or("usage: /give <item> [count] [damage]")?;
    let item: ItemRef = match name.parse::<i32>()
    {
        Ok(numeric) => ItemRef::Numeric(numeric),
        Err(_) => ItemRef::Named(vanillaId(name)?),
    };
    item.id().map_err(|_| Error(format!("unknown item {name}")))?;

    let count: i32 = number(args, 1, 1)?.clamp(1, limits::GIVE);
    let damage: i32 = number(args, 2, 0)?;
    let mut left: i32 = count;
    while left > 0
    {
        let size: i32 = left.min(limits::STACK);
        player.give(ItemStack::of(&item, size).withDamage(damage))?;
        left -= size;
    }

    let label: String = item.name().map_or_else(|| name.to_owned(), |id| id.to_string());
    Ok(format!("gave {count} {label}"))
}

fn toggle(player: Player, label: &str, flip: fn(&mut Abilities) -> bool) -> Result<String>
{
    let mut state: Abilities = abilities::of(player);
    let enabled: bool = flip(&mut state);

    abilities::set(player, state)?;
    Ok(format!("{label} {}", if enabled { "on" } else { "off" }))
}

fn fly(_: &[String], player: Player) -> Result<String>
{
    toggle(player, "flight", |state|
    {
        state.m_fly = !state.m_fly;
        state.m_fly
    })
}

fn god(_: &[String], player: Player) -> Result<String>
{
    toggle(player, "god mode", |state|
    {
        state.m_god = !state.m_god;
        state.m_god
    })
}

fn heal(_: &[String], player: Player) -> Result<String>
{
    let (entity, _): (Entity, EntityInfo) = entityOf(player)?;
    entity.setField(EntityField::Health, entity.field(EntityField::MaxHealth));
    entity.setField(EntityField::FireTicks, 0.0);
    Ok("healed".to_owned())
}

fn repair(_: &[String], player: Player) -> Result<String>
{
    let held: ItemStack = player.held().ok_or("hold an item to repair")?;
    player.setHeld(Some(held.withDamage(0)))?;
    Ok("repaired".to_owned())
}

fn clear(_: &[String], player: Player) -> Result<String>
{
    for slot in 0..limits::MAIN_INVENTORY
    {
        player.setSlot(slot, None)?;
    }

    Ok("inventory cleared".to_owned())
}

fn time(args: &[String], player: Player) -> Result<String>
{
    let target: i64 = match arg(args, 0).ok_or("usage: /time <day|noon|night|midnight|ticks>")?
    {
        "day" => clock::MORNING,
        "noon" => clock::NOON,
        "night" => clock::NIGHT,
        "midnight" => clock::MIDNIGHT,
        _ => number(args, 0, 0)?,
    };

    let (entity, _): (Entity, EntityInfo) = entityOf(player)?;
    let now: i64 = entity.world.time();
    let dayStart: i64 = now - now.rem_euclid(clock::DAY_LENGTH);
    let next: i64 = if dayStart + target > now
    {
        dayStart + target
    }
    else
    {
        dayStart + clock::DAY_LENGTH + target
    };
    setWorldTime(next);
    Ok(format!("time set to {}", next.rem_euclid(clock::DAY_LENGTH)))
}

fn coordinate(args: &[String], index: usize, current: f64) -> Result<f64>
{
    let text: &str = arg(args, index).ok_or("usage: /tp <x> <y> <z>, ~ for relative")?;
    let (base, rest): (f64, &str) = match text.strip_prefix('~')
    {
        Some(rest) => (current, rest),
        None => (0.0, text),
    };
    let offset: f64 = match rest
    {
        "" => 0.0,
        _ => rest.parse().map_err(|_| Error(format!("not a coordinate: {text}")))?,
    };
    Ok(base + offset)
}

fn teleport(args: &[String], player: Player) -> Result<String>
{
    let (_, info): (Entity, EntityInfo) = entityOf(player)?;
    let target: [f64; 3] =
        [coordinate(args, 0, info.pos[0])?, coordinate(args, 1, info.pos[1])?, coordinate(args, 2, info.pos[2])?];
    player.teleport(info.world, target)?;
    Ok(format!("teleported to {:.1} {:.1} {:.1}", target[0], target[1], target[2]))
}

fn summon(args: &[String], player: Player) -> Result<String>
{
    let name: &str = arg(args, 0).ok_or("usage: /summon <entity> [count]")?;
    let kind: String = vanillaId(name)?.to_string();
    let count: i32 = number(args, 1, 1)?.clamp(1, limits::SUMMON);
    let (entity, info): (Entity, EntityInfo) = entityOf(player)?;

    for _ in 0..count
    {
        entity.world.spawnEntity(&kind, info.pos).map_err(|_| Error(format!("unknown entity {name}")))?;
    }

    Ok(format!("summoned {count} {kind}"))
}

fn locate(args: &[String], player: Player) -> Result<String>
{
    let name: &str = arg(args, 0).ok_or("usage: /locate <structure> [radius], such as dungeon or mod:name")?;
    let kind: Identifier = vanillaId(name)?;
    let radius: i32 = number(args, 1, limits::LOCATE)?;
    let (entity, info): (Entity, EntityInfo) = entityOf(player)?;
    let from: BlockPos = blockOf(info.pos);

    let Some(found) = entity.world.locate(&kind, from, radius)?
    else
    {
        return Ok(format!("no {kind} within {radius} chunks"));
    };

    let state: &str = if found.generated
    {
        "built"
    }
    else
    {
        "not built yet"
    };
    Ok(format!("{kind} at {} {}, {state}", found.x, found.z))
}

// mod status effects, beta has none of its own; level 1 is the first level
fn effect(args: &[String], player: Player) -> Result<String>
{
    let usage: &str = "usage: /effect <mod:effect> [level] [seconds] or /effect remove <mod:effect>";
    let (entity, _): (Entity, EntityInfo) = entityOf(player)?;

    if arg(args, 0) == Some("remove")
    {
        let name: &str = arg(args, 1).ok_or(usage)?;
        entity.removeEffect(name)?;
        return Ok(format!("removed {name}"));
    }

    let name: &str = arg(args, 0).ok_or(usage)?;
    let level: i32 = number(args, 1, 1)?.max(1);
    let seconds: i32 = number(args, 2, limits::EFFECT_SECONDS)?.max(1);
    entity.addEffect(name, level - 1, seconds * limits::TICKS_PER_SECOND)?;
    Ok(format!("{name} {level} for {seconds} seconds"))
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for spec in available()
    {
        let run: fn(&[String], Player) -> Result<String> = spec.m_run;

        ctx.commands().registerWithUsage(spec.m_name, Permission::Op, spec.m_usage, move |command|
        {
            let player: Player = command.player().ok_or("players only")?;
            let reply: String = run(&command.args, player)?;
            command.reply(&reply);
            Ok(())
        })?;
    }

    Ok(())
}
