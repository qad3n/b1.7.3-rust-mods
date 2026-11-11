use crate::ids::blockId;
use b173_api::prelude::*;

const NAME: &str = "tnt_present";
const PRESENT: &str = "present";

mod fuse
{
    // EntityTNTPresent pops up, falls, bounces and goes off after ten ticks
    pub const TICKS: f64 = 10.0;
    pub const POP: f64 = 0.2;
    pub const GRAVITY: f64 = 0.04;
    pub const DRAG: f64 = 0.98;
    pub const BOUNCE: f64 = -0.5;
    pub const POWER: f32 = 0.4;
    pub const SMOKE_RISE: f64 = 0.5;
    // the loader's moving block stands at its center, the source's box at its bottom
    pub const HALF: f64 = 0.49;
    pub const CENTER: f64 = 0.5;
    // EntitiesAether registers it with a 160 block range and an update every 20 ticks
    pub const TRACKING: i32 = 160;
    pub const UPDATES: i32 = 20;
}

// BlockPresent.harvestBlock primes a present that falls and goes off
pub(crate) fn prime(world: World, pos: BlockPos)
{
    let center: [f64; 3] =
        [f64::from(pos.x) + fuse::CENTER, f64::from(pos.y) + fuse::HALF, f64::from(pos.z) + fuse::CENTER];
    match world.spawnEntity(&format!("aether:{NAME}"), center)
    {
        Ok(entity) =>
        {
            entity.setField(EntityField::Block, f64::from(blockId(PRESENT)));

            let _ = entity.setMotion([0.0, fuse::POP, 0.0]);
        }

        Err(error) => warn(&format!("could not prime a present: {}", error.0)),
    }
}

// EntityTNTPresent.onUpdate
struct TntPresent;

#[entity_behavior]
impl EntityBehavior for TntPresent
{
    fn onTick(&self, entity: Entity, info: &EntityInfo)
    {
        let world: World = entity.world;
        entity.setField(EntityField::Block, f64::from(blockId(PRESENT)));

        if !world.isAuthority()
        {
            world.particle("smoke", [info.pos[0], info.pos[1] - fuse::HALF + fuse::SMOKE_RISE, info.pos[2]], [0.0; 3]);
            return;
        }

        let fall: f64 = (info.motion[1] - fuse::GRAVITY) * fuse::DRAG;
        let rise: f64 = if entity.flag(EntityField::OnGround)
        {
            fall * fuse::BOUNCE
        }
        else
        {
            fall
        };
        let _ = entity.setMotion([info.motion[0], rise, info.motion[2]]);

        if entity.field(EntityField::TicksExisted) <= fuse::TICKS
        {
            return;
        }

        entity.kill();

        let bottom: [f64; 3] = [info.pos[0], info.pos[1] - fuse::HALF, info.pos[2]];
        if let Err(error) = world.explode(bottom, fuse::POWER)
        {
            warn(&format!("a present could not explode: {}", error.0));
        }
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.entities().register(
        NAME,
        EntitySpec
        {
            template: Identifier::parse("minecraft:moving_block").ok(),
            tracking_range: Some(fuse::TRACKING),
            update_frequency: Some(fuse::UPDATES),
            send_velocity: Some(true),
            ..EntitySpec::default()
        },
        Some(std::rc::Rc::new(TntPresent)),
    )?;
    Ok(())
}
