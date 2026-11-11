use super::fraction;
use crate::models::{DEGREES_PER_RADIAN, Phases, Program, WALK_RATE, valkyrie as model};
use crate::names::item::VICTORY_MEDAL;
use b173_api::math;
use b173_api::prelude::*;
pub(crate) use look::{AIR_SINAGE, GROUND_SINAGE};
use std::cell::RefCell;
use std::collections::HashMap;
use std::f32::consts::{PI, TAU};

const NAME: &str = "valkyrie";
const TEXTURE: &str = "aether:entity/valkyrie/valkyrie";
const HURT: &str = "random.hurt";

mod valk
{
    // EntityValkyrie
    pub const CHAT_COOLDOWN: i32 = 60;
    pub const CHASE_RANGE: f64 = 16.0;
    pub const LUNGE: f64 = 0.25;
    // a fall step between these is plain gravity, which the wings soften
    pub const GRAVITY_LOW: f64 = 0.07;
    pub const GRAVITY_HIGH: f64 = 0.09;
    pub const LIFT: f64 = 0.055;
    pub const TERMINAL: f64 = -0.275;
    pub const MEDALS_PROUD: i32 = 10;
    pub const MEDALS_FEW: i32 = 5;
    pub const LINES: i32 = 3;
    pub const POOF: i32 = 20;
    pub const EYES: f64 = 1.4;
    pub const DAMAGE: i32 = 7;
}

pub(crate) mod look
{
    // ValkyrieModel.setRotationAngles
    pub(crate) const ARM_SWING: f32 = 0.5;
    pub(crate) const LEG_SWING: f32 = 1.4;
    pub(crate) const ARM_SPREAD: f32 = 0.05;
    pub(crate) const BREATH: f32 = 0.09;
    pub(crate) const SWAY: f32 = 0.067;
    pub(crate) const RIDE_ARM: f32 = -0.6283185;
    pub(crate) const RIDE_LEG: f32 = -1.256637;
    pub(crate) const RIDE_SPLAY: f32 = 0.3141593;
    pub(crate) const SWING_TWIST: f32 = 0.2;
    pub(crate) const SWING_LIFT: f32 = 1.2;
    pub(crate) const SWING_ROLL: f32 = -0.4;
    pub(crate) const HEAD_TILT: f32 = 0.7;
    pub(crate) const SWING_REACH: f32 = 0.75;
    pub(crate) const WING_YAW: f32 = 0.2;
    pub(crate) const WING_ROLL: f32 = 0.125;
    pub(crate) const WING_BEAT: f64 = 6.0;
    pub(crate) const GROUND_FLAP: f64 = 8.0;
    pub(crate) const AIR_FLAP: f64 = 3.0;
    pub(crate) const SKIRT: f32 = 0.2;
    pub(crate) const SKIRT_KICK: f32 = 0.3;
    // EntityValkyrie.sinage steps
    pub(crate) const AIR_SINAGE: f32 = 0.75;
    pub(crate) const GROUND_SINAGE: f32 = 0.15;
}

#[derive(Default, Clone, Copy)]
struct Mood
{
    m_chatTime: i32,
    m_lastMotionY: f64,
}

thread_local!
{
    static MOODS: RefCell<HashMap<i32, Mood>> = RefCell::new(HashMap::new());
    static SINAGE: Phases = Phases::default();
}

fn mood(entity: Entity) -> Mood { MOODS.with(|moods| moods.borrow().get(&entity.id).copied()).unwrap_or_default() }

fn setMood(entity: Entity, mood: Mood) { MOODS.with(|moods| moods.borrow_mut().insert(entity.id, mood)); }

// EntityValkyrie.chatItUp: one line per three seconds, picked from a numbered lang group
fn chatItUp(entity: Entity, player: Player, group: &str)
{
    let mut current: Mood = mood(entity);
    if current.m_chatTime > 0
    {
        return;
    }

    let line: i32 = valk::LINES - entity.world.random(valk::LINES);
    let name: String = player.info().and_then(|info| info.player).unwrap_or_default();
    player.chat(&translate(&format!("aether.gui.valkyrie.dialog.{group}{line}")).replace("%s", &name));
    current.m_chatTime = valk::CHAT_COOLDOWN;
    setMood(entity, current);
}

fn says(entity: Entity, player: Player, key: &str)
{
    let mut current: Mood = mood(entity);
    if current.m_chatTime <= 0
    {
        player.chat(&translate(&format!("aether.gui.valkyrie.dialog.{key}")));
        current.m_chatTime = valk::CHAT_COOLDOWN;
        setMood(entity, current);
    }
}

pub(crate) fn target(entity: Entity) -> Option<EntityInfo>
{
    let id: i32 = entity.field(EntityField::TargetId) as i32;
    (id >= 0).then(|| Entity { world: entity.world, id }.info()).flatten()
}

pub(crate) fn canSee(world: World, from: &EntityInfo, to: &EntityInfo) -> bool
{
    let eyes = |info: &EntityInfo| [info.pos[0], info.pos[1] + valk::EYES, info.pos[2]];
    world.rayTrace(eyes(from), eyes(to), false).is_none()
}

// EntityValkyrie.onUpdate: lunges at the target from the top of a jump and glides down on its wings
pub(crate) fn glide(entity: Entity, info: &EntityInfo, lastMotionY: &mut f64)
{
    let onGround: bool = entity.flag(EntityField::OnGround);
    let mut motion: [f64; 3] = info.motion;

    if !onGround && let Some(goal) = target(entity)
    {
        let gap: f64 = math::distance(goal.pos, info.pos);
        if *lastMotionY >= 0.0 && motion[1] < 0.0 && gap <= valk::CHASE_RANGE && canSee(entity.world, info, &goal)
        {
            let angle: f64 = (goal.pos[0] - info.pos[0]).atan2(goal.pos[2] - info.pos[2]);
            motion[0] = angle.sin() * valk::LUNGE;
            motion[2] = angle.cos() * valk::LUNGE;
        }
    }

    let fall: f64 = (motion[1] - *lastMotionY).abs();
    if !onGround && fall > valk::GRAVITY_LOW && fall < valk::GRAVITY_HIGH
    {
        motion[1] = (motion[1] + valk::LIFT).max(valk::TERMINAL);
    }

    if motion != info.motion
    {
        let _ = entity.setMotion(motion);
    }
    *lastMotionY = motion[1];
}

struct Valkyrie;

#[entity_behavior]
impl EntityBehavior for Valkyrie
{
    fn onTick(&self, entity: Entity, info: &EntityInfo)
    {
        if !entity.world.isAuthority()
        {
            return;
        }

        entity.setField(EntityField::AttackStrength, valk::DAMAGE as f64);

        let mut current: Mood = mood(entity);
        glide(entity, info, &mut current.m_lastMotionY);
        current.m_chatTime -= 1;
        setMood(entity, current);

        if let Some(goal) = target(entity).filter(|goal| !goal.alive && goal.player.is_some())
        {
            chatItUp(entity, Player { id: goal.id }, "playerdead.");
            entity.setField(EntityField::TargetId, -1.0);
        }
    }

    // EntityValkyrie.interact: turns to the player and talks, prouder words for a pile of medals
    fn onInteract(&self, entity: Entity, player: Player) -> bool
    {
        if let (Some(info), Some(seen)) = (entity.info(), player.info())
        {
            entity.setField(EntityField::Yaw, f64::from(math::yawTowards(info.pos, seen.pos)));
        }

        if !entity.world.isAuthority()
        {
            return true;
        }

        let medals: i32 = player.held().filter(|held| held.item.is(VICTORY_MEDAL)).map_or(0, |held| held.count);
        match medals
        {
            0 => chatItUp(entity, player, ""),
            count if count >= valk::MEDALS_PROUD => says(entity, player, "medal.1"),
            count if count >= valk::MEDALS_FEW => says(entity, player, "medal.2"),
            _ => says(entity, player, "medal.3"),
        }

        true
    }

    // EntityValkyrie.attackEntityFrom: only players hurt it, anything else is shrugged off and the fire put out
    fn onHurt(&self, entity: Entity, source: Option<i32>, amount: i32) -> Option<i32>
    {
        let attacker: Option<EntityInfo> = source.and_then(|id| Entity { world: entity.world, id }.info());
        let Some(player) = attacker.filter(|attacker| attacker.player.is_some())
        else
        {
            entity.setField(EntityField::FireTicks, 0.0);
            return None;
        };

        if target(entity).is_none()
        {
            let mut current: Mood = mood(entity);
            current.m_chatTime = 0;
            setMood(entity, current);
            chatItUp(entity, Player { id: player.id }, "attack.");
        }

        Some(amount)
    }

    // a beaten valkyrie gives up with a line and vanishes in a puff, leaving its medal
    fn onDeath(&self, entity: Entity, source: Option<i32>)
    {
        let Some(info) = entity.info().filter(|_| entity.world.isAuthority())
        else
        {
            return;
        };

        if let Some(killer) =
            source.and_then(|id| Entity { world: entity.world, id }.info()).filter(|killer| killer.player.is_some())
        {
            let mut current: Mood = mood(entity);
            current.m_chatTime = 0;
            setMood(entity, current);
            chatItUp(entity, Player { id: killer.id }, "defeated.");
        }

        for _ in 0..valk::POOF
        {
            let spot: [f64; 3] = [
                info.pos[0] + fraction(entity.world) - 0.5,
                info.pos[1] + fraction(entity.world) * 2.0,
                info.pos[2] + fraction(entity.world) - 0.5,
            ];
            entity.world.particle("explode", spot, [0.0; 3]);
        }

        MOODS.with(|moods| moods.borrow_mut().remove(&entity.id));
        entity.kill();
    }

    fn pose(&self, input: &PoseInput) -> Pose
    {
        let step: f32 = if input.on_ground
        {
            look::GROUND_SINAGE
        }
        else
        {
            look::AIR_SINAGE
        };
        let sinage: f32 = SINAGE.with(|phases| phases.advance(input.entity.id, input.age as i32, step, TAU));
        Pose { pre: Vec::new(), model: valkyriePose(input, sinage, 0.0).take() }
    }
}

// ValkyrieModel.setRotationAngles and render, shared with the valkyrie queen
pub(crate) fn valkyriePose(input: &PoseInput, sinage: f32, swing: f32) -> Program
{
    let pivot = |part: u16| model::PIVOTS[part as usize];
    let walk: f32 = input.walk * WALK_RATE;
    let speed: f32 = input.walk_speed;
    let head: [f32; 3] = [input.head_pitch / DEGREES_PER_RADIAN, input.head_yaw / DEGREES_PER_RADIAN, 0.0];
    let mut rightArm: [f32; 3] = [math::cos(walk + PI) * 2.0 * speed * look::ARM_SWING, 0.0, 0.0];
    let mut leftArm: [f32; 3] = [math::cos(walk) * 2.0 * speed * look::ARM_SWING, 0.0, -look::ARM_SPREAD];
    let mut rightLeg: [f32; 3] = [math::cos(walk) * look::LEG_SWING * speed, 0.0, 0.0];
    let mut leftLeg: [f32; 3] = [math::cos(walk + PI) * look::LEG_SWING * speed, 0.0, 0.0];

    if input.riding
    {
        rightArm[0] += look::RIDE_ARM;
        leftArm[0] += look::RIDE_ARM;
        rightLeg = [look::RIDE_LEG, look::RIDE_SPLAY, 0.0];
        leftLeg = [look::RIDE_LEG, -look::RIDE_SPLAY, 0.0];
    }

    let twist: f32 = math::sin(swing.sqrt() * PI * 2.0) * look::SWING_TWIST;
    rightArm[1] = twist;
    leftArm[1] = twist;
    leftArm[0] += twist;

    let eased: f32 = 1.0 - (1.0 - swing).powi(4);
    let reach: f32 = math::sin(swing * PI) * -(head[0] - look::HEAD_TILT) * look::SWING_REACH;
    rightArm[0] -= math::sin(eased * PI) * look::SWING_LIFT + reach;
    rightArm[1] += twist * 2.0;
    rightArm[2] = math::sin(swing * PI) * look::SWING_ROLL;

    let breath: f32 = math::cos(input.age * look::BREATH) * look::ARM_SPREAD + look::ARM_SPREAD;
    rightArm[2] += breath;
    leftArm[2] -= breath;
    rightArm[0] += math::sin(input.age * look::SWAY) * look::ARM_SPREAD;
    leftArm[0] -= math::sin(input.age * look::SWAY) * look::ARM_SPREAD;

    let flap: f64 = if input.on_ground
    {
        look::GROUND_FLAP
    }
    else
    {
        look::AIR_FLAP
    };
    let beat: f32 = ((sinage as f64).sin() / look::WING_BEAT) as f32;
    let roll: f32 = ((sinage as f64).cos() / flap) as f32;
    let mut skirt: [[f32; 3]; 6] = [
        [-look::SKIRT, 0.0, 0.0],
        [-look::SKIRT, 0.0, 0.0],
        [look::SKIRT, 0.0, 0.0],
        [look::SKIRT, 0.0, 0.0],
        [0.0, 0.0, look::SKIRT],
        [0.0, 0.0, -look::SKIRT],
    ];

    if leftLeg[0] < -look::SKIRT_KICK
    {
        skirt[1][0] += leftLeg[0] + look::SKIRT_KICK;
        skirt[2][0] -= leftLeg[0] + look::SKIRT_KICK;
    }

    if leftLeg[0] > look::SKIRT_KICK
    {
        skirt[3][0] += leftLeg[0] - look::SKIRT_KICK;
        skirt[0][0] -= leftLeg[0] - look::SKIRT_KICK;
    }

    let mut body: Program = Program::new();
    body.part(model::HEAD, head, pivot(model::HEAD));
    body.part(model::BODY, [0.0, twist, 0.0], pivot(model::BODY));
    body.part(model::RIGHT_ARM, rightArm, pivot(model::RIGHT_ARM));
    body.part(model::LEFT_ARM, leftArm, pivot(model::LEFT_ARM));
    body.part(model::RIGHT_LEG, rightLeg, pivot(model::RIGHT_LEG));
    body.part(model::LEFT_LEG, leftLeg, pivot(model::LEFT_LEG));
    body.part(model::BIPED_BODY2, [0.0, twist, 0.0], pivot(model::BIPED_BODY2));
    body.part(model::BIPED_RIGHT_ARM2, rightArm, pivot(model::BIPED_RIGHT_ARM2));
    body.part(model::BIPED_LEFT_ARM2, [leftArm[0], 0.0, leftArm[2]], pivot(model::BIPED_LEFT_ARM2));
    body.part(model::WING_LEFT, [0.0, -look::WING_YAW + beat, -look::WING_ROLL + roll], pivot(model::WING_LEFT));
    body.part(model::WING_RIGHT, [0.0, look::WING_YAW - beat, look::WING_ROLL - roll], pivot(model::WING_RIGHT));

    for sword in model::SWORD_0..=model::SWORD_4
    {
        body.part(sword, rightArm, pivot(sword));
    }

    for (index, angles) in skirt.iter().enumerate()
    {
        let piece: u16 = model::SKIRT_0 + index as u16;
        body.part(piece, *angles, pivot(piece));
    }

    for strand in model::STRAND_0..=model::STRAND_21
    {
        body.part(strand, head, pivot(strand));
    }

    body.blend(true);

    for halo in model::HALO_0..=model::HALO_3
    {
        body.part(halo, head, pivot(halo));
    }

    body.blend(false);
    body
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.entities().register(
        NAME,
        EntitySpec
        {
            template: Identifier::parse("minecraft:neutral_monster").ok(),
            texture: Some(TEXTURE.to_owned()),
            size: Some([0.8, 1.6]),
            health: Some(50),
            speed: Some(0.7),
            drops: Some(vec![super::drop(VICTORY_MEDAL, [1, 1])]),
            sounds: super::sounds("", HURT, HURT),
            model: Some(EntityModelSpec { shadow: 0.3, ..model::model() }),
            ..EntitySpec::default()
        },
        Some(std::rc::Rc::new(Valkyrie)),
    )?;
    Ok(())
}
