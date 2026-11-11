use crate::names::block;
use crate::world::palette::Props;

// a 1.7 block and metadata the structure code names, as the beta or forest state it becomes
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Resolved
{
    // a beta block with the metadata beta stores for it, which beta keeps as its only state
    Vanilla(&'static str, i32),
    // one of the forest's blocks with up to three named properties
    Twilight(&'static str, Props),
}

const fn vanilla(path: &'static str, data: i32) -> Resolved { Resolved::Vanilla(path, data) }

const fn tf(name: &'static str) -> Resolved { Resolved::Twilight(name, [None, None, None]) }

const fn tfWith(name: &'static str, property: &'static str, value: &'static str) -> Resolved
{
    Resolved::Twilight(name, [Some((property, value)), None, None])
}

// the values numbered and boolean properties take, picked by the 1.7 metadata
const NUMBERS: [&str; 16] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15"];
const FLAGS: [&str; 2] = ["false", "true"];

fn number(data: i32) -> &'static str { NUMBERS[data.rem_euclid(NUMBERS.len() as i32) as usize] }

// a bit of the 1.7 metadata as a boolean property value
fn flag(data: i32, bit: i32) -> &'static str { FLAGS[usize::from(data & bit != 0)] }

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
pub(crate) fn resolve(name: &'static str, data: i32) -> Resolved
{
    if let Some(field) = name.strip_prefix("tf:")
    {
        return twilight(field, data);
    }
    if SAME.contains(&name)
    {
        return vanilla(name, data);
    }

    match name
    {
        "log" => vanilla("log", (data & 3) % 3),
        "leaves" => vanilla("leaves", (data & 3) % 3),
        "log2" => vanilla("log", 0),
        "leaves2" => vanilla("leaves", 0),
        "red_flower" => vanilla("red_flower", 0),
        "double_plant" => vanilla("tallgrass", 1),
        "stone_slab" => vanilla("stone_slab", slab(data)),
        "double_stone_slab" => vanilla("double_stone_slab", slab(data)),
        "wooden_slab" => vanilla("stone_slab", 2),
        "double_wooden_slab" => vanilla("double_stone_slab", 2),
        "oak_stairs" | "spruce_stairs" | "birch_stairs" | "jungle_stairs" | "acacia_stairs" =>
        {
            vanilla("oak_stairs", data & 3)
        }
        "stone_stairs"
        | "stone_brick_stairs"
        | "brick_stairs"
        | "nether_brick_stairs"
        | "sandstone_stairs"
        | "quartz_stairs" => vanilla("stone_stairs", data & 3),
        "stonebrick" => stoneBrick(data),
        "monster_egg" => match data
        {
            1 => vanilla("cobblestone", 0),
            2..=5 => stoneBrick(data - 2),
            _ => vanilla("stone", 0),
        },
        "iron_bars" | "nether_brick_fence" | "fence_gate" => vanilla("fence", 0),
        "glass_pane" | "stained_glass" | "stained_glass_pane" => vanilla("glass", 0),
        "cobblestone_wall" => vanilla("cobblestone", 0),
        "trapped_chest" | "ender_chest" => vanilla("chest", 0),
        "redstone_lamp" | "lit_redstone_lamp" => vanilla("glowstone", 0),
        "quartz_block" | "quartz_ore" | "end_stone" => vanilla("sandstone", 0),
        "emerald_block" => vanilla("diamond_block", 0),
        "emerald_ore" => vanilla("diamond_ore", 0),
        "packed_ice" => vanilla("ice", 0),
        "mycelium" => vanilla("grass", 0),
        "nether_brick" => vanilla("netherrack", 0),
        "anvil" => vanilla("iron_block", 0),
        "enchanting_table" | "end_portal_frame" | "coal_block" => vanilla("obsidian", 0),
        "redstone_block" => vanilla("lit_redstone_ore", 0),
        "stained_hardened_clay" => vanilla("wool", data),
        "hardened_clay" => vanilla("wool", 12),
        "hay_block" => vanilla("wool", 4),
        "melon_block" => vanilla("pumpkin", 0),
        "red_mushroom_block" => mushroom(block::HUGE_RED_MUSHROOM, data),
        "brown_mushroom_block" => mushroom(block::HUGE_BROWN_MUSHROOM, data),
        _ => vanilla("air", 0),
    }
}

// beta slabs are stone sandstone wood and cobblestone, with no top halves
fn slab(data: i32) -> i32
{
    match data & 7
    {
        value @ 0..=3 => value,
        6 => 3,
        _ => 0,
    }
}

// beta has no stone bricks, so plain mossy cracked and chiseled fall back to stone mossy cobblestone and cobblestone
fn stoneBrick(data: i32) -> Resolved
{
    match data
    {
        1 => vanilla("mossy_cobblestone", 0),
        2 => vanilla("cobblestone", 0),
        _ => vanilla("stone", 0),
    }
}

// BlockHugeMushroom metadata, ten and fifteen are stem, the rest some cap
fn mushroom(name: &'static str, data: i32) -> Resolved
{
    let part: &str = if matches!(data, 10 | 15)
    {
        "stem"
    }
    else
    {
        "cap"
    };
    tfWith(name, "part", part)
}

// TFBlocks fields by their metadata, the port splits most metadata variants into named blocks or states
fn twilight(field: &str, data: i32) -> Resolved
{
    match field
    {
        "nagastone" =>
        {
            let (name, property, value): (&str, &str, &str) = crate::blocks::sourceState(data);
            tfWith(name, property, value)
        }

        "mazestone" => tf([
            block::MAZESTONE,
            block::MAZESTONE_BRICK,
            "chiseled_mazestone",
            "decorative_mazestone",
            block::CRACKED_MAZESTONE,
            block::MOSSY_MAZESTONE,
            block::MAZESTONE_MOSAIC,
            block::MAZESTONE_BORDER,
        ][(data & 7) as usize]),
        "towerWood" => tf([
            block::TOWERWOOD,
            block::ENCASED_TOWERWOOD,
            block::CRACKED_TOWERWOOD,
            block::MOSSY_TOWERWOOD,
            block::INFESTED_TOWERWOOD,
        ][(data.clamp(0, 4)) as usize]),
        "towerDevice" => tf(device(data)),
        "log" => tf([block::OAK_LOG, block::CANOPY_LOG, block::MANGROVE_LOG, block::DARKWOOD_LOG][(data & 3) as usize]),
        "leaves" => tf([block::OAK_LEAVES, block::CANOPY_LEAVES, block::MANGROVE_LEAVES, block::RAINBOW_LEAVES]
            [(data & 3) as usize]),
        "thorns" =>
        {
            let name: &str = if data & 3 == 1
            {
                block::GREEN_THORNS
            }
            else
            {
                block::BROWN_THORNS
            };
            tfWith(name, "axis", crate::blocks::AXES[thornAxis(data)])
        }

        // the rune brick's inverted bit, spare bit and color in the low bits
        "castleMagic" => Resolved::Twilight(
            block::CASTLE_RUNE_BRICK,
            [Some(("inverted", flag(data, 8))), Some(("spare", flag(data, 4))), Some(("color", number(data & 3)))],
        ),
        // the door's active bit over its color
        "castleDoor" => Resolved::Twilight(
            block::CASTLE_DOOR,
            [Some(("active", flag(data, 8))), Some(("color", number(data & 7))), None],
        ),
        "castleBlock" =>
        {
            tf([block::CASTLE_BRICK, block::WORN_CASTLE_BRICK, block::CRACKED_CASTLE_BRICK, block::CASTLE_ROOF]
                [(data & 3) as usize])
        }
        "underBrick" =>
        {
            tf([block::KNIGHTBRICK, block::MOSSY_KNIGHTBRICK, block::CRACKED_KNIGHTBRICK, "underbrick_floor"]
                [(data & 3) as usize])
        }
        "deadrock" =>
        {
            tf([block::DEADROCK, block::CRACKED_DEADROCK, block::SOLID_DEADROCK][(data.clamp(0, 2)) as usize])
        }
        "forceField" => tf([
            "purple_force_field",
            "pink_force_field",
            "orange_force_field",
            "green_force_field",
            "blue_force_field",
        ][(data.clamp(0, 4)) as usize]),
        "root" => tf(
            if data == 1
            {
                block::LIVEROOT
            }
            else
            {
                block::ROOT
            },
        ),
        "plant" => tf(plant(data)),
        "giantLeaves" => tf(block::GIANT_LEAVES),
        "giantLog" => tf(block::GIANT_LOG),
        "giantCobble" => tf(block::GIANT_COBBLE),
        "giantObsidian" => tf(block::GIANT_OBSIDIAN),
        "firefly" => tfWith(block::FIREFLY, "facing", critterFacing(data)),
        "cicada" => tfWith(block::CICADA, "facing", critterFacing(data)),
        "bossSpawner" => tfWith(crate::bosses::SPAWNER, "boss", crate::bosses::KINDS[data.clamp(0, 5) as usize]),
        "trollSteinn" => tf(block::TROLLSTEINN),
        "fluffyCloud" => tf(block::FLUFFY_CLOUD),
        // a facing past the six faces is the latent shield a trophy pedestal clears
        "shield" => tfWith(block::SHIELD, "facing", crate::blocks::SIDES[data.clamp(0, 6) as usize]),
        "auroraBlock" => tfWith(block::AURORA_BRICK, "phase", number(data)),
        "auroraPillar" => tf(block::AURORA_PILLAR),
        "uberousSoil" => tf(block::UBEROUS_SOIL),
        // the pedestal's latent bit and spare bit over the turn of its faces
        "trophyPedestal" => Resolved::Twilight(
            block::TROPHY_PEDESTAL,
            [Some(("latent", flag(data, 8))), Some(("spare", flag(data, 4))), Some(("turn", number(data & 3)))],
        ),
        "hugeGloomBlock" => mushroom(block::HUGE_GLOOM, data),
        "hedge" => tf(block::HEDGE),
        _ => vanilla("air", 0),
    }
}

// BlockTFCritter metadata to the facing it clings by
fn critterFacing(data: i32) -> &'static str
{
    let facings: [&str; 7] = crate::blocks::CRITTER_FACINGS;
    facings[data.clamp(0, facings.len() as i32 - 1) as usize]
}

// BlockTFThorns keeps its axis in the top two bits
fn thornAxis(data: i32) -> usize
{
    match data & 12
    {
        4 => 1,
        8 => 2,
        _ => 0,
    }
}

// BlockTFTowerDevice metadata to the port's device blocks
fn device(data: i32) -> &'static str
{
    match data
    {
        0 => block::REAPPEARING,
        1 => "reappearing_block_active",
        2 => block::VANISHING,
        3 => "vanishing_block_active",
        4 => block::LOCKED_VANISHING,
        5 => block::VANISHING,
        6 => block::BUILDER,
        7 => "carminite_builder_active",
        8 => "carminite_builder_timeout",
        9 => block::ANTIBUILDER,
        10 => block::GHAST_TRAP,
        11 => "ghast_trap_active",
        12 => block::REACTOR,
        _ => "carminite_reactor_active",
    }
}

// BlockTFPlant metadata to the port's plants
fn plant(data: i32) -> &'static str
{
    match data
    {
        3 => "moss_patch",
        4 => "mayapple",
        5 => "clover_patch",
        8 => block::FIDDLEHEAD,
        9 => block::MUSHGLOOM,
        11 => block::DRIED_BUSH,
        13 => block::TORCHBERRY_PLANT,
        14 => block::ROOT_STRAND,
        _ => block::FOREST_GRASS,
    }
}

// TFCreatures.getSpawnerNameFor and the vanilla names structures give their spawners
pub(crate) fn mob(name: &str) -> &'static str
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

#[cfg(test)]
mod tests
{
    use super::*;

    // the stored metadata of a state, the last property changing fastest as the loader lays states out
    fn stored(props: Props, table: &[(&str, &[&str])]) -> i32
    {
        table.iter().fold(0, |index, (property, values)|
        {
            let value: &str =
                props.iter().flatten().find(|(name, _)| name == property).map_or(values[0], |pair| pair.1);
            let at: usize = values.iter().position(|known| *known == value).expect("value is one the block registers");
            index * values.len() as i32 + at as i32
        })
    }

    fn props(field: &str, data: i32) -> Props
    {
        let Resolved::Twilight(_, props) = twilight(field, data)
        else
        {
            panic!("{field} is one of the forest's blocks");
        };
        props
    }

    #[test]
    fn castle_runes_doors_and_pedestals_keep_their_metadata()
    {
        let rune: [(&str, &[&str]); 3] = [("inverted", &FLAGS), ("spare", &FLAGS), ("color", &NUMBERS[..4])];
        let door: [(&str, &[&str]); 2] = [("active", &FLAGS), ("color", &NUMBERS[..8])];
        let pedestal: [(&str, &[&str]); 3] = [("latent", &FLAGS), ("spare", &FLAGS), ("turn", &NUMBERS[..4])];

        for data in 0..16
        {
            assert_eq!(stored(props("castleMagic", data), &rune), data);
            assert_eq!(stored(props("castleDoor", data), &door), data);
            assert_eq!(stored(props("trophyPedestal", data), &pedestal), data);
        }
    }

    #[test]
    fn shields_critters_and_spawners_keep_their_metadata()
    {
        for data in 0..7
        {
            assert_eq!(stored(props("shield", data), &[("facing", &crate::blocks::SIDES)]), data);
            assert_eq!(stored(props("firefly", data), &[("facing", &crate::blocks::CRITTER_FACINGS)]), data);
        }

        for data in 0..6
        {
            assert_eq!(stored(props("bossSpawner", data), &[("boss", &crate::bosses::KINDS)]), data);
        }
    }
}
