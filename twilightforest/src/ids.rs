use b173_api::prelude::*;

// registry names, shared by registration, generation and behaviors
pub mod block
{
    pub const KNIGHTMETAL_BLOCK: &str = "knightmetal_block";
    pub const AURORA_PILLAR: &str = "aurora_pillar";
    pub const GIANT_COBBLE: &str = "giant_cobblestone";
    pub const GIANT_LOG: &str = "giant_log";
    pub const GIANT_LEAVES: &str = "giant_leaves";
    pub const GIANT_OBSIDIAN: &str = "giant_obsidian";
    pub const SLIDER: &str = "slider";
    pub const ANTIBUILDER: &str = "antibuilder";
    pub const ANTIBUILT: &str = "antibuilt_block";
    pub const AURORA_BRICK: &str = "aurora_brick";
    pub const AURORA_DOUBLE_SLAB: &str = "aurora_double_slab";
    pub const AURORA_SLAB: &str = "aurora_slab";
    pub const BEANSTALK_LEAVES: &str = "beanstalk_leaves";
    pub const BROWN_THORNS: &str = "brown_thorns";
    pub const BUILDER: &str = "carminite_builder";
    pub const BUILT: &str = "built_block";
    pub const BURNT_THORNS: &str = "burnt_thorns";
    pub const CANOPY_LEAVES: &str = "canopy_leaves";
    pub const CANOPY_LOG: &str = "canopy_log";
    pub const CASTLE_BRICK: &str = "castle_brick";
    pub const CASTLE_DOOR: &str = "castle_door";
    pub const CASTLE_DOOR_VANISHED: &str = "castle_door_vanished";
    pub const CASTLE_ROOF: &str = "castle_roof";
    pub const CASTLE_RUNE_BRICK: &str = "castle_rune_brick";
    pub const CICADA: &str = "cicada";
    pub const CINDER_LOG: &str = "cinder_log";
    pub const CRACKED_CASTLE_BRICK: &str = "cracked_castle_brick";
    pub const CRACKED_DEADROCK: &str = "cracked_deadrock";
    pub const CRACKED_KNIGHTBRICK: &str = "cracked_underbrick";
    pub const CRACKED_MAZESTONE: &str = "cracked_mazestone";
    pub const CRACKED_TOWERWOOD: &str = "cracked_towerwood";
    pub const DARK_LEAVES: &str = "darkwood_leaves";
    pub const DARKWOOD_LOG: &str = "darkwood_log";
    pub const DEADROCK: &str = "weathered_deadrock";
    pub const DRIED_BUSH: &str = "dried_bush";
    pub const ENCASED_FIRE_JET: &str = "encased_fire_jet";
    pub const ENCASED_SMOKER: &str = "encased_smoker";
    pub const ENCASED_TOWERWOOD: &str = "encased_towerwood";
    pub const FALSE_DIAMOND: &str = "false_diamond";
    pub const FALSE_GOLD: &str = "false_gold";
    pub const FIDDLEHEAD: &str = "fiddlehead";
    pub const FIREFLY: &str = "firefly";
    pub const FIREFLY_JAR: &str = "firefly_jar";
    pub const FIRE_JET: &str = "fire_jet";
    pub const FLUFFY_CLOUD: &str = "fluffy_cloud";
    pub const FOREST_GRASS: &str = "forest_grass";
    pub const GHAST_TRAP: &str = "ghast_trap";
    pub const GREEN_THORNS: &str = "green_thorns";
    pub const HEDGE: &str = "hedge";
    pub const HUGE_GLOOM: &str = "huge_mushgloom";
    pub const HUGE_BROWN_MUSHROOM: &str = "huge_brown_mushroom";
    pub const HUGE_RED_MUSHROOM: &str = "huge_red_mushroom";
    pub const HUGE_LILY_PAD: &str = "huge_lily_pad";
    pub const HUGE_STALK: &str = "huge_stalk";
    pub const HUGE_WATER_LILY: &str = "huge_water_lily";
    pub const INFESTED_TOWERWOOD: &str = "infested_towerwood";
    pub const KNIGHTBRICK: &str = "underbrick";
    pub const LIVEROOT: &str = "liveroot_block";
    pub const LOCKED_VANISHING: &str = "locked_vanishing_block";
    pub const MANGROVE_LEAVES: &str = "mangrove_leaves";
    pub const MANGROVE_LOG: &str = "mangrove_log";
    pub const MAZESTONE: &str = "mazestone";
    pub const MAZESTONE_BORDER: &str = "mazestone_border";
    pub const MAZESTONE_BRICK: &str = "mazestone_brick";
    pub const MAZESTONE_MOSAIC: &str = "mazestone_mosaic";
    pub const MINE_CORE: &str = "minewood_core";
    pub const MINE_LEAVES: &str = "minewood_leaves";
    pub const MINEWOOD: &str = "minewood";
    pub const MOONWORM: &str = "moonworm";
    pub const MOSSY_KNIGHTBRICK: &str = "mossy_underbrick";
    pub const MOSSY_MAZESTONE: &str = "mossy_mazestone";
    pub const MOSSY_TOWERWOOD: &str = "mossy_towerwood";
    pub const MUSHGLOOM: &str = "mushgloom";
    pub const NAGASTONE: &str = "nagastone";
    pub const PORTAL: &str = "portal";
    pub const NAGASTONE_HEAD: &str = "nagastone_head";
    pub const OAK_LEAVES: &str = "twilight_oak_leaves";
    pub const OAK_LOG: &str = "twilight_oak_log";
    pub const OAK_SAPLING: &str = "twilight_oak_sapling";
    pub const RAINBOW_LEAVES: &str = "rainbow_oak_leaves";
    pub const REACTOR: &str = "carminite_reactor";
    pub const REACTOR_DEBRIS: &str = "reactor_debris";
    pub const REAPPEARING: &str = "reappearing_block";
    pub const ROOT: &str = "root";
    pub const ROOT_STRAND: &str = "root_strand";
    pub const SHIELD: &str = "shield";
    pub const SMOKER: &str = "smoker";
    pub const SOLID_DEADROCK: &str = "deadrock";
    pub const SORT_CORE: &str = "sorting_engine";
    pub const SORTINGWOOD: &str = "sortingwood";
    pub const SORT_LEAVES: &str = "sortingwood_leaves";
    pub const THORN_LEAVES: &str = "thorn_leaves";
    pub const THORN_ROSE: &str = "thorn_rose";
    pub const TIME_CORE: &str = "timewood_clock";
    pub const TIME_LEAVES: &str = "timewood_leaves";
    pub const TIMEWOOD: &str = "timewood";
    pub const TORCHBERRY_PLANT: &str = "torchberry_plant";
    pub const TOWERWOOD: &str = "towerwood";
    pub const TRANS_CORE: &str = "transformation_heart";
    pub const TRANS_LEAVES: &str = "transwood_leaves";
    pub const TRANSWOOD: &str = "transwood";
    pub const TROLLBER: &str = "trollber";
    pub const TROLL_ROOT: &str = "troll_root";
    pub const TROLLSTEINN: &str = "trollsteinn";
    pub const TROPHY_PEDESTAL: &str = "trophy_pedestal";
    pub const UBEROUS_SOIL: &str = "uberous_soil";
    pub const UNRIPE_TROLLBER: &str = "unripe_trollber";
    pub const VANISHING: &str = "vanishing_block";
    pub const WISPY_CLOUD: &str = "wispy_cloud";
    pub const WORN_CASTLE_BRICK: &str = "worn_castle_brick";
}
pub mod item
{
    pub const ADHERENT_FRAGMENT: &str = "adherent_fragment";
    pub const ALPHA_FUR: &str = "alpha_fur";
    pub const ARCTIC_FUR: &str = "arctic_fur";
    pub const ARMOR_SHARD: &str = "armor_shard";
    pub const BORER_ESSENCE: &str = "borer_essence";
    pub const CARMINITE: &str = "carminite";
    pub const CHARM_OF_KEEPING_1: &str = "charm_of_keeping_1";
    pub const CHARM_OF_KEEPING_2: &str = "charm_of_keeping_2";
    pub const CHARM_OF_KEEPING_3: &str = "charm_of_keeping_3";
    pub const CHARM_OF_LIFE_1: &str = "charm_of_life_1";
    pub const CHARM_OF_LIFE_2: &str = "charm_of_life_2";
    pub const CUBE_TALISMAN: &str = "cube_talisman";
    pub const EXPERIMENT_115: &str = "experiment_115";
    pub const FIERY_BLOOD: &str = "fiery_blood";
    pub const FIERY_INGOT: &str = "fiery_ingot";
    pub const FIERY_PICK: &str = "fiery_pickaxe";
    pub const FIERY_SWORD: &str = "fiery_sword";
    pub const FIERY_TEARS: &str = "fiery_tears";
    pub const GIANT_PICK: &str = "giant_pickaxe";
    pub const GIANT_SWORD: &str = "giant_sword";
    pub const GLASS_SWORD: &str = "glass_sword";
    pub const HARBINGER_FRAGMENT: &str = "harbinger_fragment";
    pub const HYDRA_CHOP: &str = "hydra_chop";
    pub const ICE_SWORD: &str = "ice_sword";
    pub const IRONWOOD_INGOT: &str = "ironwood_ingot";
    pub const IRONWOOD_PICK: &str = "ironwood_pickaxe";
    pub const IRONWOOD_SWORD: &str = "ironwood_sword";
    pub const KNIGHTLY_AXE: &str = "knightly_axe";
    pub const KNIGHTLY_PICK: &str = "knightly_pickaxe";
    pub const KNIGHTLY_SWORD: &str = "knightly_sword";
    pub const KNIGHTMETAL: &str = "knightmetal";
    pub const KNIGHTMETAL_RING: &str = "knightmetal_ring";
    pub const LIVEROOT: &str = "liveroot";
    pub const MAGIC_MAP_FOCUS: &str = "magic_map_focus";
    pub const MAZEBREAKER_PICK: &str = "mazebreaker_pickaxe";
    pub const MAZE_MAP_FOCUS: &str = "maze_map_focus";
    pub const MAZE_WAFER: &str = "maze_wafer";
    pub const MEEF_STEAK: &str = "meef_steak";
    pub const MEEF_STROGANOFF: &str = "meef_stroganoff";
    pub const MINOTAUR_AXE: &str = "minotaur_axe";
    pub const NAGA_SCALE: &str = "naga_scale";
    pub const RAVEN_FEATHER: &str = "raven_feather";
    pub const RAW_IRONWOOD: &str = "raw_ironwood";
    pub const RAW_MEEF: &str = "raw_meef";
    pub const RAW_VENISON: &str = "raw_venison";
    pub const SHARD_CLUSTER: &str = "shard_cluster";
    pub const STEELEAF: &str = "steeleaf_ingot";
    pub const TORCHBERRIES: &str = "torchberries";
    pub const TRANSFORM_POWDER: &str = "transform_powder";
    pub const VENISON: &str = "venison_steak";
    pub const SCEPTER_TWILIGHT: &str = "twilight_scepter";
    pub const SCEPTER_LIFE_DRAIN: &str = "lifedrain_scepter";
    pub const SCEPTER_ZOMBIE: &str = "zombie_scepter";
    pub const HYDRA_TROPHY: &str = "hydra_trophy";
    pub const NAGA_TROPHY: &str = "naga_trophy";
    pub const LICH_TROPHY: &str = "lich_trophy";
    pub const UR_GHAST_TROPHY: &str = "ur_ghast_trophy";
    pub const SNOW_QUEEN_TROPHY: &str = "snow_queen_trophy";
    pub const TRIPLE_BOW: &str = "triple_bow";
    pub const SEEKER_BOW: &str = "seeker_bow";
    pub const ICE_BOW: &str = "ice_bow";
    pub const ENDER_BOW: &str = "ender_bow";
    pub const MOONWORM_QUEEN: &str = "moonworm_queen";
    pub const CHAIN_BLOCK: &str = "chain_block";
    pub const CUBE_OF_ANNIHILATION: &str = "cube_of_annihilation";
    pub const MAGIC_MAP: &str = "magic_map";
    pub const EMPTY_MAGIC_MAP: &str = "empty_magic_map";
    pub const MAZE_MAP: &str = "maze_map";
    pub const EMPTY_MAZE_MAP: &str = "empty_maze_map";
    pub const ORE_MAP: &str = "ore_map";
    pub const EMPTY_ORE_MAP: &str = "empty_ore_map";
    pub const PHANTOM_HELM: &str = "phantom_helmet";
    pub const PHANTOM_PLATE: &str = "phantom_chestplate";
    pub const YETI_HELM: &str = "yeti_helmet";
    pub const YETI_PLATE: &str = "yeti_chestplate";
    pub const YETI_LEGS: &str = "yeti_leggings";
    pub const YETI_BOOTS: &str = "yeti_boots";
    pub const ORE_METER: &str = "ore_meter";
    pub const ORE_MAGNET: &str = "ore_magnet";
    pub const CRUMBLE_HORN: &str = "crumble_horn";
    pub const PEACOCK_FAN: &str = "peacock_fan";
    pub const LAMP_OF_CINDERS: &str = "lamp_of_cinders";
    pub const MAGIC_BEANS: &str = "magic_beans";
    pub const TOWER_KEY: &str = "tower_key";
}

pub fn blockId(name: &str) -> i32 { b173_api::Blocks.id(name).unwrap_or(0) }

pub fn itemId(name: &str) -> i32 { b173_api::Items.get(name).and_then(|item| item.id()).unwrap_or(0) }

pub fn cell(at: [f64; 3]) -> BlockPos
{
    BlockPos::new(at[0].floor() as i32, at[1].floor() as i32, at[2].floor() as i32)
}

pub fn center(pos: BlockPos) -> [f64; 3] { [f64::from(pos.x) + 0.5, f64::from(pos.y) + 0.5, f64::from(pos.z) + 0.5] }

// the six neighbours of a block, down up north south west east
pub const NEIGHBORS: [[i32; 3]; 6] = [[0, -1, 0], [0, 1, 0], [0, 0, -1], [0, 0, 1], [-1, 0, 0], [1, 0, 0]];

// what plants and saplings take root in, grass dirt and farmland as in beta
pub fn soil(block: i32) -> bool
{
    ["minecraft:grass", "minecraft:dirt", "minecraft:farmland", block::UBEROUS_SOIL]
        .iter()
        .any(|name| blockId(name) == block)
}

// a random fraction from the world random, the SDK only hands out bounded integers
pub fn fraction(world: World) -> f32
{
    let resolution: i32 = 1 << 16;
    world.random(resolution) as f32 / resolution as f32
}

fn anyOf(block: i32, names: &[&str]) -> bool { names.iter().any(|name| blockId(name) == block) }

// blocks of rock material that troll roots hang from
pub fn stony(block: i32) -> bool
{
    anyOf(
        block,
        &[
            "minecraft:stone",
            "minecraft:cobblestone",
            "minecraft:mossy_cobblestone",
            "minecraft:sandstone",
            "minecraft:coal_ore",
            "minecraft:iron_ore",
            "minecraft:gold_ore",
            "minecraft:diamond_ore",
            "minecraft:lapis_ore",
            "minecraft:redstone_ore",
            "minecraft:obsidian",
            block::DEADROCK,
            block::CRACKED_DEADROCK,
            block::SOLID_DEADROCK,
            block::TROLLSTEINN,
        ],
    )
}

// logs and thorns, BlockLog.canSustainLeaves and the blocks that override it
pub fn holdsLeaves(block: i32) -> bool
{
    anyOf(
        block,
        &[
            "minecraft:log",
            block::OAK_LOG,
            block::CANOPY_LOG,
            block::MANGROVE_LOG,
            block::DARKWOOD_LOG,
            block::TIMEWOOD,
            block::TRANSWOOD,
            block::MINEWOOD,
            block::SORTINGWOOD,
            block::BROWN_THORNS,
            block::GREEN_THORNS,
            block::HUGE_STALK,
        ],
    )
}

// BlockLeaves and the leaves built on it, the ones a broken log tells to decay
pub fn decayingLeaves(block: i32) -> bool
{
    anyOf(
        block,
        &[
            "minecraft:leaves",
            self::block::OAK_LEAVES,
            self::block::CANOPY_LEAVES,
            self::block::MANGROVE_LEAVES,
            self::block::RAINBOW_LEAVES,
            self::block::TIME_LEAVES,
            self::block::TRANS_LEAVES,
            self::block::MINE_LEAVES,
            self::block::SORT_LEAVES,
            self::block::THORN_LEAVES,
            self::block::BEANSTALK_LEAVES,
        ],
    )
}

pub fn giantBlock(block: i32) -> bool
{
    anyOf(
        block,
        &[self::block::GIANT_COBBLE, self::block::GIANT_LOG, self::block::GIANT_LEAVES, self::block::GIANT_OBSIDIAN],
    )
}

pub fn mazestone(block: i32) -> bool
{
    anyOf(
        block,
        &[
            self::block::MAZESTONE,
            self::block::MAZESTONE_BRICK,
            "chiseled_mazestone",
            "decorative_mazestone",
            self::block::CRACKED_MAZESTONE,
            self::block::MOSSY_MAZESTONE,
            self::block::MAZESTONE_MOSAIC,
            self::block::MAZESTONE_BORDER,
        ],
    )
}

// every leaf block the forest grows, so generators treat them as vanilla leaves
pub fn leaves(block: i32) -> bool
{
    anyOf(
        block,
        &[
            "minecraft:leaves",
            self::block::OAK_LEAVES,
            self::block::CANOPY_LEAVES,
            self::block::MANGROVE_LEAVES,
            self::block::RAINBOW_LEAVES,
            self::block::TIME_LEAVES,
            self::block::TRANS_LEAVES,
            self::block::MINE_LEAVES,
            self::block::SORT_LEAVES,
            self::block::DARK_LEAVES,
            self::block::THORN_LEAVES,
            self::block::BEANSTALK_LEAVES,
        ],
    )
}
