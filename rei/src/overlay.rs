use crate::catalog::{self, Catalog, Key};
use crate::{cheat, config};
use b173_api::prelude::*;
use std::cell::{Ref, RefCell};
use std::rc::Rc;

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
    pub const HOVER: u32 = 0x80FF_FFFF;
    // GuiButton label colors, give mode in the red REI uses for cheating
    pub const BUTTON_HOVER: u32 = 0xFFFFA0;
    pub const GIVE_MODE: u32 = 0xFF5555;
    pub const CRAFTABLE_TINT: u32 = 0x4000_FF00;
    pub const FIELD_EDGE: u32 = 0xFFA0_A0A0;
    pub const FIELD_FOCUS: u32 = 0xFFFF_FFFF;
    pub const FIELD_FACE: u32 = 0xFF00_0000;
    pub const TEXT: u32 = 0xE0E0E0;
    pub const HINT: u32 = 0x707070;
    pub const TOOLTIP: u32 = 0xC000_0000;
    pub const TITLE: u32 = 0x404040;
    pub const RECIPE_ID: u32 = 0x707070;
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

mod timing
{
    // frames between inventory reads for the craftable filter
    pub const CRAFTABLE_REFRESH: u32 = 20;
    // frames each damage of a wildcard ingredient shows
    pub const CYCLE: u32 = 30;
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
    fn of(container: [i32; 2]) -> Layout
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
        Layout
        {
            m_screen: [-left, -top, width, height],
            m_container: [0, 0, container[0], container[1]],
            m_grid: grid,
            m_columns: columns,
            m_rows: rows,
            m_previous: [grid[0], barTop, layout::ARROW, layout::BAR],
            m_next: [gridRight - layout::ARROW, barTop, layout::ARROW, layout::BAR],
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

// the recipe pages of one item, the recipes making it or the recipes using it
#[derive(Clone)]
struct View
{
    m_focus: Key,
    m_recipes: Vec<usize>,
    m_index: usize,
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
    m_frame: u32,
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

fn cursorHolds() -> Option<Option<Key>>
{
    localPlayer().and_then(|player| player.cursor()).map(|held| Key::ofStack(&held))
}

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
        let recipes: Vec<usize> = if usages
        {
            catalog.usagesOf(key)
        }
        else
        {
            catalog.recipesFor(key)
        };
        if recipes.is_empty()
        {
            return;
        }
        match self.m_view.take()
        {
            Some(current) if drill => self.m_history.push(current),
            _ => self.m_history.clear(),
        }
        self.m_view = Some(View { m_focus: key, m_recipes: recipes, m_index: 0 });
    }

    fn back(&mut self) { self.m_view = self.m_history.pop(); }

    // every screen change starts on the plain container, so reopening the inventory never lands on an old recipe
    fn leave(&mut self)
    {
        self.close();
        self.m_focused = false;
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
            view.m_index = cycle(view.m_index, view.m_recipes.len(), forward);
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
        let onPage: bool = self.m_view.is_some() && inside(self.m_mouse, self.m_layout.m_container);
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
            None if cheat::enabled() && cursorHolds().is_some() => cheat::trash(),
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
        let paged: bool = view.m_recipes.len() > 1;
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
        let forward: bool = delta < 0;
        if self.m_hidden
        {
            return false;
        }
        if self.m_view.is_some() && inside(point, self.m_layout.m_container)
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
    m_frame: u32,
    m_hovered: Option<Key>,
    m_hint: Option<&'static str>,
    m_slots: Vec<(Rect, Key)>,
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

    // a wildcard ingredient cycles through every listed damage of its item
    fn item(&mut self, x: i32, y: i32, key: Key) { self.stack(x, y, key, 1); }

    // the host draws the count as vanilla slots do
    fn stack(&mut self, x: i32, y: i32, key: Key, count: i32)
    {
        let variants: Vec<Key> = self.m_catalog.variants(key);
        let shown: Key = variants[(self.m_frame / timing::CYCLE) as usize % variants.len()];
        self.m_draw.item(x, y, &shown.stack(count));
        let rect: Rect = [x, y, layout::ICON, layout::ICON];
        if inside(self.m_mouse, rect)
        {
            self.m_draw.rect(x, y, layout::ICON, layout::ICON, color::HOVER);
            self.m_hovered = Some(shown);
        }
        self.m_slots.push((rect, shown));
    }

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

    // the name, and for mod items the owning mod as REI shows it
    fn tooltip(&mut self, screen: Rect)
    {
        let mut lines: Vec<(String, u32)> = match (self.m_hint, self.m_hovered)
        {
            (Some(hint), _) => vec![(hint.to_owned(), color::WHITE)],
            (None, Some(key)) => vec![(self.m_catalog.name(key), color::WHITE)],
            (None, None) => return,
        };
        if let Some(origin) =
            self.m_hovered.filter(|_| self.m_hint.is_none()).and_then(|key| origin(self.m_catalog, key))
        {
            lines.push((origin.to_owned(), color::WHITE));
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
        let mut shown: &str = &state.m_query;
        while textWidth(shown) > room
        {
            let mut chars: std::str::Chars = shown.chars();
            chars.next();
            shown = chars.as_str();
        }
        let textX: i32 = x + layout::TEXT_INSET;
        let textY: i32 = y + (h - layout::FONT_HEIGHT) / 2;
        if shown.is_empty() && !state.m_focused
        {
            self.m_draw.text(textX, textY, &translate(text::SEARCH), color::HINT);
            return;
        }
        let blink: bool = state.m_focused && (state.m_frame / timing::BLINK).is_multiple_of(2);
        self.m_draw.text(textX, textY, &format!("{shown}{}", if blink { "_" } else { "" }), color::TEXT);
    }

    fn list(&mut self, state: &State, layout: &Layout)
    {
        let perPage: usize = layout.perPage();
        for (offset, key) in state.m_visible.iter().skip(state.m_page * perPage).take(perPage).enumerate()
        {
            let [x, y] = layout.cell(offset);
            self.item(x, y, *key);
        }
        self.button(layout.m_previous, &translate(text::LEFT));
        self.button(layout.m_next, &translate(text::RIGHT));
        let label: String = format!("{}/{}", state.m_page + 1, state.pages());
        let middle: i32 = (layout.m_previous[0] + layout.m_next[0] + layout.m_next[2]) / 2;
        let textTop: i32 = (layout::BAR - layout::FONT_HEIGHT) / 2;
        self.m_draw.centered(middle, layout.m_previous[1] + textTop, &label, color::WHITE);
    }

    fn toggles(&mut self, state: &State, layout: &Layout)
    {
        self.button(layout.m_cheat, "");
        let [x, y, w, _] = layout.m_cheat;
        let tint: u32 = if inside(self.m_mouse, layout.m_cheat)
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
        if inside(self.m_mouse, layout.m_cheat)
        {
            self.m_hint = Some(cheat::mode());
        }
        if inside(self.m_mouse, layout.m_craftable)
        {
            self.m_hint = Some(text::CRAFTABLE);
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
        let mut shown: &str = if textWidth(id) <= room
        {
            id
        }
        else
        {
            id.split_once(':').map_or(id, |(_, path)| path)
        };
        while textWidth(shown) > room
        {
            let mut chars: std::str::Chars = shown.chars();
            chars.next_back();
            shown = chars.as_str();
        }
        let x: i32 = (container[2] - textWidth(shown)) / 2;
        self.m_draw.plain(x, top, shown, color::RECIPE_ID);
    }

    fn view(&mut self, view: &View, container: Rect)
    {
        let [x, y, w, h] = container;
        self.m_draw.panel(x, y, w, h);
        let recipe: &RecipeView = &self.m_catalog.m_recipes[view.m_recipes[view.m_index]];
        let name: String = self.m_catalog.name(Key::of(&recipe.output));
        self.m_draw.plain((container[2] - textWidth(&name)) / 2, page::TITLE_TOP, &name, color::TITLE);

        let paged: bool = view.m_recipes.len() > 1;
        if paged
        {
            let index: String = format!("{}/{}", view.m_index + 1, view.m_recipes.len());
            self.m_draw.plain(container[2] - page::INSET - textWidth(&index), page::TITLE_TOP, &index, color::TITLE);
        }

        let [previous, next, back] = pageButtons(container);
        let idTop: i32 = back[1] - layout::LINE - page::ID_GAP;
        // the grid sits centered between the title and the recipe id
        let bodyTop: i32 = page::TITLE_TOP + layout::FONT_HEIGHT;
        let body: i32 = page::GRID as i32 * layout::CELL;
        self.recipe(recipe, (bodyTop + idTop - body) / 2);
        if let Some(id) = &recipe.id
        {
            self.recipeId(&id.to_string(), container, idTop);
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
    state.m_frame = state.m_frame.wrapping_add(1);
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
    if state.m_dirty || (state.m_craftable && state.m_frame.is_multiple_of(timing::CRAFTABLE_REFRESH))
    {
        state.refilter(&catalog);
    }
    state.m_page = state.m_page.min(state.pages() - 1);
    let mut painter: Painter = Painter
    {
        m_draw: draw,
        m_catalog: &catalog,
        m_mouse: mouse,
        m_frame: state.m_frame,
        m_hovered: None,
        m_hint: None,
        m_slots: Vec::new(),
    };
    painter.toggles(state, &layout);
    painter.search(state, layout.m_search);
    if layout.perPage() > 0
    {
        painter.list(state, &layout);
    }
    // only recipe page slots answer clicks, the list maps clicks through its grid
    painter.m_slots.clear();
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
            !state.m_hidden && state.m_view.is_some() && inside([x, y], state.m_layout.m_container)
        })
    }

    // the recipe page fills the container so other mods' panels beside it hide
    fn exclusive(&mut self) -> bool
    {
        STATE.with(|state|
        {
            let state: Ref<State> = state.borrow();
            !state.m_hidden && state.m_view.is_some()
        })
    }
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

pub fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for target in TARGETS
    {
        ctx.screens().overlay(target, Viewer)?;
    }
    ctx.events().on::<ScreenOpen>(Priority::Normal, |_| STATE.with(|state| state.borrow_mut().leave()))?;
    ctx.screens().tooltip(|stack, lines|
    {
        let catalog: Rc<Catalog> = catalog::get();
        if let Some(origin) = Key::ofStack(stack).and_then(|key| origin(&catalog, key))
        {
            lines.push(origin.to_owned());
        }
    })
}
