use crate::names::{block, item};
use b173_api::prelude::*;

// EnumToolMaterial rows the Aether tools are built on
#[derive(Clone, Copy)]
struct Tier
{
    m_level: i32,
    m_uses: i32,
    m_speed: f32,
    m_damage: i32,
}

const WOOD: Tier = Tier { m_level: 0, m_uses: 59, m_speed: 2.0, m_damage: 0 };
const STONE: Tier = Tier { m_level: 1, m_uses: 131, m_speed: 4.0, m_damage: 1 };
const IRON: Tier = Tier { m_level: 2, m_uses: 250, m_speed: 6.0, m_damage: 2 };
const EMERALD: Tier = Tier { m_level: 3, m_uses: 1561, m_speed: 8.0, m_damage: 3 };

// ItemPickaxe, ItemAxe, ItemSpade and ItemSword base damage before the tier bonus
fn baseDamage(class: ToolClass, tier: Tier) -> i32
{
    match class
    {
        ToolClass::Pickaxe => 2 + tier.m_damage,
        ToolClass::Axe => 3 + tier.m_damage,
        ToolClass::Shovel => 1 + tier.m_damage,
        _ => 4 + tier.m_damage * 2,
    }
}

// ItemArmor.damageReduceAmountArray and maxDamageArray, iron sheets shift the durability by 2, diamond by 3
const PROTECTION: [i32; 4] = [3, 8, 6, 3];
const DURABILITY: [i32; 4] = [11, 16, 15, 13];
const IRON_ARMOR: i32 = 2;
const DIAMOND_ARMOR: i32 = 3;
const SLOTS: [(ArmorSlot, &str, &str); 4] = [
    (ArmorSlot::Head, "helmet", "minecraft:head_armor"),
    (ArmorSlot::Chest, "chestplate", "minecraft:chest_armor"),
    (ArmorSlot::Legs, "leggings", "minecraft:leg_armor"),
    (ArmorSlot::Feet, "boots", FOOT_ARMOR),
];
const FOOT_ARMOR: &str = "minecraft:foot_armor";
const SWORD_TAG: &str = "minecraft:swords";

// one Aether armor set, tinted sets reuse the vanilla iron sheet
struct ArmorSet
{
    m_name: &'static str,
    m_level: i32,
    m_sheet: &'static str,
    m_color: Option<u32>,
}

const ARMOR: [ArmorSet; 6] = [
    ArmorSet { m_name: "zanite", m_level: IRON_ARMOR, m_sheet: "minecraft:armor/iron", m_color: Some(7412456) },
    ArmorSet { m_name: "gravitite", m_level: DIAMOND_ARMOR, m_sheet: "minecraft:armor/iron", m_color: Some(15160027) },
    ArmorSet { m_name: "neptune", m_level: DIAMOND_ARMOR, m_sheet: "minecraft:armor/iron", m_color: Some(2512127) },
    ArmorSet { m_name: "obsidian", m_level: DIAMOND_ARMOR, m_sheet: "minecraft:armor/iron", m_color: Some(1774663) },
    ArmorSet { m_name: "phoenix", m_level: DIAMOND_ARMOR, m_sheet: "aether:armor/phoenix_layer", m_color: None },
    ArmorSet { m_name: "valkyrie", m_level: DIAMOND_ARMOR, m_sheet: "aether:armor/valkyrie_layer", m_color: None },
];

const TOOL_SETS: [(&str, Tier); 5] =
    [("skyroot", WOOD), ("holystone", STONE), ("zanite", IRON), ("gravitite", EMERALD), ("valkyrie", EMERALD)];
const TOOL_KINDS: [(ToolClass, &str, &str); 3] = [
    (ToolClass::Pickaxe, "pickaxe", "minecraft:pickaxes"),
    (ToolClass::Axe, "axe", "minecraft:axes"),
    (ToolClass::Shovel, "shovel", "minecraft:shovels"),
];
const SWORDS: [(&str, Tier); 4] = [("skyroot", WOOD), ("holystone", STONE), ("zanite", IRON), ("gravitite", EMERALD)];

// name, texture under item/ and heal amount; heals are b1.7.3 half hearts from the Aether food constructors
const FOODS: [(&str, &str, i32); 8] = [
    (item::BLUEBERRY, "food/blueberry", 2),
    ("enchanted_blueberry", "food/enchanted_blueberry", 8),
    (item::WHITE_APPLE, "food/white_apple", 0),
    ("blue_gummy_swet", "food/blue_gummy_swet", 20),
    ("golden_gummy_swet", "food/golden_gummy_swet", 20),
    // ItemHealingStone heals nothing itself, its regeneration does
    (HEALING_STONE, "food/healing_stone", 0),
    ("candy_cane", "food/candycane", 2),
    ("gingerbread_man", "food/gingerbread_man", 2),
];

// ItemsAether swords on vanilla tool materials, elemental blades and the pig slayer wear out at their own rate
const GOLD: Tier = Tier { m_level: 0, m_uses: 32, m_speed: 12.0, m_damage: 0 };
const WEAPONS: [(&str, &str, Tier, Option<i32>); 8] = [
    ("flaming_sword", "flaming_sword", EMERALD, Some(502)),
    ("lightning_sword", "lightning_sword", EMERALD, Some(502)),
    ("holy_sword", "holy_sword", EMERALD, Some(502)),
    ("vampire_blade", "vampire_blade", EMERALD, None),
    ("pig_slayer", "pig_slayer", IRON, Some(200)),
    ("candy_cane_sword", "candycane_sword", GOLD, None),
    ("notch_hammer", "notch_hammer", IRON, None),
    ("valkyrie_lance", "valkyrie_lance", EMERALD, None),
];

// plain items whose behavior lives in their own systems: name, icon under item/, stack size
// ItemAetherDisc rows: name, artist and the song title from item.tooltip
const DISCS: [(&str, &str, &str); 4] = [
    ("aether_tune", "Noisestorm", "Aether Tune"),
    ("ascending_dawn", "Emile van Krieken", "Ascending Dawn"),
    ("welcoming_skies", "Voyed", "Welcoming Skies"),
    ("legacy", "Jon Lachney", "Legacy"),
];

const SINGLES: [(&str, &str, i32); 21] = [
    ("bronze_key", "misc/keys/bronze_key", 1),
    ("silver_key", "misc/keys/silver_key", 1),
    ("golden_key", "misc/keys/golden_key", 1),
    ("skyroot_bucket", "misc/buckets/skyroot_bucket", 16),
    ("skyroot_water_bucket", "misc/buckets/skyroot_water_bucket", 1),
    ("skyroot_milk_bucket", "misc/buckets/skyroot_milk_bucket", 1),
    ("skyroot_poison_bucket", "misc/buckets/skyroot_poison_bucket", 1),
    ("skyroot_remedy_bucket", "misc/buckets/skyroot_remedy_bucket", 1),
    ("cold_parachute", "misc/parachutes/cold_parachute", 1),
    ("golden_parachute", "misc/parachutes/golden_parachute", 1),
    ("life_shard", "misc/life_shard", 1),
    ("cloud_staff", "staff/cloud_staff", 1),
    ("nature_staff", "staff/nature_staff", 1),
    ("golden_dart_shooter", "projectile/golden_dart_shooter", 1),
    ("poison_dart_shooter", "projectile/poison_dart_shooter", 1),
    ("enchanted_dart_shooter", "projectile/enchanted_dart_shooter", 1),
    ("golden_dart", "projectile/golden_dart", 64),
    ("poison_dart", "projectile/poison_dart", 64),
    ("enchanted_dart", "projectile/enchanted_dart", 64),
    ("lightning_knife", "weapons/lightning_knife", 16),
    ("aether_portal_frame", "aether_portal_frame", 1),
];
// ItemPhoenixBow and the staffs wear out
const PHOENIX_BOW_USES: i32 = 384;
const CLOUD_STAFF_USES: i32 = 60;
const NATURE_STAFF_USES: i32 = 100;

const MATERIALS: [(&str, &str); 6] = [
    (item::ZANITE_GEMSTONE, "misc/zanite_gemstone"),
    (item::AMBROSIUM_SHARD, "misc/ambrosium_shard"),
    (item::GOLDEN_AMBER, "misc/golden_amber"),
    (item::AECHOR_PETAL, "misc/aechor_petal"),
    (item::SWET_BALL, "misc/swet_ball"),
    (item::SKYROOT_STICK, "skyroot_stick"),
];

const VICTORY_MEDAL_STACK: i32 = 10;

fn texture(path: &str) -> String { format!("aether:item/{path}") }

// ItemWhiteApple: eating it cures inebriation, the food itself is eaten as usual
struct WhiteApple;

#[item_behavior]
impl ItemBehavior for WhiteApple
{
    fn onRightClick(&self, world: World, player: Player, _stack: ItemStack) -> Use
    {
        crate::player::cure(Entity { world, id: player.id });
        Use::Pass
    }
}

const HEALING_STONE: &str = "healing_stone";
// ItemHealingStone.onFoodEaten
const HEALING_TICKS: i32 = 610;

// ItemHealingStone: eating it gives half a minute of regeneration, the food itself is eaten as usual
struct HealingStone;

#[item_behavior]
impl ItemBehavior for HealingStone
{
    fn onRightClick(&self, world: World, player: Player, _stack: ItemStack) -> Use
    {
        if world.isAuthority()
        {
            crate::effects::give(Entity { world, id: player.id }, crate::effects::Effect::Regeneration, HEALING_TICKS);
        }

        Use::Pass
    }
}

// ItemAmbrosiumShard.onItemUse, a shard enchants the aether grass it is used on
struct AmbrosiumShard;

#[item_behavior]
impl ItemBehavior for AmbrosiumShard
{
    fn onUseOnBlock(&self, world: World, _player: Player, stack: ItemStack, pos: BlockPos, _face: i32) -> Use
    {
        if !world.is(pos, block::AETHER_GRASS)
        {
            return Use::Pass;
        }

        if world.isAuthority()
        {
            let _ = world.place(pos, block::ENCHANTED_AETHER_GRASS);
        }

        let remaining: i32 = stack.count - 1;
        Use::Done((remaining > 0).then_some(ItemStack { count: remaining, ..stack }))
    }
}

fn registerArmor(ctx: &mut RegisterCtx) -> Result<()>
{
    for set in &ARMOR
    {
        for (index, (slot, piece, tag)) in SLOTS.iter().enumerate()
        {
            let name: String = format!("{}_{piece}", set.m_name);

            let armor: ArmorSpec = ArmorSpec
            {
                slot: *slot,
                protection: PROTECTION[index],
                durability: (DURABILITY[index] * 3) << set.m_level,
                texture: set.m_sheet.to_owned(),
                color: set.m_color,
            };
            ctx.items().register(
                &name,
                ItemSpecBuilder::new().texture(&texture(&format!("armor/{name}"))).armor(armor).tag(tag),
            )?;
        }
    }

    let sentry: ArmorSpec = ArmorSpec
    {
        slot: ArmorSlot::Feet,
        protection: PROTECTION[3],
        durability: (DURABILITY[3] * 3) << DIAMOND_ARMOR,
        texture: "aether:armor/sentry_layer".to_owned(),
        color: None,
    };
    ctx.items().register(
        "sentry_boots",
        ItemSpecBuilder::new().texture(&texture("armor/sentry_boots")).armor(sentry).tag(FOOT_ARMOR),
    )?;
    Ok(())
}

// the durability of one piece of an Aether armor set, index 0 boots up to 3 helmet
pub(crate) fn armorUses(set: &str, index: usize) -> Option<i32>
{
    let row: &ArmorSet = ARMOR.iter().find(|row| row.m_name == set)?;
    let piece: usize = SLOTS.len() - 1 - index;
    Some((DURABILITY[piece] * 3) << row.m_level)
}

// the uses of an Aether tool or sword by name
pub(crate) fn toolUses(name: &str) -> Option<i32>
{
    let (material, _) = name.split_once('_')?;
    TOOL_SETS.iter().chain(SWORDS.iter()).find(|(set, _)| *set == material).map(|(_, tier)| tier.m_uses)
}

fn registerTools(ctx: &mut RegisterCtx) -> Result<()>
{
    for (material, tier) in TOOL_SETS
    {
        for (class, kind, tag) in TOOL_KINDS
        {
            let name: String = format!("{material}_{kind}");
            let mut spec: ItemSpecBuilder = ItemSpecBuilder::new()
                .texture(&texture(&format!("tools/{name}")))
                .tool(class, tier.m_level, tier.m_speed, baseDamage(class, tier), tier.m_uses)
                .tag(tag);

            match material
            {
                "holystone" => spec = spec.behavior(crate::tools::holystone()),
                "gravitite" => spec = spec.behavior(crate::tools::gravitite()),
                _ =>
                {}
            }

            ctx.items().register(&name, spec)?;
        }
    }

    for (material, tier) in SWORDS
    {
        let name: String = format!("{material}_sword");
        ctx.items().register(
            &name,
            ItemSpecBuilder::new()
                .texture(&texture(&format!("weapons/{name}")))
                .tool(ToolClass::Sword, tier.m_level, tier.m_speed, baseDamage(ToolClass::Sword, tier), tier.m_uses)
                .tag(SWORD_TAG),
        )?;
    }

    Ok(())
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for (name, path) in MATERIALS
    {
        let spec: ItemSpecBuilder = ItemSpecBuilder::new().texture(&texture(path));
        let spec: ItemSpecBuilder = if name == item::AMBROSIUM_SHARD
        {
            spec.behavior(AmbrosiumShard)
        }
        else
        {
            spec
        };
        ctx.items().register(name, spec)?;
    }

    for (name, path, heal) in FOODS
    {
        let spec: ItemSpecBuilder = ItemSpecBuilder::new().texture(&texture(path)).food(heal);
        let spec: ItemSpecBuilder = match name
        {
            item::WHITE_APPLE => spec.behavior(WhiteApple),
            HEALING_STONE => spec.behavior(HealingStone),
            _ => spec,
        };
        ctx.items().register(name, spec)?;
    }

    ctx.items().register(
        item::VICTORY_MEDAL,
        ItemSpecBuilder::new().texture(&texture(item::VICTORY_MEDAL)).maxStack(VICTORY_MEDAL_STACK),
    )?;

    for (name, icon, stack) in SINGLES
    {
        let uses: Option<i32> = match name
        {
            "cloud_staff" => Some(CLOUD_STAFF_USES),
            "nature_staff" => Some(NATURE_STAFF_USES),
            _ => None,
        };

        let mut spec: ItemSpecBuilder = ItemSpecBuilder::new().texture(&texture(icon)).maxStack(stack);

        if let Some(uses) = uses
        {
            spec = spec.maxDamage(uses);
        }

        if let Some(shooter) = crate::weapons::shooter(name)
        {
            spec = spec.behavior(shooter);
        }
        else if name == "lightning_knife"
        {
            spec = spec.behavior(crate::weapons::knife());
        }
        else if let Some(bucket) = crate::gear::bucket(name)
        {
            spec = spec.behavior(bucket);
        }
        else if let Some(chute) = crate::gear::parachute(name)
        {
            spec = spec.behavior(chute);
        }
        else if name == "cloud_staff"
        {
            spec = spec.behavior(crate::entities::mini_cloud::staff());
        }
        else if name == "life_shard"
        {
            spec = spec.behavior(crate::accessories::lifeShard());
        }
        ctx.items().register(name, spec)?;
    }

    for (name, artist, song) in DISCS
    {
        ctx.items().register(
            name,
            ItemSpecBuilder::new()
                .texture(&texture(&format!("music/{name}")))
                .record(&format!("aether.record.{name}"), &format!("{artist} - {song}")),
        )?;
    }

    crate::weapons::reservePulls();
    ctx.items().register(
        "phoenix_bow",
        ItemSpecBuilder::new()
            .texture(&texture("weapons/phoenix_bow"))
            .maxStack(1)
            .maxDamage(PHOENIX_BOW_USES)
            .behavior(crate::weapons::PhoenixBow),
    )?;

    for (name, icon, tier, uses) in WEAPONS
    {
        let durability: i32 = uses.unwrap_or(tier.m_uses);
        let mut spec: ItemSpecBuilder = ItemSpecBuilder::new()
            .texture(&texture(&format!("weapons/{icon}")))
            .tool(ToolClass::Sword, tier.m_level, tier.m_speed, baseDamage(ToolClass::Sword, tier), durability)
            .tag(SWORD_TAG);

        if let Some(blade) = crate::weapons::blade(name)
        {
            spec = spec.behavior(blade);
        }
        else if name == "notch_hammer"
        {
            spec = spec.behavior(crate::weapons::hammer());
        }
        ctx.items().register(name, spec)?;
    }

    registerTools(ctx)?;
    registerArmor(ctx)
}
