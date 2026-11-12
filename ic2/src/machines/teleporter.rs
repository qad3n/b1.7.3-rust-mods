use super::{Faces, Look, Machine, registerBlock};
use crate::core::{dir, random};
use crate::energy::net;
use crate::energy::storage::{Bank, anyBankAt, bankEnergy, setBankEnergy};
use b173_api::prelude::*;
use std::collections::BTreeMap;

pub(crate) static MACHINE: Machine = Machine
{
    m_name: "teleporter",
    m_look: Look
    {
        m_idle: ["machine_top", "tesla_coil", "advanced_machine", "advanced_machine"],
        m_active: ["machine_top", "tesla_coil", "advanced_machine", "advanced_machine"],
        m_faces: Faces::Machine,
    },
    m_hardness: 2.0,
    m_screen: false,
    m_wrench: 1.0,
    m_drop: "teleporter",
    m_passes: None,
    m_slots: 0,
};

const REGISTRY: &str = "teleporters";

mod port
{
    pub const FREQUENCIES: i32 = 32000;
    pub const ANIMAL: i32 = 100;
    pub const PLAYER: i32 = 1000;
    pub const OTHER: i32 = 500;
    pub const PER_STACK: i32 = 100;
    pub const PER_ARMOR: i32 = 100;
    pub const MAIN_SLOTS: i32 = 36;
    pub const ARMOR_SLOTS: [i32; 4] = [36, 37, 38, 39];
    pub const IDLE_SPARKS: i32 = 2;
    pub const JUMP_SPARKS: i32 = 20;
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Link
{
    m_target: i32,
    m_own: i32,
    m_registered: bool,
}

impl Default for Link
{
    fn default() -> Self { Link { m_target: -1, m_own: -1, m_registered: false } }
}

// TileEntityTeleporter.teleporters, kept with the world so links outlive a restart
fn registry() -> BTreeMap<i32, [i32; 4]> { worldData(REGISTRY) }

fn setRegistry(map: &BTreeMap<i32, [i32; 4]>) { setWorldData(REGISTRY, map); }

fn place(world: World, pos: BlockPos) -> [i32; 4] { [world.dimension, pos.x, pos.y, pos.z] }

// TileEntityTeleporter.register: false when another teleporter already holds the frequency
fn claim(world: World, pos: BlockPos, frequency: i32) -> bool
{
    let mut map: BTreeMap<i32, [i32; 4]> = registry();
    if map.get(&frequency).is_some_and(|other| *other != place(world, pos))
    {
        return false;
    }

    map.insert(frequency, place(world, pos));
    setRegistry(&map);
    true
}

fn unregister(world: World, pos: BlockPos)
{
    let mut map: BTreeMap<i32, [i32; 4]> = registry();
    map.retain(|_, at| *at != place(world, pos));
    setRegistry(&map);
}

fn sparks(world: World, count: i32, pos: BlockPos)
{
    for _ in 0..count
    {
        for lift in [1.0, 2.0]
        {
            let at: [f64; 3] = [
                pos.x as f64 + random::float(world),
                pos.y as f64 + lift + random::float(world),
                pos.z as f64 + random::float(world),
            ];
            world.particle("reddust", at, [-1.0, 0.0, 1.0]);
        }
    }
}

// TileEntityTeleporter.getWeightOf: animals 100, players 1000 plus their load, the rest 500
fn weight(info: &EntityInfo) -> i32
{
    if info.player.is_some()
    {
        let player: Player = Player { id: info.id };
        let mut weight: i32 = port::PLAYER;

        for slot in 0..port::MAIN_SLOTS
        {
            if let Some(stack) = player.slot(slot)
            {
                weight += port::PER_STACK * stack.count / crate::core::stacks::maxStack(&stack);
            }
        }

        weight +=
            port::ARMOR_SLOTS.iter().filter(|slot| player.slot(**slot).is_some()).count() as i32 * port::PER_ARMOR;
        return weight;
    }

    let animal: bool =
        info.kind.as_ref().is_some_and(|kind| tagMembers(RegistryKind::Entity, "ic2:animals").contains(kind));
    if animal { port::ANIMAL } else { port::OTHER }
}

const SIDES: [i32; 5] = [dir::DOWN, dir::EAST, dir::WEST, dir::SOUTH, dir::NORTH];

// TileEntityTeleporter.consumeEnergy: drawn evenly from the charged storage blocks around it
fn drain(world: World, pos: BlockPos, amount: i32)
{
    let mut amount: i32 = amount;

    loop
    {
        let banks: Vec<Bank> = SIDES
            .iter()
            .filter_map(|face| anyBankAt(world, dir::step(pos, *face)))
            .filter(|bank| bankEnergy(world, bank) > 0)
            .collect();
        let count: i32 = banks.len() as i32;
        if count == 0
        {
            return;
        }

        let share: i32 = amount / count;
        let mut drained: i32 = 0;

        for bank in &banks
        {
            let take: i32 = share.min(bankEnergy(world, bank));
            setBankEnergy(world, bank, bankEnergy(world, bank) - take);
            drained += take;
        }

        amount -= drained;
        if amount <= 0 || amount < count || drained == 0
        {
            return;
        }
    }
}

fn available(world: World, pos: BlockPos) -> i32
{
    SIDES.iter().filter_map(|face| anyBankAt(world, dir::step(pos, *face))).map(|bank| bankEnergy(world, &bank)).sum()
}

// TileEntityTeleporter.teleport: the cost grows with the distance and the weight carried
fn teleport(world: World, pos: BlockPos, info: &EntityInfo, target: [i32; 4])
{
    let delta: [i32; 3] = [target[1] - pos.x, target[2] - pos.y, target[3] - pos.z];
    let distance: f64 = f64::from(delta.iter().map(|value| value * value).sum::<i32>()).sqrt();
    let cost: f64 = f64::from(weight(info)) * distance;

    if cost > f64::from(available(world, pos))
    {
        return;
    }

    drain(world, pos, cost as i32);
    world.playSound(info.pos, "portal.portal", 1.0, 1.0);
    sparks(world, port::JUMP_SPARKS, pos);

    let to: [f64; 3] =
        [info.pos[0] + f64::from(delta[0]), info.pos[1] + f64::from(delta[1]), info.pos[2] + f64::from(delta[2])];
    let moved = match info.player
    {
        Some(_) => Player { id: info.id }.teleport(world.dimension, to),
        None => Entity { world, id: info.id }.setPosition(to),
    };

    if let Err(error) = moved
    {
        warn(&format!("the teleporter at {pos:?} could not move entity {}: {error}", info.id));
        return;
    }

    sparks(world, port::JUMP_SPARKS, pos.offset(delta[0], delta[1], delta[2]));
    world.playSound(to, "portal.portal", 1.0, 1.0);
}

fn tick(world: World, pos: BlockPos, link: &mut Link)
{
    if link.m_own < 0
    {
        link.m_own = world.random(port::FREQUENCIES);
    }

    if !link.m_registered
    {
        if !claim(world, pos, link.m_own)
        {
            net::overload(world, pos);
            return;
        }

        link.m_registered = true;
        return;
    }

    if link.m_target == -1 || !world.isPowered(pos)
    {
        return;
    }

    sparks(world, port::IDLE_SPARKS, pos);
    let Some(target) = registry().get(&link.m_target).copied().filter(|target| target[0] == world.dimension)
    else
    {
        return;
    };

    let min: [f64; 3] = [pos.x as f64 - 1.0, pos.y as f64, pos.z as f64 - 1.0];
    let max: [f64; 3] = [pos.x as f64 + 2.0, pos.y as f64 + 2.0, pos.z as f64 + 2.0];
    if let Some(info) = world.entitiesIn(min, max).into_iter().rfind(|info| info.health.is_some())
    {
        teleport(world, pos, &info, target);
    }
}

struct TeleporterTile;

#[tile_behavior]
impl TileBehavior for TeleporterTile
{
    fn onTick(&self, ctx: &mut BlockCtx)
    {
        if !ctx.isAuthority()
        {
            return;
        }

        let before: Link = ctx.tileData();
        let mut link: Link = before;
        tick(ctx.world(), ctx.pos(), &mut link);

        if link != before && ctx.world().block(ctx.pos()) != 0
        {
            let _ = ctx.setTileData(&link);
        }
    }

    fn onRemoved(&self, ctx: &mut BlockCtx) { unregister(ctx.world(), ctx.pos()); }
}

// BlockMachine2.blockActivated with a transmitter: the teleporter's target becomes the transmitter's frequency
pub(crate) fn link(world: World, pos: BlockPos, player: Player, transmitter: &ItemStack) -> bool
{
    let mut link: Link = world.tileData(pos);
    let frequency: i32 = transmitter.damage;
    if frequency == 0 || frequency == link.m_own || frequency == link.m_target
    {
        return false;
    }

    link.m_target = frequency;
    let _ = world.setTileData(pos, &link);
    player.chat(&format!("Frequency of Teleporter set to: {frequency}"));
    true
}

// BlockMachine2.onBlockClicked with a transmitter: it takes the teleporter's own frequency
pub(crate) fn tune(world: World, pos: BlockPos, player: Player, transmitter: ItemStack)
{
    let link: Link = world.tileData(pos);
    if transmitter.damage == link.m_own
    {
        return;
    }

    let _ = player.setHeld(Some(ItemStack { damage: link.m_own, ..transmitter }));
    player.chat(&format!("Frequency of Transmitter set to: {}", link.m_own));
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()> { registerBlock(ctx, &MACHINE, TeleporterTile, false) }
