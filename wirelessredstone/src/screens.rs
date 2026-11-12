use crate::blocks::{CLOCKER, Kind, RECEIVER, TRANSMITTER, Tile};
use crate::{items, net};
use b173_api::prelude::*;
use b173_api::world::World;
use std::cell::{Cell, RefCell};
use wirelessredstone_api::FREQUENCIES;

pub(crate) const REMOTE: &str = "remote";
pub(crate) const CONFIGURATOR: &str = "power_configurator";
pub(crate) const SNIFFER: &str = "sniffer";
pub(crate) const TRIANGULATOR: &str = "triangulator";

mod sheet
{
    pub const SMALL: &str = "wirelessredstone:gui/wifi_small";
    pub const MEDIUM: &str = "wirelessredstone:gui/wifi_medium";
    pub const LARGE: &str = "wirelessredstone:gui/wifi_large";
    pub const SNIFFER: &str = "wirelessredstone:gui/sniffer";
    pub const BUTTON_ON: &str = "wirelessredstone:gui/button_on";
    pub const BUTTON_OFF: &str = "wirelessredstone:gui/button_off";
}

mod gui
{
    pub const WIDTH: i32 = 176;
    pub const HEIGHT: i32 = 166;
    // the source placed buttons from the screen center, this is the center inside the gui
    pub const CENTER: [i32; 2] = [WIDTH / 2, HEIGHT / 2];
    pub const TEXT: u32 = 0x404040;
    pub const BORDER: u32 = 0xFF000000;
    pub const FILL: u32 = 0xFFFFFFFF;
    pub const TITLE_Y: i32 = 6;
    pub const VALUE_Y: i32 = HEIGHT / 2 - 35;
    pub const BUTTON_HEIGHT: i32 = 20;
}

// drawStringBorder margins around a label, the box reaches further below the text than above
mod frame
{
    pub const EDGE: i32 = 3;
    pub const EDGE_BELOW: i32 = 10;
    pub const FILL: i32 = 2;
    pub const FILL_BELOW: i32 = 9;
}

mod sniffer
{
    pub const WIDTH: i32 = 256;
    pub const HEIGHT: i32 = 200;
    pub const GRID: &str = "sniffer_grid";
    pub const GRID_TEXTURE: &str = "wirelessredstone:sniffer_grid";
    pub const GRID_AT: [i32; 2] = [3, 20];
    pub const COLUMNS: usize = 125;
    pub const ROWS: usize = 80;
    // a 1 pixel cell and a 1 pixel gap
    pub const PITCH: usize = 2;
    pub const SHEET: usize = 256;
    pub const ON: u32 = 0xFF00FF00;
    pub const OFF: u32 = 0xFFFF0000;
}

// GuiButton sheet rows and label colors the source's GuiButtonBoolean drew from guiOn and guiOff
mod skin
{
    pub const FIRST_ROW: i32 = 46;
    pub const ROW: i32 = 20;
    pub const WIDTH: i32 = 200;
    pub const NORMAL: i32 = 1;
    pub const HOVER: i32 = 2;
    pub const LABEL: u32 = 0xE0E0E0;
    pub const LABEL_HOVER: u32 = 0xFFFFA0;
    pub const LABEL_Y: i32 = 8;
}

pub(crate) struct StepButton
{
    m_id: i32,
    m_at: [i32; 2],
    m_width: i32,
    m_label: &'static str,
    pub m_step: i32,
}

const fn step(id: i32, x: i32, y: i32, width: i32, label: &'static str, step: i32) -> StepButton
{
    StepButton { m_id: id, m_at: [x, y], m_width: width, m_label: label, m_step: step }
}

pub(crate) const FREQ_STEPS: [StepButton; 8] = [
    step(0, 10, -20, 20, "+", 1),
    step(1, -30, -20, 20, "-", -1),
    step(2, 32, -20, 20, "+10", 10),
    step(3, -52, -20, 20, "-10", -10),
    step(4, 54, -20, 26, "+100", 100),
    step(5, -80, -20, 26, "-100", -100),
    step(6, 48, -42, 32, "+1000", 1000),
    step(7, -80, -42, 32, "-1000", -1000),
];

pub(crate) const CLOCK_STEPS: [StepButton; 6] = [
    step(8, 1, 45, 24, "+10", 10),
    step(9, -25, 45, 24, "-10", -10),
    step(10, 28, 45, 24, "+s", 1000),
    step(11, -53, 45, 24, "-s", -1000),
    step(12, 56, 45, 24, "+m", 60000),
    step(13, -81, 45, 24, "-m", -60000),
];

// GuiRedstoneWirelessPowerDirector, one column per side with its label and source side index
struct RouteButton
{
    m_x: i32,
    m_label: &'static str,
    m_side: u8,
}

const ROUTES: [RouteButton; 6] = [
    RouteButton { m_x: -60, m_label: "N", m_side: 5 },
    RouteButton { m_x: -40, m_label: "E", m_side: 3 },
    RouteButton { m_x: -20, m_label: "S", m_side: 4 },
    RouteButton { m_x: 0, m_label: "W", m_side: 2 },
    RouteButton { m_x: 20, m_label: "U", m_side: 0 },
    RouteButton { m_x: 40, m_label: "D", m_side: 1 },
];

mod route
{
    pub const DIRECT_Y: i32 = -42;
    pub const INDIRECT_Y: i32 = 0;
    pub const SIZE: i32 = 20;
    pub const DIRECTION_Y: i32 = 28;
    pub const INDIRECT_LABEL_Y: i32 = 70;
}

mod remote
{
    pub const FREQ_LABEL_Y: i32 = 26;
    pub const PULSE: [i32; 4] = [-20, 5, 40, 20];
}

const FREQ_LABEL_Y: i32 = 32;
const PERIOD_LABEL_Y: i32 = 97;
const PERIOD_Y: i32 = gui::HEIGHT / 2 + 30;

thread_local!
{
    static ACTIVE: RefCell<Vec<u16>> = const { RefCell::new(Vec::new()) };
    static CLOSING: Cell<bool> = const { Cell::new(false) };
}

// closes the remote gui a pulse asked to close, from the client tick outside any screen callback
pub(crate) fn tick()
{
    if !CLOSING.with(|closing| closing.replace(false))
    {
        return;
    }

    if let Some(player) = localPlayer()
    {
        player.closeScreen();
    }
}

fn at(offset: [i32; 2]) -> [i32; 2] { [gui::CENTER[0] + offset[0], gui::CENTER[1] + offset[1]] }

fn local() -> Option<(Player, World)>
{
    let player: Player = localPlayer()?;
    let world: World = World { dimension: player.info()?.world };
    Some((player, world))
}

fn sendServer<T: Serialize>(channel: &str, message: &T)
{
    if let Err(error) = send(channel, Target::Server, message)
    {
        warn(&format!("{channel} message: {error:?}"));
    }
}

// drawStringBorder then the label, a white box with a black edge around dark text centered on the gui
fn boxed(draw: &mut Draw, text: &str, y: i32)
{
    let half: i32 = textWidth(text) / 2;
    let (left, right): (i32, i32) = (gui::CENTER[0] - half, gui::CENTER[0] + half);

    for (pad, below, color) in
        [(frame::EDGE, frame::EDGE_BELOW, gui::BORDER), (frame::FILL, frame::FILL_BELOW, gui::FILL)]
    {
        draw.rect(left - pad, y - pad, right - left + pad * 2, pad + below, color);
    }

    draw.text(left, y, text, gui::TEXT);
}

fn background(draw: &mut Draw, sheet: &str) { draw.texture(sheet, 0, 0, 0, 0, gui::WIDTH, gui::HEIGHT); }

fn stepButtons(draw: &mut Draw, buttons: &[StepButton])
{
    for button in buttons
    {
        let [x, y] = at(button.m_at);
        draw.button(button.m_id, x, y, button.m_width, gui::BUTTON_HEIGHT, button.m_label);
    }
}

fn stepOf(buttons: &[StepButton], id: i32) -> Option<i32>
{
    buttons.iter().find(|button| button.m_id == id).map(|button| button.m_step)
}

fn inside(mouse: [i32; 2], rect: [i32; 4]) -> bool
{
    let [x, y, w, h] = rect;
    mouse[0] >= x && mouse[1] >= y && mouse[0] < x + w && mouse[1] < y + h
}

// GuiButtonBoolean and GuiButtonPowerDirection, a vanilla button drawn from the on or off sheet
fn skinned(draw: &mut Draw, mouse: [i32; 2], rect: [i32; 4], label: &str, on: bool)
{
    let [x, y, w, h] = rect;
    let sheet: &str = if on
    {
        sheet::BUTTON_ON
    }
    else
    {
        sheet::BUTTON_OFF
    };
    let hovered: bool = inside(mouse, rect);
    let row: i32 = skin::FIRST_ROW + if hovered { skin::HOVER } else { skin::NORMAL } * skin::ROW;
    let color: u32 = if hovered
    {
        skin::LABEL_HOVER
    }
    else
    {
        skin::LABEL
    };

    draw.texture(sheet, x, y, 0, row, w / 2, h);
    draw.texture(sheet, x + w / 2, y, skin::WIDTH - w / 2, row, w / 2, h);
    draw.centered(x + w / 2, y + (h - skin::LABEL_Y) / 2, label, color);
}

// vanilla GuiScreen plays this when a button takes a click
fn click()
{
    if let Some((player, world)) = local()
        && let Some(info) = player.info()
    {
        world.playSound(info.pos, "random.click", 1.0, 1.0);
    }
}

// GuiRedstoneWirelessR, T and Clocker
struct FrequencyScreen
{
    m_kind: Kind,
    m_pos: Option<BlockPos>,
}

impl FrequencyScreen
{
    fn title(&self) -> String
    {
        let name: &str = match self.m_kind
        {
            Kind::Receiver => RECEIVER,
            Kind::Transmitter => TRANSMITTER,
            Kind::Clocker => CLOCKER,
        };
        translate(&format!("tile.wirelessredstone.{name}.name"))
    }
}

impl Screen for FrequencyScreen
{
    fn init(&mut self, _width: i32, _height: i32, pos: Option<BlockPos>, _data: &[u8]) { self.m_pos = pos; }

    fn draw(&mut self, draw: &mut Draw, _mouse: [i32; 2], _partial: f32)
    {
        let clocker: bool = self.m_kind == Kind::Clocker;
        let tile: Tile = match (local(), self.m_pos)
        {
            (Some((_, world)), Some(pos)) => world.tileData(pos),
            _ => Tile::new(),
        };

        background(draw, if clocker { sheet::LARGE } else { sheet::SMALL });
        boxed(draw, &self.title(), gui::TITLE_Y);
        boxed(draw, &translate("wirelessredstone.gui.frequency"), FREQ_LABEL_Y);

        if clocker
        {
            boxed(draw, &translate("wirelessredstone.gui.clock"), PERIOD_LABEL_Y);
        }

        stepButtons(draw, &FREQ_STEPS);

        if clocker
        {
            stepButtons(draw, &CLOCK_STEPS);
        }

        boxed(draw, &tile.m_freq.to_string(), gui::VALUE_Y);

        if clocker
        {
            boxed(draw, &tile.m_period.to_string(), PERIOD_Y);
        }
    }

    fn button(&mut self, id: i32)
    {
        if let Some(step) = stepOf(&FREQ_STEPS, id)
        {
            sendServer(net::FREQ, &net::Step { m_pos: self.m_pos, m_step: step });
        }

        if let Some(step) = stepOf(&CLOCK_STEPS, id)
        {
            sendServer(net::PERIOD, &net::Step { m_pos: self.m_pos, m_step: step });
        }
    }
}

fn heldFreq() -> i32
{
    localPlayer().map_or(0, |player| stackComponent::<i32>(StackRef::Held { player: player.id }, items::FREQ))
}

// GuiRedstoneWirelessRemote and GuiRedstoneWirelessTriangulator, the frequency of the held stack
struct ItemScreen
{
    m_remote: bool,
    m_pos: Option<BlockPos>,
}

impl ItemScreen
{
    fn pulseRect() -> [i32; 4]
    {
        let [x, y, w, h] = remote::PULSE;
        let [left, top] = at([x, y]);
        [left, top, w, h]
    }
}

impl Screen for ItemScreen
{
    fn init(&mut self, _width: i32, _height: i32, pos: Option<BlockPos>, _data: &[u8]) { self.m_pos = pos; }

    fn draw(&mut self, draw: &mut Draw, mouse: [i32; 2], _partial: f32)
    {
        let (sheet, title, labelY): (&str, &str, i32) = if self.m_remote
        {
            (sheet::MEDIUM, "item.wirelessredstone.remote.name", remote::FREQ_LABEL_Y)
        }
        else
        {
            (sheet::SMALL, "item.wirelessredstone.triangulator.name", FREQ_LABEL_Y)
        };

        background(draw, sheet);
        boxed(draw, &translate(title), gui::TITLE_Y);
        boxed(draw, &translate("wirelessredstone.gui.frequency"), labelY);
        boxed(draw, &heldFreq().to_string(), gui::VALUE_Y);
        stepButtons(draw, &FREQ_STEPS);

        if self.m_remote
        {
            skinned(draw, mouse, Self::pulseRect(), &translate("wirelessredstone.gui.pulse"), false);
        }
    }

    fn button(&mut self, id: i32)
    {
        if let Some(step) = stepOf(&FREQ_STEPS, id)
        {
            sendServer(net::ITEM_FREQ, &net::Step { m_pos: None, m_step: step });
        }
    }

    fn mouse(&mut self, x: i32, y: i32, button: i32)
    {
        let pressed: bool = self.m_remote && button == 0 && inside([x, y], Self::pulseRect());

        let Some(pos) = self.m_pos.filter(|_| pressed)
        else
        {
            return;
        };

        click();
        sendServer(net::PULSE, &net::Pulse { m_pos: pos });

        // the source closed the gui as the pulse started, closing inside this callback would reenter the screen
        CLOSING.with(|closing| closing.set(true));
    }
}

// GuiRedstoneWirelessPowerDirector
struct ConfiguratorScreen
{
    m_pos: Option<BlockPos>,
}

fn routeRect(button: &RouteButton, row: i32) -> [i32; 4]
{
    let [x, y] = at([button.m_x, row]);
    [x, y, route::SIZE, route::SIZE]
}

impl Screen for ConfiguratorScreen
{
    fn init(&mut self, _width: i32, _height: i32, pos: Option<BlockPos>, _data: &[u8]) { self.m_pos = pos; }

    fn draw(&mut self, draw: &mut Draw, mouse: [i32; 2], _partial: f32)
    {
        let tile: Tile = match (local(), self.m_pos)
        {
            (Some((_, world)), Some(pos)) => world.tileData(pos),
            _ => Tile::new(),
        };

        background(draw, sheet::MEDIUM);
        boxed(draw, &translate("tile.wirelessredstone.receiver.name"), gui::TITLE_Y);
        boxed(draw, &translate("wirelessredstone.gui.direction"), route::DIRECTION_Y);
        boxed(draw, &translate("wirelessredstone.gui.indirect"), route::INDIRECT_LABEL_Y);

        for button in &ROUTES
        {
            let side: usize = usize::from(button.m_side);
            skinned(draw, mouse, routeRect(button, route::DIRECT_Y), button.m_label, tile.m_direct[side]);
            skinned(draw, mouse, routeRect(button, route::INDIRECT_Y), button.m_label, tile.m_indirect[side]);
        }
    }

    fn mouse(&mut self, x: i32, y: i32, button: i32)
    {
        let Some(pos) = self.m_pos.filter(|_| button == 0)
        else
        {
            return;
        };

        for route in &ROUTES
        {
            let direct: bool = inside([x, y], routeRect(route, route::DIRECT_Y));
            let indirect: bool = inside([x, y], routeRect(route, route::INDIRECT_Y));

            if direct || indirect
            {
                click();
                sendServer(net::ROUTE, &net::Route { m_pos: pos, m_side: route.m_side, m_indirect: indirect });
                return;
            }
        }
    }
}

// GuiRedstoneWirelessSniffer, every frequency as a green or red cell, drawn from one texture painted on change
struct SnifferScreen;

pub(crate) fn sniffed(freqs: Vec<u16>)
{
    let changed: bool = ACTIVE.with(|active| *active.borrow() != freqs);

    if changed
    {
        paintGrid(&freqs);
        ACTIVE.with(|active| *active.borrow_mut() = freqs);
    }
}

fn paintGrid(active: &[u16])
{
    let mut pixels: Vec<u32> = vec![0; sniffer::SHEET * sniffer::SHEET];

    for freq in 0..FREQUENCIES as usize
    {
        let (row, column): (usize, usize) = (freq / sniffer::COLUMNS, freq % sniffer::COLUMNS);
        let on: bool = active.contains(&(freq as u16));
        pixels[row * sniffer::PITCH * sniffer::SHEET + column * sniffer::PITCH] =
            if on { sniffer::ON } else { sniffer::OFF };
    }

    if let Err(error) = setDynamicTexture(sniffer::GRID, sniffer::SHEET as u32, sniffer::SHEET as u32, pixels)
    {
        warn(&format!("sniffer grid: {error:?}"));
    }
}

impl Screen for SnifferScreen
{
    fn init(&mut self, _width: i32, _height: i32, _pos: Option<BlockPos>, _data: &[u8])
    {
        let active: Vec<u16> = ACTIVE.with(|active| active.borrow().clone());
        paintGrid(&active);
        sendServer(net::SNIFF, &());
    }

    fn draw(&mut self, draw: &mut Draw, _mouse: [i32; 2], _partial: f32)
    {
        let [x, y] = sniffer::GRID_AT;
        let size: [i32; 2] = [(sniffer::COLUMNS * sniffer::PITCH) as i32, (sniffer::ROWS * sniffer::PITCH) as i32];
        let title: String = translate("item.wirelessredstone.sniffer.name");

        draw.texture(sheet::SNIFFER, 0, 0, 0, 0, sniffer::WIDTH, sniffer::HEIGHT);
        draw.texture(sniffer::GRID_TEXTURE, x, y, 0, 0, size[0], size[1]);
        draw.text(sniffer::WIDTH / 2 - textWidth(&title) / 2, gui::TITLE_Y, &title, gui::TEXT);
    }

    // the source redrew from the live ether every frame, the server answers once per tick
    fn tick(&mut self) { sendServer(net::SNIFF, &()); }
}

pub(crate) fn screenFor(kind: Kind) -> &'static str
{
    match kind
    {
        Kind::Receiver => RECEIVER,
        Kind::Transmitter => TRANSMITTER,
        Kind::Clocker => CLOCKER,
    }
}

fn plain(width: i32, height: i32) -> ScreenBuilder { ScreenBuilder::new("", width, height) }

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for kind in [Kind::Receiver, Kind::Transmitter, Kind::Clocker]
    {
        ctx.screens().register(
            screenFor(kind),
            plain(gui::WIDTH, gui::HEIGHT),
            FrequencyScreen { m_kind: kind, m_pos: None },
        )?;
    }

    ctx.screens().register(REMOTE, plain(gui::WIDTH, gui::HEIGHT), ItemScreen { m_remote: true, m_pos: None })?;
    ctx.screens().register(
        TRIANGULATOR,
        plain(gui::WIDTH, gui::HEIGHT),
        ItemScreen { m_remote: false, m_pos: None },
    )?;
    ctx.screens().register(CONFIGURATOR, plain(gui::WIDTH, gui::HEIGHT), ConfiguratorScreen { m_pos: None })?;
    ctx.screens().register(SNIFFER, plain(sniffer::WIDTH, sniffer::HEIGHT), SnifferScreen)
}
