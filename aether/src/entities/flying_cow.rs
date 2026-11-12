use super::mount::{Saddle, WINGED, saddleInteract, travel, wingPose};
use super::{creature, drop, onAetherGrass};
use crate::models::{Program, combine, flying_cow, flying_cow_saddle, flying_cow_wing, quadruped, quadrupedPose};
use b173_api::prelude::*;

pub(crate) const NAME: &str = "flying_cow";
const TEXTURE: &str = "aether:entity/flying_cow/flying_cow";
const WINGS: &str = "aether:entity/flying_cow/wings";
const SADDLE_TEXTURE: &str = "aether:entity/flying_cow/saddle";
const MILK_BUCKET: &str = "minecraft:milk_bucket";
const BUCKET: &str = "minecraft:bucket";
// FlyingCowModel: quadruped parts, then horn1, horn2, udders
const PARTS: u16 = quadruped::PARTS + 3;
const WING_BASE: u16 = PARTS;
const SADDLE_BASE: u16 = WING_BASE + 4;
// FlyingCowWingModel lifts the wings ten model units over the cow's back
const WING_LIFT: f32 = -10.0 * 0.0625;

fn cowPose(program: &mut Program, base: u16, pivots: &[[f32; 3]], input: &PoseInput)
{
    quadrupedPose(program, base, pivots, input, None);

    let head: [f32; 3] = [input.head_pitch.to_radians(), input.head_yaw.to_radians(), 0.0];
    program.part(base + flying_cow::HORN1, head, pivots[flying_cow::HORN1 as usize]);
    program.part(base + flying_cow::HORN2, head, pivots[flying_cow::HORN2 as usize]);
    program.rest(base + flying_cow::UDDERS);
}

struct FlyingCow;

#[entity_behavior]
impl EntityBehavior for FlyingCow
{
    fn onTick(&self, entity: Entity, _info: &EntityInfo)
    {
        entity.setField(EntityField::FallDistance, 0.0);
        super::mount::seat(entity, super::mount::ride::COW_SEAT);
    }

    // EntityFlyingCow.interact: a bucket milks it, a saddle or an empty hand rides it
    fn onInteract(&self, entity: Entity, player: Player) -> bool
    {
        if let Some(held) = player.held().filter(|held| held.item.is(BUCKET))
        {
            if held.count == 1
            {
                let _ = player.setHeld(Some(ItemStack::of(MILK_BUCKET, 1)));
            }

            return true;
        }

        if crate::gear::milk(player)
        {
            return true;
        }

        saddleInteract(entity, player)
    }

    fn onHurt(&self, entity: Entity, source: Option<i32>, amount: i32) -> Option<i32>
    {
        if source.is_some() && source == entity.rider()
        {
            None
        }
        else
        {
            Some(amount)
        }
    }

    fn onTravel(&self, entity: Entity, _info: &EntityInfo, _strafe: f32, _forward: f32) -> bool
    {
        travel(entity, WINGED)
    }

    fn canSpawn(&self, entity: Entity, info: &EntityInfo) -> bool
    {
        crate::config::spawnRoll(entity, "flying_cow") && onAetherGrass(info)
    }

    // FlyingCowRenderer: the cow, its wings, then the saddle when it has one
    fn pose(&self, input: &PoseInput) -> Pose
    {
        let saddle: Saddle = b173_api::fromBytes(&input.synced).unwrap_or_default();
        let mut body: Program = Program::new();
        cowPose(&mut body, 0, &flying_cow::PIVOTS, input);
        body.texture(WINGS).push().translate(0.0, WING_LIFT, 0.0);
        wingPose(&mut body, WING_BASE, &flying_cow_wing::ROTATIONS, input);
        body.pop();

        if saddle.m_saddled
        {
            body.texture(SADDLE_TEXTURE);
            cowPose(&mut body, SADDLE_BASE, &flying_cow_saddle::PIVOTS, input);
        }

        Pose { pre: Vec::new(), model: body.take() }
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.entities().register(
        NAME,
        EntitySpec
        {
            template: Identifier::parse("minecraft:cow").ok(),
            texture: Some(TEXTURE.to_owned()),
            size: Some([0.9, 1.3]),
            health: Some(10),
            spawn: creature(8),
            drops: Some(vec![drop("minecraft:leather", [0, 2])]),
            sounds: Some(EntitySounds
            {
                living: Some("mob.cow".to_owned()),
                hurt: Some("mob.cowhurt".to_owned()),
                death: Some("mob.cowhurt".to_owned()),
            }),

            model: Some(EntityModelSpec
            {
                shadow: 0.7,
                ..combine(vec![flying_cow::model(), flying_cow_wing::model(), flying_cow_saddle::model()])
            }),
            ..EntitySpec::default()
        },
        Some(std::rc::Rc::new(FlyingCow)),
    )?;
    Ok(())
}
