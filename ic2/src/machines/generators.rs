use super::processing::burnValue;
use super::screen::{self, Bar};
use super::{Faces, Look, Machine, registerBlock, setActive};
use crate::core::{dir, ids, own, sound, stacks};
use crate::electric;
use crate::energy::{self, EnergyTile, net};
use b173_api::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

mod slot
{
    pub const CHARGE: i32 = 0;
    pub const FUEL: i32 = 1;
}

// BlockGenerator.setHardness, TileEntityBaseGenerator.wrenchRate
const HARDNESS: f32 = 3.0;
const WRENCH: f32 = 0.9;
const TWO_SLOTS: i32 = 2;
const ONE_SLOT: i32 = 1;

// BlockGenerator.damageDropped keeps the water mill and turns every other generator back into a plain one
const fn generator(name: &'static str, look: Look, wrench: f32, drop: &'static str, slots: i32) -> Machine
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

const fn look(idle: [&'static str; 4], active: [&'static str; 4], faces: Faces) -> Look
{
    Look { m_idle: idle, m_active: active, m_faces: faces }
}

pub(crate) static MACHINES: [Machine; 5] = [
    generator(
        "generator",
        look(
            ["machine_bottom", "machine_top", "machine_side", "generator_front"],
            ["machine_bottom", "machine_top", "machine_side", "generator_front_active"],
            Faces::Machine,
        ),
        1.0,
        "generator",
        TWO_SLOTS,
    ),
    generator(
        "geothermal_generator",
        look(
            ["machine_bottom", "machine_top", "machine_side", "geothermal_generator_front"],
            ["machine_bottom", "machine_top", "machine_side", "geothermal_generator_front_active"],
            Faces::Machine,
        ),
        WRENCH,
        "generator",
        TWO_SLOTS,
    ),
    generator(
        "water_mill",
        look(
            ["machine_bottom", "machine_top", "water_mill_side", "water_mill_side"],
            ["machine_bottom", "machine_top", "water_mill_side", "water_mill_side"],
            Faces::Machine,
        ),
        WRENCH,
        "water_mill",
        TWO_SLOTS,
    ),
    generator(
        "solar_panel",
        look(
            ["machine_bottom", "solar_panel_top", "machine_side", "machine_side"],
            ["machine_bottom", "solar_panel_top", "machine_side", "machine_side"],
            Faces::Machine,
        ),
        WRENCH,
        "generator",
        ONE_SLOT,
    ),
    generator(
        "wind_mill",
        look(
            ["machine_bottom", "machine_top", "machine_side", "wind_mill_side"],
            ["machine_bottom", "machine_top", "machine_side", "wind_mill_side"],
            Faces::Axis,
        ),
        WRENCH,
        "generator",
        ONE_SLOT,
    ),
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind
{
    Fuel,
    Geothermal,
    Water,
    Solar,
    Wind,
}

// TileEntityBaseGenerator's numbers per kind: storage, EU a tick and its sound loop
struct Source
{
    m_name: &'static str,
    m_kind: Kind,
    m_capacity: i32,
    m_production: i32,
    m_loop: &'static str,
    m_loopTicks: i32,
}

static SOURCES: [Source; 5] = [
    Source
    {
        m_name: "generator",
        m_kind: Kind::Fuel,
        m_capacity: 4000,
        m_production: 5,
        m_loop: "generatorGeneratorLoop",
        m_loopTicks: 64,
    },
    Source
    {
        m_name: "geothermal_generator",
        m_kind: Kind::Geothermal,
        m_capacity: 100,
        m_production: 10,
        m_loop: "generatorGeoLoop",
        m_loopTicks: 32,
    },
    Source
    {
        m_name: "water_mill",
        m_kind: Kind::Water,
        m_capacity: 100,
        m_production: 2,
        m_loop: "generatorWaterLoop",
        m_loopTicks: 64,
    },
    Source
    {
        m_name: "solar_panel",
        m_kind: Kind::Solar,
        m_capacity: 100,
        m_production: 1,
        m_loop: "",
        m_loopTicks: 256,
    },
    Source
    {
        m_name: "wind_mill",
        m_kind: Kind::Wind,
        m_capacity: 100,
        m_production: 5,
        m_loop: "generatorWindLoop",
        m_loopTicks: 64,
    },
];

mod fuel
{
    // TileEntityGeoGenerator holds 24000 lava ticks, a bucket or cell gives 2000
    pub const MAX_LAVA: i32 = 24000;
    pub const LAVA: i32 = 2000;
    // TileEntityWaterGenerator holds 2000, a bucket gives 500, flowing water fills a micro store of 100
    pub const MAX_WATER: i32 = 2000;
    pub const WATER: i32 = 500;
    pub const MICRO: i32 = 100;
    // the water, solar and wind checks run once every 128 ticks
    pub const RECHECK: i32 = 128;
    pub const WIND_RECOUNT: i32 = 8;
    pub const SKY: i32 = 128;
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug, Default)]
pub(crate) struct Power
{
    pub(crate) m_storage: i32,
    pub(crate) m_fuel: i32,
    pub(crate) m_itemFuel: i32,
    pub(crate) m_ticker: i32,
    pub(crate) m_checked: bool,
    // water blocks around the mill, the sun's visibility, the wind's obstruction
    pub(crate) m_found: i32,
    pub(crate) m_micro: i32,
    pub(crate) m_sub: f64,
    pub(crate) m_subStorage: f64,
    pub(crate) m_production: i32,
    m_sound: i32,
}

thread_local!
{
    // mod_IC2.windStrength, drifting between about 10 and 20 every 128 ticks
    static WIND: Cell<i32> = const { Cell::new(15) };
    static TICKS: Cell<i64> = const { Cell::new(0) };
}

// mod_IC2.updateWind
fn updateWind(world: World)
{
    const CALM: i32 = 10;
    const STRONG: i32 = 20;
    const PERCENT: i32 = 100;

    let wind: i32 = WIND.with(Cell::get);
    let mut rise: i32 = CALM;
    let mut fall: i32 = CALM;
    if wind > STRONG
    {
        rise -= wind - STRONG;
    }

    if wind < CALM
    {
        fall -= CALM - wind;
    }

    if world.random(PERCENT) <= rise
    {
        WIND.with(|cell| cell.set(wind + 1));
    }
    else if world.random(PERCENT) <= fall
    {
        WIND.with(|cell| cell.set(wind - 1));
    }
}

pub(crate) fn wind() -> i32 { WIND.with(Cell::get) }

// TileEntityGeoGenerator.distributeLava: pumped lava tops up the tank, the rest goes back
pub(crate) fn addLava(world: World, pos: BlockPos, amount: i32) -> i32
{
    let mut power: Power = world.tileData(pos);
    let need: i32 = (fuel::MAX_LAVA - power.m_fuel).min(amount).max(0);
    power.m_fuel += need;
    let _ = world.setTileData(pos, &power);
    amount - need
}

fn countAround(world: World, pos: BlockPos, low: [i32; 3], high: [i32; 3], test: impl Fn(i32) -> bool) -> i32
{
    let mut count: i32 = 0;

    for x in low[0]..high[0]
    {
        for y in low[1]..high[1]
        {
            for z in low[2]..high[2]
            {
                if test(world.block(pos.offset(x, y, z)))
                {
                    count += 1;
                }
            }
        }
    }

    count
}

// TileEntitySolarGenerator.updateSunVisibility: daytime and nothing but glass up to the sky
fn sunVisible(world: World, pos: BlockPos) -> bool
{
    const DAY: i64 = 24000;
    const DUSK: i64 = 12000;

    let glass: Option<i32> = ids::block("minecraft:glass");
    let day: bool = world.time().rem_euclid(DAY) < DUSK;
    day && (pos.y + 1..fuel::SKY).all(|y|
    {
        let block: i32 = world.block(BlockPos::new(pos.x, y, pos.z));
        block == 0 || Some(block) == glass
    })
}

// gainFuel of each generator, true when it changed the inventory
fn gainFuel(source: &Source, world: World, pos: BlockPos, power: &mut Power) -> bool
{
    match source.m_kind
    {
        Kind::Fuel =>
        {
            let Some(item) = world.tileSlot(pos, slot::FUEL)
            else
            {
                return false;
            };

            let value: i32 = burnValue(&item) / 2;
            if item.item.is("minecraft:lava_bucket") || value <= 0
            {
                return false;
            }

            power.m_fuel += value;
            power.m_itemFuel = value;
            stacks::set(world, pos, slot::FUEL, stacks::consume(item));
            true
        }

        Kind::Geothermal =>
        {
            let Some(item) = world.tileSlot(pos, slot::FUEL).filter(|_| fuel::MAX_LAVA - power.m_fuel >= fuel::LAVA)
            else
            {
                return false;
            };

            let left: Option<ItemStack> = match item.item.name().map(|id| id.to_string()).as_deref()
            {
                Some("minecraft:lava_bucket") => Some(ItemStack::of("minecraft:bucket", 1)),
                Some("ic2:lava_cell") => stacks::less(item, 1),
                _ => return false,
            };
            power.m_fuel += fuel::LAVA;
            stacks::set(world, pos, slot::FUEL, left);
            true
        }

        Kind::Water =>
        {
            let bucket: Option<ItemStack> =
                world.tileSlot(pos, slot::FUEL).filter(|item| item.item.is("minecraft:water_bucket"));
            if bucket.is_some() && fuel::MAX_WATER - power.m_fuel >= fuel::WATER
            {
                power.m_fuel += fuel::WATER;
                stacks::set(world, pos, slot::FUEL, Some(ItemStack::of("minecraft:bucket", 1)));
                return true;
            }

            if power.m_fuel <= 0
            {
                flowPower(world, pos, power);
            }

            false
        }

        Kind::Solar =>
        {
            if power.m_ticker % fuel::RECHECK == 0 || !power.m_checked
            {
                power.m_found = i32::from(sunVisible(world, pos));
                power.m_checked = true;
            }

            power.m_ticker += 1;
            if power.m_found == 0
            {
                return false;
            }

            power.m_storage += 1;
            true
        }

        Kind::Wind =>
        {
            windPower(world, pos, power);
            true
        }
    }
}

// TileEntityWaterGenerator.flowPower: each water block around adds a hundredth of an EU a tick
fn flowPower(world: World, pos: BlockPos, power: &mut Power)
{
    let ticker: i32 = power.m_ticker;
    power.m_ticker += 1;

    if ticker % fuel::RECHECK == 0 || !power.m_checked
    {
        let water: [Option<i32>; 2] = [ids::block("minecraft:water"), ids::block("minecraft:flowing_water")];
        power.m_found = countAround(world, pos, [-1; 3], [2; 3], |block| water.contains(&Some(block)));
        power.m_checked = true;
    }

    if power.m_found > 0
    {
        power.m_micro += power.m_found;
        if power.m_micro >= fuel::MICRO
        {
            power.m_micro -= fuel::MICRO;
            power.m_storage += 1;
        }
    }
}

// TileEntityWindGenerator.gainFuel: height above the blocks around it times the wind
fn windPower(world: World, pos: BlockPos, power: &mut Power)
{
    const SHARE: f64 = 2.0 / 3.0;
    const SCALE: f64 = 550.0;

    power.m_ticker += 1;
    let recheck: bool = power.m_ticker % fuel::RECHECK == 0;

    if recheck || !power.m_checked
    {
        if power.m_ticker % (fuel::WIND_RECOUNT * fuel::RECHECK) != 0 || !power.m_checked
        {
            power.m_found = countAround(world, pos, [-4, -2, -4], [5, 5, 5], |block| block != 0) - 1;
            power.m_checked = true;
        }

        power.m_sub = SHARE * f64::from(wind()) * f64::from(pos.y - power.m_found) / SCALE;
        power.m_production = power.m_sub as i32;
    }

    power.m_subStorage += power.m_sub;
    while power.m_subStorage >= 1.0
    {
        power.m_subStorage -= 1.0;
        power.m_storage += 1;
    }
}

fn production(source: &Source, power: &Power) -> i32
{
    if source.m_kind == Kind::Wind
    {
        power.m_production
    }
    else
    {
        source.m_production
    }
}

fn converting(source: &Source, power: &Power) -> bool
{
    match source.m_kind
    {
        Kind::Fuel => power.m_fuel > 0,
        _ => power.m_fuel > 0 && power.m_storage + production(source, power) <= source.m_capacity,
    }
}

fn needsFuel(source: &Source, power: &Power) -> bool
{
    match source.m_kind
    {
        Kind::Geothermal => power.m_fuel <= fuel::MAX_LAVA,
        Kind::Water => power.m_fuel <= fuel::MAX_WATER,
        Kind::Solar | Kind::Wind => true,
        Kind::Fuel => power.m_fuel <= 0 && power.m_storage + production(source, power) <= source.m_capacity,
    }
}

// TileEntityBaseGenerator.updateEntity
fn generate(source: &Source, world: World, pos: BlockPos, power: &mut Power)
{
    let was: bool = converting(source, power);
    if was
    {
        power.m_fuel -= 1;
    }

    if needsFuel(source, power)
    {
        gainFuel(source, world, pos, power);
    }

    if was
    {
        power.m_storage += production(source, power);
        power.m_sound += 1;
        if !source.m_loop.is_empty() && power.m_sound % source.m_loopTicks == 0
        {
            world.playSound(dir::center(pos), &sound(source.m_loop), 1.0, 1.0);
        }
    }

    if power.m_storage > 0
    {
        match world.tileSlot(pos, slot::CHARGE).filter(|item| electric::chargeable(item) && item.damage != 1)
        {
            Some(mut item) =>
            {
                power.m_storage -= electric::give(&mut item, power.m_storage, 1);
                stacks::set(world, pos, slot::CHARGE, Some(item));
            }

            None =>
            {
                let output: i32 = production(source, power).min(power.m_storage);
                power.m_storage -= output;
                if output > 0
                {
                    power.m_storage += net::emit(world, pos, output);
                }
            }
        }
    }

    power.m_storage = power.m_storage.min(source.m_capacity);
    setActive(world, pos, converting(source, power));
}

struct SourceTile
{
    m_source: &'static Source,
}

#[tile_behavior]
impl TileBehavior for SourceTile
{
    fn onTick(&self, ctx: &mut BlockCtx)
    {
        if !ctx.isAuthority()
        {
            return;
        }

        let before: Power = ctx.tileData();
        let mut power: Power = before;
        generate(self.m_source, ctx.world(), ctx.pos(), &mut power);

        if power != before
        {
            let _ = ctx.setTileData(&power);
        }
    }
}

// TileEntityBaseGenerator.emitsEnergyTo every side
impl EnergyTile for Source
{
    fn emits(&self, _world: World, _pos: BlockPos, _face: i32) -> bool { true }
}

// GuiGenerator and its siblings
struct SourceScreen
{
    m_source: &'static Source,
    m_pos: Option<BlockPos>,
}

mod gauge
{
    use super::Bar;

    pub const FLAME: Bar = Bar { m_at: [66, 36], m_uv: [176, 0], m_size: [14, 12] };
    pub const STORAGE: Bar = Bar { m_at: [94, 35], m_uv: [176, 14], m_size: [24, 17] };
    pub const WATER: Bar = Bar { m_at: [80, 36], m_uv: [176, 0], m_size: [14, 14] };
    pub const SUN: Bar = Bar { m_at: [80, 45], m_uv: [176, 0], m_size: [14, 14] };
    pub const WIND: Bar = Bar { m_at: [80, 45], m_uv: [176, 0], m_size: [14, 14] };
    pub const FLAME_EXTRA: i32 = 2;
    pub const WIND_FULL: f64 = 3.0;
}

impl Screen for SourceScreen
{
    fn init(&mut self, _width: i32, _height: i32, pos: Option<BlockPos>, _data: &[u8]) { self.m_pos = pos; }

    fn draw(&mut self, draw: &mut Draw, _mouse: [i32; 2], _partial: f32)
    {
        const TITLE_X: [i32; 5] = [58, 52, 55, 55, 55];

        let source: &Source = self.m_source;
        let index: usize = SOURCES.iter().position(|row| row.m_name == source.m_name).unwrap_or(0);
        screen::labels(draw, &screen::name(source.m_name), TITLE_X[index]);

        let power: Power = screen::state(self.m_pos).unwrap_or_default();
        let sheet: &str = sheet(source.m_kind);

        match source.m_kind
        {
            Kind::Fuel =>
            {
                if power.m_fuel > 0
                {
                    let fuelTime: i32 = if power.m_itemFuel > 0
                    {
                        power.m_itemFuel
                    }
                    else
                    {
                        power.m_fuel
                    };
                    let level: i32 = (power.m_fuel * gauge::FLAME.m_size[1] / fuelTime).min(gauge::FLAME.m_size[1]);
                    screen::rising(draw, sheet, &gauge::FLAME, level, gauge::FLAME_EXTRA);
                }

                let stored: i32 = power.m_storage * gauge::STORAGE.m_size[0] / source.m_capacity;
                screen::across(draw, sheet, &gauge::STORAGE, stored);
            }

            Kind::Geothermal if power.m_fuel > 0 =>
            {
                screen::across(draw, sheet, &gauge::STORAGE, power.m_fuel * gauge::STORAGE.m_size[0] / fuel::MAX_LAVA);
            }

            Kind::Water if power.m_fuel > 0 =>
            {
                screen::rising(draw, sheet, &gauge::WATER, power.m_fuel * gauge::WATER.m_size[1] / fuel::MAX_WATER, 0);
            }

            Kind::Solar if power.m_found > 0 => screen::across(draw, sheet, &gauge::SUN, gauge::SUN.m_size[0]),
            Kind::Wind =>
            {
                let level: i32 = (f64::from(gauge::WIND.m_size[1]) * power.m_sub / gauge::WIND_FULL) as i32;
                screen::rising(draw, sheet, &gauge::WIND, level, 0);
            }

            _ =>
            {}
        }
    }
}

fn sheet(kind: Kind) -> &'static str
{
    match kind
    {
        Kind::Fuel => "generator",
        Kind::Geothermal => "geogenerator",
        Kind::Water => "watergenerator",
        Kind::Solar => "solargenerator",
        Kind::Wind => "windgenerator",
    }
}

fn registerScreen(ctx: &mut RegisterCtx, source: &'static Source) -> Result<()>
{
    // ContainerBaseGenerator, ContainerWaterGenerator, ContainerSolarGenerator and ContainerWindGenerator
    const FUEL_LEFT: [[i32; 2]; 2] = [[65, 17], [65, 53]];
    const FUEL_MIDDLE: [[i32; 2]; 2] = [[80, 17], [80, 53]];
    const SINGLE: [i32; 2] = [80, 26];

    let mut builder: ScreenBuilder = screen::container(sheet(source.m_kind));
    builder = match source.m_kind
    {
        Kind::Fuel | Kind::Geothermal => builder.slot(slot::CHARGE, FUEL_LEFT[0][0], FUEL_LEFT[0][1]).slot(
            slot::FUEL,
            FUEL_LEFT[1][0],
            FUEL_LEFT[1][1],
        ),
        Kind::Water => builder.slot(slot::CHARGE, FUEL_MIDDLE[0][0], FUEL_MIDDLE[0][1]).slot(
            slot::FUEL,
            FUEL_MIDDLE[1][0],
            FUEL_MIDDLE[1][1],
        ),
        Kind::Solar | Kind::Wind => builder.slot(slot::CHARGE, SINGLE[0], SINGLE[1]),
    };
    ctx.screens().register(source.m_name, builder, SourceScreen { m_source: source, m_pos: None })
}

// what each generator burns or needs as recipe viewers show it, fuel ticks at its EU a tick
fn registerPages(ctx: &mut RegisterCtx, source: &Source) -> Result<()>
{
    const WIND_NOTES: [&str; 2] = ["About wind x (height - blocks around) / 825 EU/t", "Wind drifts between 10 and 20"];

    let name: &str = source.m_name;
    let rate: i32 = source.m_production;
    ctx.machines().register(name, None, &[name])?;

    let fuelPage = |ctx: &mut RegisterCtx, item: &str, ticks: i32| -> Result<()>
    {
        ctx.machines()
            .recipe(name)
            .input(ItemStack::of(item, 1).withDamage(-1))
            .note(&format!("{} EU at {rate} EU/t", ticks * rate))
            .build()
    };

    match source.m_kind
    {
        Kind::Fuel =>
        {
            for (item, burn) in super::processing::fuels()
            {
                fuelPage(ctx, item, burn / 2)?;
            }

            Ok(())
        }

        Kind::Geothermal =>
        {
            fuelPage(ctx, "minecraft:lava_bucket", fuel::LAVA)?;
            fuelPage(ctx, "ic2:lava_cell", fuel::LAVA)
        }

        Kind::Water =>
        {
            fuelPage(ctx, "minecraft:water_bucket", fuel::WATER)?;
            ctx.machines().recipe(name).note("Water blocks around it add power when it has no bucket").build()
        }

        Kind::Solar => ctx.machines().recipe(name).note(&format!("{rate} EU/t in daylight under open sky")).build(),
        Kind::Wind => WIND_NOTES.iter().fold(ctx.machines().recipe(name), |page, note| page.note(note)).build(),
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for source in &SOURCES
    {
        registerPages(ctx, source)?;

        let Some(machine) = MACHINES.iter().find(|row| row.m_name == source.m_name)
        else
        {
            continue;
        };

        registerBlock(ctx, machine, Rc::new(SourceTile { m_source: source }), true)?;
        registerScreen(ctx, source)?;
        energy::add(own(source.m_name), source);
    }

    ctx.events().on::<TickPost>(Priority::Normal, |_|
    {
        const EVERY: i64 = 128;

        let ticks: i64 = TICKS.with(|ticks|
        {
            ticks.set(ticks.get() + 1);
            ticks.get()
        });
        if ticks % EVERY == 0
        {
            updateWind(World::OVERWORLD);
        }
    })
}
