use super::{active, facing};
use crate::core::{dir, random};
use b173_api::prelude::*;

pub(crate) const WIDTH: i32 = 176;
pub(crate) const HEIGHT: i32 = 166;
pub(crate) const INVENTORY: [i32; 2] = [8, 84];
const TEXT: u32 = 4210752;
const TITLE_Y: i32 = 6;
// drawGuiContainerForegroundLayer puts the inventory label at ySize - 96 + 2
const INVENTORY_LABEL: i32 = HEIGHT - 96 + 2;

pub(crate) fn texture(name: &str) -> String { format!("ic2:gui/{name}") }

pub(crate) fn container(background: &str) -> ScreenBuilder
{
    ScreenBuilder::new("", WIDTH, HEIGHT).background(&texture(background)).playerInventory(INVENTORY[0], INVENTORY[1])
}

pub(crate) fn labels(draw: &mut Draw, title: &str, x: i32)
{
    draw.text(x, TITLE_Y, title, TEXT);
    draw.text(INVENTORY[0], INVENTORY_LABEL, &translate("ic2.gui.inventory"), TEXT);
}

pub(crate) fn text(draw: &mut Draw, x: i32, y: i32, line: &str) { draw.text(x, y, line, TEXT); }

pub(crate) fn name(block: &str) -> String { translate(&format!("tile.ic2.{block}.name")) }

// a gauge on the background sheet: where it sits, where its full picture is, and its full size
pub(crate) struct Bar
{
    pub(crate) m_at: [i32; 2],
    pub(crate) m_uv: [i32; 2],
    pub(crate) m_size: [i32; 2],
}

// a gauge that fills from the bottom, as the fuel flames, level counted in pixels
pub(crate) fn rising(draw: &mut Draw, sheet: &str, bar: &Bar, level: i32, extra: i32)
{
    let [x, y] = bar.m_at;
    let [u, v] = bar.m_uv;
    let [width, height] = bar.m_size;
    let level: i32 = level.clamp(0, height);
    draw.texture(&texture(sheet), x, y + height - level, u, v + height - level, width, level + extra);
}

// a gauge that fills to the right, as the progress arrows
pub(crate) fn across(draw: &mut Draw, sheet: &str, bar: &Bar, level: i32)
{
    let [x, y] = bar.m_at;
    let [u, v] = bar.m_uv;
    let [_, height] = bar.m_size;
    draw.texture(&texture(sheet), x, y, u, v, level.max(0), height);
}

// the tile data of the block a screen was opened on, as the client last got it
pub(crate) fn state<T: serde::de::DeserializeOwned + Default>(pos: Option<BlockPos>) -> Option<T>
{
    let pos: BlockPos = pos?;
    let local: EntityInfo = localPlayer()?.info()?;
    Some(World { dimension: local.world }.tileData(pos))
}

// BlockMachine and BlockGenerator.randomDisplayTick: an active iron furnace or generator smokes and flames at its front
pub(crate) fn flames(name: &str, ctx: &mut BlockCtx)
{
    const HEIGHT: f64 = 6.0 / 16.0;
    const REACH: f64 = 0.52;
    const SPREAD: f64 = 0.6;
    const SIDE: f64 = 0.3;

    let Some(state) = ctx.stateAt()
    else
    {
        return;
    };

    if !matches!(name, "iron_furnace" | "generator") || !active(&state)
    {
        return;
    }

    let world: World = ctx.world();
    let [x, _, z] = dir::center(ctx.pos());
    let y: f64 = ctx.pos().y as f64 + random::float(world) * HEIGHT;
    let side: f64 = random::float(world) * SPREAD - SIDE;
    let spot: [f64; 3] = match facing(&state)
    {
        dir::WEST => [x - REACH, y, z + side],
        dir::EAST => [x + REACH, y, z + side],
        dir::NORTH => [x + side, y, z - REACH],
        dir::SOUTH => [x + side, y, z + REACH],
        _ => return,
    };

    world.particle("smoke", spot, [0.0; 3]);
    world.particle("flame", spot, [0.0; 3]);
}
