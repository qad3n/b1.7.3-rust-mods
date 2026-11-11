use super::{closestPlayer, fraction, monster, sounds};
use crate::ids::{block, item};
use crate::models::{Program, combine, slime_inner, slime_outer};
use b173_api::prelude::*;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;

const NAME: &str = "swet";
const TEXTURES: [&str; 2] = ["aether:entity/swet/swet_blue", "aether:entity/swet/swet_golden"];
const GOLDEN: u8 = 1;
const SLIME: &str = "mob.slime";

mod hunt
{
    // EntitySwet.updateEntityActionState
    pub const SIGHT: f64 = 25.0;
    pub const DELAY_MIN: i32 = 10;
    pub const DELAY_SPREAD: i32 = 20;
    pub const HUNTING_DIVISOR: i32 = 3;
    pub const CATCH_REACH: f64 = 1.0;
    // the bounces a caught player rides before the swet bursts
    pub const BOUNCES: [f64; 3] = [0.6499999940395356, 0.7499999880790711, 1.55];
    pub const SPLASHES: i32 = 50;
    pub const SPLASH_REACH: f64 = 0.25;
    pub const SPLASH_SPREAD: f64 = 0.5;
    pub const SPLASH_LIFT: f64 = 1.25;
    pub const SPLASH_UP: f64 = 4.0;
    pub const SPLASH_OUT: f64 = 1.5;
    pub const DRIP_HEIGHT: f64 = 0.55;
    pub const BOUNCE_PUSH: f64 = 0.2;
    pub const JUMP_PITCH: f32 = 0.8;
}

mod squash
{
    // EntitySwet.onUpdate client shape while jumping
    pub const STEP: f32 = 0.25;
    pub const JUMP_HEIGHT: f32 = 1.425;
    pub const JUMP_WIDTH: f32 = 0.875;
    pub const STRETCH_AFTER: f32 = 3.0;
    pub const STRETCH_LIMIT: f32 = 10.0;
    pub const STRETCH: f32 = 0.05;
    pub const SIZE: f32 = 1.5;
    pub const RIDER_SIZE: f32 = 0.75;
}

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct SwetState
{
    m_ready: bool,
    m_kind: u8,
}

#[derive(Default, Clone, Copy)]
struct Hunt
{
    m_delay: i32,
    m_bounces: usize,
    m_wasOnGround: bool,
}

#[derive(Clone, Copy)]
struct Shape
{
    m_height: f32,
    m_width: f32,
    m_airTicks: f32,
    m_lastTick: i32,
}

thread_local!
{
    static HUNTS: RefCell<HashMap<i32, Hunt>> = RefCell::new(HashMap::new());
    static SHAPES: RefCell<HashMap<i32, Shape>> = RefCell::new(HashMap::new());
}

fn state(entity: Entity) -> SwetState
{
    let mut state: SwetState = entity.synced();
    if !state.m_ready && entity.world.isAuthority()
    {
        state = SwetState { m_ready: true, m_kind: entity.world.random(TEXTURES.len() as i32) as u8 };
        let _ = entity.setSynced(&state);
    }

    state
}

// EntitySwet.dissolveSwet: a burst of splashes and the swet is gone
fn dissolve(entity: Entity, info: &EntityInfo)
{
    for _ in 0..hunt::SPLASHES
    {
        let angle: f64 = fraction(entity.world) * std::f64::consts::TAU;
        let reach: f64 = hunt::SPLASH_REACH + fraction(entity.world) * hunt::SPLASH_SPREAD;
        let offset: [f64; 2] = [angle.sin() * reach, angle.cos() * reach];
        entity.world.particle(
            "splash",
            [info.pos[0] + offset[0], info.pos[1] + hunt::SPLASH_LIFT, info.pos[2] + offset[1]],
            [offset[0] * hunt::SPLASH_OUT, hunt::SPLASH_UP, offset[1] * hunt::SPLASH_OUT],
        );
    }

    entity.world.playSound(info.pos, SLIME, 1.0, 1.0);

    if let Some(rider) = entity.rider()
    {
        let _ = Entity { world: entity.world, id: rider }.mount(None);
    }

    entity.kill();
}

struct Swet;

#[entity_behavior]
impl EntityBehavior for Swet
{
    fn canSpawn(&self, entity: Entity, _info: &EntityInfo) -> bool { crate::config::spawnRoll(entity, "swet") }

    fn onTick(&self, entity: Entity, info: &EntityInfo)
    {
        state(entity);

        if entity.world.isAuthority() && entity.flag(EntityField::InWater)
        {
            dissolve(entity, info);
            return;
        }

        if entity.rider().is_none()
        {
            entity.world.particle("splash", [info.pos[0], info.pos[1] + hunt::DRIP_HEIGHT, info.pos[2]], [0.0; 3]);
        }
    }

    // EntitySwet.updateEntityActionState: hop toward a player, swallow it, bounce three times and burst
    fn onAi(&self, entity: Entity, info: &EntityInfo)
    {
        let world: World = entity.world;
        let onGround: bool = entity.flag(EntityField::OnGround);
        let mut hunt: Hunt = HUNTS.with(|hunts| hunts.borrow().get(&entity.id).copied()).unwrap_or_default();

        if entity.rider().is_some()
        {
            if onGround && !hunt.m_wasOnGround
            {
                world.playSound(info.pos, SLIME, 1.0, 1.0);

                match hunt::BOUNCES.get(hunt.m_bounces)
                {
                    Some(bounce) =>
                    {
                        let heading: f64 = fraction(world) * std::f64::consts::TAU;
                        let push: f64 = hunt::BOUNCE_PUSH * (1 + hunt.m_bounces) as f64;
                        let _ =
                            entity.setMotion([-heading.sin() * push, info.motion[1] + bounce, heading.cos() * push]);
                        hunt.m_bounces += 1;
                    }

                    None =>
                    {
                        dissolve(entity, info);
                        return;
                    }
                }
            }
        }
        else
        {
            let target: Option<EntityInfo> = closestPlayer(world, info, hunt::SIGHT);
            if let Some(player) = &target
            {
                let [dx, dz] = [player.pos[0] - info.pos[0], player.pos[2] - info.pos[2]];
                entity.setField(EntityField::Yaw, dz.atan2(dx).to_degrees() - 90.0);

                let gap: f64 = ((0..3).map(|axis| (player.pos[axis] - info.pos[axis]).powi(2)).sum::<f64>()).sqrt();
                if gap < hunt::CATCH_REACH + 0.5 && world.isAuthority()
                {
                    world.playSound(info.pos, "mob.attack", 0.5, 1.0);

                    let _ = Entity { world, id: player.id }.mount(Some(entity.id));

                    hunt.m_bounces = 0;
                }
            }

            hunt.m_delay -= 1;
            let jump: bool = onGround && hunt.m_delay <= 0;
            if jump
            {
                hunt.m_delay = hunt::DELAY_MIN + world.random(hunt::DELAY_SPREAD);

                if target.is_some()
                {
                    hunt.m_delay /= hunt::HUNTING_DIVISOR;
                }

                world.playSound(info.pos, SLIME, 1.0, hunt::JUMP_PITCH);
                entity.setField(EntityField::MoveStrafe, 1.0 - fraction(world) * 2.0);
                entity.setField(EntityField::MoveForward, 1.0);
            }
            else if onGround
            {
                entity.setField(EntityField::MoveStrafe, 0.0);
                entity.setField(EntityField::MoveForward, 0.0);
            }
            entity.setField(EntityField::Jumping, if jump { 1.0 } else { 0.0 });
        }
        hunt.m_wasOnGround = onGround;
        HUNTS.with(|hunts| hunts.borrow_mut().insert(entity.id, hunt));
    }

    fn onDeath(&self, entity: Entity, _source: Option<i32>)
    {
        let current: SwetState = state(entity);

        let Some(info) = entity.info().filter(|_| entity.world.isAuthority())
        else
        {
            return;
        };
        let count: i32 = entity.world.random(2);
        if count == 0
        {
            return;
        }

        if current.m_kind == GOLDEN
        {
            let _ = entity.world.spawnItem(info.pos, ItemStack::of("minecraft:glowstone", count));
        }
        else
        {
            let _ = entity.world.spawnItem(info.pos, ItemStack::of(block::BLUE_AERCLOUD, count));
            let _ = entity.world.spawnItem(info.pos, ItemStack::of(item::SWET_BALL, count));
        }
    }

    // SwetRenderer: the inner slime, then the translucent gel stretched by the jump and the swallowed player
    fn pose(&self, input: &PoseInput) -> Pose
    {
        let current: SwetState = b173_api::fromBytes(&input.synced).unwrap_or_default();

        let (height, width) = SHAPES.with(|shapes|
        {
            let mut shapes: RefMut<HashMap<i32, Shape>> = shapes.borrow_mut();
            let tick: i32 = input.age as i32;

            let shape: &mut Shape = shapes.entry(input.entity.id).or_insert(Shape
            {
                m_height: 1.0,
                m_width: 1.0,
                m_airTicks: 0.0,
                m_lastTick: tick,
            });
            while shape.m_lastTick < tick
            {
                if input.on_ground
                {
                    shape.m_airTicks = 0.0;
                    shape.m_height = (shape.m_height + squash::STEP).min(1.0);
                    shape.m_width = (shape.m_width - squash::STEP).max(1.0);
                }
                else
                {
                    shape.m_airTicks += 1.0;
                    shape.m_height = squash::JUMP_HEIGHT;
                    shape.m_width = squash::JUMP_WIDTH;

                    if shape.m_airTicks > squash::STRETCH_AFTER
                    {
                        let stretch: f32 = shape.m_airTicks.min(squash::STRETCH_LIMIT) * squash::STRETCH;
                        shape.m_height -= stretch;
                        shape.m_width += stretch;
                    }
                }
                shape.m_lastTick += 1;
            }

            (shape.m_height, shape.m_width)
        });

        let size: f32 = squash::SIZE + input.rider_size * squash::RIDER_SIZE;
        let mut pre: Program = Program::new();
        pre.scale(width * size, height * size, width * size);

        let mut body: Program = Program::new();
        body.texture(TEXTURES[(current.m_kind as usize).min(TEXTURES.len() - 1)]);
        for part in [
            slime_inner::SLIME_BODIES,
            slime_inner::SLIME_RIGHT_EYE,
            slime_inner::SLIME_LEFT_EYE,
            slime_inner::SLIME_MOUTH,
        ]
        {
            body.rest(part);
        }
        let outer: u16 = slime_inner::PIVOTS.len() as u16 + slime_outer::SLIME_BODIES;
        body.blend(true).rest(outer).blend(false).color(1.0, 1.0, 1.0, 1.0);
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
            texture: Some(TEXTURES[0].to_owned()),
            size: Some([0.8, 0.8]),
            health: Some(25),
            spawn: monster(10),
            drops: Some(Vec::new()),
            sounds: sounds(SLIME, SLIME, SLIME),
            model: Some(EntityModelSpec { shadow: 0.3, ..combine(vec![slime_inner::model(), slime_outer::model()]) }),
            ..EntitySpec::default()
        },
        Some(std::rc::Rc::new(Swet)),
    )?;
    Ok(())
}
