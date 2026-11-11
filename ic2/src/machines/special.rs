use super::processing::{provide, smelted};
use super::screen::{self, Bar};
use super::{Faces, Look, Machine, registerBlock, setActive};
use crate::blocks::{IRON_FENCE, fenceCharge, setFenceCharge};
use crate::core::{dir, own, sound, stacks};
use crate::energy::storage::{Bank, bankAt, bankEnergy, setBankEnergy};
use crate::energy::{self, EnergyTile, net};
use crate::explosives;
use b173_api::prelude::*;
use std::rc::Rc;

const HARDNESS: f32 = 2.0;

const fn same(sprites: [&'static str; 4]) -> Look
{
    Look { m_idle: sprites, m_active: sprites, m_faces: Faces::Machine }
}

const fn lit(idle: [&'static str; 4], active: [&'static str; 4]) -> Look
{
    Look { m_idle: idle, m_active: active, m_faces: Faces::Machine }
}

const fn machine(name: &'static str, look: Look, wrench: f32, drop: &'static str, screen: bool, slots: i32) -> Machine
{
    Machine
    {
        m_name: name,
        m_look: look,
        m_hardness: HARDNESS,
        m_screen: screen,
        m_wrench: wrench,
        m_drop: drop,
        m_passes: Some("ic2:machine_passes"),
        m_slots: slots,
    }
}

pub(crate) static MACHINES: [Machine; 6] = [
    machine(
        "magnetizer",
        same(["machine_bottom", "machine_top", "machine_side", "machine_side"]),
        0.0,
        "magnetizer",
        false,
        0,
    ),
    machine(
        electrolyzer::NAME,
        lit(
            ["machine_bottom", "machine_top", "extractor_side", "extractor_side"],
            ["machine_bottom", "machine_top", "miner_side", "miner_side"],
        ),
        1.0,
        "machine",
        true,
        electrolyzer::SLOTS,
    ),
    machine(
        "induction_furnace",
        lit(
            ["machine_bottom", "advanced_machine", "machine_side", "induction_furnace_front"],
            ["machine_bottom", "advanced_machine", "machine_side", "induction_furnace_front_active"],
        ),
        0.8,
        "advanced_machine",
        true,
        induction::SLOTS,
    ),
    machine(
        matter::NAME,
        lit(
            ["machine_bottom", "advanced_machine", "extractor_side", "recycler_top"],
            ["machine_bottom", "advanced_machine", "extractor_side_active", "recycler_top"],
        ),
        0.7,
        "advanced_machine",
        true,
        matter::SLOTS,
    ),
    machine("tesla_coil", same(["tesla_coil"; 4]), 1.0, "tesla_coil", false, 0),
    machine("advanced_machine", same(["advanced_machine"; 4]), 0.0, "advanced_machine", false, 0),
];

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) struct State
{
    pub(crate) m_energy: i32,
    pub(crate) m_progress: i32,
    // the induction furnace's heat, the mass fabricator's scrap amplifier
    pub(crate) m_heat: i32,
    m_ticker: i32,
}

mod electrolyzer
{
    pub const HUM_EVERY: i32 = 32;
    pub const RATE: i32 = 10;
    pub const SLOTS: i32 = 2;
    // a cell takes 15000 EU to split and gives 13500 back
    pub const NAME: &str = "electrolyzer";
    pub const SPLIT: i32 = 15000;
    pub const JOIN: i32 = 13500;
    pub const DRAIN_ABOVE: f64 = 0.7;
    pub const POWER_BELOW: f64 = 0.3;
    pub const IN: i32 = 0;
    pub const OUT: i32 = 1;
}

fn bank(world: World, pos: BlockPos) -> Option<Bank>
{
    [dir::DOWN, dir::UP, dir::WEST, dir::EAST, dir::NORTH, dir::SOUTH]
        .iter()
        .find_map(|face| bankAt(world, dir::step(pos, *face)))
}

// the electrolyzer job a cell matches with its output and the energy the picked side carries
fn cellJob(cell: ItemStack, energy: impl Fn(&JobMatch) -> Option<&JobEnergySpec>) -> Option<(ItemStack, i64)>
{
    let job: JobMatch = matchJob(electrolyzer::NAME, &[Some(cell)], &[])?;
    let amount: i64 = energy(&job)?.amount;
    let output: JobOutput = job.outputs.into_iter().next()?;
    let made: ItemStack = output.stack.map(|stack| ItemStack { count: output.count, ..ItemStack::fromRaw(stack) })?;
    Some((made, amount))
}

// TileEntityElectrolyzer.updateEntity: water cells soak up an MFE's surplus and give it back when it runs low
fn electrolyze(world: World, pos: BlockPos, state: &mut State)
{
    state.m_ticker += 1;
    let Some(bank) = bank(world, pos)
    else
    {
        return;
    };

    let hum: i32 = state.m_ticker;
    state.m_ticker += 1;
    if hum % electrolyzer::HUM_EVERY == 0
    {
        world.playSound(dir::center(pos), &sound("machineElectrolyzerLoop"), 1.0, 1.0);
    }

    let input: Option<ItemStack> = world.tileSlot(pos, electrolyzer::IN);
    let output: Option<ItemStack> = world.tileSlot(pos, electrolyzer::OUT);
    let fill: f64 = f64::from(bankEnergy(world, &bank)) / f64::from(bank.m_capacity);
    let mut working: bool = false;

    // splitting takes the job the water cell in the input slot matches, its energy in a total cost
    let split: Option<(ItemStack, i64)> = input.clone().and_then(|cell| cellJob(cell, |job| job.energy_in.as_ref()));
    if fill >= electrolyzer::DRAIN_ABOVE
        && let Some((made, cost)) = split.filter(|(made, _)| stacks::fits(&output, made))
    {
        setBankEnergy(world, &bank, bankEnergy(world, &bank) - electrolyzer::RATE);
        state.m_energy += electrolyzer::RATE;
        if i64::from(state.m_energy) >= cost
        {
            state.m_energy -= cost as i32;
            stacks::set(world, pos, electrolyzer::IN, input.clone().and_then(|cell| stacks::less(cell, 1)));
            stacks::set(world, pos, electrolyzer::OUT, Some(stacks::merged(output.clone(), &made)));
        }

        working = true;
    }

    // joining takes the job the electrolyzed cell in the output slot matches, its energy given back in total
    let input: Option<ItemStack> = world.tileSlot(pos, electrolyzer::IN);
    let output: Option<ItemStack> = world.tileSlot(pos, electrolyzer::OUT);
    let join: Option<(ItemStack, i64)> = output.clone().and_then(|cell| cellJob(cell, |job| job.energy_out.as_ref()));
    let canPower: bool = join.as_ref().is_some_and(|(made, _)| stacks::fits(&input, made));
    if fill <= electrolyzer::POWER_BELOW && (canPower || state.m_energy > 0)
    {
        if state.m_energy > 0
        {
            let out: i32 = electrolyzer::RATE.min(state.m_energy);
            state.m_energy -= out;
            setBankEnergy(world, &bank, bankEnergy(world, &bank) + out);
        }
        else if let (Some(cell), Some((made, gain))) = (output, join)
        {
            state.m_energy += gain as i32;
            stacks::set(world, pos, electrolyzer::OUT, stacks::less(cell, 1));
            stacks::set(world, pos, electrolyzer::IN, Some(stacks::merged(input, &made)));
        }

        working = true;
    }

    setActive(world, pos, working);
}

mod induction
{
    pub const MAX_HEAT: i32 = 10000;
    pub const INPUT: i32 = 128;
    pub const COOLING: i32 = 4;
    pub const RUN_COST: i32 = 15;
    pub const DONE: i32 = 4000;
    pub const HEAT_STEP: i32 = 30;
    pub const GLOW: i32 = 3000;
    pub const HUM_EVERY: i32 = 64;
    pub const FUEL: i32 = 1;
    pub const SLOTS: i32 = 5;
    // two lines: input 0 to output 2 and input 3 to output 4
    pub const LINES: [(i32, i32); 2] = [(0, 2), (3, 4)];
}

fn lineReady(world: World, pos: BlockPos, line: (i32, i32)) -> bool
{
    let Some(input) = world.tileSlot(pos, line.0)
    else
    {
        return false;
    };

    smelted(&input).is_some_and(|made| stacks::fits(&world.tileSlot(pos, line.1), &made))
}

fn smeltLine(world: World, pos: BlockPos, line: (i32, i32))
{
    let Some(input) = world.tileSlot(pos, line.0).filter(|_| lineReady(world, pos, line))
    else
    {
        return;
    };

    if let Some(made) = smelted(&input)
    {
        stacks::set(world, pos, line.1, Some(stacks::merged(world.tileSlot(pos, line.1), &made)));
        stacks::set(world, pos, line.0, stacks::consume(input));
    }
}

// the GretoxTweaks change in this build: finished items move into a chest next to the furnace
fn deposit(world: World, pos: BlockPos)
{
    for dx in -1..=1
    {
        for dz in -1..=1
        {
            for dy in -1..=1
            {
                let at: BlockPos = pos.offset(dx, dy, dz);
                if at == pos || !world.tagged(at, "c:chests")
                {
                    continue;
                }

                for (_, output) in induction::LINES
                {
                    let Some(stack) = world.tileSlot(pos, output)
                    else
                    {
                        continue;
                    };
                    let left: i32 = world.insertItem(at, dir::UP, stack.clone(), false).unwrap_or(stack.count);
                    stacks::set(world, pos, output, (left > 0).then_some(ItemStack { count: left, ..stack }));
                }

                return;
            }
        }
    }
}

// TileEntityInduction.updateEntity
fn induce(world: World, pos: BlockPos, state: &mut State)
{
    let ready: bool = induction::LINES.iter().any(|line| lineReady(world, pos, *line));
    let mut working: bool = false;

    if state.m_energy > 0 && (ready || world.isPowered(pos))
    {
        state.m_energy -= 1;
        if state.m_heat < induction::MAX_HEAT
        {
            state.m_heat += 1;
        }

        working = true;
    }
    else if state.m_heat > 0
    {
        state.m_heat -= induction::COOLING;
    }

    deposit(world, pos);

    if state.m_energy > 0 && ready
    {
        state.m_energy -= induction::RUN_COST;
        working = true;
    }

    if state.m_heat == 0
    {
        state.m_progress = 0;
    }

    if state.m_energy <= induction::MAX_HEAT
    {
        provide(&(world, pos), induction::FUEL, &mut state.m_energy, induction::MAX_HEAT);
    }

    if state.m_energy > 0 && ready
    {
        state.m_ticker += 1;
        if state.m_ticker % induction::HUM_EVERY == 0
        {
            world.playSound(dir::center(pos), &sound("machineInductionLoop"), 1.0, 1.0);
        }

        state.m_progress += state.m_heat / induction::HEAT_STEP;
        if state.m_progress >= induction::DONE
        {
            state.m_progress = 0;
            for line in induction::LINES
            {
                smeltLine(world, pos, line);
            }
        }
    }

    setActive(world, pos, working || state.m_heat >= induction::GLOW);
}

mod matter
{
    pub const CAPACITY: i32 = 1100000;
    pub const INPUT: i32 = 512;
    pub const NAME: &str = "mass_fabricator";
    pub const COST: i32 = 1000000;
    pub const SCRAP_BELOW: i32 = 1000;
    pub const SCRAP: i32 = 5000;
    pub const BONUS: i32 = 5;
    pub const HUM_EVERY: i32 = 32;
    pub const BLAST: f32 = 60.0;
    pub const AMPLIFIER: i32 = 0;
    pub const SLOTS: i32 = 2;
    pub const OUTPUT: i32 = 1;
}

// TileEntityMatter.updateEntity: scrap feeds the amplifier, a million EU makes one UU-Matter
fn fabricate(world: World, pos: BlockPos, state: &mut State)
{
    state.m_ticker += 1;
    if state.m_ticker % matter::HUM_EVERY == 0
    {
        world.playSound(dir::center(pos), &sound("machineMatterLoop"), 1.0, 1.0);
    }

    // the amplifier value of a stack is the fabricator's fuel entry, so other mods can feed it
    if state.m_heat < matter::SCRAP_BELOW
        && let Some(scrap) = world.tileSlot(pos, matter::AMPLIFIER)
    {
        let amplifier: i32 = machineFuel(matter::NAME, &scrap, false);
        if amplifier > 0
        {
            stacks::set(world, pos, matter::AMPLIFIER, stacks::less(scrap, 1));
            state.m_heat += amplifier;
        }
    }

    if state.m_energy >= matter::COST
    {
        let uu: ItemStack = ItemStack::of("ic2:uu_matter", 1);
        let out: Option<ItemStack> = world.tileSlot(pos, matter::OUTPUT);
        if stacks::fits(&out, &uu)
        {
            state.m_energy -= matter::COST;
            stacks::set(world, pos, matter::OUTPUT, Some(stacks::merged(out, &uu)));
        }
    }

    setActive(world, pos, state.m_energy > 0);
}

mod magnet
{
    pub const CAPACITY: i32 = 100;
    pub const INPUT: i32 = 32;
    pub const REACH: i32 = 20;
    pub const FULL: i32 = 15;
    pub const REST: i32 = 10;
    pub const TOP: i32 = 128;
}

// TileEntityMagnetizer.updateEntity: charges the iron fence poles above and below it
fn magnetize(world: World, pos: BlockPos, state: &mut State)
{
    let ticker: i32 = state.m_ticker;
    state.m_ticker -= 1;
    if ticker > 0
    {
        return;
    }

    let mut charged: bool = false;

    for step in [-1, 1]
    {
        let mut y: i32 = pos.y + step;

        while y > 0 && y < magnet::TOP && (y - pos.y).abs() <= magnet::REACH && state.m_energy > 0
        {
            let at: BlockPos = BlockPos::new(pos.x, y, pos.z);
            let Some(charge) = fenceCharge(world, at)
            else
            {
                break;
            };

            let need: i32 = magnet::FULL - charge;
            if need > 0
            {
                charged = true;
                state.m_energy = state.m_energy.max(need);
                setFenceCharge(world, at, charge + need);
                state.m_energy -= need;
            }

            y += step;
        }
    }

    if !charged
    {
        state.m_ticker = magnet::REST;
    }
}

mod tesla
{
    pub const CAPACITY: i32 = 10000;
    pub const INPUT: i32 = 128;
    pub const COST: i32 = 400;
    pub const EVERY: i32 = 32;
    pub const LOW: f64 = 4.0;
    pub const HIGH: f64 = 5.0;
    pub const SPARK_HEIGHT: f64 = 2.0;
}

// TileEntityTesla.updateEntity: while powered by redstone it shocks everything living within four blocks
fn zap(world: World, pos: BlockPos, state: &mut State)
{
    if state.m_energy < tesla::COST || !world.isPowered(pos)
    {
        return;
    }

    let damage: i32 = state.m_energy / tesla::COST;
    state.m_energy -= 1;
    if state.m_ticker % tesla::EVERY != 0
    {
        return;
    }

    let min: [f64; 3] = [pos.x as f64 - tesla::LOW, pos.y as f64 - tesla::LOW, pos.z as f64 - tesla::LOW];
    let max: [f64; 3] = [pos.x as f64 + tesla::HIGH, pos.y as f64 + tesla::HIGH, pos.z as f64 + tesla::HIGH];
    let living: Vec<EntityInfo> = world.entitiesIn(min, max).into_iter().filter(|info| info.health.is_some()).collect();

    for info in &living
    {
        Entity { world, id: info.id }.damage(damage, None);
        for _ in 0..damage
        {
            let r = || crate::core::random::float(world);
            world.particle(
                "reddust",
                [info.pos[0] + r(), info.pos[1] + r() * tesla::SPARK_HEIGHT, info.pos[2] + r()],
                [0.0, 0.0, 1.0],
            );
        }
    }

    if !living.is_empty()
    {
        state.m_energy = 0;
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind
{
    Magnetizer,
    Electrolyzer,
    Induction,
    Matter,
    Tesla,
}

// each machine's energy rules, IEnergySink with its capacity and rating; the electrolyzer takes none from cables
struct Special
{
    m_name: &'static str,
    m_kind: Kind,
    m_capacity: i32,
    m_input: i32,
}

static SPECIALS: [Special; 5] = [
    Special { m_name: "magnetizer", m_kind: Kind::Magnetizer, m_capacity: magnet::CAPACITY, m_input: magnet::INPUT },
    Special { m_name: electrolyzer::NAME, m_kind: Kind::Electrolyzer, m_capacity: 0, m_input: 0 },
    Special
    {
        m_name: "induction_furnace",
        m_kind: Kind::Induction,
        m_capacity: induction::MAX_HEAT,
        m_input: induction::INPUT,
    },
    Special { m_name: matter::NAME, m_kind: Kind::Matter, m_capacity: matter::CAPACITY, m_input: matter::INPUT },
    Special { m_name: "tesla_coil", m_kind: Kind::Tesla, m_capacity: tesla::CAPACITY, m_input: tesla::INPUT },
];

impl Special
{
    // TileEntityElecMachine.demandsEnergy leaves room for a packet, the magnetizer and tesla fill to the brim
    fn room(&self) -> i32
    {
        match self.m_kind
        {
            Kind::Induction | Kind::Matter => self.m_capacity - self.m_input,
            _ => self.m_capacity,
        }
    }

    fn take(&self, world: World, pos: BlockPos, state: &mut State, amount: i32) -> i32
    {
        if amount > self.m_input
        {
            if self.m_kind == Kind::Matter
            {
                let _ = world.clear(pos);
                explosives::blast(world, [pos.x as f64, pos.y as f64, pos.z as f64], matter::BLAST, explosives::MATTER);
                net::changed(world);
            }
            else
            {
                net::overload(world, pos);
            }

            return 0;
        }

        let mut gain: i32 = amount;
        if self.m_kind == Kind::Matter
        {
            let bonus: i32 = amount.min(state.m_heat);
            state.m_heat -= bonus;
            gain += matter::BONUS * bonus;
        }

        state.m_energy += gain;
        let left: i32 = (state.m_energy - self.m_capacity).max(0);
        state.m_energy -= left;
        left
    }
}

impl EnergyTile for Special
{
    fn accepts(&self, _world: World, _pos: BlockPos, _face: i32) -> bool { self.m_kind != Kind::Electrolyzer }

    fn sink(&self) -> bool { self.m_kind != Kind::Electrolyzer }

    // TileEntityElectrolyzer is no energy tile at all, cables pass it by
    fn joins(&self, _world: World, _pos: BlockPos, _from: BlockPos) -> bool { self.m_kind != Kind::Electrolyzer }

    fn demands(&self, world: World, pos: BlockPos) -> bool { world.tileData::<State>(pos).m_energy < self.room() }

    fn inject(&self, world: World, pos: BlockPos, _face: i32, amount: i32) -> i32
    {
        let mut state: State = world.tileData(pos);
        let left: i32 = self.take(world, pos, &mut state, amount);
        if world.block(pos) != 0
        {
            let _ = world.setTileData(pos, &state);
        }

        left
    }
}

struct SpecialTile
{
    m_special: &'static Special,
}

#[tile_behavior]
impl TileBehavior for SpecialTile
{
    fn onTick(&self, ctx: &mut BlockCtx)
    {
        if !ctx.isAuthority()
        {
            return;
        }

        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();
        let before: State = ctx.tileData();
        let mut state: State = before;

        match self.m_special.m_kind
        {
            Kind::Magnetizer => magnetize(world, pos, &mut state),
            Kind::Electrolyzer => electrolyze(world, pos, &mut state),
            Kind::Induction => induce(world, pos, &mut state),
            Kind::Matter => fabricate(world, pos, &mut state),
            Kind::Tesla => zap(world, pos, &mut state),
        }

        if state != before
        {
            let _ = ctx.setTileData(&state);
        }
    }

    fn energyDemand(&self, ctx: &mut BlockCtx, _face: i32) -> Option<EnergyDemand>
    {
        let special: &Special = self.m_special;
        let full: bool = ctx.tileData::<State>().m_energy >= special.room();
        if !special.sink() || full
        {
            return None;
        }

        energy::demand(special.m_input)
    }

    fn receiveEnergy(&self, ctx: &mut BlockCtx, _face: i32, amount: i64, simulate: bool) -> i64
    {
        let special: &Special = self.m_special;
        let mut state: State = ctx.tileData();
        let offered: i32 = amount.clamp(0, i64::from(special.m_input)) as i32;
        let taken: i32 = offered.min(special.m_capacity - state.m_energy).max(0);

        if !simulate
        {
            special.take(ctx.world(), ctx.pos(), &mut state, taken);
            let _ = ctx.setTileData(&state);
        }

        i64::from(taken)
    }
}

// a container slot: its index, where it sits and whether it only gives items out
type SlotAt = (i32, [i32; 2], bool);

// GuiElectrolyzer, GuiInduction and GuiMatter
struct SpecialScreen
{
    m_special: &'static Special,
    m_pos: Option<BlockPos>,
}

mod layout
{
    use super::Bar;

    pub const ARROW: Bar = Bar { m_at: [79, 34], m_uv: [176, 14], m_size: [24, 16] };
    pub const ENERGY: Bar = Bar { m_at: [56, 36], m_uv: [176, 0], m_size: [14, 14] };
    pub const ARROW_EXTRA: i32 = 1;
    pub const HEAT_LABEL: [i32; 2] = [10, 36];
    pub const HEAT_VALUE: [i32; 2] = [10, 44];
    pub const PROGRESS_LABEL: [i32; 2] = [16, 20];
    pub const PROGRESS_VALUE: [i32; 2] = [16, 28];
    pub const AMPLIFIER_LABEL: [i32; 2] = [16, 44];
    pub const AMPLIFIER_VALUE: [i32; 2] = [16, 56];
    pub const PERCENT: i32 = 100;
    pub const PROGRESS_STEP: i32 = 10000;
}

impl Screen for SpecialScreen
{
    fn init(&mut self, _width: i32, _height: i32, pos: Option<BlockPos>, _data: &[u8]) { self.m_pos = pos; }

    fn draw(&mut self, draw: &mut Draw, _mouse: [i32; 2], _partial: f32)
    {
        let special: &Special = self.m_special;
        let state: State = screen::state(self.m_pos).unwrap_or_default();

        match special.m_kind
        {
            Kind::Electrolyzer =>
            {
                const TITLE_X: i32 = 56;

                screen::labels(draw, &screen::name(special.m_name), TITLE_X);
                if state.m_energy > 0
                {
                    let level: i32 =
                        (state.m_energy * layout::ARROW.m_size[0] / electrolyzer::SPLIT).min(layout::ARROW.m_size[0]);
                    screen::across(draw, "electrolyzer", &layout::ARROW, level + layout::ARROW_EXTRA);
                }
            }

            Kind::Induction =>
            {
                const TITLE_X: i32 = 49;

                screen::labels(draw, &screen::name(special.m_name), TITLE_X);
                screen::text(draw, layout::HEAT_LABEL[0], layout::HEAT_LABEL[1], "Heat:");
                let heat: String = format!("{}%", state.m_heat * layout::PERCENT / induction::MAX_HEAT);
                screen::text(draw, layout::HEAT_VALUE[0], layout::HEAT_VALUE[1], &heat);

                if state.m_energy > 0
                {
                    let level: i32 = layout::ENERGY.m_size[1] * state.m_energy / induction::MAX_HEAT;
                    screen::rising(draw, "induction", &layout::ENERGY, level, 0);
                }

                let arrow: i32 = layout::ARROW.m_size[0] * state.m_progress / induction::DONE;
                screen::across(draw, "induction", &layout::ARROW, arrow + layout::ARROW_EXTRA);
            }

            Kind::Matter =>
            {
                const TITLE_X: i32 = 54;

                screen::labels(draw, &screen::name(special.m_name), TITLE_X);
                screen::text(draw, layout::PROGRESS_LABEL[0], layout::PROGRESS_LABEL[1], "Progress:");
                let progress: String = format!("{}%", (state.m_energy / layout::PROGRESS_STEP).min(layout::PERCENT));
                screen::text(draw, layout::PROGRESS_VALUE[0], layout::PROGRESS_VALUE[1], &progress);

                if state.m_heat > 0
                {
                    screen::text(draw, layout::AMPLIFIER_LABEL[0], layout::AMPLIFIER_LABEL[1], "Amplifier:");
                    screen::text(
                        draw,
                        layout::AMPLIFIER_VALUE[0],
                        layout::AMPLIFIER_VALUE[1],
                        &state.m_heat.to_string(),
                    );
                }
            }

            _ =>
            {}
        }
    }
}

fn registerScreen(ctx: &mut RegisterCtx, special: &'static Special) -> Result<()>
{
    // ContainerElectrolyzer, ContainerInduction and ContainerMatter slot positions, outputs marked
    const ELECTROLYZER: [SlotAt; 2] = [(electrolyzer::IN, [56, 35], false), (electrolyzer::OUT, [116, 35], false)];
    const INDUCTION: [SlotAt; 5] = [
        (0, [47, 17], false),
        (induction::FUEL, [56, 53], false),
        (2, [113, 35], true),
        (3, [63, 17], false),
        (4, [131, 35], true),
    ];
    const MATTER: [SlotAt; 2] = [(matter::AMPLIFIER, [114, 54], false), (matter::OUTPUT, [114, 18], true)];

    let (sheet, slots): (&str, &[SlotAt]) = match special.m_kind
    {
        Kind::Electrolyzer => ("electrolyzer", &ELECTROLYZER),
        Kind::Induction => ("induction", &INDUCTION),
        Kind::Matter => ("matter", &MATTER),
        _ => return Ok(()),
    };
    let mut builder: ScreenBuilder = screen::container(sheet);
    for (index, [x, y], output) in slots
    {
        builder = if *output
        {
            builder.outputSlot(*index, *x, *y)
        }
        else
        {
            builder.slot(*index, *x, *y)
        };
    }

    ctx.screens().register(special.m_name, builder, SpecialScreen { m_special: special, m_pos: None })
}

// what each special machine does as recipe viewers show it
fn registerPages(ctx: &mut RegisterCtx, special: &Special) -> Result<()>
{
    const AMPLIFY: i32 = matter::BONUS + 1;

    let name: &str = special.m_name;
    if special.m_kind == Kind::Induction
    {
        return ctx.machines().workstation("minecraft:smelting", name);
    }

    ctx.machines().register(name, None, &[name])?;

    match special.m_kind
    {
        Kind::Electrolyzer =>
        {
            // the electrolyzer works one cell at a time as its bank allows, so its jobs carry no fixed duration
            ctx.machines()
                .job(name, 0)
                .input("ic2:water_cell", 1)
                .output("ic2:electrolyzed_water_cell", 1)
                .energyIn(crate::energy::KIND, i64::from(electrolyzer::SPLIT), false)
                .note(&format!("Stores {} EU from an MFE or MFSU above 70%", electrolyzer::SPLIT))
                .build()?;
            ctx.machines()
                .job(name, 0)
                .input("ic2:electrolyzed_water_cell", 1)
                .output("ic2:water_cell", 1)
                .energyOut(crate::energy::KIND, i64::from(electrolyzer::JOIN), false)
                .note(&format!("Gives {} EU back below 30%", electrolyzer::JOIN))
                .build()
        }

        Kind::Matter =>
        {
            ctx.machines().fuel(name, "ic2:scrap", matter::SCRAP)?;
            ctx.machines()
                .recipe(name)
                .output(ItemStack::of("ic2:uu_matter", 1))
                .note(&format!("{} EU, up to {} EU/t", matter::COST, matter::INPUT))
                .build()?;
            ctx.machines()
                .recipe(name)
                .input(ItemStack::of("ic2:scrap", 1))
                .note(&format!("Amplifier: the next {} EU count {AMPLIFY} times", matter::SCRAP))
                .build()
        }

        Kind::Magnetizer => ctx
            .machines()
            .recipe(name)
            .input(ItemStack::of(IRON_FENCE, 1))
            .note(&format!("Charges iron fences up to {} blocks above and below", magnet::REACH))
            .note("Players in metal boots ride charged fences up")
            .build(),

        Kind::Tesla => ctx
            .machines()
            .recipe(name)
            .note("Shocks every living thing within 4 blocks")
            .note(&format!("While redstone powered, {} EU a shock every {} ticks", tesla::COST, tesla::EVERY))
            .build(),

        Kind::Induction => Ok(()),
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for special in &SPECIALS
    {
        registerPages(ctx, special)?;

        let Some(machine) = MACHINES.iter().find(|row| row.m_name == special.m_name)
        else
        {
            continue;
        };

        registerBlock(ctx, machine, Rc::new(SpecialTile { m_special: special }), true)?;
        registerScreen(ctx, special)?;
        energy::add(own(special.m_name), special);
    }

    let frame: &Machine = &MACHINES[MACHINES.len() - 1];
    ctx.blocks().register(
        frame.m_name,
        BlockSpecBuilder::template("minecraft:block")
            .material("iron")
            .sound("metal")
            .hardness(frame.m_hardness)
            .texture(&crate::core::blockTexture("advanced_machine"))
            .tool(ToolClass::Pickaxe, 0),
    )?;
    Ok(())
}
