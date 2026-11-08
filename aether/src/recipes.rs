use crate::ids::{block, item};
use b173_api::prelude::*;

// one recipe key: symbol, item and metadata, ANY matches every metadata
type Key = (char, &'static str, i32);

// AetherRegistries.initializeRecipes rows: name, result, count, pattern, keys
struct Shaped
{
    m_name: &'static str,
    m_result: &'static str,
    m_count: i32,
    m_pattern: &'static [&'static str],
    m_keys: &'static [Key],
}

// AetherRegistries.initializeShapelessRecipes rows
struct Shapeless
{
    m_name: &'static str,
    m_result: &'static str,
    m_ingredients: &'static [&'static str],
}

const ANY: i32 = -1;
const PLANKS: &str = "skyroot_planks";
const ENCHANTED_GRAVITITE: &str = "enchanted_gravitite";
const POISON_BUCKET: &str = "skyroot_poison_bucket";
const STICK: Key = ('Y', item::SKYROOT_STICK, 0);

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

// beta doors and signs stack to one, so they craft one at a time instead of the source's three
const SHAPED: [Shaped; 37] = [
    shaped(
        "nature_staff",
        "nature_staff",
        1,
        &["Y", "X"],
        &[('Y', item::ZANITE_GEMSTONE, 0), ('X', item::SKYROOT_STICK, 0)],
    ),
    shaped("skyroot_stick", item::SKYROOT_STICK, 4, &["X", "X"], &[('X', PLANKS, 0)]),
    shaped("trapdoor", "minecraft:trapdoor", 2, &["XXX", "XXX"], &[('X', PLANKS, 0)]),
    shaped("holystone_brick", "holystone_brick", 4, &["XX", "XX"], &[('X', block::HOLYSTONE, 0)]),
    shaped("zanite_block", "zanite_block", 1, &["XXX", "XXX", "XXX"], &[('X', item::ZANITE_GEMSTONE, 0)]),
    shaped("zanite_gemstone", item::ZANITE_GEMSTONE, 9, &["X"], &[('X', "zanite_block", 0)]),
    shaped(
        "golden_dart_shooter",
        "golden_dart_shooter",
        1,
        &["X  ", " Y ", "  Y"],
        &[('X', item::GOLDEN_AMBER, 0), ('Y', PLANKS, 0)],
    ),
    shaped(
        "golden_dart",
        "golden_dart",
        1,
        &["X", "Y", "Z"],
        &[('X', "minecraft:feather", 0), STICK, ('Z', item::GOLDEN_AMBER, 0)],
    ),
    shaped(
        "poison_dart",
        "poison_dart",
        8,
        &["XXX", "XYX", "XXX"],
        &[('X', "golden_dart", 0), ('Y', POISON_BUCKET, 0)],
    ),
    shaped(
        "incubator",
        "incubator",
        1,
        &["XXX", "XZX", "XXX"],
        &[('X', block::HOLYSTONE, 0), ('Z', "ambrosium_torch", 0)],
    ),
    shaped(
        "freezer",
        "freezer",
        1,
        &["XXX", "XYX", "ZZZ"],
        &[('X', block::HOLYSTONE, 0), ('Y', block::ICESTONE, 0), ('Z', PLANKS, 0)],
    ),
    shaped(
        "enchanter",
        "enchanter",
        1,
        &["XXX", "XYX", "XXX"],
        &[('X', block::HOLYSTONE, 0), ('Y', item::ZANITE_GEMSTONE, 0)],
    ),
    shaped("furnace", "minecraft:furnace", 1, &["XXX", "X X", "XXX"], &[('X', block::HOLYSTONE, 0)]),
    shaped("ladder", "minecraft:ladder", 4, &["X X", "XXX", "X X"], &[('X', item::SKYROOT_STICK, 0)]),
    shaped(
        "jukebox",
        "minecraft:jukebox",
        1,
        &["XXX", "XYX", "XXX"],
        &[('X', PLANKS, 0), ('Y', ENCHANTED_GRAVITITE, 0)],
    ),
    shaped("oak_door", "minecraft:wooden_door", 1, &["XX", "XX", "XX"], &[('X', PLANKS, 0)]),
    shaped("sign", "minecraft:sign", 1, &["XXX", "XXX", " Y "], &[('X', PLANKS, 0), STICK]),
    shaped("ambrosium_torch", "ambrosium_torch", 2, &["Z", "Y"], &[('Z', item::AMBROSIUM_SHARD, 0), STICK]),
    shaped("cold_parachute", "cold_parachute", 1, &["XX", "XX"], &[('X', block::COLD_AERCLOUD, 0)]),
    shaped("golden_parachute", "golden_parachute", 1, &["XX", "XX"], &[('X', block::GOLDEN_AERCLOUD, 0)]),
    shaped(
        "saddle",
        "minecraft:saddle",
        1,
        &["XXX", "XZX"],
        &[('X', "minecraft:leather", 0), ('Z', "minecraft:string", 0)],
    ),
    shaped("chest", "minecraft:chest", 1, &["XXX", "X X", "XXX"], &[('X', PLANKS, 0)]),
    shaped("skyroot_bucket", "skyroot_bucket", 1, &["Z Z", " Z "], &[('Z', PLANKS, 0)]),
    shaped("crafting_table", "minecraft:crafting_table", 1, &["XX", "XX"], &[('X', PLANKS, 0)]),
    shaped("white_cape", "white_cape", 1, &["ZZ", "ZZ", "ZZ"], &[('Z', "minecraft:wool", 0)]),
    shaped(
        "iron_pendant",
        "iron_pendant",
        1,
        &[" Z ", "Z Z", " ZS"],
        &[('Z', "minecraft:iron_ingot", 0), ('S', "minecraft:string", 0)],
    ),
    shaped(
        "golden_pendant",
        "golden_pendant",
        1,
        &[" Z ", "Z Z", " ZS"],
        &[('Z', "minecraft:gold_ingot", 0), ('S', "minecraft:string", 0)],
    ),
    shaped("leather_gloves", "leather_gloves", 1, &["C C"], &[('C', "minecraft:leather", 0)]),
    shaped("iron_gloves", "iron_gloves", 1, &["C C"], &[('C', "minecraft:iron_ingot", 0)]),
    shaped("golden_gloves", "golden_gloves", 1, &["C C"], &[('C', "minecraft:gold_ingot", 0)]),
    shaped("diamond_gloves", "diamond_gloves", 1, &["C C"], &[('C', "minecraft:diamond", 0)]),
    shaped("zanite_gloves", "zanite_gloves", 1, &["C C"], &[('C', item::ZANITE_GEMSTONE, 0)]),
    shaped("gravitite_gloves", "gravitite_gloves", 1, &["C C"], &[('C', ENCHANTED_GRAVITITE, 0)]),
    shaped("skyroot_fence", "skyroot_fence", 3, &["ZXZ", "ZXZ"], &[('Z', PLANKS, 0), ('X', item::SKYROOT_STICK, 0)]),
    shaped(
        "skyroot_fence_gate",
        "skyroot_fence_gate",
        1,
        &["ZXZ", "ZXZ"],
        &[('X', PLANKS, 0), ('Z', item::SKYROOT_STICK, 0)],
    ),
    shaped(
        "skyroot_bookshelf",
        "skyroot_bookshelf",
        1,
        &["ZZZ", "XXX", "ZZZ"],
        &[('Z', PLANKS, 0), ('X', "minecraft:book", 0)],
    ),
    shaped(
        "skyroot_bed_item",
        "skyroot_bed_item",
        1,
        &["XXX", "ZZZ"],
        &[('Z', PLANKS, 0), ('X', "minecraft:wool", ANY)],
    ),
];

const SHAPELESS: [Shapeless; 4] = [
    Shapeless {
        m_name: "poison_dart_shooter",
        m_result: "poison_dart_shooter",
        m_ingredients: &["golden_dart_shooter", POISON_BUCKET],
    },
    Shapeless {
        m_name: "book_of_lore_ambrosium",
        m_result: "lore_book",
        m_ingredients: &["minecraft:book", item::AMBROSIUM_SHARD],
    },
    Shapeless {
        m_name: "book_of_lore_flint",
        m_result: "lore_book",
        m_ingredients: &["minecraft:book", "minecraft:flint"],
    },
    Shapeless {
        m_name: "book_of_lore_glowstone",
        m_result: "lore_book",
        m_ingredients: &["minecraft:book", "minecraft:glowstone_dust"],
    },
];

// slab, wall and stair families: block prefix, the block they are cut from, and whether walls exist
const FAMILIES: [(&str, &str, bool); 7] = [
    ("carved", "carved_stone", true),
    ("angelic", "angelic_stone", true),
    ("hellfire", "hellfire_stone", true),
    ("holystone", block::HOLYSTONE, true),
    ("mossy_holystone", "mossy_holystone", true),
    ("holystone_brick", "holystone_brick", true),
    ("skyroot", PLANKS, false),
];
const FAMILY_SHAPES: [(&str, i32, &[&str]); 3] =
    [("slab", 6, &["ZZZ"]), ("wall", 6, &["ZZZ", "ZZZ"]), ("stairs", 4, &["Z  ", "ZZ ", "ZZZ"])];

// tool and armor shapes shared by every material
const PICKAXE: [&str; 3] = ["ZZZ", " Y ", " Y "];
const AXE: [&str; 3] = ["ZZ", "ZY", " Y"];
const SHOVEL: [&str; 3] = ["Z", "Y", "Y"];
const SWORD: [&str; 3] = ["Z", "Z", "Y"];
const TOOL_SHAPES: [(&str, [&str; 3]); 4] = [("pickaxe", PICKAXE), ("axe", AXE), ("shovel", SHOVEL), ("sword", SWORD)];
const TOOL_MATERIALS: [(&str, &str); 4] = [
    ("skyroot", PLANKS),
    ("holystone", block::HOLYSTONE),
    ("zanite", item::ZANITE_GEMSTONE),
    ("gravitite", ENCHANTED_GRAVITITE),
];
const ARMOR_SHAPES: [(&str, &[&str]); 4] = [
    ("helmet", &["XXX", "X X"]),
    ("chestplate", &["X X", "XXX", "XXX"]),
    ("leggings", &["XXX", "X X", "X X"]),
    ("boots", &["X X", "X X"]),
];
const ARMOR_MATERIALS: [(&str, &str); 2] = [("zanite", item::ZANITE_GEMSTONE), ("gravitite", ENCHANTED_GRAVITITE)];

// vanilla dye metadata for purple
const PURPLE_DYE: i32 = 5;
const PURPLE_DYE_COUNT: i32 = 2;
const PLANKS_PER_LOG: i32 = 4;
// furnace output: charcoal is coal metadata 1
const CHARCOAL: i32 = 1;

fn build(ctx: &mut RegisterCtx, name: &str, result: ItemStack, pattern: &[&str], keys: &[Key]) -> Result<()>
{
    let mut builder: ShapedBuilder = ctx.recipes().shaped(name, result).rows(pattern);
    for (symbol, ingredient, meta) in keys
    {
        builder = builder.keyStack(*symbol, ItemStack::of(*ingredient, 1).withDamage(*meta));
    }
    builder.build()
}

fn families(ctx: &mut RegisterCtx) -> Result<()>
{
    for (prefix, source, walls) in FAMILIES
    {
        for (shape, count, pattern) in FAMILY_SHAPES
        {
            if shape == "wall" && !walls
            {
                continue;
            }
            let name: String = format!("{prefix}_{shape}");
            build(ctx, &name, ItemStack::of(name.as_str(), count), pattern, &[('Z', source, 0)])?;
        }
    }
    Ok(())
}

fn gear(ctx: &mut RegisterCtx) -> Result<()>
{
    for (material, ingredient) in TOOL_MATERIALS
    {
        for (kind, pattern) in TOOL_SHAPES
        {
            let name: String = format!("{material}_{kind}");
            build(ctx, &name, ItemStack::of(name.as_str(), 1), &pattern, &[('Z', ingredient, 0), STICK])?;
        }
    }
    for (material, ingredient) in ARMOR_MATERIALS
    {
        for (piece, pattern) in ARMOR_SHAPES
        {
            let name: String = format!("{material}_{piece}");
            build(ctx, &name, ItemStack::of(name.as_str(), 1), pattern, &[('X', ingredient, 0)])?;
        }
    }
    Ok(())
}

pub fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for row in &SHAPED
    {
        build(ctx, row.m_name, ItemStack::of(row.m_result, row.m_count), row.m_pattern, row.m_keys)?;
    }
    for row in &SHAPELESS
    {
        let ingredients: Vec<ItemStack> = row.m_ingredients.iter().map(|name| ItemStack::of(*name, 1)).collect();
        ctx.recipes().shapeless(row.m_name, ItemStack::of(row.m_result, 1), &ingredients)?;
    }
    families(ctx)?;
    gear(ctx)?;
    for log in [block::SKYROOT_LOG, block::GOLDEN_OAK_LOG]
    {
        ctx.recipes().shapeless(
            &format!("planks_from_{log}"),
            ItemStack::of(PLANKS, PLANKS_PER_LOG),
            &[ItemStack::of(log, 1)],
        )?;
        ctx.recipes().smelting(
            &format!("charcoal_from_{log}"),
            log,
            ItemStack::of("minecraft:coal", 1).withDamage(CHARCOAL),
        )?;
    }
    ctx.recipes().shapeless(
        "purple_dye",
        ItemStack::of("minecraft:dye", PURPLE_DYE_COUNT).withDamage(PURPLE_DYE),
        &[ItemStack::of(block::PURPLE_FLOWER, 1)],
    )?;
    Ok(())
}
