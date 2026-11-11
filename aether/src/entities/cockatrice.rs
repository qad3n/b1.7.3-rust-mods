use super::projectile::{self, POISON_NEEDLE};
use super::{bird, drop, monster, sounds};
use crate::models::{Program, cockatrice as model};
use b173_api::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

const NAME: &str = "cockatrice";
const TEXTURE: &str = "aether:entity/cockatrice/cockatrice";

mod aim
{
    // EntityCockatrice.onUpdate: the shot timer climbs two a tick to twenty, then waits sixty after a shot
    pub const READY: i32 = 20;
    pub const CLIMB: i32 = 2;
    pub const RELOAD: i32 = -60;
    pub const SPEED: f64 = 1.0;
    pub const SPREAD: f64 = 1.0;
    // EntityLiving.getEyeHeight of a two block tall mob
    pub const EYE: f64 = 1.7;
    pub const SLOW_FALL: f64 = 0.6;
}

thread_local!
{
    static TIMERS: RefCell<HashMap<i32, i32>> = RefCell::new(HashMap::new());
}

struct Cockatrice;

fn eyes(info: &EntityInfo) -> [f64; 3] { [info.pos[0], info.pos[1] + aim::EYE, info.pos[2]] }

#[entity_behavior]
impl EntityBehavior for Cockatrice
{
    fn canSpawn(&self, entity: Entity, _info: &EntityInfo) -> bool { crate::config::spawnRoll(entity, "cockatrice") }

    // EntityCockatrice.onUpdate and onLivingUpdate
    fn onTick(&self, entity: Entity, info: &EntityInfo)
    {
        entity.setField(EntityField::FallDistance, 0.0);

        if !entity.flag(EntityField::OnGround) && info.motion[1] < 0.0
        {
            let _ = entity.setMotion([info.motion[0], info.motion[1] * aim::SLOW_FALL, info.motion[2]]);
        }

        if !entity.world.isAuthority()
        {
            return;
        }
        let target: i32 = entity.field(EntityField::TargetId) as i32;
        let Some(goal) = (target >= 0).then(|| Entity { world: entity.world, id: target }.info()).flatten()
        else
        {
            return;
        };

        let (dx, dz): (f64, f64) = (goal.pos[0] - info.pos[0], goal.pos[2] - info.pos[2]);
        entity.setField(EntityField::Yaw, dz.atan2(dx).to_degrees() - 90.0);

        let timer: i32 = TIMERS.with(|timers| timers.borrow().get(&entity.id).copied()).unwrap_or(0);
        let seen: bool = entity.world.rayTrace(eyes(info), eyes(&goal), false).is_none();
        let next: i32 = if timer >= aim::READY && seen
        {
            if projectile::fire(
                entity.world,
                POISON_NEEDLE,
                eyes(info),
                eyes(&goal),
                aim::SPEED,
                aim::SPREAD,
                Some(entity.id),
            )
            .is_some()
            {
                entity.world.playSound(info.pos, "random.bow", 1.0, 1.2);
            }
            aim::RELOAD
        }
        else if timer < aim::READY
        {
            timer + aim::CLIMB
        }
        else
        {
            timer
        };
        TIMERS.with(|timers| timers.borrow_mut().insert(entity.id, next));
    }

    fn onHurt(&self, entity: Entity, source: Option<i32>, amount: i32) -> Option<i32>
    {
        if let Some(attacker) = source
        {
            entity.setField(EntityField::TargetId, attacker as f64);
        }

        Some(amount)
    }

    fn pose(&self, input: &PoseInput) -> Pose
    {
        let mut body: Program = Program::new();
        bird::pose(&mut body, 0, &model::PIVOTS, &model::ROTATIONS, input, false, bird::wingSwing(input));
        Pose { pre: Vec::new(), model: body.take() }
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.entities().register(
        NAME,
        EntitySpec
        {
            template: Identifier::parse("minecraft:monster").ok(),
            texture: Some(TEXTURE.to_owned()),
            size: Some([1.0, 2.0]),
            health: Some(10),
            spawn: monster(100),
            drops: Some(vec![drop("minecraft:feather", [1, 4])]),
            sounds: sounds("aether.aemob.moa.say", "aether.aemob.moa.say", "aether.aemob.moa.say"),
            model: Some(EntityModelSpec { shadow: 1.0, ..model::model() }),
            ..EntitySpec::default()
        },
        Some(std::rc::Rc::new(Cockatrice)),
    )?;
    Ok(())
}
