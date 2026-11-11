use super::{EnergyTile, net};
use crate::core::{own, stacks};
use crate::electric;
use crate::machines::screen::{self, Bar};
use crate::machines::{Faces, Look, Machine, facingAt, registerBlock};
use b173_api::prelude::*;
use std::rc::Rc;

mod slot
{
    pub const CHARGE: i32 = 0;
    pub const DISCHARGE: i32 = 1;
}

// BlockElectric.setHardness
const HARDNESS: f32 = 1.5;
const STORAGE_SLOTS: i32 = 2;

const fn look(bottom: &'static str, top: &'static str, side: &'static str, front: &'static str) -> Look
{
    Look { m_idle: [bottom, top, side, front], m_active: [bottom, top, side, front], m_faces: Faces::Storage }
}

const fn block(name: &'static str, look: Look, drop: &'static str, screen: bool, slots: i32) -> Machine
{
    Machine
    {
        m_name: name,
        m_look: look,
        m_hardness: HARDNESS,
        m_screen: screen,
        m_wrench: 1.0,
        m_drop: drop,
        m_passes: Some("ic2:storage_passes"),
        m_slots: slots,
    }
}

pub(crate) static MACHINES: [Machine; 6] = [
    block("batbox", look("batbox_side", "batbox_side", "batbox_side", "batbox_front"), "batbox", true, STORAGE_SLOTS),
    block("mfe", look("machine_bottom", "machine_top", "machine_side", "mfe_front"), "machine", true, STORAGE_SLOTS),
    block("mfsu", look("mfsu_side", "mfsu_side", "mfsu_side", "mfsu_front"), "machine", true, STORAGE_SLOTS),
    block(
        "lv_transformer",
        look("batbox_front", "batbox_front", "batbox_front", "lv_transformer_front"),
        "lv_transformer",
        false,
        0,
    ),
    block("mv_transformer", look("mfe_front", "mfe_front", "mfe_front", "mv_transformer_front"), "machine", false, 0),
    block(
        "hv_transformer",
        look("mfsu_front", "mfsu_front", "mfsu_front", "hv_transformer_front"),
        "machine",
        false,
        0,
    ),
];

// TileEntityElectricBlock's tier, packet size and capacity
struct Store
{
    m_name: &'static str,
    m_tier: i32,
    m_output: i32,
    m_capacity: i32,
    m_titleX: i32,
}

static STORES: [Store; 3] = [
    Store { m_name: "batbox", m_tier: 1, m_output: 32, m_capacity: 40000, m_titleX: 62 },
    Store { m_name: "mfe", m_tier: 2, m_output: 128, m_capacity: 600000, m_titleX: 62 },
    Store { m_name: "mfsu", m_tier: 3, m_output: 512, m_capacity: 10000000, m_titleX: 62 },
];

// TileEntityTransformer: the low side, the high side and its buffer
struct Transformer
{
    m_name: &'static str,
    m_low: i32,
    m_high: i32,
    m_capacity: i32,
}

static TRANSFORMERS: [Transformer; 3] = [
    Transformer { m_name: "lv_transformer", m_low: 32, m_high: 128, m_capacity: 256 },
    Transformer { m_name: "mv_transformer", m_low: 128, m_high: 512, m_capacity: 1024 },
    Transformer { m_name: "hv_transformer", m_low: 512, m_high: 2048, m_capacity: 4096 },
];

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) struct Charge
{
    pub(crate) m_energy: i32,
    m_redstone: bool,
}

// TileEntityElectricBlock.updateEntity: charge the top item, drain the bottom one, push a packet out of the front
fn storeTick(store: &Store, world: World, pos: BlockPos, charge: &mut Charge)
{
    const REDSTONE: i32 = 500;
    const SINGLE_USE: i32 = 1000;

    if charge.m_energy > 0
        && let Some(mut item) = world.tileSlot(pos, slot::CHARGE)
        && electric::chargeable(&item)
    {
        let given: i32 = electric::give(&mut item, charge.m_energy, store.m_tier);
        charge.m_energy -= given;
        stacks::set(world, pos, slot::CHARGE, Some(item));
    }

    if charge.m_energy < store.m_capacity
        && let Some(mut item) = world.tileSlot(pos, slot::DISCHARGE)
    {
        if electric::isBattery(&item)
        {
            charge.m_energy += electric::take(&mut item, store.m_tier);
            stacks::set(world, pos, slot::DISCHARGE, Some(item));
        }
        else
        {
            let gain: i32 = match item.item.name().map(|id| id.to_string()).as_deref()
            {
                Some("minecraft:redstone") => REDSTONE,
                Some("ic2:single_use_battery") => SINGLE_USE,
                _ => 0,
            };

            if gain > 0 && gain <= store.m_capacity - charge.m_energy
            {
                charge.m_energy += gain;
                stacks::set(world, pos, slot::DISCHARGE, stacks::less(item, 1));
            }
        }
    }

    let sending: bool =
        charge.m_energy >= store.m_output && !world.isPowered(pos) || charge.m_energy >= store.m_capacity;
    if sending
    {
        charge.m_energy -= store.m_output;
        charge.m_energy += net::emit(world, pos, store.m_output);
    }
}

// TileEntityTransformer.updateEntity: four low packets a tick, or one high packet while powered
fn transformerTick(transformer: &Transformer, world: World, pos: BlockPos, charge: &mut Charge)
{
    const LOW_PACKETS: i32 = 4;

    let powered: bool = world.isPowered(pos);
    if powered != charge.m_redstone
    {
        charge.m_redstone = powered;
        net::changed(world);
    }

    if charge.m_redstone
    {
        if charge.m_energy >= transformer.m_high
        {
            charge.m_energy -= transformer.m_high;
            charge.m_energy += net::emit(world, pos, transformer.m_high);
        }

        return;
    }

    for _ in 0..LOW_PACKETS
    {
        if charge.m_energy < transformer.m_low
        {
            break;
        }

        charge.m_energy -= transformer.m_low;
        charge.m_energy += net::emit(world, pos, transformer.m_low);
    }
}

impl EnergyTile for Store
{
    fn accepts(&self, world: World, pos: BlockPos, face: i32) -> bool { facingAt(world, pos) != face }

    fn emits(&self, world: World, pos: BlockPos, face: i32) -> bool { facingAt(world, pos) == face }

    fn sink(&self) -> bool { true }

    fn demands(&self, world: World, pos: BlockPos) -> bool { world.tileData::<Charge>(pos).m_energy < self.m_capacity }

    fn inject(&self, world: World, pos: BlockPos, _face: i32, amount: i32) -> i32
    {
        if amount > self.m_output
        {
            net::overload(world, pos);
            return 0;
        }

        let mut charge: Charge = world.tileData(pos);
        let taken: i32 = amount.min(self.m_capacity - charge.m_energy).max(0);
        charge.m_energy += taken;
        let _ = world.setTileData(pos, &charge);
        amount - taken
    }
}

impl EnergyTile for Transformer
{
    fn accepts(&self, world: World, pos: BlockPos, face: i32) -> bool
    {
        world.tileData::<Charge>(pos).m_redstone != (facingAt(world, pos) == face)
    }

    fn emits(&self, world: World, pos: BlockPos, face: i32) -> bool
    {
        world.tileData::<Charge>(pos).m_redstone == (facingAt(world, pos) == face)
    }

    fn sink(&self) -> bool { true }

    fn demands(&self, world: World, pos: BlockPos) -> bool { world.tileData::<Charge>(pos).m_energy < self.m_capacity }

    // TileEntityTransformer.injectEnergy: the low side takes low packets, the high side high ones
    fn inject(&self, world: World, pos: BlockPos, _face: i32, amount: i32) -> i32
    {
        let mut charge: Charge = world.tileData(pos);
        let limit: i32 = if charge.m_redstone
        {
            self.m_low
        }
        else
        {
            self.m_high
        };
        if amount > limit
        {
            net::overload(world, pos);
            return 0;
        }

        let taken: i32 = amount.min(self.m_capacity - charge.m_energy).max(0);
        charge.m_energy += taken;
        let _ = world.setTileData(pos, &charge);
        amount - taken
    }
}

struct StoreTile
{
    m_store: &'static Store,
}

#[tile_behavior]
impl TileBehavior for StoreTile
{
    fn onTick(&self, ctx: &mut BlockCtx)
    {
        if !ctx.isAuthority()
        {
            return;
        }

        let before: Charge = ctx.tileData();
        let mut charge: Charge = before;
        storeTick(self.m_store, ctx.world(), ctx.pos(), &mut charge);

        if charge != before
        {
            let _ = ctx.setTileData(&charge);
        }
    }

    fn energyDemand(&self, ctx: &mut BlockCtx, face: i32) -> Option<EnergyDemand>
    {
        let room: bool = self.m_store.demands(ctx.world(), ctx.pos());
        if !room || !self.m_store.accepts(ctx.world(), ctx.pos(), face)
        {
            return None;
        }

        super::demand(self.m_store.m_output)
    }

    fn receiveEnergy(&self, ctx: &mut BlockCtx, _face: i32, amount: i64, simulate: bool) -> i64
    {
        let mut charge: Charge = ctx.tileData();
        let offered: i32 = amount.clamp(0, i64::from(self.m_store.m_output)) as i32;
        let taken: i32 = offered.min(self.m_store.m_capacity - charge.m_energy).max(0);

        if !simulate
        {
            charge.m_energy += taken;
            let _ = ctx.setTileData(&charge);
        }

        i64::from(taken)
    }
}

struct TransformerTile
{
    m_transformer: &'static Transformer,
}

#[tile_behavior]
impl TileBehavior for TransformerTile
{
    fn onTick(&self, ctx: &mut BlockCtx)
    {
        if !ctx.isAuthority()
        {
            return;
        }

        let before: Charge = ctx.tileData();
        let mut charge: Charge = before;
        transformerTick(self.m_transformer, ctx.world(), ctx.pos(), &mut charge);

        if charge != before
        {
            let _ = ctx.setTileData(&charge);
        }
    }
}

// BlockElectric.alterFacing through the wrench: false when it already faces there
pub(crate) fn turn(world: World, pos: BlockPos, face: i32) -> bool
{
    if facingAt(world, pos) == face
    {
        return false;
    }

    crate::machines::setFacing(world, pos, face);
    net::changed(world);
    true
}

pub(crate) fn isStorage(world: World, pos: BlockPos) -> bool
{
    let block: i32 = world.block(pos);
    MACHINES.iter().any(|row| blockIs(block, row.m_name))
}

fn storeAt(world: World, pos: BlockPos) -> Option<&'static Store>
{
    let block: i32 = world.block(pos);
    STORES.iter().find(|row| blockIs(block, row.m_name))
}

// GuiElectricBlock: the charge arrow and the numbers
struct StoreScreen
{
    m_store: &'static Store,
    m_pos: Option<BlockPos>,
}

mod layout
{
    use super::Bar;

    pub const ARROW: Bar = Bar { m_at: [79, 34], m_uv: [176, 14], m_size: [24, 16] };
    pub const ARROW_EXTRA: i32 = 1;
    pub const LEVEL: [i32; 2] = [79, 25];
    pub const STORED: [i32; 2] = [110, 35];
    pub const CAPACITY: [i32; 2] = [110, 45];
    pub const OUTPUT: [i32; 2] = [85, 60];
}

impl Screen for StoreScreen
{
    fn init(&mut self, _width: i32, _height: i32, pos: Option<BlockPos>, _data: &[u8]) { self.m_pos = pos; }

    fn draw(&mut self, draw: &mut Draw, _mouse: [i32; 2], _partial: f32)
    {
        let store: &Store = self.m_store;
        screen::labels(draw, &screen::name(store.m_name), store.m_titleX);

        let charge: Charge = screen::state(self.m_pos).unwrap_or_default();
        screen::text(draw, layout::LEVEL[0], layout::LEVEL[1], &translate("ic2.gui.power_level"));
        screen::text(draw, layout::STORED[0], layout::STORED[1], &format!(" {}", charge.m_energy));
        screen::text(draw, layout::CAPACITY[0], layout::CAPACITY[1], &format!("/{}", store.m_capacity));
        screen::text(draw, layout::OUTPUT[0], layout::OUTPUT[1], &format!("Out: {} EU/t", store.m_output));

        if charge.m_energy > 0
        {
            let level: i32 =
                (i64::from(charge.m_energy) * i64::from(layout::ARROW.m_size[0]) / i64::from(store.m_capacity)) as i32;
            screen::across(draw, "electricblock", &layout::ARROW, level + layout::ARROW_EXTRA);
        }
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    // ContainerElectricBlock slot positions
    const CHARGE_AT: [i32; 2] = [56, 17];
    const DISCHARGE_AT: [i32; 2] = [56, 53];

    for store in &STORES
    {
        let Some(machine) = MACHINES.iter().find(|row| row.m_name == store.m_name)
        else
        {
            continue;
        };

        registerBlock(ctx, machine, Rc::new(StoreTile { m_store: store }), true)?;
        super::add(own(store.m_name), store);
        let builder: ScreenBuilder = screen::container("electricblock")
            .slot(slot::CHARGE, CHARGE_AT[0], CHARGE_AT[1])
            .slot(slot::DISCHARGE, DISCHARGE_AT[0], DISCHARGE_AT[1]);
        ctx.screens().register(store.m_name, builder, StoreScreen { m_store: store, m_pos: None })?;
    }

    for transformer in &TRANSFORMERS
    {
        let Some(machine) = MACHINES.iter().find(|row| row.m_name == transformer.m_name)
        else
        {
            continue;
        };

        registerBlock(ctx, machine, Rc::new(TransformerTile { m_transformer: transformer }), true)?;
        super::add(own(transformer.m_name), transformer);
    }

    Ok(())
}

// TileEntityElectrolyzer.lookForMFE: an MFE or MFSU next to it, whose charge it reads and changes directly
pub(crate) struct Bank
{
    pub(crate) m_pos: BlockPos,
    pub(crate) m_capacity: i32,
}

pub(crate) fn bankAt(world: World, pos: BlockPos) -> Option<Bank>
{
    let store: &Store = storeAt(world, pos)?;
    (store.m_tier > 1).then_some(Bank { m_pos: pos, m_capacity: store.m_capacity })
}

// TileEntityTeleporter.isElectricBlock takes any storage block
pub(crate) fn anyBankAt(world: World, pos: BlockPos) -> Option<Bank>
{
    let store: &Store = storeAt(world, pos)?;
    Some(Bank { m_pos: pos, m_capacity: store.m_capacity })
}

pub(crate) fn bankEnergy(world: World, bank: &Bank) -> i32 { world.tileData::<Charge>(bank.m_pos).m_energy }

pub(crate) fn setBankEnergy(world: World, bank: &Bank, energy: i32)
{
    let mut charge: Charge = world.tileData(bank.m_pos);
    charge.m_energy = energy;
    let _ = world.setTileData(bank.m_pos, &charge);
}
