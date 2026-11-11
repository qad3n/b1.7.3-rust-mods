use super::{Faces, Look, Machine, registerBlock};
use crate::core::{dir, ids, itemTexture, own, sound, stacks};
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
    m_passes: &[],
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

// ItemTFBPFlatification.loadRemoveables
const REMOVABLE: [&str; 20] = [
    "minecraft:snow_layer",
    "minecraft:ice",
    "minecraft:grass",
    "minecraft:stone",
    "minecraft:gravel",
    "minecraft:sand",
    "minecraft:dirt",
    "ic2:rubber_leaves",
    "ic2:rubber_sapling",
    "ic2:rubber_wood",
    "minecraft:leaves",
    "minecraft:log",
    "minecraft:tallgrass",
    "minecraft:red_flower",
    "minecraft:yellow_flower",
    "minecraft:sapling",
    "minecraft:wheat",
    "minecraft:red_mushroom",
    "minecraft:brown_mushroom",
    "minecraft:pumpkin",
];

fn id(name: &str) -> i32 { ids::block(name).unwrap_or(-1) }

fn at(x: i32, y: i32, z: i32) -> BlockPos { BlockPos::new(x, y, z) }

fn set(world: World, pos: BlockPos, name: &str, meta: i32)
{
    let block: i32 = if name.is_empty() { 0 } else { id(name) };
    if let Err(error) = world.setBlock(pos, block, meta)
    {
        warn(&format!("the terraformer could not change {pos:?}: {error}"));
    }
}

// TileEntityTerra.getFirstSolidBlockFrom and getFirstBlockFrom
fn firstFrom(world: World, x: i32, z: i32, mut y: i32, solid: bool) -> i32
{
    while y > 0
    {
        let block: i32 = world.block(at(x, y, z));
        let found: bool = if solid
        {
            block != 0 && World::blockInfo(block).opaque
        }
        else
        {
            block != 0
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
        while world.block(at(x, y - 1, z)) == id(from) && id(from) > 0
        {
            y -= 1;
        }

        if start == y
        {
            return false;
        }

        set(world, at(x, y, z), to, 0);
        return true;
    }

    while world.block(at(x, y, z)) == id(to) && id(to) > 0
    {
        y -= 1;
    }

    if world.block(at(x, y, z)) != id(from) || id(from) <= 0
    {
        return false;
    }

    set(world, at(x, y, z), to, 0);
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

    let block: i32 = world.block(at(x, y, z));
    if block == id("minecraft:dirt")
    {
        set(world, at(x, y, z), "minecraft:grass", 0);
        return true;
    }

    if block != id("minecraft:grass")
    {
        return false;
    }

    let above: i32 = world.block(at(x, y + 1, z));
    if above != 0 && !(above == id("minecraft:tallgrass") && world.random(GRASS_REPLACE) == 0)
    {
        return false;
    }

    let plant: &str = randomPlant(world);
    if plant == "minecraft:wheat"
    {
        set(world, at(x, y, z), "minecraft:farmland", 0);
    }

    set(world, at(x, y + 1, z), plant, i32::from(plant == "minecraft:tallgrass"));
    true
}

fn spreadGrass(world: World, x: i32, y: i32, z: i32) -> bool
{
    if world.random(2) == 0
    {
        return false;
    }

    let y: i32 = firstFrom(world, x, z, y, false);
    let block: i32 = world.block(at(x, y, z));

    if block == id("minecraft:dirt")
    {
        set(world, at(x, y, z), "minecraft:grass", 0);
        return true;
    }

    if block == id("minecraft:grass")
    {
        set(world, at(x, y + 1, z), "minecraft:tallgrass", 1);
        return true;
    }

    false
}

fn leaves(world: World, pos: BlockPos, meta: i32)
{
    if world.isAir(pos)
    {
        set(world, pos, "minecraft:leaves", meta);
    }
}

fn irrigate(world: World, x: i32, z: i32, top: i32) -> bool
{
    const RAIN_CHANCE: i32 = 48000;
    const RAIN_TICKS: i32 = 12000;
    const REACH: i32 = 10;
    const RIPE: i32 = 7;

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
    let block: i32 = world.block(pos);

    if block == id("minecraft:tallgrass")
    {
        return spreadGrass(world, x + 1, y, z)
            || spreadGrass(world, x - 1, y, z)
            || spreadGrass(world, x, y, z + 1)
            || spreadGrass(world, x, y, z - 1);
    }

    if block == id("minecraft:sapling")
    {
        let _ = world.fertilize(pos);
        return true;
    }

    if block == id("ic2:rubber_sapling")
    {
        crate::world::tree::grow(world, pos);
        return true;
    }

    if block == id("minecraft:log")
    {
        let meta: i32 = world.meta(pos);
        set(world, at(x, y + 1, z), "minecraft:log", meta);
        for spot in
            [at(x, y + 2, z), at(x + 1, y + 1, z), at(x - 1, y + 1, z), at(x, y + 1, z + 1), at(x, y + 1, z - 1)]
        {
            leaves(world, spot, meta);
        }

        return true;
    }

    if block == id("minecraft:wheat")
    {
        let _ = world.setMeta(pos, RIPE);
        return true;
    }

    if block == id("minecraft:fire")
    {
        set(world, pos, "", 0);
        return true;
    }

    false
}

fn water(world: World, pos: BlockPos) -> bool
{
    let block: i32 = world.block(pos);
    block == id("minecraft:water") || block == id("minecraft:flowing_water")
}

// BlockSnow.canPlaceBlockAt: an opaque solid block below
fn snowFits(world: World, pos: BlockPos) -> bool
{
    let below: i32 = world.block(pos.offset(0, -1, 0));
    world.isAir(pos) && below != 0 && World::blockInfo(below).opaque && World::blockInfo(below).solid
}

fn snowHere(world: World, x: i32, y: i32, z: i32) -> bool
{
    const REACH: i32 = 16;

    let top: i32 = firstFrom(world, x, z, y + REACH, false);
    if y > top
    {
        return false;
    }

    let block: i32 = world.block(at(x, top, z));
    if block == id("minecraft:snow_layer") || block == id("minecraft:snow")
    {
        return true;
    }

    if snowFits(world, at(x, top + 1, z)) || block == id("minecraft:ice")
    {
        set(world, at(x, top + 1, z), "minecraft:snow_layer", 0);
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
        set(world, pos, "minecraft:ice", 0);
        return true;
    }

    let block: i32 = world.block(pos);
    if block == id("minecraft:ice") && water(world, at(x, y - 1, z))
    {
        set(world, at(x, y - 1, z), "minecraft:ice", 0);
        return true;
    }

    let surrounded: bool = snowHere(world, x + 1, y, z)
        && snowHere(world, x - 1, y, z)
        && snowHere(world, x, y, z + 1)
        && snowHere(world, x, y, z - 1);
    if block == id("minecraft:snow_layer") && surrounded
    {
        set(world, pos, "minecraft:snow", 0);
        return true;
    }

    if snowFits(world, at(x, y + 1, z)) || block == id("minecraft:ice")
    {
        set(world, at(x, y + 1, z), "minecraft:snow_layer", 0);
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
        || ["minecraft:snow_layer", "minecraft:leaves", "ic2:rubber_leaves"]
            .iter()
            .chain(PLANTS.iter())
            .any(|name| block == id(name));

    if cleared
    {
        set(world, pos, "", 0);
        return true;
    }

    if block == id("minecraft:ice") || block == id("minecraft:snow")
    {
        set(world, pos, "minecraft:flowing_water", 0);
        return true;
    }

    let wooden: bool = ["minecraft:planks", "minecraft:log", "ic2:rubber_wood"].iter().any(|name| block == id(name));
    if wooden && world.random(BURN) == 0
    {
        set(world, pos, "minecraft:fire", 0);
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

    if world.block(at(x, y, z)) == id("minecraft:snow_layer")
    {
        y -= 1;
    }

    if y == top
    {
        return false;
    }

    if y < top
    {
        set(world, at(x, y + 1, z), "minecraft:dirt", 0);
        return true;
    }

    let block: i32 = world.block(at(x, y, z));
    if !REMOVABLE.iter().any(|name| block == id(name))
    {
        return false;
    }

    set(world, at(x, y, z), "", 0);
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

pub(crate) fn isTerraformer(world: World, pos: BlockPos) -> bool
{
    ids::block(&own(MACHINE.m_name)) == Some(world.block(pos))
}

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
    energy::add(own(MACHINE.m_name), &TERRAFORMER);

    for row in &BLUEPRINTS
    {
        ctx.items().register(
            row.m_item,
            ItemSpecBuilder::new().texture(&itemTexture(row.m_item)).maxStack(1).behavior(BlueprintItem),
        )?;
    }

    Ok(())
}
