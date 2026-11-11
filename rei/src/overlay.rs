use crate::catalog::{self, Catalog, Entry, Key};
use crate::display::{self, Display, Tab};
use crate::{cheat, config, spawner};
use b173_api::prelude::*;
use std::cell::{Ref, RefCell};
use std::rc::Rc;

mod pages;

// container screens that get the item list, as REI shows it beside every handled screen
const TARGETS: [&str; 5] = ["GuiInventory", "GuiCrafting", "GuiFurnace", "GuiChest", "GuiDispenser"];

mod layout
{
    pub const CELL: i32 = 18;
    pub const ICON: i32 = 16;
    pub const MARGIN: i32 = 4;
    pub const GAP: i32 = 6;
    pub const SPACING: i32 = 2;
    // vanilla buttons are 20 high
    pub const BAR: i32 = 20;
    pub const ARROW: i32 = 20;
    pub const FIELD: i32 = 20;
    pub const TOGGLE: i32 = 20;
    pub const TEXT_INSET: i32 = 4;
    pub const FONT_HEIGHT: i32 = 8;
    pub const LINE: i32 = 10;
    pub const TOOLTIP_PAD: i32 = 3;
    pub const TOOLTIP_OFFSET: i32 = 12;
    pub const TOGGLE_ICON: i32 = 2;
    pub const FIELD_EDGE: i32 = 1;
}

mod color
{
    pub const WHITE: u32 = 0xFFFFFF;
    pub const HOVER: u32 = 0x80FFFFFF;
    // GuiButton label colors, give mode in the red REI uses for cheating
    pub const BUTTON_HOVER: u32 = 0xFFFFA0;
    pub const GIVE_MODE: u32 = 0xFF5555;
    pub const CRAFTABLE_TINT: u32 = 0x4000FF00;
    pub const FIELD_EDGE: u32 = 0xFFA0A0A0;
    pub const FIELD_FOCUS: u32 = 0xFFFFFFFF;
    pub const FIELD_FACE: u32 = 0xFF000000;
    pub const TEXT: u32 = 0xE0E0E0;
    pub const HINT: u32 = 0x707070;
    pub const TOOLTIP: u32 = 0xC0000000;
    pub const TITLE: u32 = 0x404040;
    pub const RECIPE_ID: u32 = 0x707070;
    // the dark gray vanilla advanced tooltips use
    pub const DETAIL: u32 = 0x555555;
    // the light gray of a vanilla tooltip's extra lines
    pub const NOTE: u32 = 0xAAAAAA;
    // the face Draw::panel fills a gui with
    pub const PANEL_FACE: u32 = 0xFFC6C6C6;
}

// lwjgl key codes
mod key
{
    pub const ESCAPE: i32 = 1;
    pub const BACK: i32 = 14;
    pub const RECIPE: i32 = 19;
    pub const USAGE: i32 = 22;
    pub const HIDE: i32 = 24;
    pub const RETURN: i32 = 28;
    pub const LEFT: i32 = 203;
    pub const RIGHT: i32 = 205;
}

mod text
{
    pub const SEARCH: &str = "rei.text.rei.search.field.suggestion";
    pub const BACK: &str = "rei.text.rei.back";
    pub const LEFT: &str = "rei.text.rei.left_arrow";
    pub const RIGHT: &str = "rei.text.rei.right_arrow";
    pub const CRAFTABLE: &str = "Craftable Only";
}

// counted in client ticks so the pace never follows the frame rate
mod timing
{
    // ticks between inventory reads for the craftable filter
    pub const CRAFTABLE_REFRESH: u32 = 10;
    // ticks each damage of a wildcard ingredient shows
    pub const CYCLE: u32 = 20;
    // GuiTextField flips its cursor every 6 ticks
    pub const BLINK: u32 = 6;
}

// the recipe page drawn over the container, positions from its corner as the vanilla crafting and furnace screens
mod page
{
    pub const INSET: i32 = 8;
    pub const TITLE_TOP: i32 = 6;
    pub const GRID: usize = 3;
    pub const GRID_LEFT: i32 = 30;
    pub const ARROW_LEFT: i32 = 90;
    pub const OUTPUT_LEFT: i32 = 124;
    pub const OUTPUT_FRAME: i32 = 4;
    pub const SMELT_INPUT: i32 = 56;
    pub const SMELT_ARROW: i32 = 79;
    pub const SMELT_OUTPUT: i32 = 116;
    pub const FLAME_TOP: i32 = 20;
    pub const BUTTON_WIDTH: i32 = 20;
    pub const BACK_WIDTH: i32 = 50;
    pub const BUTTON_HEIGHT: i32 = 20;
    pub const BUTTON_INSET: i32 = 8;
    pub const ID_GAP: i32 = 2;
    pub const ARROW_TEXTURE: &str = "/gui/furnace.png";
    pub const ARROW_UV: [i32; 4] = [79, 34, 24, 17];
    pub const FLAME_UV: [i32; 4] = [176, 0, 14, 14];
    pub const BODY_ROWS: i32 = 5;
}

// the category tabs standing in a column left of the recipe page
mod tab
{
    pub const WIDTH: i32 = 28;
    pub const HEIGHT: i32 = 24;
    pub const PITCH: i32 = 26;
    pub const TOP: i32 = 4;
    // how far a tab reaches under the page so the chosen one joins it
    pub const OVERLAP: i32 = 4;
    pub const ICON_INSET: i32 = 4;
    // the light rim and dark edge of Draw::panel
    pub const RIM: i32 = 2;
}

const LEFT_BUTTON: i32 = 0;
const RIGHT_BUTTON: i32 = 1;
const INVENTORY_SLOTS: i32 = 36;
const CRAFTABLE_ICON: &str = "minecraft:crafting_table";

type Rect = [i32; 4];

fn inside(point: [i32; 2], rect: Rect) -> bool
{
    point[0] >= rect[0] && point[1] >= rect[1] && point[0] < rect[0] + rect[2] && point[1] < rect[1] + rect[3]
}

// every rectangle of one frame, relative to the container's corner, kept for the clicks that follow
#[derive(Default, Clone, Copy)]
struct Layout
{
    m_screen: Rect,
    m_container: Rect,
    m_grid: Rect,
    m_columns: i32,
    m_rows: i32,
    m_previous: Rect,
    m_next: Rect,
    m_search: Rect,
    m_craftable: Rect,
    m_cheat: Rect,
}

impl Layout
{
    fn of(container: [i32; 2]) -> Self
    {
        let [width, height] = screenSize().unwrap_or(container);
        let left: i32 = (width - container[0]) / 2;
        let top: i32 = (height - container[1]) / 2;
        let panelRight: i32 = width - left - layout::MARGIN;
        let barTop: i32 = layout::MARGIN - top;
        let rowTop: i32 = height - top - layout::MARGIN - layout::FIELD;
        let columns: i32 = ((panelRight - container[0] - layout::GAP) / layout::CELL).max(0);
        let gridWidth: i32 = columns * layout::CELL;
        let searchWidth: i32 = (gridWidth - (layout::TOGGLE + layout::SPACING) * 2).max(0);
        let room: i32 = rowTop - layout::MARGIN - barTop - layout::BAR - layout::MARGIN;
        let rows: i32 = (room / layout::CELL).max(0);
        // the bars hug the screen edges so the leftover space of a partial cell goes beside and around the list
        let gridTop: i32 = barTop + layout::BAR + layout::MARGIN + (room - rows * layout::CELL) / 2;
        let grid: Rect = [panelRight - gridWidth, gridTop, gridWidth, rows * layout::CELL];
        let gridRight: i32 = grid[0] + grid[2];
        let craftableLeft: i32 = grid[0] + searchWidth + layout::SPACING;
        let arrow: i32 = layout::ARROW.min(gridWidth / 2);

        Self
        {
            m_screen: [-left, -top, width, height],
            m_container: [0, 0, container[0], container[1]],
            m_grid: grid,
            m_columns: columns,
            m_rows: rows,
            m_previous: [grid[0], barTop, arrow, layout::BAR],
            m_next: [gridRight - arrow, barTop, arrow, layout::BAR],
            m_search: [grid[0], rowTop, searchWidth, layout::FIELD],
            m_craftable: [craftableLeft, rowTop, layout::TOGGLE, layout::TOGGLE],
            m_cheat: [gridRight - layout::TOGGLE, rowTop, layout::TOGGLE, layout::TOGGLE],
        }
    }

    fn perPage(&self) -> usize { (self.m_columns * self.m_rows).max(0) as usize }

    fn cellAt(&self, point: [i32; 2]) -> usize
    {
        let column: i32 = (point[0] - self.m_grid[0]) / layout::CELL;
        let row: i32 = (point[1] - self.m_grid[1]) / layout::CELL;
        (row * self.m_columns + column) as usize
    }

    fn cell(&self, index: usize) -> [i32; 2]
    {
        let column: i32 = index as i32 % self.m_columns;
        let row: i32 = index as i32 / self.m_columns;
        [self.m_grid[0] + column * layout::CELL + 1, self.m_grid[1] + row * layout::CELL + 1]
    }
}

fn pageButtons(container: Rect) -> [Rect; 3]
{
    let y: i32 = container[3] - page::BUTTON_HEIGHT - page::BUTTON_INSET;
    let (width, height): (i32, i32) = (page::BUTTON_WIDTH, page::BUTTON_HEIGHT);
    [
        [page::BUTTON_INSET, y, width, height],
        [container[2] - width - page::BUTTON_INSET, y, width, height],
        [(container[2] - page::BACK_WIDTH) / 2, y, page::BACK_WIDTH, height],
    ]
}

fn tabRect(index: usize) -> Rect
{
    [tab::OVERLAP - tab::WIDTH, tab::TOP + index as i32 * tab::PITCH, tab::WIDTH, tab::HEIGHT]
}

// the pages of one item, how to get it or what it is used for, in category tabs
#[derive(Clone)]
struct View
{
    m_focus: Key,
    m_tabs: Vec<Tab>,
    m_tab: usize,
    m_index: usize,
}

impl View
{
    fn displays(&self) -> &[Display] { &self.m_tabs[self.m_tab].m_displays }

    fn display(&self) -> &Display { &self.displays()[self.m_index] }

    fn tabAt(&self, point: [i32; 2]) -> Option<usize>
    {
        (0..self.m_tabs.len()).find(|index| inside(point, tabRect(*index)))
    }
}

#[derive(Default)]
struct State
{
    m_hidden: bool,
    m_query: String,
    m_focused: bool,
    m_page: usize,
    m_craftable: bool,
    m_view: Option<View>,
    m_history: Vec<View>,
    m_visible: Vec<Key>,
    m_dirty: bool,
    m_ticks: u32,
    m_layout: Layout,
    m_hovered: Option<Key>,
    m_mouse: [i32; 2],
    m_slots: Vec<(Rect, Key)>,
    m_generation: i32,
}

thread_local!
{
    // one state for every container screen, so search and page survive switching screens
    static STATE: RefCell<State> = RefCell::new(State { m_dirty: true, ..State::default() });
}

fn heldItems() -> Vec<(Key, i32)>
{
    let Some(player) = localPlayer()
    else
    {
        return Vec::new();
    };

    (0..INVENTORY_SLOTS)
        .filter_map(|slot| player.slot(slot))
        .filter_map(|stack| Key::ofStack(&stack).map(|key| (key, stack.count)))
        .collect()
}

fn holding() -> bool { localPlayer().and_then(|player| player.cursor()).is_some() }

// a damaged tool or other unlisted damage is looked up as its plain item
fn listedKey(catalog: &Catalog, key: Key) -> Key
{
    if catalog.entry(key).is_some()
    {
        key
    }
    else
    {
        Key { m_id: key.m_id, m_damage: 0 }
    }
}

fn cycle(index: usize, count: usize, forward: bool) -> usize
{
    if forward
    {
        (index + 1) % count
    }
    else
    {
        (index + count - 1) % count
    }
}

// drops characters from one end until the text fits the width
fn clip(text: &str, room: i32, dropFront: bool) -> &str
{
    let mut shown: &str = text;
    while textWidth(shown) > room
    {
        let mut chars: std::str::Chars = shown.chars();

        if dropFront
        {
            chars.next();
        }
        else
        {
            chars.next_back();
        }
        shown = chars.as_str();
    }

    shown
}

impl State
{
    // recipe indices and listed keys belong to one catalog, a rebuilt one closes the page and filters again
    fn catalog(&mut self) -> Rc<Catalog>
    {
        let catalog: Rc<Catalog> = catalog::get();
        if catalog.m_generation != self.m_generation
        {
            self.m_generation = catalog.m_generation;
            self.close();
            self.m_dirty = true;
        }

        catalog
    }

    fn refilter(&mut self, catalog: &Catalog)
    {
        let matches: Vec<Key> =
            catalog.search(&self.m_query).into_iter().map(|index| catalog.m_entries[index].m_key).collect();

        self.m_visible = match self.m_craftable
        {
            true =>
            {
                let craftable: Vec<Key> = catalog.craftable(&heldItems());
                matches
                    .into_iter()
                    .filter(|key|
                    {
                        craftable
                            .binary_search_by_key(&(key.m_id, key.m_damage), |out| (out.m_id, out.m_damage))
                            .is_ok()
                    })
                    .collect()
            }

            false => matches,
        };

        self.m_dirty = false;
    }

    fn pages(&self) -> usize { self.m_visible.len().div_ceil(self.m_layout.perPage().max(1)).max(1) }

    // a lookup from a recipe page can be backed out of, one from the list or a vanilla slot starts over
    fn open(&mut self, catalog: &Catalog, key: Key, usages: bool, drill: bool)
    {
        let key: Key = listedKey(catalog, key);
        let tabs: Vec<Tab> = display::tabs(catalog, key, usages);

        if tabs.is_empty()
        {
            return;
        }

        match self.m_view.take()
        {
            Some(current) if drill => self.m_history.push(current),
            _ => self.m_history.clear(),
        }

        self.m_view = Some(View { m_focus: key, m_tabs: tabs, m_tab: 0, m_index: 0 });
    }

    fn back(&mut self) { self.m_view = self.m_history.pop(); }

    // the recipe page is up and covers the container
    fn paging(&self) -> bool { !self.m_hidden && self.m_view.is_some() }

    // every screen change starts on the plain container, so reopening the inventory never lands on an old recipe
    fn leave(&mut self)
    {
        self.close();
        self.m_focused = false;
    }

    fn tick(&mut self)
    {
        self.m_ticks = self.m_ticks.wrapping_add(1);
        if self.m_craftable && self.m_ticks.is_multiple_of(timing::CRAFTABLE_REFRESH)
        {
            self.m_dirty = true;
        }
    }

    fn close(&mut self)
    {
        self.m_view = None;
        self.m_history.clear();
    }

    fn turnRecipe(&mut self, forward: bool)
    {
        if let Some(view) = &mut self.m_view
        {
            view.m_index = cycle(view.m_index, view.displays().len(), forward);
        }
    }

    fn turnPage(&mut self, forward: bool) { self.m_page = cycle(self.m_page, self.pages(), forward); }

    fn turn(&mut self, forward: bool)
    {
        if self.m_view.is_some()
        {
            self.turnRecipe(forward);
        }
        else
        {
            self.turnPage(forward);
        }
    }

    // the focused search field takes every key, so typing never closes the screen
    fn typeKey(&mut self, character: char, code: i32) -> bool
    {
        match code
        {
            key::ESCAPE | key::RETURN => self.m_focused = false,
            key::BACK => self.edit(|query|
            {
                query.pop();
            }),
            _ if !character.is_control() && character != '\0' => self.edit(|query| query.push(character)),
            _ =>
            {}
        }

        true
    }

    fn edit(&mut self, change: impl FnOnce(&mut String))
    {
        change(&mut self.m_query);
        self.m_dirty = true;
        self.m_page = 0;
    }

    fn key(&mut self, character: char, code: i32, hovered: Option<Key>) -> bool
    {
        if self.m_focused
        {
            return self.typeKey(character, code);
        }

        match code
        {
            key::HIDE =>
            {
                self.m_hidden = !self.m_hidden;
                true
            }

            _ if self.m_hidden => false,

            key::ESCAPE if self.m_view.is_some() =>
            {
                self.close();
                true
            }

            key::BACK if self.m_view.is_some() =>
            {
                self.back();
                true
            }

            key::LEFT | key::RIGHT =>
            {
                self.turn(code == key::RIGHT);
                true
            }

            key::RECIPE | key::USAGE => self.lookup(code == key::USAGE, hovered),
            _ => false,
        }
    }

    fn lookup(&mut self, usages: bool, hovered: Option<Key>) -> bool
    {
        // the recipe page covers the vanilla slots under it
        let onPage: bool = self.paging() && inside(self.m_mouse, self.m_layout.m_container);
        let target: Option<Key> = if onPage
        {
            self.m_hovered
        }
        else
        {
            self.m_hovered.or(hovered)
        };

        let Some(target) = target
        else
        {
            return false;
        };

        let catalog: Rc<Catalog> = self.catalog();
        self.open(&catalog, target, usages, onPage);
        true
    }

    // cheating gives a full stack on left click and one on right click
    fn clickItem(&mut self, catalog: &Catalog, key: Key, button: i32)
    {
        if !cheat::enabled()
        {
            self.open(catalog, key, button == RIGHT_BUTTON, false);
            return;
        }

        let max: i32 = catalog.entry(key).map_or(1, |entry| entry.m_max);
        cheat::grab(key, if button == RIGHT_BUTTON { 1 } else { max });
    }

    fn clickGrid(&mut self, catalog: &Catalog, point: [i32; 2], button: i32)
    {
        let index: usize = self.m_page * self.m_layout.perPage() + self.m_layout.cellAt(point);
        match self.m_visible.get(index).copied()
        {
            Some(key) => self.clickItem(catalog, key, button),
            None if cheat::enabled() && holding() => cheat::trash(),
            None =>
            {}
        }
    }

    // the recipe page is modal, it takes every click on the container
    fn clickView(&mut self, catalog: &Catalog, point: [i32; 2], button: i32) -> bool
    {
        let Some(view) = &self.m_view
        else
        {
            return false;
        };

        let container: Rect = self.m_layout.m_container;
        let [previous, next, back] = pageButtons(container);
        let focus: Key = view.m_focus;
        let paged: bool = view.displays().len() > 1;
        let tab: Option<usize> = view.tabAt(point);

        if let (Some(tab), Some(view)) = (tab, &mut self.m_view)
        {
            view.m_tab = tab;
            view.m_index = 0;
            return true;
        }

        if paged && (inside(point, previous) || inside(point, next))
        {
            self.turnRecipe(inside(point, next));
        }
        else if inside(point, back)
        {
            self.back();
        }
        else if let Some((_, key)) = self.m_slots.iter().find(|(rect, _)| inside(point, *rect)).copied()
            && (key != focus || button == RIGHT_BUTTON)
        {
            self.open(catalog, key, button == RIGHT_BUTTON, true);
        }
        inside(point, container)
    }

    fn mouse(&mut self, point: [i32; 2], button: i32) -> bool
    {
        if self.m_hidden || (button != LEFT_BUTTON && button != RIGHT_BUTTON)
        {
            return false;
        }

        let layout: Layout = self.m_layout;
        let onSearch: bool = inside(point, layout.m_search);
        self.m_focused = onSearch && button == LEFT_BUTTON;

        let catalog: Rc<Catalog> = self.catalog();

        if onSearch
        {
            if button == RIGHT_BUTTON
            {
                self.edit(String::clear);
            }
        }
        else if inside(point, layout.m_cheat)
        {
            cheat::toggle();
        }
        else if inside(point, layout.m_craftable)
        {
            self.m_craftable = !self.m_craftable;
            self.edit(|_| {});
        }
        else if inside(point, layout.m_previous) || inside(point, layout.m_next)
        {
            self.turnPage(inside(point, layout.m_next));
        }
        else if inside(point, layout.m_grid)
        {
            self.clickGrid(&catalog, point, button);
        }
        else
        {
            return self.clickView(&catalog, point, button);
        }

        true
    }

    fn scroll(&mut self, point: [i32; 2], delta: i32) -> bool
    {
        if self.m_hidden
        {
            return false;
        }

        let forward: bool = delta < 0;

        if self.paging() && inside(point, self.m_layout.m_container)
        {
            self.turnRecipe(forward);
            return true;
        }

        if inside(point, self.m_layout.m_grid)
        {
            self.turnPage(forward);
            return true;
        }

        false
    }
}

struct Painter<'a>
{
    m_draw: &'a mut Draw,
    m_catalog: &'a Catalog,
    m_mouse: [i32; 2],
    m_ticks: u32,
    m_hovered: Option<Key>,
    m_hint: Option<String>,
    // extra tooltip lines for the hovered stack, as a drop's count and chance
    m_notes: Vec<String>,
    m_slots: Vec<(Rect, Key)>,
    m_detailed: bool,
}

impl Painter<'_>
{
    fn button(&mut self, rect: Rect, label: &str)
    {
        let [x, y, w, h] = rect;
        self.m_draw.button(0, x, y, w, h, label);
    }

    fn texture(&mut self, x: i32, y: i32, uv: [i32; 4])
    {
        let [u, v, w, h] = uv;
        self.m_draw.texture(page::ARROW_TEXTURE, x, y, u, v, w, h);
    }

    fn slot(&mut self, x: i32, y: i32, size: i32) { self.m_draw.slot(x, y, size); }

    // a wildcard ingredient cycles through every listed damage of its item, the host draws the count as vanilla slots do
    fn icon(&mut self, x: i32, y: i32, key: Key, count: i32) -> Key
    {
        let shown: Key = self.m_catalog.variant(key, (self.m_ticks / timing::CYCLE) as usize);
        self.m_draw.item(x, y, &shown.stack(count));

        if inside(self.m_mouse, [x, y, layout::ICON, layout::ICON])
        {
            self.m_draw.rect(x, y, layout::ICON, layout::ICON, color::HOVER);
            self.m_hovered = Some(shown);
        }

        shown
    }

    // recipe page stacks answer clicks, the list maps clicks through its grid instead
    fn stack(&mut self, x: i32, y: i32, key: Key, count: i32)
    {
        let shown: Key = self.icon(x, y, key, count);
        self.m_slots.push(([x, y, layout::ICON, layout::ICON], shown));
    }

    fn item(&mut self, x: i32, y: i32, key: Key) { self.stack(x, y, key, 1); }

    fn slotItem(&mut self, x: i32, y: i32, key: Option<Key>)
    {
        self.slot(x, y, layout::ICON);

        if let Some(key) = key
        {
            self.item(x, y, key);
        }
    }

    fn recipe(&mut self, recipe: &RecipeView, top: i32)
    {
        let output: Key = Key::of(&recipe.output);
        let count: i32 = recipe.output.count;

        match &recipe.shape
        {
            RecipeShape::Smelting { input } =>
            {
                self.slotItem(page::SMELT_INPUT, top, Some(Key::of(input)));
                self.texture(page::SMELT_INPUT, top + page::FLAME_TOP, page::FLAME_UV);
                self.texture(page::SMELT_ARROW, top + layout::CELL, page::ARROW_UV);
                self.output(page::SMELT_OUTPUT, top + layout::CELL, output, count);
            }

            RecipeShape::Shaped { width, inputs } =>
            {
                let keys: Vec<Option<Key>> = inputs.iter().map(|input| input.as_ref().map(Key::of)).collect();
                self.grid(top, *width as usize, &keys, output, count);
            }

            RecipeShape::Shapeless { inputs } =>
            {
                let keys: Vec<Option<Key>> = inputs.iter().map(|input| Some(Key::of(input))).collect();
                self.grid(top, page::GRID, &keys, output, count);
            }
        }
    }

    fn grid(&mut self, top: i32, width: usize, inputs: &[Option<Key>], output: Key, count: i32)
    {
        for row in 0..page::GRID
        {
            for column in 0..page::GRID
            {
                let input: Option<Key> = if column < width
                {
                    inputs.get(row * width + column).copied().flatten()
                }
                else
                {
                    None
                };
                let x: i32 = page::GRID_LEFT + column as i32 * layout::CELL;
                self.slotItem(x, top + row as i32 * layout::CELL, input);
            }
        }

        self.texture(page::ARROW_LEFT, top + layout::CELL, page::ARROW_UV);
        self.output(page::OUTPUT_LEFT, top + layout::CELL, output, count);
    }

    fn output(&mut self, x: i32, y: i32, output: Key, count: i32)
    {
        let frame: i32 = page::OUTPUT_FRAME;
        self.slot(x - frame, y - frame, layout::ICON + frame * 2);
        self.stack(x, y, output, count);
    }

    // the name, for mod items the owning mod as REI shows it, and on a recipe page the registry id and tags
    fn tooltip(&mut self, screen: Rect)
    {
        let hinted: bool = self.m_hint.is_some();
        let mut lines: Vec<(String, u32)> = match (self.m_hint.take(), self.m_hovered)
        {
            (Some(hint), _) => vec![(hint, color::WHITE)],
            (None, Some(key)) => vec![(self.m_catalog.name(key), color::WHITE)],
            (None, None) => return,
        };

        // a hint is the whole tooltip
        let item: bool = !hinted;

        if let Some(origin) = self.m_hovered.filter(|_| item).and_then(|key| origin(self.m_catalog, key))
        {
            lines.push((origin.to_owned(), color::WHITE));
        }

        if item
        {
            lines.extend(self.m_notes.drain(..).map(|note| (note, color::NOTE)));
        }

        if let Some(key) = self.m_hovered.filter(|_| self.m_detailed && item)
        {
            lines.extend(self.m_catalog.details(key.m_id).iter().map(|line| (line.clone(), color::DETAIL)));
        }

        let width: i32 = lines.iter().map(|(line, _)| textWidth(line)).max().unwrap_or(0);
        let height: i32 = (lines.len() as i32 - 1) * layout::LINE + layout::FONT_HEIGHT;
        let pad: i32 = layout::TOOLTIP_PAD;
        let [mouseX, mouseY] = self.m_mouse;
        let right: bool = mouseX + layout::TOOLTIP_OFFSET + width + pad <= screen[0] + screen[2];
        let x: i32 = if right
        {
            mouseX + layout::TOOLTIP_OFFSET
        }
        else
        {
            mouseX - layout::TOOLTIP_OFFSET - width
        };
        let y: i32 = (mouseY - layout::TOOLTIP_OFFSET).max(screen[1] + pad);
        self.m_draw.rect(x - pad, y - pad, width + pad * 2, height + pad * 2, color::TOOLTIP);

        for (index, (line, tint)) in lines.iter().enumerate()
        {
            self.m_draw.text(x, y + index as i32 * layout::LINE, line, *tint);
        }
    }

    fn search(&mut self, state: &State, rect: Rect)
    {
        let [x, y, w, h] = rect;
        let edge: i32 = layout::FIELD_EDGE;

        let border: u32 = if state.m_focused
        {
            color::FIELD_FOCUS
        }
        else
        {
            color::FIELD_EDGE
        };
        self.m_draw.rect(x, y, w, h, border);
        self.m_draw.rect(x + edge, y + edge, w - edge * 2, h - edge * 2, color::FIELD_FACE);

        let room: i32 = w - layout::TEXT_INSET * 2;
        let shown: &str = clip(&state.m_query, room, true);
        let textX: i32 = x + layout::TEXT_INSET;
        let textY: i32 = y + (h - layout::FONT_HEIGHT) / 2;

        if shown.is_empty() && !state.m_focused
        {
            let hint: String = translate(text::SEARCH);
            self.m_draw.text(textX, textY, clip(&hint, room, false), color::HINT);
            return;
        }

        let blink: bool = state.m_focused && (state.m_ticks / timing::BLINK).is_multiple_of(2);
        self.m_draw.text(textX, textY, &format!("{shown}{}", if blink { "_" } else { "" }), color::TEXT);
    }

    fn list(&mut self, state: &State, layout: &Layout)
    {
        let perPage: usize = layout.perPage();
        for (offset, key) in state.m_visible.iter().skip(state.m_page * perPage).take(perPage).enumerate()
        {
            let [x, y] = layout.cell(offset);
            self.icon(x, y, *key, 1);
        }

        self.button(layout.m_previous, &translate(text::LEFT));
        self.button(layout.m_next, &translate(text::RIGHT));

        let current: usize = state.m_page + 1;
        let room: i32 = layout.m_next[0] - layout.m_previous[0] - layout.m_previous[2] - layout::SPACING * 2;
        // a narrow bar drops the page count first and the label entirely when even the page number overflows
        let Some(label) = [format!("{current}/{}", state.pages()), current.to_string()]
            .into_iter()
            .find(|label| textWidth(label) <= room)
        else
        {
            return;
        };

        let middle: i32 = (layout.m_previous[0] + layout.m_next[0] + layout.m_next[2]) / 2;
        let textTop: i32 = (layout::BAR - layout::FONT_HEIGHT) / 2;
        self.m_draw.centered(middle, layout.m_previous[1] + textTop, &label, color::WHITE);
    }

    fn toggles(&mut self, state: &State, layout: &Layout)
    {
        self.button(layout.m_cheat, "");

        let [x, y, w, _] = layout.m_cheat;
        let overCheat: bool = inside(self.m_mouse, layout.m_cheat);
        let tint: u32 = if overCheat
        {
            color::BUTTON_HOVER
        }
        else if cheat::enabled()
        {
            color::GIVE_MODE
        }
        else
        {
            color::TEXT
        };
        let textTop: i32 = (layout::BAR - layout::FONT_HEIGHT) / 2;
        self.m_draw.centered(x + w / 2, y + textTop, cheat::button(), tint);

        if overCheat
        {
            self.m_hint = Some(cheat::mode().to_owned());
        }

        if inside(self.m_mouse, layout.m_craftable)
        {
            self.m_hint = Some(text::CRAFTABLE.to_owned());
        }

        self.button(layout.m_craftable, "");

        let [x, y, w, h] = layout.m_craftable;
        // the button texture keeps a two pixel rim inside which the tint and icon sit
        let rim: i32 = layout::TOGGLE_ICON;

        if state.m_craftable
        {
            self.m_draw.rect(x + rim, y + rim, w - rim * 2, h - rim * 2, color::CRAFTABLE_TINT);
        }

        self.m_draw.item(x + rim, y + rim, &ItemStack::of(CRAFTABLE_ICON, 1));
    }

    // a recipe id too wide for the panel drops its namespace and then gets cut short
    fn recipeId(&mut self, id: &str, container: Rect, top: i32)
    {
        let room: i32 = container[2] - page::INSET * 2;
        let short: &str = if textWidth(id) <= room
        {
            id
        }
        else
        {
            id.split_once(':').map_or(id, |(_, path)| path)
        };
        let shown: &str = clip(short, room, false);
        let x: i32 = (container[2] - textWidth(shown)) / 2;
        self.m_draw.plain(x, top, shown, color::RECIPE_ID);
    }

    fn recipePage(&mut self, index: usize, top: i32, bottom: i32) -> (String, Option<String>)
    {
        let catalog: &Catalog = self.m_catalog;
        let recipe: &RecipeView = &catalog.m_recipes[index];
        let body: i32 = page::GRID as i32 * layout::CELL;
        self.recipe(recipe, (top + bottom - body) / 2);
        (catalog.name(Key::of(&recipe.output)), recipe.id.as_ref().map(Identifier::to_string))
    }

    fn view(&mut self, view: &View, container: Rect)
    {
        // the page covers the inner edge of every tab but the chosen one, which opens into it
        for index in (0..view.m_tabs.len()).filter(|index| *index != view.m_tab)
        {
            self.tab(view, index);
        }

        let [x, y, w, h] = container;
        self.m_draw.panel(x, y, w, h);
        self.tab(view, view.m_tab);

        let [previous, next, back] = pageButtons(container);
        let idTop: i32 = back[1] - layout::LINE - page::ID_GAP;
        // the body sits centered between the title and the id line
        let bodyTop: i32 = page::TITLE_TOP + layout::FONT_HEIGHT;

        let (title, id): (String, Option<String>) = match view.display()
        {
            Display::Recipe(index) => self.recipePage(*index, bodyTop, idTop),
            Display::Machine { machine, recipe } => self.machinePage(*machine, *recipe, [bodyTop, idTop], w),
            Display::Loot { table, start } => self.lootPage(*table, *start, bodyTop, idTop),
            Display::Tag { tag, start } => self.tagPage(tag, *start, [bodyTop, idTop], w),
        };

        let count: usize = view.displays().len();
        let paged: bool = count > 1;
        let index: String = if paged
        {
            format!("{}/{count}", view.m_index + 1)
        }
        else
        {
            String::new()
        };
        let indexLeft: i32 = w - page::INSET - textWidth(&index);
        self.m_draw.plain(indexLeft, page::TITLE_TOP, &index, color::TITLE);

        // a long title keeps clear of the page index
        let room: i32 = indexLeft - page::INSET * 2;
        let title: &str = clip(&title, room, false);
        self.m_draw.plain((w - textWidth(title)) / 2, page::TITLE_TOP, title, color::TITLE);

        if let Some(id) = id
        {
            self.recipeId(&id, container, idTop);
        }

        if paged
        {
            self.button(previous, &translate(text::LEFT));
            self.button(next, &translate(text::RIGHT));
        }

        self.button(back, &translate(text::BACK));
    }
}

fn drawFrame(draw: &mut Draw, state: &mut State, size: [i32; 2], mouse: [i32; 2])
{
    state.m_hovered = None;
    state.m_mouse = mouse;
    state.m_slots.clear();

    if state.m_hidden
    {
        return;
    }

    let catalog: Rc<Catalog> = state.catalog();
    let layout: Layout = Layout::of(size);
    state.m_layout = layout;

    if state.m_dirty
    {
        state.refilter(&catalog);
    }

    state.m_page = state.m_page.min(state.pages() - 1);

    let mut painter: Painter = Painter
    {
        m_draw: draw,
        m_catalog: &catalog,
        m_mouse: mouse,
        m_ticks: state.m_ticks,
        m_hovered: None,
        m_hint: None,
        m_notes: Vec::new(),
        m_slots: Vec::new(),
        m_detailed: state.m_view.is_some(),
    };
    painter.toggles(state, &layout);
    painter.search(state, layout.m_search);

    if layout.perPage() > 0
    {
        painter.list(state, &layout);
    }

    if let Some(view) = &state.m_view
    {
        painter.view(view, layout.m_container);
    }

    painter.tooltip(layout.m_screen);
    state.m_hovered = painter.m_hovered;
    state.m_slots = painter.m_slots;
}

// the item list, search field and recipe pages beside a vanilla container screen
struct Viewer;

impl Overlay for Viewer
{
    fn draw(&mut self, draw: &mut Draw, size: [i32; 2], mouse: [i32; 2])
    {
        STATE.with(|state| drawFrame(draw, &mut state.borrow_mut(), size, mouse));
    }

    fn mouse(&mut self, x: i32, y: i32, button: i32) -> bool
    {
        STATE.with(|state| state.borrow_mut().mouse([x, y], button))
    }

    fn key(&mut self, character: char, code: i32, hovered: Option<ItemStack>) -> bool
    {
        let hovered: Option<Key> = hovered.as_ref().and_then(Key::ofStack);
        STATE.with(|state| state.borrow_mut().key(character, code, hovered))
    }

    fn scroll(&mut self, x: i32, y: i32, delta: i32) -> bool
    {
        STATE.with(|state| state.borrow_mut().scroll([x, y], delta))
    }

    // the recipe page hides the vanilla slots under it
    fn covered(&mut self, x: i32, y: i32) -> bool
    {
        STATE.with(|state|
        {
            let state: Ref<State> = state.borrow();
            state.paging() && inside([x, y], state.m_layout.m_container)
        })
    }

    // the recipe page fills the container so other mods' panels beside it hide
    fn exclusive(&mut self) -> bool { STATE.with(|state| state.borrow().paging()) }
}

// the owning mod's line under an item name, when the option is on
fn origin(catalog: &Catalog, key: Key) -> Option<&str>
{
    if !config::modName()
    {
        return None;
    }

    catalog.origin(key.m_id)
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for target in TARGETS
    {
        ctx.screens().overlay(target, Viewer)?;
    }

    ctx.events().on::<ScreenOpen>(Priority::Normal, |_| STATE.with(|state| state.borrow_mut().leave()))?;
    ctx.events().on::<ClientTick>(Priority::Normal, |_| STATE.with(|state| state.borrow_mut().tick()))?;

    ctx.screens().tooltip(|stack, lines|
    {
        let catalog: Rc<Catalog> = catalog::get();
        // beta names every spawner Mob Spawner so the mob it holds takes the name line
        let spawner: Option<&Entry> = Key::ofStack(stack).filter(spawner::held).and_then(|key| catalog.entry(key));
        if let (Some(entry), Some(name)) = (spawner, lines.first_mut())
        {
            name.clone_from(&entry.m_name);
        }

        if let Some(origin) = Key::ofStack(stack).and_then(|key| origin(&catalog, key))
        {
            lines.push(origin.to_owned());
        }
    })
}
