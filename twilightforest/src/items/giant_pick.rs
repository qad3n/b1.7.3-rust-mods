use crate::blocks::giant::cells;
use crate::names::{block, item};
use b173_api::prelude::*;
use std::cell::Cell;

const COBBLESTONE: &str = "minecraft:cobblestone";
// a pickaxe digs what it is made for faster than a bare hand does
const BARE_HAND: f32 = 1.0;

thread_local!
{
    // the block whose cobblestone becomes one giant cobblestone, TFEventListener.shouldMakeGiantCobble
    static COBBLE_AT: Cell<Option<(i32, BlockPos)>> = const { Cell::new(None) };
}

// TFEventListener.breakBlock: the giant's pickaxe takes every block like its target out of the four block cell the
// target sits in, each one wearing the pick, and a cell of nothing but cobblestone leaves one giant cobblestone
fn breakCell(event: &mut Event)
{
    if event.isCanceled()
    {
        return;
    }

    let EventData::Block { world, pos, block: broken, meta, player: Some(player) } = event.data().clone()
    else
    {
        return;
    };
    let world: World = World { dimension: world };
    let player: Player = Player { id: player };
    let wielding: bool = player.held().is_some_and(|held| held.item.is(item::GIANT_PICK));
    if !wielding || !world.isAuthority() || player.digSpeed(broken) <= BARE_HAND
    {
        return;
    }

    let allCobble: bool = cells(pos).all(|at| world.blockDropStack(at).is_some_and(|drop| drop.item.is(COBBLESTONE)));
    let twins: Vec<BlockPos> =
        cells(pos).filter(|at| *at != pos && world.block(*at) == broken && world.meta(*at) == meta).collect();

    for at in &twins
    {
        if !allCobble
        {
            let _ = world.dropBlockItems(*at, broken, meta);
        }

        let _ = world.clear(*at);
    }

    if let Err(error) = player.damageHeld(twins.len() as i32)
    {
        warn(&format!("giant pick wear: {error}"));
    }

    COBBLE_AT.set(allCobble.then_some((world.dimension, pos)));
}

// TFEventListener.harvestDrops: the cobblestone of the block that was hit turns into the giant cobblestone
fn giantDrop(event: &mut Event)
{
    let Some((dimension, at)) = COBBLE_AT.take()
    else
    {
        return;
    };
    let EventData::Drops { world, pos, drops, .. } = event.dataMut()
    else
    {
        return;
    };
    if *world != dimension || *pos != at || drops.is_empty()
    {
        return;
    }

    drops.remove(0);

    if let Ok(giant) = ItemStack::of(block::GIANT_COBBLE, 1).toRaw()
    {
        drops.push(giant);
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.events().on::<BlockBreak>(Priority::Low, breakCell)?;
    ctx.events().on::<BlockHarvestDrops>(Priority::Normal, giantDrop)
}
