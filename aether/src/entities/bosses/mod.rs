mod names;
pub(crate) mod slider;
pub(crate) mod sun_spirit;
pub(crate) mod valkyrie_queen;

use crate::entities::fraction;
use crate::names::{Pick, handle};
use b173_api::prelude::*;
use b173_api::screen::Draw;

pub(crate) const SLIDER: &str = "slider";
pub(crate) const VALKYRIE_QUEEN: &str = "valkyrie_queen";
pub(crate) const SUN_SPIRIT: &str = "sun_spirit";

// where a boss was placed by its dungeon: the corner its arena counts from, the way it faces and the door it shut
#[derive(Serialize, Deserialize, Default, Clone, Copy)]
pub(crate) struct Home
{
    pub m_dungeon: [i32; 3],
    pub m_direction: i32,
    pub m_door: Option<[[i32; 3]; 2]>,
}

// what the boss bar shows: AetherNameGen name, health and the player it is fighting
#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
pub(crate) struct BossBar
{
    pub m_name: String,
    pub m_health: i32,
    pub m_maxHealth: i32,
    pub m_focus: Option<i32>,
}

mod bar
{
    // AetherOverlay.renderBossHP
    pub const SHEET: &str = "aether:gui/boss_bar";
    pub const WIDTH: i32 = 256;
    pub const HEIGHT: i32 = 16;
    pub const TOP: i32 = 12;
    pub const TITLE_Y: i32 = 2;
    pub const WHITE: u32 = 0xFFFFFF;
    pub const SEARCH: f64 = 64.0;
    pub const MIDDLES: i32 = 2;
}

// the dungeons place their boss once the chunk holding it populates, setDungeon in each Java boss
pub(crate) fn spawn(world: World, kind: &str, pos: [f64; 3], dungeon: [i32; 3], direction: i32)
{
    match world.spawnEntity(&format!("aether:{kind}"), pos)
    {
        Ok(boss) =>
        {
            let _ = boss.setData(&Home { m_dungeon: dungeon, m_direction: direction, m_door: None });
        }

        Err(error) => log(&format!("no {kind} for the dungeon at {dungeon:?}: {}", error.0)),
    }
}

fn pick<'a>(world: World, words: &[&'a str]) -> &'a str { words[world.random(words.len() as i32) as usize] }

// AetherNameGen.gen
pub(crate) fn bossName(world: World, title: &str) -> String
{
    let mut name: String = pick(world, &names::FIRST).to_owned();

    for _ in 0..bar::MIDDLES + world.random(2)
    {
        name.push_str(pick(world, &names::MIDDLE));
    }

    name.push_str(pick(world, &names::LAST));
    format!("{name}, {}", translate(&format!("aether.title.{title}")))
}

// AetherNameGen.valkGen
pub(crate) fn valkyrieName(world: World) -> String
{
    [&names::VALKYRIE_PREFIX[..], &names::VALKYRIE_MIDDLE[..], &names::VALKYRIE_SUFFIX[..]]
        .iter()
        .map(|words| pick(world, words))
        .collect()
}

// a boss's chat line to one player, at most one every three seconds
pub(crate) fn say(player: Player, key: &str) { player.chat(&translate(&format!("aether.{key}"))); }

// BlockDungeonBase.getUnlockedBlock spread through every touching locked block of the dungeon's own kind
pub(crate) fn unlock(world: World, from: [i32; 3], locked: &[&str])
{
    let mut pending: Vec<[i32; 3]> = vec![from];
    while let Some([x, y, z]) = pending.pop()
    {
        let pos: BlockPos = BlockPos::new(x, y, z);
        let Some(name) = locked.iter().find(|name| world.is(pos, name))
        else
        {
            continue;
        };

        let _ = world.place(pos, crate::world::dungeons::unlocked(name));

        pending.extend([[x + 1, y, z], [x - 1, y, z], [x, y + 1, z], [x, y - 1, z], [x, y, z + 1], [x, y, z - 1]]);
    }
}

// a locked dungeon stone and the plain stone a beaten boss leaves in its place
pub(crate) type Swap = (&'static str, &'static str);

// every locked stone in the box from low up to high turns plain
pub(crate) fn freeBox(world: World, low: [i32; 3], high: [i32; 3], swaps: &[Swap])
{
    for x in low[0]..high[0]
    {
        for y in low[1]..high[1]
        {
            for z in low[2]..high[2]
            {
                let pos: BlockPos = BlockPos::new(x, y, z);
                if let Some((_, plain)) = swaps.iter().find(|(locked, _)| world.is(pos, locked))
                {
                    let _ = world.place(pos, plain);
                }
            }
        }
    }
}

// beta trapdoors have no named states so their hinge side is the metadata the loader reports as meta
pub(crate) const TRAPDOOR_LOW_X: Pick = Pick::with("minecraft:trapdoor", &[("meta", "3")]);
pub(crate) const TRAPDOOR_HIGH_X: Pick = Pick::with("minecraft:trapdoor", &[("meta", "2")]);

// the trapdoors a beaten boss lays over its treasure, one above the floor at each offset
pub(crate) fn trapdoors(world: World, floor: [i32; 3], rows: &[([i32; 2], Pick)])
{
    let [x, y, z] = floor;
    for ([dx, dz], pick) in rows
    {
        let _ = handle(*pick).place(world, BlockPos::new(x + dx, y + 1, z + dz), true);
    }
}

// a fresh 0.9 to 1.1 pitch the bosses play their sounds at, 1 / (nextFloat * 0.2 + 0.9)
pub(crate) fn pitch(world: World) -> f32 { 1.0 / (fraction(world) as f32 * 0.2 + 0.9) }

fn barOf(kind: &str, boss: Entity) -> Option<BossBar>
{
    match kind
    {
        SLIDER => Some(slider::bar(boss)),
        VALKYRIE_QUEEN => Some(valkyrie_queen::bar(boss)),
        SUN_SPIRIT => Some(sun_spirit::bar(boss)),
        _ => None,
    }
}

// AetherOverlay.renderBossHP: the name over a bar of the boss the local player is fighting
fn drawBar(draw: &mut Draw, width: i32)
{
    let Some(local) = localPlayer().and_then(Player::info)
    else
    {
        return;
    };

    let world: World = World { dimension: local.world };
    let found: Option<BossBar> = world.entitiesNear(local.pos, bar::SEARCH).into_iter().find_map(|other|
    {
        let kind: Identifier = other.kind.filter(|kind| kind.namespace() == "aether")?;
        let shown: BossBar = barOf(kind.path(), Entity { world, id: other.id })?;
        (shown.m_focus == Some(local.id) && shown.m_health > 0).then_some(shown)
    });
    let Some(shown) = found
    else
    {
        return;
    };

    let left: i32 = width / 2 - bar::WIDTH / 2;

    draw.centered(width / 2, bar::TITLE_Y, &shown.m_name, bar::WHITE);
    draw.texture(bar::SHEET, left, bar::TOP, 0, bar::HEIGHT, bar::WIDTH, bar::HEIGHT);

    let filled: i32 = shown.m_health * bar::WIDTH / shown.m_maxHealth.max(1);
    draw.texture(bar::SHEET, left, bar::TOP, 0, 0, filled, bar::HEIGHT);
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    slider::register(ctx)?;
    valkyrie_queen::register(ctx)?;
    sun_spirit::register(ctx)?;
    ctx.screens().hud(|draw, _partial, width, _height| drawBar(draw, width))
}
