use crate::ids::item;
use crate::world::biomes::{self, Biome};
use crate::world::features::Feature;
use b173_api::prelude::*;

// TFAchievementPage, the forest's achievements kept per player since beta has no mod achievements
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Gate
{
    Portal,
    Arrival,
    Hunter,
    MagicMapFocus,
    Hill1,
    Hill2,
    Hill3,
    Hedge,
    MagicMap,
    KillNaga,
    Naga,
    KillLich,
    Lich,
    Labyrinth,
    KillHydra,
    Hydra,
    Trophy,
    Knights,
    UrGhast,
    Yeti,
    Glacier,
    Troll,
    Thorns,
    Castle,
    NagaArmors,
    LichScepters,
    MazeMap,
    OreMap,
    HydraChop,
    Mazebreaker,
    FierySet,
    QuestRam,
}

struct Step
{
    m_gate: Gate,
    m_label: &'static str,
    // an achievement unlocks only once its parent has, StatFileWriter.canUnlockAchievement
    m_parent: Option<Gate>,
}

const fn step(gate: Gate, label: &'static str, parent: Option<Gate>) -> Step
{
    Step { m_gate: gate, m_label: label, m_parent: parent }
}

// rows in Gate order
const STEPS: [Step; 32] = [
    step(Gate::Portal, "Into the Twilight Forest", None),
    step(Gate::Arrival, "Twilight Arrival", Some(Gate::Portal)),
    step(Gate::Hunter, "Twilight Hunter", Some(Gate::Arrival)),
    step(Gate::MagicMapFocus, "Magic Map Focus", Some(Gate::Hunter)),
    step(Gate::Hill1, "Small Hollow Hill", Some(Gate::Arrival)),
    step(Gate::Hill2, "Medium Hollow Hill", Some(Gate::Arrival)),
    step(Gate::Hill3, "Large Hollow Hill", Some(Gate::Arrival)),
    step(Gate::Hedge, "Hedge Maze", Some(Gate::Arrival)),
    step(Gate::MagicMap, "Magic Map", Some(Gate::MagicMapFocus)),
    step(Gate::KillNaga, "Kill the Naga", Some(Gate::Hunter)),
    step(Gate::Naga, "Naga Scales", Some(Gate::KillNaga)),
    step(Gate::KillLich, "Kill the Lich", Some(Gate::Naga)),
    step(Gate::Lich, "Lich Scepters", Some(Gate::KillLich)),
    step(Gate::Labyrinth, "Labyrinth Treasure", Some(Gate::Lich)),
    step(Gate::KillHydra, "Kill the Hydra", Some(Gate::Labyrinth)),
    step(Gate::Hydra, "Fiery Blood", Some(Gate::KillHydra)),
    step(Gate::Trophy, "Trophy Pedestal", Some(Gate::Hydra)),
    step(Gate::Knights, "Phantom Knights", Some(Gate::Trophy)),
    step(Gate::UrGhast, "Ur-Ghast", Some(Gate::Knights)),
    step(Gate::Yeti, "Alpha Yeti", Some(Gate::UrGhast)),
    step(Gate::Glacier, "Snow Queen", Some(Gate::Yeti)),
    step(Gate::Troll, "Lamp of Cinders", Some(Gate::Glacier)),
    step(Gate::Thorns, "Thornlands", Some(Gate::Troll)),
    step(Gate::Castle, "Final Castle", Some(Gate::Thorns)),
    step(Gate::NagaArmors, "Naga Armor", Some(Gate::KillNaga)),
    step(Gate::LichScepters, "Scepter Mastery", Some(Gate::KillLich)),
    step(Gate::MazeMap, "Maze Map", Some(Gate::Lich)),
    step(Gate::OreMap, "Ore Map", Some(Gate::MazeMap)),
    step(Gate::HydraChop, "Hydra Chop", Some(Gate::KillHydra)),
    step(Gate::Mazebreaker, "Mazebreaker", Some(Gate::Lich)),
    step(Gate::FierySet, "Fiery Set", Some(Gate::Hydra)),
    step(Gate::QuestRam, "Quest Ram", Some(Gate::Arrival)),
];

const KEY: &str = "progress";

pub fn earned(player: Player) -> u64 { player.data(KEY) }

pub fn has(player: Player, gate: Gate) -> bool { earned(player) & 1 << gate as u32 != 0 }

// EntityPlayer.triggerAchievement, told in chat as beta shows only its own achievements
pub fn grant(player: Player, gate: Gate)
{
    let row: &Step = &STEPS[gate as usize];
    if has(player, gate) || row.m_parent.is_some_and(|parent| !has(player, parent))
    {
        return;
    }
    if let Err(error) = player.setData(KEY, &(earned(player) | 1 << gate as u32))
    {
        warn(&format!("progress {gate:?}: {}", error.0));
        return;
    }
    player.chat(&format!("Achievement get! {}", row.m_label));
}

// TFEventListener.pickupItem, picking up a boss's prize in the forest moves the player along
fn pickedUp(player: Player, stack: &ItemStack)
{
    let prizes: [(&[&str], Gate); 9] = [
        (&[item::SCEPTER_TWILIGHT, item::SCEPTER_LIFE_DRAIN, item::SCEPTER_ZOMBIE], Gate::Lich),
        (&[item::NAGA_SCALE], Gate::Naga),
        (&[item::MAZEBREAKER_PICK], Gate::Mazebreaker),
        (&[item::MEEF_STROGANOFF, item::MINOTAUR_AXE], Gate::Labyrinth),
        (&[item::FIERY_BLOOD], Gate::Hydra),
        (&[item::PHANTOM_HELM, item::PHANTOM_PLATE], Gate::Knights),
        (&[item::FIERY_TEARS], Gate::UrGhast),
        (&[item::ALPHA_FUR, item::YETI_HELM, item::YETI_PLATE, item::YETI_LEGS, item::YETI_BOOTS], Gate::Yeti),
        (&[item::LAMP_OF_CINDERS], Gate::Troll),
    ];
    if let Some((_, gate)) = prizes.iter().find(|(names, _)| names.iter().any(|name| stack.item.is(name)))
    {
        grant(player, *gate);
    }
    let trophies: [(&str, Gate); 5] = [
        (item::HYDRA_TROPHY, Gate::KillHydra),
        (item::NAGA_TROPHY, Gate::KillNaga),
        (item::LICH_TROPHY, Gate::KillLich),
        (item::UR_GHAST_TROPHY, Gate::UrGhast),
        (item::SNOW_QUEEN_TROPHY, Gate::Glacier),
    ];
    if let Some((_, gate)) = trophies.iter().find(|(name, _)| stack.item.is(name))
    {
        grant(player, *gate);
    }
}

// TFBiomeBase.getRequiredAchievement of each biome
pub fn required(biome: Biome) -> Option<Gate>
{
    match biome
    {
        Biome::Swamp => Some(Gate::Lich),
        Biome::FireSwamp => Some(Gate::Labyrinth),
        Biome::DarkForest | Biome::DarkForestCenter => Some(Gate::Hydra),
        Biome::SnowyForest | Biome::Glacier => Some(Gate::UrGhast),
        Biome::Highlands | Biome::Thornlands | Biome::FinalPlateau => Some(Gate::Glacier),
        _ => None,
    }
}

mod enforce
{
    // TFBiome*.enforceProgession acts every three seconds, the highlands' acid every quarter second
    pub const CHECK: i64 = 60;
    pub const ACID: i64 = 5;
    // beta has no slowness, the snow and the glacier bite with cold instead, the glacier harder
    pub const SNOW_BITE: i32 = 1;
    pub const GLACIER_BITE: i32 = 2;
    pub const BURN_TICKS: f64 = 8.0 * 20.0;
    // one check in four may bring a hint kobold for the feature that unlocks the biome
    pub const HINT_ODDS: i32 = 4;
}

// TFBiome*.enforceProgession, the forest pushes back on players who arrive too early; beta has no slowness,
// blindness or hunger, so the cold biomes hurt instead and the swamp and the dark forest only send hint kobolds
fn punish(world: World, player: Player, info: &EntityInfo, biome: Biome, now: i64)
{
    let entity: Entity = Entity { world, id: player.id };
    if biome == Biome::Highlands
    {
        entity.damage(1, None);
        world.playSound(info.pos, "random.fizz", 1.0, 1.0);
    }
    if now % enforce::CHECK != 0
    {
        return;
    }
    match biome
    {
        Biome::SnowyForest => frostbite(world, entity, info, enforce::SNOW_BITE),
        Biome::Glacier => frostbite(world, entity, info, enforce::GLACIER_BITE),
        Biome::FireSwamp => entity.setField(EntityField::FireTicks, enforce::BURN_TICKS),
        _ =>
        {}
    }
    let hint: Option<Feature> = match biome
    {
        Biome::SnowyForest => Some(Feature::YetiCave),
        Biome::Glacier => Some(Feature::IceTower),
        Biome::DarkForest | Biome::DarkForestCenter => Some(Feature::Stronghold),
        Biome::Swamp => Some(Feature::Labyrinth),
        Biome::FireSwamp => Some(Feature::HydraLair),
        Biome::Highlands => Some(Feature::TrollCave),
        _ => None,
    };
    if let Some(feature) = hint.filter(|_| world.random(enforce::HINT_ODDS) == 0)
    {
        let pos: BlockPos =
            BlockPos::new(info.pos[0].floor() as i32, info.pos[1].floor() as i32, info.pos[2].floor() as i32);
        crate::hints::trySpawn(world, player, feature, pos);
    }
}

fn frostbite(world: World, entity: Entity, info: &EntityInfo, bite: i32)
{
    entity.damage(bite, None);
    crate::particles::spawn(world, "snowstuff", info.pos, [0.0; 3]);
}

fn checkPlayers()
{
    let Some(twilight) = crate::world::dimension()
    else
    {
        return;
    };
    let world: World = World { dimension: twilight };
    if !world.isAuthority() || !crate::config::enforced()
    {
        return;
    }
    let now: i64 = world.time();
    if now % enforce::ACID != 0
    {
        return;
    }
    for player in world.players()
    {
        let Some(info) = player.info()
        else
        {
            continue;
        };
        let biome: Biome = biomes::at(world.seed(), info.pos[0].floor() as i32, info.pos[2].floor() as i32);
        let Some(gate) = required(biome)
        else
        {
            continue;
        };
        if !has(player, gate)
        {
            punish(world, player, &info, biome, now);
        }
    }
}

// what a biome needs before the player may stay there, for the teleporter's safety check
pub fn safe(player: Player, biome: Biome) -> bool { required(biome).is_none_or(|gate| has(player, gate)) }

pub struct Pedestal;

#[block_behavior]
impl BlockBehavior for Pedestal {}

pub fn doorLocked(_world: World, _pos: BlockPos) -> bool { false }

pub fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.events().on::<ItemPickup>(Priority::Normal, |event|
    {
        let (Some(player), Some(stack), Some(world)) = (event.player(), event.stack(), event.world())
        else
        {
            return;
        };
        if world.isAuthority() && crate::world::isTwilight(world)
        {
            pickedUp(player, &stack);
        }
    })?;
    ctx.events().on::<TickPost>(Priority::Normal, |_| checkPlayers())?;
    ctx.commands().registerWithUsage("tfprogress", Permission::Op, "<player> <boss>", |command|
    {
        let reply: String = setProgress(command.arg(0), command.arg(1));
        command.reply(&reply);
        Ok(())
    })?;
    Ok(())
}

// CommandTFProgress, each boss step with the achievements it grants, a step grants every one before it too
struct BossStep
{
    m_name: &'static str,
    m_gates: &'static [Gate],
}

const BOSS_STEPS: [BossStep; 11] = [
    BossStep { m_name: "none", m_gates: &[] },
    BossStep { m_name: "naga", m_gates: &[Gate::Portal, Gate::Arrival, Gate::Hunter, Gate::KillNaga, Gate::Naga] },
    BossStep { m_name: "lich", m_gates: &[Gate::KillLich, Gate::Lich] },
    BossStep { m_name: "mooshroom", m_gates: &[Gate::Labyrinth] },
    BossStep { m_name: "hydra", m_gates: &[Gate::KillHydra, Gate::Hydra] },
    BossStep { m_name: "knights", m_gates: &[Gate::Trophy, Gate::Knights] },
    BossStep { m_name: "urghast", m_gates: &[Gate::UrGhast] },
    BossStep { m_name: "yeti", m_gates: &[Gate::Yeti] },
    BossStep { m_name: "snowqueen", m_gates: &[Gate::Glacier] },
    BossStep { m_name: "giants", m_gates: &[Gate::Troll] },
    BossStep { m_name: "final", m_gates: &[Gate::Thorns, Gate::Castle] },
];

fn playerNamed(name: &str) -> Option<Player>
{
    let dimensions: Vec<i32> = [0, -1].into_iter().chain(crate::world::dimension()).collect();
    dimensions
        .into_iter()
        .flat_map(|dimension| World { dimension }.players())
        .find(|player| player.info().and_then(|info| info.player).is_some_and(|other| other.eq_ignore_ascii_case(name)))
}

fn setProgress(name: Option<&str>, boss: Option<&str>) -> String
{
    let (Some(name), Some(boss)) = (name, boss)
    else
    {
        return "tfprogress <player> <boss>".to_owned();
    };
    let Some(player) = playerNamed(name)
    else
    {
        return format!("player {name} is not online");
    };
    // getBossIndex falls back to none for an unknown boss
    let index: usize = BOSS_STEPS.iter().position(|step| step.m_name.eq_ignore_ascii_case(boss)).unwrap_or(0);
    for step in &BOSS_STEPS[..=index]
    {
        for gate in step.m_gates
        {
            grant(player, *gate);
        }
    }
    format!("Setting player {name} progress to past boss {}.", BOSS_STEPS[index].m_name)
}
