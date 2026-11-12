use super::{Painter, View, clip, color, inside, layout, page, tab, tabRect, timing};
use crate::catalog::{Catalog, Key};
use crate::display::{LOOT_PAGE, TAG_PAGE};
use crate::spawner;
use b173_api::prelude::*;
use std::rc::Rc;

// positions on a loot page from the page's corner
mod loot
{
    pub const SOURCE_LEFT: i32 = 8;
    pub const ARROW_LEFT: i32 = 30;
    pub const COLUMNS: [i32; 2] = [60, 118];
    pub const ROWS: usize = 5;
    pub const TEXT_GAP: i32 = 2;
    pub const TEXT_ROOM: i32 = 38;
}

// positions on a machine page from the page's corner
mod machine
{
    pub const COLUMNS: usize = 2;
    pub const INPUT_LEFT: i32 = 30;
    pub const ARROW_LEFT: i32 = 78;
    pub const OUTPUT_LEFT: i32 = 112;
    pub const NOTES_GAP: i32 = 4;
    pub const NOTES_ROOM: i32 = 160;
}
const TAG_COLUMNS: usize = 8;
const PERCENT: f32 = 100.0;
// the mark on a drop with a condition its tooltip names
const CONDITION: &str = "*";

impl Painter<'_>
{
    pub(crate) fn tab(&mut self, view: &View, index: usize)
    {
        let category: &crate::display::Tab = &view.m_tabs[index];
        let [x, y, w, h] = tabRect(index);
        self.m_draw.panel(x, y, w, h);

        if index == view.m_tab
        {
            // paints over the tab's inner edge and the page's outer edge so the two read as one panel
            let seam: i32 = tab::OVERLAP + tab::RIM;
            self.m_draw.rect(x + w - seam, y + tab::RIM, seam, h - tab::RIM * 2, color::PANEL_FACE);
        }

        self.m_draw.item(x + tab::ICON_INSET, y + (h - layout::ICON) / 2, &category.icon(self.m_catalog));

        if inside(self.m_mouse, [x, y, w - tab::OVERLAP, h])
        {
            self.m_hint = Some(category.label(self.m_catalog));
        }
    }

    // the source, an arrow, then two columns of drops each with its count and chance
    pub(crate) fn lootPage(&mut self, table: usize, start: usize, top: i32, bottom: i32) -> (String, Option<String>)
    {
        let catalog: &Catalog = self.m_catalog;
        let view: &LootView = &catalog.m_loot[table];
        let bodyTop: i32 = (top + bottom - page::BODY_ROWS * layout::CELL) / 2;
        let middle: i32 = bodyTop + page::BODY_ROWS / 2 * layout::CELL;

        self.slotItem(loot::SOURCE_LEFT, middle, catalog.source(&view.source));
        self.texture(loot::ARROW_LEFT, middle, page::ARROW_UV);

        for (offset, drop) in view.drops.iter().skip(start).take(LOOT_PAGE).enumerate()
        {
            let x: i32 = loot::COLUMNS[offset / loot::ROWS];
            let y: i32 = bodyTop + (offset % loot::ROWS) as i32 * layout::CELL;
            self.slotItem(x, y, Some(Key::of(&drop.stack)));

            let label: String = amount(drop, view.sampled);
            let textTop: i32 = y + (layout::ICON - layout::FONT_HEIGHT) / 2;
            let textLeft: i32 = x + layout::ICON + loot::TEXT_GAP;
            self.m_draw.text(textLeft, textTop, clip(&label, loot::TEXT_ROOM, false), color::TITLE);

            if inside(self.m_mouse, [x, y, layout::ICON, layout::ICON])
            {
                self.m_notes = notes(drop, view);
            }
        }

        (title(catalog, view), Some(sourceLine(view)))
    }

    // the inputs in up to two columns, an arrow, the outputs with their chance, then the machine's notes
    pub(crate) fn machinePage(
        &mut self,
        machine: usize,
        recipe: usize,
        span: [i32; 2],
        width: i32,
    ) -> (String, Option<String>)
    {
        let catalog: &Catalog = self.m_catalog;
        let view: &MachineView = &catalog.m_machines[machine];
        let recipe: &MachineRecipeView = &view.recipes[recipe];
        let rows: i32 = recipe.inputs.len().max(recipe.outputs.len()).div_ceil(machine::COLUMNS).max(1) as i32;
        let noteRows: i32 = recipe.notes.len() as i32;
        let height: i32 = rows * layout::CELL + noteRows * layout::LINE;
        let bodyTop: i32 = (span[0] + span[1] - height) / 2;
        let middle: i32 = bodyTop + (rows * layout::CELL - layout::ICON) / 2;

        for (offset, slot) in recipe.inputs.iter().enumerate()
        {
            let x: i32 = machine::INPUT_LEFT + (offset % machine::COLUMNS) as i32 * layout::CELL;
            let y: i32 = bodyTop + (offset / machine::COLUMNS) as i32 * layout::CELL;
            self.machineSlot(x, y, slot);
        }

        self.texture(machine::ARROW_LEFT, middle, page::ARROW_UV);

        for (offset, slot) in recipe.outputs.iter().enumerate()
        {
            let y: i32 = bodyTop + offset as i32 * layout::CELL;
            self.machineSlot(machine::OUTPUT_LEFT, y, slot);

            if slot.chance < 1.0
            {
                let label: String = percent(slot.chance, false);
                let textTop: i32 = y + (layout::ICON - layout::FONT_HEIGHT) / 2;
                let textLeft: i32 = machine::OUTPUT_LEFT + layout::ICON + loot::TEXT_GAP;
                self.m_draw.text(textLeft, textTop, &label, color::TITLE);
            }
        }

        let notesTop: i32 = bodyTop + rows * layout::CELL + machine::NOTES_GAP;
        for (row, note) in recipe.notes.iter().enumerate()
        {
            let room: i32 = machine::NOTES_ROOM;
            let line: &str = clip(note, room, false);
            let x: i32 = (width - textWidth(line)) / 2;
            self.m_draw.text(x, notesTop + row as i32 * layout::LINE, line, color::TITLE);
        }

        (view.title.clone(), Some(view.id.to_string()))
    }

    // a slot a tag fills shows each of its items in turn
    fn machineSlot(&mut self, x: i32, y: i32, slot: &MachineSlot)
    {
        self.slot(x, y, layout::ICON);

        let count: usize = slot.stacks.len();
        if count == 0
        {
            return;
        }

        let stack: &StackData = &slot.stacks[(self.m_ticks / timing::CYCLE) as usize % count];
        self.stack(x, y, Key::of(stack), stack.count);
    }

    // every item the tag holds in a grid, members with subtypes cycling through them
    pub(crate) fn tagPage(
        &mut self,
        tag: &Identifier,
        start: usize,
        span: [i32; 2],
        width: i32,
    ) -> (String, Option<String>)
    {
        let members: Rc<[Key]> = self.m_catalog.members(tag);
        let bodyTop: i32 = (span[0] + span[1] - page::BODY_ROWS * layout::CELL) / 2;
        let left: i32 = (width - TAG_COLUMNS as i32 * layout::CELL) / 2;

        for (offset, key) in members.iter().skip(start).take(TAG_PAGE).enumerate()
        {
            let x: i32 = left + (offset % TAG_COLUMNS) as i32 * layout::CELL + 1;
            let y: i32 = bodyTop + (offset / TAG_COLUMNS) as i32 * layout::CELL + 1;
            self.slotItem(x, y, Some(*key));
        }

        (format!("#{tag}"), Some(format!("{} entries", members.len())))
    }
}

fn title(catalog: &Catalog, view: &LootView) -> String
{
    match &view.source
    {
        LootSource::Block { block, meta } => catalog.name(Key { m_id: *block, m_damage: *meta }),
        LootSource::Entity { entity } => spawner::humanize(entity.path()),
        LootSource::Chest { table } => format!("{} Chest", spawner::humanize(table.path())),
    }
}

fn sourceLine(view: &LootView) -> String
{
    let id: String = match &view.source
    {
        LootSource::Block { block, .. } =>
        {
            nameOf(RegistryKind::Block, *block).map(|id| id.to_string()).unwrap_or_default()
        }
        LootSource::Entity { entity } => entity.to_string(),
        LootSource::Chest { table } => table.to_string(),
    };

    if view.rolls > 1
    {
        format!("{id}, {} rolls", view.rolls)
    }
    else
    {
        id
    }
}

// the count range and chance beside a drop, a sampled chance marked as measured
fn amount(drop: &LootDrop, sampled: bool) -> String
{
    let (min, max): (i32, i32) = (drop.stack.count, drop.max);
    let mut parts: Vec<String> = Vec::new();

    if min != max
    {
        parts.push(format!("{min}-{max}"));
    }
    else if min != 1
    {
        parts.push(min.to_string());
    }

    if drop.chance < 1.0
    {
        parts.push(percent(drop.chance, sampled));
    }

    let mut label: String = parts.join(" ");
    if drop.player || drop.note.is_some()
    {
        label.push_str(CONDITION);
    }

    label
}

fn percent(chance: f32, sampled: bool) -> String
{
    let value: f32 = chance * PERCENT;
    let shown: String = if value >= 1.0
    {
        format!("{value:.0}%")
    }
    else
    {
        format!("{value:.1}%")
    };

    if sampled { format!("~{shown}") } else { shown }
}

// the tooltip lines under a drop's name spelling out its count, chance and conditions
fn notes(drop: &LootDrop, view: &LootView) -> Vec<String>
{
    let (min, max): (i32, i32) = (drop.stack.count, drop.max);
    let count: String = if min == max
    {
        format!("Count {min}")
    }
    else
    {
        format!("Count {min} to {max}")
    };
    let mut lines: Vec<String> = vec![count];

    if drop.chance < 1.0
    {
        lines.push(format!("Chance {}", percent(drop.chance, view.sampled)));
    }

    if view.rolls > 1
    {
        lines.push(format!("Rolled {} times", view.rolls));
    }

    if view.sampled
    {
        lines.push("Measured from the block's drop code".to_owned());
    }

    if drop.player
    {
        lines.push("Only when a player kills it".to_owned());
    }

    lines.extend(drop.note.clone());
    lines
}
