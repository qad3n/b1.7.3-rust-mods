use super::{put, texture};
use crate::ids::item;
use b173_api::prelude::*;

// TFItems tool materials, damage is the 1.7 bonus a beta tier doubles for swords as beta does
#[derive(Clone, Copy)]
struct Tier
{
    m_level: i32,
    m_uses: i32,
    m_speed: f32,
    m_damage: f32,
}

const IRONWOOD: Tier = Tier { m_level: 2, m_uses: 512, m_speed: 6.5, m_damage: 2.0 };
const FIERY: Tier = Tier { m_level: 4, m_uses: 1024, m_speed: 9.0, m_damage: 4.0 };
const STEELEAF: Tier = Tier { m_level: 3, m_uses: 131, m_speed: 8.0, m_damage: 3.0 };
const KNIGHTLY: Tier = Tier { m_level: 3, m_uses: 512, m_speed: 8.0, m_damage: 3.0 };
const GIANT: Tier = Tier { m_level: 1, m_uses: 1024, m_speed: 4.0, m_damage: 1.0 };
const ICE: Tier = Tier { m_level: 0, m_uses: 32, m_speed: 1.0, m_damage: 3.5 };
const GLASS: Tier = Tier { m_level: 0, m_uses: 1, m_speed: 1.0, m_damage: 36.0 };
const EMERALD: Tier = Tier { m_level: 3, m_uses: 1561, m_speed: 8.0, m_damage: 3.0 };

// how a tool class turns a tier into beta damage, ItemPickaxe ItemAxe ItemSpade ItemSword and ItemHoe
#[derive(Clone, Copy, PartialEq)]
enum Class
{
    Pickaxe,
    Axe,
    Shovel,
    Sword,
    Hoe,
    // the knightly axe minotaur axe and giant pick hit as hard as a sword of their tier
    Heavy(ToolClass),
}

fn damage(class: Class, tier: Tier) -> i32
{
    let bonus: f32 = tier.m_damage;
    let value: f32 = match class
    {
        Class::Pickaxe => 2.0 + bonus,
        Class::Axe => 3.0 + bonus,
        Class::Shovel => 1.0 + bonus,
        Class::Hoe => 1.0,
        Class::Sword | Class::Heavy(_) => 4.0 + bonus * 2.0,
    };
    value as i32
}

fn toolClass(class: Class) -> ToolClass
{
    match class
    {
        Class::Pickaxe => ToolClass::Pickaxe,
        Class::Axe => ToolClass::Axe,
        Class::Shovel => ToolClass::Shovel,
        Class::Sword => ToolClass::Sword,
        Class::Hoe => ToolClass::Hoe,
        Class::Heavy(class) => class,
    }
}

struct Tool
{
    m_name: &'static str,
    m_label: &'static str,
    m_texture: &'static str,
    m_class: Class,
    m_tier: Tier,
    // the efficiency the source enchants the crafted tool with, beta has no enchantments so it is part of the speed
    m_efficiency: i32,
}

const fn tool(
    name: &'static str,
    label: &'static str,
    texture: &'static str,
    class: Class,
    tier: Tier,
    efficiency: i32,
) -> Tool
{
    Tool { m_name: name, m_label: label, m_texture: texture, m_class: class, m_tier: tier, m_efficiency: efficiency }
}

const TOOLS: [Tool; 21] = [
    tool(item::IRONWOOD_SWORD, "Ironwood Sword", "ironwoodsword", Class::Sword, IRONWOOD, 0),
    tool("ironwood_shovel", "Ironwood Shovel", "ironwoodshovel", Class::Shovel, IRONWOOD, 0),
    tool(item::IRONWOOD_PICK, "Ironwood Pick", "ironwoodpick", Class::Pickaxe, IRONWOOD, 1),
    tool("ironwood_axe", "Ironwood Axe", "ironwoodaxe", Class::Axe, IRONWOOD, 0),
    tool("ironwood_hoe", "Ironwood Hoe", "ironwoodhoe", Class::Hoe, IRONWOOD, 0),
    tool(item::FIERY_SWORD, "Fiery Sword", "fierysword", Class::Sword, FIERY, 0),
    tool(item::FIERY_PICK, "Fiery Pick", "fierypick", Class::Pickaxe, FIERY, 0),
    tool("steeleaf_sword", "Steeleaf Sword", "steeleafsword", Class::Sword, STEELEAF, 0),
    tool("steeleaf_shovel", "Steeleaf Shovel", "steeleafshovel", Class::Shovel, STEELEAF, 2),
    tool("steeleaf_pickaxe", "Steeleaf Pick", "steeleafpick", Class::Pickaxe, STEELEAF, 0),
    tool("steeleaf_axe", "Steeleaf Axe", "steeleafaxe", Class::Axe, STEELEAF, 2),
    tool("steeleaf_hoe", "Steeleaf Hoe", "steeleafhoe", Class::Hoe, STEELEAF, 0),
    tool(item::KNIGHTLY_SWORD, "Knightly Sword", "knightlysword", Class::Sword, KNIGHTLY, 0),
    tool(item::KNIGHTLY_PICK, "Knightly Pick", "knightlypick", Class::Pickaxe, KNIGHTLY, 0),
    tool(item::KNIGHTLY_AXE, "Knightly Axe", "knightlyaxe", Class::Heavy(ToolClass::Axe), KNIGHTLY, 0),
    tool(item::MINOTAUR_AXE, "Minotaur Axe", "minotauraxe", Class::Heavy(ToolClass::Axe), EMERALD, 0),
    tool(item::MAZEBREAKER_PICK, "Mazebreaker", "mazebreakerpick", Class::Pickaxe, EMERALD, 4),
    tool(item::GIANT_PICK, "Giant's Pickaxe", "minecraft:97", Class::Heavy(ToolClass::Pickaxe), GIANT, 0),
    tool(item::GIANT_SWORD, "Giant's Sword", "minecraft:65", Class::Sword, GIANT, 0),
    tool(item::ICE_SWORD, "Ice Sword", "icesword", Class::Sword, ICE, 0),
    tool(item::GLASS_SWORD, "Glass Sword", "glasssword", Class::Sword, GLASS, 0),
];

// TFItems armor materials, the 1.7 per slot base durability times the material factor
mod armor
{
    pub const BASE: [i32; 4] = [11, 16, 15, 13];
}

struct ArmorSet
{
    m_name: &'static str,
    m_sheet: &'static str,
    m_factor: i32,
    m_protection: [i32; 4],
    m_pieces: [Option<(&'static str, &'static str)>; 4],
}

const ARMOR: [ArmorSet; 8] = [
    ArmorSet
    {
        m_name: "naga",
        m_sheet: "naga_scale",
        m_factor: 21,
        m_protection: [2, 7, 6, 3],
        m_pieces: [None, Some(("Naga Scale Tunic", "platenaga")), Some(("Naga Scale Leggings", "legsnaga")), None],
    },
    ArmorSet
    {
        m_name: "ironwood",
        m_sheet: "ironwood",
        m_factor: 20,
        m_protection: [2, 7, 5, 2],
        m_pieces: [
            Some(("Ironwood Helm", "ironwoodhelm")),
            Some(("Ironwood Plate", "ironwoodplate")),
            Some(("Ironwood Legs", "ironwoodlegs")),
            Some(("Ironwood Boots", "ironwoodboots")),
        ],
    },
    ArmorSet
    {
        m_name: "fiery",
        m_sheet: "fiery",
        m_factor: 25,
        m_protection: [4, 9, 7, 4],
        m_pieces: [
            Some(("Fiery Helm", "fieryhelm")),
            Some(("Fiery Plate", "fieryplate")),
            Some(("Fiery Legs", "fierylegs")),
            Some(("Fiery Boots", "fieryboots")),
        ],
    },
    ArmorSet
    {
        m_name: "steeleaf",
        m_sheet: "steeleaf",
        m_factor: 10,
        m_protection: [3, 8, 6, 3],
        m_pieces: [
            Some(("Steeleaf Helm", "steeleafhelm")),
            Some(("Steeleaf Plate", "steeleafplate")),
            Some(("Steeleaf Legs", "steeleaflegs")),
            Some(("Steeleaf Boots", "steeleafboots")),
        ],
    },
    ArmorSet
    {
        m_name: "knightly",
        m_sheet: "knightly",
        m_factor: 20,
        m_protection: [3, 8, 6, 3],
        m_pieces: [
            Some(("Knightly Helm", "knightlyhelm")),
            Some(("Knightly Plate", "knightlyplate")),
            Some(("Knightly Greaves", "knightlylegs")),
            Some(("Knightly Boots", "knightlyboots")),
        ],
    },
    ArmorSet
    {
        m_name: "phantom",
        m_sheet: "phantom",
        m_factor: 30,
        m_protection: [3, 8, 6, 3],
        m_pieces: [Some(("Phantom Helm", "phantomhelm")), Some(("Phantom Plate", "phantomplate")), None, None],
    },
    ArmorSet
    {
        m_name: "yeti",
        m_sheet: "yetiarmor",
        m_factor: 20,
        m_protection: [4, 7, 6, 3],
        m_pieces: [
            Some(("Yeti Horned Helm", "yetihelm")),
            Some(("Yeti Jacket", "yetiplate")),
            Some(("Yeti Leggings", "yetilegs")),
            Some(("Yeti Boots", "yetiboots")),
        ],
    },
    ArmorSet
    {
        m_name: "arctic",
        m_sheet: "arcticarmor",
        m_factor: 10,
        m_protection: [2, 7, 5, 2],
        m_pieces: [
            Some(("Arctic Hood", "arctichelm")),
            Some(("Arctic Jacket", "arcticplate")),
            Some(("Arctic Leggings", "arcticlegs")),
            Some(("Arctic Boots", "arcticboots")),
        ],
    },
];

const SLOTS: [(ArmorSlot, &str); 4] = [
    (ArmorSlot::Head, "helmet"),
    (ArmorSlot::Chest, "chestplate"),
    (ArmorSlot::Legs, "leggings"),
    (ArmorSlot::Feet, "boots"),
];

pub fn armorName(set: &str, slot: usize) -> String { format!("{set}_{}", SLOTS[slot].1) }

fn iconTexture(name: &str) -> String
{
    match name.strip_prefix("minecraft:")
    {
        Some(index) => format!("minecraft:items/{index}"),
        None => texture(name),
    }
}

// EfficiencyEnchantment adds the square of its level plus one to a tool that is already effective
fn speed(row: &Tool) -> f32
{
    let level: i32 = row.m_efficiency;
    if level == 0
    {
        row.m_tier.m_speed
    }
    else
    {
        row.m_tier.m_speed + (level * level + 1) as f32
    }
}

fn registerTools(ctx: &mut RegisterCtx) -> Result<()>
{
    for row in &TOOLS
    {
        let class: ToolClass = toolClass(row.m_class);
        let mut spec: ItemSpecBuilder = ItemSpecBuilder::new()
            .name(row.m_label)
            .texture(&iconTexture(row.m_texture))
            .tool(class, row.m_tier.m_level, speed(row), damage(row.m_class, row.m_tier), row.m_tier.m_uses);
        spec = match row.m_name
        {
            item::GIANT_PICK => spec.behavior(Giant),
            item::MAZEBREAKER_PICK => spec.behavior(Mazebreaker),
            item::GLASS_SWORD => spec.behavior(Shatter),
            item::FIERY_PICK => spec.behavior(Smelting),
            _ => spec,
        };
        put(ctx, row.m_name, spec)?;
    }
    Ok(())
}

fn registerArmor(ctx: &mut RegisterCtx) -> Result<()>
{
    for set in &ARMOR
    {
        for (slot, piece) in set.m_pieces.iter().enumerate()
        {
            let Some((label, icon)) = piece
            else
            {
                continue;
            };
            let spec: ArmorSpec = ArmorSpec
            {
                slot: SLOTS[slot].0,
                protection: set.m_protection[slot],
                durability: armor::BASE[slot] * set.m_factor,
                texture: format!("twilightforest:armor/{}", set.m_sheet),
                color: None,
            };
            put(
                ctx,
                &armorName(set.m_name, slot),
                ItemSpecBuilder::new().name(label).texture(&texture(icon)).armor(spec),
            )?;
        }
    }
    Ok(())
}

// ItemTFGiantPick digs giant blocks sixty four times faster and giant obsidian sixty four times again
struct Giant;

mod giant
{
    pub const BOOST: f32 = 64.0;
}

#[item_behavior]
impl ItemBehavior for Giant
{
    fn strVsBlock(&self, _stack: ItemStack, block: i32) -> f32
    {
        let base: f32 = GIANT.m_speed;
        if crate::ids::giantBlock(block)
        {
            let obsidian: bool = block == crate::ids::blockId(crate::ids::block::GIANT_OBSIDIAN);
            base * giant::BOOST * if obsidian { giant::BOOST } else { 1.0 }
        }
        else
        {
            1.0
        }
    }
}

// ItemTFMazebreakerPick breaks mazestone sixteen times faster than its tier
struct Mazebreaker;

#[item_behavior]
impl ItemBehavior for Mazebreaker
{
    fn strVsBlock(&self, _stack: ItemStack, block: i32) -> f32
    {
        let boost: f32 = 16.0;
        if crate::ids::mazestone(block)
        {
            (EMERALD.m_speed + 17.0) * boost
        }
        else
        {
            1.0
        }
    }
}

// ItemTFFieryPick smelts whatever it digs out, TFEventListener.harvestDrops
struct Smelting;

#[item_behavior]
impl ItemBehavior for Smelting
{
    fn onBlockDestroyed(&self, _stack: ItemStack, _block: i32, pos: BlockPos, player: Player) -> bool
    {
        if let Some(info) = player.info()
        {
            crate::combat::markSmelt(World { dimension: info.world }, pos);
        }
        true
    }
}

// ItemTFGlassSword shatters on the first hit it lands
struct Shatter;

#[item_behavior]
impl ItemBehavior for Shatter
{
    fn onHitEntity(&self, _stack: ItemStack, target: Entity, attacker: i32) -> bool
    {
        let shards: i32 = 20;
        let world: World = target.world;
        if let Some(info) = target.info()
        {
            for _ in 0..shards
            {
                crate::particles::spawn(world, "iconcrack_20_0", info.pos, [0.0; 3]);
            }
            world.playSound(info.pos, "random.glass", 1.0, 0.5);
        }
        let _ = Player { id: attacker }.damageHeld(GLASS.m_uses + 1);
        false
    }
}

// armor slots run from boots up, the reverse of SLOTS
fn wornIn(player: Player, slot: usize) -> Option<ItemStack>
{
    player.slot(super::ARMOR_FIRST_SLOT + (SLOTS.len() - 1 - slot) as i32)
}

pub fn armorWorn(player: Player, set: &str) -> usize
{
    (0..SLOTS.len())
        .filter(|slot| wornIn(player, *slot).is_some_and(|stack| stack.item.is(&armorName(set, *slot))))
        .count()
}

pub fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    registerTools(ctx)?;
    registerArmor(ctx)?;
    crate::combat::register(ctx)
}
