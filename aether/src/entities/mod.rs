mod aechor_plant;
mod aerbunny;
mod aerwhale;
mod bird;
pub(crate) mod bosses;
mod cockatrice;
pub(crate) mod crystal;
pub(crate) mod fire_minion;
pub(crate) mod floating_block;
mod flying_cow;
pub(crate) mod mimic;
pub(crate) mod mini_cloud;
pub(crate) mod moa;
mod mount;
mod phyg;
pub(crate) mod present;
pub(crate) mod projectile;
mod sentry;
mod sheepuff;
mod swet;
pub(crate) mod valkyrie;
mod whirlwind;
mod zephyr;

use crate::names::block;
use b173_api::math::{blockOf, distanceSquared};
use b173_api::prelude::*;

pub(crate) const BIOME: &str = "aether:highlands";

// EntityAetherAnimal.getCanSpawnHere: on aether grass in some light
const SPAWN_LIGHT: i32 = 9;

pub(crate) fn onAetherGrass(info: &EntityInfo) -> bool
{
    let world: World = World { dimension: info.world };
    let feet: BlockPos = blockOf(info.pos);
    world.is(feet.offset(0, -1, 0), block::AETHER_GRASS) && world.light(feet) >= SPAWN_LIGHT
}

pub(crate) fn creature(weight: i32) -> Option<SpawnSpec>
{
    Some(SpawnSpec
    {
        kind: CreatureKind::Creature,
        weight,
        biomes: Identifier::parse(BIOME).ok().into_iter().collect(),
    })
}

pub(crate) fn monster(weight: i32) -> Option<SpawnSpec>
{
    Some(SpawnSpec { kind: CreatureKind::Monster, weight, biomes: Identifier::parse(BIOME).ok().into_iter().collect() })
}

pub(crate) fn sounds(living: &str, hurt: &str, death: &str) -> Option<EntitySounds>
{
    let name = |sound: &str| Some(sound.to_owned());
    Some(EntitySounds { living: name(living), hurt: name(hurt), death: name(death) })
}

pub(crate) fn drop(item: &str, count: [i32; 2]) -> DropSpec
{
    DropSpec
    {
        item: Identifier::parseIn(item, "aether").unwrap_or_else(|_| Identifier::minecraft("air")),
        count: count.to_vec(),
        meta: 0,
        chance: 1.0,
    }
}

// World.getClosestPlayerToEntity
pub(crate) fn closestPlayer(world: World, info: &EntityInfo, range: f64) -> Option<EntityInfo>
{
    world
        .players()
        .into_iter()
        .filter_map(Player::info)
        .filter(|player| distanceSquared(player.pos, info.pos) < range * range)
        .min_by(|a, b| distanceSquared(a.pos, info.pos).total_cmp(&distanceSquared(b.pos, info.pos)))
}

// Entity.getLookVec
pub(crate) fn lookOf(info: &EntityInfo) -> [f64; 3]
{
    let (yaw, pitch) = ((info.yaw as f64).to_radians(), (info.pitch as f64).to_radians());
    [-yaw.sin() * pitch.cos(), -pitch.sin(), yaw.cos() * pitch.cos()]
}

// a fraction in [0, 1) from the world random, for the Java nextFloat calls
pub(crate) fn fraction(world: World) -> f64
{
    const STEPS: i32 = 1000000;
    world.random(STEPS) as f64 / STEPS as f64
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    aerbunny::register(ctx)?;
    sheepuff::register(ctx)?;
    phyg::register(ctx)?;
    flying_cow::register(ctx)?;
    aerwhale::register(ctx)?;
    moa::register(ctx)?;
    projectile::register(ctx)?;
    cockatrice::register(ctx)?;
    zephyr::register(ctx)?;
    swet::register(ctx)?;
    aechor_plant::register(ctx)?;
    whirlwind::register(ctx)?;
    mimic::register(ctx)?;
    sentry::register(ctx)?;
    valkyrie::register(ctx)?;
    fire_minion::register(ctx)?;
    floating_block::register(ctx)?;
    present::register(ctx)?;
    crystal::register(ctx)?;
    mini_cloud::register(ctx)?;
    bosses::register(ctx)?;
    loot(ctx)
}

// drops that depend on a mob's state go through loot conditions, so recipe viewers list them
fn loot(ctx: &mut RegisterCtx) -> Result<()>
{
    sheepuff::loot(ctx)?;
    swet::loot(ctx)?;
    mount::loot(ctx, &[phyg::NAME, flying_cow::NAME])?;
    moa::loot(ctx)
}
