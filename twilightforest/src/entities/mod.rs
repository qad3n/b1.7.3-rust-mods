pub(crate) mod ai;
pub(crate) mod arrows;
mod breath;
mod flock;
mod goblin;
mod goblin_knight;
mod mobs;
mod quest_ram;
mod redcap;
mod roving_cube;
pub(crate) mod shots;
pub(crate) mod spawns;

use b173_api::math::distanceSquared;
use b173_api::prelude::*;

pub(crate) const MINI_GHAST: &str = "twilightforest:mini_ghast";
pub(crate) const TOWER_GHAST: &str = "twilightforest:tower_ghast";
pub(crate) const PENGUIN: &str = "twilightforest:penguin";
pub(crate) const RAVEN: &str = "twilightforest:forest_raven";
pub(crate) const SKELETON_DRUID: &str = "twilightforest:skeleton_druid";
pub(crate) const SWARM_SPIDER: &str = "twilightforest:swarm_spider";
pub(crate) const UR_GHAST: &str = "twilightforest:ur_ghast";

// writes per-entity state and logs a failed host call since the next tick reads the default
pub(crate) fn store<T>(state: &EntityState<T>, entity: Entity, value: &T)
where T: Serialize + serde::de::DeserializeOwned + Default
{
    if let Err(error) = state.set(entity, value)
    {
        warn(&format!("entity {} state: {error}", entity.id));
    }
}

// drops per-entity state before the entity leaves its world, as a boss that keeps ticking while it dies
pub(crate) fn clear<T>(state: &EntityState<T>, entity: Entity)
where T: Serialize + serde::de::DeserializeOwned + Default
{
    if let Err(error) = state.clear(entity)
    {
        warn(&format!("entity {} state: {error}", entity.id));
    }
}

// TileEntityTFGhastTrapActive.setInTrap, a held ur-ghast stops hunting while the trap pulls it down
pub(crate) fn ghastTrapped(entity: Entity, boss: bool)
{
    if boss
    {
        crate::bosses::ur_ghast::trapped(entity);
    }
}

// a mob placed by world generation, a mob type that is not registered yet is logged and skipped
pub(crate) fn spawnAt(world: World, kind: &str, pos: [f64; 3], yaw: f32) -> Option<Entity>
{
    match world.spawnEntity(kind, pos)
    {
        Ok(entity) =>
        {
            entity.setField(EntityField::Yaw, f64::from(yaw));
            Some(entity)
        }

        Err(error) =>
        {
            b173_api::warn(&format!("cannot spawn {kind}: {}", error.0));
            None
        }
    }
}

pub(crate) fn drop(item: &str, count: [i32; 2]) -> DropSpec
{
    DropSpec
    {
        item: Identifier::parseIn(item, "twilightforest").unwrap_or_else(|_| Identifier::minecraft("air")),
        count: count.to_vec(),
        meta: 0,
        chance: 1.0,
    }
}

pub(crate) fn sounds(living: Option<&str>, hurt: &str, death: &str) -> Option<EntitySounds>
{
    Some(EntitySounds { living: living.map(str::to_owned), hurt: Some(hurt.to_owned()), death: Some(death.to_owned()) })
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

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    mobs::register(ctx)?;
    spawns::register(ctx)?;
    roving_cube::register(ctx)?;
    quest_ram::register(ctx)?;
    arrows::register(ctx)?;
    shots::register(ctx)
}
