use crate::names::item;
use crate::world::biomes::{self, Biome};
use crate::world::features::Feature;
use b173_api::math::{blockOf, floor};
use b173_api::prelude::*;

// TFAchievementPage, the forest's achievements on the loader's mod achievements
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Gate
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
    m_name: &'static str,
    m_title: &'static str,
    m_description: &'static str,
    // TFAchievementPage's icon, beta has no stone bricks so the castle shows castle brick
    m_icon: &'static str,
    m_parent: Option<Gate>,
}

// rows in Gate order, every parent before its children
const STEPS: [Step; 32] = [
    Step
    {
        m_gate: Gate::Portal,
        m_name: "portal",
        m_title: "Twilight Portal",
        m_description: "Build a portal to the Twilight Forest.",
        m_icon: "twilightforest:portal",
        m_parent: None,
    },
    Step
    {
        m_gate: Gate::Arrival,
        m_name: "arrival",
        m_title: "Twilight Arrival",
        m_description: "Arrive in the Twilight Forest.",
        m_icon: "twilightforest:canopy_log",
        m_parent: Some(Gate::Portal),
    },
    Step
    {
        m_gate: Gate::Hunter,
        m_name: "hunter",
        m_title: "Twilight Hunter",
        m_description: "Hunt some of the local wildlife.",
        m_icon: "twilightforest:raven_feather",
        m_parent: Some(Gate::Arrival),
    },
    Step
    {
        m_gate: Gate::MagicMapFocus,
        m_name: "magic_map_focus",
        m_title: "With Fire It Writes",
        m_description: "Craft the magic map focus with a raven's feather, glowstone dust, and torchberries.",
        m_icon: "twilightforest:magic_map_focus",
        m_parent: Some(Gate::Hunter),
    },
    Step
    {
        m_gate: Gate::Hill1,
        m_name: "hill1",
        m_title: "The Boots Are Mine!",
        m_description: "Defeat a redcap goblin in a small hollow hill.",
        m_icon: "minecraft:iron_ore",
        m_parent: Some(Gate::Arrival),
    },
    Step
    {
        m_gate: Gate::Hill2,
        m_name: "hill2",
        m_title: "What Was That Noise?",
        m_description: "Defeat a redcap sapper in a medium hollow hill.",
        m_icon: "minecraft:gold_ore",
        m_parent: Some(Gate::Arrival),
    },
    Step
    {
        m_gate: Gate::Hill3,
        m_name: "hill3",
        m_title: "I See Right Through You",
        m_description: "Defeat a twilight wraith in a large hollow hill.",
        m_icon: "minecraft:diamond_ore",
        m_parent: Some(Gate::Arrival),
    },
    Step
    {
        m_gate: Gate::Hedge,
        m_name: "hedge",
        m_title: "Bug Stomper",
        m_description: "Defeat a spider in a hedge maze.",
        m_icon: "twilightforest:hedge",
        m_parent: Some(Gate::Arrival),
    },
    Step
    {
        m_gate: Gate::MagicMap,
        m_name: "magic_map",
        m_title: "I Can See For Miles",
        m_description: "Craft the magic map.",
        m_icon: "twilightforest:magic_map",
        m_parent: Some(Gate::MagicMapFocus),
    },
    Step
    {
        m_gate: Gate::KillNaga,
        m_name: "kill_naga",
        m_title: "Naga Hunter",
        m_description: "Slay the naga in its forest courtyard.",
        m_icon: "twilightforest:naga_trophy",
        m_parent: Some(Gate::Hunter),
    },
    Step
    {
        m_gate: Gate::Naga,
        m_name: "progress_naga",
        m_title: "Shimmering Scale",
        m_description: "Obtain a naga scale, allowing you to overcome the barrier magic surrounding the Lich's tower.",
        m_icon: "twilightforest:naga_scale",
        m_parent: Some(Gate::KillNaga),
    },
    Step
    {
        m_gate: Gate::KillLich,
        m_name: "kill_lich",
        m_title: "Lich Hunter",
        m_description: "Slay the lich at the top of his tower.",
        m_icon: "twilightforest:lich_trophy",
        m_parent: Some(Gate::Naga),
    },
    Step
    {
        m_gate: Gate::Lich,
        m_name: "progress_lich",
        m_title: "Touched by Undeath",
        m_description: "Use the magic of the the Lich's scepter to clear away poisonous mosquitoes in the swamp.",
        m_icon: "twilightforest:lifedrain_scepter",
        m_parent: Some(Gate::KillLich),
    },
    Step
    {
        m_gate: Gate::Labyrinth,
        m_name: "progress_labyrinth",
        m_title: "Mighty Stroganoff",
        m_description: "The meat-and-mushroom stew in the labyrinth makes you feel ready to enter the Fire Swamp.",
        m_icon: "twilightforest:meef_stroganoff",
        m_parent: Some(Gate::Lich),
    },
    Step
    {
        m_gate: Gate::KillHydra,
        m_name: "kill_hydra",
        m_title: "Hydra Slayer",
        m_description: "Defeat the mighty hydra.",
        m_icon: "twilightforest:hydra_trophy",
        m_parent: Some(Gate::Labyrinth),
    },
    Step
    {
        m_gate: Gate::Hydra,
        m_name: "progress_hydra",
        m_title: "Of One Blood",
        m_description: "A touch of hydra blood in your eyes makes you resist the blindness curse in the Dark Forest.",
        m_icon: "twilightforest:fiery_blood",
        m_parent: Some(Gate::KillHydra),
    },
    Step
    {
        m_gate: Gate::Trophy,
        m_name: "progress_trophy_pedestal",
        m_title: "Trophied Champion",
        m_description: "Claim your title by placing a trophy on the pedestal in the dark forest ruins.",
        m_icon: "twilightforest:trophy_pedestal",
        m_parent: Some(Gate::Hydra),
    },
    Step
    {
        m_gate: Gate::Knights,
        m_name: "progress_knights",
        m_title: "Carminite Acclimation",
        m_description: "Settle the restless phantoms in the Knight's Tomb and the carminite tower's devices will obey you.",
        m_icon: "twilightforest:phantom_helmet",
        m_parent: Some(Gate::Trophy),
    },
    Step
    {
        m_gate: Gate::UrGhast,
        m_name: "progress_urghast",
        m_title: "Tears of Fire",
        m_description: "Touch the fiery red tears of the Ur-ghast, allowing you to resist the snow forest's chill.",
        m_icon: "twilightforest:ur_ghast_trophy",
        m_parent: Some(Gate::Knights),
    },
    Step
    {
        m_gate: Gate::Yeti,
        m_name: "progress_yeti",
        m_title: "Alpha Fur",
        m_description: "Line your garments with the soft fur from the alpha yeti, keeping you safe from the glacier's cold.",
        m_icon: "twilightforest:alpha_fur",
        m_parent: Some(Gate::UrGhast),
    },
    Step
    {
        m_gate: Gate::Glacier,
        m_name: "progress_glacier",
        m_title: "Clear Skies",
        m_description: "Defeat the Snow Queen atop the Aurora Palace, clearing away the acid rain from the highlands.",
        m_icon: "twilightforest:snow_queen_trophy",
        m_parent: Some(Gate::Yeti),
    },
    Step
    {
        m_gate: Gate::Troll,
        m_name: "progress_troll",
        m_title: "I Wish For More Burning",
        m_description: "Find the Lamp of Cinders in the troll caves, and you can burn away the thorn barriers.",
        m_icon: "twilightforest:lamp_of_cinders",
        m_parent: Some(Gate::Glacier),
    },
    Step
    {
        m_gate: Gate::Thorns,
        m_name: "progress_thorns",
        m_title: "Past the Thorns",
        m_description: "Make it past the thornlands, and unlock the door of the castle.",
        m_icon: "twilightforest:brown_thorns",
        m_parent: Some(Gate::Troll),
    },
    Step
    {
        m_gate: Gate::Castle,
        m_name: "progress_castle",
        m_title: "So Castle Very Wow",
        m_description: "What could even be in that castle?!?",
        m_icon: "twilightforest:castle_brick",
        m_parent: Some(Gate::Thorns),
    },
    Step
    {
        m_gate: Gate::NagaArmors,
        m_name: "naga_armors",
        m_title: "Naga Armorer",
        m_description: "Craft both naga scale chest and leg armor.",
        m_icon: "twilightforest:naga_chestplate",
        m_parent: Some(Gate::KillNaga),
    },
    Step
    {
        m_gate: Gate::LichScepters,
        m_name: "lich_scepters",
        m_title: "Scepter Mastery",
        m_description: "Acquire all three scepters of power.",
        m_icon: "twilightforest:zombie_scepter",
        m_parent: Some(Gate::KillLich),
    },
    Step
    {
        m_gate: Gate::MazeMap,
        m_name: "maze_map",
        m_title: "And Now, to Find the Exit",
        m_description: "Craft the maze map after obtaining the focus from the labyrinth.",
        m_icon: "twilightforest:maze_map",
        m_parent: Some(Gate::Lich),
    },
    Step
    {
        m_gate: Gate::OreMap,
        m_name: "ore_map",
        m_title: "How Can That Be Worth It?",
        m_description: "Craft the maze/ore map.",
        m_icon: "twilightforest:ore_map",
        m_parent: Some(Gate::MazeMap),
    },
    Step
    {
        m_gate: Gate::HydraChop,
        m_name: "hydra_chop",
        m_title: "Massive Chops",
        m_description: "Chow down on some overpowered hydra meat when your food bar is empty.",
        m_icon: "twilightforest:hydra_chop",
        m_parent: Some(Gate::KillHydra),
    },
    Step
    {
        m_gate: Gate::Mazebreaker,
        m_name: "mazebreaker",
        m_title: "Breaking the Maze",
        m_description: "Find the Mazebreaker pick in the secret labyrinth vault.",
        m_icon: "twilightforest:mazebreaker_pickaxe",
        m_parent: Some(Gate::Lich),
    },
    Step
    {
        m_gate: Gate::FierySet,
        m_name: "fiery_set",
        m_title: "Gallons of Blood and Tears",
        m_description: "Wield a fiery tool or weapon while wearing at least one piece of fiery armor.",
        m_icon: "twilightforest:fiery_sword",
        m_parent: Some(Gate::Hydra),
    },
    Step
    {
        m_gate: Gate::QuestRam,
        m_name: "quest_ram",
        m_title: "Consumate Baaahs",
        m_description: "Give the Quest Ram what it is missing.",
        m_icon: "twilightforest:crumble_horn",
        m_parent: Some(Gate::Arrival),
    },
];

// the player data bits progress lived in before the loader kept mod achievements
const LEGACY_KEY: &str = "progress";

fn id(gate: Gate) -> String { format!("twilightforest:{}", STEPS[gate as usize].m_name) }

// achievements earned before the loader kept them move over once, parents first
fn adopt(player: Player)
{
    let bits: u64 = player.data(LEGACY_KEY);
    if bits == 0
    {
        return;
    }

    for step in STEPS.iter().filter(|step| bits & 1 << step.m_gate as u32 != 0)
    {
        unlock(player, step.m_gate);
    }

    if let Err(error) = player.setData(LEGACY_KEY, &0u64)
    {
        warn(&format!("progress adopt: {}", error.0));
    }
}

fn unlock(player: Player, gate: Gate)
{
    if let Err(error) = player.unlockAchievement(id(gate))
    {
        warn(&format!("progress {gate:?}: {}", error.0));
    }
}

pub(crate) fn earned(player: Player) -> u64
{
    adopt(player);

    let earned = STEPS.iter().filter(|step| player.hasAchievement(id(step.m_gate)));
    earned.fold(0, |bits, step| bits | 1 << step.m_gate as u32)
}

pub(crate) fn has(player: Player, gate: Gate) -> bool
{
    if player.hasAchievement(id(gate))
    {
        return true;
    }

    adopt(player);
    player.hasAchievement(id(gate))
}

// EntityPlayer.triggerAchievement, the loader holds back a child until its parent is earned and shows beta's toast
pub(crate) fn grant(player: Player, gate: Gate)
{
    adopt(player);
    unlock(player, gate);
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
pub(crate) fn required(biome: Biome) -> Option<Gate>
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
    // TFBiomeSnow and TFBiomeGlacier, five seconds of slowness three and four
    pub const SNOW_SLOW: i32 = 2;
    pub const GLACIER_SLOW: i32 = 3;
    pub const SLOW_TICKS: i32 = 100;
    pub const BURN_TICKS: f64 = 8.0 * 20.0;
    // one check in four may bring a hint kobold for the feature that unlocks the biome
    pub const HINT_ODDS: i32 = 4;
}

// TFBiome*.enforceProgession, the forest pushes back on players who arrive too early; beta has no blindness or
// hunger, so the swamp and the dark forest only send hint kobolds
fn punish(world: World, player: Player, info: &EntityInfo, biome: Biome, now: i64)
{
    let entity: Entity = Entity { world, id: player.id };

    if biome == Biome::Highlands
    {
        crate::combat::hit(entity, 1, None, crate::combat::kind::MAGIC);
        world.playSound(info.pos, "random.fizz", 1.0, 1.0);
    }

    if now % enforce::CHECK != 0
    {
        return;
    }

    match biome
    {
        Biome::SnowyForest => chill(world, entity, info, enforce::SNOW_SLOW),
        Biome::Glacier => chill(world, entity, info, enforce::GLACIER_SLOW),
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
        let pos: BlockPos = blockOf(info.pos);
        crate::hints::trySpawn(world, player, feature, pos);
    }
}

fn chill(world: World, entity: Entity, info: &EntityInfo, level: i32)
{
    crate::effects::give(entity, crate::effects::Effect::Slowness, level, enforce::SLOW_TICKS);
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
        let biome: Biome = biomes::at(world.seed(), floor(info.pos[0]), floor(info.pos[2]));
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

pub(crate) fn doorLocked(_world: World, _pos: BlockPos) -> bool { false }

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for step in &STEPS
    {
        let mut spec: Achievement = Achievement::new(step.m_title, step.m_description, step.m_icon);
        if let Some(parent) = step.m_parent
        {
            spec = spec.after(id(parent));
        }

        ctx.achievements().register(step.m_name, spec)?;
    }

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
