mod blank;

use b173_api::prelude::*;
use b173_api::vanilla::entity;

pub(crate) const DISC_TAG: &str = "c:music_discs";
const SKELETON_KILL: &str = "killed_by_skeleton";
const GUNPOWDER: &str = "minecraft:gunpowder";

// each name is also the streaming sound beta already loads from resources/streaming
const DISCS: [&str; 9] = ["blocks", "chirp", "far", "mall", "mellohi", "stal", "strad", "ward", "11"];
const BETA_DISCS: [&str; 2] = ["minecraft:record_13", "minecraft:record_cat"];

#[mod_main]
fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for disc in DISCS
    {
        let spec: ItemSpecBuilder = ItemSpecBuilder::new()
            .texture(&format!("discs:item/record_{disc}"))
            .tag(DISC_TAG)
            .record(disc, &format!("C418 - {disc}"));
        ctx.items().register(&format!("record_{disc}"), spec)?;
    }

    creeper(ctx)?;
    blank::register(ctx)
}

fn skeleton(query: &LootQuery) -> bool
{
    let kind: Option<Identifier> = query.killer.and_then(Entity::info).and_then(|info| info.kind);
    kind.is_some_and(|kind| Identifier::parse(entity::SKELETON).is_ok_and(|skeleton| skeleton == kind))
}

// beta creeper drops replaced by its gunpowder and one disc of all eleven when a skeleton made the kill
fn creeper(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.loot().condition(SKELETON_KILL, "When a skeleton's arrow kills it", skeleton)?;
    ctx.loot().entity(entity::CREEPER, LootSpec { replace: true, entries: Vec::new() })?;

    let gunpowder: PoolBuilder = PoolBuilder::new(1, 1).entry(GUNPOWDER, 1, 0, 2)?;
    ctx.loot().pool(LootTarget::Entity, entity::CREEPER, gunpowder.m_spec)?;

    let mut discs: PoolBuilder = PoolBuilder::new(1, 1);
    for disc in BETA_DISCS
    {
        discs = discs.entry(disc, 1, 1, 1)?.when(SKELETON_KILL)?;
    }

    for disc in DISCS
    {
        discs = discs.entry(&format!("record_{disc}"), 1, 1, 1)?.when(SKELETON_KILL)?;
    }

    ctx.loot().pool(LootTarget::Entity, entity::CREEPER, discs.m_spec)
}
