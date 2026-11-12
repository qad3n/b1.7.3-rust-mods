use super::mount::{Saddle, WINGED, saddleInteract, travel, wingPose};
use super::{creature, drop, onAetherGrass};
use crate::models::{Program, combine, phyg_wing, pig, pig_saddle, quadruped, quadrupedPose};
use crate::progress::{self, Award};
use b173_api::prelude::*;

pub(crate) const NAME: &str = "phyg";
const TEXTURE: &str = "aether:entity/phyg/phyg";
const WINGS: &str = "aether:entity/phyg/wings";
const SADDLE_TEXTURE: &str = "minecraft:mob/saddle";
const WING_BASE: u16 = quadruped::PARTS;
const SADDLE_BASE: u16 = WING_BASE + 4;

struct Phyg;

#[entity_behavior]
impl EntityBehavior for Phyg
{
    fn onTick(&self, entity: Entity, _info: &EntityInfo)
    {
        entity.setField(EntityField::FallDistance, 0.0);
        super::mount::seat(entity, super::mount::ride::PHYG_SEAT);

        if !entity.world.isAuthority()
        {
            return;
        }

        // EntityPhyg.onUpdate, a player riding earns it
        if let Some(rider) =
            entity.rider().and_then(|id| entity.world.entity(id)).filter(|rider| rider.player.is_some())
        {
            progress::grant(Player { id: rider.id }, Award::FlyingPig);
        }
    }

    fn onInteract(&self, entity: Entity, player: Player) -> bool { saddleInteract(entity, player) }

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
        crate::config::spawnRoll(entity, "phyg") && onAetherGrass(info)
    }

    // PhygRenderer: the pig, its wings, then the saddle when it has one
    fn pose(&self, input: &PoseInput) -> Pose
    {
        let saddle: Saddle = b173_api::fromBytes(&input.synced).unwrap_or_default();
        let mut body: Program = Program::new();
        quadrupedPose(&mut body, 0, &pig::PIVOTS, input, None);
        body.texture(WINGS);
        wingPose(&mut body, WING_BASE, &phyg_wing::ROTATIONS, input);

        if saddle.m_saddled
        {
            body.texture(SADDLE_TEXTURE);
            quadrupedPose(&mut body, SADDLE_BASE, &pig_saddle::PIVOTS, input, None);
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
            template: Identifier::parse("minecraft:pig").ok(),
            texture: Some(TEXTURE.to_owned()),
            size: Some([0.9, 1.3]),
            health: Some(10),
            spawn: creature(12),
            drops: Some(vec![drop("minecraft:feather", [1, 3]), drop("minecraft:porkchop", [1, 3])]),
            sounds: Some(EntitySounds
            {
                living: Some("mob.pig".to_owned()),
                hurt: Some("mob.pig".to_owned()),
                death: Some("mob.pigdeath".to_owned()),
            }),

            model: Some(EntityModelSpec
            {
                shadow: 0.7,
                ..combine(vec![pig::model(), phyg_wing::model(), pig_saddle::model()])
            }),
            ..EntitySpec::default()
        },
        Some(std::rc::Rc::new(Phyg)),
    )?;
    Ok(())
}
