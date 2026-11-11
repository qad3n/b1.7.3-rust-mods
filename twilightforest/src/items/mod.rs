mod bows;
mod gear;
pub(crate) mod magic;
mod maps;
pub(crate) mod moonworm;
mod powder;
mod scepters;
mod thrown;

use crate::names::item;
use b173_api::prelude::*;
pub(crate) use gear::{armorName, armorWorn};

// InventoryPlayer keeps boots, leggings, chestplate and helmet after the 36 main slots
pub(crate) const ARMOR_FIRST_SLOT: i32 = 36;

// a plain item with an icon and a name, ItemTF and ItemTFMeta
struct Plain
{
    m_name: &'static str,
    m_label: &'static str,
    m_texture: &'static str,
    m_stack: i32,
}

const fn plain(name: &'static str, label: &'static str, texture: &'static str) -> Plain
{
    Plain { m_name: name, m_label: label, m_texture: texture, m_stack: STACK }
}

const STACK: i32 = 64;

const MATERIALS: [Plain; 26] = [
    plain(item::NAGA_SCALE, "Naga Scale", "nagascale"),
    plain(item::RAVEN_FEATHER, "Raven's Feather", "tffeather"),
    plain(item::MAGIC_MAP_FOCUS, "Magic Map Focus", "magicmapfocus"),
    plain(item::MAZE_MAP_FOCUS, "Maze Map Focus", "mazemapfocus"),
    plain(item::LIVEROOT, "Liveroot", "liveroot"),
    plain(item::RAW_IRONWOOD, "Raw Ironwood Materials", "ironwoodraw"),
    plain(item::IRONWOOD_INGOT, "Ironwood Ingot", "ironwoodingot"),
    plain(item::TORCHBERRIES, "Torchberries", "torchberries"),
    plain(item::FIERY_BLOOD, "Fiery Blood", "fieryblood"),
    plain(item::FIERY_TEARS, "Fiery Tears", "fierytears"),
    plain(item::FIERY_INGOT, "Fiery Ingot", "fieryingot"),
    plain(item::STEELEAF, "Steeleaf", "steeleafingot"),
    plain(item::BORER_ESSENCE, "Borer Essence", "boreressence"),
    plain(item::CARMINITE, "Carminite", "carminite"),
    plain(item::ARMOR_SHARD, "Armor Shard", "armorshards"),
    plain(item::KNIGHTMETAL, "Knightmetal Ingot", "knightmetal"),
    plain(item::SHARD_CLUSTER, "Armor Shard Cluster", "shardcluster"),
    plain(item::ALPHA_FUR, "Alpha Yeti Fur", "alphafur"),
    plain(item::ARCTIC_FUR, "Arctic Fur", "arcticfur"),
    plain(item::KNIGHTMETAL_RING, "Knightmetal Loop", "knightmetalring"),
    plain(item::CUBE_TALISMAN, "Talisman of the Cube", "cubetalisman"),
    plain(item::ADHERENT_FRAGMENT, "Cloth Of The Adherent", "adherentfragment"),
    plain(item::HARBINGER_FRAGMENT, "Fragment Of The Harbinger", "harbingerfragment"),
    plain(item::CHARM_OF_LIFE_1, "Charm of Life I", "charmoflife1"),
    plain(item::CHARM_OF_LIFE_2, "Charm of Life II", "charmoflife2"),
    plain(item::CHARM_OF_KEEPING_1, "Charm of Keeping I", "charmofkeeping1"),
];

const KEEPING: [Plain; 2] = [
    plain(item::CHARM_OF_KEEPING_2, "Charm of Keeping II", "charmofkeeping2"),
    plain(item::CHARM_OF_KEEPING_3, "Charm of Keeping III", "charmofkeeping3"),
];

// ItemTFFood heal values are hunger points, a beta food heals that many half hearts at once
struct Food
{
    m_name: &'static str,
    m_label: &'static str,
    m_texture: &'static str,
    m_heal: i32,
}

const FOODS: [Food; 8] = [
    Food { m_name: item::RAW_VENISON, m_label: "Raw Venison", m_texture: "venisonraw", m_heal: 3 },
    Food { m_name: item::VENISON, m_label: "Venison Steak", m_texture: "venisoncooked", m_heal: 8 },
    Food { m_name: item::HYDRA_CHOP, m_label: "Hydra Chop", m_texture: "hydrachop", m_heal: 18 },
    Food { m_name: item::RAW_MEEF, m_label: "Raw Meef", m_texture: "meefraw", m_heal: 2 },
    Food { m_name: item::MEEF_STEAK, m_label: "Meef Steak", m_texture: "meefsteak", m_heal: 6 },
    Food { m_name: item::MEEF_STROGANOFF, m_label: "Meef Stroganoff", m_texture: "meefstroganoff", m_heal: 8 },
    Food { m_name: item::MAZE_WAFER, m_label: "Maze Wafer", m_texture: "mazewafer", m_heal: 4 },
    Food { m_name: item::EXPERIMENT_115, m_label: "Experiment 115", m_texture: "experiment115", m_heal: 4 },
];

pub(crate) fn texture(name: &str) -> String { format!("twilightforest:item/{name}") }

pub(crate) fn put(ctx: &mut RegisterCtx, name: &str, spec: ItemSpecBuilder) -> Result<()>
{
    ctx.items().register(name, spec)?;
    Ok(())
}

fn registerPlain(ctx: &mut RegisterCtx, rows: &[Plain], stack: Option<i32>) -> Result<()>
{
    for row in rows
    {
        let spec: ItemSpecBuilder = ItemSpecBuilder::new()
            .name(row.m_label)
            .texture(&texture(row.m_texture))
            .maxStack(stack.unwrap_or(row.m_stack));
        put(ctx, row.m_name, spec)?;
    }

    Ok(())
}

// beta foods heal on the spot and never stack
fn registerFood(ctx: &mut RegisterCtx) -> Result<()>
{
    for row in &FOODS
    {
        put(
            ctx,
            row.m_name,
            ItemSpecBuilder::new().name(row.m_label).texture(&texture(row.m_texture)).food(row.m_heal),
        )?;
    }

    Ok(())
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    registerPlain(ctx, &MATERIALS, None)?;
    // charms of keeping beyond the first never stack in the source either
    registerPlain(ctx, &KEEPING, Some(1))?;
    registerFood(ctx)?;
    magic::register(ctx)?;
    scepters::register(ctx)?;
    bows::register(ctx)?;
    powder::register(ctx)?;
    moonworm::register(ctx)?;
    thrown::register(ctx)?;
    maps::register(ctx)?;
    gear::register(ctx)
}

// wears the held tool down by this many uses unless it is one of the exempt items
pub(crate) fn wear(player: Player, uses: i32, exempt: &[&str])
{
    let Some(stack) = player.held()
    else
    {
        return;
    };
    if exempt.iter().any(|name| stack.item.is(name))
    {
        return;
    }

    if let Err(error) = player.damageHeld(uses)
    {
        b173_api::warn(&format!("wearing a held tool failed: {}", error.0));
    }
}
