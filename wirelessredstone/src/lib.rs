use b173_api::prelude::*;

mod blocks;
mod ether;
mod items;
mod labels;
mod math;
mod net;
mod recipes;
mod screens;
mod triangulator;

fn registerEvents(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.events().on::<TickPost>(Priority::Normal, |_|
    {
        net::tickPulses();

        if isClient()
        {
            screens::tick();
            triangulator::tick();
        }
    })?;

    ctx.events().on::<ChunkUnload>(Priority::Normal, |event|
    {
        if let (Some(world), EventData::Chunk { x, z, .. }) = (event.world(), event.data())
            && world.isAuthority()
        {
            ether::unloadChunk(world, *x, *z);
        }
    })?;

    ctx.events().on::<WorldUnload>(Priority::Normal, |event|
    {
        if let Some(world) = event.world()
        {
            ether::clear(world);
        }
    })
}

#[mod_main]
fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    blocks::register(ctx)?;
    items::register(ctx)?;
    recipes::register(ctx)?;
    screens::register(ctx)?;
    net::register(ctx)?;
    registerEvents(ctx)
}
