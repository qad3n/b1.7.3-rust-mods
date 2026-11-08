use crate::ids::{block, item};
use b173_api::prelude::*;

// a block a player placed keeps metadata 1, only blocks the world grew give skyroot tools a second drop
const PLACED: u8 = 1;
const NATURAL: u8 = 0;
// the metadata above as a named state, values in metadata order so saved blocks keep their meaning
const PLACED_VALUES: [&str; 2] = ["false", "true"];

// blocks that remember being placed by a player save that as placed=true instead of bare metadata
pub fn registerStates(ctx: &mut RegisterCtx, names: &[&str]) -> Result<()>
{
    for name in names
    {
        let property: PropertySpec =
            PropertySpec { name: "placed".to_owned(), values: PLACED_VALUES.map(str::to_owned).to_vec() };
        ctx.blocks().states(name, BlockStatesSpec { properties: vec![property], ..BlockStatesSpec::default() })?;
    }
    Ok(())
}
// BlockAetherLog: the stronger axes chip one or two golden amber out of golden oak
const AMBER_MIN: i32 = 1;
const AMBER_SPREAD: i32 = 2;
const STRONG_AXES: [&str; 4] = ["zanite_axe", "gravitite_axe", "valkyrie_axe", "minecraft:diamond_axe"];
const ONCE: i32 = 1;
const TWICE: i32 = 2;

fn holds(player: Player, name: &str) -> bool { player.held().is_some_and(|stack| stack.item.is(name)) }

fn skyroot(class: &str) -> String { format!("skyroot_{class}") }

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
        ctx.setMeta(PLACED);
    }
}

// DoubleDropHelper.dropBlock: the matching skyroot tool drops a grown block twice
pub fn doubled(ctx: &mut BlockCtx, player: Player, class: &str)
{
    let twice: bool = ctx.meta() == NATURAL && holds(player, &skyroot(class));
    drop(ctx, if twice { TWICE } else { ONCE });
}

pub fn placed(ctx: &mut BlockCtx, entity: Entity) { placedByPlayer(ctx, entity) }

// holystone, aether dirt and quicksoil
pub struct DoubleDrop
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
pub struct Ore;

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
pub struct Log;

#[block_behavior]
impl BlockBehavior for Log
{
    fn onPlacedBy(&self, ctx: &mut BlockCtx, entity: Entity) { placedByPlayer(ctx, entity) }

    fn onHarvest(&self, ctx: &mut BlockCtx, player: Player)
    {
        let world: World = ctx.world();
        let strong: bool = STRONG_AXES.iter().any(|axe| holds(player, axe));
        if strong && ctx.block() == crate::ids::blockId(block::GOLDEN_OAK_LOG)
        {
            let amber: i32 = AMBER_MIN + world.random(AMBER_SPREAD);
            let pos: BlockPos = ctx.pos();
            let _ = world.spawnItem(
                [f64::from(pos.x), f64::from(pos.y), f64::from(pos.z)],
                ItemStack::of(item::GOLDEN_AMBER, amber),
            );
        }
        let twice: bool = ctx.meta() == NATURAL && holds(player, &skyroot("axe"));
        drop(ctx, if twice { TWICE } else { ONCE });
    }
}
