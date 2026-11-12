use super::{closestPlayer, fraction, sounds, store};
use crate::models::{Program, slime_outer};
use b173_api::math::{distance, flatLength, yawTowards};
use b173_api::prelude::*;

const NAME: &str = "sentry";
const SLEEPING: &str = "aether:entity/sentry/sentry";
const AWAKE: &str = "aether:entity/sentry/sentry_lit";
const EYE: &str = "aether:entity/sentry/eye";
const SLIME: &str = "mob.slime";

mod sentry
{
    // EntitySentry
    pub const SIGHT: f64 = 8.0;
    pub const SEARCH_TICKS: i32 = 8;
    pub const LOST_SEARCHES: i32 = 4;
    // a sentry keeps a target it can see within this distance
    pub const KEEP: f64 = 16.0;
    pub const HOP_PITCH: f32 = 0.8;
    pub const HOP_PITCH_SPREAD: f32 = 0.2;
    pub const JUMP_MIN: i32 = 10;
    pub const JUMP_SPREAD: i32 = 20;
    pub const LEAP: f64 = 3.0;
    pub const BLAST_REACH: f64 = 1.2;
    pub const DAMAGE: i32 = 2;
    pub const KNOCKBACK: f64 = 4.0;
    pub const SPARKS: i32 = 40;
    pub const SCALE: f32 = 1.75;
}

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct Watch
{
    m_awake: bool,
}

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct Search
{
    m_ticks: i32,
    m_lost: i32,
    m_jump: i32,
    m_target: Option<i32>,
    m_wasOnGround: bool,
}

static SEARCHES: EntityState<Search> = EntityState::new("search");

// EntityLivingBase.canEntityBeSeen from the sentry's middle to the target's eyes
fn visible(world: World, info: &EntityInfo, target: &EntityInfo) -> bool
{
    let from: [f64; 3] = [info.pos[0], info.pos[1] + 0.5, info.pos[2]];
    let to: [f64; 3] = [target.pos[0], target.pos[1] + 1.0, target.pos[2]];
    world.rayTrace(from, to, false).is_none()
}

fn lose(search: &mut Search)
{
    search.m_lost += 1;
    if search.m_lost >= sentry::LOST_SEARCHES
    {
        search.m_lost = 0;
        search.m_target = None;
    }
}

// one search: a sleeping sentry wakes on a seen player, an awake one keeps its target while it sees it close enough
fn look(world: World, info: &EntityInfo, search: &mut Search, awake: bool, player: Option<&EntityInfo>)
{
    let current: Option<EntityInfo> =
        search.m_target.and_then(|id| Entity { world, id }.info()).filter(|target| target.alive);

    match (awake, current, player)
    {
        (_, None, Some(found)) =>
        {
            search.m_lost = 0;
            search.m_target = Some(found.id);
        }

        (true, None, None) => lose(search),

        (true, Some(target), _) =>
        {
            if visible(world, info, &target) && distance(target.pos, info.pos) < sentry::KEEP
            {
                search.m_lost = 0;
            }
            else
            {
                lose(search);
            }
        }

        (false, _, _) =>
        {}
    }
}

fn setAwake(entity: Entity, awake: bool)
{
    let watch: Watch = entity.synced();
    if watch.m_awake != awake
    {
        let _ = entity.setSynced(&Watch { m_awake: awake });
    }
}

// EntitySentry.applyEntityCollision: a sentry that reaches its target bursts and flings it
fn burst(entity: Entity, info: &EntityInfo, target: &EntityInfo)
{
    let victim: Entity = Entity { world: entity.world, id: target.id };
    crate::damage::hit(victim, sentry::DAMAGE, Some(entity.id), crate::damage::kind::MOB);

    let [dx, dz] = [target.pos[0] - info.pos[0], target.pos[2] - info.pos[2]];
    let length: f64 = flatLength(dx, dz).max(1.0e-4);
    let _ = victim.setMotion([
        dx / length * sentry::KNOCKBACK * 0.4,
        target.motion[1] * sentry::KNOCKBACK + 0.4,
        dz / length * sentry::KNOCKBACK * 0.4,
    ]);

    for _ in 0..sentry::SPARKS
    {
        let angle: f64 = fraction(entity.world) * std::f64::consts::TAU;
        entity.world.particle(
            "explode",
            [info.pos[0], info.pos[1] + 0.5, info.pos[2]],
            [-angle.sin() * 0.75, 0.125, angle.cos() * 0.75],
        );
    }

    entity.world.playSound(info.pos, "random.explode", 1.0, 1.0);
    entity.kill();
}

struct Sentry;

#[entity_behavior]
impl EntityBehavior for Sentry
{
    // EntitySentry.updateEntityActionState: wakes on a player in sight, hops at it, sleeps again when it loses it
    fn onAi(&self, entity: Entity, info: &EntityInfo)
    {
        let world: World = entity.world;
        let mut search: Search = SEARCHES.get(entity);
        let watch: Watch = entity.synced();
        let onGround: bool = entity.flag(EntityField::OnGround);

        let player: Option<EntityInfo> =
            closestPlayer(world, info, sentry::SIGHT).filter(|player| visible(world, info, player));

        // EntitySentry.updateEntityActionState searches every ninth tick, and once awake it never sleeps again
        if search.m_ticks < sentry::SEARCH_TICKS
        {
            search.m_ticks += 1;
        }
        else
        {
            search.m_ticks = 0;
            look(world, info, &mut search, watch.m_awake, player.as_ref());

            if player.is_some()
            {
                setAwake(entity, true);
            }
        }

        let target: Option<EntityInfo> =
            search.m_target.and_then(|id| Entity { world, id }.info()).filter(|target| target.alive);

        if let Some(goal) = &target
        {
            let gap: f64 = distance(goal.pos, info.pos);
            if gap < sentry::BLAST_REACH && world.isAuthority()
            {
                burst(entity, info, goal);
                return;
            }

            entity.setField(EntityField::Yaw, f64::from(yawTowards(info.pos, goal.pos)));
        }

        if entity.synced::<Watch>().m_awake
        {
            search.m_jump -= 1;
            let jump: bool = onGround && search.m_jump <= 0;
            if jump
            {
                search.m_jump = sentry::JUMP_MIN + world.random(sentry::JUMP_SPREAD);

                if target.is_some()
                {
                    search.m_jump /= 2;
                }

                entity.setField(EntityField::MoveForward, 1.0);
                entity.setField(EntityField::MoveStrafe, 0.5 - fraction(world));
                let spread: f32 = (fraction(world) - fraction(world)) as f32 * sentry::HOP_PITCH_SPREAD;
                world.playSound(info.pos, SLIME, 1.0, (spread + 1.0) * sentry::HOP_PITCH);
            }
            else if onGround
            {
                entity.setField(EntityField::MoveForward, 0.0);
                entity.setField(EntityField::MoveStrafe, 0.0);
            }
            entity.setField(EntityField::Jumping, if jump { 1.0 } else { 0.0 });

            if !onGround && search.m_wasOnGround && target.is_some()
            {
                let _ =
                    entity.setMotion([info.motion[0] * sentry::LEAP, info.motion[1], info.motion[2] * sentry::LEAP]);
            }
        }

        search.m_wasOnGround = onGround;
        store(&SEARCHES, entity, &search);
    }

    fn onHurt(&self, entity: Entity, source: Option<i32>, amount: i32) -> Option<i32>
    {
        if let Some(attacker) = source
        {
            if let Err(error) = SEARCHES.update(entity, |search| search.m_target = Some(attacker))
            {
                warn(&format!("sentry {} target: {error}", entity.id));
            }

            setAwake(entity, true);
        }

        Some(amount)
    }

    // SentryRenderer: a big slime cube, lit with a glowing eye while awake
    fn pose(&self, input: &PoseInput) -> Pose
    {
        let watch: Watch = b173_api::fromBytes(&input.synced).unwrap_or_default();
        let mut pre: Program = Program::new();
        pre.scale(sentry::SCALE, sentry::SCALE, sentry::SCALE);

        let mut body: Program = Program::new();
        body.texture(if watch.m_awake { AWAKE } else { SLEEPING }).rest(slime_outer::SLIME_BODIES);

        if watch.m_awake
        {
            body.texture(EYE).blend(true).lighting(false).rest(slime_outer::SLIME_BODIES).lighting(true).blend(false);
        }

        Pose { pre: pre.take(), model: body.take() }
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.entities().register(
        NAME,
        EntitySpec
        {
            template: Identifier::parse("minecraft:monster").ok(),
            texture: Some(SLEEPING.to_owned()),
            size: Some([0.85, 0.85]),
            // 1.7 max health default, the class sets none
            health: Some(20),
            drops: Some(Vec::new()),
            sounds: sounds("", SLIME, SLIME),
            model: Some(EntityModelSpec { shadow: 0.3, ..slime_outer::model() }),
            ..EntitySpec::default()
        },
        Some(std::rc::Rc::new(Sentry)),
    )?;
    Ok(())
}
