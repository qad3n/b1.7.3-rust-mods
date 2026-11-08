mod names;
pub mod slider;
pub mod sun_spirit;
pub mod valkyrie_queen;

use crate::entities::fraction;
use crate::ids::blockId;
use b173_api::prelude::*;
use b173_api::screen::Draw;

pub const SLIDER: &str = "slider";
pub const VALKYRIE_QUEEN: &str = "valkyrie_queen";
pub const SUN_SPIRIT: &str = "sun_spirit";

// where a boss was placed by its dungeon: the corner its arena counts from, the way it faces and the door it shut
#[derive(Serialize, Deserialize, Default, Clone, Copy)]
pub struct Home
{
    pub m_dungeon: [i32; 3],
    pub m_direction: i32,
    pub m_door: Option<[[i32; 3]; 2]>,
}

// what the boss bar shows: AetherNameGen name, health and the player it is fighting
#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct BossBar
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
pub fn spawn(world: World, kind: &str, pos: [f64; 3], dungeon: [i32; 3], direction: i32)
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
pub fn bossName(world: World, title: &str) -> String
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
pub fn valkyrieName(world: World) -> String
{
    [&names::VALKYRIE_PREFIX[..], &names::VALKYRIE_MIDDLE[..], &names::VALKYRIE_SUFFIX[..]]
        .iter()
        .map(|words| pick(world, words))
        .collect()
}

// a boss's chat line to one player, at most one every three seconds
pub fn say(player: Player, key: &str) { player.chat(&translate(&format!("aether.{key}"))); }

// BlockDungeonBase.getUnlockedBlock spread through every touching locked block of the dungeon's own kind
pub fn unlock(world: World, from: [i32; 3], locked: &[&str])
{
    let swaps: Vec<(i32, i32)> =
        locked.iter().map(|name| (blockId(name), blockId(crate::world::dungeons::unlocked(name)))).collect();
    let mut pending: Vec<[i32; 3]> = vec![from];
    while let Some([x, y, z]) = pending.pop()
    {
        let pos: BlockPos = BlockPos::new(x, y, z);
        let Some((_, plain)) = swaps.iter().find(|(lockedId, _)| *lockedId == world.block(pos))
        else
        {
            continue;
        };
        let _ = world.setBlock(pos, *plain, 0);
        pending.extend([[x + 1, y, z], [x - 1, y, z], [x, y + 1, z], [x, y - 1, z], [x, y, z + 1], [x, y, z - 1]]);
    }
}

// a fresh 0.9 to 1.1 pitch the bosses play their sounds at, 1 / (nextFloat * 0.2 + 0.9)
pub fn pitch(world: World) -> f32 { 1.0 / (fraction(world) as f32 * 0.2 + 0.9) }

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
    let found: Option<BossBar> = world.entitiesNear(local.pos, bar::SEARCH).into_iter().find_map(|other| {
        let kind = other.kind.filter(|kind| kind.namespace() == "aether")?;
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

pub fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    slider::register(ctx)?;
    valkyrie_queen::register(ctx)?;
    sun_spirit::register(ctx)?;
    ctx.screens().hud(|draw, _partial, width, _height| drawBar(draw, width))
}
