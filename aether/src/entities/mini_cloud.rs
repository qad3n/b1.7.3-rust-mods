use super::crystal::{self, Kind};
use super::{fraction, lookOf, store};
use crate::models::{Program, mini_cloud as model};
use b173_api::math::{distance, length, sub};
use b173_api::prelude::*;

const NAME: &str = "mini_cloud";
const TEXTURE: &str = "aether:entity/mini_cloud/mini_cloud";

mod cloud
{
    // EntityMiniCloud
    pub const LIFESPAN: f64 = 3600.0;
    pub const SHOULDER: f64 = 1.05;
    pub const LIFT: f64 = 1.1;
    pub const FAR: f64 = 2.0;
    pub const SETTLED: f64 = 0.3;
    pub const APPROACH: f64 = 3.25;
    pub const HOLD: f64 = 0.65;
    pub const RELOAD: i32 = 40;
    pub const MUZZLE: f64 = 1.6;
    pub const SHOT_SPEED: f64 = 1.5;
    pub const VOLUME: f32 = 0.75;
    pub const WOBBLE: f64 = 0.2;
}

// which player a cloud keeps to and on which shoulder
#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct Bond
{
    m_owner: i32,
    m_left: bool,
}

// the Java saves it as ShotTimer
static RELOADS: EntityState<i32> = EntityState::new("reload").saved();

fn target(owner: &EntityInfo, cloud: &EntityInfo, left: bool) -> [f64; 3]
{
    let gap: f64 = distance(owner.pos, cloud.pos);
    if gap > cloud::FAR
    {
        return [owner.pos[0], owner.pos[1] + cloud::LIFT, owner.pos[2]];
    }

    let side: f64 = (owner.yaw as f64 + if left { -90.0 } else { 90.0 }).to_radians();
    [
        owner.pos[0] - side.sin() * cloud::SHOULDER,
        owner.pos[1] + cloud::LIFT,
        owner.pos[2] + side.cos() * cloud::SHOULDER,
    ]
}

// EntityMiniCloud.onEntityUpdate: drifts to its owner's shoulder and fires a cloud crystal each time the owner swings
struct MiniCloud;

#[entity_behavior]
impl EntityBehavior for MiniCloud
{
    fn onAi(&self, _entity: Entity, _info: &EntityInfo) {}

    fn onTick(&self, entity: Entity, info: &EntityInfo)
    {
        let world: World = entity.world;
        if !world.isAuthority()
        {
            return;
        }

        let bond: Bond = entity.synced();
        let owner: Option<EntityInfo> = Entity { world, id: bond.m_owner }.info().filter(|owner| owner.alive);
        let Some(owner) = owner.filter(|_| entity.field(EntityField::TicksExisted) < cloud::LIFESPAN)
        else
        {
            world.particle("explode", info.pos, [0.0; 3]);
            entity.kill();
            return;
        };

        let mut reload: i32 = RELOADS.get(entity);
        reload -= 1;

        let goal: [f64; 3] = target(&owner, info, bond.m_left);
        let offset: [f64; 3] = sub(goal, info.pos);
        let gap: f64 = length(offset);
        if gap < cloud::SETTLED
        {
            let _ = entity.setMotion(info.motion.map(|axis| axis * cloud::HOLD));

            entity.setField(EntityField::Yaw, owner.yaw as f64 + if bond.m_left { 1.0 } else { -1.0 });
            entity.setField(EntityField::Pitch, owner.pitch as f64);

            let swinging: bool = Entity { world, id: owner.id }.field(EntityField::SwingProgress) > 0.0;
            if reload <= 0 && swinging
            {
                let side: f64 = (owner.yaw as f64).to_radians();
                let muzzle: [f64; 3] = [
                    info.pos[0] - side.sin() * cloud::MUZZLE,
                    info.pos[1] + 1.0,
                    info.pos[2] + side.cos() * cloud::MUZZLE,
                ];

                let aim: [f64; 3] = lookOf(&owner).map(|axis| axis * cloud::SHOT_SPEED);
                crystal::spawnMoving(world, Kind::Cloud, muzzle, Some(aim), None, Some(owner.id));

                let pitch: f32 = ((fraction(world) - fraction(world)) * cloud::WOBBLE + 1.0) as f32;
                world.playSound(info.pos, "aether.aemob.zephyr.shoot", cloud::VOLUME, pitch);
                reload = cloud::RELOAD;
            }
        }
        else
        {
            let pull: f64 = gap * cloud::APPROACH;
            let _ = entity.setMotion([
                (info.motion[0] + offset[0] / pull) / 2.0,
                (info.motion[1] + offset[1] / pull) / 2.0,
                (info.motion[2] + offset[2] / pull) / 2.0,
            ]);
        }
        store(&RELOADS, entity, &reload);
    }

    // its owner's own clouds and walls never hurt it
    fn onHurt(&self, entity: Entity, source: Option<i32>, amount: i32) -> Option<i32>
    {
        let bond: Bond = entity.synced();
        (source.is_some() && source != Some(bond.m_owner)).then_some(amount)
    }

    fn pose(&self, _input: &PoseInput) -> Pose
    {
        let mut body: Program = Program::new();

        for part in model::HEAD_0..=model::HEAD_4
        {
            body.rest(part);
        }

        Pose { pre: Vec::new(), model: body.take() }
    }
}

// ItemCloudStaff: two clouds, one each side, while the player has none; sneaking dismisses them
struct CloudStaff;

#[item_behavior]
impl ItemBehavior for CloudStaff
{
    fn onRightClick(&self, world: World, player: Player, stack: ItemStack) -> Use
    {
        let Some(info) = player.info().filter(|_| world.isAuthority())
        else
        {
            return Use::Done(Some(stack));
        };

        let mine: Vec<EntityInfo> = world
            .entitiesNear(info.pos, cloud::FAR * 8.0)
            .into_iter()
            .filter(|other|
            {
                other.kind.as_ref().is_some_and(|kind| kind.path() == NAME)
                    && Entity { world, id: other.id }.synced::<Bond>().m_owner == player.id
            })
            .collect();

        if mine.is_empty()
        {
            for left in [true, false]
            {
                if let Ok(cloud) =
                    world.spawnEntity(&format!("aether:{NAME}"), [info.pos[0], info.pos[1] + cloud::LIFT, info.pos[2]])
                {
                    let _ = cloud.setSynced(&Bond { m_owner: player.id, m_left: left });
                }
            }

            return Use::Done(Some(ItemStack { damage: stack.damage + 1, ..stack }));
        }

        if (Entity { world, id: player.id }).flag(EntityField::Sneaking)
        {
            for cloud in mine
            {
                Entity { world, id: cloud.id }.kill();
            }
        }

        Use::Done(Some(stack))
    }
}

pub(crate) fn staff() -> impl ItemBehavior { CloudStaff }

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.entities().register(
        NAME,
        EntitySpec
        {
            template: Identifier::parse("minecraft:ghast").ok(),
            texture: Some(TEXTURE.to_owned()),
            size: Some([0.5, 0.45]),
            // 1.7 max health default, the class sets none
            health: Some(20),
            drops: Some(Vec::new()),
            sounds: super::sounds("", "", ""),
            model: Some(EntityModelSpec { shadow: 0.5, ..model::model() }),
            ..EntitySpec::default()
        },
        Some(std::rc::Rc::new(MiniCloud)),
    )?;
    Ok(())
}
