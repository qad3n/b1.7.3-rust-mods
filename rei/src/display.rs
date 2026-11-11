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
    // a mod machine by its index in the catalog, its tabs sit between smelting and the drops
    Machine(usize),
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
    Machine
    {
        machine: usize,
        recipe: usize,
    },
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
    pub m_category: Category,
    pub m_displays: Vec<Display>,
}

impl Tab
{
    // a machine tab shows its first workstation and title
    pub(crate) fn icon(&self, catalog: &Catalog) -> ItemStack
    {
        match self.m_category
        {
            Category::Machine(machine) => catalog.m_machines[machine]
                .workstations
                .first()
                .map_or_else(|| ItemStack::of(CRAFTING_ICON, 1), |stack| ItemStack::fromRaw(stack.clone())),
            category => ItemStack::of(row(category).m_icon, 1),
        }
    }

    pub(crate) fn label(&self, catalog: &Catalog) -> String
    {
        match self.m_category
        {
            Category::Machine(machine) => catalog.m_machines[machine].title.clone(),
            category => row(category).label(),
        }
    }
}

const CRAFTING_ICON: &str = "minecraft:crafting_table";

fn row(category: Category) -> &'static CategoryRow
{
    CATEGORIES.iter().find(|row| row.m_category == category).unwrap_or(&CATEGORIES[0])
}

// tab order: crafting and smelting, every machine as registered, then drops and tags
fn order(catalog: &Catalog) -> Vec<Category>
{
    let (recipes, rest): (Vec<&CategoryRow>, Vec<&CategoryRow>) =
        CATEGORIES.iter().partition(|row| matches!(row.m_category, Category::Crafting | Category::Smelting));
    let machines = (0..catalog.m_machines.len()).map(Category::Machine);

    recipes.iter().map(|row| row.m_category).chain(machines).chain(rest.iter().map(|row| row.m_category)).collect()
}

fn recipeCategory(recipe: &RecipeView) -> Category
{
    match recipe.shape
    {
        RecipeShape::Smelting { .. } => Category::Smelting,
        _ => Category::Crafting,
    }
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

    let machines: Vec<(usize, usize)> = if usages
    {
        catalog.machineUsagesOf(key)
    }
    else
    {
        catalog.machineRecipesFor(key)
    };

    let mut displays: Vec<(Category, Display)> = Vec::new();

    for index in recipes
    {
        displays.push((recipeCategory(&catalog.m_recipes[index]), Display::Recipe(index)));
    }

    // a workstation of crafting or smelting lists every recipe it runs
    if usages
    {
        for category in catalog.vanillaStations(key)
        {
            let all =
                (0..catalog.m_recipes.len()).filter(|index| recipeCategory(&catalog.m_recipes[*index]) == category);
            displays.extend(all.map(|index| (category, Display::Recipe(index))));
        }
    }

    for (machine, recipe) in machines
    {
        displays.push((Category::Machine(machine), Display::Machine { machine, recipe }));
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

    order(catalog)
        .into_iter()
        .map(|category| Tab
        {
            m_category: category,
            m_displays: displays
                .iter()
                .filter(|(shown, _)| *shown == category)
                .map(|(_, display)| display.clone())
                .collect(),
        })
        .filter(|tab| !tab.m_displays.is_empty())
        .collect()
}
