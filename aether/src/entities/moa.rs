use super::mount::{SADDLE, steering};
use super::{bird, creature};
use crate::models::{Program, combine, moa as model};
use b173_api::prelude::*;

const NAME: &str = "moa";
const PETAL: &str = "aechor_petal";
const NATURE_STAFF: &str = "nature_staff";
const EGG_TEXTURE: &str = "aether:item/misc/egg/moa_egg";
const SADDLE_TEXTURE: &str = "aether:entity/moa/moa_saddle";
const BLACK_SADDLE_TEXTURE: &str = "aether:entity/moa/black_moa_saddle";
const PARTS: u16 = 11;

// AetherMoaTypes: name, egg tint, air jumps, mounted speed
struct MoaType
{
    m_name: &'static str,
    m_color: u32,
    m_jumps: i32,
    m_speed: f32,
}

const TYPES: [MoaType; 4] = [
    MoaType { m_name: "blue", m_color: 0x7777FF, m_jumps: 3, m_speed: 0.3 },
    MoaType { m_name: "orange", m_color: 0xF3C388, m_jumps: 2, m_speed: 0.6 },
    MoaType { m_name: "white", m_color: 0xFFFFFF, m_jumps: 4, m_speed: 0.3 },
    MoaType { m_name: "black", m_color: 0x222222, m_jumps: 8, m_speed: 0.3 },
];
const BLACK: u8 = 3;

mod moa
{
    // EntityMoa.onMountedJump and onUpdate
    pub const GROUND_JUMP: f32 = 0.89;
    pub const AIR_JUMP: f32 = 0.7;
    pub const EGG_SECONDS: i32 = 775;
    pub const EGG_SPREAD: i32 = 50;
    pub const HUNGER_SECONDS: i32 = 40;
    pub const FEEDS_TO_GROW: u8 = 3;
    pub const BABY_SCALE: f32 = 1.0;
    pub const ADULT_SCALE: f32 = 1.8;
    pub const SADDLE_LIFT: f32 = 0.02;
    pub const TICKS_PER_SECOND: i32 = 20;
}

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct MoaState
{
    m_ready: bool,
    m_kind: u8,
    m_saddled: bool,
    m_playerGrown: bool,
    m_baby: bool,
    m_hungry: bool,
    m_fed: u8,
    m_sitting: bool,
    m_eggSeconds: i32,
    m_hungerSeconds: i32,
}

fn state(entity: Entity) -> MoaState
{
    let mut state: MoaState = entity.synced();
    if !state.m_ready && entity.world.isAuthority()
    {
        state = MoaState {
            m_ready: true,
            m_kind: entity.world.random(TYPES.len() as i32) as u8,
            m_eggSeconds: moa::EGG_SECONDS + entity.world.random(moa::EGG_SPREAD),
            ..state
        };
        let _ = entity.setSynced(&state);
    }
    state
}

fn moaType(state: &MoaState) -> &'static MoaType { &TYPES[(state.m_kind as usize).min(TYPES.len() - 1)] }

fn eggName(kind: &MoaType) -> String { format!("{}_moa_egg", kind.m_name) }

// TileEntityIncubator.updateEntity: a hatched moa is a player grown baby of the egg's type
pub fn hatch(world: World, egg: &ItemStack, pos: [f64; 3]) -> bool
{
    let Some(kind) = TYPES.iter().position(|kind| egg.item.is(&eggName(kind)))
    else
    {
        return false;
    };
    let Ok(moa) = world.spawnEntity(&format!("aether:{NAME}"), pos)
    else
    {
        return false;
    };
    let state: MoaState = MoaState {
        m_ready: true,
        m_kind: kind as u8,
        m_playerGrown: true,
        m_baby: true,
        m_eggSeconds: moa::EGG_SECONDS + world.random(moa::EGG_SPREAD),
        ..MoaState::default()
    };
    moa.setSynced(&state).is_ok()
}

pub fn isEgg(stack: &ItemStack) -> bool { TYPES.iter().any(|kind| stack.item.is(&eggName(kind))) }

struct Moa;

#[entity_behavior]
impl EntityBehavior for Moa
{
    // EntityMoa.onUpdate: hunger, eggs, and the type's jumps and speed for the rider
    fn onTick(&self, entity: Entity, info: &EntityInfo)
    {
        entity.setField(EntityField::FallDistance, 0.0);
        let mut current: MoaState = state(entity);
        let kind: &MoaType = moaType(&current);
        entity.setField(EntityField::AirJumps, kind.m_jumps as f64);
        entity.setField(EntityField::MountSpeed, kind.m_speed as f64);
        if !entity.world.isAuthority() || entity.field(EntityField::TicksExisted) as i32 % moa::TICKS_PER_SECOND != 0
        {
            return;
        }
        let mut changed: bool = false;
        if current.m_hungerSeconds > 0
        {
            current.m_hungerSeconds -= 1;
        }
        else if !current.m_hungry
        {
            current.m_hungry = true;
            changed = true;
        }
        if !current.m_baby && entity.rider().is_none()
        {
            current.m_eggSeconds -= 1;
            if current.m_eggSeconds <= 0
            {
                entity.world.playSound(info.pos, "mob.chickenplop", 1.0, 1.0);
                let _ = entity.world.spawnItem(info.pos, ItemStack::of(eggName(kind).as_str(), 1));
                current.m_eggSeconds = moa::EGG_SECONDS + entity.world.random(moa::EGG_SPREAD);
            }
            changed = true;
        }
        if changed
        {
            let _ = entity.setSynced(&current);
        }
    }

    // EntityMoa.interact and EntitySaddleMount.interact for player raised moas
    fn onInteract(&self, entity: Entity, player: Player) -> bool
    {
        let mut current: MoaState = state(entity);
        let held: Option<ItemStack> = player.held();
        if current.m_playerGrown
            && let Some(held) = &held
        {
            if current.m_baby && current.m_hungry && current.m_fed < moa::FEEDS_TO_GROW && held.item.is(PETAL)
            {
                current.m_fed += 1;
                current.m_hungry = false;
                current.m_hungerSeconds = moa::HUNGER_SECONDS + entity.world.random(moa::HUNGER_SECONDS);
                current.m_baby = current.m_fed < moa::FEEDS_TO_GROW;
                let left: i32 = held.count - 1;
                let _ = player.setHeld((left > 0).then_some(ItemStack { count: left, ..held.clone() }));
                let _ = entity.setSynced(&current);
                return true;
            }
            if held.item.is(NATURE_STAFF)
            {
                current.m_sitting = !current.m_sitting;
                let _ = entity.setSynced(&current);
                let _ = player.setHeld(Some(ItemStack { damage: held.damage + 2, ..held.clone() }));
                return true;
            }
        }
        if current.m_baby || !current.m_playerGrown
        {
            return false;
        }
        if !current.m_saddled
        {
            let Some(held) = held.filter(|held| held.item.is(SADDLE))
            else
            {
                return false;
            };
            current.m_saddled = true;
            let _ = entity.setSynced(&current);
            let left: i32 = held.count - 1;
            let _ = player.setHeld((left > 0).then_some(ItemStack { count: left, ..held }));
            return true;
        }
        if entity.rider().is_none() && entity.world.isAuthority()
        {
            let _ = Entity { world: entity.world, id: player.id }.mount(Some(entity.id));
        }
        true
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

    fn onDeath(&self, entity: Entity, _source: Option<i32>)
    {
        let current: MoaState = state(entity);
        if let Some(info) = entity.info()
            && current.m_saddled
            && entity.world.isAuthority()
        {
            let _ = entity.world.spawnItem(info.pos, ItemStack::of(SADDLE, 1));
        }
    }

    fn canSpawn(&self, entity: Entity, info: &EntityInfo) -> bool
    {
        crate::config::spawnRoll(entity, "moa") && super::onAetherGrass(info)
    }

    // MoaRenderer: the type's sheet, the saddle layer, babies at full size and adults at 1.8
    fn pose(&self, input: &PoseInput) -> Pose
    {
        let current: MoaState = b173_api::fromBytes(&input.synced).unwrap_or_default();
        let kind: &MoaType = moaType(&current);
        let scale: f32 = if current.m_baby
        {
            moa::BABY_SCALE
        }
        else
        {
            moa::ADULT_SCALE
        };
        let mut pre: Program = Program::new();
        pre.scale(scale, scale, scale);
        let swing: f32 = bird::wingSwing(input);
        let mut body: Program = Program::new();
        body.texture(&format!("aether:entity/moa/moa_{}", kind.m_name));
        bird::pose(&mut body, 0, &model::PIVOTS, &model::ROTATIONS, input, current.m_sitting, swing);
        if current.m_saddled
        {
            body.texture(
                if current.m_kind == BLACK
                {
                    BLACK_SADDLE_TEXTURE
                }
                else
                {
                    SADDLE_TEXTURE
                },
            );
            body.push().translate(0.0, moa::SADDLE_LIFT, 0.0);
            bird::pose(&mut body, PARTS, &model::PIVOTS, &model::ROTATIONS, input, current.m_sitting, swing);
            body.pop();
        }
        Pose { pre: pre.take(), model: body.take() }
    }
}

pub fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for kind in &TYPES
    {
        ctx.items()
            .register(&eggName(kind), ItemSpecBuilder::new().texture(EGG_TEXTURE).maxStack(1).color(kind.m_color))?;
    }
    ctx.entities().register(
        NAME,
        EntitySpec {
            template: Identifier::parse("minecraft:pig").ok(),
            texture: Some("aether:entity/moa/moa_blue".to_owned()),
            size: Some([1.0, 2.0]),
            health: Some(35),
            spawn: creature(5),
            drops: Some(vec![super::drop("minecraft:feather", [0, 2])]),
            sounds: super::sounds("aether.aemob.moa.say", "aether.aemob.moa.say", "aether.aemob.moa.say"),
            model: Some(EntityModelSpec {
                shadow: 1.0,
                ..combine(vec![model::model(), crate::models::moa_saddle::model()])
            }),
            steering: Some(SteeringSpec {
                jump: moa::GROUND_JUMP,
                air_jump: moa::AIR_JUMP,
                air_jump_falling: true,
                glide: 1.0,
                ..steering(TYPES[0].m_jumps).unwrap_or_default()
            }),
            ..EntitySpec::default()
        },
        Some(std::rc::Rc::new(Moa)),
    )?;
    Ok(())
}
