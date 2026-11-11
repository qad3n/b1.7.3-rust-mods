use super::{Conductor, EnergyTile, connects, net};
use crate::core::{blockTexture, dir, own, sound, stacks};
use crate::items::COLORS;
use b173_api::prelude::*;

// TileEntityCable by cable type: the block, its thickness in sixteenths, its numbers and whether paint holds on it
struct Cable
{
    m_name: &'static str,
    m_thickness: f32,
    m_conductor: Conductor,
    m_paintable: bool,
    // the cable the insulation cutter leaves behind and the one rubber turns it into
    m_stripped: Option<&'static str>,
    m_insulated: Option<&'static str>,
}

const fn cable(
    name: &'static str,
    thickness: f32,
    numbers: (f64, i32, i32),
    paintable: bool,
    stripped: Option<&'static str>,
    insulated: Option<&'static str>,
) -> Cable
{
    let (loss, absorption, breakdown) = numbers;
    Cable
    {
        m_name: name,
        m_thickness: thickness,
        m_conductor: Conductor { m_loss: loss, m_absorption: absorption, m_breakdown: breakdown },
        m_paintable: paintable,
        m_stripped: stripped,
        m_insulated: insulated,
    }
}

// TileEntityCable.getInsulationBreakdownEnergy is 9001 for all, and removeInsulation does nothing
static CABLES: [Cable; 11] = [
    cable("copper_cable", 6.0, (0.2, 32, 33), true, Some("uninsulated_copper_cable"), None),
    cable("uninsulated_copper_cable", 4.0, (0.3, 8, 33), false, None, Some("copper_cable")),
    cable("gold_cable", 3.0, (0.5, 8, 129), false, None, Some("insulated_gold_cable")),
    cable("insulated_gold_cable", 5.0, (0.45, 32, 129), true, Some("gold_cable"), Some("double_insulated_gold_cable")),
    cable("double_insulated_gold_cable", 6.0, (0.4, 128, 129), true, Some("insulated_gold_cable"), None),
    cable("hv_cable", 6.0, (1.0, 0, 2049), false, None, Some("insulated_hv_cable")),
    cable("insulated_hv_cable", 8.0, (0.95, 128, 2049), true, Some("hv_cable"), Some("double_insulated_hv_cable")),
    cable(
        "double_insulated_hv_cable",
        10.0,
        (0.9, 512, 2049),
        true,
        Some("insulated_hv_cable"),
        Some("quadruple_insulated_hv_cable"),
    ),
    cable("quadruple_insulated_hv_cable", 12.0, (0.8, 9001, 2049), true, Some("double_insulated_hv_cable"), None),
    cable("glass_fibre_cable", 4.0, (0.025, 9001, 513), true, None, None),
    cable("ultra_low_current_cable", 5.0, (0.001, 0, 4), false, None, None),
];

const HARDNESS: f32 = 0.4;
const SIXTEENTHS: f32 = 16.0;
// TileEntityCable.color as a block state, the painter's colors with black standing for unpainted
const COLOR: &str = "color";
const UNPAINTED: &str = "none";

fn cableAt(world: World, pos: BlockPos) -> Option<&'static Cable>
{
    let block: i32 = world.block(pos);
    CABLES.iter().find(|row| blockIs(block, row.m_name))
}

// the color state values in painter order, so a painter's color index is its value's index
fn colorName(color: usize) -> &'static str
{
    match color
    {
        0 => UNPAINTED,
        color => COLORS[color.min(COLORS.len() - 1)],
    }
}

fn painted(state: &StateData) -> Option<&str> { state.get(COLOR).filter(|color| *color != UNPAINTED) }

// TileEntityCable.canInteractWithCable: unpainted cables join anything, painted ones only their own color
fn sameColor(world: World, a: BlockPos, b: BlockPos) -> bool
{
    let (Some(first), Some(second)) = (world.stateAt(a), world.stateAt(b))
    else
    {
        return true;
    };

    match (painted(&first), painted(&second))
    {
        (Some(first), Some(second)) => first == second,
        _ => true,
    }
}

impl EnergyTile for Cable
{
    fn accepts(&self, world: World, pos: BlockPos, face: i32) -> bool
    {
        let other: BlockPos = dir::step(pos, face);
        cableAt(world, other).is_none() || sameColor(world, pos, other)
    }

    fn emits(&self, world: World, pos: BlockPos, face: i32) -> bool { self.accepts(world, pos, face) }

    fn conductor(&self, world: World, pos: BlockPos) -> Option<&'static Conductor>
    {
        cableAt(world, pos).map(|row| &row.m_conductor)
    }

    fn joins(&self, world: World, pos: BlockPos, from: BlockPos) -> bool
    {
        cableAt(world, from).is_none() || sameColor(world, pos, from)
    }
}

// TileEntityCable.changeColor through the painter, true when the color took
pub(crate) fn paint(world: World, pos: BlockPos, color: usize) -> bool
{
    let (Some(cable), Some(state)) = (cableAt(world, pos), world.stateAt(pos))
    else
    {
        return false;
    };

    let name: &str = colorName(color);
    if !cable.m_paintable || state.get(COLOR) == Some(name)
    {
        return false;
    }

    let _ = world.setBlockState(pos, (*state).clone().with(COLOR, name));
    net::changed(world);
    true
}

// swaps a cable for another of its line keeping the paint, as the insulation cutter sets cableType
fn swap(world: World, pos: BlockPos, name: &str) -> bool
{
    let (Some(state), Ok(block)) = (world.stateAt(pos), Identifier::parse(&own(name)))
    else
    {
        return false;
    };

    let swapped: bool = world.setBlockState(pos, StateData { block, ..(*state).clone() }).unwrap_or(false);
    net::changed(world);
    swapped
}

// ItemToolCutter.cutInsulationFrom, a left click strips one layer and drops its rubber
pub(crate) fn strip(world: World, pos: BlockPos) -> bool
{
    let Some(stripped) = cableAt(world, pos).and_then(|cable| cable.m_stripped)
    else
    {
        return false;
    };

    if !swap(world, pos, stripped)
    {
        return false;
    }

    stacks::drop(world, pos, &ItemStack::of(own("rubber").as_str(), 1));
    world.playSound(dir::center(pos), &sound("insulationCutter"), 1.0, 1.0);
    true
}

// ItemToolCutter.onItemUse, a right click wraps one layer of rubber around the cable
pub(crate) fn insulate(world: World, pos: BlockPos) -> bool
{
    match cableAt(world, pos).and_then(|cable| cable.m_insulated)
    {
        Some(insulated) => swap(world, pos, insulated),
        None => false,
    }
}

#[cfg(test)]
pub(crate) fn conductorOf(name: &str) -> &'static Conductor
{
    &CABLES.iter().find(|row| row.m_name == name).expect("a cable of TileEntityCable").m_conductor
}

pub(crate) fn canInsulate(world: World, pos: BlockPos) -> bool
{
    cableAt(world, pos).is_some_and(|cable| cable.m_insulated.is_some())
}

pub(crate) fn isCable(world: World, pos: BlockPos) -> bool { cableAt(world, pos).is_some() }

struct CableBlock;

#[block_behavior]
impl BlockBehavior for CableBlock
{
    fn connections(&self, ctx: &mut BlockCtx) -> u8
    {
        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();
        dir::all()
            .filter(|face| connects(world, pos, dir::step(pos, *face), *face))
            .fold(0, |mask, face| mask | 1 << face)
    }

    fn onAdded(&self, ctx: &mut BlockCtx) { net::changed(ctx.world()); }

    fn onRemoved(&self, ctx: &mut BlockCtx) { net::changed(ctx.world()); }

    // BlockCable.onBlockClicked with the insulation cutter
    fn onClicked(&self, ctx: &mut BlockCtx, player: Player)
    {
        let world: World = ctx.world();
        let Some(held) = player.held()
        else
        {
            return;
        };

        if !held.item.is("ic2:insulation_cutter") || !ctx.isAuthority() || !strip(world, ctx.pos())
        {
            return;
        }

        const CUT_WEAR: i32 = 3;
        if let Err(error) = player.setHeld(crate::items::wear(held, CUT_WEAR))
        {
            warn(&format!("could not wear the insulation cutter: {error}"));
        }
    }
}

// BlockCable.getBlockTexture, a column of the cable sheet per paint color
fn colorStates(name: &str) -> BlockStatesSpec
{
    let colors: Vec<&str> = (0..COLORS.len()).map(colorName).collect();
    let variants: Vec<StateVariantSpec> = colors
        .iter()
        .skip(1)
        .map(|color| StateVariantSpec
        {
            when: [(COLOR.to_owned(), (*color).to_owned())].into(),
            texture: Some(blockTexture(&format!("{name}_{color}"))),
            textures: None,
        })
        .collect();
    BlockStatesSpec { properties: vec![PropertySpec::new(COLOR, &colors)], variants, ..BlockStatesSpec::default() }
}

// RenderBlockCable: the core box and an arm toward each joined face
fn model(thickness: f32) -> BlockModelSpec
{
    let low: f32 = (SIXTEENTHS - thickness) / 2.0;
    let high: f32 = low + thickness;
    let arm = |face: i32| -> Cuboid
    {
        let [dx, dy, dz] = dir::offset(face);
        let span = |offset: i32| -> (f32, f32)
        {
            match offset
            {
                -1 => (0.0, low),
                1 => (high, SIXTEENTHS),
                _ => (low, high),
            }
        };
        let (x0, x1) = span(dx);
        let (y0, y1) = span(dy);
        let (z0, z1) = span(dz);
        Cuboid { from: [x0, y0, z0], to: [x1, y1, z1], texture: None, metas: Vec::new(), connect: Some(face) }
    };
    let mut cuboids: Vec<Cuboid> = vec![Cuboid { from: [low; 3], to: [high; 3], ..Cuboid::default() }];
    cuboids.extend(dir::all().map(arm));
    BlockModelSpec { cuboids, ..BlockModelSpec::default() }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for row in &CABLES
    {
        ctx.blocks().register(
            row.m_name,
            BlockSpecBuilder::template("minecraft:block")
                .material("sand")
                .sound("cloth")
                .hardness(HARDNESS)
                .texture(&blockTexture(row.m_name))
                .tool(ToolClass::Axe, 0)
                .opacity(0)
                .model(model(row.m_thickness))
                .drops(vec![DropSpec
                {
                    item: Identifier::parse(&own(row.m_name))?,
                    count: vec![1],
                    meta: 0,
                    chance: 1.0,
                }])
                .tag("ic2:cables")
                .behavior(CableBlock),
        )?;
        ctx.blocks().states(row.m_name, colorStates(row.m_name))?;
        super::add(own(row.m_name), row);
    }

    Ok(())
}
