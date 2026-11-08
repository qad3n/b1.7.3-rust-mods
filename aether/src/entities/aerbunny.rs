use super::{creature, drop, onAetherGrass, sounds};
use crate::models::{DEGREES_PER_RADIAN, HALF_PI, Program, aerbunny as model};
use b173_api::prelude::*;
use b173_api::terrain::math;
use std::cell::RefCell;
use std::collections::HashMap;

const NAME: &str = "aerbunny";
const TEXTURE: &str = "aether:entity/aerbunny/aerbunny";

mod hop
{
    // EntityLiving.jump in b1.7.3
    pub const JUMP: f64 = 0.42;
    pub const MAX_FALL: f64 = -0.1;
    // a carried bunny slows its carrier's fall and lets it hop once more in the air
    pub const LIFT: f64 = 0.05;
    pub const DOUBLE_JUMP_BELOW: f64 = -0.225;
    pub const DOUBLE_JUMP: f64 = 0.125;
    pub const PUFF_TICKS: f32 = 11.0;
    pub const PUFF_SCALE: f32 = 0.5;
    pub const SMOKE: i32 = 3;
}

// the client works the puff out from the hops it sees, EntityAerbunny.puffiness counts down from 11 after a jump
thread_local! {
    static HOPS: RefCell<HashMap<i32, (bool, f32)>> = RefCell::new(HashMap::new());
}

fn puffiness(input: &PoseInput) -> f32
{
    HOPS.with(|hops| {
        let mut hops = hops.borrow_mut();
        let state: &mut (bool, f32) = hops.entry(input.entity.id).or_insert((input.on_ground, f32::MIN));
        if state.0 && !input.on_ground && input.entity.motion[1] > 0.0
        {
            state.1 = input.age;
        }
        state.0 = input.on_ground;
        (hop::PUFF_TICKS - (input.age - state.1)).clamp(0.0, hop::PUFF_TICKS) / 10.0
    })
}

struct Aerbunny;

#[entity_behavior]
impl EntityBehavior for Aerbunny
{
    // EntityAerbunny.onLivingUpdate
    fn onLivingUpdate(&self, entity: Entity, info: &EntityInfo)
    {
        let [motionX, motionY, motionZ] = info.motion;
        if entity.flag(EntityField::OnGround) && info.health.is_some_and(|health| health > 0)
        {
            let _ = entity.setMotion([motionX, hop::JUMP, motionZ]);
            for _ in 0..hop::SMOKE
            {
                entity.world.particle("explode", info.pos, [0.0; 3]);
            }
        }
        else if motionY < hop::MAX_FALL
        {
            let _ = entity.setMotion([motionX, hop::MAX_FALL, motionZ]);
        }
        let Some(carrier) = entity.riding()
        else
        {
            return;
        };
        let player: Entity = Entity { world: entity.world, id: carrier };
        player.setField(EntityField::FallDistance, 0.0);
        if let Some(carried) = player.info()
            && !player.flag(EntityField::OnGround)
        {
            let [x, y, z] = carried.motion;
            let lifted: f64 = y + hop::LIFT;
            let next: f64 = if lifted < hop::DOUBLE_JUMP_BELOW && player.flag(EntityField::Jumping)
            {
                hop::DOUBLE_JUMP
            }
            else
            {
                lifted
            };
            let _ = player.setMotion([x, next, z]);
        }
    }

    // EntityAerbunny.interact, an empty hand picks the bunny up or sets it down
    fn onInteract(&self, entity: Entity, player: Player) -> bool
    {
        if player.held().is_some()
        {
            return false;
        }
        if let Some(info) = entity.info()
        {
            entity.world.playSound(info.pos, "aether.aemob.aerbunny.lift", 1.0, 1.0);
        }
        let carried: bool = entity.riding() == Some(player.id);
        let _ = entity.mount(if carried { None } else { Some(player.id) });
        true
    }

    fn onHurt(&self, entity: Entity, source: Option<i32>, amount: i32) -> Option<i32>
    {
        if source.is_some() && source == entity.riding()
        {
            None
        }
        else
        {
            Some(amount)
        }
    }

    fn canSpawn(&self, entity: Entity, info: &EntityInfo) -> bool
    {
        crate::config::spawnRoll(entity, "aerbunny") && onAetherGrass(info)
    }

    // AerbunnyModel.render and setRotationAngles, with AerbunnyRenderer.rotateAerbunny before it
    fn pose(&self, input: &PoseInput) -> Pose
    {
        let mut pre: Program = Program::new();
        if !input.riding
        {
            pre.translate(0.0, 0.2, 0.0);
        }
        else if input.riding_local
        {
            pre.translate(0.0, 1.7, 0.0);
        }
        if !input.on_ground
        {
            let lean: f32 = (input.entity.motion[1] as f32 * 30.0).clamp(-15.0, 15.0);
            pre.rotate(lean, -1.0, 0.0, 0.0);
        }
        let head: [f32; 3] = [-(input.head_pitch / DEGREES_PER_RADIAN), input.head_yaw / DEGREES_PER_RADIAN, 0.0];
        let swing: f32 = math::cos(input.walk * 0.6662) * input.walk_speed;
        let swingBack: f32 = math::cos(input.walk * 0.6662 + std::f32::consts::PI) * 1.2 * input.walk_speed;
        let pivot = |part: u16| model::PIVOTS[part as usize];
        let puff: f32 = 1.0 + puffiness(input) * hop::PUFF_SCALE;
        let mut body: Program = Program::new();
        for part in [model::A, model::G, model::G2, model::H, model::H2]
        {
            body.part(part, head, pivot(part));
        }
        body.part(model::B, [HALF_PI, 0.0, 0.0], pivot(model::B));
        body.part(model::B2, [HALF_PI, 0.0, 0.0], pivot(model::B2));
        body.push().translate(0.0, 1.0, 0.0).scale(puff, puff, puff);
        body.part(model::B3, [HALF_PI, 0.0, 0.0], pivot(model::B3));
        body.pop();
        body.part(model::E1, [swing, 0.0, 0.0], pivot(model::E1));
        body.part(model::E2, [swing, 0.0, 0.0], pivot(model::E2));
        body.part(model::FF1, [swingBack, 0.0, 0.0], pivot(model::FF1));
        body.part(model::FF2, [swingBack, 0.0, 0.0], pivot(model::FF2));
        Pose { pre: pre.take(), model: body.take() }
    }
}

pub fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.entities().register(
        NAME,
        EntitySpec {
            template: Identifier::parse("minecraft:pig").ok(),
            texture: Some(TEXTURE.to_owned()),
            size: Some([0.4, 0.4]),
            health: Some(5),
            spawn: creature(6),
            drops: Some(vec![drop("minecraft:string", [1, 1])]),
            sounds: sounds("", "aether.aemob.aerbunny.hurt", "aether.aemob.aerbunny.death"),
            model: Some(EntityModelSpec { shadow: 0.3, ..model::model() }),
            ..EntitySpec::default()
        },
        Some(std::rc::Rc::new(Aerbunny)),
    )?;
    Ok(())
}
