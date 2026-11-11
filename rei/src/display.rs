use crate::catalog::{Catalog, Key};
use b173_api::prelude::*;

// drops on one loot page, two columns of five rows
pub(crate) const LOOT_PAGE: usize = 10;
// members on one tag page, eight columns of five rows
pub(crate) const TAG_PAGE: usize = 40;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Category
{
    Crafting,
    Smelting,
    Mobs,
    Blocks,
    Chests,
    Tags,
}

pub(crate) struct CategoryRow
{
    pub m_category: Category,
    pub m_icon: &'static str,
    // a REI lang key, or the text itself where REI has no category
    pub m_label: &'static str,
    pub m_lang: bool,
}

// the vertical tabs of a recipe page in order
pub(crate) const CATEGORIES: [CategoryRow; 6] = [
    CategoryRow
    {
        m_category: Category::Crafting,
        m_icon: "minecraft:crafting_table",
        m_label: "rei.category.rei.crafting",
        m_lang: true,
    },
    CategoryRow
    {
        m_category: Category::Smelting,
        m_icon: "minecraft:furnace",
        m_label: "rei.category.rei.smelting",
        m_lang: true,
    },
    CategoryRow { m_category: Category::Mobs, m_icon: "minecraft:mob_spawner", m_label: "Mob Drops", m_lang: false },
    CategoryRow
    {
        m_category: Category::Blocks,
        m_icon: "minecraft:iron_pickaxe",
        m_label: "Block Drops",
        m_lang: false,
    },
    CategoryRow { m_category: Category::Chests, m_icon: "minecraft:chest", m_label: "Chest Loot", m_lang: false },
    CategoryRow { m_category: Category::Tags, m_icon: "minecraft:book", m_label: "Tags", m_lang: false },
];

impl CategoryRow
{
    pub(crate) fn label(&self) -> String
    {
        if self.m_lang
        {
            translate(self.m_label)
        }
        else
        {
            self.m_label.to_owned()
        }
    }
}

// one page of a tab, a long drop table or tag spans several
#[derive(Clone)]
pub(crate) enum Display
{
    Recipe(usize),
    Loot
    {
        table: usize,
        start: usize,
    },
    Tag
    {
        tag: Identifier,
        start: usize,
    },
}

#[derive(Clone)]
pub(crate) struct Tab
{
    pub m_row: &'static CategoryRow,
    pub m_displays: Vec<Display>,
}

// how to get the item, or with usages what it makes, drops or holds, one tab per category with anything to show
pub(crate) fn tabs(catalog: &Catalog, key: Key, usages: bool) -> Vec<Tab>
{
    let recipes: Vec<usize> = if usages
    {
        catalog.usagesOf(key)
    }
    else
    {
        catalog.recipesFor(key)
    };

    let loot: Vec<usize> = if usages
    {
        catalog.lootFrom(key)
    }
    else
    {
        catalog.lootFor(key)
    };

    let mut displays: Vec<(Category, Display)> = Vec::new();

    for index in recipes
    {
        let category: Category = match catalog.m_recipes[index].shape
        {
            RecipeShape::Smelting { .. } => Category::Smelting,
            _ => Category::Crafting,
        };
        displays.push((category, Display::Recipe(index)));
    }

    for table in loot
    {
        let view: &LootView = &catalog.m_loot[table];
        let category: Category = match view.source
        {
            LootSource::Block { .. } => Category::Blocks,
            LootSource::Entity { .. } => Category::Mobs,
            LootSource::Chest { .. } => Category::Chests,
        };
        let pages = (0..view.drops.len()).step_by(LOOT_PAGE).map(|start| Display::Loot { table, start });
        displays.extend(pages.map(|display| (category, display)));
    }

    for tag in catalog.tags(key.m_id)
    {
        let count: usize = catalog.members(&tag).len();
        let pages = (0..count).step_by(TAG_PAGE).map(|start| Display::Tag { tag: tag.clone(), start });
        displays.extend(pages.map(|display| (Category::Tags, display)));
    }

    CATEGORIES
        .iter()
        .map(|row| Tab
        {
            m_row: row,
            m_displays: displays
                .iter()
                .filter(|(category, _)| *category == row.m_category)
                .map(|(_, display)| display.clone())
                .collect(),
        })
        .filter(|tab| !tab.m_displays.is_empty())
        .collect()
}
