use crate::blocks::{RESIN, Sap, resinAt, rubberWoodTickRate, setResin, tapChance, tapSound};
use crate::core::{dir, itemTexture, own, sound, stacks};
use b173_api::prelude::*;

// a plain ItemIC2 or ItemGradual: its stack size, damage range and the shared tags it joins
struct Plain
{
    m_name: &'static str,
    m_stack: Option<i32>,
    m_damage: Option<i32>,
    m_tags: &'static [&'static str],
    // the item whose sprite it draws with, as the source gives both one icon index
    m_sprite: Option<&'static str>,
}

const fn plainItem(name: &'static str, tags: &'static [&'static str]) -> Plain
{
    Plain { m_name: name, m_stack: None, m_damage: None, m_tags: tags, m_sprite: None }
}

const fn sharing(name: &'static str, sprite: &'static str) -> Plain
{
    Plain { m_name: name, m_stack: None, m_damage: None, m_tags: &[], m_sprite: Some(sprite) }
}

// ItemGradual holds one and wears down over ten thousand points, the reactor counts its life in them
const fn gradual(name: &'static str) -> Plain
{
    const LIFE: i32 = 10000;

    Plain { m_name: name, m_stack: Some(1), m_damage: Some(LIFE), m_tags: &[], m_sprite: None }
}

const fn single(name: &'static str) -> Plain
{
    Plain { m_name: name, m_stack: Some(1), m_damage: None, m_tags: &[], m_sprite: None }
}

const PLAIN: [Plain; 54] = [
    plainItem("coal_dust", &["c:dusts/coal"]),
    plainItem("iron_dust", &["c:dusts/iron"]),
    plainItem("gold_dust", &["c:dusts/gold"]),
    plainItem("copper_dust", &["c:dusts/copper"]),
    plainItem("tin_dust", &["c:dusts/tin"]),
    plainItem("bronze_dust", &["c:dusts/bronze"]),
    plainItem("small_iron_dust", &["c:tiny_dusts/iron"]),
    plainItem("refined_iron", &["c:ingots/refined_iron"]),
    plainItem("copper_ingot", &["c:ingots/copper"]),
    plainItem("tin_ingot", &["c:ingots/tin"]),
    plainItem("bronze_ingot", &["c:ingots/bronze"]),
    plainItem("mixed_metal_ingot", &[]),
    plainItem("refined_uranium", &["c:ingots/uranium"]),
    plainItem("uranium_chunk", &["c:raw_materials/uranium"]),
    plainItem("single_use_battery", &[]),
    plainItem("lava_cell", &[]),
    plainItem("fuel_can_empty", &[]),
    plainItem("coal_cell", &[]),
    plainItem("bio_cell", &[]),
    sharing("coalfuel_cell", "coal_cell"),
    sharing("biofuel_cell", "bio_cell"),
    plainItem("hydrated_coal_dust", &[]),
    plainItem("hydrated_coal", &[]),
    plainItem("plantball", &[]),
    plainItem("compressed_plants", &[]),
    plainItem("tin_can", &[]),
    plainItem("water_cell", &[]),
    plainItem("rubber", &["c:rubbers"]),
    plainItem("electronic_circuit", &["c:circuits/basic"]),
    plainItem("advanced_circuit", &["c:circuits/advanced"]),
    plainItem("scrap", &[]),
    plainItem("uu_matter", &[]),
    plainItem("advanced_alloy", &["c:plates/advanced_alloy"]),
    plainItem("industrial_credit", &[]),
    plainItem("electrolyzed_water_cell", &[]),
    plainItem("raw_carbon_fibre", &[]),
    plainItem("raw_carbon_mesh", &[]),
    plainItem("carbon_plate", &["c:plates/carbon"]),
    plainItem("iridium_plate", &["c:plates/iridium"]),
    plainItem("tfbp", &[]),
    plainItem("coal_ball", &[]),
    plainItem("compressed_coal_ball", &[]),
    plainItem("coal_chunk", &[]),
    plainItem("industrial_diamond", &[]),
    plainItem("re_enriched_uranium_cell", &[]),
    plainItem("near_depleted_uranium_cell", &[]),
    plainItem("painter", &[]),
    single("frequency_transmitter"),
    gradual("uranium_cell"),
    gradual("coolant_cell"),
    gradual("reactor_plating"),
    gradual("heat_disperser"),
    gradual("depleted_isotope_cell"),
    plainItem("sticky_resin", &["c:slimeballs/resin"]),
];

fn registerPlain(ctx: &mut RegisterCtx) -> Result<()>
{
    for row in &PLAIN
    {
        let mut spec: ItemSpecBuilder =
            ItemSpecBuilder::new().texture(&itemTexture(row.m_sprite.unwrap_or(row.m_name)));

        if let Some(stack) = row.m_stack
        {
            spec = spec.maxStack(stack);
        }

        if let Some(damage) = row.m_damage
        {
            spec = spec.maxDamage(damage);
        }

        for tag in row.m_tags
        {
            spec = spec.tag(tag);
        }

        if row.m_name == "sticky_resin"
        {
            spec = spec.behavior(ResinItem);
        }

        ctx.items().register(row.m_name, spec)?;
    }

    Ok(())
}

// ItemResin: placed as a puddle on top of a block
struct ResinItem;

#[item_behavior]
impl ItemBehavior for ResinItem
{
    fn onUseOnBlock(&self, world: World, _player: Player, stack: ItemStack, pos: BlockPos, face: i32) -> Use
    {
        let above: BlockPos = pos.offset(0, 1, 0);
        if face != dir::UP || !world.isAir(above) || !crate::blocks::resinHolds(world, above)
        {
            return Use::Pass;
        }

        if let Err(error) = world.place(above, RESIN)
        {
            warn(&format!("could not place resin at {above:?}: {error}"));
            return Use::Pass;
        }

        Use::Done(stacks::less(stack, 1))
    }
}

mod tap
{
    pub const USES: i32 = 16;
    // a fresh hole gives one to three resin, a dry one still gives one now and then
    pub const FRESH_SPREAD: i32 = 3;
    pub const OFFSET: f64 = 0.3;
}

fn ejectResin(world: World, pos: BlockPos, face: i32, count: i32)
{
    let [dx, _, dz] = dir::offset(face);
    let [x, y, z] = dir::center(pos);
    let at: [f64; 3] = [x + f64::from(dx) * tap::OFFSET, y, z + f64::from(dz) * tap::OFFSET];

    for _ in 0..count
    {
        if let Err(error) = world.spawnItem(at, ItemStack::of(own("sticky_resin").as_str(), 1))
        {
            warn(&format!("the treetap could not drop resin: {error}"));
        }
    }
}

// ItemTreetap.onItemUse: tapping a rubber log's resin hole on its own face
struct Treetap;

#[item_behavior]
impl ItemBehavior for Treetap
{
    fn onUseOnBlock(&self, world: World, _player: Player, stack: ItemStack, pos: BlockPos, face: i32) -> Use
    {
        let Some((sap, hole)) = resinAt(world, pos)
        else
        {
            return Use::Pass;
        };

        if !matches!(sap, Sap::Wet | Sap::Dry) || hole != face
        {
            return Use::Pass;
        }

        if sap == Sap::Wet
        {
            setResin(world, pos, Sap::Dry);
            if world.isAuthority()
            {
                ejectResin(world, pos, face, world.random(tap::FRESH_SPREAD) + 1);
                tapSound(world, pos);
            }

            let _ = world.scheduleTick(pos, world.block(pos), rubberWoodTickRate());
        }
        else
        {
            if tapChance(world)
            {
                setResin(world, pos, Sap::Plain);
            }

            if tapChance(world) && world.isAuthority()
            {
                ejectResin(world, pos, face, 1);
                tapSound(world, pos);
            }
        }

        Use::Done(wear(stack, 1))
    }
}

// the shared material tag an IC2 item joins, such as c:ingots/copper, which recipes take instead of the item
pub(crate) fn materialTag(item: &str) -> Option<&'static str>
{
    let name: &str = item.strip_prefix("ic2:")?;
    PLAIN.iter().find(|row| row.m_name == name).and_then(|row| row.m_tags.first().copied())
}

// Item.damageItem: the stack breaks once worn past its last point
pub(crate) fn wear(stack: ItemStack, amount: i32) -> Option<ItemStack>
{
    let max: i32 = maxDamage(&stack.item);
    let damage: i32 = stack.damage + amount;
    if max > 0 && damage > max
    {
        return stacks::less(ItemStack { damage: 0, ..stack }, 1);
    }

    Some(ItemStack { damage, ..stack })
}

mod cell
{
    pub const REACH: f64 = 5.0;
    pub const EYE: f64 = 1.62;
}

// ItemCell.onItemRightClick: an empty cell scoops a still water or lava source the player looks at
struct EmptyCell;

#[item_behavior]
impl ItemBehavior for EmptyCell
{
    fn onRightClick(&self, world: World, player: Player, stack: ItemStack) -> Use
    {
        let Some(info) = player.info()
        else
        {
            return Use::Pass;
        };
        let eye: [f64; 3] = [info.pos[0], info.pos[1] + cell::EYE, info.pos[2]];
        let look: [f64; 3] = crate::core::math::look(info.yaw, info.pitch);
        let to: [f64; 3] =
            [eye[0] + look[0] * cell::REACH, eye[1] + look[1] * cell::REACH, eye[2] + look[2] * cell::REACH];
        let Some(hit) = world.rayTrace(eye, to, true)
        else
        {
            return Use::Pass;
        };
        let pos: BlockPos = hit.pos;
        // any fluid some cell holds, as the loader's fluid containers name them
        let Some((filled, _)) =
            world.fluidAt(pos).filter(|fluid| fluid.source).and_then(|fluid| fill(&stack, &fluid.fluid.to_string()))
        else
        {
            return Use::Pass;
        };

        let _ = world.clear(pos);
        if let Err(error) = player.give(filled)
        {
            warn(&format!("could not give a filled cell: {error}"));
        }

        Use::Done(stacks::less(stack, 1))
    }
}

// ItemTinCan.onItemRightClick: half a heart at once, the empty can goes back into the inventory
struct FilledCan;

#[item_behavior]
impl ItemBehavior for FilledCan
{
    fn onRightClick(&self, world: World, player: Player, stack: ItemStack) -> Use
    {
        const HEAL: f64 = 2.0;

        if !world.isAuthority()
        {
            return Use::Done(Some(stack));
        }

        let Some(info) = player.info()
        else
        {
            return Use::Pass;
        };
        let entity: Entity = Entity { world, id: info.id };
        let health: f64 = entity.field(EntityField::Health);
        let max: f64 = entity.field(EntityField::MaxHealth);
        entity.setField(EntityField::Health, (health + HEAL).min(max));

        if let Err(error) = player.give(ItemStack::of(own("tin_can").as_str(), 1))
        {
            warn(&format!("could not return the empty tin can: {error}"));
        }

        Use::Done(stacks::less(stack, 1))
    }
}

// ItemScrapbox.randomizeDropID: thirty three equally likely things, dirt twice
const SCRAP_DROPS: [&str; 33] = [
    "minecraft:dirt",
    "minecraft:dirt",
    "minecraft:iron_ore",
    "minecraft:gold_ore",
    "minecraft:sandstone",
    "minecraft:netherrack",
    "minecraft:pumpkin",
    "minecraft:soul_sand",
    "minecraft:apple",
    "minecraft:wooden_hoe",
    "minecraft:bread",
    "minecraft:golden_helmet",
    "minecraft:cooked_porkchop",
    "minecraft:sign",
    "minecraft:minecart",
    "minecraft:leather",
    "minecraft:slime_ball",
    "minecraft:glowstone_dust",
    "minecraft:redstone",
    "minecraft:cake",
    "minecraft:diamond",
    "ic2:copper_ore",
    "ic2:tin_ore",
    "ic2:coal_dust",
    "ic2:iron_dust",
    "ic2:gold_dust",
    "ic2:tin_dust",
    "ic2:copper_dust",
    "ic2:rubber",
    "ic2:plantball",
    "ic2:filled_tin_can",
    "ic2:single_use_battery",
    "minecraft:grass",
];

const SCRAP_BOX: &str = "scrap_box";

// the scrap box's drops as a weighted loot table, one pick a box and duplicate rows as extra weight, so opening a box
// rolls it and other mods add their own drops
fn registerScrap(ctx: &mut RegisterCtx) -> Result<()>
{
    const PICK: i32 = 1;

    let pool: PoolBuilder =
        SCRAP_DROPS.iter().try_fold(PoolBuilder::new(PICK, PICK), |pool, drop| pool.entry(drop, 1, 1, 1))?;
    ctx.loot().pool(LootTarget::Chest, SCRAP_BOX, pool.m_spec)
}

// ItemScrapbox.randomizeDropID, one roll of the scrap box table
fn scrapDrop(world: World) -> Option<ItemStack>
{
    let table: Identifier = Identifier::parse(&own(SCRAP_BOX)).ok()?;
    rollLoot(&table, i64::from(world.random(i32::MAX))).into_iter().next()
}

struct ScrapBox;

#[item_behavior]
impl ItemBehavior for ScrapBox
{
    fn onRightClick(&self, world: World, player: Player, stack: ItemStack) -> Use
    {
        const TOSS: f64 = 0.3;

        if !world.isAuthority()
        {
            return Use::Done(Some(stack));
        }

        let Some(thing) = scrapDrop(world)
        else
        {
            return Use::Done(Some(stack));
        };

        if let Some(info) = player.info()
        {
            let look: [f64; 3] = crate::core::math::look(info.yaw, info.pitch);
            let at: [f64; 3] = [info.pos[0], info.pos[1] + cell::EYE - TOSS, info.pos[2]];
            let motion: [f64; 3] = [look[0] * TOSS, look[1] * TOSS, look[2] * TOSS];
            if let Err(error) = world.dropItem(at, &thing, motion, 40)
            {
                warn(&format!("the scrap box could not drop {:?}: {error}", thing.item));
            }
        }

        Use::Done(stacks::less(stack, 1))
    }
}

pub(crate) const COLORS: [&str; 16] = [
    "black",
    "red",
    "green",
    "brown",
    "blue",
    "purple",
    "cyan",
    "light_gray",
    "gray",
    "pink",
    "lime",
    "yellow",
    "light_blue",
    "magenta",
    "orange",
    "white",
];

// ItemToolPainter: sixteen coats of one color on cables, then the plain painter is left
struct Painter
{
    m_color: usize,
}

mod paint
{
    pub const COATS: i32 = 16;
}

#[item_behavior]
impl ItemBehavior for Painter
{
    fn onUseOnBlock(&self, world: World, _player: Player, stack: ItemStack, pos: BlockPos, _face: i32) -> Use
    {
        if !crate::energy::cable::paint(world, pos, self.m_color)
        {
            return Use::Pass;
        }

        world.playSound(dir::center(pos), &sound("painter"), 1.0, 1.0);
        if stack.damage >= paint::COATS - 1
        {
            return Use::Done(Some(ItemStack::of(own("painter").as_str(), 1)));
        }

        Use::Done(Some(ItemStack { damage: stack.damage + 1, ..stack }))
    }
}

fn registerTools(ctx: &mut RegisterCtx) -> Result<()>
{
    const CELL_STACK: i32 = 64;
    const SCRAP_STACK: i32 = 64;

    ctx.items().register(
        "treetap",
        ItemSpecBuilder::new().texture(&itemTexture("treetap")).maxStack(1).maxDamage(tap::USES).behavior(Treetap),
    )?;
    ctx.items().register(
        "empty_cell",
        ItemSpecBuilder::new().texture(&itemTexture("empty_cell")).maxStack(CELL_STACK).behavior(EmptyCell),
    )?;
    ctx.items().register(
        "filled_tin_can",
        ItemSpecBuilder::new().texture(&itemTexture("filled_tin_can")).behavior(FilledCan),
    )?;
    ctx.items().register(
        "scrap_box",
        ItemSpecBuilder::new().texture(&itemTexture("scrap_box")).maxStack(SCRAP_STACK).behavior(ScrapBox),
    )?;
    registerScrap(ctx)?;

    for (color, name) in COLORS.iter().enumerate()
    {
        let name: String = format!("painter_{name}");
        ctx.items().register(
            &name,
            ItemSpecBuilder::new()
                .texture(&itemTexture(&name))
                .maxStack(1)
                .maxDamage(paint::COATS)
                .behavior(Painter { m_color: color }),
        )?;
    }

    // ItemIC2 with a container: a burnt out fuel can is handed back empty
    let mut fuelCan: ItemSpecBuilder = ItemSpecBuilder::new().texture(&itemTexture("fuel_can")).maxStack(1);
    fuelCan.m_spec.container_item = Some(Identifier::parse(&own("fuel_can_empty"))?);
    ctx.items().register("fuel_can", fuelCan)?;
    Ok(())
}

// the bronze tools take iron's tier with 350 uses, ItemArmorIC2 multiplies the leather durability per piece
mod bronze
{
    pub const LEVEL: i32 = 2;
    pub const SPEED: f32 = 6.0;
    pub const USES: i32 = 350;
    pub const ARMOR_MULTIPLIER: i32 = 15;
    pub const VEST_MULTIPLIER: i32 = 50;
}

// ItemArmor.damageReduceAmountArray and maxDamageArray, helmet first
pub(crate) const ARMOR_REDUCE: [i32; 4] = [3, 8, 6, 3];
pub(crate) const ARMOR_DURABILITY: [i32; 4] = [11, 16, 15, 13];
pub(crate) const ARMOR_SLOTS: [ArmorSlot; 4] = [ArmorSlot::Head, ArmorSlot::Chest, ArmorSlot::Legs, ArmorSlot::Feet];

pub(crate) fn armor(slot: usize, durability: i32, layer: &str) -> ArmorSpec
{
    ArmorSpec
    {
        slot: ARMOR_SLOTS[slot],
        protection: ARMOR_REDUCE[slot],
        durability,
        texture: format!("ic2:armor/{layer}_layer"),
        color: None,
    }
}

fn registerBronze(ctx: &mut RegisterCtx) -> Result<()>
{
    // beta's iron tool damage, ItemIC2Sword's own weaponDamage of 7 shadows the private one ItemSword reads
    for (name, class, damage) in [
        ("bronze_pickaxe", ToolClass::Pickaxe, 4),
        ("bronze_axe", ToolClass::Axe, 5),
        ("bronze_sword", ToolClass::Sword, 6),
        ("bronze_shovel", ToolClass::Shovel, 3),
        ("bronze_hoe", ToolClass::Hoe, 1),
    ]
    {
        ctx.items().register(
            name,
            ItemSpecBuilder::new()
                .texture(&itemTexture(name))
                .tool(class, bronze::LEVEL, bronze::SPEED, damage, bronze::USES)
                .maxDamage(bronze::USES),
        )?;
    }

    for (slot, piece) in ["helmet", "chestplate", "leggings", "boots"].iter().enumerate()
    {
        let name: String = format!("bronze_{piece}");
        let durability: i32 = ARMOR_DURABILITY[slot] * bronze::ARMOR_MULTIPLIER;
        ctx.items().register(
            &name,
            ItemSpecBuilder::new().texture(&itemTexture(&name)).armor(armor(slot, durability, "bronze")),
        )?;
    }

    let vest: i32 = ARMOR_DURABILITY[1] * bronze::VEST_MULTIPLIER;
    ctx.items().register(
        "composite_vest",
        ItemSpecBuilder::new().texture(&itemTexture("composite_vest")).armor(armor(1, vest, "alloy")),
    )?;
    Ok(())
}

// the cells that carry a fluid and drain back to an empty cell
const CELLS: [(&str, &str); 2] = [("water_cell", "minecraft:water"), ("lava_cell", "minecraft:lava")];

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    const BUCKET: i32 = 1000;

    registerPlain(ctx)?;

    for (cell, fluid) in CELLS
    {
        ctx.fluids().container(cell, Some("empty_cell"), fluid, BUCKET)?;
    }

    registerTools(ctx)?;
    registerBronze(ctx)
}

// ItemFood.getHealAmount for beta's foods, the canner fills a tin per two points
// the food the canner fills cans from with its healing
pub(crate) const FOODS: [(&str, i32); 10] = [
    ("minecraft:apple", 4),
    ("minecraft:mushroom_stew", 10),
    ("minecraft:bread", 5),
    ("minecraft:porkchop", 3),
    ("minecraft:cooked_porkchop", 8),
    ("minecraft:golden_apple", 42),
    ("minecraft:fish", 2),
    ("minecraft:cooked_fish", 5),
    ("minecraft:cookie", 1),
    ("ic2:filled_tin_can", 2),
];
