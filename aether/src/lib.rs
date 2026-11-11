mod abilities;
mod accessories;
mod blocks;
mod config;
mod damage;
mod effects;
mod entities;
mod gear;
mod harvest;
mod ids;
mod items;
mod lore;
mod machines;
mod models;
mod player;
mod portal;
mod portal_overlay;
mod progress;
mod recipes;
mod shapes;
mod sparkle;
mod sun_altar;
mod tools;
mod weapons;
mod world;

use b173_api::prelude::*;

#[mod_main]
fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    config::register(ctx)?;
    effects::register(ctx)?;
    blocks::register(ctx)?;
    shapes::register(ctx)?;
    sun_altar::register(ctx)?;
    items::register(ctx)?;
    lore::register(ctx)?;
    accessories::register(ctx)?;
    abilities::register(ctx)?;
    recipes::register(ctx)?;
    world::register(ctx)?;
    entities::register(ctx)?;
    machines::register(ctx)?;
    portal::registerEvents(ctx)?;
    portal_overlay::register(ctx)?;
    player::register(ctx)?;
    tools::register(ctx)?;
    gear::register(ctx)?;
    progress::register(ctx)?;
    Ok(())
}
