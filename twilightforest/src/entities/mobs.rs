use super::ai::{self, Ai};
use super::{drop, shots};
use crate::ids::item;
use crate::models::armor::{Piece, Worn};
use crate::models::{self, Program, bipedPose, quadrupedPose};
use crate::world::biomes::Biome;
use b173_api::prelude::*;
use std::rc::Rc;

// how the generated model moves, ModelBiped and ModelQuadruped setRotationAngles or standing still
#[derive(Clone, Copy, PartialEq)]
pub enum Look
{
    Rest,
    Biped,
    Quadruped,
    // ModelTFApocalypseCube keeps the quadruped's legs but stands its body upright
    Upright,
    Spider,
    Wolf,
    Ghast,
    // ModelZombie's level arms on a biped
    Zombie,
    // a model with its own setRotationAngles in models/animated.rs
    Posed(fn(&PoseInput) -> Pose),
}

// where a creature spawns on its own in TFBiomeBase's lists, monsters spawn from the lists in spawns.rs
#[derive(Clone, Copy, PartialEq)]
pub enum Spawning
{
    Never,
    // every twilight biome but the highlands center
    Forest(i32),
    Everywhere(i32),
    Only(i32, &'static [Biome]),
    Monster,
}

pub struct Mob
{
    pub m_name: &'static str,
    pub m_label: &'static str,
    pub m_template: &'static str,
    pub m_texture: &'static str,
    pub m_size: [f32; 2],
    pub m_health: i32,
    // the 1.7 movement speed attribute, beta's moveSpeed scale is applied when registering
    pub m_speed: f32,
    pub m_attack: i32,
    pub m_drops: &'static [(&'static str, [i32; 2])],
    pub m_sounds: [Option<&'static str>; 3],
    pub m_model: fn() -> EntityModelSpec,
    pub m_pivots: &'static [[f32; 3]],
    pub m_rotations: &'static [[f32; 3]],
    pub m_look: Look,
    pub m_shadow: f32,
    pub m_spawn: Spawning,
    pub m_ai: Ai,
    // the renderer's preRenderCallback scale
    pub m_scale: f32,
    // EntityLiving.dropRareDrop, five in two hundred kills by a player
    pub m_rare: Option<&'static str>,
    // the textures a renderer picks by the mob's random type, the bunny and the tiny bird
    pub m_variants: &'static [&'static str],
    // setCurrentItemOrArmor slot 0, what the mob holds in its right hand
    pub m_held: Option<&'static str>,
    // biped parts the Java model's render leaves out
    pub m_hidden: &'static [u16],
    // root parts the Java setRotationAngles turns with the head
    pub m_headed: &'static [u16],
    // setCurrentItemOrArmor slots 1 to 4, drawn by RenderBiped's armor passes
    pub m_armor: &'static [Worn],
}

mod speed
{
    // beta's zombie walks at 0.5 where the 1.7 attribute says 0.23, its pig at 0.7 for 0.25
    pub const MONSTER: f32 = 0.5 / 0.23;
    pub const ANIMAL: f32 = 0.7 / 0.25;
}

const fn tf(path: &'static str) -> &'static str { path }

macro_rules! model
{
    ($module:ident) =>
    {
        (models::$module::model, &models::$module::PIVOTS, &models::$module::ROTATIONS)
    };
}

type Parts = (fn() -> EntityModelSpec, &'static [[f32; 3]], &'static [[f32; 3]]);

#[allow(clippy::too_many_arguments)]
const fn mob(
    name: &'static str,
    label: &'static str,
    template: &'static str,
    texture: &'static str,
    size: [f32; 2],
    health: i32,
    speed: f32,
    attack: i32,
    parts: Parts,
    look: Look,
    spawn: Spawning,
) -> Mob
{
    Mob
    {
        m_name: name,
        m_label: label,
        m_template: template,
        m_texture: texture,
        m_size: size,
        m_health: health,
        m_speed: speed,
        m_attack: attack,
        m_drops: &[],
        m_sounds: [None; 3],
        m_model: parts.0,
        m_pivots: parts.1,
        m_rotations: parts.2,
        m_look: look,
        m_shadow: 0.625,
        m_spawn: spawn,
        m_ai: Ai::Plain,
        m_scale: 1.0,
        m_rare: None,
        m_variants: &[],
        m_held: None,
        m_hidden: &[],
        m_headed: &[],
        m_armor: &[],
    }
}

const NO_DROPS: &[(&str, [i32; 2])] = &[];

const fn drops(mob: Mob, drops: &'static [(&'static str, [i32; 2])]) -> Mob { Mob { m_drops: drops, ..mob } }

const fn voiced(mob: Mob, living: Option<&'static str>, hurt: &'static str, death: &'static str) -> Mob
{
    Mob { m_sounds: [living, Some(hurt), Some(death)], ..mob }
}

const fn shadow(mob: Mob, shadow: f32) -> Mob { Mob { m_shadow: shadow, ..mob } }

const fn acting(mob: Mob, ai: Ai) -> Mob { Mob { m_ai: ai, ..mob } }

const fn scaled(mob: Mob, scale: f32) -> Mob { Mob { m_scale: scale, ..mob } }

const fn rare(mob: Mob, item: &'static str) -> Mob { Mob { m_rare: Some(item), ..mob } }

const RARE_CHANCE: f32 = 5.0 / 200.0;

const fn varied(mob: Mob, variants: &'static [&'static str]) -> Mob { Mob { m_variants: variants, ..mob } }

const fn holding(mob: Mob, item: &'static str) -> Mob { Mob { m_held: Some(item), ..mob } }

const fn hiding(mob: Mob, parts: &'static [u16]) -> Mob { Mob { m_hidden: parts, ..mob } }

const fn headed(mob: Mob, parts: &'static [u16]) -> Mob { Mob { m_headed: parts, ..mob } }

const fn wearing(mob: Mob, armor: &'static [Worn]) -> Mob { Mob { m_armor: armor, ..mob } }

const IRON: &str = "minecraft:armor/iron";

// EntityTFArmoredGiant's iron set
const GIANT_ARMOR: &[Worn] = &[
    Worn { m_piece: Piece::Helmet, m_sheet: IRON },
    Worn { m_piece: Piece::Chestplate, m_sheet: IRON },
    Worn { m_piece: Piece::Leggings, m_sheet: IRON },
    Worn { m_piece: Piece::Boots, m_sheet: IRON },
];
// EntityTFRedcap's iron boots and EntityTFRedcapSapper's ironwood ones
const REDCAP_BOOTS: &[Worn] = &[Worn { m_piece: Piece::Boots, m_sheet: IRON }];
const SAPPER_BOOTS: &[Worn] = &[Worn { m_piece: Piece::Boots, m_sheet: "twilightforest:armor/ironwood" }];

// ModelTFRedcap copies the head's angles onto its ears
const REDCAP_EARS: &[u16] = &[models::tf_redcap::GOBLIN_RIGHT_EAR, models::tf_redcap::GOBLIN_LEFT_EAR];

// RenderTFBunny by getBunnyType, the first two types are both dutch
const BUNNIES: [&str; 4] = ["bunnydutch", "bunnydutch", "bunnywhite", "bunnybrown"];
// RenderTFTinyBird by getBirdType
const BIRDS: [&str; 4] = ["tinybirdbrown", "tinybirdblue", "tinybirdred", "tinybirdgold"];

// the random type EntityTFBunny and EntityTFTinyBird roll when they spawn, mirrored for the renderer
#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct Variant
{
    m_rolled: bool,
    m_index: u8,
}

const FIREBALL: &str = "mob.ghast.fireball";

const GIANT_SKIN: &str = "minecraft:mob/char";
const GIANT_SCALE: f32 = 4.0;

const MONSTER: &str = "minecraft:monster";
const SPIDER: &str = "minecraft:spider";
const PIG: &str = "minecraft:pig";
const CHICKEN: &str = "minecraft:chicken";
const GHAST: &str = "minecraft:ghast";
const SLIME: &str = "minecraft:slime";

const ICE_SOUNDS: [&str; 3] =
    ["twilightforest.mob.ice.noise", "twilightforest.mob.ice.hurt", "twilightforest.mob.ice.death"];
const REDCAP_SOUNDS: [&str; 3] =
    ["twilightforest.mob.redcap.redcap", "twilightforest.mob.redcap.hurt", "twilightforest.mob.redcap.die"];

const fn iced(mob: Mob) -> Mob { voiced(mob, Some(ICE_SOUNDS[0]), ICE_SOUNDS[1], ICE_SOUNDS[2]) }

const fn redcapped(mob: Mob) -> Mob { voiced(mob, Some(REDCAP_SOUNDS[0]), REDCAP_SOUNDS[1], REDCAP_SOUNDS[2]) }

// beta has no beetle, silverfish or iron golem sounds, the spider's and the closest ones stand in
const fn chitinous(mob: Mob) -> Mob { voiced(mob, None, "mob.spider", "mob.spiderdeath") }

const GLACIER: &[Biome] = &[Biome::Glacier];
const SNOWBALL: &str = "minecraft:snowball";

pub const MOBS: [Mob; 44] = [
    // passive animals, TFBiomeBase creature list
    voiced(
        drops(
            shadow(
                mob(
                    "wild_boar",
                    "Wild Boar",
                    PIG,
                    tf("wildboar"),
                    [0.9, 0.9],
                    10,
                    0.25,
                    0,
                    model!(tf_boar),
                    Look::Quadruped,
                    Spawning::Forest(10),
                ),
                0.7,
            ),
            &[("minecraft:porkchop", [0, 2])],
        ),
        Some("mob.pig"),
        "mob.pig",
        "mob.pigdeath",
    ),
    voiced(
        drops(
            shadow(
                mob(
                    "bighorn_sheep",
                    "Bighorn Sheep",
                    "minecraft:sheep",
                    tf("bighorn"),
                    [0.9, 1.3],
                    8,
                    0.23,
                    0,
                    model!(tf_bighorn),
                    Look::Quadruped,
                    Spawning::Forest(12),
                ),
                0.7,
            ),
            &[("minecraft:wool", [1, 1])],
        ),
        Some("mob.sheep"),
        "mob.sheep",
        "mob.sheep",
    ),
    voiced(
        drops(
            shadow(
                mob(
                    "wild_deer",
                    "Wild Deer",
                    "minecraft:cow",
                    tf("wilddeer"),
                    [0.7, 2.3],
                    10,
                    0.2,
                    0,
                    model!(tf_deer),
                    Look::Quadruped,
                    Spawning::Forest(15),
                ),
                0.7,
            ),
            &[("minecraft:leather", [0, 2]), (item::RAW_VENISON, [1, 3])],
        ),
        None,
        "mob.cowhurt",
        "mob.cowhurt",
    ),
    varied(
        acting(
            shadow(
                mob(
                    "forest_bunny",
                    "Forest Bunny",
                    PIG,
                    tf("bunnybrown"),
                    [0.3, 0.7],
                    3,
                    0.3,
                    0,
                    model!(tf_bunny),
                    Look::Posed(models::animated::bunnyPose),
                    Spawning::Forest(10),
                ),
                0.3,
            ),
            Ai::Skittish,
        ),
        &BUNNIES,
    ),
    acting(
        shadow(
            mob(
                "forest_squirrel",
                "Forest Squirrel",
                PIG,
                tf("squirrel2"),
                [0.3, 0.7],
                1,
                0.3,
                0,
                model!(tf_squirrel),
                Look::Posed(models::animated::squirrelPose),
                Spawning::Forest(10),
            ),
            0.3,
        ),
        Ai::Skittish,
    ),
    varied(
        acting(
            voiced(
                drops(
                    shadow(
                        mob(
                            "tiny_bird",
                            "Tiny Bird",
                            CHICKEN,
                            tf("tinybirdbrown"),
                            [0.5, 0.9],
                            1,
                            0.2,
                            0,
                            model!(tf_tiny_bird),
                            Look::Posed(models::animated::tinyBirdPose),
                            Spawning::Forest(15),
                        ),
                        0.3,
                    ),
                    &[("minecraft:feather", [0, 2])],
                ),
                Some("twilightforest.mob.tinybird.chirp"),
                "twilightforest.mob.tinybird.hurt",
                "twilightforest.mob.tinybird.hurt",
            ),
            Ai::Bird,
        ),
        &BIRDS,
    ),
    acting(
        voiced(
            drops(
                shadow(
                    mob(
                        "forest_raven",
                        "Forest Raven",
                        CHICKEN,
                        tf("raven"),
                        [0.3, 0.7],
                        10,
                        0.2,
                        0,
                        model!(tf_raven),
                        Look::Posed(models::animated::ravenPose),
                        Spawning::Everywhere(10),
                    ),
                    0.3,
                ),
                &[(item::RAVEN_FEATHER, [0, 2])],
            ),
            Some("twilightforest.mob.raven.caw"),
            "twilightforest.mob.raven.squawk",
            "twilightforest.mob.raven.squawk",
        ),
        Ai::Bird,
    ),
    drops(
        shadow(
            mob(
                "glacier_penguin",
                "Glacier Penguin",
                PIG,
                tf("penguin"),
                [0.5, 0.9],
                10,
                0.2,
                0,
                model!(tf_penguin),
                Look::Posed(models::animated::penguinPose),
                Spawning::Only(10, GLACIER),
            ),
            0.3,
        ),
        &[("minecraft:feather", [0, 2])],
    ),
    acting(
        shadow(
            mob(
                "firefly",
                "Firefly",
                CHICKEN,
                tf("firefly-tiny"),
                [0.5, 0.5],
                6,
                0.2,
                0,
                model!(tf_firefly),
                Look::Rest,
                Spawning::Forest(10),
            ),
            0.0,
        ),
        Ai::Flyer { speed: 0.1 },
    ),
    // monsters, their own AI comes on top of the monster template
    wearing(
        headed(
            holding(
                acting(
                    redcapped(drops(
                        mob(
                            "redcap",
                            "Redcap",
                            MONSTER,
                            tf("redcap"),
                            [0.9, 1.4],
                            20,
                            0.28,
                            4,
                            model!(tf_redcap),
                            Look::Biped,
                            Spawning::Monster,
                        ),
                        &[("minecraft:coal", [0, 2])],
                    )),
                    Ai::Redcap { sapper: false },
                ),
                "minecraft:iron_pickaxe",
            ),
            REDCAP_EARS,
        ),
        REDCAP_BOOTS,
    ),
    wearing(
        headed(
            holding(
                acting(
                    redcapped(drops(
                        mob(
                            "redcap_sapper",
                            "Redcap Sapper",
                            MONSTER,
                            tf("redcapsapper"),
                            [0.9, 1.4],
                            30,
                            0.28,
                            4,
                            model!(tf_redcap),
                            Look::Biped,
                            Spawning::Monster,
                        ),
                        &[("minecraft:coal", [0, 2])],
                    )),
                    Ai::Redcap { sapper: true },
                ),
                item::IRONWOOD_PICK,
            ),
            REDCAP_EARS,
        ),
        SAPPER_BOOTS,
    ),
    hiding(
        acting(
            voiced(
                drops(
                    mob(
                        "kobold",
                        "Twilight Kobold",
                        MONSTER,
                        tf("kobold"),
                        [0.8, 1.1],
                        13,
                        0.28,
                        4,
                        model!(tf_kobold),
                        Look::Biped,
                        Spawning::Monster,
                    ),
                    &[("minecraft:wheat", [0, 2]), ("minecraft:gold_ingot", [0, 1])],
                ),
                Some("twilightforest.mob.kobold.kobold"),
                "twilightforest.mob.kobold.hurt",
                "twilightforest.mob.kobold.die",
            ),
            Ai::Kobold,
        ),
        &[models::biped::HEADWEAR],
    ),
    holding(
        acting(
            voiced(
                drops(
                    mob(
                        super::SKELETON_DRUID,
                        "Skeleton Druid",
                        MONSTER,
                        tf("skeletondruid"),
                        [0.6, 1.8],
                        20,
                        0.25,
                        4,
                        model!(tf_skeleton_druid),
                        Look::Biped,
                        Spawning::Monster,
                    ),
                    &[(item::TORCHBERRIES, [0, 2]), ("minecraft:bone", [0, 2])],
                ),
                Some("mob.skeleton"),
                "mob.skeletonhurt",
                "mob.skeletonhurt",
            ),
            Ai::Shooter { shot: shots::NATURE_BOLT, period: 60, sound: FIREBALL },
        ),
        "minecraft:golden_hoe",
    ),
    voiced(
        mob(
            super::SWARM_SPIDER,
            "Swarm Spider",
            SPIDER,
            tf("swarmspider"),
            [0.8, 0.4],
            3,
            0.3,
            1,
            model!(tf_swarm_spider),
            Look::Spider,
            Spawning::Never,
        ),
        Some("mob.spider"),
        "mob.spider",
        "mob.spiderdeath",
    ),
    voiced(
        drops(
            mob(
                "hedge_spider",
                "Hedge Spider",
                SPIDER,
                tf("hedgespider"),
                [1.4, 0.9],
                16,
                0.3,
                2,
                model!(spider),
                Look::Spider,
                Spawning::Never,
            ),
            &[("minecraft:string", [0, 2])],
        ),
        Some("mob.spider"),
        "mob.spider",
        "mob.spiderdeath",
    ),
    voiced(
        drops(
            mob(
                "king_spider",
                "King Spider",
                SPIDER,
                tf("kingspider"),
                [1.6, 1.6],
                30,
                0.35,
                6,
                model!(spider),
                Look::Spider,
                Spawning::Monster,
            ),
            &[("minecraft:string", [0, 2])],
        ),
        Some("mob.spider"),
        "mob.spider",
        "mob.spiderdeath",
    ),
    voiced(
        mob(
            "tower_broodling",
            "Redscale Broodling",
            SPIDER,
            tf("towerbroodling"),
            [0.8, 0.4],
            7,
            0.3,
            4,
            model!(tf_swarm_spider),
            Look::Spider,
            Spawning::Monster,
        ),
        Some("mob.spider"),
        "mob.spider",
        "mob.spiderdeath",
    ),
    voiced(
        mob(
            "hostile_wolf",
            "Hostile Wolf",
            MONSTER,
            "minecraft:mob/wolf_angry",
            [0.8, 0.8],
            10,
            0.3,
            4,
            model!(wolf),
            Look::Wolf,
            Spawning::Never,
        ),
        Some("mob.wolf.growl"),
        "mob.wolf.hurt",
        "mob.wolf.death",
    ),
    voiced(
        mob(
            "mist_wolf",
            "Mist Wolf",
            MONSTER,
            tf("mistwolf"),
            [1.4, 1.9],
            30,
            0.3,
            6,
            model!(wolf),
            Look::Wolf,
            Spawning::Monster,
        ),
        Some("mob.wolf.growl"),
        "mob.wolf.hurt",
        "mob.wolf.death",
    ),
    acting(
        voiced(
            drops(
                mob(
                    "winter_wolf",
                    "Winter Wolf",
                    MONSTER,
                    tf("winterwolf"),
                    [1.4, 1.9],
                    30,
                    0.3,
                    6,
                    model!(wolf),
                    Look::Wolf,
                    Spawning::Monster,
                ),
                &[(item::ARCTIC_FUR, [0, 2])],
            ),
            Some("mob.wolf.growl"),
            "mob.wolf.hurt",
            "mob.wolf.death",
        ),
        Ai::Breather(&super::breath::FROST),
    ),
    hiding(
        acting(
            voiced(
                drops(
                    mob(
                        "twilight_wraith",
                        "Twilight Wraith",
                        MONSTER,
                        tf("ghost"),
                        [0.6, 1.8],
                        20,
                        0.5,
                        5,
                        model!(tf_wraith),
                        Look::Zombie,
                        Spawning::Monster,
                    ),
                    &[("minecraft:glowstone_dust", [0, 2])],
                ),
                Some("twilightforest.mob.wraith.wraith"),
                "twilightforest.mob.wraith.wraith",
                "twilightforest.mob.wraith.wraith",
            ),
            Ai::Flyer { speed: 0.2 },
        ),
        &[models::biped::RIGHT_LEG, models::biped::LEFT_LEG],
    ),
    voiced(
        mob(
            "mosquito_swarm",
            "Mosquito Swarm",
            MONSTER,
            tf("mosquitoswarm"),
            [0.7, 1.9],
            12,
            0.23,
            3,
            model!(tf_mosquito_swarm),
            Look::Posed(models::animated::mosquitoPose),
            Spawning::Monster,
        ),
        Some("twilightforest.mob.mosquito.mosquito"),
        "twilightforest.mob.mosquito.mosquito",
        "twilightforest.mob.mosquito.mosquito",
    ),
    acting(
        drops(
            mob(
                "death_tome",
                "Death Tome",
                MONSTER,
                "minecraft:item/book",
                [0.6, 1.1],
                30,
                0.25,
                4,
                model!(tf_death_tome),
                Look::Posed(models::animated::deathTomePose),
                Spawning::Monster,
            ),
            &[("minecraft:paper", [0, 2]), ("minecraft:book", [0, 1])],
        ),
        Ai::Shooter { shot: shots::TOME_BOLT, period: 60, sound: FIREBALL },
    ),
    holding(
        acting(
            voiced(
                drops(
                    mob(
                        "minotaur",
                        "Minotaur",
                        MONSTER,
                        tf("minotaur"),
                        [0.6, 2.1],
                        30,
                        0.25,
                        7,
                        model!(tf_minotaur),
                        Look::Biped,
                        Spawning::Monster,
                    ),
                    &[(item::RAW_MEEF, [1, 3])],
                ),
                Some("mob.cow"),
                "mob.cowhurt",
                "mob.cowhurt",
            ),
            Ai::Charger,
        ),
        "minecraft:golden_axe",
    ),
    acting(
        chitinous(drops(
            mob(
                "fire_beetle",
                "Fire Beetle",
                MONSTER,
                tf("firebeetle"),
                [1.1, 0.75],
                25,
                0.23,
                4,
                model!(tf_fire_beetle),
                Look::Posed(models::animated::fireBeetlePose),
                Spawning::Monster,
            ),
            &[("minecraft:gunpowder", [0, 2])],
        )),
        Ai::Breather(&super::breath::FIRE),
    ),
    acting(
        chitinous(drops(
            mob(
                "slime_beetle",
                "Slime Beetle",
                MONSTER,
                tf("slimebeetle"),
                [0.9, 1.75],
                25,
                0.23,
                4,
                model!(tf_slime_beetle),
                Look::Posed(models::animated::slimeBeetlePose),
                Spawning::Monster,
            ),
            &[("minecraft:slime_ball", [0, 2])],
        )),
        Ai::Shooter { shot: shots::SLIME_BLOB, period: 60, sound: "mob.slime" },
    ),
    acting(
        chitinous(mob(
            "pinch_beetle",
            "Pinch Beetle",
            MONSTER,
            tf("pinchbeetle"),
            [1.2, 1.1],
            40,
            0.23,
            4,
            model!(tf_pinch_beetle),
            Look::Posed(models::animated::pinchBeetlePose),
            Spawning::Monster,
        )),
        Ai::Pincher,
    ),
    drops(
        mob(
            "maze_slime",
            "Maze Slime",
            SLIME,
            tf("mazeslime"),
            [0.6, 0.6],
            4,
            0.3,
            2,
            model!(slime),
            Look::Rest,
            Spawning::Monster,
        ),
        &[("minecraft:slime_ball", [0, 2])],
    ),
    voiced(
        drops(
            mob(
                "tower_golem",
                "Tower Golem",
                MONSTER,
                tf("carminitegolem"),
                [1.4, 2.9],
                40,
                0.25,
                9,
                model!(tf_tower_golem),
                Look::Posed(models::animated::towerGolemPose),
                Spawning::Monster,
            ),
            &[("minecraft:iron_ingot", [0, 2]), (crate::ids::block::TOWERWOOD, [0, 2])],
        ),
        None,
        "random.break",
        "random.break",
    ),
    acting(
        chitinous(drops(
            mob(
                "tower_termite",
                "Tower Termite",
                MONSTER,
                tf("towertermite"),
                [0.3, 0.7],
                15,
                0.27,
                5,
                model!(tf_cicada),
                Look::Rest,
                Spawning::Never,
            ),
            &[(item::BORER_ESSENCE, [0, 2])],
        )),
        Ai::Termite,
    ),
    acting(
        voiced(
            drops(
                mob(
                    "block_goblin",
                    "Block&Chain Goblin",
                    MONSTER,
                    tf("blockgoblin"),
                    [0.9, 1.4],
                    20,
                    0.28,
                    // EntityTFBlockGoblin.attackEntityAsMob only swings, the block does the hurting
                    0,
                    (super::goblin::model, &models::tf_block_goblin::PIVOTS, &models::tf_block_goblin::ROTATIONS),
                    Look::Biped,
                    Spawning::Monster,
                ),
                // getDropItemId is a stale name that 1.7 never calls, so it drops nothing
                &[],
            ),
            Some(REDCAP_SOUNDS[0]),
            REDCAP_SOUNDS[1],
            REDCAP_SOUNDS[2],
        ),
        Ai::Goblin,
    ),
    acting(
        drops(
            mob(
                "goblin_knight_upper",
                "Upper Goblin Knight",
                MONSTER,
                tf("doublegoblin"),
                [1.1, 1.3],
                30,
                0.28,
                4,
                (
                    super::goblin_knight::upperModel,
                    &models::tf_goblin_knight_upper::PIVOTS,
                    &models::tf_goblin_knight_upper::ROTATIONS,
                ),
                Look::Biped,
                Spawning::Never,
            ),
            &[(item::ARMOR_SHARD, [0, 2])],
        ),
        Ai::KnightUpper,
    ),
    acting(
        mob(
            "goblin_knight_lower",
            "Lower Goblin Knight",
            MONSTER,
            tf("doublegoblin"),
            [0.7, 1.1],
            20,
            0.28,
            4,
            model!(tf_goblin_knight_lower),
            Look::Biped,
            Spawning::Monster,
        ),
        Ai::KnightLower,
    ),
    acting(
        chitinous(drops(
            mob(
                "helmet_crab",
                "Helmet Crab",
                MONSTER,
                tf("helmetcrab"),
                [0.8, 1.1],
                13,
                0.28,
                3,
                model!(tf_helmet_crab),
                Look::Posed(models::animated::helmetCrabPose),
                Spawning::Monster,
            ),
            &[(item::ARMOR_SHARD, [0, 2])],
        )),
        Ai::Leaper,
    ),
    acting(
        drops(
            mob(
                "yeti",
                "Yeti",
                MONSTER,
                tf("yeti2"),
                [1.4, 2.4],
                20,
                0.38,
                0,
                model!(tf_yeti),
                Look::Biped,
                Spawning::Monster,
            ),
            &[(item::ARCTIC_FUR, [0, 2])],
        ),
        Ai::Grabber,
    ),
    iced(drops(
        mob(
            "snow_guardian",
            "Snow Guardian",
            MONSTER,
            "twilightforest:armor/yetiarmor_1",
            [0.6, 1.8],
            10,
            0.23,
            3,
            model!(tf_snow_guardian),
            Look::Biped,
            Spawning::Monster,
        ),
        &[(SNOWBALL, [0, 2])],
    )),
    acting(
        iced(drops(
            mob(
                "stable_ice_core",
                "Stable Ice Core",
                MONSTER,
                tf("iceshooter"),
                [0.8, 1.8],
                10,
                0.23,
                3,
                (
                    models::animated::iceShooterModel,
                    &models::tf_ice_shooter::PIVOTS,
                    &models::tf_ice_shooter::ROTATIONS,
                ),
                Look::Posed(models::animated::iceShooterPose),
                Spawning::Monster,
            ),
            &[(SNOWBALL, [0, 2])],
        )),
        Ai::Shooter { shot: shots::ICE_SNOWBALL, period: 20, sound: "random.bow" },
    ),
    acting(
        iced(drops(
            mob(
                "unstable_ice_core",
                "Unstable Ice Core",
                MONSTER,
                tf("iceexploder"),
                [0.8, 1.8],
                10,
                0.23,
                3,
                (
                    models::animated::iceExploderModel,
                    &models::tf_ice_exploder::PIVOTS,
                    &models::tf_ice_exploder::ROTATIONS,
                ),
                Look::Posed(models::animated::iceExploderPose),
                Spawning::Monster,
            ),
            &[(SNOWBALL, [0, 2])],
        )),
        Ai::Exploder,
    ),
    acting(
        rare(
            drops(
                mob(
                    "troll",
                    "Troll",
                    MONSTER,
                    tf("troll"),
                    [1.4, 2.4],
                    30,
                    0.28,
                    7,
                    model!(tf_troll),
                    Look::Biped,
                    Spawning::Monster,
                ),
                NO_DROPS,
            ),
            item::MAGIC_BEANS,
        ),
        Ai::Troll,
    ),
    iced(drops(
        mob(
            "ice_crystal",
            "Ice Crystal",
            MONSTER,
            tf("icecrystal"),
            [0.6, 1.8],
            10,
            0.23,
            5,
            (models::animated::iceCrystalModel, &models::tf_ice_crystal::PIVOTS, &models::tf_ice_crystal::ROTATIONS),
            Look::Posed(models::animated::iceCrystalPose),
            Spawning::Never,
        ),
        &[(SNOWBALL, [0, 2])],
    )),
    acting(
        drops(
            mob(
                "adherent",
                "Adherent [NYI]",
                MONSTER,
                tf("adherent"),
                [0.8, 2.2],
                20,
                0.25,
                // EntityMob's default attack, the adherent never sets its own
                2,
                model!(tf_adherent),
                Look::Biped,
                Spawning::Monster,
            ),
            &[(item::ADHERENT_FRAGMENT, [0, 2])],
        ),
        Ai::Shooter { shot: shots::NATURE_BOLT, period: 60, sound: FIREBALL },
    ),
    // RenderTFGiant draws the viewer's own skin on a biped four times the size
    holding(
        scaled(
            drops(
                mob(
                    "giant_miner",
                    "Giant Miner",
                    MONSTER,
                    GIANT_SKIN,
                    [2.4, 7.2],
                    80,
                    0.23,
                    10,
                    model!(tf_loyal_zombie),
                    Look::Biped,
                    Spawning::Never,
                ),
                &[(item::GIANT_PICK, [1, 1])],
            ),
            GIANT_SCALE,
        ),
        "minecraft:stone_pickaxe",
    ),
    wearing(
        holding(
            scaled(
                drops(
                    mob(
                        "armored_giant",
                        "Armored Giant",
                        MONSTER,
                        GIANT_SKIN,
                        [2.4, 7.2],
                        80,
                        0.23,
                        10,
                        model!(tf_loyal_zombie),
                        Look::Biped,
                        Spawning::Never,
                    ),
                    &[(item::GIANT_SWORD, [1, 1])],
                ),
                GIANT_SCALE,
            ),
            "minecraft:stone_sword",
        ),
        GIANT_ARMOR,
    ),
    drops(
        mob(
            "harbinger_cube",
            "Harbinger Cube [NYI]",
            MONSTER,
            tf("apocalypse2"),
            [1.9, 2.4],
            40,
            0.23,
            2,
            model!(tf_apocalypse_cube),
            Look::Upright,
            Spawning::Never,
        ),
        &[(item::HARBINGER_FRAGMENT, [0, 2])],
    ),
];

// ghasts keep the ghast template's flight and fireballs
pub const GHASTS: [Mob; 2] = [
    voiced(
        mob(
            "mini_ghast",
            "Mini Ghast",
            GHAST,
            tf("towerghast"),
            [1.1, 1.5],
            10,
            0.3,
            0,
            (models::ghastModel, &models::tf_ghast::PIVOTS, &models::tf_ghast::ROTATIONS),
            Look::Ghast,
            Spawning::Monster,
        ),
        Some("mob.ghast.moan"),
        "mob.ghast.scream",
        "mob.ghast.death",
    ),
    voiced(
        mob(
            "tower_ghast",
            "Tower Ghast",
            GHAST,
            tf("towerghast"),
            [4.0, 6.0],
            30,
            0.3,
            0,
            (models::ghastModel, &models::tf_ghast::PIVOTS, &models::tf_ghast::ROTATIONS),
            Look::Ghast,
            Spawning::Monster,
        ),
        Some("mob.ghast.moan"),
        "mob.ghast.scream",
        "mob.ghast.death",
    ),
];

pub fn row(name: &str) -> Option<&'static Mob>
{
    MOBS.iter().chain(GHASTS.iter()).find(|row| row.m_name == name || format!("twilightforest:{}", row.m_name) == name)
}

fn texture(mob: &Mob) -> String
{
    if mob.m_texture.contains(':')
    {
        mob.m_texture.to_owned()
    }
    else
    {
        format!("twilightforest:model/{}", mob.m_texture)
    }
}

struct Posed
{
    m_mob: &'static Mob,
}

#[entity_behavior]
impl EntityBehavior for Posed
{
    fn onTick(&self, entity: Entity, info: &EntityInfo)
    {
        let variants: usize = self.m_mob.m_variants.len();
        if variants > 0 && entity.world.isAuthority() && !entity.synced::<Variant>().m_rolled
        {
            let index: u8 = entity.world.random(variants as i32) as u8;
            if let Err(error) = entity.setSynced(&Variant { m_rolled: true, m_index: index })
            {
                warn(&format!("mob variant: {}", error.0));
            }
        }
        ai::tick(self.m_mob.m_ai, entity, info);
    }

    fn onDeath(&self, entity: Entity, _source: Option<i32>)
    {
        match (self.m_mob.m_ai, entity.info())
        {
            (Ai::Kobold, Some(info)) =>
            {
                super::flock::died(entity, &info);
                crate::hints::dropNotes(entity, &info);
            }
            (Ai::Troll, Some(info)) => ai::ripen(entity, &info),
            _ =>
            {}
        }
        ai::forget(entity);
    }

    fn onHurt(&self, entity: Entity, source: Option<i32>, amount: i32) -> Option<i32>
    {
        match self.m_mob.m_ai
        {
            Ai::Redcap { .. } =>
            {
                super::redcap::hurt(entity);
                Some(amount)
            }
            Ai::KnightUpper => super::goblin_knight::upperHurt(entity, source, amount),
            Ai::KnightLower => super::goblin_knight::lowerHurt(entity, source, amount),
            Ai::Termite =>
            {
                ai::termiteHurt(entity, source);
                Some(amount)
            }
            _ => Some(amount),
        }
    }

    fn pose(&self, input: &PoseInput) -> Pose
    {
        let mob: &Mob = self.m_mob;
        match mob.m_ai
        {
            Ai::KnightUpper => return super::goblin_knight::upperPose(input),
            Ai::KnightLower => return super::goblin_knight::lowerPose(input),
            _ =>
            {}
        }
        let mut body: Program = Program::new();
        if !mob.m_variants.is_empty()
        {
            let variant: Variant = b173_api::fromBytes(&input.synced).unwrap_or_default();
            let look: &str = mob.m_variants.get(usize::from(variant.m_index)).copied().unwrap_or(mob.m_variants[0]);
            body.texture(&format!("twilightforest:model/{look}"));
        }
        match mob.m_look
        {
            Look::Rest => return Pose::default(),
            Look::Spider => return models::spiderPose(input),
            Look::Wolf => return models::wolfPose(input),
            Look::Ghast => return models::ghastPose(input),
            Look::Posed(pose) =>
            {
                let posed: Pose = pose(input);
                body.extend(posed.model);
                return Pose { pre: posed.pre, model: body.take() };
            }
            Look::Biped => bipedPose(&mut body, mob.m_pivots, mob.m_rotations, input),
            Look::Zombie => models::zombiePose(&mut body, mob.m_pivots, mob.m_rotations, input),
            Look::Quadruped | Look::Upright =>
            {
                let pitch: f32 = if mob.m_look == Look::Upright
                {
                    0.0
                }
                else
                {
                    models::HALF_PI
                };
                quadrupedPose(&mut body, 0, mob.m_pivots, input, pitch);
                for part in models::quadruped::PARTS..mob.m_pivots.len() as u16
                {
                    body.part(part, mob.m_rotations[part as usize], mob.m_pivots[part as usize]);
                }
            }
        }
        fitParts(&mut body, mob, input);
        if mob.m_ai == Ai::Goblin
        {
            super::goblin::draw(&mut body, input);
        }
        models::armor::draw(&mut body, mob.m_pivots.len() as u16, mob.m_armor, input);
        if let Some(item) = mob.m_held
        {
            body.held(models::biped::RIGHT_ARM, item);
        }
        let mut pre: Program = Program::new();
        if mob.m_scale != 1.0
        {
            pre.scale(mob.m_scale, mob.m_scale, mob.m_scale);
        }
        Pose { pre: pre.take(), model: body.take() }
    }
}

// drops the parts the Java render skips and turns head followers with the head
fn fitParts(body: &mut Program, mob: &Mob, input: &PoseInput)
{
    let head: [f32; 3] =
        [input.head_pitch / models::DEGREES_PER_RADIAN, input.head_yaw / models::DEGREES_PER_RADIAN, 0.0];
    body.m_ops.retain(|op| !matches!(op, ModelOp::Part { part, .. } if mob.m_hidden.contains(part)));
    for op in &mut body.m_ops
    {
        if let ModelOp::Part { part, rotation, .. } = op
            && mob.m_headed.contains(part)
        {
            *rotation = head;
        }
    }
}

// a mob that wears armor carries both armor layers after its own parts
fn wornModel(mob: &Mob) -> EntityModelSpec
{
    if mob.m_armor.is_empty()
    {
        return (mob.m_model)();
    }
    models::combine(vec![(mob.m_model)(), models::armor::model()])
}

fn registerMob(ctx: &mut RegisterCtx, mob: &'static Mob) -> Result<()>
{
    let animal: bool = matches!(mob.m_spawn, Spawning::Forest(_) | Spawning::Everywhere(_) | Spawning::Only(..));
    let scale: f32 = if animal
    {
        speed::ANIMAL
    }
    else
    {
        speed::MONSTER
    };
    let [living, hurt, death] = mob.m_sounds;
    let sounds: Option<EntitySounds> = hurt.map(|hurt| EntitySounds
    {
        living: living.map(str::to_owned),
        hurt: Some(hurt.to_owned()),
        death: death.map(str::to_owned),
    });
    ctx.entities().register(
        mob.m_name,
        EntitySpec
        {
            template: Identifier::parse(mob.m_template).ok(),
            texture: Some(texture(mob)),
            size: Some(mob.m_size),
            health: Some(mob.m_health),
            speed: Some(mob.m_speed * scale),
            name: Some(mob.m_label.to_owned()),
            drops: Some(
                mob.m_drops
                    .iter()
                    .map(|(item, count)| drop(item, *count))
                    .chain(mob.m_rare.map(|item| DropSpec { chance: RARE_CHANCE, ..drop(item, [1, 1]) }))
                    .collect(),
            ),
            sounds,
            model: Some(EntityModelSpec { shadow: mob.m_shadow, ..wornModel(mob) }),
            ..EntitySpec::default()
        },
        Some(Rc::new(Posed { m_mob: mob })),
    )?;
    Ok(())
}

// EntityMob.attackEntityAsMob hits as hard as the attack damage attribute
fn armed(event: &mut Event)
{
    let EventData::Entity { entity } = event.data().clone()
    else
    {
        return;
    };
    let Some(mob) =
        entity.kind.as_ref().filter(|kind| kind.namespace() == "twilightforest").and_then(|kind| row(kind.path()))
    else
    {
        return;
    };
    let world: World = World { dimension: entity.world };
    let unarmed: bool = matches!(mob.m_ai, Ai::Goblin | Ai::Grabber);
    if (mob.m_attack > 0 || unarmed) && world.isAuthority()
    {
        Entity { world, id: entity.id }.setField(EntityField::AttackStrength, f64::from(mob.m_attack));
    }
}

pub fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for mob in MOBS.iter().chain(GHASTS.iter())
    {
        registerMob(ctx, mob)?;
    }
    ctx.events().on::<EntityJoinWorld>(Priority::Normal, armed)?;
    Ok(())
}
