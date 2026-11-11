use crate::hint_books::{AUTHOR, HINTS, Hint};
use crate::items::put;
use crate::world::features::Feature;
use b173_api::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

// TFFeature.trySpawnHintMonster, a kobold carrying the explorer's notes turns up near a player a guarded feature
// turns away, at most once a minute per feature, and drops the notes when it dies
const KOBOLD: &str = "twilightforest:kobold";
// beta's book icon, Item.book at column 11 row 3 of the items sheet
const BOOK_ICON: &str = "minecraft:items/59";

mod tuning
{
    pub const COOLDOWN: i64 = 1200;
    pub const TRIES: i32 = 20;
    pub const SPREAD: i32 = 16;
    pub const RISE: i32 = 4;
    // a kobold is a block wide and a little over one tall
    pub const HEADROOM: i32 = 2;
    pub const EYES: f64 = 0.85;
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub(crate) struct Notes
{
    pub m_book: Option<String>,
}

thread_local!
{
    static LAST: RefCell<HashMap<Option<Feature>, i64>> = RefCell::new(HashMap::new());
}

fn hintFor(feature: Feature) -> &'static Hint
{
    HINTS.iter().find(|hint| hint.m_feature == Some(feature)).unwrap_or(&HINTS[HINTS.len() - 1])
}

fn clear(world: World, pos: BlockPos) -> bool
{
    (0..tuning::HEADROOM).all(|dy|
    {
        let at: BlockPos = pos.offset(0, dy, 0);
        world.block(at) == 0
    }) && World::blockInfo(world.block(pos.offset(0, -1, 0))).solid
}

// didSpawnHintMonster, a clear spot within sixteen blocks that can see the player
fn spot(world: World, player: &EntityInfo, pos: BlockPos) -> Option<BlockPos>
{
    let spread = |range: i32| world.random(range) - world.random(range);

    (0..tuning::TRIES).find_map(|_|
    {
        let at: BlockPos = pos.offset(spread(tuning::SPREAD), spread(tuning::RISE), spread(tuning::SPREAD));
        let eyes: [f64; 3] = [f64::from(at.x) + 0.5, f64::from(at.y) + tuning::EYES, f64::from(at.z) + 0.5];
        (clear(world, at) && world.rayTrace(eyes, player.pos, false).is_none()).then_some(at)
    })
}

pub(crate) fn trySpawn(world: World, player: Player, feature: Feature, pos: BlockPos)
{
    let now: i64 = world.time();
    let hint: &Hint = hintFor(feature);
    let last: i64 = LAST.with(|last| last.borrow().get(&hint.m_feature).copied().unwrap_or(i64::MIN / 2));
    if now >= last && now - last <= tuning::COOLDOWN
    {
        return;
    }
    let Some(info) = player.info()
    else
    {
        return;
    };
    let Some(at) = spot(world, &info, pos)
    else
    {
        return;
    };

    let center: [f64; 3] = [f64::from(at.x) + 0.5, f64::from(at.y), f64::from(at.z) + 0.5];
    let Some(kobold) = crate::entities::spawnAt(world, KOBOLD, center, 0.0)
    else
    {
        return;
    };

    if let Err(error) = kobold.setData(&Notes { m_book: Some(hint.m_item.to_owned()) })
    {
        warn(&format!("hint kobold notes: {}", error.0));
    }

    LAST.with(|last| last.borrow_mut().insert(hint.m_feature, now));
}

// the notes a hint kobold drops on death, every time as its held book
pub(crate) fn dropNotes(entity: Entity, info: &EntityInfo)
{
    let notes: Notes = entity.data();
    let Some(book) = notes.m_book
    else
    {
        return;
    };

    if let Err(error) = entity.world.spawnItem(info.pos, ItemStack::of(book.as_str(), 1))
    {
        warn(&format!("hint kobold book: {}", error.0));
    }
}

// beta has no written book screen, the notes read out in chat with the book's light text
struct Book
{
    m_hint: &'static Hint,
}

#[item_behavior]
impl ItemBehavior for Book
{
    fn onRightClick(&self, world: World, player: Player, stack: ItemStack) -> Use
    {
        if world.isAuthority()
        {
            player.chat(&format!("§e{}§f by {AUTHOR}", self.m_hint.m_title));

            for page in self.m_hint.m_pages
            {
                for line in page.replace("§0", "§f").lines()
                {
                    player.chat(line);
                }
            }
        }

        Use::Done(Some(stack))
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for hint in &HINTS
    {
        put(
            ctx,
            hint.m_item,
            ItemSpecBuilder::new().name(hint.m_title).texture(BOOK_ICON).maxStack(1).behavior(Book { m_hint: hint }),
        )?;
    }

    Ok(())
}
