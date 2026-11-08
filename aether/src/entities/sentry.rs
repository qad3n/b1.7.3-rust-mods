use super::{closestPlayer, fraction, sounds};
use crate::models::{Program, slime_outer};
use b173_api::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

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

#[derive(Default, Clone, Copy)]
struct Search
{
    m_ticks: i32,
    m_lost: i32,
    m_jump: i32,
    m_target: Option<i32>,
    m_wasOnGround: bool,
}

thread_local! {
    static SEARCHES: RefCell<HashMap<i32, Search>> = RefCell::new(HashMap::new());
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
    victim.damage(sentry::DAMAGE, Some(entity.id));
    let [dx, dz] = [target.pos[0] - info.pos[0], target.pos[2] - info.pos[2]];
    let length: f64 = (dx * dx + dz * dz).sqrt().max(1.0e-4);
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
        let mut search: Search =
            SEARCHES.with(|searches| searches.borrow().get(&entity.id).copied()).unwrap_or_default();
        let watch: Watch = entity.synced();
        let onGround: bool = entity.flag(EntityField::OnGround);
        let player: Option<EntityInfo> = closestPlayer(world, info, sentry::SIGHT).filter(|player| {
            world
                .rayTrace(
                    [info.pos[0], info.pos[1] + 0.5, info.pos[2]],
                    [player.pos[0], player.pos[1] + 1.0, player.pos[2]],
                    false,
                )
                .is_none()
        });
        search.m_ticks += 1;
        if search.m_ticks >= sentry::SEARCH_TICKS
        {
            search.m_ticks = 0;
            match (&player, watch.m_awake)
            {
                (Some(found), _) =>
                {
                    search.m_lost = 0;
                    search.m_target = Some(found.id);
                    setAwake(entity, true);
                }
                (None, true) =>
                {
                    search.m_lost += 1;
                    if search.m_lost >= sentry::LOST_SEARCHES
                    {
                        search.m_target = None;
                        setAwake(entity, false);
                    }
                }
                (None, false) =>
                {}
            }
        }
        let target: Option<EntityInfo> =
            search.m_target.and_then(|id| Entity { world, id }.info()).filter(|target| target.alive);
        if let Some(goal) = &target
        {
            let gap: f64 = ((0..3).map(|axis| (goal.pos[axis] - info.pos[axis]).powi(2)).sum::<f64>()).sqrt();
            if gap < sentry::BLAST_REACH && world.isAuthority()
            {
                burst(entity, info, goal);
                return;
            }
            let [dx, dz] = [goal.pos[0] - info.pos[0], goal.pos[2] - info.pos[2]];
            entity.setField(EntityField::Yaw, dz.atan2(dx).to_degrees() - 90.0);
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
                world.playSound(info.pos, SLIME, 1.0, 0.8);
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
        SEARCHES.with(|searches| searches.borrow_mut().insert(entity.id, search));
    }

    fn onHurt(&self, entity: Entity, source: Option<i32>, amount: i32) -> Option<i32>
    {
        if let Some(attacker) = source
        {
            SEARCHES.with(|searches| searches.borrow_mut().entry(entity.id).or_default().m_target = Some(attacker));
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

pub fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.entities().register(
        NAME,
        EntitySpec {
            template: Identifier::parse("minecraft:monster").ok(),
            texture: Some(SLEEPING.to_owned()),
            size: Some([0.85, 0.85]),
            health: Some(10),
            drops: Some(Vec::new()),
            sounds: sounds("", SLIME, SLIME),
            model: Some(EntityModelSpec { shadow: 0.3, ..slime_outer::model() }),
            ..EntitySpec::default()
        },
        Some(std::rc::Rc::new(Sentry)),
    )?;
    Ok(())
}
