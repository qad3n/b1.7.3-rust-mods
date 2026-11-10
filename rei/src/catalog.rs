use b173_api::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

const ANY_DAMAGE: i32 = -1;
// blue italics as later REI versions name the owning mod under an item
const ORIGIN_STYLE: &str = "§9§o";

// an item and damage pair, the identity every list, filter and recipe lookup works on
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Key
{
    pub m_id: i32,
    pub m_damage: i32,
}

impl Key
{
    pub fn of(stack: &StackData) -> Key { Key { m_id: stack.id, m_damage: stack.damage } }

    pub fn ofStack(stack: &ItemStack) -> Option<Key>
    {
        stack.item.id().ok().map(|id| Key { m_id: id, m_damage: stack.damage })
    }

    // a recipe ingredient with damage -1 takes every damage of its item
    pub fn accepts(self, other: Key) -> bool
    {
        self.m_id == other.m_id && (self.m_damage == ANY_DAMAGE || self.m_damage == other.m_damage)
    }

    pub fn stack(self, count: i32) -> ItemStack
    {
        ItemStack::fromRaw(StackData { id: self.m_id, count, damage: self.m_damage.max(0) })
    }
}

pub struct Entry
{
    pub m_key: Key,
    pub m_max: i32,
    pub m_name: String,
    pub m_search: String,
    pub m_namespace: String,
}

pub struct Catalog
{
    pub m_entries: Vec<Entry>,
    pub m_recipes: Vec<RecipeView>,
    m_byKey: HashMap<Key, usize>,
    m_origins: HashMap<i32, String>,
    pub m_generation: i32,
}

thread_local!
{
    static CATALOG: RefCell<Option<Rc<Catalog>>> = const { RefCell::new(None) };
}

// built on first use, after every mod registered its items and recipes
// rebuilt whenever the loader remaps ids or rebuilds recipes, as on joining a server with its own recipe table
pub fn get() -> Rc<Catalog>
{
    let generation: i32 = contentGeneration();
    CATALOG.with(|cached|
    {
        let mut cached = cached.borrow_mut();
        match cached.as_ref().filter(|catalog| catalog.m_generation == generation)
        {
            Some(catalog) => Rc::clone(catalog),
            None => Rc::clone(cached.insert(Rc::new(Catalog::build(generation)))),
        }
    })
}

pub fn inputs(recipe: &RecipeView) -> Vec<Key>
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
    fn build(generation: i32) -> Catalog
    {
        let entries: Vec<Entry> = itemCatalog().into_iter().filter_map(Catalog::listed).collect();
        let byKey: HashMap<Key, usize> =
            entries.iter().enumerate().map(|(index, entry)| (entry.m_key, index)).collect();
        let origins: HashMap<i32, String> = origins(&entries);
        Catalog
        {
            m_entries: entries,
            m_recipes: recipeList(),
            m_byKey: byKey,
            m_origins: origins,
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

    pub fn entry(&self, key: Key) -> Option<&Entry> { self.m_byKey.get(&key).map(|index| &self.m_entries[*index]) }

    // keyed by id alone so damaged tools find their mod too
    pub fn origin(&self, id: i32) -> Option<&str> { self.m_origins.get(&id).map(String::as_str) }

    pub fn name(&self, key: Key) -> String
    {
        self.entry(key).map_or_else(|| displayName(&key.stack(1)), |entry| entry.m_name.clone())
    }

    // the damage a wildcard ingredient shows at one step of its cycle through every damage listed for its item
    pub fn variant(&self, key: Key, step: usize) -> Key
    {
        if key.m_damage != ANY_DAMAGE
        {
            return key;
        }

        let count: usize = self.damages(key).count();
        self.damages(key).nth(step % count.max(1)).unwrap_or(Key { m_id: key.m_id, m_damage: 0 })
    }

    pub fn recipesFor(&self, key: Key) -> Vec<usize>
    {
        (0..self.m_recipes.len()).filter(|index| Key::of(&self.m_recipes[*index].output) == key).collect()
    }

    pub fn usagesOf(&self, key: Key) -> Vec<usize>
    {
        (0..self.m_recipes.len())
            .filter(|index| inputs(&self.m_recipes[*index]).iter().any(|input| input.accepts(key)))
            .collect()
    }

    // a search matches every word against the name, @word against the owning mod id
    pub fn search(&self, query: &str) -> Vec<usize>
    {
        let lowered: String = query.to_lowercase();
        let words: Vec<&str> = lowered.split_whitespace().collect();
        (0..self.m_entries.len())
            .filter(|index|
            {
                let entry: &Entry = &self.m_entries[*index];
                words.iter().all(|word| match word.strip_prefix('@')
                {
                    Some(namespace) => entry.m_namespace.contains(namespace),
                    None => entry.m_search.contains(word),
                })
            })
            .collect()
    }

    // outputs of crafting recipes the held items cover right now, smelting needs a furnace and is left out
    pub fn craftable(&self, held: &[(Key, i32)]) -> Vec<Key>
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

// one tooltip line per item id, asking the host once per namespace
fn origins(entries: &[Entry]) -> HashMap<i32, String>
{
    let mut lines: HashMap<&str, String> = HashMap::new();
    let mut origins: HashMap<i32, String> = HashMap::new();
    for entry in entries.iter().filter(|entry| !entry.m_namespace.is_empty())
    {
        let line: &String = lines.entry(&entry.m_namespace).or_insert_with(|| origin(&entry.m_namespace));
        origins.insert(entry.m_key.m_id, line.clone());
    }
    origins
}

fn origin(namespace: &str) -> String
{
    let name: String = modName(namespace);
    let shown: &str = if name.is_empty() { namespace } else { &name };
    format!("{ORIGIN_STYLE}{shown}")
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
