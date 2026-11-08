use crate::config;
use crate::portal::aetherDimension;
use b173_api::prelude::*;
use serde::{Deserialize, Serialize};

const NAME: &str = "sun_altar";
const SKY: &str = "aether_sky";
const SET_TIME: &str = "sun_altar_time";
const STATE: &str = "eternal_day";
const GUI: &str = "aether:gui/sun_altar";
const BUTTONS: &str = "minecraft:gui/gui";

mod day
{
    pub const LENGTH: i64 = 24000;
    // AetherWorldProvider holds the sun at noon until the sun spirit falls
    pub const NOON: i64 = 6000;
    // how often a steady sky is sent again, for players who joined since
    pub const RESEND: i64 = 20;
}

mod altar
{
    pub const HARDNESS: f32 = 2.5;
    pub const WIDTH: i32 = 175;
    pub const HEIGHT: i32 = 78;
    pub const TITLE_Y: i32 = 20;
    pub const TITLE_COLOR: u32 = 0x404040;
}

mod slider
{
    pub const WIDTH: i32 = 150;
    pub const HEIGHT: i32 = 20;
    pub const LEFT: i32 = 75;
    pub const KNOB: i32 = 4;
    pub const TRACK: i32 = WIDTH - 2 * KNOB;
    // GuiSunAltarSlider.getHoverState is always 0, the dark button row of gui.png
    pub const BACK_V: i32 = 46;
    pub const BACK_RIGHT_U: i32 = 200;
    pub const KNOB_V: i32 = 66;
    pub const KNOB_RIGHT_U: i32 = 196;
    pub const TEXT_COLOR: u32 = 0xE0E0E0;
    pub const HOVER_COLOR: u32 = 0xFFFFA0;
    pub const TEXT_Y: i32 = (HEIGHT - 8) / 2;
}

// AetherData: the sun spirit's defeat and the sky's walk back to the world time
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
struct Sky
{
    m_unlocked: bool,
    m_caughtUp: bool,
    m_time: i64,
}

impl Default for Sky
{
    fn default() -> Sky { Sky { m_unlocked: false, m_caughtUp: false, m_time: day::NOON } }
}

fn eternalDay() -> bool { !config::enabled(config::DISABLE_ETERNAL_DAY) }

// EntitySunSpirit.onDeath: the sky starts walking back to the world time
pub fn unlock()
{
    let mut sky: Sky = worldData(STATE);
    if eternalDay() && !sky.m_unlocked
    {
        sky.m_unlocked = true;
        setWorldData(STATE, &sky);
    }
}

// AetherWorldProvider.calculateCelestialAngle: noon until unlocked, then one tick back per tick until it meets the world
fn advance(sky: &mut Sky, worldTime: i64)
{
    if !sky.m_unlocked
    {
        sky.m_time = day::NOON;
    }
    else if !sky.m_caughtUp
    {
        let met: bool =
            sky.m_time == worldTime % day::LENGTH || sky.m_time == worldTime + 1 || sky.m_time == worldTime - 1;
        if met
        {
            sky.m_caughtUp = true;
        }
        else
        {
            sky.m_time = (sky.m_time - 1).rem_euclid(day::LENGTH);
        }
    }
}

fn shown(sky: &Sky) -> Option<i64> { (!sky.m_caughtUp && eternalDay()).then_some(sky.m_time) }

fn tick()
{
    let Some(dimension) = aetherDimension()
    else
    {
        return;
    };
    let world: World = World { dimension };
    let occupied: bool = allPlayers().iter().any(|player| player.info().is_some_and(|info| info.world == dimension));
    if !occupied || !world.isAuthority()
    {
        return;
    }
    let before: Sky = worldData(STATE);
    let mut sky: Sky = before;
    let worldTime: i64 = world.time();
    if eternalDay()
    {
        advance(&mut sky, worldTime);
    }
    if sky != before
    {
        setWorldData(STATE, &sky);
    }
    let time: Option<i64> = shown(&sky);
    world.setSkyTime(time);
    if sky != before || worldTime % day::RESEND == 0
    {
        let _ = send(SKY, Target::Dimension(dimension), &time);
    }
}

fn received(time: Option<i64>, _player: Player)
{
    if let Some(dimension) = aetherDimension()
    {
        World { dimension }.setSkyTime(time);
    }
}

// GuiSunAltarSlider and PacketSetTime: the time a slider position asks for, always ahead of the current time
fn timeFor(value: f32, worldTime: i64) -> i64
{
    let wanted: i64 = (day::LENGTH as f32 * value) as i64;
    let remainder: i64 = worldTime % day::LENGTH;
    let add: i64 = if wanted > remainder
    {
        wanted - remainder
    }
    else
    {
        wanted + day::LENGTH - remainder
    };
    worldTime + add
}

fn allowed(player: Player) -> bool { player.isOp() || config::enabled(config::SUN_ALTAR_MULTIPLAYER) }

fn setTime(value: f32, player: Player)
{
    let inAether: bool = player.info().is_some_and(|info| Some(info.world) == aetherDimension());
    let sky: Sky = worldData(STATE);
    if !inAether || !allowed(player) || !value.is_finite() || !(sky.m_caughtUp || !eternalDay())
    {
        return;
    }
    let Some(info) = player.info()
    else
    {
        return;
    };
    setWorldTime(timeFor(value.clamp(0.0, 1.0), World { dimension: info.world }.time()));
}

struct SunAltar;

#[block_behavior]
impl BlockBehavior for SunAltar
{
    // BlockSunAltar.onBlockActivated: the altar answers only in the Aether and only once the sun spirit is gone
    fn onActivated(&self, ctx: &mut BlockCtx, player: Player) -> bool
    {
        if !ctx.isAuthority()
        {
            return true;
        }
        let sky: Sky = worldData(STATE);
        let key: Option<&str> = if Some(ctx.world().dimension) != aetherDimension()
        {
            Some("message")
        }
        else if eternalDay() && !sky.m_unlocked
        {
            Some("eternal_day")
        }
        else if eternalDay() && !sky.m_caughtUp
        {
            Some("cycle_catchup")
        }
        else if !allowed(player)
        {
            Some("permission")
        }
        else
        {
            None
        };
        match key
        {
            Some(key) => player.chat(&translate(&format!("aether.gui.sun_altar.{key}"))),
            None =>
            {
                let _ = player.openScreen(NAME, None, Vec::new());
            }
        }
        true
    }
}

// GuiSunAltar with its one slider, dragging previews the time here and the release asks the server
#[derive(Default)]
struct AltarScreen
{
    m_dimension: i32,
    m_button: [i32; 2],
    m_dragging: bool,
    m_value: f32,
}

impl AltarScreen
{
    fn over(&self, x: i32, y: i32) -> bool
    {
        let [left, top] = self.m_button;
        x >= left && y >= top && x < left + slider::WIDTH && y < top + slider::HEIGHT
    }

    fn valueAt(&self, x: i32) -> f32 { (x - (self.m_button[0] + slider::KNOB)) as f32 / slider::TRACK as f32 }
}

impl Screen for AltarScreen
{
    fn init(&mut self, width: i32, height: i32, _pos: Option<BlockPos>, _data: &[u8])
    {
        let origin: [i32; 2] = [(width - altar::WIDTH) / 2, (height - altar::HEIGHT) / 2];
        self.m_button = [width / 2 - slider::LEFT - origin[0], height / 2 - origin[1]];
        self.m_dimension = localPlayer().and_then(|player| player.info()).map_or(0, |info| info.world);
        self.m_dragging = false;
    }

    fn draw(&mut self, draw: &mut Draw, mouse: [i32; 2], _partial: f32)
    {
        let world: World = World { dimension: self.m_dimension };
        let title: &str = "Sun Altar";
        draw.texture(GUI, 0, 0, 0, 0, altar::WIDTH, altar::HEIGHT);
        draw.plain((altar::WIDTH - textWidth(title)) / 2, altar::TITLE_Y, title, altar::TITLE_COLOR);
        let [left, top] = self.m_button;
        let half: i32 = slider::WIDTH / 2;
        draw.texture(BUTTONS, left, top, 0, slider::BACK_V, half, slider::HEIGHT);
        draw.texture(BUTTONS, left + half, top, slider::BACK_RIGHT_U - half, slider::BACK_V, half, slider::HEIGHT);
        self.m_value = (world.time() % day::LENGTH) as f32 / day::LENGTH as f32;
        if self.m_dragging
        {
            self.m_value = self.valueAt(mouse[0]);
            setWorldTime(timeFor(self.m_value, world.time()));
            self.m_value = self.m_value.clamp(0.0, 1.0);
        }
        let knob: i32 = left + (self.m_value * slider::TRACK as f32) as i32;
        draw.texture(BUTTONS, knob, top, 0, slider::KNOB_V, slider::KNOB, slider::HEIGHT);
        draw.texture(
            BUTTONS,
            knob + slider::KNOB,
            top,
            slider::KNOB_RIGHT_U,
            slider::KNOB_V,
            slider::KNOB,
            slider::HEIGHT,
        );
        let color: u32 = if self.over(mouse[0], mouse[1])
        {
            slider::HOVER_COLOR
        }
        else
        {
            slider::TEXT_COLOR
        };
        draw.centered(left + half, top + slider::TEXT_Y, "Select Time", color);
    }

    fn mouse(&mut self, x: i32, y: i32, button: i32)
    {
        if button == 0 && self.over(x, y)
        {
            self.m_value = self.valueAt(x).clamp(0.0, 1.0);
            self.m_dragging = true;
        }
    }

    fn release(&mut self, _x: i32, _y: i32, _button: i32)
    {
        if self.m_dragging
        {
            self.m_dragging = false;
            let _ = send(SET_TIME, Target::Server, &self.m_value);
        }
    }
}

pub fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    let texture = |name: &str| format!("aether:block/{name}");
    ctx.blocks().register(
        NAME,
        BlockSpecBuilder::template("minecraft:block")
            .material("rock")
            .faces(FaceTextures {
                top: Some(texture("sun_altar_top")),
                bottom: Some(texture("hellfire_stone")),
                side: Some(texture("sun_altar_side")),
                front: None,
            })
            .hardness(altar::HARDNESS)
            .sound("stone")
            .tool(ToolClass::Pickaxe, 0)
            .behavior(SunAltar),
    )?;
    ctx.screens().register(NAME, ScreenBuilder::new("", altar::WIDTH, altar::HEIGHT), AltarScreen::default())?;
    ctx.channels().register::<Option<i64>>(SKY, Direction::ToClient, received)?;
    ctx.channels().register::<f32>(SET_TIME, Direction::ToServer, setTime)?;
    ctx.events().on::<TickPost>(Priority::Normal, |_| tick())?;
    Ok(())
}
