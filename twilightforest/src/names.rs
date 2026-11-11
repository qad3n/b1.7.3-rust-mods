use b173_api::prelude::*;
use std::rc::Rc;

// registry names shared by registration generation and behaviors
pub(crate) mod block
{
    pub(crate) const KNIGHTMETAL_BLOCK: &str = "knightmetal_block";
    pub(crate) const AURORA_PILLAR: &str = "aurora_pillar";
    pub(crate) const GIANT_COBBLE: &str = "giant_cobblestone";
    pub(crate) const GIANT_LOG: &str = "giant_log";
    pub(crate) const GIANT_LEAVES: &str = "giant_leaves";
    pub(crate) const GIANT_OBSIDIAN: &str = "giant_obsidian";
    pub(crate) const SLIDER: &str = "slider";
    pub(crate) const ANTIBUILDER: &str = "antibuilder";
    pub(crate) const ANTIBUILT: &str = "antibuilt_block";
    pub(crate) const AURORA_BRICK: &str = "aurora_brick";
    pub(crate) const AURORA_DOUBLE_SLAB: &str = "aurora_double_slab";
    pub(crate) const AURORA_SLAB: &str = "aurora_slab";
    pub(crate) const BEANSTALK_LEAVES: &str = "beanstalk_leaves";
    pub(crate) const BROWN_THORNS: &str = "brown_thorns";
    pub(crate) const BUILDER: &str = "carminite_builder";
    pub(crate) const BUILT: &str = "built_block";
    pub(crate) const BURNT_THORNS: &str = "burnt_thorns";
    pub(crate) const CANOPY_LEAVES: &str = "canopy_leaves";
    pub(crate) const CANOPY_LOG: &str = "canopy_log";
    pub(crate) const CASTLE_BRICK: &str = "castle_brick";
    pub(crate) const CASTLE_DOOR: &str = "castle_door";
    pub(crate) const CASTLE_DOOR_VANISHED: &str = "castle_door_vanished";
    pub(crate) const CASTLE_ROOF: &str = "castle_roof";
    pub(crate) const CASTLE_RUNE_BRICK: &str = "castle_rune_brick";
    pub(crate) const CICADA: &str = "cicada";
    pub(crate) const CINDER_LOG: &str = "cinder_log";
    pub(crate) const CRACKED_CASTLE_BRICK: &str = "cracked_castle_brick";
    pub(crate) const CRACKED_DEADROCK: &str = "cracked_deadrock";
    pub(crate) const CRACKED_KNIGHTBRICK: &str = "cracked_underbrick";
    pub(crate) const CRACKED_MAZESTONE: &str = "cracked_mazestone";
    pub(crate) const CRACKED_TOWERWOOD: &str = "cracked_towerwood";
    pub(crate) const DARK_LEAVES: &str = "darkwood_leaves";
    pub(crate) const DARKWOOD_LOG: &str = "darkwood_log";
    pub(crate) const DEADROCK: &str = "weathered_deadrock";
    pub(crate) const DRIED_BUSH: &str = "dried_bush";
    pub(crate) const ENCASED_FIRE_JET: &str = "encased_fire_jet";
    pub(crate) const ENCASED_SMOKER: &str = "encased_smoker";
    pub(crate) const ENCASED_TOWERWOOD: &str = "encased_towerwood";
    pub(crate) const FALSE_DIAMOND: &str = "false_diamond";
    pub(crate) const FALSE_GOLD: &str = "false_gold";
    pub(crate) const FIDDLEHEAD: &str = "fiddlehead";
    pub(crate) const FIREFLY: &str = "firefly";
    pub(crate) const FIREFLY_JAR: &str = "firefly_jar";
    pub(crate) const FIRE_JET: &str = "fire_jet";
    pub(crate) const FLUFFY_CLOUD: &str = "fluffy_cloud";
    pub(crate) const FOREST_GRASS: &str = "forest_grass";
    pub(crate) const GHAST_TRAP: &str = "ghast_trap";
    pub(crate) const GREEN_THORNS: &str = "green_thorns";
    pub(crate) const HEDGE: &str = "hedge";
    pub(crate) const HUGE_GLOOM: &str = "huge_mushgloom";
    pub(crate) const HUGE_BROWN_MUSHROOM: &str = "huge_brown_mushroom";
    pub(crate) const HUGE_RED_MUSHROOM: &str = "huge_red_mushroom";
    pub(crate) const HUGE_LILY_PAD: &str = "huge_lily_pad";
    pub(crate) const HUGE_STALK: &str = "huge_stalk";
    pub(crate) const HUGE_WATER_LILY: &str = "huge_water_lily";
    pub(crate) const INFESTED_TOWERWOOD: &str = "infested_towerwood";
    pub(crate) const KNIGHTBRICK: &str = "underbrick";
    pub(crate) const LIVEROOT: &str = "liveroot_block";
    pub(crate) const LOCKED_VANISHING: &str = "locked_vanishing_block";
    pub(crate) const MANGROVE_LEAVES: &str = "mangrove_leaves";
    pub(crate) const MANGROVE_LOG: &str = "mangrove_log";
    pub(crate) const MAZESTONE: &str = "mazestone";
    pub(crate) const MAZESTONE_BORDER: &str = "mazestone_border";
    pub(crate) const MAZESTONE_BRICK: &str = "mazestone_brick";
    pub(crate) const MAZESTONE_MOSAIC: &str = "mazestone_mosaic";
    pub(crate) const MINE_CORE: &str = "minewood_core";
    pub(crate) const MINE_LEAVES: &str = "minewood_leaves";
    pub(crate) const MINEWOOD: &str = "minewood";
    pub(crate) const MOONWORM: &str = "moonworm";
    pub(crate) const MOSSY_KNIGHTBRICK: &str = "mossy_underbrick";
    pub(crate) const MOSSY_MAZESTONE: &str = "mossy_mazestone";
    pub(crate) const MOSSY_TOWERWOOD: &str = "mossy_towerwood";
    pub(crate) const MUSHGLOOM: &str = "mushgloom";
    pub(crate) const NAGASTONE: &str = "nagastone";
    pub(crate) const PORTAL: &str = "portal";
    pub(crate) const NAGASTONE_HEAD: &str = "nagastone_head";
    pub(crate) const OAK_LEAVES: &str = "twilight_oak_leaves";
    pub(crate) const OAK_LOG: &str = "twilight_oak_log";
    pub(crate) const OAK_SAPLING: &str = "twilight_oak_sapling";
    pub(crate) const RAINBOW_LEAVES: &str = "rainbow_oak_leaves";
    pub(crate) const REACTOR: &str = "carminite_reactor";
    pub(crate) const REACTOR_DEBRIS: &str = "reactor_debris";
    pub(crate) const REAPPEARING: &str = "reappearing_block";
    pub(crate) const ROOT: &str = "root";
    pub(crate) const ROOT_STRAND: &str = "root_strand";
    pub(crate) const SHIELD: &str = "shield";
    pub(crate) const SMOKER: &str = "smoker";
    pub(crate) const SOLID_DEADROCK: &str = "deadrock";
    pub(crate) const SORT_CORE: &str = "sorting_engine";
    pub(crate) const SORTINGWOOD: &str = "sortingwood";
    pub(crate) const SORT_LEAVES: &str = "sortingwood_leaves";
    pub(crate) const THORN_LEAVES: &str = "thorn_leaves";
    pub(crate) const THORN_ROSE: &str = "thorn_rose";
    pub(crate) const TIME_CORE: &str = "timewood_clock";
    pub(crate) const TIME_LEAVES: &str = "timewood_leaves";
    pub(crate) const TIMEWOOD: &str = "timewood";
    pub(crate) const TORCHBERRY_PLANT: &str = "torchberry_plant";
    pub(crate) const TOWERWOOD: &str = "towerwood";
    pub(crate) const TRANS_CORE: &str = "transformation_heart";
    pub(crate) const TRANS_LEAVES: &str = "transwood_leaves";
    pub(crate) const TRANSWOOD: &str = "transwood";
    pub(crate) const TROLLBER: &str = "trollber";
    pub(crate) const TROLL_ROOT: &str = "troll_root";
    pub(crate) const TROLLSTEINN: &str = "trollsteinn";
    pub(crate) const TROPHY_PEDESTAL: &str = "trophy_pedestal";
    pub(crate) const UBEROUS_SOIL: &str = "uberous_soil";
    pub(crate) const UNRIPE_TROLLBER: &str = "unripe_trollber";
    pub(crate) const VANISHING: &str = "vanishing_block";
    pub(crate) const WISPY_CLOUD: &str = "wispy_cloud";
    pub(crate) const WORN_CASTLE_BRICK: &str = "worn_castle_brick";
}

pub(crate) mod item
{
    pub(crate) const ADHERENT_FRAGMENT: &str = "adherent_fragment";
    pub(crate) const ALPHA_FUR: &str = "alpha_fur";
    pub(crate) const ARCTIC_FUR: &str = "arctic_fur";
    pub(crate) const ARMOR_SHARD: &str = "armor_shard";
    pub(crate) const BORER_ESSENCE: &str = "borer_essence";
    pub(crate) const CARMINITE: &str = "carminite";
    pub(crate) const CHARM_OF_KEEPING_1: &str = "charm_of_keeping_1";
    pub(crate) const CHARM_OF_KEEPING_2: &str = "charm_of_keeping_2";
    pub(crate) const CHARM_OF_KEEPING_3: &str = "charm_of_keeping_3";
    pub(crate) const CHARM_OF_LIFE_1: &str = "charm_of_life_1";
    pub(crate) const CHARM_OF_LIFE_2: &str = "charm_of_life_2";
    pub(crate) const CUBE_TALISMAN: &str = "cube_talisman";
    pub(crate) const EXPERIMENT_115: &str = "experiment_115";
    pub(crate) const FIERY_BLOOD: &str = "fiery_blood";
    pub(crate) const FIERY_INGOT: &str = "fiery_ingot";
    pub(crate) const FIERY_PICK: &str = "fiery_pickaxe";
    pub(crate) const FIERY_SWORD: &str = "fiery_sword";
    pub(crate) const FIERY_TEARS: &str = "fiery_tears";
    pub(crate) const GIANT_PICK: &str = "giant_pickaxe";
    pub(crate) const GIANT_SWORD: &str = "giant_sword";
    pub(crate) const GLASS_SWORD: &str = "glass_sword";
    pub(crate) const HARBINGER_FRAGMENT: &str = "harbinger_fragment";
    pub(crate) const HYDRA_CHOP: &str = "hydra_chop";
    pub(crate) const ICE_SWORD: &str = "ice_sword";
    pub(crate) const IRONWOOD_INGOT: &str = "ironwood_ingot";
    pub(crate) const IRONWOOD_PICK: &str = "ironwood_pickaxe";
    pub(crate) const IRONWOOD_SWORD: &str = "ironwood_sword";
    pub(crate) const KNIGHTLY_AXE: &str = "knightly_axe";
    pub(crate) const KNIGHTLY_PICK: &str = "knightly_pickaxe";
    pub(crate) const KNIGHTLY_SWORD: &str = "knightly_sword";
    pub(crate) const KNIGHTMETAL: &str = "knightmetal";
    pub(crate) const KNIGHTMETAL_RING: &str = "knightmetal_ring";
    pub(crate) const LIVEROOT: &str = "liveroot";
    pub(crate) const MAGIC_MAP_FOCUS: &str = "magic_map_focus";
    pub(crate) const MAZEBREAKER_PICK: &str = "mazebreaker_pickaxe";
    pub(crate) const MAZE_MAP_FOCUS: &str = "maze_map_focus";
    pub(crate) const MAZE_WAFER: &str = "maze_wafer";
    pub(crate) const MEEF_STEAK: &str = "meef_steak";
    pub(crate) const MEEF_STROGANOFF: &str = "meef_stroganoff";
    pub(crate) const MINOTAUR_AXE: &str = "minotaur_axe";
    pub(crate) const NAGA_SCALE: &str = "naga_scale";
    pub(crate) const RAVEN_FEATHER: &str = "raven_feather";
    pub(crate) const RAW_IRONWOOD: &str = "raw_ironwood";
    pub(crate) const RAW_MEEF: &str = "raw_meef";
    pub(crate) const RAW_VENISON: &str = "raw_venison";
    pub(crate) const SHARD_CLUSTER: &str = "shard_cluster";
    pub(crate) const STEELEAF: &str = "steeleaf_ingot";
    pub(crate) const TORCHBERRIES: &str = "torchberries";
    pub(crate) const TRANSFORM_POWDER: &str = "transform_powder";
    pub(crate) const VENISON: &str = "venison_steak";
    pub(crate) const SCEPTER_TWILIGHT: &str = "twilight_scepter";
    pub(crate) const SCEPTER_LIFE_DRAIN: &str = "lifedrain_scepter";
    pub(crate) const SCEPTER_ZOMBIE: &str = "zombie_scepter";
    pub(crate) const HYDRA_TROPHY: &str = "hydra_trophy";
    pub(crate) const NAGA_TROPHY: &str = "naga_trophy";
    pub(crate) const LICH_TROPHY: &str = "lich_trophy";
    pub(crate) const UR_GHAST_TROPHY: &str = "ur_ghast_trophy";
    pub(crate) const SNOW_QUEEN_TROPHY: &str = "snow_queen_trophy";
    pub(crate) const TRIPLE_BOW: &str = "triple_bow";
    pub(crate) const SEEKER_BOW: &str = "seeker_bow";
    pub(crate) const ICE_BOW: &str = "ice_bow";
    pub(crate) const ENDER_BOW: &str = "ender_bow";
    pub(crate) const MOONWORM_QUEEN: &str = "moonworm_queen";
    pub(crate) const CHAIN_BLOCK: &str = "chain_block";
    pub(crate) const CUBE_OF_ANNIHILATION: &str = "cube_of_annihilation";
    pub(crate) const MAGIC_MAP: &str = "magic_map";
    pub(crate) const EMPTY_MAGIC_MAP: &str = "empty_magic_map";
    pub(crate) const MAZE_MAP: &str = "maze_map";
    pub(crate) const EMPTY_MAZE_MAP: &str = "empty_maze_map";
    pub(crate) const ORE_MAP: &str = "ore_map";
    pub(crate) const EMPTY_ORE_MAP: &str = "empty_ore_map";
    pub(crate) const PHANTOM_HELM: &str = "phantom_helmet";
    pub(crate) const PHANTOM_PLATE: &str = "phantom_chestplate";
    pub(crate) const YETI_HELM: &str = "yeti_helmet";
    pub(crate) const YETI_PLATE: &str = "yeti_chestplate";
    pub(crate) const YETI_LEGS: &str = "yeti_leggings";
    pub(crate) const YETI_BOOTS: &str = "yeti_boots";
    pub(crate) const ORE_METER: &str = "ore_meter";
    pub(crate) const ORE_MAGNET: &str = "ore_magnet";
    pub(crate) const CRUMBLE_HORN: &str = "crumble_horn";
    pub(crate) const PEACOCK_FAN: &str = "peacock_fan";
    pub(crate) const LAMP_OF_CINDERS: &str = "lamp_of_cinders";
    pub(crate) const MAGIC_BEANS: &str = "magic_beans";
    pub(crate) const TOWER_KEY: &str = "tower_key";
}

// the loader property that carries metadata for blocks without named states, vanilla ones included
pub(crate) const META: &str = "meta";

// a vanilla block with the metadata beta stores for it
pub(crate) fn vanilla(path: &str, meta: i32) -> StateData
{
    StateData::of(Identifier::minecraft(path)).with(META, meta)
}

// a block's default state, a bare name means one of the forest's own
pub(crate) fn state(name: &str) -> StateData
{
    StateData::of(Identifier::parseIn(name, &modId()).expect("block names in code are valid identifiers"))
}

// a true or false property, false when the block has no such state
pub(crate) fn flag(state: Option<Rc<StateData>>, property: &str) -> bool
{
    state.is_some_and(|state| state.flag(property))
}

// a numbered property such as a color or a phase, zero when absent
pub(crate) fn number(state: Option<Rc<StateData>>, property: &str) -> i32
{
    state.and_then(|state| state.get(property)?.parse().ok()).unwrap_or(0)
}

// where a property's value sits in a list, so callers index their own tables
pub(crate) fn pick(state: Option<Rc<StateData>>, property: &str, values: &[&str]) -> Option<usize>
{
    let state: Rc<StateData> = state?;
    let value: &str = state.get(property)?;
    values.iter().position(|known| *known == value)
}

// a still source of a fluid, beta keeps the fluid level in metadata and a source holds zero
pub(crate) fn source(world: World, pos: BlockPos, fluid: &str) -> bool
{
    world.tagged(pos, fluid) && world.stateAt(pos).is_some_and(|state| state.get(META) == Some("0"))
}

// a block's default state with its stored id and metadata resolved once per content epoch
pub(crate) fn handle(name: &str) -> StateHandle { StateHandle::new(state(name)) }
