pub mod ai;
pub mod arrows;
mod breath;
mod flock;
mod goblin;
mod goblin_knight;
mod mobs;
mod quest_ram;
mod redcap;
mod roving_cube;
pub mod shots;
pub mod spawns;

use b173_api::prelude::*;

pub const MINI_GHAST: &str = "twilightforest:mini_ghast";
pub const TOWER_GHAST: &str = "twilightforest:tower_ghast";
pub const PENGUIN: &str = "twilightforest:penguin";
pub const RAVEN: &str = "twilightforest:forest_raven";
pub const SKELETON_DRUID: &str = "twilightforest:skeleton_druid";
pub const SWARM_SPIDER: &str = "twilightforest:swarm_spider";
pub const UR_GHAST: &str = "twilightforest:ur_ghast";

// TileEntityTFGhastTrapActive.setInTrap, a held ur-ghast stops hunting while the trap pulls it down
pub fn ghastTrapped(entity: Entity, boss: bool)
{
    if boss
    {
        crate::bosses::ur_ghast::trapped(entity);
    }
}

// a mob placed by world generation, a mob type that is not registered yet is logged and skipped
pub fn spawnAt(world: World, kind: &str, pos: [f64; 3], yaw: f32) -> Option<Entity>
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

pub fn drop(item: &str, count: [i32; 2]) -> DropSpec
{
    DropSpec
    {
        item: Identifier::parseIn(item, "twilightforest").unwrap_or_else(|_| Identifier::minecraft("air")),
        count: count.to_vec(),
        meta: 0,
        chance: 1.0,
    }
}

pub fn sounds(living: Option<&str>, hurt: &str, death: &str) -> Option<EntitySounds>
{
    Some(EntitySounds { living: living.map(str::to_owned), hurt: Some(hurt.to_owned()), death: Some(death.to_owned()) })
}

fn distanceSquared(a: &EntityInfo, b: &EntityInfo) -> f64
{
    (0..3).map(|axis| (a.pos[axis] - b.pos[axis]).powi(2)).sum()
}

// World.getClosestPlayerToEntity
pub fn closestPlayer(world: World, info: &EntityInfo, range: f64) -> Option<EntityInfo>
{
    world
        .players()
        .into_iter()
        .filter_map(Player::info)
        .filter(|player| distanceSquared(player, info) < range * range)
        .min_by(|a, b| distanceSquared(a, info).total_cmp(&distanceSquared(b, info)))
}

pub fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    mobs::register(ctx)?;
    roving_cube::register(ctx)?;
    quest_ram::register(ctx)?;
    arrows::register(ctx)?;
    shots::register(ctx)
}
