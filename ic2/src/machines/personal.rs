use super::{Faces, Look, Machine, registerBlock, screen};
use crate::core::{dir, ids, own, sound};
use b173_api::prelude::*;
use std::rc::Rc;

// BlockPersonal.setBlockUnbreakable: only the owner's wrench takes these apart
const UNBREAKABLE: f32 = -1.0;

pub(crate) static MACHINES: [Machine; 2] = [
    Machine
    {
        m_name: "personal_safe",
        m_look: Look
        {
            m_idle: ["personal_safe_bottom", "personal_safe_top", "personal_safe_side", "personal_safe_front"],
            m_active: ["personal_safe_bottom", "personal_safe_top", "personal_safe_side", "personal_safe_front"],
            m_faces: Faces::Machine,
        },
        m_hardness: UNBREAKABLE,
        m_screen: false,
        m_wrench: 1.0,
        m_drop: "personal_safe",
        m_passes: &[],
        m_slots: safe::SLOTS,
    },
    Machine
    {
        m_name: "trade_o_mat",
        m_look: Look
        {
            m_idle: ["machine_bottom", "machine_top", "machine_side", "trade_o_mat_front"],
            m_active: ["machine_bottom", "machine_top", "machine_side", "trade_o_mat_front"],
            m_faces: Faces::Machine,
        },
        m_hardness: UNBREAKABLE,
        m_screen: false,
        m_wrench: 1.0,
        m_drop: "trade_o_mat",
        m_passes: &[],
        m_slots: trade::SLOTS,
    },
];

mod safe
{
    pub const SLOTS: i32 = 54;
    pub const COLUMNS: i32 = 9;
    pub const SLOT: i32 = 18;
    pub const LEFT: i32 = 8;
    pub const TOP: i32 = 18;
    pub const HEIGHT: i32 = 222;
    pub const INVENTORY_Y: i32 = 140;
}

mod trade
{
    pub const SLOTS: i32 = 4;
    pub const WANT: i32 = 0;
    pub const OFFER: i32 = 1;
    pub const PAID: i32 = 2;
    pub const GOODS: i32 = 3;
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, Default)]
pub(crate) struct Owned
{
    pub(crate) m_owner: Option<String>,
    m_trades: i32,
    m_ticker: i32,
}

fn name(player: Player) -> Option<String> { player.info().and_then(|info| info.player) }

// IPersonalBlock.canAccess: the first player to use it becomes its owner
pub(crate) fn canAccess(world: World, pos: BlockPos, player: Player) -> bool
{
    let mut owned: Owned = world.tileData(pos);
    let Some(name) = name(player)
    else
    {
        return false;
    };

    match &owned.m_owner
    {
        Some(owner) => owner.eq_ignore_ascii_case(&name),
        None =>
        {
            owned.m_owner = Some(name);
            let _ = world.setTileData(pos, &owned);
            true
        }
    }
}

pub(crate) fn isPersonal(world: World, pos: BlockPos) -> bool
{
    let block: i32 = world.block(pos);
    MACHINES.iter().any(|row| ids::block(&own(row.m_name)) == Some(block))
}

fn same(a: &Option<ItemStack>, b: &Option<ItemStack>) -> bool
{
    match (a, b)
    {
        (Some(a), Some(b)) => crate::core::stacks::same(a, b) && a.count == b.count,
        _ => false,
    }
}

// TileEntityTradeOMat.isAccessibleChest: a chest, or a personal safe of the same owner
fn tradeChest(world: World, pos: BlockPos, owner: &Option<String>) -> Option<BlockPos>
{
    let chest: Option<i32> = ids::block("minecraft:chest");
    let safe: Option<i32> = ids::block("ic2:personal_safe");

    [dir::UP, dir::DOWN, dir::EAST, dir::WEST, dir::SOUTH, dir::NORTH].iter().map(|face| dir::step(pos, *face)).find(
        |at|
        {
            let block: i32 = world.block(*at);
            Some(block) == chest || Some(block) == safe && &world.tileData::<Owned>(*at).m_owner == owner
        },
    )
}

// TileEntityTradeOMat.attemptTrade: the payment goes into an empty chest slot, the offered goods come out
fn attemptTrade(world: World, pos: BlockPos, chest: BlockPos) -> bool
{
    let offer: Option<ItemStack> = world.tileSlot(pos, trade::OFFER);
    let size: i32 = world.inventorySize(chest);
    let goods: Option<i32> = (0..size).find(|slot| same(&offer, &world.tileSlot(chest, *slot)));
    let empty: Option<i32> = (0..size).find(|slot| world.tileSlot(chest, *slot).is_none());

    let (Some(goods), Some(empty)) = (goods, empty)
    else
    {
        return false;
    };

    crate::core::stacks::set(world, chest, empty, world.tileSlot(pos, trade::PAID));
    crate::core::stacks::set(world, chest, goods, None);
    crate::core::stacks::set(world, pos, trade::PAID, None);
    crate::core::stacks::set(world, pos, trade::GOODS, offer);
    true
}

fn tradeTick(world: World, pos: BlockPos, owned: &mut Owned)
{
    owned.m_ticker += 1;
    let Some(chest) = tradeChest(world, pos, &owned.m_owner)
    else
    {
        return;
    };

    let want: Option<ItemStack> = world.tileSlot(pos, trade::WANT);
    if want.is_none() || world.tileSlot(pos, trade::OFFER).is_none()
    {
        owned.m_trades = 0;
        return;
    }

    let ready: bool = world.tileSlot(pos, trade::GOODS).is_none() && same(&want, &world.tileSlot(pos, trade::PAID));
    if ready && attemptTrade(world, pos, chest)
    {
        owned.m_trades += 1;
        world.playSound(dir::center(pos), &sound("machineTradeOMat"), 1.0, 1.0);
    }
}

struct TradeTile;

#[tile_behavior]
impl TileBehavior for TradeTile
{
    fn onTick(&self, ctx: &mut BlockCtx)
    {
        if !ctx.isAuthority()
        {
            return;
        }

        let before: Owned = ctx.tileData();
        let mut owned: Owned = before.clone();
        tradeTick(ctx.world(), ctx.pos(), &mut owned);

        if owned != before
        {
            let _ = ctx.setTileData(&owned);
        }
    }
}

struct SafeTile;

#[tile_behavior]
impl TileBehavior for SafeTile {}

// BlockPersonal.blockActivated: the owner opens the safe or configures the Trade-O-Mat, others only trade
pub(crate) fn activate(ctx: &mut BlockCtx, player: Player, block: &str) -> bool
{
    let world: World = ctx.world();
    let pos: BlockPos = ctx.pos();
    if !ctx.isAuthority()
    {
        return true;
    }

    let owner: bool = canAccess(world, pos, player);
    let screen: &str = match (block, owner)
    {
        ("trade_o_mat", true) => "trade_o_mat",
        ("trade_o_mat", false) => "trade_o_mat_closed",
        (_, true) => "personal_safe",
        _ => return false,
    };

    if let Err(error) = player.openScreen(screen, Some(pos), Vec::new())
    {
        warn(&format!("could not open the {screen} screen: {error}"));
    }

    true
}

// GuiChest for the safe, GuiTradeOMatOpen and GuiTradeOMatClosed
struct PersonalScreen
{
    m_screen: &'static str,
    m_pos: Option<BlockPos>,
}

mod layout
{
    pub const TITLE_X: i32 = 56;
    pub const SAFE_TITLE: [i32; 2] = [8, 6];
    pub const PERFORMED: [i32; 2] = [112, 20];
    pub const TRADES: [i32; 2] = [112, 28];
    pub const COUNT: [i32; 2] = [112, 36];
    pub const WANT: [i32; 2] = [12, 20];
    pub const WANT_VALUE: [i32; 2] = [12, 28];
    pub const OFFER: [i32; 2] = [12, 44];
    pub const OFFER_VALUE: [i32; 2] = [12, 56];
}

fn describe(stack: Option<ItemStack>) -> String
{
    stack.map_or(String::new(), |stack| format!("{} {}", stack.count, displayName(&stack)))
}

impl Screen for PersonalScreen
{
    fn init(&mut self, _width: i32, _height: i32, pos: Option<BlockPos>, _data: &[u8]) { self.m_pos = pos; }

    fn draw(&mut self, draw: &mut Draw, _mouse: [i32; 2], _partial: f32)
    {
        if self.m_screen == "personal_safe"
        {
            screen::text(draw, layout::SAFE_TITLE[0], layout::SAFE_TITLE[1], &screen::name("personal_safe"));
            return;
        }

        screen::labels(draw, &screen::name("trade_o_mat"), layout::TITLE_X);
        let owned: Owned = screen::state(self.m_pos).unwrap_or_default();

        if self.m_screen == "trade_o_mat"
        {
            screen::text(draw, layout::PERFORMED[0], layout::PERFORMED[1], "Performed");
            screen::text(draw, layout::TRADES[0], layout::TRADES[1], "Trades:");
            screen::text(draw, layout::COUNT[0], layout::COUNT[1], &owned.m_trades.to_string());
            return;
        }

        let Some((pos, world)) =
            self.m_pos.zip(localPlayer().and_then(|player| player.info()).map(|info| World { dimension: info.world }))
        else
        {
            return;
        };
        let offering: bool = world.tileSlot(pos, trade::WANT).is_some() && world.tileSlot(pos, trade::OFFER).is_some();
        screen::text(draw, layout::WANT[0], layout::WANT[1], "Want:");
        screen::text(draw, layout::OFFER[0], layout::OFFER[1], "Offer:");
        if offering
        {
            screen::text(
                draw,
                layout::WANT_VALUE[0],
                layout::WANT_VALUE[1],
                &describe(world.tileSlot(pos, trade::WANT)),
            );
            screen::text(
                draw,
                layout::OFFER_VALUE[0],
                layout::OFFER_VALUE[1],
                &describe(world.tileSlot(pos, trade::OFFER)),
            );
        }
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    // ContainerTradeOMatOpen and ContainerTradeOMatClosed slot positions
    const OPEN: [(i32, [i32; 2]); 4] =
        [(trade::WANT, [24, 17]), (trade::OFFER, [24, 53]), (trade::PAID, [80, 17]), (trade::GOODS, [80, 53])];
    const CLOSED: [(i32, [i32; 2]); 2] = [(trade::PAID, [143, 17]), (trade::GOODS, [143, 53])];

    registerBlock(ctx, &MACHINES[0], Rc::new(SafeTile), true)?;
    registerBlock(ctx, &MACHINES[1], Rc::new(TradeTile), true)?;

    let mut chest: ScreenBuilder = ScreenBuilder::new("", screen::WIDTH, safe::HEIGHT)
        .background("minecraft:gui/container")
        .playerInventory(screen::INVENTORY[0], safe::INVENTORY_Y);
    for slot in 0..safe::SLOTS
    {
        chest = chest.slot(
            slot,
            safe::LEFT + slot % safe::COLUMNS * safe::SLOT,
            safe::TOP + slot / safe::COLUMNS * safe::SLOT,
        );
    }

    ctx.screens().register("personal_safe", chest, PersonalScreen { m_screen: "personal_safe", m_pos: None })?;

    for (name, sheet, slots) in
        [("trade_o_mat", "tradeomatopen", &OPEN[..]), ("trade_o_mat_closed", "tradeomatclosed", &CLOSED[..])]
    {
        let mut builder: ScreenBuilder = screen::container(sheet);
        for (slot, [x, y]) in slots
        {
            builder = builder.slot(*slot, *x, *y);
        }

        ctx.screens().register(name, builder, PersonalScreen { m_screen: name, m_pos: None })?;
    }

    Ok(())
}
