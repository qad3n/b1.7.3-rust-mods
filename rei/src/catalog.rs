use b173_api::prelude::*;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;

const ANY_DAMAGE: i32 = -1;
// blue italics as later REI versions name the owning mod under an item
const ORIGIN_STYLE: &str = "§9§o";

// an item and damage pair, the identity every list, filter and recipe lookup works on
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct Key
{
    pub m_id: i32,
    pub m_damage: i32,
}

impl Key
{
    pub(crate) fn of(stack: &StackData) -> Self { Self { m_id: stack.id, m_damage: stack.damage } }

    pub(crate) fn ofStack(stack: &ItemStack) -> Option<Self>
    {
        stack.item.id().ok().map(|id| Self { m_id: id, m_damage: stack.damage })
    }

    // a recipe ingredient with damage -1 takes every damage of its item
    pub(crate) fn accepts(self, other: Self) -> bool
    {
        self.m_id == other.m_id && (self.m_damage == ANY_DAMAGE || self.m_damage == other.m_damage)
    }

    pub(crate) fn stack(self, count: i32) -> ItemStack
    {
        ItemStack::fromRaw(StackData { id: self.m_id, count, damage: self.m_damage.max(0) })
    }
}

pub(crate) struct Entry
{
    pub m_key: Key,
    pub m_max: i32,
    pub m_name: String,
    pub m_search: String,
    pub m_namespace: String,
}

pub(crate) struct Catalog
{
    pub m_entries: Vec<Entry>,
    pub m_recipes: Vec<RecipeView>,
    m_byKey: HashMap<Key, usize>,
    m_origins: HashMap<i32, String>,
    m_mods: HashMap<String, String>,
    pub m_generation: i32,
}

thread_local!
{
    static CATALOG: RefCell<Option<Rc<Catalog>>> = const { RefCell::new(None) };
}

// built on first use, after every mod registered its items and recipes
// rebuilt whenever the loader remaps ids or rebuilds recipes, as on joining a server with its own recipe table
pub(crate) fn get() -> Rc<Catalog>
{
    let generation: i32 = contentGeneration();

    CATALOG.with(|cached|
    {
        let mut cached: RefMut<Option<Rc<Catalog>>> = cached.borrow_mut();
        match cached.as_ref().filter(|catalog| catalog.m_generation == generation)
        {
            Some(catalog) => Rc::clone(catalog),
            None => Rc::clone(cached.insert(Rc::new(Catalog::build(generation)))),
        }
    })
}

pub(crate) fn inputs(recipe: &RecipeView) -> Vec<Key>
{
    match &recipe.shape
    {
        RecipeShape::Shaped { inputs, .. } => inputs.iter().flatten().map(Key::of).collect(),
        RecipeShape::Shapeless { inputs } => inputs.iter().map(Key::of).collect(),
        RecipeShape::Smelting { input } => vec![Key::of(input)],
    }
}

impl Catalog
{
    fn build(generation: i32) -> Self
    {
        let entries: Vec<Entry> = itemCatalog().into_iter().filter_map(Self::listed).collect();
        let byKey: HashMap<Key, usize> =
            entries.iter().enumerate().map(|(index, entry)| (entry.m_key, index)).collect();
        let names: HashMap<String, String> = names(&entries);
        let origins: HashMap<i32, String> = origins(&entries, &names);
        let mods: HashMap<String, String> =
            names.into_iter().map(|(namespace, name)| (namespace, name.to_lowercase())).collect();

        Self
        {
            m_entries: entries,
            m_recipes: recipeList(),
            m_byKey: byKey,
            m_origins: origins,
            m_mods: mods,
            m_generation: generation,
        }
    }

    fn listed(stack: ItemStack) -> Option<Entry>
    {
        let key: Key = Key::ofStack(&stack)?;
        let name: String = displayName(&key.stack(1));
        let namespace: String =
            nameOf(RegistryKind::Item, key.m_id).map(|id| id.namespace().to_owned()).unwrap_or_default();

        Some(Entry
        {
            m_key: key,
            m_max: stack.count,
            m_search: name.to_lowercase(),
            m_name: name,
            m_namespace: namespace,
        })
    }

    pub(crate) fn entry(&self, key: Key) -> Option<&Entry>
    {
        self.m_byKey.get(&key).map(|index| &self.m_entries[*index])
    }

    // keyed by id alone so damaged tools find their mod too
    pub(crate) fn origin(&self, id: i32) -> Option<&str> { self.m_origins.get(&id).map(String::as_str) }

    pub(crate) fn name(&self, key: Key) -> String
    {
        self.entry(key).map_or_else(|| displayName(&key.stack(1)), |entry| entry.m_name.clone())
    }

    // the damage a wildcard ingredient shows at one step of its cycle through every damage listed for its item
    pub(crate) fn variant(&self, key: Key, step: usize) -> Key
    {
        if key.m_damage != ANY_DAMAGE
        {
            return key;
        }

        let count: usize = self.damages(key).count();
        self.damages(key).nth(step % count.max(1)).unwrap_or(Key { m_id: key.m_id, m_damage: 0 })
    }

    pub(crate) fn recipesFor(&self, key: Key) -> Vec<usize>
    {
        (0..self.m_recipes.len()).filter(|index| Key::of(&self.m_recipes[*index].output) == key).collect()
    }

    pub(crate) fn usagesOf(&self, key: Key) -> Vec<usize>
    {
        (0..self.m_recipes.len())
            .filter(|index| inputs(&self.m_recipes[*index]).iter().any(|input| input.accepts(key)))
            .collect()
    }

    // a search matches every word against the name, @word against the owning mod id or name
    pub(crate) fn search(&self, query: &str) -> Vec<usize>
    {
        let lowered: String = query.to_lowercase();
        let words: Vec<&str> = lowered.split_whitespace().collect();
        (0..self.m_entries.len())
            .filter(|index|
            {
                let entry: &Entry = &self.m_entries[*index];

                words.iter().all(|word| match word.strip_prefix('@')
                {
                    Some(owner) => self.owned(entry, owner),
                    None => entry.m_search.contains(word),
                })
            })
            .collect()
    }

    fn owned(&self, entry: &Entry, owner: &str) -> bool
    {
        entry.m_namespace.contains(owner)
            || self.m_mods.get(&entry.m_namespace).is_some_and(|name| name.contains(owner))
    }

    // outputs of crafting recipes the held items cover right now, smelting needs a furnace and is left out
    pub(crate) fn craftable(&self, held: &[(Key, i32)]) -> Vec<Key>
    {
        let mut outputs: Vec<Key> = self
            .m_recipes
            .iter()
            .filter(|recipe| !matches!(recipe.shape, RecipeShape::Smelting { .. }) && covers(held, &inputs(recipe)))
            .map(|recipe| Key::of(&recipe.output))
            .collect();
        outputs.sort_by_key(|key| (key.m_id, key.m_damage));
        outputs.dedup();
        outputs
    }

    fn damages(&self, key: Key) -> impl Iterator<Item = Key> + '_
    {
        self.m_entries.iter().map(|entry| entry.m_key).filter(move |listed| key.accepts(*listed))
    }
}

// the shown name of every namespace with listed items, asking the host once per namespace
fn names(entries: &[Entry]) -> HashMap<String, String>
{
    let mut names: HashMap<String, String> = HashMap::new();

    for entry in entries.iter().filter(|entry| !entry.m_namespace.is_empty())
    {
        names.entry(entry.m_namespace.clone()).or_insert_with(|| shown(&entry.m_namespace));
    }

    names
}

fn shown(namespace: &str) -> String
{
    let name: String = modName(namespace);
    if name.is_empty()
    {
        namespace.to_owned()
    }
    else
    {
        name
    }
}

// one tooltip line per item id
fn origins(entries: &[Entry], names: &HashMap<String, String>) -> HashMap<i32, String>
{
    entries
        .iter()
        .filter_map(|entry|
        {
            names.get(&entry.m_namespace).map(|name| (entry.m_key.m_id, format!("{ORIGIN_STYLE}{name}")))
        })
        .collect()
}

// greedy, exact damages first so a wildcard ingredient never takes the one stack an exact ingredient needs
fn covers(held: &[(Key, i32)], needed: &[Key]) -> bool
{
    let mut left: Vec<(Key, i32)> = held.to_vec();
    let mut ordered: Vec<Key> = needed.to_vec();
    ordered.sort_by_key(|key| key.m_damage == ANY_DAMAGE);

    ordered.iter().all(|ingredient|
    {
        match left.iter_mut().find(|(have, count)| *count > 0 && ingredient.accepts(*have))
        {
            Some((_, count)) =>
            {
                *count -= 1;
                true
            }

            None => false,
        }
    })
}
