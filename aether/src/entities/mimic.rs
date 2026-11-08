use super::sounds;
use crate::models::{Program, mimic as model};
use b173_api::prelude::*;
use std::f32::consts::PI;

const NAME: &str = "mimic";
const HEAD: &str = "aether:entity/mimic/mimic_head";
const LEGS: &str = "aether:entity/mimic/mimic_legs";
const SLIME: &str = "mob.slime";

mod mimic
{
    // EntityMimic: AttackOnCollide for three, axes cut half again as deep
    pub const ATTACK: f64 = 3.0;
    pub const AXE_BONUS: f32 = 1.5;
    pub const MOUTH_PERIOD: f32 = 10.0;
    pub const MOUTH_OPEN: f32 = 0.6;
    // legs swing back and forth over ten ticks while it walks
    pub const STRIDE_TICKS: f32 = 10.0;
    pub const WALKING: f32 = 0.01;
    // MimicRenderer draws without RenderLiving's 1.5 block model offset
    pub const MODEL_DROP: f32 = 1.5078125;
}

fn isAxe(stack: &ItemStack) -> bool { stack.item.name().is_some_and(|name| name.path().ends_with("_axe")) }

struct Mimic;

#[entity_behavior]
impl EntityBehavior for Mimic
{
    fn onTick(&self, entity: Entity, _info: &EntityInfo)
    {
        entity.setField(EntityField::AttackStrength, mimic::ATTACK);
    }

    // EntityMimic.attackEntityFrom: other mimics cannot hurt it, axes hurt it more
    fn onHurt(&self, entity: Entity, source: Option<i32>, amount: i32) -> Option<i32>
    {
        let Some(attacker) = source
        else
        {
            return Some(amount);
        };
        let attackerInfo: Option<EntityInfo> = Entity { world: entity.world, id: attacker }.info();
        if attackerInfo.as_ref().and_then(|info| info.kind.as_ref()).is_some_and(|kind| kind.path() == NAME)
        {
            return None;
        }
        entity.setField(EntityField::TargetId, attacker as f64);
        let axe: bool = attackerInfo.is_some_and(|info| info.player.is_some())
            && Player { id: attacker }.held().is_some_and(|held| isAxe(&held));
        Some(
            if axe
            {
                (amount as f32 * mimic::AXE_BONUS) as i32
            }
            else
            {
                amount
            },
        )
    }

    // MimicRenderer: the chest body on the head sheet, lid and legs on the legs sheet
    fn pose(&self, input: &PoseInput) -> Pose
    {
        let mouth: f32 = ((input.age / mimic::MOUTH_PERIOD * PI).cos() + 1.0) * mimic::MOUTH_OPEN;
        let phase: f32 = (input.age / mimic::STRIDE_TICKS).rem_euclid(2.0);
        let legs: f32 = if input.walk_speed > mimic::WALKING
        {
            if phase < 1.0
            {
                phase * 2.0 - 1.0
            }
            else
            {
                3.0 - phase * 2.0
            }
        }
        else
        {
            0.0
        };
        let mut pre: Program = Program::new();
        pre.translate(0.0, mimic::MODEL_DROP, 0.0);
        let mut body: Program = Program::new();
        body.texture(HEAD).rest(model::BOX);
        body.texture(LEGS);
        body.part(model::BOX_LID, [PI - mouth, 0.0, 0.0], model::PIVOTS[model::BOX_LID as usize]);
        body.part(model::LEFT_LEG, [-legs, 0.0, 0.0], model::PIVOTS[model::LEFT_LEG as usize]);
        body.part(model::RIGHT_LEG, [legs, 0.0, 0.0], model::PIVOTS[model::RIGHT_LEG as usize]);
        Pose { pre: pre.take(), model: body.take() }
    }
}

pub fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.entities().register(
        NAME,
        EntitySpec {
            template: Identifier::parse("minecraft:monster").ok(),
            texture: Some(HEAD.to_owned()),
            size: Some([1.0, 2.0]),
            health: Some(40),
            speed: Some(0.68 * 0.7 / 0.25),
            drops: Some(vec![super::drop("minecraft:chest", [1, 1])]),
            sounds: sounds("", SLIME, SLIME),
            model: Some(EntityModelSpec { shadow: 0.0, ..model::model() }),
            ..EntitySpec::default()
        },
        Some(std::rc::Rc::new(Mimic)),
    )?;
    Ok(())
}
