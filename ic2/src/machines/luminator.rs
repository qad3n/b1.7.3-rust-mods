use crate::core::{blockTexture, dir};
use crate::energy::{self, Conductor, EnergyTile, net};
use crate::explosives;
use b173_api::prelude::*;
use std::rc::Rc;

const NAME: &str = "luminator";

mod lamp
{
    pub const HARDNESS: f32 = 0.6;
    pub const RESISTANCE: f32 = 0.5;
    // TileEntityLuminator ticks its work every 20 ticks and pays for its light every 160
    pub const WORK_EVERY: i32 = 20;
    pub const PAY_EVERY: i32 = 160;
    pub const LOW_COST: i32 = 5;
    pub const HIGH_COST: i32 = 40;
    pub const MODES: i32 = 3;
    pub const CAPACITY: [i32; 3] = [10, 20, 80];
    pub const INPUT: i32 = 32;
    pub const POOF: f32 = 0.5;
    pub const BURN_REACH: f64 = 3.0;
    pub const OPEN_REACH: i32 = 3;
    pub const FIRE_STEP: f64 = 30.0;
    pub const FIRE_MAX: f64 = 100.0;
}

// TileEntityLuminator as a conductor: no loss, and it bursts on 33 EU
static LAMP_CONDUCTOR: Conductor = Conductor { m_loss: 0.0, m_absorption: 32, m_breakdown: 33 };

// the hostile mobs, EntityMob and its kinds in beta
const MOBS: &str = "ic2:monsters";

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) struct Lamp
{
    m_energy: i32,
    m_mode: i32,
    m_powered: bool,
    m_ticker: i32,
}

fn poof(world: World, pos: BlockPos)
{
    let _ = world.clear(pos);
    explosives::blast(world, dir::center(pos), lamp::POOF, explosives::LUMINATOR);
    net::changed(world);
}

// the reach of burnMobs on one axis: three blocks toward open air or glass, one block otherwise
fn reach(world: World, pos: BlockPos, face: i32, crossOpen: bool) -> i32
{
    let at: BlockPos = dir::step(pos, face);
    let open: bool = world.isAir(at) || world.tagged(at, "c:glass_blocks") || world.is(at, "reinforced_glass");
    match (open, crossOpen)
    {
        (true, _) => lamp::OPEN_REACH,
        (false, true) => 1,
        _ => 0,
    }
}

fn open(world: World, pos: BlockPos, face: i32) -> bool { reach(world, pos, face, false) > 0 }

// TileEntityLuminator.burnMobs: monsters in its light catch fire
fn burnMobs(world: World, pos: BlockPos)
{
    let side = |faces: [i32; 4]| faces.iter().any(|face| open(world, pos, *face));
    let px: i32 = reach(world, pos, dir::EAST, side([dir::UP, dir::DOWN, dir::SOUTH, dir::NORTH]));
    let nx: i32 = reach(world, pos, dir::WEST, side([dir::UP, dir::DOWN, dir::SOUTH, dir::NORTH]));
    let py: i32 = reach(world, pos, dir::UP, side([dir::EAST, dir::WEST, dir::SOUTH, dir::NORTH]));
    let ny: i32 = reach(world, pos, dir::DOWN, side([dir::EAST, dir::WEST, dir::SOUTH, dir::NORTH]));
    let pz: i32 = reach(world, pos, dir::SOUTH, side([dir::UP, dir::DOWN, dir::EAST, dir::WEST]));
    let nz: i32 = reach(world, pos, dir::NORTH, side([dir::UP, dir::DOWN, dir::EAST, dir::WEST]));
    let low: [f64; 3] = [f64::from(pos.x - nx), f64::from(pos.y - ny), f64::from(pos.z - nz)];
    let high: [f64; 3] = [f64::from(pos.x + px + 1), f64::from(pos.y + py + 2), f64::from(pos.z + pz + 1)];
    let center: [f64; 3] = [pos.x as f64, pos.y as f64, pos.z as f64];
    let near: [f64; 3] = center.map(|value| value - lamp::BURN_REACH);
    let far: [f64; 3] = center.map(|value| value + lamp::BURN_REACH);

    let mobs: Rc<std::collections::HashSet<Identifier>> = tagMembers(RegistryKind::Entity, MOBS);

    for mob in world.entitiesIn(near, far)
    {
        let hostile: bool = mob.kind.as_ref().is_some_and(|kind| mobs.contains(kind));
        let inside: bool = (0..3).all(|axis| mob.pos[axis] >= low[axis] && mob.pos[axis] <= high[axis]);
        if !hostile || !inside
        {
            continue;
        }

        let entity: Entity = Entity { world, id: mob.id };
        let fire: f64 = entity.field(EntityField::FireTicks);
        if fire < lamp::FIRE_MAX
        {
            entity.setField(EntityField::FireTicks, (fire + lamp::FIRE_STEP).min(lamp::FIRE_MAX));
        }
    }
}

fn tick(world: World, pos: BlockPos, state: &mut Lamp)
{
    state.m_ticker += 1;
    if state.m_ticker % lamp::WORK_EVERY != 0
    {
        return;
    }

    if state.m_ticker % lamp::PAY_EVERY == 0
    {
        // modes one and two both fall through to the 40 EU cost
        let cost: i32 = if state.m_mode == 0
        {
            lamp::LOW_COST
        }
        else
        {
            lamp::HIGH_COST
        };
        if cost > state.m_energy
        {
            state.m_energy = 0;
            state.m_powered = false;
        }
        else
        {
            state.m_energy -= cost;
            state.m_powered = true;
        }
    }

    if state.m_powered
    {
        burnMobs(world, pos);
    }
}

struct Luminator;

impl EnergyTile for Luminator
{
    fn accepts(&self, _world: World, _pos: BlockPos, _face: i32) -> bool { true }

    fn emits(&self, _world: World, _pos: BlockPos, _face: i32) -> bool { true }

    fn conductor(&self, _world: World, _pos: BlockPos) -> Option<&'static Conductor> { Some(&LAMP_CONDUCTOR) }

    fn sink(&self) -> bool { true }

    fn demands(&self, world: World, pos: BlockPos) -> bool
    {
        let lamp: Lamp = world.tileData(pos);
        lamp.m_energy < lamp::CAPACITY[lamp.m_mode.rem_euclid(lamp::MODES) as usize]
    }

    fn inject(&self, world: World, pos: BlockPos, _face: i32, amount: i32) -> i32
    {
        if amount > lamp::INPUT
        {
            poof(world, pos);
            return 0;
        }

        let mut lamp: Lamp = world.tileData(pos);
        let capacity: i32 = lamp::CAPACITY[lamp.m_mode.rem_euclid(lamp::MODES) as usize];
        lamp.m_energy += amount;
        let left: i32 = (lamp.m_energy - capacity).max(0);
        lamp.m_energy -= left;
        let _ = world.setTileData(pos, &lamp);
        left
    }

    // TileEntityLuminator.canCableConnectFrom: only the cable behind the face its facing names
    fn joins(&self, world: World, pos: BlockPos, from: BlockPos) -> bool
    {
        dir::step(pos, dir::reverse(super::facingAt(world, pos))) == from
    }
}

static LUMINATOR: Luminator = Luminator;

struct LampTile;

#[tile_behavior]
impl TileBehavior for LampTile
{
    fn onTick(&self, ctx: &mut BlockCtx)
    {
        if !ctx.isAuthority()
        {
            return;
        }

        let before: Lamp = ctx.tileData();
        let mut lamp: Lamp = before;
        tick(ctx.world(), ctx.pos(), &mut lamp);

        if lamp != before
        {
            let _ = ctx.setTileData(&lamp);
        }
    }
}

struct LampBlock;

#[block_behavior]
impl BlockBehavior for LampBlock
{
    // TileEntityLuminator.switchStrength
    fn onActivated(&self, ctx: &mut BlockCtx, _player: Player) -> bool
    {
        if ctx.isAuthority()
        {
            let mut lamp: Lamp = ctx.tileData();
            lamp.m_mode = (lamp.m_mode + 1) % lamp::MODES;
            let _ = ctx.setTileData(&lamp);
        }

        true
    }

    fn onAdded(&self, ctx: &mut BlockCtx) { net::changed(ctx.world()); }

    fn onRemoved(&self, ctx: &mut BlockCtx) { net::changed(ctx.world()); }
}

pub(crate) fn isLuminator(world: World, pos: BlockPos) -> bool { world.is(pos, NAME) }

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.tileEntities().register(
        NAME,
        TileEntitySpec { inventory: 0, ticks: true, sync: false, behavior: None },
        Some(Rc::new(LampTile)),
    )?;
    ctx.blocks().register(
        NAME,
        BlockSpecBuilder::template("minecraft:block")
            .material("glass")
            .sound("glass")
            .hardness(lamp::HARDNESS)
            .resistance(lamp::RESISTANCE)
            .texture(&blockTexture(NAME))
            .opacity(0)
            .tileEntity(NAME)
            .drops(Vec::new())
            .behavior(LampBlock),
    )?;
    // BlockLuminator metadata, the side it faces, which only the item damage sets
    ctx.blocks().states(
        NAME,
        BlockStatesSpec
        {
            properties: vec![PropertySpec::new(super::state::FACING, &dir::NAMES)],
            ..BlockStatesSpec::default()
        },
    )?;
    energy::add(NAME, &LUMINATOR);
    Ok(())
}
