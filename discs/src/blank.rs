use b173_api::prelude::*;
use std::cell::Cell;

const BLANK: &str = "blank_disc";
const BURNED: &str = "burned_disc";
const SCREEN: &str = "burn";
const BURN: &str = "burn";
const PLAY: &str = "play";
const SONG: &str = "discs:song";
const JUKEBOX: &str = "minecraft:jukebox";
const EMPTY_JUKEBOX: i32 = 0;
const INVENTORY_SLOTS: i32 = 36;
const MAX_LINK: usize = 512;
const SECONDS_PER_MINUTE: i64 = 60;
const BYTES_PER_MEGABYTE: i64 = 1024 * 1024;

mod config
{
    pub const MINUTES: &str = "burn.max_minutes";
    pub const MEGABYTES: &str = "burn.max_megabytes";
    pub const DEFAULT_MINUTES: i64 = 10;
    pub const DEFAULT_MEGABYTES: i64 = 20;
    pub const MOST_MINUTES: i64 = 60;
    pub const MOST_MEGABYTES: i64 = 100;
}

mod keys
{
    pub const BACKSPACE: i32 = 14;
    pub const ENTER: i32 = 28;
    // lwjgl types control v as this character
    pub const PASTE: char = '\u{16}';
}

mod layout
{
    pub const WIDTH: i32 = 220;
    pub const HEIGHT: i32 = 92;
    pub const TITLE_Y: i32 = 8;
    pub const FIELD_X: i32 = 10;
    pub const FIELD_Y: i32 = 26;
    pub const FIELD_WIDTH: i32 = 200;
    pub const FIELD_HEIGHT: i32 = 16;
    pub const TEXT_INSET: i32 = 4;
    pub const HINT_Y: i32 = 46;
    pub const BUTTON_Y: i32 = 62;
    pub const BUTTON_WIDTH: i32 = 96;
    pub const BUTTON_HEIGHT: i32 = 20;
    pub const CANCEL_X: i32 = 114;
    // the caret shows for half of every second
    pub const BLINK_TICKS: i32 = 6;
}

mod color
{
    pub const TITLE: u32 = 0x404040;
    pub const HINT: u32 = 0x707070;
    pub const BORDER: u32 = 0xFFA0A0A0;
    pub const FIELD: u32 = 0xFF000000;
    pub const TEXT: u32 = 0xE0E0E0;
}

mod button
{
    pub const BURN: i32 = 0;
    pub const CANCEL: i32 = 1;
}

// what a burned disc holds, kept as a component on its stack
#[derive(Serialize, Deserialize, Default)]
struct Burned
{
    hash: String,
    title: String,
    seconds: i32,
}

#[derive(Serialize, Deserialize)]
struct Started
{
    world: i32,
    pos: BlockPos,
    song: Song,
}

thread_local!
{
    // the one import a server runs at a time and the player it is for
    static BURNING: Cell<Option<(i32, i32)>> = const { Cell::new(None) };
}

struct BlankDisc;

#[item_behavior]
impl ItemBehavior for BlankDisc
{
    fn onRightClick(&self, world: World, player: Player, _stack: ItemStack) -> Use
    {
        if world.isAuthority()
            && let Err(error) = player.openScreen(SCREEN, None, Vec::new())
        {
            warn(&format!("burn screen did not open: {error}"));
        }

        Use::Pass
    }
}

struct BurnedDisc;

#[item_behavior]
impl ItemBehavior for BurnedDisc
{
    // as ItemRecord.onItemUse the disc goes into an empty jukebox and every player near it hears the song from its start
    fn onUseOnBlock(&self, world: World, player: Player, _stack: ItemStack, pos: BlockPos, _face: i32) -> Use
    {
        let empty: bool = world.is(pos, JUKEBOX) && world.meta(pos) == EMPTY_JUKEBOX;
        if !empty || !world.isAuthority()
        {
            return Use::Pass;
        }

        let burned: Burned = stackComponent(StackRef::Held { player: player.id }, SONG);
        if burned.hash.is_empty()
        {
            return Use::Pass;
        }

        if let Err(error) = insertHeld(world, pos, player)
        {
            warn(&format!("disc did not go in: {error}"));
            return Use::Pass;
        }

        let song: Song = Song { hash: burned.hash, title: burned.title, start: world.time() };
        if let Err(error) =
            send(PLAY, Target::Tracking(world.dimension, pos), &Started { world: world.dimension, pos, song })
        {
            warn(&format!("song did not reach the players: {error}"));
        }

        Use::Done(None)
    }
}

#[derive(Default)]
struct BurnScreen
{
    m_link: String,
    m_ticks: i32,
}

impl BurnScreen
{
    // the end of a long link stays in view as it is typed
    fn shown(&self) -> &str
    {
        let room: i32 = layout::FIELD_WIDTH - layout::TEXT_INSET * 2;
        let mut start: usize = 0;
        while textWidth(&self.m_link[start..]) > room
        {
            start += self.m_link[start..].chars().next().map_or(1, char::len_utf8);
        }
        &self.m_link[start..]
    }

    fn add(&mut self, text: &str)
    {
        for character in text.chars().filter(|character| !character.is_control())
        {
            if self.m_link.len() + character.len_utf8() > MAX_LINK
            {
                return;
            }
            self.m_link.push(character);
        }
    }

    fn submit(&mut self)
    {
        let link: String = self.m_link.trim().to_owned();
        if link.is_empty()
        {
            return;
        }

        if let Err(error) = send(BURN, Target::Server, &link)
        {
            warn(&format!("burn request was not sent: {error}"));
        }

        if let Some(player) = localPlayer()
        {
            player.closeScreen();
        }
    }
}

impl Screen for BurnScreen
{
    fn init(&mut self, _width: i32, _height: i32, _pos: Option<BlockPos>, _data: &[u8]) { *self = Self::default(); }

    fn draw(&mut self, draw: &mut Draw, _mouse: [i32; 2], _partial: f32)
    {
        draw.panel(0, 0, layout::WIDTH, layout::HEIGHT);
        draw.centered(layout::WIDTH / 2, layout::TITLE_Y, &translate("gui.discs.burn.title"), color::TITLE);

        let (x, y, width, height): (i32, i32, i32, i32) =
            (layout::FIELD_X, layout::FIELD_Y, layout::FIELD_WIDTH, layout::FIELD_HEIGHT);
        draw.rect(x - 1, y - 1, width + 2, height + 2, color::BORDER);
        draw.rect(x, y, width, height, color::FIELD);

        let caret: &str = if self.m_ticks / layout::BLINK_TICKS % 2 == 0
        {
            "_"
        }
        else
        {
            ""
        };
        let textY: i32 = y + (height - layout::TEXT_INSET * 2) / 2;
        draw.shadowed(x + layout::TEXT_INSET, textY, &format!("{}{caret}", self.shown()), color::TEXT);

        draw.text(layout::FIELD_X, layout::HINT_Y, &translate("gui.discs.burn.hint"), color::HINT);
        draw.button(
            button::BURN,
            layout::FIELD_X,
            layout::BUTTON_Y,
            layout::BUTTON_WIDTH,
            layout::BUTTON_HEIGHT,
            &translate("gui.discs.burn.done"),
        );
        draw.button(
            button::CANCEL,
            layout::CANCEL_X,
            layout::BUTTON_Y,
            layout::BUTTON_WIDTH,
            layout::BUTTON_HEIGHT,
            &translate("gui.cancel"),
        );
    }

    fn button(&mut self, id: i32)
    {
        match id
        {
            button::BURN => self.submit(),
            button::CANCEL =>
            {
                if let Some(player) = localPlayer()
                {
                    player.closeScreen();
                }
            }
            _ =>
            {}
        }
    }

    fn key(&mut self, character: char, key: i32)
    {
        match (character, key)
        {
            (keys::PASTE, _) => self.add(&clipboard().unwrap_or_default()),
            (_, keys::BACKSPACE) =>
            {
                self.m_link.pop();
            }
            (_, keys::ENTER) => self.submit(),
            _ => self.add(&character.to_string()),
        }
    }

    fn tick(&mut self) { self.m_ticks += 1; }
}

fn blankSlot(player: Player) -> Option<i32>
{
    (0..INVENTORY_SLOTS).find(|slot| player.slot(*slot).is_some_and(|stack| stack.item.is(BLANK)))
}

// a player asked to burn a link, the server downloads it once nothing else is burning
fn requested(link: String, player: Player)
{
    let link: &str = link.trim();
    if link.is_empty() || link.len() > MAX_LINK
    {
        return;
    }

    if blankSlot(player).is_none()
    {
        player.chat(&translate("chat.discs.no_blank"));
        return;
    }

    if BURNING.get().is_some()
    {
        player.chat(&translate("chat.discs.busy"));
        return;
    }

    let limit: MediaLimit = MediaLimit
    {
        seconds: (Config.getInt(config::MINUTES) * SECONDS_PER_MINUTE) as i32,
        bytes: Config.getInt(config::MEGABYTES) * BYTES_PER_MEGABYTE,
    };

    match importMedia(link, limit)
    {
        Ok(job) =>
        {
            BURNING.set(Some((job, player.id)));
            player.chat(&translate("chat.discs.burning"));
        }
        Err(error) => player.chat(&format!("{} {error}", translate("chat.discs.failed"))),
    }
}

fn tick()
{
    let Some((job, id)) = BURNING.get()
    else
    {
        return;
    };

    let player: Player = Player { id };
    match mediaStatus(job)
    {
        MediaStatus::Running => return,
        MediaStatus::Done(info) => finish(player, info),
        MediaStatus::Failed(reason) => player.chat(&format!("{} {reason}", translate("chat.discs.failed"))),
        MediaStatus::Unknown =>
        {}
    }

    BURNING.set(None);
}

// the first blank disc in the inventory becomes the burned one
fn finish(player: Player, info: MediaInfo)
{
    let Some(slot) = blankSlot(player)
    else
    {
        player.chat(&translate("chat.discs.no_blank"));
        return;
    };

    let burned: Burned = Burned { hash: info.hash, title: info.title, seconds: info.seconds };
    let title: String = burned.title.clone();
    let written: Result<()> = player
        .setSlot(slot, Some(ItemStack::of(BURNED, 1)))
        .and_then(|()| setStackComponent(StackRef::Inventory { player: player.id, slot }, SONG, &burned));

    match written
    {
        Ok(()) => player.chat(&format!("{} {title}", translate("chat.discs.burned"))),
        Err(error) => warn(&format!("burned disc not given: {error}")),
    }
}

fn started(message: Started, _player: Player)
{
    if let Err(error) = playSong(World { dimension: message.world }, message.pos, message.song)
    {
        warn(&format!("song did not start: {error}"));
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.config().options(vec![
        Setting::int(config::MINUTES, config::DEFAULT_MINUTES, 1..=config::MOST_MINUTES)
            .label("Longest Song (minutes)"),
        Setting::int(config::MEGABYTES, config::DEFAULT_MEGABYTES, 1..=config::MOST_MEGABYTES)
            .label("Largest Song (MB)"),
    ])?;

    ctx.items()
        .register(BLANK, ItemSpecBuilder::new().texture("discs:item/blank_disc").maxStack(1).behavior(BlankDisc))?;
    ctx.items().register(
        BURNED,
        ItemSpecBuilder::new().texture("discs:item/burned_disc").maxStack(1).tag(super::DISC_TAG).behavior(BurnedDisc),
    )?;

    ctx.recipes()
        .shaped(BLANK, ItemStack::of(BLANK, 1))
        .rows(&[" C ", "CFC", " C "])
        .key('C', "minecraft:coal")
        .key('F', "minecraft:flint")
        .build()?;
    // a burned disc wipes back to blank
    ctx.recipes().shapeless("wipe_disc", ItemStack::of(BLANK, 1), &[ItemStack::of(BURNED, 1)])?;

    ctx.screens().register(SCREEN, ScreenBuilder::new("", layout::WIDTH, layout::HEIGHT), BurnScreen::default())?;
    ctx.channels().register::<String>(BURN, Direction::ToServer, requested)?;
    ctx.channels().register::<Started>(PLAY, Direction::ToClient, started)?;
    ctx.events().on::<TickPost>(Priority::Normal, |_| tick())
}
