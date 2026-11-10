use crate::ids::block;

// a 1.7 block the structure code names, resolved to a beta or port block and its metadata
pub type Resolved = (&'static str, i32);

// 1.7 blocks beta has under the same name and metadata
const SAME: &[&str] = &[
    "air",
    "stone",
    "grass",
    "dirt",
    "cobblestone",
    "planks",
    "sapling",
    "bedrock",
    "flowing_water",
    "water",
    "flowing_lava",
    "lava",
    "sand",
    "gravel",
    "gold_ore",
    "iron_ore",
    "coal_ore",
    "sponge",
    "glass",
    "lapis_ore",
    "lapis_block",
    "dispenser",
    "sandstone",
    "sticky_piston",
    "web",
    "tallgrass",
    "deadbush",
    "piston",
    "wool",
    "yellow_flower",
    "brown_mushroom",
    "red_mushroom",
    "gold_block",
    "iron_block",
    "brick_block",
    "tnt",
    "bookshelf",
    "mossy_cobblestone",
    "obsidian",
    "torch",
    "fire",
    "mob_spawner",
    "chest",
    "redstone_wire",
    "diamond_ore",
    "diamond_block",
    "crafting_table",
    "farmland",
    "furnace",
    "standing_sign",
    "wooden_door",
    "ladder",
    "lever",
    "stone_pressure_plate",
    "iron_door",
    "wooden_pressure_plate",
    "redstone_ore",
    "lit_redstone_ore",
    "unlit_redstone_torch",
    "redstone_torch",
    "stone_button",
    "snow_layer",
    "ice",
    "snow",
    "cactus",
    "clay",
    "fence",
    "pumpkin",
    "netherrack",
    "soul_sand",
    "glowstone",
    "lit_pumpkin",
    "unpowered_repeater",
    "trapdoor",
    "jukebox",
];

// 1.7 and TF names the structures place, with beta stand ins where beta lacks the block
pub fn resolve(name: &'static str, meta: i32) -> Resolved
{
    if let Some(tf) = name.strip_prefix("tf:")
    {
        return twilight(tf, meta);
    }
    if SAME.contains(&name)
    {
        return (name, meta);
    }
    match name
    {
        "log" | "log2" => ("log", if name == "log" { (meta & 3) % 3 } else { 0 }),
        "leaves" | "leaves2" => ("leaves", if name == "leaves" { (meta & 3) % 3 } else { 0 }),
        "red_flower" => ("red_flower", 0),
        "double_plant" => ("tallgrass", 1),
        "stone_slab" => ("stone_slab", slab(meta)),
        "double_stone_slab" => ("double_stone_slab", slab(meta)),
        "wooden_slab" => ("stone_slab", 2),
        "double_wooden_slab" => ("double_stone_slab", 2),
        "oak_stairs" | "spruce_stairs" | "birch_stairs" | "jungle_stairs" | "acacia_stairs" => ("oak_stairs", meta & 3),
        "stone_stairs"
        | "stone_brick_stairs"
        | "brick_stairs"
        | "nether_brick_stairs"
        | "sandstone_stairs"
        | "quartz_stairs" => ("stone_stairs", meta & 3),
        "stonebrick" => stoneBrick(meta),
        "monster_egg" => match meta
        {
            1 => ("cobblestone", 0),
            2..=5 => stoneBrick(meta - 2),
            _ => ("stone", 0),
        },
        "iron_bars" | "nether_brick_fence" | "fence_gate" => ("fence", 0),
        "glass_pane" | "stained_glass" | "stained_glass_pane" => ("glass", 0),
        "cobblestone_wall" => ("cobblestone", 0),
        "trapped_chest" | "ender_chest" => ("chest", 0),
        "redstone_lamp" | "lit_redstone_lamp" => ("glowstone", 0),
        "quartz_block" | "quartz_ore" | "end_stone" => ("sandstone", 0),
        "emerald_block" => ("diamond_block", 0),
        "emerald_ore" => ("diamond_ore", 0),
        "packed_ice" => ("ice", 0),
        "mycelium" => ("grass", 0),
        "nether_brick" => ("netherrack", 0),
        "anvil" => ("iron_block", 0),
        "enchanting_table" | "end_portal_frame" | "coal_block" => ("obsidian", 0),
        "redstone_block" => ("lit_redstone_ore", 0),
        "stained_hardened_clay" => ("wool", meta),
        "hardened_clay" => ("wool", 12),
        "hay_block" => ("wool", 4),
        "melon_block" => ("pumpkin", 0),
        "red_mushroom_block" => (block::HUGE_RED_MUSHROOM, meta),
        "brown_mushroom_block" => (block::HUGE_BROWN_MUSHROOM, meta),
        _ => ("air", 0),
    }
}

// beta slabs are stone sandstone wood and cobblestone, with no top halves
fn slab(meta: i32) -> i32
{
    match meta & 7
    {
        value @ 0..=3 => value,
        6 => 3,
        _ => 0,
    }
}

// beta has no stone bricks, so plain mossy cracked and chiseled fall back to stone mossy cobblestone and cobblestone
fn stoneBrick(meta: i32) -> Resolved
{
    match meta
    {
        1 => ("mossy_cobblestone", 0),
        2 => ("cobblestone", 0),
        _ => ("stone", 0),
    }
}

// TFBlocks fields by their metadata, the port splits most metadata variants into named blocks
fn twilight(field: &str, meta: i32) -> Resolved
{
    match field
    {
        "nagastone" => match meta
        {
            0..=3 => (block::NAGASTONE_HEAD, crate::blocks::headMeta(meta)),
            _ => (block::NAGASTONE, crate::blocks::bodyMeta(meta)),
        },
        "mazestone" => (
            [
                block::MAZESTONE,
                block::MAZESTONE_BRICK,
                "chiseled_mazestone",
                "decorative_mazestone",
                block::CRACKED_MAZESTONE,
                block::MOSSY_MAZESTONE,
                block::MAZESTONE_MOSAIC,
                block::MAZESTONE_BORDER,
            ][(meta & 7) as usize],
            0,
        ),
        "towerWood" => (
            [
                block::TOWERWOOD,
                block::ENCASED_TOWERWOOD,
                block::CRACKED_TOWERWOOD,
                block::MOSSY_TOWERWOOD,
                block::INFESTED_TOWERWOOD,
            ][(meta.clamp(0, 4)) as usize],
            0,
        ),
        "towerDevice" => device(meta),
        "log" =>
        {
            ([block::OAK_LOG, block::CANOPY_LOG, block::MANGROVE_LOG, block::DARKWOOD_LOG][(meta & 3) as usize], 0)
        }
        "leaves" => (
            [block::OAK_LEAVES, block::CANOPY_LEAVES, block::MANGROVE_LEAVES, block::RAINBOW_LEAVES]
                [(meta & 3) as usize],
            0,
        ),
        "thorns" => (
            if meta & 3 == 1
            {
                block::GREEN_THORNS
            }
            else
            {
                block::BROWN_THORNS
            },
            thornAxis(meta),
        ),
        "castleMagic" => (block::CASTLE_RUNE_BRICK, meta),
        "castleDoor" => (block::CASTLE_DOOR, meta),
        "castleBlock" => (
            [block::CASTLE_BRICK, block::WORN_CASTLE_BRICK, block::CRACKED_CASTLE_BRICK, block::CASTLE_ROOF]
                [(meta & 3) as usize],
            0,
        ),
        "underBrick" => (
            [block::KNIGHTBRICK, block::MOSSY_KNIGHTBRICK, block::CRACKED_KNIGHTBRICK, "underbrick_floor"]
                [(meta & 3) as usize],
            0,
        ),
        "deadrock" =>
        {
            ([block::DEADROCK, block::CRACKED_DEADROCK, block::SOLID_DEADROCK][(meta.clamp(0, 2)) as usize], 0)
        }
        "forceField" => (
            ["purple_force_field", "pink_force_field", "orange_force_field", "green_force_field", "blue_force_field"]
                [(meta.clamp(0, 4)) as usize],
            0,
        ),
        "root" => (
            if meta == 1
            {
                block::LIVEROOT
            }
            else
            {
                block::ROOT
            },
            0,
        ),
        "plant" => plant(meta),
        "giantLeaves" => (block::GIANT_LEAVES, 0),
        "giantLog" => (block::GIANT_LOG, 0),
        "giantCobble" => (block::GIANT_COBBLE, 0),
        "giantObsidian" => (block::GIANT_OBSIDIAN, 0),
        "firefly" => (block::FIREFLY, meta),
        "cicada" => (block::CICADA, meta),
        "bossSpawner" => (crate::bosses::SPAWNER, meta),
        "trollSteinn" => (block::TROLLSTEINN, 0),
        "fluffyCloud" => (block::FLUFFY_CLOUD, 0),
        // 15 is the latent shield a trophy pedestal clears
        "shield" => (
            block::SHIELD,
            if meta > 5
            {
                crate::blocks::SHIELD_LATENT
            }
            else
            {
                meta
            },
        ),
        "auroraBlock" => (block::AURORA_BRICK, meta),
        "auroraPillar" => (block::AURORA_PILLAR, meta),
        "uberousSoil" => (block::UBEROUS_SOIL, 0),
        "trophyPedestal" => (block::TROPHY_PEDESTAL, meta),
        "hugeGloomBlock" => (block::HUGE_GLOOM, meta),
        "hedge" => (block::HEDGE, 0),
        _ => ("air", 0),
    }
}

// BlockTFThorns keeps its axis in the top two bits, the port in metadata 0 1 2
fn thornAxis(meta: i32) -> i32
{
    match meta & 12
    {
        4 => 1,
        8 => 2,
        _ => 0,
    }
}

// BlockTFTowerDevice metadata to the port's device blocks
fn device(meta: i32) -> Resolved
{
    match meta
    {
        0 => (block::REAPPEARING, 0),
        1 => ("reappearing_block_active", 0),
        2 => (block::VANISHING, 0),
        3 => ("vanishing_block_active", 0),
        4 => (block::LOCKED_VANISHING, 0),
        5 => (block::VANISHING, 0),
        6 => (block::BUILDER, 0),
        7 => ("carminite_builder_active", 0),
        8 => ("carminite_builder_timeout", 0),
        9 => (block::ANTIBUILDER, 0),
        10 => (block::GHAST_TRAP, 0),
        11 => ("ghast_trap_active", 0),
        12 => (block::REACTOR, 0),
        _ => ("carminite_reactor_active", 0),
    }
}

// BlockTFPlant metadata to the port's plants
fn plant(meta: i32) -> Resolved
{
    match meta
    {
        3 => ("moss_patch", 0),
        4 => ("mayapple", 0),
        5 => ("clover_patch", 0),
        8 => (block::FIDDLEHEAD, 0),
        9 => (block::MUSHGLOOM, 0),
        11 => (block::DRIED_BUSH, 0),
        13 => (block::TORCHBERRY_PLANT, 0),
        14 => (block::ROOT_STRAND, 0),
        _ => (block::FOREST_GRASS, 0),
    }
}

// TFCreatures.getSpawnerNameFor and the vanilla names structures give their spawners
pub fn mob(name: &str) -> &'static str
{
    match name
    {
        "Spider" | "CaveSpider" | "Silverfish" => "Spider",
        "Zombie" => "Zombie",
        "Skeleton" => "Skeleton",
        "Creeper" => "Creeper",
        "Blaze" => "PigZombie",
        "Redscale Broodling" => "twilightforest:tower_broodling",
        "Swarm Spider" => "twilightforest:swarm_spider",
        "Hedge Spider" => "twilightforest:hedge_spider",
        "Helmet Crab" => "twilightforest:helmet_crab",
        "Hostile Wolf" => "twilightforest:hostile_wolf",
        "Mini Ghast" => "twilightforest:mini_ghast",
        "Minotaur" => "twilightforest:minotaur",
        "Pinch Beetle" => "twilightforest:pinch_beetle",
        "Fire Beetle" => "twilightforest:fire_beetle",
        "Slime Beetle" => "twilightforest:slime_beetle",
        "Redcap" => "twilightforest:redcap",
        "Tower Golem" => "twilightforest:tower_golem",
        "Twilight Wraith" => "twilightforest:twilight_wraith",
        "Skeleton Druid" => "twilightforest:skeleton_druid",
        "King Spider" => "twilightforest:king_spider",
        "Kobold" => "twilightforest:kobold",
        "Wild Boar" => "twilightforest:wild_boar",
        _ => "Pig",
    }
}
