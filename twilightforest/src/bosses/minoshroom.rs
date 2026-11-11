use super::killer;
use crate::entities::ai::{self, Ai};
use crate::models::{Program, bipedPose, tf_minoshroom};
use crate::names::item;
use crate::progress::{self, Gate};
use b173_api::prelude::*;
use std::rc::Rc;

// EntityTFMinoshroom, the minotaur boss of the labyrinth's lowest room
pub(crate) const NAME: &str = "minoshroom";
const LABEL: &str = "Minoshroom";

mod stats
{
    pub const HEALTH: i32 = 120;
    // EntityTFMinotaur keeps the zombie's 0.25 walk, beta's monsters walk at twice that
    pub const SPEED: f32 = 0.5;
    pub const ATTACK: f64 = 7.0;
    // EntityTFMinoshroom.dropFewItems draws 2 to 5 stroganoffs
    pub const STROGANOFF_MIN: i32 = 2;
    pub const STROGANOFF_SPREAD: i32 = 4;
}

struct Minoshroom;

#[entity_behavior]
impl EntityBehavior for Minoshroom
{
    fn onTick(&self, entity: Entity, info: &EntityInfo) { ai::tick(Ai::Charger, entity, info); }

    fn onDeath(&self, entity: Entity, source: Option<i32>)
    {
        ai::forget(entity);

        let world: World = entity.world;

        let Some(info) = entity.info()
        else
        {
            return;
        };

        super::conquer(world, info.pos);

        if let Some(player) = killer(world, source)
        {
            progress::grant(player, Gate::Hunter);
        }
    }

    fn pose(&self, input: &PoseInput) -> Pose
    {
        let mut body: Program = Program::new();
        bipedPose(&mut body, &tf_minoshroom::PIVOTS, &tf_minoshroom::ROTATIONS, input);
        body.held(crate::models::biped::RIGHT_ARM, crate::names::item::MINOTAUR_AXE);
        Pose { pre: Vec::new(), model: body.take() }
    }
}

// EntityMob.attackEntityAsMob hits with the minotaur's attack damage
fn armed(event: &mut Event)
{
    let EventData::Entity { entity } = event.data().clone()
    else
    {
        return;
    };
    if entity.kind.as_ref().is_none_or(|kind| kind.namespace() != "twilightforest" || kind.path() != NAME)
    {
        return;
    }

    let world: World = World { dimension: entity.world };
    if world.isAuthority()
    {
        Entity { world, id: entity.id }.setField(EntityField::AttackStrength, stats::ATTACK);
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.entities().register(
        NAME,
        EntitySpec
        {
            template: Identifier::parse("minecraft:monster").ok(),
            texture: Some("twilightforest:model/minoshroomtaur".to_owned()),
            size: Some([1.49, 2.9]),
            health: Some(stats::HEALTH),
            speed: Some(stats::SPEED),
            name: Some(LABEL.to_owned()),
            sounds: Some(EntitySounds
            {
                living: Some("mob.cow".to_owned()),
                hurt: Some("mob.cowhurt".to_owned()),
                death: Some("mob.cowhurt".to_owned()),
            }),
            drops: Some(vec![
                crate::entities::drop(
                    item::MEEF_STROGANOFF,
                    [stats::STROGANOFF_MIN, stats::STROGANOFF_MIN + stats::STROGANOFF_SPREAD - 1],
                ),
                crate::entities::drop(item::MINOTAUR_AXE, [1, 1]),
            ]),
            model: Some(EntityModelSpec { shadow: 1.0, ..tf_minoshroom::model() }),
            persistent: true,
            ..EntitySpec::default()
        },
        Some(Rc::new(Minoshroom)),
    )?;
    ctx.events().on::<EntityJoinWorld>(Priority::Normal, armed)?;
    Ok(())
}
