use super::travel::Traveler;
use b173_api::prelude::*;

// TileDiamondPipe: nine filter slots per face, an item goes out of faces whose filters name it, or the unfiltered ones
pub(crate) const SLOTS: i32 = 54;
pub(crate) const SCREEN: &str = "diamond_pipe";
const PER_FACE: i32 = 9;
const FILTER_LIMIT: i32 = 1;

mod gui
{
    // GuiDiamondPipe on filter.png
    pub const WIDTH: i32 = 175;
    pub const HEIGHT: i32 = 225;
    pub const ROWS: i32 = 6;
    pub const COLUMNS: i32 = 9;
    pub const SLOT: i32 = 18;
    pub const FIRST: [i32; 2] = [8, 18];
    pub const INVENTORY: [i32; 2] = [8, 140];
    pub const TITLE: [i32; 2] = [8, 6];
    pub const INVENTORY_LABEL_DROP: i32 = 97;
    pub const TEXT: u32 = 4210752;
    pub const BACKGROUND: &str = "buildcraft:gui/filter";
}

fn matches(filter: &ItemStack, item: &ItemStack) -> bool
{
    if filter.item.id().ok() != item.item.id().ok()
    {
        return false;
    }

    // CoreProxy.isDamageable: tools match whatever their wear
    let damageable: bool = item.info().is_some_and(|info| info.max_damage > 0);
    damageable || filter.damage == item.damage
}

// getPossibleMovements, faces once per filter naming the item, faces with no filter at all otherwise
pub(crate) fn filter(world: World, pos: BlockPos, item: &Traveler, open: Vec<i32>) -> Vec<i32>
{
    let stack: ItemStack = item.m_stack.stack();
    let mut filtered: Vec<i32> = Vec::new();
    let mut unfiltered: Vec<i32> = Vec::new();

    for face in open
    {
        let filters: Vec<ItemStack> =
            (0..PER_FACE).filter_map(|slot| world.tileSlot(pos, face * PER_FACE + slot)).collect();

        if filters.is_empty()
        {
            unfiltered.push(face);
        }

        // each matching filter lists the face again, so more filters weigh the random pick toward it
        filtered.extend(filters.iter().filter(|filter| matches(filter, &stack)).map(|_| face));
    }

    if filtered.is_empty()
    {
        unfiltered
    }
    else
    {
        filtered
    }
}

struct FilterScreen;

impl Screen for FilterScreen
{
    fn draw(&mut self, draw: &mut Draw, _mouse: [i32; 2], _partial: f32)
    {
        draw.text(gui::TITLE[0], gui::TITLE[1], &translate("buildcraft.gui.filters"), gui::TEXT);
        draw.text(
            gui::TITLE[0],
            gui::HEIGHT - gui::INVENTORY_LABEL_DROP,
            &translate("buildcraft.gui.inventory"),
            gui::TEXT,
        );
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.tileEntities().stackLimit(SCREEN, FILTER_LIMIT)?;

    let mut screen: ScreenBuilder = ScreenBuilder::new("", gui::WIDTH, gui::HEIGHT)
        .background(gui::BACKGROUND)
        .playerInventory(gui::INVENTORY[0], gui::INVENTORY[1]);

    for row in 0..gui::ROWS
    {
        for column in 0..gui::COLUMNS
        {
            let [x, y] = gui::FIRST;
            screen = screen.slot(row * gui::COLUMNS + column, x + column * gui::SLOT, y + row * gui::SLOT);
        }
    }

    ctx.screens().register(SCREEN, screen, FilterScreen)
}
