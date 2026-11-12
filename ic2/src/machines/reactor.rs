use super::{Faces, Look, Machine, registerBlock, screen, setActive};
use crate::core::{blockTexture, dir, random, sound, stacks};
use crate::energy::{self, EnergyTile, net};
use crate::explosives;
use b173_api::prelude::*;

pub(crate) static MACHINE: Machine = Machine
{
    m_name: "nuclear_reactor",
    m_look: Look
    {
        m_idle: ["machine_bottom", "nuclear_reactor_top", "nuclear_reactor_side", "nuclear_reactor_side"],
        m_active: [
            "machine_bottom",
            "nuclear_reactor_top",
            "nuclear_reactor_side_active",
            "nuclear_reactor_side_active",
        ],
        m_faces: Faces::Machine,
    },
    m_hardness: 3.0,
    m_screen: false,
    m_wrench: 0.8,
    m_drop: "generator",
    m_passes: None,
    m_slots: grid::SLOTS,
};

const CHAMBER: &str = "ic2:reactor_chamber";

mod grid
{
    pub const SLOTS: i32 = 54;
    pub const COLUMNS: i32 = 9;
    pub const ROWS: i32 = 6;
    pub const BASE: i32 = 3;
    // ItemGradual.setMaxDamage, every part that wears in a reactor
    pub const LIFE: i32 = 10000;
}

mod heat
{
    pub const TICK_RATE: i32 = 20;
    pub const PULSE: i32 = 10;
    pub const DANGER: i32 = 4000;
    pub const HULL: i32 = 10000;
    pub const HULL_PER_COLUMN: i32 = 1000;
    pub const PLATING: i32 = 100;
    pub const BLAST_BASE: i32 = 10;
    pub const BLAST_PER_CELL: i32 = 5;
    pub const BLAST_MAX: i32 = 160;
    pub const GLOW: i32 = 1000;
    pub const GEIGER_STEP: i32 = 2000;
    pub const HUM_EVERY: i32 = 64;
    pub const WATER_COOLING: i32 = 500;
    pub const LAVA_HEAT: i32 = 2000;
    pub const ICE_COOLING: i32 = 200;
    pub const SWITCH_MAX: i32 = 6;
    pub const DISPERSE_MAX: i32 = 25;
    pub const PLATING_COOL_CHANCE: i32 = 10;
    pub const LAST_LIFE: i32 = 9999;
    pub const SPENT_CHANCE: i32 = 3;
    pub const REACH: i32 = 2;
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) struct Core
{
    pub(crate) m_heat: i32,
    pub(crate) m_output: i32,
    m_ticker: i32,
    m_sound: i32,
}

// the reactor grid, read once per update and written back as it changes; the dice are the world's random
struct Grid
{
    m_size: i32,
    m_slots: Vec<Option<ItemStack>>,
    m_dice: Box<dyn FnMut(i32) -> i32>,
}

impl Grid
{
    fn load(world: World, pos: BlockPos) -> Grid
    {
        let slots: Vec<Option<ItemStack>> = (0..grid::SLOTS).map(|slot| world.tileSlot(pos, slot)).collect();
        Grid { m_size: size(world, pos), m_slots: slots, m_dice: Box::new(move |bound| world.random(bound)) }
    }

    fn save(&self, world: World, pos: BlockPos, before: &[Option<ItemStack>])
    {
        for (slot, stack) in self.m_slots.iter().enumerate()
        {
            if before[slot] != *stack
            {
                stacks::set(world, pos, slot as i32, stack.clone());
            }
        }
    }

    fn roll(&mut self, bound: i32) -> i32 { (self.m_dice)(bound) }

    // getMatrixCoord, empty outside the nine by six grid
    fn at(&self, x: i32, y: i32) -> Option<&ItemStack>
    {
        let inside: bool = (0..grid::COLUMNS).contains(&x) && (0..grid::ROWS).contains(&y);
        if !inside
        {
            return None;
        }

        self.m_slots[(x + y * grid::COLUMNS) as usize].as_ref()
    }

    fn set(&mut self, x: i32, y: i32, stack: Option<ItemStack>)
    {
        if (0..grid::COLUMNS).contains(&x) && (0..grid::ROWS).contains(&y)
        {
            self.m_slots[(x + y * grid::COLUMNS) as usize] = stack;
        }
    }

    fn is(&self, x: i32, y: i32, item: &str) -> bool { self.at(x, y).is_some_and(|stack| stack.item.is(item)) }

    fn damage(&self, x: i32, y: i32) -> i32 { self.at(x, y).map_or(0, |stack| stack.damage) }

    fn setDamage(&mut self, x: i32, y: i32, damage: i32)
    {
        if let Some(stack) = self.at(x, y).cloned()
        {
            self.set(x, y, Some(ItemStack { damage, ..stack }));
        }
    }

    // ItemStack.damageItem on an ItemGradual part: worn past its ten thousand points it breaks
    fn wear(&mut self, x: i32, y: i32, amount: i32)
    {
        if let Some(stack) = self.at(x, y).cloned()
        {
            let damage: i32 = stack.damage + amount;
            self.set(x, y, (damage <= grid::LIFE).then_some(ItemStack { damage, ..stack }));
        }
    }
}

// TileEntityNuclearReactor.getReactorSize: three columns and one more per chamber around it
pub(crate) fn size(world: World, pos: BlockPos) -> i32
{
    let mut size: i32 = grid::BASE;

    for dx in -1..=1
    {
        for dy in -1..=1
        {
            for dz in -1..=1
            {
                if world.is(pos.offset(dx, dy, dz), CHAMBER)
                {
                    size += 1;
                }
            }
        }
    }

    size
}

// TileEntityNuclearReactor.isUsefulItem
const USEFUL: &str = "ic2:reactor_components";

// dropAllUnfittingStuff: what sits outside the grid's width or does nothing in a reactor falls out
fn dropUnfitting(grid: &mut Grid) -> Vec<ItemStack>
{
    let mut ejected: Vec<ItemStack> = Vec::new();

    for x in 0..grid::COLUMNS
    {
        for y in 0..grid::ROWS
        {
            let Some(stack) = grid.at(x, y).cloned()
            else
            {
                continue;
            };

            if stack.count <= 0
            {
                grid.set(x, y, None);
                continue;
            }

            if x >= grid.m_size || !stack.item.tagged(USEFUL)
            {
                ejected.push(stack);
                grid.set(x, y, None);
            }
        }
    }

    ejected
}

// coolReactorFromOutside: chambers and water cool it, air helps a little, fire and lava heat it
fn outsideCooling(world: World, pos: BlockPos) -> i32
{
    const CHAMBER_COOLING: i32 = 2;
    const LAVA_HEATING: i32 = 3;
    const FIRE_HEATING: i32 = 2;
    const AIR_SHARE: i32 = 4;

    let mut cooling: i32 = 1;
    let mut air: i32 = 0;

    for dx in -1..=1
    {
        for dy in -1..=1
        {
            for dz in -1..=1
            {
                let at: BlockPos = pos.offset(dx, dy, dz);
                let block: i32 = world.block(at);
                if blockIs(block, CHAMBER)
                {
                    cooling += CHAMBER_COOLING;
                }

                if blockTagged(block, "minecraft:water")
                {
                    cooling += 1;
                }

                if world.isAir(at)
                {
                    air += 1;
                }

                if blockTagged(block, "minecraft:fire")
                {
                    air -= FIRE_HEATING;
                }

                if blockTagged(block, "minecraft:lava")
                {
                    cooling -= LAVA_HEATING;
                }
            }
        }
    }

    (cooling + air / AIR_SHARE).max(0)
}

fn randomAround(world: World, pos: BlockPos) -> Option<BlockPos>
{
    let span: i32 = 2 * heat::REACH + 1;
    let at: BlockPos = pos.offset(
        world.random(span) - heat::REACH,
        world.random(span) - heat::REACH,
        world.random(span) - heat::REACH,
    );
    (at != pos).then_some(at)
}

// Material.rock, iron, lava, ground and clay melt into lava, the rest catches fire
fn melts(block: i32) -> bool { blockTagged(block, "ic2:meltable") }

// Material.wood, leaves and cloth
fn burns(block: i32) -> bool { blockTagged(block, "ic2:burnable") }

fn place(world: World, pos: BlockPos, name: &str)
{
    if let Err(error) = world.place(pos, name)
    {
        warn(&format!("the reactor could not set {name} at {pos:?}: {error}"));
    }
}

// calculateHeatEffects: true when the reactor blew up
fn heatEffects(world: World, pos: BlockPos, grid: &Grid, core: &Core) -> bool
{
    const BOIL: f32 = 0.5;
    const IGNITE: f32 = 0.4;
    const IGNITE_SCALE: f32 = 1.5;
    const BURN: f32 = 0.7;
    const MELT: f32 = 0.85;
    const MELT_SCALE: f32 = 4.0;
    const HURT_LOW: f64 = 3.0;
    const HURT_HIGH: f64 = 4.0;

    if core.m_heat < heat::DANGER
    {
        return false;
    }

    let platings: i32 = (0..grid::ROWS)
        .flat_map(|y| (0..grid.m_size).map(move |x| (x, y)))
        .filter(|(x, y)| grid.is(*x, *y, "ic2:reactor_plating"))
        .count() as i32;
    let hull: i32 = heat::HULL + heat::HULL_PER_COLUMN * (grid.m_size - grid::BASE) + heat::PLATING * platings;
    let ratio: f32 = core.m_heat as f32 / hull as f32;

    if ratio >= 1.0
    {
        let cells: i32 = (0..grid::ROWS)
            .flat_map(|y| (0..grid.m_size).map(move |x| (x, y)))
            .filter(|(x, y)| grid.is(*x, *y, "ic2:uranium_cell"))
            .count() as i32;
        let power: i32 = (heat::BLAST_BASE + heat::BLAST_PER_CELL * cells - platings).min(heat::BLAST_MAX);
        let _ = world.clear(pos);
        explosives::blast(world, [pos.x as f64, pos.y as f64, pos.z as f64], power as f32, explosives::MATTER);
        net::changed(world);
        return true;
    }

    if ratio >= MELT
        && (random::float(world) as f32) <= MELT_SCALE * (ratio - BURN)
        && let Some(at) = randomAround(world, pos)
    {
        let block: i32 = world.block(at);
        if world.isAir(at)
        {
            place(world, at, "minecraft:fire");
        }
        else if melts(block)
        {
            place(world, at, "minecraft:flowing_lava");
        }
        else if !blockIs(block, "minecraft:bedrock")
        {
            place(world, at, "minecraft:fire");
        }
    }

    if ratio >= BURN
    {
        let low: [f64; 3] = [pos.x as f64 - HURT_LOW, pos.y as f64 - HURT_LOW, pos.z as f64 - HURT_LOW];
        let high: [f64; 3] = [pos.x as f64 + HURT_HIGH, pos.y as f64 + HURT_HIGH, pos.z as f64 + HURT_HIGH];
        for living in world.entitiesIn(low, high).into_iter().filter(|info| info.health.is_some())
        {
            Entity { world, id: living.id }.damage(1, None);
        }
    }

    if ratio >= BOIL
        && let Some(at) = randomAround(world, pos)
        && world.tagged(at, "minecraft:water")
    {
        let _ = world.clear(at);
    }

    if ratio >= IGNITE
        && (random::float(world) as f32) <= IGNITE_SCALE * (ratio - IGNITE)
        && let Some(at) = randomAround(world, pos)
        && burns(world.block(at))
    {
        place(world, at, "minecraft:fire");
    }

    false
}

fn takesHeat(grid: &Grid, x: i32, y: i32, plating: bool, disperser: bool) -> i32
{
    let coolant: bool = grid.is(x, y, "ic2:coolant_cell");
    let plate: bool = plating && grid.is(x, y, "ic2:reactor_plating");
    let spreader: bool = disperser && grid.is(x, y, "ic2:heat_disperser");
    i32::from(coolant || plate || spreader)
}

const AROUND: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
// spreadHeat hands heat on in this order
const SPREAD: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

fn spreadHeatTo(grid: &mut Grid, x: i32, y: i32, amount: i32, deep: bool)
{
    if takesHeat(grid, x, y, deep, false) == 0
    {
        return;
    }

    if grid.is(x, y, "ic2:reactor_plating") && deep
    {
        spreadHeat(grid, x, y, amount, false);
        return;
    }

    grid.wear(x, y, amount);
}

// spreadHeat: plating shares its heat evenly with the parts that can hold it
fn spreadHeat(grid: &mut Grid, x: i32, y: i32, amount: i32, deep: bool)
{
    let takers: i32 = AROUND.iter().map(|(dx, dy)| takesHeat(grid, x + dx, y + dy, deep, false)).sum();
    if takers == 0
    {
        grid.wear(x, y, amount);
        return;
    }

    let mut amount: i32 = amount;
    while amount % takers != 0 && grid.damage(x, y) > 0
    {
        amount += 1;
        let damage: i32 = grid.damage(x, y);
        grid.setDamage(x, y, damage - 1);
    }

    let share: i32 = amount / takers;
    amount -= share * takers;
    if amount > 0
    {
        grid.wear(x, y, amount);
    }

    for (dx, dy) in SPREAD
    {
        spreadHeatTo(grid, x + dx, y + dy, share, deep);
    }
}

fn giveHeat(grid: &mut Grid, x: i32, y: i32, amount: i32)
{
    if takesHeat(grid, x, y, true, true) == 0
    {
        return;
    }

    if grid.is(x, y, "ic2:reactor_plating")
    {
        spreadHeat(grid, x, y, amount, true);
        return;
    }

    grid.wear(x, y, amount);
}

// enrichDepleted: a depleted cell next to a working one may take two points back, faster when hot
fn enrich(grid: &mut Grid, core: &Core, x: i32, y: i32) -> i32
{
    const ODDS: [(i32, i32); 3] = [(9000, 2), (6000, 4), (3000, 8)];
    const COLD_ODDS: i32 = 16;
    const STEP: i32 = 2;

    if !grid.is(x, y, "ic2:depleted_isotope_cell")
    {
        return 0;
    }

    let odds: i32 = ODDS.iter().find(|(above, _)| core.m_heat >= *above).map_or(COLD_ODDS, |(_, odds)| *odds);
    if grid.roll(odds) != 0
    {
        return 1;
    }

    let damage: i32 = grid.damage(x, y);
    if damage <= 0
    {
        grid.set(x, y, Some(ItemStack::of("ic2:re_enriched_uranium_cell", 1)));
    }
    else
    {
        grid.setDamage(x, y, damage - STEP);
    }

    1
}

// generateEnergy: a uranium cell pulses once plus once per neighbouring cell, each pulse heating its surroundings
fn generate(grid: &mut Grid, core: &mut Core, x: i32, y: i32)
{
    let neighbours: i32 = AROUND.iter().filter(|(dx, dy)| grid.is(x + dx, y + dy, "ic2:uranium_cell")).count() as i32;
    let pulses: i32 = 1 + neighbours;
    core.m_output += pulses * heat::PULSE;

    let enriched: i32 = AROUND.iter().map(|(dx, dy)| enrich(grid, core, x + dx, y + dy)).sum();
    for _ in 0..pulses + enriched
    {
        let takers: i32 = AROUND.iter().map(|(dx, dy)| takesHeat(grid, x + dx, y + dy, true, true)).sum();
        let pulse: i32 = match takers
        {
            2 => 4,
            3 => 2,
            4 => 1,
            _ => 10,
        };

        if takers == 0
        {
            core.m_heat += pulse;
            continue;
        }

        for (dx, dy) in AROUND
        {
            giveHeat(grid, x + dx, y + dy, pulse);
        }
    }

    if grid.damage(x, y) == heat::LAST_LIFE && grid.roll(heat::SPENT_CHANCE) == 0
    {
        grid.set(x, y, Some(ItemStack::of("ic2:near_depleted_uranium_cell", 1)));
    }
    else
    {
        grid.wear(x, y, 1);
    }
}

// switchHeat: a disperser trades heat with neighbouring coolant and plating, six points at most
fn switchHeat(grid: &mut Grid, x: i32, y: i32, ox: i32, oy: i32)
{
    let coolant: bool = grid.is(ox, oy, "ic2:coolant_cell");
    if !coolant && !grid.is(ox, oy, "ic2:reactor_plating")
    {
        return;
    }

    let mine: i32 = grid.damage(x, y);
    let theirs: i32 = grid.damage(ox, oy);
    let flow: i32 = (mine - theirs) / 2;

    if flow > 0
    {
        let flow: i32 = flow.min(heat::SWITCH_MAX);
        grid.setDamage(x, y, mine - flow);
        if coolant
        {
            grid.wear(ox, oy, flow);
        }
        else
        {
            spreadHeat(grid, ox, oy, flow, false);
        }

        return;
    }

    let flow: i32 = (-flow).min(heat::SWITCH_MAX);
    grid.wear(x, y, flow);
    grid.setDamage(ox, oy, theirs - flow);
}

// disperseHeat: the disperser evens out with its neighbours and then with the hull
fn disperse(grid: &mut Grid, core: &mut Core, x: i32, y: i32)
{
    for (dx, dy) in SPREAD
    {
        switchHeat(grid, x, y, x + dx, y + dy);
    }

    let flow: i32 = (grid.damage(x, y) - core.m_heat) / 2;
    if flow > 0
    {
        let flow: i32 = flow.min(heat::DISPERSE_MAX);
        core.m_heat += flow;
        let damage: i32 = grid.damage(x, y);
        grid.setDamage(x, y, damage - flow);
        return;
    }

    let flow: i32 = (-flow).min(heat::DISPERSE_MAX);
    core.m_heat -= flow;
    grid.wear(x, y, flow);
}

fn swapItem(grid: &mut Grid, x: i32, y: i32, item: &str)
{
    if let Some(stack) = grid.at(x, y).cloned()
    {
        grid.set(x, y, Some(ItemStack { item: ItemRef::from(item), ..stack }));
    }
}

// processChamber for one grid slot
fn chamber(grid: &mut Grid, core: &mut Core, x: i32, y: i32, producing: bool)
{
    let Some(stack) = grid.at(x, y).cloned()
    else
    {
        return;
    };
    let name: String = stack.item.name().map(|id| id.to_string()).unwrap_or_default();

    match name.as_str()
    {
        "ic2:coolant_cell" if stack.damage > 0 => grid.setDamage(x, y, stack.damage - 1),
        "ic2:reactor_plating" if stack.damage > 0 && grid.roll(heat::PLATING_COOL_CHANCE) == 0 =>
        {
            grid.setDamage(x, y, stack.damage - 1);
        }
        "ic2:near_depleted_uranium_cell" | "ic2:depleted_isotope_cell" | "ic2:re_enriched_uranium_cell" =>
        {
            core.m_heat += 1
        }
        "minecraft:water_bucket" if core.m_heat > heat::DANGER =>
        {
            core.m_heat -= heat::WATER_COOLING;
            swapItem(grid, x, y, "minecraft:bucket");
        }
        "minecraft:lava_bucket" =>
        {
            core.m_heat += heat::LAVA_HEAT;
            swapItem(grid, x, y, "minecraft:bucket");
        }
        "minecraft:ice" if core.m_heat > heat::ICE_COOLING =>
        {
            core.m_heat -= heat::ICE_COOLING;
            grid.set(x, y, stacks::less(stack, 1));
        }
        "ic2:heat_disperser" => disperse(grid, core, x, y),
        "ic2:uranium_cell" if producing => generate(grid, core, x, y),
        _ =>
        {}
    }
}

// processChambers: every slot of the reactor's width, row by row
fn processChambers(grid: &mut Grid, core: &mut Core, producing: bool)
{
    for y in 0..grid::ROWS
    {
        for x in 0..grid.m_size
        {
            chamber(grid, core, x, y, producing);
        }
    }
}

// TileEntityNuclearReactor.sendEnergy: what the net does not take goes out through the chambers
fn send(world: World, pos: BlockPos, amount: i32)
{
    let mut left: i32 = net::emit(world, pos, amount);

    for face in [dir::UP, dir::DOWN, dir::EAST, dir::WEST, dir::SOUTH, dir::NORTH]
    {
        let at: BlockPos = dir::step(pos, face);
        if left > 0 && world.is(at, CHAMBER)
        {
            left = net::emit(world, at, left);
        }
    }
}

fn geiger(level: i32) -> Option<&'static str>
{
    match level
    {
        level if level < 1 => None,
        1 => Some("generatorNuclearGeiger"),
        2 => Some("generatorNuclearGeigerOne"),
        _ => Some("generatorNuclearGeigerTwo"),
    }
}

// TileEntityNuclearReactor.updateEntity
fn tick(world: World, pos: BlockPos, core: &mut Core)
{
    send(world, pos, core.m_output);

    core.m_ticker += 1;
    if core.m_ticker < heat::TICK_RATE
    {
        return;
    }

    core.m_ticker = 0;
    let mut grid: Grid = Grid::load(world, pos);
    let before: Vec<Option<ItemStack>> = grid.m_slots.clone();
    for stack in dropUnfitting(&mut grid)
    {
        stacks::drop(world, pos, &stack);
    }

    if core.m_heat > 0
    {
        core.m_heat -= outsideCooling(world, pos);
        if core.m_heat <= 0
        {
            core.m_heat = 0;
        }
        else if heatEffects(world, pos, &grid, core)
        {
            return;
        }
    }

    core.m_output = 0;
    core.m_sound += 1;
    if core.m_sound % heat::HUM_EVERY == 0
    {
        world.playSound(dir::center(pos), &sound("generatorNuclearLoop"), 1.0, 1.0);
        if let Some(click) = geiger(core.m_heat / heat::GEIGER_STEP - 1)
        {
            world.playSound(dir::center(pos), &sound(click), 1.0, 1.0);
        }
    }

    processChambers(&mut grid, core, !world.isPowered(pos));

    grid.save(world, pos, &before);
    setActive(world, pos, core.m_heat >= heat::GLOW || core.m_output > 0);
}

struct Reactor;

// TileEntityNuclearReactor and TileEntityReactorChamber both emit to every side
impl EnergyTile for Reactor
{
    fn emits(&self, _world: World, _pos: BlockPos, _face: i32) -> bool { true }
}

static REACTOR: Reactor = Reactor;

struct ReactorTile;

#[tile_behavior]
impl TileBehavior for ReactorTile
{
    fn onTick(&self, ctx: &mut BlockCtx)
    {
        if !ctx.isAuthority()
        {
            return;
        }

        let before: Core = ctx.tileData();
        let mut core: Core = before;
        tick(ctx.world(), ctx.pos(), &mut core);

        if core != before && ctx.world().block(ctx.pos()) != 0
        {
            let _ = ctx.setTileData(&core);
        }
    }
}

// the reactor next to a chamber, BlockReactorChamber.getReactorEntity
fn reactorBeside(world: World, pos: BlockPos) -> Option<BlockPos>
{
    [dir::EAST, dir::WEST, dir::UP, dir::DOWN, dir::SOUTH, dir::NORTH]
        .iter()
        .map(|face| dir::step(pos, *face))
        .find(|at| world.is(*at, "nuclear_reactor"))
}

fn reactorsBeside(world: World, pos: BlockPos) -> usize
{
    dir::all().filter(|face| world.is(dir::step(pos, *face), "nuclear_reactor")).count()
}

pub(crate) fn openScreen(world: World, pos: BlockPos, player: Player)
{
    let screen: String = format!("nuclear_reactor_{}", size(world, pos));
    if let Err(error) = player.openScreen(&screen, Some(pos), Vec::new())
    {
        warn(&format!("could not open the reactor screen: {error}"));
    }
}

// BlockReactorChamber: touches exactly one reactor, opens its screen and smokes with its heat
struct ChamberBlock;

#[block_behavior]
impl BlockBehavior for ChamberBlock
{
    fn canPlaceAt(&self, ctx: &mut BlockCtx) -> bool { reactorsBeside(ctx.world(), ctx.pos()) == 1 }

    fn canStay(&self, ctx: &mut BlockCtx) -> bool { reactorsBeside(ctx.world(), ctx.pos()) == 1 }

    fn onActivated(&self, ctx: &mut BlockCtx, player: Player) -> bool
    {
        let Some(reactor) = reactorBeside(ctx.world(), ctx.pos())
        else
        {
            return false;
        };

        if ctx.isAuthority()
        {
            openScreen(ctx.world(), reactor, player);
        }

        true
    }

    fn onRandomDisplayTick(&self, ctx: &mut BlockCtx)
    {
        if let Some(reactor) = reactorBeside(ctx.world(), ctx.pos())
        {
            smoke(ctx.world(), ctx.pos(), reactor);
        }
    }

    fn onAdded(&self, ctx: &mut BlockCtx) { net::changed(ctx.world()); }

    fn onRemoved(&self, ctx: &mut BlockCtx) { net::changed(ctx.world()); }
}

// BlockGenerator and BlockReactorChamber.randomDisplayTick: smoke and flames by the reactor's heat
pub(crate) fn smoke(world: World, at: BlockPos, reactor: BlockPos)
{
    const PER_PUFF: i32 = 1000;
    const SMOKE_HEIGHT: f64 = 0.95;
    const FLAME_SPREAD: i32 = 4;
    const FLAME_SKIP: i32 = 3;

    let mut puffs: i32 = world.tileData::<Core>(reactor).m_heat / PER_PUFF;
    if puffs <= 0
    {
        return;
    }

    puffs = world.random(puffs);
    let spot =
        |height: f64| [at.x as f64 + random::float(world), at.y as f64 + height, at.z as f64 + random::float(world)];
    for _ in 0..puffs
    {
        world.particle("smoke", spot(SMOKE_HEIGHT), [0.0; 3]);
    }

    puffs -= world.random(FLAME_SPREAD) + FLAME_SKIP;
    for _ in 0..puffs
    {
        world.particle("flame", spot(1.0), [0.0; 3]);
    }
}

// GuiNuclearReactor: the 6byN sheet for the reactor's width, its title and the grid
struct ReactorScreen;

impl Screen for ReactorScreen
{
    fn draw(&mut self, draw: &mut Draw, _mouse: [i32; 2], _partial: f32)
    {
        const TITLE: [i32; 2] = [55, 6];
        const INVENTORY_LABEL: [i32; 2] = [8, 128];

        screen::text(draw, TITLE[0], TITLE[1], &screen::name("nuclear_reactor"));
        screen::text(draw, INVENTORY_LABEL[0], INVENTORY_LABEL[1], &translate("ic2.gui.inventory"));
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    const HEIGHT: i32 = 222;
    const INVENTORY_Y: i32 = 140;
    const CENTER: i32 = 89;
    const HALF_SLOT: i32 = 9;
    const TOP: i32 = 18;
    const SLOT: i32 = 18;
    const MAX_SIZE: i32 = 9;
    const CHAMBER_HARDNESS: f32 = 3.0;

    registerBlock(ctx, &MACHINE, ReactorTile, true)?;
    energy::add(MACHINE.m_name, &REACTOR);

    // ContainerNuclearReactor lays out the columns the reactor has, centered
    for width in grid::BASE..=MAX_SIZE
    {
        let start: i32 = CENTER - HALF_SLOT * width;
        let mut builder: ScreenBuilder = ScreenBuilder::new("", screen::WIDTH, HEIGHT)
            .background(&screen::texture(&format!("6by{width}")))
            .playerInventory(screen::INVENTORY[0], INVENTORY_Y);

        for slot in 0..grid::SLOTS
        {
            let (x, y): (i32, i32) = (slot % grid::COLUMNS, slot / grid::COLUMNS);
            if x < width
            {
                builder = builder.slot(slot, start + SLOT * x, TOP + SLOT * y);
            }
        }

        ctx.screens().register(&format!("nuclear_reactor_{width}"), builder, ReactorScreen)?;
    }

    ctx.blocks().register(
        "reactor_chamber",
        BlockSpecBuilder::template("minecraft:block")
            .material("iron")
            .sound("metal")
            .hardness(CHAMBER_HARDNESS)
            .faces(FaceTextures
            {
                top: Some(blockTexture("machine_top")),
                bottom: Some(blockTexture("machine_bottom")),
                side: Some(blockTexture("reactor_chamber_side")),
                front: None,
            })
            .tool(ToolClass::Pickaxe, 0)
            .behavior(ChamberBlock),
    )?;
    energy::add(CHAMBER, &REACTOR);
    Ok(())
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn grid(width: i32, parts: &[(i32, i32, &str)], dice: i32) -> Grid
    {
        let mut slots: Vec<Option<ItemStack>> = vec![None; grid::SLOTS as usize];
        for (x, y, item) in parts
        {
            slots[(x + y * grid::COLUMNS) as usize] = Some(ItemStack::of(*item, 1));
        }

        Grid { m_size: width, m_slots: slots, m_dice: Box::new(move |_| dice) }
    }

    fn cycle(grid: &mut Grid, core: &mut Core)
    {
        core.m_output = 0;
        processChambers(grid, core, true);
    }

    #[test]
    fn loneCellPulsesOnceAndHeatsTheHull()
    {
        let mut grid: Grid = grid(3, &[(1, 1, "ic2:uranium_cell")], 1);
        let mut core: Core = Core::default();
        cycle(&mut grid, &mut core);

        assert_eq!(core.m_output, 10);
        assert_eq!(core.m_heat, 10);
        assert_eq!(grid.damage(1, 1), 1);
    }

    #[test]
    fn neighbouringCellsPulseForEachOther()
    {
        let mut grid: Grid = grid(3, &[(0, 0, "ic2:uranium_cell"), (1, 0, "ic2:uranium_cell")], 1);
        let mut core: Core = Core::default();
        cycle(&mut grid, &mut core);

        assert_eq!(core.m_output, 40);
        assert_eq!(core.m_heat, 40);
    }

    #[test]
    fn coolantAroundACellTakesItsHeatInSlotOrder()
    {
        let parts: [(i32, i32, &str); 5] = [
            (1, 0, "ic2:coolant_cell"),
            (0, 1, "ic2:coolant_cell"),
            (1, 1, "ic2:uranium_cell"),
            (2, 1, "ic2:coolant_cell"),
            (1, 2, "ic2:coolant_cell"),
        ];
        let mut grid: Grid = grid(3, &parts, 1);
        let mut core: Core = Core::default();
        cycle(&mut grid, &mut core);

        assert_eq!(core.m_heat, 0);
        assert_eq!(core.m_output, 10);
        // the cells before the uranium cool down before they are heated, the ones after cool straight away
        assert_eq!([grid.damage(1, 0), grid.damage(0, 1), grid.damage(2, 1), grid.damage(1, 2)], [1, 1, 0, 0]);
    }

    #[test]
    fn twoCoolantNeighboursShareFourPointsEach()
    {
        let parts: [(i32, i32, &str); 3] =
            [(0, 1, "ic2:coolant_cell"), (1, 1, "ic2:uranium_cell"), (2, 1, "ic2:coolant_cell")];
        let mut grid: Grid = grid(3, &parts, 1);
        let mut core: Core = Core::default();
        cycle(&mut grid, &mut core);

        assert_eq!(core.m_heat, 0);
        // the left one is processed before the pulse, the right one cools a point after it
        assert_eq!([grid.damage(0, 1), grid.damage(2, 1)], [4, 3]);
    }

    #[test]
    fn aPoweredReactorMakesNoEnergy()
    {
        let mut grid: Grid = grid(3, &[(1, 1, "ic2:uranium_cell")], 1);
        let mut core: Core = Core::default();
        core.m_output = 0;
        processChambers(&mut grid, &mut core, false);

        assert_eq!((core.m_output, core.m_heat, grid.damage(1, 1)), (0, 0, 0));
    }

    #[test]
    fn lavaHeatsAndIceCools()
    {
        let mut grid: Grid = grid(3, &[(0, 0, "minecraft:lava_bucket"), (1, 0, "minecraft:ice")], 1);
        let mut core: Core = Core::default();
        cycle(&mut grid, &mut core);

        assert_eq!(core.m_heat, 2000 - 200);
        assert!(grid.is(0, 0, "minecraft:bucket"));
        assert!(grid.at(1, 0).is_none());
    }

    #[test]
    fn aDepletedCellBesideAWorkingOneIsEnrichedOnALuckyRoll()
    {
        let mut grid: Grid = grid(3, &[(0, 0, "ic2:uranium_cell"), (1, 0, "ic2:depleted_isotope_cell")], 0);
        grid.setDamage(1, 0, 10);
        let mut core: Core = Core::default();
        cycle(&mut grid, &mut core);

        assert_eq!(grid.damage(1, 0), 8);
        // two pulses of ten, one for the cell and one for the enrichment, then the depleted cell's own point
        assert_eq!(core.m_heat, 21);
    }

    #[test]
    fn aSpentCellBreaksPastItsLife()
    {
        let mut grid: Grid = grid(3, &[(0, 0, "ic2:coolant_cell")], 1);
        grid.wear(0, 0, grid::LIFE);
        assert_eq!(grid.damage(0, 0), grid::LIFE);
        grid.wear(0, 0, 1);
        assert!(grid.at(0, 0).is_none());
    }
}
