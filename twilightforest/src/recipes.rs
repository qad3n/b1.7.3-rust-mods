use crate::ids::{block, item};
use crate::items::armorName;
use b173_api::prelude::*;

// one recipe key, a symbol and the item it stands for
type Key = (char, &'static str);

// TFRecipes rows: name, result, count, pattern, keys
struct Shaped
{
    m_name: &'static str,
    m_result: &'static str,
    m_count: i32,
    m_pattern: &'static [&'static str],
    m_keys: &'static [Key],
}

struct Shapeless
{
    m_name: &'static str,
    m_result: &'static str,
    m_count: i32,
    m_ingredients: &'static [&'static str],
}

const fn shaped(
    name: &'static str,
    result: &'static str,
    count: i32,
    pattern: &'static [&'static str],
    keys: &'static [Key],
) -> Shaped
{
    Shaped { m_name: name, m_result: result, m_count: count, m_pattern: pattern, m_keys: keys }
}

const fn shapeless(
    name: &'static str,
    result: &'static str,
    count: i32,
    ingredients: &'static [&'static str],
) -> Shapeless
{
    Shapeless { m_name: name, m_result: result, m_count: count, m_ingredients: ingredients }
}

const STICK: &str = "minecraft:stick";
const PAPER: &str = "minecraft:paper";
// a recipe ingredient of this damage takes any damage, ShapelessRecipes' wildcard
const ANY_DAMAGE: i32 = -1;
const ORE_BLOCKS: [&str; 3] = ["minecraft:iron_block", "minecraft:diamond_block", "minecraft:gold_block"];
const IRON: &str = "minecraft:iron_ingot";
const REDSTONE: &str = "minecraft:redstone";
// beta has no glass bottle, gold nugget, blaze rod or ghast tear, the closest beta item stands in for each
const BOTTLE: &str = "minecraft:glass";
const NUGGET: &str = "minecraft:gold_ingot";
const BLAZE_ROD: &str = STICK;
const GHAST_TEAR: &str = "minecraft:gunpowder";

const SHAPED: [Shaped; 22] = [
    shaped(
        "empty_magic_map",
        item::EMPTY_MAGIC_MAP,
        1,
        &["###", "#X#", "###"],
        &[('#', PAPER), ('X', item::MAGIC_MAP_FOCUS)],
    ),
    shaped(
        "empty_maze_map",
        item::EMPTY_MAZE_MAP,
        1,
        &["###", "#X#", "###"],
        &[('#', PAPER), ('X', item::MAZE_MAP_FOCUS)],
    ),
    shaped(
        "uncrafting_table",
        "uncrafting_table",
        1,
        &["###", "#X#", "###"],
        &[('#', "minecraft:crafting_table"), ('X', item::MAZE_MAP_FOCUS)],
    ),
    shaped(
        "feather_arrows",
        "minecraft:arrow",
        4,
        &["X", "#", "Y"],
        &[('X', "minecraft:flint"), ('#', STICK), ('Y', item::RAVEN_FEATHER)],
    ),
    shaped("berry_torches", "minecraft:torch", 5, &["B", "S"], &[('B', item::TORCHBERRIES), ('S', STICK)]),
    shaped("towerwood", block::TOWERWOOD, 4, &["##", "##"], &[('#', block::DARKWOOD_LOG)]),
    shaped("encased_towerwood", block::ENCASED_TOWERWOOD, 3, &["#", "#", "#"], &[('#', block::TOWERWOOD)]),
    shaped(
        "carminite",
        item::CARMINITE,
        1,
        &["brb", "rgr", "brb"],
        &[('b', item::BORER_ESSENCE), ('r', REDSTONE), ('g', GHAST_TEAR)],
    ),
    shaped(
        "carminite_turned",
        item::CARMINITE,
        1,
        &["rbr", "bgb", "rbr"],
        &[('b', item::BORER_ESSENCE), ('r', REDSTONE), ('g', GHAST_TEAR)],
    ),
    shaped(
        "vanishing_block",
        block::VANISHING,
        8,
        &["ewe", "wcw", "ewe"],
        &[('e', block::ENCASED_TOWERWOOD), ('w', block::TOWERWOOD), ('c', item::CARMINITE)],
    ),
    shaped(
        "reappearing_block",
        block::REAPPEARING,
        2,
        &["ere", "rcr", "ere"],
        &[('e', block::ENCASED_TOWERWOOD), ('r', REDSTONE), ('c', item::CARMINITE)],
    ),
    shaped(
        "carminite_builder",
        block::BUILDER,
        1,
        &["ece", "cdc", "ece"],
        &[('e', block::ENCASED_TOWERWOOD), ('d', "minecraft:dispenser"), ('c', item::CARMINITE)],
    ),
    shaped(
        "carminite_reactor",
        block::REACTOR,
        1,
        &["ece", "coc", "ece"],
        &[('e', block::ENCASED_TOWERWOOD), ('o', "minecraft:redstone_ore"), ('c', item::CARMINITE)],
    ),
    shaped(
        "encased_smoker",
        block::ENCASED_SMOKER,
        1,
        &["ere", "rsr", "ere"],
        &[('e', block::ENCASED_TOWERWOOD), ('r', REDSTONE), ('s', block::SMOKER)],
    ),
    shaped(
        "encased_fire_jet",
        block::ENCASED_FIRE_JET,
        1,
        &["ere", "rjr", "lll"],
        &[('e', block::ENCASED_TOWERWOOD), ('r', REDSTONE), ('l', "minecraft:lava_bucket"), ('j', block::FIRE_JET)],
    ),
    shaped("shard_cluster", item::SHARD_CLUSTER, 1, &["###", "###", "###"], &[('#', item::ARMOR_SHARD)]),
    shaped("knightmetal_ring", item::KNIGHTMETAL_RING, 1, &[" # ", "# #", " # "], &[('#', item::KNIGHTMETAL)]),
    shaped("knightmetal_block", block::KNIGHTMETAL_BLOCK, 1, &["###", "###", "###"], &[('#', item::KNIGHTMETAL)]),
    shaped(
        "knightly_axe_mirrored",
        item::KNIGHTLY_AXE,
        1,
        &["##", "X#", "X "],
        &[('#', item::KNIGHTMETAL), ('X', STICK)],
    ),
    shaped("aurora_slab", block::AURORA_SLAB, 6, &["###"], &[('#', block::AURORA_BRICK)]),
    shaped("aurora_pillar", block::AURORA_PILLAR, 2, &["#", "#"], &[('#', block::AURORA_BRICK)]),
    shaped(
        "giant_sword",
        item::GIANT_SWORD,
        1,
        &["#", "#", "X"],
        &[('#', block::GIANT_COBBLE), ('X', block::GIANT_LOG)],
    ),
];

const SHAPELESS: [Shapeless; 13] = [
    shapeless("firefly_jar", block::FIREFLY_JAR, 1, &[block::FIREFLY, BOTTLE]),
    shapeless(
        "magic_map_focus",
        item::MAGIC_MAP_FOCUS,
        1,
        &[item::RAVEN_FEATHER, item::TORCHBERRIES, "minecraft:glowstone_dust"],
    ),
    shapeless("root_stick", STICK, 1, &[block::ROOT_STRAND]),
    shapeless("raw_ironwood", item::RAW_IRONWOOD, 1, &[item::LIVEROOT, IRON, NUGGET]),
    shapeless("fiery_ingot_blood", item::FIERY_INGOT, 1, &[item::FIERY_BLOOD, IRON]),
    shapeless("fiery_ingot_tears", item::FIERY_INGOT, 1, &[item::FIERY_TEARS, IRON]),
    shapeless(
        "charm_of_keeping_2",
        item::CHARM_OF_KEEPING_2,
        1,
        &[item::CHARM_OF_KEEPING_1, item::CHARM_OF_KEEPING_1, item::CHARM_OF_KEEPING_1, item::CHARM_OF_KEEPING_1],
    ),
    shapeless(
        "charm_of_keeping_3",
        item::CHARM_OF_KEEPING_3,
        1,
        &[item::CHARM_OF_KEEPING_2, item::CHARM_OF_KEEPING_2, item::CHARM_OF_KEEPING_2, item::CHARM_OF_KEEPING_2],
    ),
    shapeless(
        "charm_of_life_2",
        item::CHARM_OF_LIFE_2,
        1,
        &[item::CHARM_OF_LIFE_1, item::CHARM_OF_LIFE_1, item::CHARM_OF_LIFE_1, item::CHARM_OF_LIFE_1],
    ),
    shapeless("knightmetal_from_block", item::KNIGHTMETAL, 9, &[block::KNIGHTMETAL_BLOCK]),
    shapeless("cobble_from_giant", "minecraft:cobblestone", 64, &[block::GIANT_COBBLE]),
    shapeless("planks_from_giant", "minecraft:planks", 64, &[block::GIANT_LOG]),
    shapeless("leaves_from_giant", "minecraft:leaves", 64, &[block::GIANT_LEAVES]),
];

// TFRecipes smelting: logs to charcoal and the raw ores and meats
const SMELTING: [(&str, &str, i32); 4] = [
    (item::RAW_IRONWOOD, item::IRONWOOD_INGOT, 2),
    (item::RAW_VENISON, item::VENISON, 1),
    (item::RAW_MEEF, item::MEEF_STEAK, 1),
    (item::SHARD_CLUSTER, item::KNIGHTMETAL, 1),
];

const LOGS: [(&str, &str); 8] = [
    (block::OAK_LOG, "twilight_oak_planks"),
    (block::CANOPY_LOG, "canopy_planks"),
    (block::MANGROVE_LOG, "mangrove_planks"),
    (block::DARKWOOD_LOG, "darkwood_planks"),
    (block::TIMEWOOD, "timewood_planks"),
    (block::TRANSWOOD, "transwood_planks"),
    (block::MINEWOOD, "minewood_planks"),
    (block::SORTINGWOOD, "sortingwood_planks"),
];

const PLANKS_PER_LOG: i32 = 4;
const CHARCOAL: i32 = 1;

// vanilla armor and tool patterns, helmet chestplate leggings boots
const ARMOR_PATTERNS: [&[&str]; 4] = [&["###", "# #"], &["# #", "###", "###"], &["###", "# #", "# #"], &["# #", "# #"]];

struct GearSet
{
    m_set: &'static str,
    m_material: &'static str,
    m_handle: &'static str,
    // armor slots this set has, helmet first
    m_armor: [bool; 4],
    m_tools: &'static [(&'static str, &'static [&'static str])],
}

const SWORD: &[&str] = &["#", "#", "X"];
const SHOVEL: &[&str] = &["#", "X", "X"];
const PICK: &[&str] = &["###", " X ", " X "];
const AXE: &[&str] = &["##", "#X", " X"];
const HOE: &[&str] = &["##", " X", " X"];

const GEAR: [GearSet; 7] = [
    GearSet
    {
        m_set: "naga",
        m_material: item::NAGA_SCALE,
        m_handle: STICK,
        m_armor: [false, true, true, false],
        m_tools: &[],
    },
    GearSet
    {
        m_set: "ironwood",
        m_material: item::IRONWOOD_INGOT,
        m_handle: STICK,
        m_armor: [true; 4],
        m_tools: &[
            ("ironwood_sword", SWORD),
            ("ironwood_shovel", SHOVEL),
            ("ironwood_pickaxe", PICK),
            ("ironwood_axe", AXE),
            ("ironwood_hoe", HOE),
        ],
    },
    GearSet
    {
        m_set: "fiery",
        m_material: item::FIERY_INGOT,
        m_handle: BLAZE_ROD,
        m_armor: [true; 4],
        m_tools: &[(item::FIERY_SWORD, SWORD), (item::FIERY_PICK, PICK)],
    },
    GearSet
    {
        m_set: "steeleaf",
        m_material: item::STEELEAF,
        m_handle: STICK,
        m_armor: [true; 4],
        m_tools: &[
            ("steeleaf_sword", SWORD),
            ("steeleaf_shovel", SHOVEL),
            ("steeleaf_pickaxe", PICK),
            ("steeleaf_axe", AXE),
            ("steeleaf_hoe", HOE),
        ],
    },
    GearSet
    {
        m_set: "knightly",
        m_material: item::KNIGHTMETAL,
        m_handle: STICK,
        m_armor: [true; 4],
        m_tools: &[(item::KNIGHTLY_SWORD, SWORD), (item::KNIGHTLY_PICK, PICK), (item::KNIGHTLY_AXE, AXE)],
    },
    GearSet { m_set: "yeti", m_material: item::ALPHA_FUR, m_handle: STICK, m_armor: [true; 4], m_tools: &[] },
    GearSet { m_set: "arctic", m_material: item::ARCTIC_FUR, m_handle: STICK, m_armor: [true; 4], m_tools: &[] },
];

fn build(ctx: &mut RegisterCtx, name: &str, result: ItemStack, pattern: &[&str], keys: &[Key]) -> Result<()>
{
    let mut builder: ShapedBuilder = ctx.recipes().shaped(name, result).rows(pattern);

    for (symbol, item) in keys
    {
        builder = builder.key(*symbol, *item);
    }

    builder.build()
}

fn gear(ctx: &mut RegisterCtx) -> Result<()>
{
    for set in &GEAR
    {
        for (slot, pattern) in ARMOR_PATTERNS.iter().enumerate().filter(|(slot, _)| set.m_armor[*slot])
        {
            let piece: String = armorName(set.m_set, slot);
            build(ctx, &piece, ItemStack::of(piece.as_str(), 1), pattern, &[('#', set.m_material)])?;
        }

        for (tool, pattern) in set.m_tools
        {
            build(ctx, tool, ItemStack::of(*tool, 1), pattern, &[('#', set.m_material), ('X', set.m_handle)])?;
        }
    }

    build(
        ctx,
        item::GIANT_PICK,
        ItemStack::of(item::GIANT_PICK, 1),
        PICK,
        &[('#', block::GIANT_COBBLE), ('X', block::GIANT_LOG)],
    )
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for row in &SHAPED
    {
        build(ctx, row.m_name, ItemStack::of(row.m_result, row.m_count), row.m_pattern, row.m_keys)?;
    }

    for row in &SHAPELESS
    {
        let ingredients: Vec<ItemStack> = row.m_ingredients.iter().map(|name| ItemStack::of(*name, 1)).collect();
        ctx.recipes().shapeless(row.m_name, ItemStack::of(row.m_result, row.m_count), &ingredients)?;
    }

    // TFRecipes makes a blank ore map from a filled or blank maze map, a filled one of any number
    for (name, base, damage) in
        [("ore_map_from_maze_map", item::MAZE_MAP, ANY_DAMAGE), ("ore_map", item::EMPTY_MAZE_MAP, 0)]
    {
        let mut ingredients: Vec<ItemStack> = vec![ItemStack::of(base, 1).withDamage(damage)];
        ingredients.extend(ORE_BLOCKS.map(|block| ItemStack::of(block, 1)));
        ctx.recipes().shapeless(name, ItemStack::of(item::EMPTY_ORE_MAP, 1), &ingredients)?;
    }

    for (input, output, count) in SMELTING
    {
        ctx.recipes().smelting(&format!("smelt_{input}"), input, ItemStack::of(output, count))?;
    }

    for (log, planks) in LOGS
    {
        ctx.recipes().shapeless(
            &format!("{planks}_from_log"),
            ItemStack::of(planks, PLANKS_PER_LOG),
            &[ItemStack::of(log, 1)],
        )?;
        ctx.recipes().smelting(
            &format!("charcoal_from_{log}"),
            log,
            ItemStack::of("minecraft:coal", 1).withDamage(CHARCOAL),
        )?;
    }

    gear(ctx)
}

fn fits(want: Option<&StackData>, have: Option<&ItemStack>) -> bool
{
    match (want, have)
    {
        (None, None) => true,

        (Some(want), Some(have)) =>
        {
            have.item.id().is_ok_and(|id| id == want.id) && (want.damage < 0 || want.damage == have.damage)
        }

        _ => false,
    }
}

const GRID: i32 = 3;

// ShapedRecipes.matches, the pattern anywhere in the grid and mirrored left to right
fn shapedMatch(width: i32, inputs: &[Option<StackData>], grid: &[Option<ItemStack>]) -> bool
{
    let height: i32 = inputs.len() as i32 / width.max(1);

    for ox in 0..=GRID - width
    {
        for oy in 0..=GRID - height
        {
            for mirror in [false, true]
            {
                let matched: bool = (0..GRID * GRID).all(|cell|
                {
                    let (x, y): (i32, i32) = (cell % GRID - ox, cell / GRID - oy);
                    let inside: bool = (0..width).contains(&x) && (0..height).contains(&y);
                    let column: i32 = if mirror { width - 1 - x } else { x };

                    let want: Option<&StackData> = if inside
                    {
                        inputs[(column + y * width) as usize].as_ref()
                    }
                    else
                    {
                        None
                    };
                    fits(want, grid[cell as usize].as_ref())
                });
                if matched
                {
                    return true;
                }
            }
        }
    }

    false
}

// ShapelessRecipes.matches, every ingredient used once
fn shapelessMatch(inputs: &[StackData], grid: &[Option<ItemStack>]) -> bool
{
    let mut left: Vec<&StackData> = inputs.iter().collect();

    for have in grid.iter().flatten()
    {
        let Some(index) = left.iter().position(|want| fits(Some(want), Some(have)))
        else
        {
            return false;
        };

        left.remove(index);
    }

    left.is_empty()
}

// CraftingManager.findMatchingRecipe over a three by three grid
pub(crate) fn craft(grid: &[Option<ItemStack>]) -> Option<ItemStack>
{
    recipeList()
        .into_iter()
        .find(|recipe| match &recipe.shape
        {
            RecipeShape::Shaped { width, inputs } => shapedMatch(*width, inputs, grid),
            RecipeShape::Shapeless { inputs } => shapelessMatch(inputs, grid),
            RecipeShape::Smelting { .. } => false,
        })
        .map(|recipe| ItemStack::fromRaw(recipe.output))
}

// the smelting result of the input, FurnaceRecipes.getSmeltingResult
pub(crate) fn smelt(input: &ItemStack) -> Option<ItemStack>
{
    let id: i32 = input.item.id().ok()?;

    recipeList().into_iter().find_map(|recipe| match recipe.shape
    {
        RecipeShape::Smelting { input: want } if want.id == id && (want.damage < 0 || want.damage == input.damage) =>
        {
            Some(ItemStack::fromRaw(recipe.output))
        }

        _ => None,
    })
}
