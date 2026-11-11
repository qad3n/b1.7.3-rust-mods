use super::{fraction, sounds};
use crate::models::{DEGREES_PER_RADIAN, Program, sun_spirit as model};
use b173_api::prelude::*;
use b173_api::terrain::math;

const NAME: &str = "fire_minion";
pub(crate) const SPIRIT_TEXTURE: &str = "aether:entity/boss/sun_spirit/sun_spirit";
const HURT: &str = "random.hurt";

mod minion
{
    // EntityFireMinion
    pub const FLAMES: i32 = 2;
    pub const FLAME_LIFT: f64 = 0.1;
    pub const FLAME_FALL: f64 = -0.075;
    pub const DAMAGE: i32 = 10;
}

mod look
{
    // SunSpiritModel
    pub const BREATH: f32 = 0.09;
    pub const SWAY: f32 = 0.067;
    pub const SPREAD: f32 = 0.05;
    pub const ARM_OFFSET: f32 = 0.985;
    pub const LIFT: f32 = -0.25;
}

// SunSpiritModel.setRotationAngles and render: floating fists that breathe, the sun spirit draws it larger
pub(crate) fn spiritPose(input: &PoseInput, scale: f32) -> Pose
{
    let pivot = |part: u16| model::PIVOTS[part as usize];
    let head: [f32; 3] = [input.head_pitch / DEGREES_PER_RADIAN, input.head_yaw / DEGREES_PER_RADIAN, 0.0];
    let breath: f32 = math::cos(input.age * look::BREATH) * look::SPREAD - look::SPREAD;
    let sway: f32 = math::sin(input.age * look::SWAY) * look::SPREAD;
    let rightArm: [f32; 3] = [sway, 0.0, breath];
    let leftArm: [f32; 3] = [-sway, 0.0, -breath];
    let negate = |angles: [f32; 3]| angles.map(|angle| -angle);
    let mut body: Program = Program::new();
    body.scale(scale, scale, scale).translate(0.0, look::LIFT, 0.0);
    body.part(model::HEAD, head, pivot(model::HEAD));
    body.part(model::HEADWEAR, head, pivot(model::HEADWEAR));

    for piece in [model::BODY, model::BIPED_BODY2, model::BIPED_BODY3, model::BIPED_BODY4]
    {
        body.part(piece, [0.0; 3], pivot(piece));
    }

    body.push().translate(look::ARM_OFFSET, 0.0, 0.0).part(model::RIGHT_ARM, rightArm, pivot(model::RIGHT_ARM)).pop();
    body.part(model::BIPED_RIGHT_ARM2, negate(rightArm), pivot(model::BIPED_RIGHT_ARM2));
    body.part(model::BIPED_RIGHT_ARM3, negate(rightArm), pivot(model::BIPED_RIGHT_ARM3));
    body.push().translate(-look::ARM_OFFSET, 0.0, 0.0).part(model::LEFT_ARM, leftArm, pivot(model::LEFT_ARM)).pop();
    body.part(model::BIPED_LEFT_ARM2, negate(leftArm), pivot(model::BIPED_LEFT_ARM2));
    body.part(model::BIPED_LEFT_ARM3, negate(leftArm), pivot(model::BIPED_LEFT_ARM3));
    Pose { pre: Vec::new(), model: body.take() }
}

struct FireMinion;

#[entity_behavior]
impl EntityBehavior for FireMinion
{
    // EntityFireMinion.onUpdate: hunts players and trails flames
    fn onTick(&self, entity: Entity, info: &EntityInfo)
    {
        if entity.world.isAuthority()
        {
            entity.setField(EntityField::AttackStrength, minion::DAMAGE as f64);
        }

        if entity.world.isAuthority() && !isClient()
        {
            return;
        }

        for _ in 0..minion::FLAMES
        {
            let [dx, rise, dz] = [fraction(entity.world) - 0.5, fraction(entity.world), fraction(entity.world) - 0.5];
            let spot: [f64; 3] =
                [info.pos[0] + dx * rise, info.pos[1] + rise + minion::FLAME_LIFT, info.pos[2] + dz * rise];
            entity.world.particle("flame", spot, [0.0, minion::FLAME_FALL, 0.0]);
        }
    }

    fn pose(&self, input: &PoseInput) -> Pose { spiritPose(input, 1.0) }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.entities().register(
        NAME,
        EntitySpec
        {
            template: Identifier::parse("minecraft:monster").ok(),
            texture: Some(SPIRIT_TEXTURE.to_owned()),
            size: Some([1.1, 1.8]),
            health: Some(40),
            speed: Some(0.8),
            fire_immune: true,
            drops: Some(Vec::new()),
            sounds: sounds("", HURT, HURT),
            model: Some(EntityModelSpec { shadow: 0.4, ..model::model() }),
            ..EntitySpec::default()
        },
        Some(std::rc::Rc::new(FireMinion)),
    )?;
    Ok(())
}
