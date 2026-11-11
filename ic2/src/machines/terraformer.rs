use super::{Faces, Look, Machine, registerBlock};
use crate::core::{dir, itemTexture, own, sound, stacks};
use crate::energy::{self, EnergyTile, net};
use b173_api::prelude::*;
use std::rc::Rc;

pub(crate) static MACHINE: Machine = Machine
{
    m_name: "terraformer",
    m_look: Look
    {
        m_idle: ["machine_bottom", "advanced_machine", "terraformer_side", "terraformer_side"],
        m_active: ["machine_bottom", "advanced_machine", "terraformer_side", "terraformer_side"],
        m_faces: Faces::Machine,
    },
    m_hardness: 2.0,
    m_screen: false,
    m_wrench: 1.0,
    m_drop: "advanced_machine",
    m_passes: None,
    m_slots: 1,
};

mod terra
{
    pub const BLUEPRINT: i32 = 0;
    pub const CAPACITY: i32 = 100000;
    pub const INPUT: i32 = 512;
    pub const HUM_EVERY: i32 = 64;
    pub const MAX_FAILS: i32 = 4;
    pub const SPREAD_STEPS: i32 = 5;
    pub const NEAR_DIVISOR: i32 = 10;
    pub const FAIL_DIVISOR: i32 = 10;
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Plan
{
    Cultivation,
    Irrigation,
    Chilling,
    Desertification,
    Flatification,
}

// ITerraformingBP: energy per change and reach
struct Blueprint
{
    m_item: &'static str,
    m_plan: Plan,
    m_consume: i32,
    m_range: i32,
}

const BLUEPRINTS: [Blueprint; 5] = [
    Blueprint { m_item: "tfbp_cultivation", m_plan: Plan::Cultivation, m_consume: 4000, m_range: 40 },
    Blueprint { m_item: "tfbp_irrigation", m_plan: Plan::Irrigation, m_consume: 3000, m_range: 60 },
    Blueprint { m_item: "tfbp_chilling", m_plan: Plan::Chilling, m_consume: 2000, m_range: 50 },
    Blueprint { m_item: "tfbp_desertification", m_plan: Plan::Desertification, m_consume: 2500, m_range: 40 },
    Blueprint { m_item: "tfbp_flatification", m_plan: Plan::Flatification, m_consume: 4000, m_range: 40 },
];

// ItemTFBPCultivation.loadPlants in its order
const PLANTS: [&str; 9] = [
    "minecraft:tallgrass",
    "minecraft:red_flower",
    "minecraft:yellow_flower",
    "minecraft:sapling",
    "minecraft:wheat",
    "minecraft:red_mushroom",
    "minecraft:brown_mushroom",
    "ic2:rubber_sapling",
    "minecraft:pumpkin",
];

// the plants cultivation puts down, desertification clears, ItemTFBPCultivation's list as a tag
const PLANT_TAG: &str = "ic2:terraformer_plants";
// ItemTFBPFlatification.loadRemoveables
const REMOVABLE: &str = "ic2:flattenable";
// vanilla blocks without named states report their metadata under this property
const META: &str = "meta";

fn at(x: i32, y: i32, z: i32) -> BlockPos { BlockPos::new(x, y, z) }

fn setState(world: World, pos: BlockPos, state: StateData)
{
    if let Err(error) = world.setBlockState(pos, state)
    {
        warn(&format!("the terraformer could not change {pos:?}: {error}"));
    }
}

fn set(world: World, pos: BlockPos, name: &str)
{
    if let Err(error) = world.place(pos, name)
    {
        warn(&format!("the terraformer could not change {pos:?}: {error}"));
    }
}

// a vanilla block with the metadata IC2 gave it, tall grass type or crop age
fn placeVariant(world: World, pos: BlockPos, name: &str, meta: &str)
{
    match Identifier::parse(name)
    {
        Ok(block) => setState(world, pos, StateData::of(block).with(META, meta)),
        Err(error) => warn(&format!("the terraformer names no block {name}: {error}")),
    }
}

fn remove(world: World, pos: BlockPos)
{
    if let Err(error) = world.clear(pos)
    {
        warn(&format!("the terraformer could not clear {pos:?}: {error}"));
    }
}

// TileEntityTerra.getFirstSolidBlockFrom and getFirstBlockFrom
fn firstFrom(world: World, x: i32, z: i32, mut y: i32, solid: bool) -> i32
{
    while y > 0
    {
        let pos: BlockPos = at(x, y, z);
        let found: bool = if solid
        {
            World::blockInfo(world.block(pos)).opaque
        }
        else
        {
            !world.isAir(pos)
        };
        if found
        {
            return y;
        }

        y -= 1;
    }

    -1
}

// TileEntityTerra.switchGround
fn switchGround(world: World, from: &str, to: &str, x: i32, y: i32, z: i32, upwards: bool) -> bool
{
    let mut y: i32 = y;

    if upwards
    {
        y += 1;
        let start: i32 = y;
        while world.is(at(x, y - 1, z), from)
        {
            y -= 1;
        }

        if start == y
        {
            return false;
        }

        set(world, at(x, y, z), to);
        return true;
    }

    while world.is(at(x, y, z), to)
    {
        y -= 1;
    }

    if !world.is(at(x, y, z), from)
    {
        return false;
    }

    set(world, at(x, y, z), to);
    true
}

// ItemTFBPCultivation.pickRandomPlantId
fn randomPlant(world: World) -> &'static str
{
    const PICK: i32 = 5;

    PLANTS.iter().find(|_| world.random(PICK) <= 1).copied().unwrap_or(PLANTS[0])
}

fn cultivate(world: World, x: i32, z: i32, top: i32) -> bool
{
    const REACH: i32 = 10;
    const GRASS_REPLACE: i32 = 4;

    let y: i32 = firstFrom(world, x, z, top + REACH, true);
    if y == -1
    {
        return false;
    }

    if switchGround(world, "minecraft:sand", "minecraft:dirt", x, y, z, true)
    {
        return true;
    }

    let ground: BlockPos = at(x, y, z);
    if world.is(ground, "minecraft:dirt")
    {
        set(world, ground, "minecraft:grass");
        return true;
    }

    if !world.is(ground, "minecraft:grass")
    {
        return false;
    }

    let above: BlockPos = at(x, y + 1, z);
    if !world.isAir(above) && !(world.is(above, "minecraft:tallgrass") && world.random(GRASS_REPLACE) == 0)
    {
        return false;
    }

    let plant: &str = randomPlant(world);
    if plant == "minecraft:wheat"
    {
        set(world, ground, "minecraft:farmland");
    }

    match plant
    {
        "minecraft:tallgrass" => placeVariant(world, above, plant, TALL_GRASS),
        _ => set(world, above, plant),
    }

    true
}

// BlockTallGrass metadata 1, the plain grass tuft
const TALL_GRASS: &str = "1";

fn spreadGrass(world: World, x: i32, y: i32, z: i32) -> bool
{
    if world.random(2) == 0
    {
        return false;
    }

    let y: i32 = firstFrom(world, x, z, y, false);
    let ground: BlockPos = at(x, y, z);

    if world.is(ground, "minecraft:dirt")
    {
        set(world, ground, "minecraft:grass");
        return true;
    }

    if world.is(ground, "minecraft:grass")
    {
        placeVariant(world, at(x, y + 1, z), "minecraft:tallgrass", TALL_GRASS);
        return true;
    }

    false
}

// leaves of the log's wood type, which both keep in their metadata
fn leaves(world: World, pos: BlockPos, wood: &str)
{
    if world.isAir(pos)
    {
        placeVariant(world, pos, "minecraft:leaves", wood);
    }
}

fn irrigate(world: World, x: i32, z: i32, top: i32) -> bool
{
    const RAIN_CHANCE: i32 = 48000;
    const RAIN_TICKS: i32 = 12000;
    const REACH: i32 = 10;
    const RIPE: &str = "7";

    if world.random(RAIN_CHANCE) == 0
    {
        let _ = world.setWeather(true, false, RAIN_TICKS);
        return true;
    }

    let y: i32 = firstFrom(world, x, z, top + REACH, false);
    if y == -1
    {
        return false;
    }

    if switchGround(world, "minecraft:sand", "minecraft:dirt", x, y, z, true)
    {
        switchGround(world, "minecraft:sand", "minecraft:dirt", x, y, z, true);
        return true;
    }

    let pos: BlockPos = at(x, y, z);

    if world.is(pos, "minecraft:tallgrass")
    {
        return spreadGrass(world, x + 1, y, z)
            || spreadGrass(world, x - 1, y, z)
            || spreadGrass(world, x, y, z + 1)
            || spreadGrass(world, x, y, z - 1);
    }

    if world.is(pos, "minecraft:sapling")
    {
        let _ = world.fertilize(pos);
        return true;
    }

    if world.is(pos, "ic2:rubber_sapling")
    {
        crate::world::tree::grow(world, pos);
        return true;
    }

    if world.is(pos, "minecraft:log")
        && let Some(log) = world.stateAt(pos)
    {
        let wood: &str = log.get(META).unwrap_or("0");
        setState(world, at(x, y + 1, z), (*log).clone());
        for spot in
            [at(x, y + 2, z), at(x + 1, y + 1, z), at(x - 1, y + 1, z), at(x, y + 1, z + 1), at(x, y + 1, z - 1)]
        {
            leaves(world, spot, wood);
        }

        return true;
    }

    if world.is(pos, "minecraft:wheat")
    {
        placeVariant(world, pos, "minecraft:wheat", RIPE);
        return true;
    }

    if world.is(pos, "minecraft:fire")
    {
        remove(world, pos);
        return true;
    }

    false
}

fn water(world: World, pos: BlockPos) -> bool { world.tagged(pos, "minecraft:water") }

// BlockSnow.canPlaceBlockAt: an opaque solid block below
fn snowFits(world: World, pos: BlockPos) -> bool
{
    let below: BlockInfo = World::blockInfo(world.block(pos.offset(0, -1, 0)));
    world.isAir(pos) && below.opaque && below.solid
}

fn snowHere(world: World, x: i32, y: i32, z: i32) -> bool
{
    const REACH: i32 = 16;

    let top: i32 = firstFrom(world, x, z, y + REACH, false);
    if y > top
    {
        return false;
    }

    let surface: BlockPos = at(x, top, z);
    if world.tagged(surface, "minecraft:snow")
    {
        return true;
    }

    if snowFits(world, at(x, top + 1, z)) || world.tagged(surface, "minecraft:ice")
    {
        set(world, at(x, top + 1, z), "minecraft:snow_layer");
    }

    false
}

fn chill(world: World, x: i32, z: i32, top: i32) -> bool
{
    const REACH: i32 = 10;

    let y: i32 = firstFrom(world, x, z, top + REACH, false);
    if y == -1
    {
        return false;
    }

    let pos: BlockPos = at(x, y, z);
    if water(world, pos)
    {
        set(world, pos, "minecraft:ice");
        return true;
    }

    let ice: bool = world.tagged(pos, "minecraft:ice");
    if ice && water(world, at(x, y - 1, z))
    {
        set(world, at(x, y - 1, z), "minecraft:ice");
        return true;
    }

    let surrounded: bool = snowHere(world, x + 1, y, z)
        && snowHere(world, x - 1, y, z)
        && snowHere(world, x, y, z + 1)
        && snowHere(world, x, y, z - 1);
    if world.is(pos, "minecraft:snow_layer") && surrounded
    {
        set(world, pos, "minecraft:snow");
        return true;
    }

    if snowFits(world, at(x, y + 1, z)) || ice
    {
        set(world, at(x, y + 1, z), "minecraft:snow_layer");
    }

    false
}

fn desertify(world: World, x: i32, z: i32, top: i32) -> bool
{
    const REACH: i32 = 10;
    const BURN: i32 = 15;

    let y: i32 = firstFrom(world, x, z, top + REACH, false);
    if y == -1
    {
        return false;
    }

    let switched: bool = switchGround(world, "minecraft:dirt", "minecraft:sand", x, y, z, false)
        || switchGround(world, "minecraft:grass", "minecraft:sand", x, y, z, false)
        || switchGround(world, "minecraft:farmland", "minecraft:sand", x, y, z, false);
    if switched
    {
        switchGround(world, "minecraft:dirt", "minecraft:sand", x, y, z, false);
        return true;
    }

    let pos: BlockPos = at(x, y, z);
    let block: i32 = world.block(pos);
    let cleared: bool = water(world, pos)
        || blockIs(block, "minecraft:snow_layer")
        || blockTagged(block, "minecraft:leaves")
        || blockTagged(block, PLANT_TAG);

    if cleared
    {
        remove(world, pos);
        return true;
    }

    if blockTagged(block, "minecraft:ice") || blockIs(block, "minecraft:snow")
    {
        set(world, pos, "minecraft:flowing_water");
        return true;
    }

    let wooden: bool = blockTagged(block, "minecraft:planks") || blockTagged(block, "minecraft:logs");
    if wooden && world.random(BURN) == 0
    {
        set(world, pos, "minecraft:fire");
        return true;
    }

    false
}

fn flatten(world: World, x: i32, z: i32, top: i32) -> bool
{
    const REACH: i32 = 20;

    let mut y: i32 = firstFrom(world, x, z, top + REACH, false);
    if y == -1
    {
        return false;
    }

    if world.is(at(x, y, z), "minecraft:snow_layer")
    {
        y -= 1;
    }

    if y == top
    {
        return false;
    }

    if y < top
    {
        set(world, at(x, y + 1, z), "minecraft:dirt");
        return true;
    }

    if !world.tagged(at(x, y, z), REMOVABLE)
    {
        return false;
    }

    remove(world, at(x, y, z));
    true
}

fn terraform(plan: Plan, world: World, x: i32, z: i32, top: i32) -> bool
{
    match plan
    {
        Plan::Cultivation => cultivate(world, x, z, top),
        Plan::Irrigation => irrigate(world, x, z, top),
        Plan::Chilling => chill(world, x, z, top),
        Plan::Desertification => desertify(world, x, z, top),
        Plan::Flatification => flatten(world, x, z, top),
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Terra
{
    pub(crate) m_energy: i32,
    m_fails: i32,
    m_last: [i32; 3],
    m_sound: i32,
}

impl Default for Terra
{
    fn default() -> Self { Terra { m_energy: 0, m_fails: 0, m_last: [-1; 3], m_sound: 0 } }
}

fn blueprint(world: World, pos: BlockPos) -> Option<&'static Blueprint>
{
    let stack: ItemStack = world.tileSlot(pos, terra::BLUEPRINT)?;
    BLUEPRINTS.iter().find(|row| stack.item.is(&own(row.m_item)))
}

fn spread(world: World, range: i32) -> i32 { world.random(range + 1) - world.random(range + 1) }

// TileEntityTerra.updateEntity: a change somewhere near the last one, or anywhere in reach after a miss
fn terraTick(world: World, pos: BlockPos, state: &mut Terra)
{
    let Some(plan) = blueprint(world, pos)
    else
    {
        return;
    };

    if state.m_energy < plan.m_consume
    {
        return;
    }

    state.m_sound += 1;
    if state.m_sound % terra::HUM_EVERY == 0
    {
        world.playSound(dir::center(pos), &sound("machineTerraLoop"), 1.0, 1.0);
    }

    let (x, z): (i32, i32) = if state.m_last[1] > -1
    {
        let range: i32 = plan.m_range / terra::NEAR_DIVISOR;
        let x: i32 = state.m_last[0] - spread(world, range);
        (x, state.m_last[2] - spread(world, range))
    }
    else
    {
        state.m_fails = state.m_fails.min(terra::MAX_FAILS);
        let range: i32 = plan.m_range * (state.m_fails + 1) / terra::SPREAD_STEPS;
        let x: i32 = pos.x - spread(world, range);
        (x, pos.z - spread(world, range))
    };

    if terraform(plan.m_plan, world, x, z, pos.y)
    {
        state.m_energy -= plan.m_consume;
        state.m_fails = 0;
        state.m_last = [x, pos.y, z];
    }
    else
    {
        state.m_energy -= plan.m_consume / terra::FAIL_DIVISOR;
        state.m_fails += 1;
        state.m_last[1] = -1;
    }
}

// TileEntityTerra.ejectBlueprint, through the wrench
pub(crate) fn eject(world: World, pos: BlockPos) -> bool
{
    let Some(stack) = world.tileSlot(pos, terra::BLUEPRINT)
    else
    {
        return false;
    };

    stacks::drop(world, pos, &stack);
    stacks::set(world, pos, terra::BLUEPRINT, None);
    true
}

pub(crate) fn isTerraformer(world: World, pos: BlockPos) -> bool { world.is(pos, MACHINE.m_name) }

struct Terraformer;

impl EnergyTile for Terraformer
{
    fn accepts(&self, _world: World, _pos: BlockPos, _face: i32) -> bool { true }

    fn sink(&self) -> bool { true }

    fn demands(&self, world: World, pos: BlockPos) -> bool
    {
        world.tileData::<Terra>(pos).m_energy < terra::CAPACITY - terra::INPUT
    }

    fn inject(&self, world: World, pos: BlockPos, _face: i32, amount: i32) -> i32
    {
        if amount > terra::INPUT
        {
            net::overload(world, pos);
            return 0;
        }

        let mut state: Terra = world.tileData(pos);
        state.m_energy += amount;
        let left: i32 = (state.m_energy - terra::CAPACITY).max(0);
        state.m_energy -= left;
        let _ = world.setTileData(pos, &state);
        left
    }
}

static TERRAFORMER: Terraformer = Terraformer;

struct TerraTile;

#[tile_behavior]
impl TileBehavior for TerraTile
{
    fn onTick(&self, ctx: &mut BlockCtx)
    {
        if !ctx.isAuthority()
        {
            return;
        }

        let before: Terra = ctx.tileData();
        let mut state: Terra = before;
        terraTick(ctx.world(), ctx.pos(), &mut state);

        if state != before
        {
            let _ = ctx.setTileData(&state);
        }
    }

    fn energyDemand(&self, ctx: &mut BlockCtx, _face: i32) -> Option<EnergyDemand>
    {
        TERRAFORMER.demands(ctx.world(), ctx.pos()).then(|| energy::demand(terra::INPUT)).flatten()
    }

    fn receiveEnergy(&self, ctx: &mut BlockCtx, _face: i32, amount: i64, simulate: bool) -> i64
    {
        let mut state: Terra = ctx.tileData();
        let taken: i32 = (amount.clamp(0, i64::from(terra::INPUT)) as i32).min(terra::CAPACITY - state.m_energy).max(0);

        if !simulate
        {
            state.m_energy += taken;
            let _ = ctx.setTileData(&state);
        }

        i64::from(taken)
    }
}

// ItemTFBP.onItemUse: the blueprint goes into the terraformer, swapping out the one inside
struct BlueprintItem;

#[item_behavior]
impl ItemBehavior for BlueprintItem
{
    fn onUseOnBlock(&self, world: World, _player: Player, stack: ItemStack, pos: BlockPos, _face: i32) -> Use
    {
        if !isTerraformer(world, pos)
        {
            return Use::Pass;
        }

        if world.isAuthority()
        {
            eject(world, pos);
            stacks::set(world, pos, terra::BLUEPRINT, Some(stack));
        }

        Use::Done(None)
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    registerBlock(ctx, &MACHINE, Rc::new(TerraTile), false)?;
    energy::add(MACHINE.m_name, &TERRAFORMER);

    for row in &BLUEPRINTS
    {
        ctx.items().register(
            row.m_item,
            ItemSpecBuilder::new().texture(&itemTexture(row.m_item)).maxStack(1).behavior(BlueprintItem),
        )?;
    }

    Ok(())
}
