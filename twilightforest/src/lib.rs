mod blocks;
mod bosses;
mod charms;
mod colors;
mod combat;
mod config;
mod effects;
mod entities;
mod hint_books;
mod hints;
mod ids;
mod items;
mod machines;
mod magic;
mod math;
mod models;
mod particles;
mod portal;
mod progress;
mod protection;
mod recipes;
mod tower;
mod treasure;
mod weather;
mod world;

use b173_api::prelude::*;

#[mod_main]
fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    config::register(ctx)?;
    effects::register(ctx)?;
    particles::register(ctx)?;
    blocks::register(ctx)?;
    items::register(ctx)?;
    recipes::register(ctx)?;
    machines::register(ctx)?;
    bosses::register(ctx)?;
    // the dimension and its biomes come before anything that spawns in them
    world::register(ctx)?;
    portal::register(ctx)?;
    progress::register(ctx)?;
    protection::register(ctx)?;
    hints::register(ctx)?;
    weather::register(ctx)?;
    entities::register(ctx)?;
    charms::register(ctx)
}
