use b173_api::prelude::*;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;

// AchievementsAether, the Aether I page on the loader's mod achievements
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Award
{
    EnterAether,
    Bronze,
    Silver,
    Gold,
    Enchanter,
    Incubator,
    BlueCloud,
    FlyingPig,
    GravTools,
    Loreception,
}

struct Row
{
    m_award: Award,
    m_name: &'static str,
    m_title: &'static str,
    m_description: &'static str,
    m_icon: &'static str,
    // AetherMusicHandler.getAchievementSound, played to the player on a first unlock
    m_sound: &'static str,
}

mod sound
{
    pub const GENERAL: &str = "aether.achievement.achievement_gen";
    pub const BRONZE: &str = "aether.achievement.achievement_bronze";
    pub const SILVER: &str = "aether.achievement.achievement_silver";
}

// rows in Award order, enter_aether is every other row's parent
const ROWS: [Row; 10] = [
    Row
    {
        m_award: Award::EnterAether,
        m_name: "enter_aether",
        m_title: "Hostile Paradise",
        m_description: "Enter the Aether",
        m_icon: "minecraft:glowstone",
        m_sound: sound::GENERAL,
    },
    Row
    {
        m_award: Award::Bronze,
        m_name: "bronze_dungeon",
        m_title: "Like a Bossaru!",
        m_description: "Defeat the bronze dungeon boss",
        m_icon: "aether:bronze_key",
        m_sound: sound::BRONZE,
    },
    Row
    {
        m_award: Award::Silver,
        m_name: "silver_dungeon",
        m_title: "Dethroned",
        m_description: "Defeat the silver dungeon boss",
        m_icon: "aether:silver_key",
        m_sound: sound::SILVER,
    },
    Row
    {
        m_award: Award::Gold,
        m_name: "gold_dungeon",
        m_title: "Extinguished",
        m_description: "Defeat the golden dungeon boss",
        m_icon: "aether:golden_key",
        m_sound: sound::GENERAL,
    },
    Row
    {
        m_award: Award::Enchanter,
        m_name: "enchanter",
        m_title: "Do you believe in magic?",
        m_description: "Craft an Altar",
        m_icon: "aether:enchanter",
        m_sound: sound::GENERAL,
    },
    Row
    {
        m_award: Award::Incubator,
        m_name: "incubator",
        m_title: "Now you're family",
        m_description: "Incubate a Moa",
        m_icon: "aether:incubator",
        m_sound: sound::GENERAL,
    },
    Row
    {
        m_award: Award::BlueCloud,
        m_name: "blue_aercloud",
        m_title: "To infinity and beyond!",
        m_description: "Bounce on a Blue Aercloud",
        m_icon: "aether:blue_aercloud",
        m_sound: sound::GENERAL,
    },
    Row
    {
        m_award: Award::FlyingPig,
        m_name: "mount_phyg",
        m_title: "When Phygs fly",
        m_description: "Fly on a Phyg",
        m_icon: "minecraft:saddle",
        m_sound: sound::GENERAL,
    },
    Row
    {
        m_award: Award::GravTools,
        m_name: "gravitite_tools",
        m_title: "Pink is the new blue",
        m_description: "Craft a gravitite tool",
        m_icon: "aether:gravitite_pickaxe",
        m_sound: sound::GENERAL,
    },
    Row
    {
        m_award: Award::Loreception,
        m_name: "loreception",
        m_title: "Lore-ception!",
        m_description: "Put a book of lore inside a book of lore.",
        m_icon: "aether:lore_book",
        m_sound: sound::GENERAL,
    },
];

const GRAVITITE_TOOLS: [&str; 3] = ["aether:gravitite_shovel", "aether:gravitite_axe", "aether:gravitite_pickaxe"];
const ENCHANTER: &str = "aether:enchanter";
const LORE_BOOK: &str = "aether:lore_book";
const PLAYER_HALF_WIDTH: f64 = 0.3;
const PLAYER_HEIGHT: f64 = 1.8;

thread_local!
{
    // TileEntityIncubator.owner, the last player to use an incubator, kept only in memory as in the source
    static OWNERS: RefCell<HashMap<(i32, i32, i32, i32), i32>> = RefCell::new(HashMap::new());
    // players who opened a book of lore and have not put one inside it yet
    static READERS: RefCell<Vec<i32>> = const { RefCell::new(Vec::new()) };
}

fn id(award: Award) -> String { format!("aether:{}", ROWS[award as usize].m_name) }

fn identifier(text: &str) -> Result<Identifier>
{
    Identifier::parse(text).map_err(|error| Error(format!("achievement {text}: {error:?}")))
}

// EntityPlayer.triggerAchievement, a first unlock plays the Aether's own jingle as PacketAchievement does
pub(crate) fn grant(player: Player, award: Award)
{
    match player.unlockAchievement(&id(award))
    {
        Ok(true) =>
        {
            if let Some(info) = player.info()
            {
                (World { dimension: info.world }).playSound(info.pos, ROWS[award as usize].m_sound, 1.0, 1.0);
            }
        }

        Ok(false) =>
        {}
        Err(error) => warn(&format!("achievement {award:?}: {}", error.0)),
    }
}

// getPlayersInDungeon: every player whose box touches the dungeon point grown by reach
pub(crate) fn grantInside(world: World, center: [i32; 3], reach: [f64; 3], award: Award)
{
    for player in world.players()
    {
        let Some(info) = player.info()
        else
        {
            continue;
        };

        let [x, y, z] = info.pos;
        let [cx, cy, cz] = center.map(f64::from);
        let inside: bool = (x - cx).abs() < reach[0] + PLAYER_HALF_WIDTH
            && y < cy + reach[1]
            && y + PLAYER_HEIGHT > cy - reach[1]
            && (z - cz).abs() < reach[2] + PLAYER_HALF_WIDTH;

        if inside
        {
            grant(player, award);
        }
    }
}

// SlotIncubator.putStack names the player who fed the incubator as its owner
pub(crate) fn setOwner(world: World, pos: BlockPos, player: Player)
{
    OWNERS.with(|owners| owners.borrow_mut().insert((world.dimension, pos.x, pos.y, pos.z), player.id));
}

pub(crate) fn owner(world: World, pos: BlockPos) -> Option<Player>
{
    OWNERS.with(|owners| owners.borrow().get(&(world.dimension, pos.x, pos.y, pos.z)).map(|id| Player { id: *id }))
}

pub(crate) fn read(player: Player)
{
    READERS.with(|readers|
    {
        let mut readers: RefMut<Vec<i32>> = readers.borrow_mut();
        if !readers.contains(&player.id)
        {
            readers.push(player.id);
        }
    });
}

// InventoryLore.setInventorySlotContents, polled since the loader has no slot callback
fn checkReaders()
{
    let readers: Vec<i32> = READERS.with(|readers| readers.borrow().clone());
    for id in readers
    {
        let player: Player = Player { id };
        let present: bool = player.info().is_some();
        let inside: bool = present && player.openSlot(0).is_some_and(|stack| stack.item.is(LORE_BOOK));
        if present && !inside
        {
            continue;
        }

        if inside
        {
            grant(player, Award::Loreception);
        }

        READERS.with(|readers| readers.borrow_mut().retain(|reader| *reader != id));
    }
}

// AetherEventHandler.onCrafting
fn crafted(event: &mut Event)
{
    let (Some(player), Some(stack), Some(world)) = (event.player(), event.stack(), event.world())
    else
    {
        return;
    };
    if !world.isAuthority()
    {
        return;
    }

    if GRAVITITE_TOOLS.iter().any(|tool| stack.item.is(tool))
    {
        grant(player, Award::GravTools);
    }
    else if stack.item.is(ENCHANTER)
    {
        grant(player, Award::Enchanter);
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for row in &ROWS
    {
        let parent: Option<Identifier> =
            (row.m_award != Award::EnterAether).then(|| identifier(&id(Award::EnterAether))).transpose()?;

        let spec: AchievementSpec = AchievementSpec
        {
            title: row.m_title.to_owned(),
            description: row.m_description.to_owned(),
            icon: identifier(row.m_icon)?,
            parent,
        };
        ctx.achievements().register(row.m_name, spec)?;
    }

    ctx.events().on::<ItemCrafted>(Priority::Normal, crafted)?;
    ctx.events().on::<TickPost>(Priority::Normal, |_| checkReaders())?;
    Ok(())
}
