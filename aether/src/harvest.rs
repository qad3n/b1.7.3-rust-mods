use crate::names::{self, block, item};
use b173_api::prelude::*;

// a block a player placed saves placed=true, only blocks the world grew give skyroot tools a second drop
const PLACED: &str = "placed";
// golden dungeon grass is neither grown nor placed and never doubles
const DUNGEON: &str = "dungeon";

// blocks that remember being placed by a player, aether grass also remembers the golden dungeon marking it
pub(crate) fn registerStates(ctx: &mut RegisterCtx, blocks: &[&str]) -> Result<()>
{
    for name in blocks
    {
        let flags: &[&str] = if *name == block::AETHER_GRASS
        {
            &[DUNGEON, PLACED]
        }
        else
        {
            &[PLACED]
        };
        ctx.blocks().states(name, names::states(flags.iter().map(|flag| names::flag(flag)).collect()))?;
    }

    Ok(())
}

// BlockAetherLog: the stronger axes chip one or two golden amber out of golden oak
const AMBER_MIN: i32 = 1;
const AMBER_SPREAD: i32 = 2;
const ONCE: i32 = 1;
const TWICE: i32 = 2;

fn holds(player: Player, name: &str) -> bool { player.held().is_some_and(|stack| stack.item.is(name)) }

fn skyroot(class: &str) -> String { format!("skyroot_{class}") }

fn natural(ctx: &BlockCtx) -> bool { ctx.stateAt().is_some_and(|state| !state.flag(PLACED) && !state.flag(DUNGEON)) }

fn drop(ctx: &BlockCtx, times: i32)
{
    for _ in 0..times
    {
        let _ = ctx.world().dropBlockItems(ctx.pos(), ctx.block(), i32::from(ctx.meta()));
    }
}

fn placedByPlayer(ctx: &mut BlockCtx, entity: Entity)
{
    if ctx.isAuthority() && entity.info().is_some_and(|info| info.player.is_some())
    {
        names::setProperty(ctx, PLACED, true);
    }
}

// DoubleDropHelper.dropBlock: the matching skyroot tool drops a grown block twice
pub(crate) fn doubled(ctx: &mut BlockCtx, player: Player, class: &str)
{
    let twice: bool = natural(ctx) && holds(player, &skyroot(class));
    drop(ctx, if twice { TWICE } else { ONCE });
}

pub(crate) fn placed(ctx: &mut BlockCtx, entity: Entity) { placedByPlayer(ctx, entity) }

// holystone, aether dirt and quicksoil
pub(crate) struct DoubleDrop
{
    pub m_tool: &'static str,
}

#[block_behavior]
impl BlockBehavior for DoubleDrop
{
    fn onPlacedBy(&self, ctx: &mut BlockCtx, entity: Entity) { placedByPlayer(ctx, entity) }

    fn onHarvest(&self, ctx: &mut BlockCtx, player: Player) { doubled(ctx, player, self.m_tool) }
}

// BlockAetherOre.harvestBlock: a skyroot pickaxe drops the ore twice, placed or not
pub(crate) struct Ore;

#[block_behavior]
impl BlockBehavior for Ore
{
    fn onHarvest(&self, ctx: &mut BlockCtx, player: Player)
    {
        drop(
            ctx,
            if holds(player, &skyroot("pickaxe"))
            {
                TWICE
            }
            else
            {
                ONCE
            },
        );
    }
}

// BlockAetherLog.harvestBlock: skyroot axes double grown logs, strong axes free amber from golden oak
pub(crate) struct Log;

#[block_behavior]
impl BlockBehavior for Log
{
    fn onPlacedBy(&self, ctx: &mut BlockCtx, entity: Entity) { placedByPlayer(ctx, entity) }

    fn onHarvest(&self, ctx: &mut BlockCtx, player: Player)
    {
        let world: World = ctx.world();
        let strong: bool = player.held().is_some_and(|stack| stack.item.tagged(crate::tags::AMBER_AXES));
        if strong && ctx.is(block::GOLDEN_OAK_LOG)
        {
            let amber: i32 = AMBER_MIN + world.random(AMBER_SPREAD);
            let pos: BlockPos = ctx.pos();
            let _ = world.spawnItem(
                [f64::from(pos.x), f64::from(pos.y), f64::from(pos.z)],
                ItemStack::of(item::GOLDEN_AMBER, amber),
            );
        }

        let twice: bool = natural(ctx) && holds(player, &skyroot("axe"));
        drop(ctx, if twice { TWICE } else { ONCE });
    }
}
