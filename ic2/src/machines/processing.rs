use super::screen::{self, Bar};
use super::{Faces, Look, Machine, registerBlock, setActive};
use crate::core::{dir, own, sound, stacks};
use crate::electric;
use crate::energy::{self, EnergyTile, net};
use b173_api::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// the slots of a processing machine: input, fuel or battery, output, and the canner's can
mod slot
{
    pub const INPUT: i32 = 0;
    pub const FUEL: i32 = 1;
    pub const OUTPUT: i32 = 2;
    pub const CAN: i32 = 3;
}

const fn look(idle: [&'static str; 4], active: [&'static str; 4]) -> Look
{
    Look { m_idle: idle, m_active: active, m_faces: Faces::Machine }
}

// BlockMachine.setHardness
const HARDNESS: f32 = 2.0;
const SLOTS: i32 = 3;
const ANY_DAMAGE: i32 = -1;
const CANNER_SLOTS: i32 = 4;

const fn machine(name: &'static str, look: Look, wrench: f32, drop: &'static str, slots: i32) -> Machine
{
    Machine
    {
        m_name: name,
        m_look: look,
        m_hardness: HARDNESS,
        m_screen: true,
        m_wrench: wrench,
        m_drop: drop,
        m_passes: &[],
        m_slots: slots,
    }
}

pub(crate) static MACHINES: [Machine; 8] = [
    machine(
        "iron_furnace",
        look(
            ["iron_furnace_bottom", "iron_furnace_top", "iron_furnace_side", "iron_furnace_front"],
            ["iron_furnace_bottom", "iron_furnace_top", "iron_furnace_side", "iron_furnace_front_active"],
        ),
        1.0,
        "iron_furnace",
        SLOTS,
    ),
    machine(
        "electric_furnace",
        look(
            ["iron_furnace_bottom", "iron_furnace_top", "iron_furnace_side", "electric_furnace_front"],
            ["iron_furnace_bottom", "iron_furnace_top", "iron_furnace_side", "electric_furnace_front_active"],
        ),
        1.0,
        "electric_furnace",
        SLOTS,
    ),
    machine(
        "macerator",
        look(
            ["machine_bottom", "macerator_top", "machine_side", "macerator_front"],
            ["machine_bottom", "macerator_top_active", "machine_side", "macerator_front"],
        ),
        0.85,
        "machine",
        SLOTS,
    ),
    machine(
        "extractor",
        look(
            ["machine_bottom", "extractor_top", "extractor_side", "extractor_front"],
            ["machine_bottom", "extractor_top", "extractor_side_active", "extractor_front_active"],
        ),
        0.85,
        "machine",
        SLOTS,
    ),
    machine(
        "compressor",
        look(
            ["machine_bottom", "extractor_top", "machine_side", "compressor_front"],
            ["machine_bottom", "extractor_top", "machine_side", "compressor_front_active"],
        ),
        0.85,
        "machine",
        SLOTS,
    ),
    machine(
        "canning_machine",
        look(
            ["machine_bottom", "machine_top", "machine_side", "canner_front"],
            ["machine_bottom", "machine_top", "machine_side", "canner_front_active"],
        ),
        0.85,
        "machine",
        CANNER_SLOTS,
    ),
    machine(
        "recycler",
        look(
            ["machine_bottom", "recycler_top", "machine_side", "compressor_front"],
            ["machine_bottom", "recycler_top", "machine_side", "compressor_front_active"],
        ),
        0.85,
        "machine",
        SLOTS,
    ),
    machine("machine", look(["machine_top"; 4], ["machine_top"; 4]), 0.0, "machine", 0),
];

// TileEntityElectricMachine's numbers: EU a tick, ticks per operation and the packet rating
#[derive(Clone, Copy)]
struct Rates
{
    m_consume: i32,
    m_length: i32,
    m_input: i32,
}

impl Rates
{
    // TileEntityElectricMachine's constructor: room for a whole operation plus one packet
    fn capacity(self) -> i32 { self.m_consume * self.m_length + self.m_input - 1 }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind
{
    IronFurnace,
    Furnace,
    Macerator,
    Extractor,
    Compressor,
    Recycler,
    Canner,
}

// the processing machine a block is, its rates and sounds
struct Process
{
    m_name: &'static str,
    m_kind: Kind,
    m_rates: Rates,
    m_loop: &'static str,
    m_loopTicks: i32,
}

const fn process(name: &'static str, kind: Kind, rates: (i32, i32, i32), sound: &'static str, every: i32) -> Process
{
    let (consume, length, input) = rates;
    Process
    {
        m_name: name,
        m_kind: kind,
        m_rates: Rates { m_consume: consume, m_length: length, m_input: input },
        m_loop: sound,
        m_loopTicks: every,
    }
}

static PROCESSES: [Process; 7] = [
    process("iron_furnace", Kind::IronFurnace, (0, 160, 0), "", 1),
    process("electric_furnace", Kind::Furnace, (3, 100, 32), "machineElecFurnaceLoop", 64),
    process("macerator", Kind::Macerator, (2, 300, 32), "machineMaceratorLoop", 60),
    process("extractor", Kind::Extractor, (2, 300, 32), "machineExtractorLoop", 32),
    process("compressor", Kind::Compressor, (2, 300, 32), "machineCompressorLoop", 64),
    process("recycler", Kind::Recycler, (1, 35, 32), "machineRecyclerLoop", 32),
    process("canning_machine", Kind::Canner, (1, 600, 32), "machineCannerLoop", 64),
];

// TileEntityMacerator, TileEntityExtractor and TileEntityCompressor.initRecipes, keyed by the input item alone
const MACERATOR: [(&str, &str, i32); 17] = [
    ("minecraft:iron_ore", "ic2:iron_dust", 2),
    ("minecraft:gold_ore", "ic2:gold_dust", 2),
    ("ic2:tin_ore", "ic2:tin_dust", 2),
    ("ic2:copper_ore", "ic2:copper_dust", 2),
    ("minecraft:coal", "ic2:coal_dust", 1),
    ("ic2:refined_iron", "ic2:iron_dust", 1),
    ("ic2:tin_ingot", "ic2:tin_dust", 1),
    ("ic2:copper_ingot", "ic2:copper_dust", 1),
    ("minecraft:iron_ingot", "ic2:iron_dust", 1),
    ("minecraft:gold_ingot", "ic2:gold_dust", 1),
    ("minecraft:wool", "minecraft:string", 1),
    ("minecraft:gravel", "minecraft:flint", 1),
    ("minecraft:stone", "minecraft:cobblestone", 1),
    ("minecraft:cobblestone", "minecraft:sand", 1),
    ("minecraft:sandstone", "minecraft:sand", 1),
    ("minecraft:ice", "minecraft:snowball", 1),
    ("ic2:bronze_ingot", "ic2:bronze_dust", 1),
];

const EXTRACTOR: [(&str, &str, i32); 6] = [
    ("ic2:rubber_wood", "ic2:rubber", 1),
    ("ic2:rubber_sapling", "ic2:rubber", 1),
    ("ic2:sticky_resin", "ic2:rubber", 3),
    ("ic2:bio_cell", "ic2:biofuel_cell", 1),
    ("ic2:coal_cell", "ic2:coalfuel_cell", 1),
    ("ic2:water_cell", "ic2:coolant_cell", 1),
];

const COMPRESSOR: [(&str, &str, i32); 10] = [
    ("ic2:plantball", "ic2:compressed_plants", 1),
    ("ic2:hydrated_coal_dust", "ic2:hydrated_coal", 1),
    ("minecraft:sand", "minecraft:sandstone", 1),
    ("minecraft:snowball", "minecraft:ice", 1),
    ("ic2:water_cell", "minecraft:snowball", 1),
    ("ic2:uranium_chunk", "ic2:refined_uranium", 1),
    ("ic2:mixed_metal_ingot", "ic2:advanced_alloy", 1),
    ("ic2:raw_carbon_mesh", "ic2:carbon_plate", 1),
    ("ic2:coal_ball", "ic2:compressed_coal_ball", 1),
    ("ic2:coal_chunk", "ic2:industrial_diamond", 1),
];

thread_local!
{
    // FurnaceRecipes.getSmeltingResult by input item name, rebuilt when the content changes
    static SMELTING: RefCell<(i32, HashMap<Identifier, StackData>)> = RefCell::new((-1, HashMap::new()));
}

pub(crate) fn smelted(input: &ItemStack) -> Option<ItemStack>
{
    let name: Identifier = input.item.name()?;
    SMELTING.with(|cache|
    {
        let mut cache = cache.borrow_mut();
        let generation: i32 = contentGeneration();
        if cache.0 != generation
        {
            let table: HashMap<Identifier, StackData> = recipeList()
                .into_iter()
                .filter_map(|recipe| match recipe.shape
                {
                    RecipeShape::Smelting { input } => Some((nameOf(RegistryKind::Item, input.id)?, recipe.output)),
                    _ => None,
                })
                .collect();
            *cache = (generation, table);
        }

        cache.1.get(&name).cloned().map(ItemStack::fromRaw)
    })
}

fn tableResult(table: &[(&str, &str, i32)], input: &ItemStack) -> Option<ItemStack>
{
    table.iter().find(|(from, _, _)| input.item.is(from)).map(|(_, to, count)| ItemStack::of(*to, *count))
}

// TileEntityElectricMachine.getResultFor per machine
fn result(kind: Kind, input: &ItemStack) -> Option<ItemStack>
{
    match kind
    {
        Kind::IronFurnace | Kind::Furnace => smelted(input),
        Kind::Macerator => tableResult(&MACERATOR, input),
        Kind::Extractor => tableResult(&EXTRACTOR, input),
        Kind::Compressor => tableResult(&COMPRESSOR, input),
        Kind::Recycler | Kind::Canner => None,
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) struct Work
{
    pub(crate) m_energy: i32,
    pub(crate) m_progress: i32,
    // the iron furnace's burning fuel and the canner's fuel quality
    pub(crate) m_fuel: i32,
    pub(crate) m_maxFuel: i32,
    m_soundTicks: i32,
}

struct Slots
{
    m_world: World,
    m_pos: BlockPos,
}

impl Slots
{
    fn get(&self, slot: i32) -> Option<ItemStack> { self.m_world.tileSlot(self.m_pos, slot) }

    fn set(&self, slot: i32, stack: Option<ItemStack>) { stacks::set(self.m_world, self.m_pos, slot, stack); }
}

// TileEntityElecMachine.provideEnergy: a battery, redstone or a single-use battery in the fuel slot
pub(crate) fn provide(slots: &(World, BlockPos), slot: i32, energy: &mut i32, capacity: i32) -> bool
{
    const SINGLE_USE: i32 = 1000;

    let (world, pos) = *slots;
    let Some(mut fuel) = world.tileSlot(pos, slot)
    else
    {
        return false;
    };

    if electric::isBattery(&fuel)
    {
        // ItemBattery.getEnergyFrom ignores the request, a machine may hold more than its capacity
        *energy += electric::take(&mut fuel, 1);
        stacks::set(world, pos, slot, Some(fuel));
        return true;
    }

    let gain: i32 = match fuel.item.name().map(|id| id.to_string()).as_deref()
    {
        Some("minecraft:redstone") => capacity,
        Some("ic2:single_use_battery") => SINGLE_USE,
        _ => return false,
    };
    *energy += gain;
    stacks::set(world, pos, slot, stacks::less(fuel, 1));
    true
}

// TileEntityElectricMachine.canOperate for the result machines and the recycler
fn canOperate(process: &Process, slots: &Slots) -> bool
{
    const STICK: &str = "minecraft:stick";
    const SCRAP: &str = "ic2:scrap";

    let Some(input) = slots.get(slot::INPUT)
    else
    {
        return false;
    };

    match process.m_kind
    {
        Kind::Recycler =>
        {
            let room: bool = match slots.get(slot::OUTPUT)
            {
                None => true,
                Some(out) => out.item.is(SCRAP) && out.count < stacks::maxStack(&out),
            };
            room && !input.item.is(STICK) && !input.item.is(SCRAP)
        }

        Kind::Canner => canning::possible(slots.m_world, slots.m_pos, &input, process),
        kind => result(kind, &input).is_some_and(|made| stacks::fits(&slots.get(slot::OUTPUT), &made)),
    }
}

// TileEntityElectricMachine.operate and the recycler's
fn operate(process: &Process, slots: &Slots, work: &mut Work)
{
    const RECYCLE_CHANCE: i32 = 8;

    let Some(input) = slots.get(slot::INPUT)
    else
    {
        return;
    };

    match process.m_kind
    {
        Kind::Recycler =>
        {
            slots.set(slot::INPUT, stacks::less(input, 1));
            if slots.m_world.random(RECYCLE_CHANCE) == 0
            {
                slots.set(slot::OUTPUT, Some(stacks::merged(slots.get(slot::OUTPUT), &ItemStack::of("ic2:scrap", 1))));
            }
        }

        Kind::Canner => canning::operate(slots.m_world, slots.m_pos, work),
        kind =>
        {
            let Some(made) = result(kind, &input)
            else
            {
                return;
            };
            slots.set(slot::OUTPUT, Some(stacks::merged(slots.get(slot::OUTPUT), &made)));
            slots.set(slot::INPUT, stacks::consume(input));
        }
    }
}

fn loopSound(process: &Process, world: World, pos: BlockPos, work: &mut Work)
{
    work.m_soundTicks += 1;
    if work.m_soundTicks % process.m_loopTicks == 0
    {
        world.playSound(dir::center(pos), &sound(process.m_loop), 1.0, 1.0);
    }
}

// TileEntityElectricMachine.updateEntity
fn electricTick(process: &Process, world: World, pos: BlockPos, work: &mut Work)
{
    let slots: Slots = Slots { m_world: world, m_pos: pos };
    let rates: Rates = process.m_rates;
    let operating: bool = work.m_energy > 0 && canOperate(process, &slots);

    if operating
    {
        work.m_energy -= rates.m_consume;
    }

    if work.m_energy <= rates.m_consume * rates.m_length && canOperate(process, &slots)
    {
        provide(&(world, pos), slot::FUEL, &mut work.m_energy, rates.capacity());
    }

    if operating && canOperate(process, &slots)
    {
        loopSound(process, world, pos, work);
        if process.m_kind == Kind::Canner
        {
            canning::advance(world, pos, work);
        }
        else
        {
            work.m_progress += 1;
            if work.m_progress >= rates.m_length
            {
                work.m_progress = 0;
                operate(process, &slots, work);
            }
        }
    }

    if !canOperate(process, &slots)
    {
        work.m_progress = 0;
    }

    if process.m_kind == Kind::Canner
    {
        canning::settle(world, pos, work);
    }

    setActive(world, pos, operating);
}

// TileEntityIronFurnace.getFuelValueFor: wood, sticks, coal, lava, saplings and the fuel can, then other fuels
fn ironFuel(stack: &ItemStack) -> i32
{
    const WOOD: i32 = 300;
    const STICK: i32 = 100;
    const COAL: i32 = 1600;
    const LAVA: i32 = 2000;
    const SAPLING: i32 = 100;
    const FUEL_CAN: i32 = 720000;

    let wooden: bool =
        stack.info().is_some_and(|info| info.block) && stack.item.name().is_some_and(|id| woodBlock(&id.to_string()));

    match stack.item.name().map(|id| id.to_string()).as_deref()
    {
        _ if wooden => WOOD,
        Some("minecraft:stick") => STICK,
        Some("minecraft:coal") => COAL,
        Some("minecraft:lava_bucket") => LAVA,
        Some("minecraft:sapling") => SAPLING,
        Some("ic2:fuel_can") => FUEL_CAN,
        _ => fuelTicks(stack),
    }
}

// Block.blockMaterial == Material.wood for the blocks beta makes of wood
const WOODEN: [&str; 16] = [
    "minecraft:planks",
    "minecraft:log",
    "minecraft:bookshelf",
    "minecraft:chest",
    "minecraft:locked_chest",
    "minecraft:crafting_table",
    "minecraft:oak_stairs",
    "minecraft:fence",
    "minecraft:wooden_door",
    "minecraft:standing_sign",
    "minecraft:wall_sign",
    "minecraft:noteblock",
    "minecraft:jukebox",
    "minecraft:wooden_pressure_plate",
    "minecraft:trapdoor",
    "ic2:rubber_wood",
];

fn woodBlock(name: &str) -> bool { WOODEN.contains(&name) }

pub(crate) fn burnValue(stack: &ItemStack) -> i32 { ironFuel(stack) }

// the furnace fuels by name with their burn ticks, as recipe viewers list them
pub(crate) fn fuels() -> Vec<(&'static str, i32)>
{
    const WOOD: i32 = 300;
    const NAMED: [(&str, i32); 4] =
        [("minecraft:stick", 100), ("minecraft:coal", 1600), ("minecraft:sapling", 100), ("ic2:fuel_can", 720000)];

    // wooden blocks only a placed item makes
    const PLACED: [&str; 3] = ["minecraft:standing_sign", "minecraft:wall_sign", "minecraft:locked_chest"];

    WOODEN.iter().filter(|name| !PLACED.contains(name)).map(|name| (*name, WOOD)).chain(NAMED).collect()
}

// TileEntityIronFurnace.updateEntity
fn ironTick(process: &Process, world: World, pos: BlockPos, work: &mut Work)
{
    let slots: Slots = Slots { m_world: world, m_pos: pos };
    let burning: bool = work.m_fuel > 0;

    if work.m_fuel <= 0
        && canOperate(process, &slots)
        && let Some(fuel) = slots.get(slot::FUEL)
    {
        work.m_fuel = ironFuel(&fuel);
        work.m_maxFuel = work.m_fuel;
        if work.m_fuel > 0
        {
            slots.set(slot::FUEL, stacks::consume(fuel));
        }
    }

    if work.m_fuel > 0 && canOperate(process, &slots)
    {
        work.m_progress += 1;
        if work.m_progress >= process.m_rates.m_length
        {
            work.m_progress = 0;
            operate(process, &slots, work);
        }
    }
    else
    {
        work.m_progress = 0;
    }

    if work.m_fuel > 0
    {
        work.m_fuel -= 1;
    }

    if burning != (work.m_fuel > 0)
    {
        setActive(world, pos, work.m_fuel > 0);
    }
}

struct ProcessTile
{
    m_process: &'static Process,
}

#[tile_behavior]
impl TileBehavior for ProcessTile
{
    fn onTick(&self, ctx: &mut BlockCtx)
    {
        if !ctx.isAuthority()
        {
            return;
        }

        let before: Work = ctx.tileData();
        let mut work: Work = before;

        match self.m_process.m_kind
        {
            Kind::IronFurnace => ironTick(self.m_process, ctx.world(), ctx.pos(), &mut work),
            _ => electricTick(self.m_process, ctx.world(), ctx.pos(), &mut work),
        }

        if work != before
        {
            let _ = ctx.setTileData(&work);
        }
    }

    fn energyDemand(&self, ctx: &mut BlockCtx, _face: i32) -> Option<EnergyDemand>
    {
        let rates: Rates = self.m_process.m_rates;
        let work: Work = ctx.tileData();
        if self.m_process.m_kind == Kind::IronFurnace || work.m_energy >= rates.capacity() - rates.m_input
        {
            return None;
        }

        energy::demand(rates.m_input)
    }

    fn receiveEnergy(&self, ctx: &mut BlockCtx, _face: i32, amount: i64, simulate: bool) -> i64
    {
        let rates: Rates = self.m_process.m_rates;
        let mut work: Work = ctx.tileData();
        let offered: i32 = amount.clamp(0, i64::from(rates.m_input)) as i32;
        let taken: i32 = offered.min(rates.capacity() - work.m_energy).max(0);

        if !simulate
        {
            work.m_energy += taken;
            let _ = ctx.setTileData(&work);
        }

        i64::from(taken)
    }
}

// TileEntityElecMachine as an IEnergySink: takes from every side, blows up on a packet above its rating
impl EnergyTile for Process
{
    fn accepts(&self, _world: World, _pos: BlockPos, _face: i32) -> bool { self.m_kind != Kind::IronFurnace }

    fn sink(&self) -> bool { self.m_kind != Kind::IronFurnace }

    fn demands(&self, world: World, pos: BlockPos) -> bool
    {
        let work: Work = world.tileData(pos);
        work.m_energy < self.m_rates.capacity() - self.m_rates.m_input
    }

    fn inject(&self, world: World, pos: BlockPos, _face: i32, amount: i32) -> i32
    {
        if amount > self.m_rates.m_input
        {
            net::overload(world, pos);
            return 0;
        }

        let mut work: Work = world.tileData(pos);
        work.m_energy += amount;
        let left: i32 = (work.m_energy - self.m_rates.capacity()).max(0);
        work.m_energy -= left;
        let _ = world.setTileData(pos, &work);
        left
    }
}

mod canning
{
    use super::{Process, Work, slot};
    use crate::core::stacks;
    use b173_api::prelude::*;

    mod rules
    {
        // TileEntityCanner: food takes 50 ticks a can, fuel is pumped in every 100 ticks for 600
        pub const FOOD_TICKS: i32 = 50;
        pub const FUEL_STEP: i32 = 100;
        pub const FUEL_DONE: i32 = 600;
        pub const COAL_FUEL: i32 = 2880;
        pub const BIO_FUEL: i32 = 1080;
        pub const REDSTONE_SHARE: f64 = 0.2;
        pub const GLOWSTONE_SHARE: f64 = 0.3;
    }

    #[derive(PartialEq, Eq)]
    enum Mode
    {
        Idle,
        Food,
        Fuel,
    }

    fn mode(world: World, pos: BlockPos) -> Mode
    {
        match world.tileSlot(pos, slot::CAN)
        {
            Some(can) if can.item.is("ic2:tin_can") => Mode::Food,
            Some(can) if can.item.is("ic2:fuel_can_empty") || can.item.is("ic2:jetpack") => Mode::Fuel,
            _ => Mode::Idle,
        }
    }

    // TileEntityCanner.getFoodValue: half the food's healing, rounded up, as cans
    fn food(stack: &ItemStack) -> i32
    {
        const HALVES: f64 = 2.0;

        crate::items::foodHeal(stack).map_or(0, |heal| (f64::from(heal) / HALVES).ceil() as i32)
    }

    fn fuel(stack: &ItemStack, quality: i32) -> i32
    {
        match stack.item.name().map(|id| id.to_string()).as_deref()
        {
            Some("ic2:coalfuel_cell") => rules::COAL_FUEL,
            Some("ic2:biofuel_cell") => rules::BIO_FUEL,
            Some("minecraft:redstone") if quality > 0 => (f64::from(quality) * rules::REDSTONE_SHARE) as i32,
            Some("minecraft:glowstone_dust") if quality > 0 => (f64::from(quality) * rules::GLOWSTONE_SHARE) as i32,
            _ => 0,
        }
    }

    pub(super) fn possible(world: World, pos: BlockPos, input: &ItemStack, _process: &Process) -> bool
    {
        let work: Work = world.tileData(pos);

        match mode(world, pos)
        {
            Mode::Idle => false,
            Mode::Fuel => fuel(input, work.m_fuel) != 0 && world.tileSlot(pos, slot::OUTPUT).is_none(),
            Mode::Food =>
            {
                let cans: i32 = food(input);
                let out: Option<ItemStack> = world.tileSlot(pos, slot::OUTPUT);
                let held: i32 = world.tileSlot(pos, slot::CAN).map_or(0, |can| can.count);
                let room: bool = out
                    .as_ref()
                    .is_none_or(|out| out.item.is("ic2:filled_tin_can") && out.count + cans <= stacks::maxStack(out));
                cans != 0 && cans <= held && room
            }
        }
    }

    // the progress step of TileEntityCanner.updateEntity
    pub(super) fn advance(world: World, pos: BlockPos, work: &mut Work)
    {
        work.m_progress += 1;

        match mode(world, pos)
        {
            Mode::Food =>
            {
                let length: i32 = world.tileSlot(pos, slot::INPUT).map_or(0, |input| food(&input)) * rules::FOOD_TICKS;
                if work.m_progress >= length
                {
                    work.m_progress = 0;
                    operate(world, pos, work);
                }
            }

            Mode::Fuel if work.m_progress % rules::FUEL_STEP == 0 =>
            {
                operate(world, pos, work);
                if work.m_progress >= rules::FUEL_DONE
                {
                    work.m_progress = 0;
                }
            }

            _ =>
            {}
        }
    }

    pub(super) fn settle(world: World, pos: BlockPos, work: &mut Work)
    {
        let input: Option<ItemStack> = world.tileSlot(pos, slot::INPUT);
        let current: Mode = mode(world, pos);
        let stalled: bool = current == Mode::Food
            && !input.as_ref().is_some_and(|input| possible(world, pos, input, &super::PROCESSES[0]));
        if current == Mode::Idle || stalled
        {
            work.m_fuel = 0;
            work.m_progress = 0;
        }
    }

    // a page a food with the cans it fills, and the fuel can with what each fuel adds
    pub(super) fn pages(ctx: &mut RegisterCtx, name: &str, consume: i32) -> Result<()>
    {
        const FUELS: [(&str, i32); 2] =
            [("ic2:coalfuel_cell", rules::COAL_FUEL), ("ic2:biofuel_cell", rules::BIO_FUEL)];

        ctx.machines().register(name, None, &[name])?;

        for (food, heal) in crate::items::FOODS
        {
            let cans: i32 = (f64::from(heal) / 2.0).ceil() as i32;
            ctx.machines()
                .recipe(name)
                .input(ItemStack::of(food, 1))
                .input(ItemStack::of("ic2:tin_can", cans))
                .output(ItemStack::of("ic2:filled_tin_can", cans))
                .note(&format!("{consume} EU/t, {} ticks", cans * rules::FOOD_TICKS))
                .build()?;
        }

        for (cell, value) in FUELS
        {
            ctx.machines()
                .recipe(name)
                .input(ItemStack::of("ic2:fuel_can_empty", 1))
                .input(ItemStack::of(cell, 1))
                .output(ItemStack::of("ic2:fuel_can", 1))
                .note(&format!("Each cell adds {value} fuel"))
                .note("Redstone adds 20%, glowstone 30% of the fuel")
                .note(&format!("{consume} EU/t, {} ticks", rules::FUEL_DONE))
                .build()?;
        }

        Ok(())
    }

    // TileEntityCanner.operate
    pub(super) fn operate(world: World, pos: BlockPos, work: &mut Work)
    {
        let (Some(input), Some(can)) = (world.tileSlot(pos, slot::INPUT), world.tileSlot(pos, slot::CAN))
        else
        {
            return;
        };

        match mode(world, pos)
        {
            Mode::Food =>
            {
                work.m_fuel = 0;
                let cans: i32 = food(&input);
                let emptied: Option<ItemStack> = match stacks::less(input.clone(), 1)
                {
                    None if input.item.is("minecraft:mushroom_stew") => Some(ItemStack::of("minecraft:bowl", 1)),
                    left => left,
                };
                stacks::set(world, pos, slot::INPUT, emptied);
                stacks::set(world, pos, slot::CAN, stacks::less(can, cans));
                let filled: ItemStack = ItemStack::of("ic2:filled_tin_can", cans);
                stacks::set(world, pos, slot::OUTPUT, Some(stacks::merged(world.tileSlot(pos, slot::OUTPUT), &filled)));
            }

            Mode::Fuel =>
            {
                work.m_fuel += fuel(&input, work.m_fuel);
                stacks::set(world, pos, slot::INPUT, stacks::less(input, 1));

                if work.m_progress < rules::FUEL_DONE
                {
                    return;
                }

                let made: ItemStack = if can.item.is("ic2:fuel_can_empty")
                {
                    stacks::set(world, pos, slot::CAN, stacks::less(can, 1));
                    ItemStack::of("ic2:fuel_can", 1).withDamage(work.m_fuel)
                }
                else
                {
                    stacks::set(world, pos, slot::CAN, None);
                    ItemStack::of("ic2:jetpack", 1).withDamage((can.damage - work.m_fuel).max(1))
                };
                stacks::set(world, pos, slot::OUTPUT, Some(made));
            }

            Mode::Idle =>
            {}
        }
    }
}

// GuiIronFurnace, GuiElecFurnace, GuiMacerator and the rest: the fuel or energy flame and the progress arrow
struct ProcessScreen
{
    m_process: &'static Process,
    m_pos: Option<BlockPos>,
}

mod gauge
{
    use super::Bar;

    pub const ENERGY: Bar = Bar { m_at: [56, 36], m_uv: [176, 0], m_size: [14, 14] };
    pub const FLAME: Bar = Bar { m_at: [56, 36], m_uv: [176, 0], m_size: [14, 12] };
    pub const ARROW: Bar = Bar { m_at: [79, 34], m_uv: [176, 14], m_size: [24, 16] };
    pub const CANNER_ENERGY: Bar = Bar { m_at: [31, 27], m_uv: [176, 0], m_size: [14, 14] };
    pub const CANNER_ARROW: Bar = Bar { m_at: [74, 36], m_uv: [176, 15], m_size: [34, 13] };
    pub const FLAME_EXTRA: i32 = 2;
    pub const ARROW_EXTRA: i32 = 1;
    pub const TITLE_X: [(&str, i32); 7] = [
        ("iron_furnace", 55),
        ("electric_furnace", 50),
        ("macerator", 58),
        ("extractor", 58),
        ("compressor", 58),
        ("recycler", 58),
        ("canning_machine", 50),
    ];
}

impl Screen for ProcessScreen
{
    fn init(&mut self, _width: i32, _height: i32, pos: Option<BlockPos>, _data: &[u8]) { self.m_pos = pos; }

    fn draw(&mut self, draw: &mut Draw, _mouse: [i32; 2], _partial: f32)
    {
        let process: &Process = self.m_process;
        let titleX: i32 = gauge::TITLE_X.iter().find(|(name, _)| *name == process.m_name).map_or(0, |(_, x)| *x);
        screen::labels(draw, &screen::name(process.m_name), titleX);

        let Some(work) = screen::state::<Work>(self.m_pos)
        else
        {
            return;
        };
        let rates: Rates = process.m_rates;
        let sheet: &str = sheetOf(process.m_name);

        match process.m_kind
        {
            Kind::IronFurnace =>
            {
                if work.m_fuel > 0
                {
                    let max: i32 = if work.m_maxFuel > 0
                    {
                        work.m_maxFuel
                    }
                    else
                    {
                        rates.m_length
                    };
                    let level: i32 = work.m_fuel * gauge::FLAME.m_size[1] / max;
                    screen::rising(draw, sheet, &gauge::FLAME, level, gauge::FLAME_EXTRA);
                }

                let arrow: i32 = work.m_progress * gauge::ARROW.m_size[0] / rates.m_length;
                screen::across(draw, sheet, &gauge::ARROW, arrow + gauge::ARROW_EXTRA);
            }

            Kind::Canner =>
            {
                let energy: i32 = (work.m_energy * gauge::CANNER_ENERGY.m_size[1] / (rates.m_length * rates.m_consume))
                    .min(gauge::CANNER_ENERGY.m_size[1]);
                if work.m_energy > 0
                {
                    screen::rising(draw, sheet, &gauge::CANNER_ENERGY, energy, 0);
                }

                let arrow: i32 = work.m_progress * gauge::CANNER_ARROW.m_size[0] / rates.m_length;
                screen::across(draw, sheet, &gauge::CANNER_ARROW, arrow);
            }

            _ =>
            {
                if work.m_energy > 0
                {
                    let size: i32 = gauge::ENERGY.m_size[1];
                    let energy: i32 = (work.m_energy * size / (rates.m_length * rates.m_consume)).min(size);
                    screen::rising(draw, sheet, &gauge::ENERGY, energy, 0);
                }

                let arrow: i32 = work.m_progress * gauge::ARROW.m_size[0] / rates.m_length;
                screen::across(draw, sheet, &gauge::ARROW, arrow + gauge::ARROW_EXTRA);
            }
        }
    }
}

fn sheetOf(name: &str) -> &'static str
{
    match name
    {
        "iron_furnace" => "furnace",
        "electric_furnace" => "elecfurnace",
        "canning_machine" => "canner",
        "macerator" => "macerator",
        "extractor" => "extractor",
        "compressor" => "compressor",
        _ => "recycler",
    }
}

fn registerScreen(ctx: &mut RegisterCtx, process: &'static Process) -> Result<()>
{
    let background: String = match process.m_kind
    {
        Kind::IronFurnace => "minecraft:gui/furnace".to_owned(),
        _ => screen::texture(sheetOf(process.m_name)),
    };
    let mut builder: ScreenBuilder = ScreenBuilder::new("", screen::WIDTH, screen::HEIGHT)
        .background(&background)
        .playerInventory(screen::INVENTORY[0], screen::INVENTORY[1]);

    // ContainerElectricMachine, ContainerIronFurnace and ContainerCanner slot positions
    let [input, fuel, output, can]: [[i32; 2]; 4] = match process.m_kind
    {
        Kind::Canner => [[69, 17], [30, 45], [119, 35], [69, 53]],
        _ => [[56, 17], [56, 53], [116, 35], [0, 0]],
    };
    builder = builder.slot(slot::INPUT, input[0], input[1]).slot(slot::FUEL, fuel[0], fuel[1]).outputSlot(
        slot::OUTPUT,
        output[0],
        output[1],
    );
    if process.m_kind == Kind::Canner
    {
        builder = builder.slot(slot::CAN, can[0], can[1]);
    }

    ctx.screens().register(process.m_name, builder, ProcessScreen { m_process: process, m_pos: None })
}

// a recipe viewer category for a table machine, one page a recipe with the machine's rates
fn tablePages(ctx: &mut RegisterCtx, process: &Process, table: &[(&str, &str, i32)], rates: &str) -> Result<()>
{
    ctx.machines().register(process.m_name, None, &[process.m_name])?;

    for (from, to, count) in table
    {
        ctx.machines()
            .recipe(process.m_name)
            .input(ItemStack::of(*from, 1).withDamage(ANY_DAMAGE))
            .output(ItemStack::of(*to, *count))
            .note(rates)
            .build()?;
    }

    Ok(())
}

// what each processing machine does as recipe viewers show it, the furnaces join smelting
fn registerPages(ctx: &mut RegisterCtx, process: &Process) -> Result<()>
{
    const SCRAP_CHANCE: f32 = 1.0 / 8.0;

    let Rates { m_consume: consume, m_length: length, .. } = process.m_rates;
    let rates: String = format!("{consume} EU/t, {length} ticks, {} EU", consume * length);

    match process.m_kind
    {
        Kind::IronFurnace | Kind::Furnace => ctx.machines().workstation("minecraft:smelting", process.m_name),
        Kind::Macerator => tablePages(ctx, process, &MACERATOR, &rates),
        Kind::Extractor => tablePages(ctx, process, &EXTRACTOR, &rates),
        Kind::Compressor => tablePages(ctx, process, &COMPRESSOR, &rates),
        Kind::Recycler =>
        {
            ctx.machines().register(process.m_name, None, &[process.m_name])?;
            ctx.machines()
                .recipe(process.m_name)
                .chance(ItemStack::of("ic2:scrap", 1), SCRAP_CHANCE)
                .note("Takes any item but sticks and scrap")
                .note(&rates)
                .build()
        }
        Kind::Canner => canning::pages(ctx, process.m_name, consume),
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for process in &PROCESSES
    {
        registerPages(ctx, process)?;

        let Some(machine) = MACHINES.iter().find(|row| row.m_name == process.m_name)
        else
        {
            continue;
        };

        registerBlock(ctx, machine, Rc::new(ProcessTile { m_process: process }), true)?;
        registerScreen(ctx, process)?;

        if process.m_kind != Kind::IronFurnace
        {
            energy::add(own(process.m_name), process);
        }
    }

    // the plain machine block is a crafting part with no tile of its own
    let frame: &Machine = &MACHINES[MACHINES.len() - 1];
    ctx.blocks().register(
        frame.m_name,
        BlockSpecBuilder::template("minecraft:block")
            .material("iron")
            .sound("metal")
            .hardness(frame.m_hardness)
            .texture(&crate::core::blockTexture("machine_top"))
            .tool(ToolClass::Pickaxe, 0),
    )?;

    Ok(())
}
