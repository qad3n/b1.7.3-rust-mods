mod engine;
mod model;
mod oil;
mod oil_fx;

use b173_api::prelude::*;
pub(crate) use engine::acceptsPipe;
use engine::{ENGINES, EngineRow, Kind};
use std::rc::Rc;

const TILE: &str = "engine";
const HARDNESS: f32 = 0.5;

mod gui
{
    pub const WIDTH: i32 = 176;
    pub const HEIGHT: i32 = 166;
    pub const INVENTORY: [i32; 2] = [8, 84];
    pub const INVENTORY_LABEL_DROP: i32 = 94;
    pub const TEXT: u32 = 4210752;
    pub const FULL: i32 = 1000;
    pub const BAR_PAD: i32 = 2;
    pub const TITLE_Y: i32 = 6;
}

// TileEngine holds one fuel slot
const SLOTS: i32 = 1;

// GuiSteamEngine and GuiCombustionEngine: the fuel slot and the burn gauge
struct Gauge
{
    m_title: &'static str,
    m_titleX: i32,
    m_background: &'static str,
    m_slot: [i32; 2],
    m_bar: [i32; 4],
    m_width: i32,
}

const STEAM_GAUGE: Gauge = Gauge
{
    m_title: "buildcraft.gui.steam_engine",
    m_titleX: 60,
    m_background: "buildcraft:gui/steam_engine",
    m_slot: [80, 41],
    m_bar: [80, 24, 176, 12],
    m_width: 14,
};

const COMBUSTION_GAUGE: Gauge = Gauge
{
    m_title: "buildcraft.gui.combustion_engine",
    m_titleX: 50,
    m_background: "buildcraft:gui/combustion_engine",
    m_slot: [52, 41],
    m_bar: [104, 19, 176, 58],
    m_width: 16,
};

pub(crate) fn isEngine(world: World, pos: BlockPos) -> bool { engine::rowOf(world, pos).is_some() }

struct EngineScreen
{
    m_gauge: &'static Gauge,
    m_pos: Option<BlockPos>,
}

impl Screen for EngineScreen
{
    fn init(&mut self, _width: i32, _height: i32, pos: Option<BlockPos>, _data: &[u8]) { self.m_pos = pos; }

    fn draw(&mut self, draw: &mut Draw, _mouse: [i32; 2], _partial: f32)
    {
        let gauge: &Gauge = self.m_gauge;
        draw.text(gauge.m_titleX, gui::TITLE_Y, &translate(gauge.m_title), gui::TEXT);
        draw.text(
            gui::INVENTORY[0],
            gui::HEIGHT - gui::INVENTORY_LABEL_DROP,
            &translate("buildcraft.gui.inventory"),
            gui::TEXT,
        );

        let Some(pos) = self.m_pos
        else
        {
            return;
        };
        let Some(world) = localPlayer().and_then(|player| player.info()).map(|info| World { dimension: info.world })
        else
        {
            return;
        };
        let scaled: i32 = engine::shown(world, pos).map_or(0, |view| view.m_scaledBurn);
        if scaled <= 0
        {
            return;
        }

        let [x, y, u, height] = gauge.m_bar;
        let level: i32 = scaled * height / gui::FULL;
        draw.texture(gauge.m_background, x, y + height - level, u, height - level, gauge.m_width, level + gui::BAR_PAD);
    }
}

struct EngineBlock;

#[block_behavior]
impl BlockBehavior for EngineBlock
{
    fn onActivated(&self, ctx: &mut BlockCtx, player: Player) -> bool { engine::activated(ctx, player) }

    fn onPlaced(&self, ctx: &mut BlockCtx, _face: i32) { engine::placed(ctx); }

    // BlockEngine.randomDisplayTick, red dust around a burning engine
    fn onRandomDisplayTick(&self, ctx: &mut BlockCtx)
    {
        const HEIGHT: f64 = 6.0 / 16.0;
        const REACH: f64 = 0.52;
        const SPREAD: f64 = 0.6;
        const SIDE: f64 = 0.3;

        let world: World = ctx.world();
        let burning: bool = engine::shown(world, ctx.pos()).is_some_and(|view| view.m_burning);
        if !burning
        {
            return;
        }

        let [x, _, z] = crate::core::dir::center(ctx.pos());
        let y: f64 = ctx.pos().y as f64 + crate::core::random::float(world) * HEIGHT;
        let side: f64 = crate::core::random::float(world) * SPREAD - SIDE;

        for spot in
            [[x - REACH, y, z + side], [x + REACH, y, z + side], [x + side, y, z - REACH], [x + side, y, z + REACH]]
        {
            world.particle("reddust", spot, [0.0; 3]);
        }
    }
}

fn registerEngine(ctx: &mut RegisterCtx, row: &'static EngineRow) -> Result<()>
{
    ctx.blocks().register(
        row.m_name,
        BlockSpecBuilder::template("minecraft:block")
            .material("iron")
            .sound("metal")
            .hardness(HARDNESS)
            .texture("buildcraft:block/laser_transparent")
            .opacity(0)
            .model(BlockModelSpec { cuboids: Vec::new(), ..BlockModelSpec::default() })
            .collision(vec![crate::core::render::fullCuboid()])
            .tileEntity(TILE)
            .tag("buildcraft:engines")
            .behavior(EngineBlock),
    )?;

    // RenderEngine.inventoryRender, the engine at rest pointing up
    let item: ItemModelSpec = ItemModelSpec
    {
        cuboids: Vec::new(),
        tile: Some(Identifier::parse(&format!("buildcraft:{TILE}")).expect("the engine tile is an identifier")),
        pose: model::itemPose(row.m_name),
    };
    ctx.blocks().itemModel(row.m_name, item)?;

    // BuildCraftEnergy: a plate, a glass pane, two gears and a piston
    ctx.recipes()
        .shaped(row.m_name, ItemStack::of(format!("buildcraft:{}", row.m_name).as_str(), 1))
        .pattern(["www", " g ", "GpG"])
        .key('w', row.m_ingredient)
        .key('g', "minecraft:glass")
        .key('G', row.m_gear)
        .key('p', "minecraft:piston")
        .build()?;

    let gauge: &'static Gauge = match row.m_kind
    {
        Kind::Steam => &STEAM_GAUGE,
        Kind::Combustion => &COMBUSTION_GAUGE,
        Kind::Redstone => return Ok(()),
    };
    let screen: ScreenBuilder = ScreenBuilder::new("", gui::WIDTH, gui::HEIGHT)
        .background(gauge.m_background)
        .slot(engine::SLOT, gauge.m_slot[0], gauge.m_slot[1])
        .playerInventory(gui::INVENTORY[0], gui::INVENTORY[1]);
    ctx.screens().register(row.m_name, screen, EngineScreen { m_gauge: gauge, m_pos: None })
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    let tile: Rc<dyn TileBehavior> = Rc::new(engine::EngineTile);
    ctx.tileEntities().register(
        TILE,
        TileEntitySpec { inventory: SLOTS, ticks: true, sync: false, behavior: None },
        Some(tile),
    )?;
    ctx.tileEntities().model(TILE, model::spec())?;

    for row in &ENGINES
    {
        registerEngine(ctx, row)?;
    }

    engine::registerChannel(ctx)?;
    oil::register(ctx)?;
    oil_fx::register(ctx)
}
