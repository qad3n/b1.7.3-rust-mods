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

// plain items whose behavior lives in their own systems: name, icon under item/, stack size and the item one
// leaves behind in a crafting grid or machine, as ItemSkyrootBucket.setContainerItem
// ItemAetherDisc rows: name, artist and the song title from item.tooltip
const DISCS: [(&str, &str, &str); 4] = [
    ("aether_tune", "Noisestorm", "Aether Tune"),
    ("ascending_dawn", "Emile van Krieken", "Ascending Dawn"),
    ("welcoming_skies", "Voyed", "Welcoming Skies"),
    ("legacy", "Jon Lachney", "Legacy"),
];

const SINGLES: [(&str, &str, i32, Option<&str>); 21] = [
    ("bronze_key", "misc/keys/bronze_key", 1, None),
    ("silver_key", "misc/keys/silver_key", 1, None),
    ("golden_key", "misc/keys/golden_key", 1, None),
    ("skyroot_bucket", "misc/buckets/skyroot_bucket", 16, None),
    ("skyroot_water_bucket", "misc/buckets/skyroot_water_bucket", 1, Some("skyroot_bucket")),
    ("skyroot_milk_bucket", "misc/buckets/skyroot_milk_bucket", 1, Some("skyroot_bucket")),
    ("skyroot_poison_bucket", "misc/buckets/skyroot_poison_bucket", 1, Some("skyroot_bucket")),
    ("skyroot_remedy_bucket", "misc/buckets/skyroot_remedy_bucket", 1, Some("skyroot_bucket")),
    ("cold_parachute", "misc/parachutes/cold_parachute", 1, None),
    ("golden_parachute", "misc/parachutes/golden_parachute", 1, None),
    ("life_shard", "misc/life_shard", 1, None),
    ("cloud_staff", "staff/cloud_staff", 1, None),
    ("nature_staff", "staff/nature_staff", 1, None),
    ("golden_dart_shooter", "projectile/golden_dart_shooter", 1, None),
    ("poison_dart_shooter", "projectile/poison_dart_shooter", 1, None),
    ("enchanted_dart_shooter", "projectile/enchanted_dart_shooter", 1, None),
    ("golden_dart", "projectile/golden_dart", 64, None),
    ("poison_dart", "projectile/poison_dart", 64, None),
    ("enchanted_dart", "projectile/enchanted_dart", 64, None),
    ("lightning_knife", "weapons/lightning_knife", 16, None),
    ("aether_portal_frame", "aether_portal_frame", 1, None),
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
// ItemAmbrosiumShard.onItemRightClick heals this much
const SHARD_HEAL: i32 = 1;
// ItemSwettyBall.onItemUse: the dirt it is used on and the grass it leaves
const SWET_GROWS: [(&str, &str); 2] =
    [(block::AETHER_DIRT, block::AETHER_GRASS), ("minecraft:dirt", "minecraft:grass")];

fn texture(path: &str) -> String { format!("aether:item/{path}") }

// ItemWhiteApple.onFoodEaten, its setCured
const APPLE_CURE: i32 = 300;

// ItemWhiteApple: eating it cures inebriation, the food itself is eaten as usual
struct WhiteApple;

#[item_behavior]
impl ItemBehavior for WhiteApple
{
    fn onRightClick(&self, world: World, player: Player, _stack: ItemStack) -> Use
    {
        crate::player::cure(Entity { world, id: player.id }, APPLE_CURE);
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

fn oneLess(stack: ItemStack) -> Option<ItemStack>
{
    let remaining: i32 = stack.count - 1;
    (remaining > 0).then_some(ItemStack { count: remaining, ..stack })
}

// ItemAmbrosiumShard: a shard enchants the aether grass it is used on, and eaten out of hand it heals half a heart
struct AmbrosiumShard;

#[item_behavior]
impl ItemBehavior for AmbrosiumShard
{
    // any block takes the click, so a shard is only eaten when used on nothing
    fn onUseOnBlock(&self, world: World, _player: Player, stack: ItemStack, pos: BlockPos, _face: i32) -> Use
    {
        if !world.is(pos, block::AETHER_GRASS)
        {
            return Use::Done(Some(stack));
        }

        if world.isAuthority()
        {
            let _ = world.place(pos, block::ENCHANTED_AETHER_GRASS);
        }

        Use::Done(oneLess(stack))
    }

    fn onRightClick(&self, world: World, player: Player, stack: ItemStack) -> Use
    {
        let entity: Entity = Entity { world, id: player.id };
        let health: i32 = entity.field(EntityField::Health) as i32;
        if health <= 0 || health >= entity.field(EntityField::MaxHealth) as i32
        {
            return Use::Pass;
        }

        if world.isAuthority()
        {
            entity.setField(EntityField::Health, f64::from(health + SHARD_HEAL));
        }

        Use::Done(oneLess(stack))
    }
}

// ItemSwettyBall: a swet ball grows grass over the dirt it is used on
struct SwetBall;

#[item_behavior]
impl ItemBehavior for SwetBall
{
    fn onUseOnBlock(&self, world: World, _player: Player, stack: ItemStack, pos: BlockPos, _face: i32) -> Use
    {
        let Some((_, grass)) = SWET_GROWS.iter().find(|(dirt, _)| world.is(pos, dirt))
        else
        {
            return Use::Pass;
        };

        if world.isAuthority()
        {
            let _ = world.place(pos, grass);
        }

        Use::Done(oneLess(stack))
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
        let mut spec: ItemSpecBuilder = ItemSpecBuilder::new()
            .texture(&texture(&format!("weapons/{name}")))
            .tool(ToolClass::Sword, tier.m_level, tier.m_speed, baseDamage(ToolClass::Sword, tier), tier.m_uses)
            .tag(SWORD_TAG);

        if let Some(blade) = crate::weapons::blade(&name)
        {
            spec = spec.behavior(blade);
        }

        ctx.items().register(&name, spec)?;
    }

    Ok(())
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for (name, path) in MATERIALS
    {
        let spec: ItemSpecBuilder = ItemSpecBuilder::new().texture(&texture(path));
        let spec: ItemSpecBuilder = match name
        {
            item::AMBROSIUM_SHARD => spec.behavior(AmbrosiumShard),
            item::SWET_BALL => spec.behavior(SwetBall),
            _ => spec,
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

    for (name, icon, stack, container) in SINGLES
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

        if let Some(container) = container
        {
            spec.m_spec.container_item = Some(Identifier::parseIn(container, "aether")?);
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
        else if name == "aether_portal_frame"
        {
            spec = spec.behavior(crate::portal::FrameKit);
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
