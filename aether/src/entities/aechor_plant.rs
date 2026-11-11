use super::projectile::{self, POISON_NEEDLE};
use super::{closestPlayer, creature, onAetherGrass};
use crate::ids::{Palette, item};
use crate::models::{Program, aechor_plant as model};
use b173_api::prelude::*;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::f32::consts::PI;

const NAME: &str = "aechor_plant";
const TEXTURE: &str = "aether:entity/aechor_plant/aechor_plant";

mod plant
{
    // EntityAechorPlant
    pub const MIN_SIZE: i32 = 1;
    pub const SIZE_SPREAD: i32 = 4;
    pub const BASE_WIDTH: f32 = 0.75;
    pub const WIDTH_PER_SIZE: f32 = 0.125;
    pub const BASE_HEIGHT: f32 = 0.5;
    pub const HEIGHT_PER_SIZE: f32 = 0.075;
    pub const SIGHT: f64 = 10.0;
    pub const LOOK: f64 = 5.5;
    pub const READY: i32 = 20;
    pub const RELOAD: i32 = -10;
    pub const MUZZLE: f64 = 1.0;
    pub const SHOT_SPEED: f64 = 0.285;
    pub const SHOT_LIFT: f64 = 0.05;
    pub const PETALS: u16 = 10;
    pub const STAMENS: u16 = 3;
    pub const THORNS: u16 = 4;
    // EntityAechorPlant.poisonRemaining starts at two to five
    pub const POISON_MIN: i32 = 2;
    pub const POISON_SPREAD: i32 = 4;
}

mod sway
{
    // the sinage the plant sways by, faster when hurt or hunting
    pub const HURT: f32 = 0.9;
    pub const HUNTING: f32 = 0.3;
    pub const IDLE: f32 = 0.1;
    pub const LIFT: f32 = 1.2;
    pub const STEM_REST: f32 = 1.75;
    pub const HURT_WAVE: f32 = 0.45;
    pub const HURT_DIP: f32 = 0.125;
    pub const CALM_WAVE: f32 = 0.125;
    pub const HURT_PHASE: f32 = 2.0;
    pub const HURT_STRETCH: f32 = 1.5;
    pub const BASE_SCALE: f32 = 0.625;
    pub const SCALE_DIVISOR: f32 = 6.0;
}

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct PlantState
{
    m_size: i32,
    m_hunting: bool,
}

thread_local!
{
    static RELOADS: RefCell<HashMap<i32, i32>> = RefCell::new(HashMap::new());
    static SINAGE: RefCell<HashMap<i32, (f32, i32)>> = RefCell::new(HashMap::new());
    static POISON_LEFT: RefCell<HashMap<i32, i32>> = RefCell::new(HashMap::new());
}

fn size(entity: Entity) -> i32
{
    let mut state: PlantState = entity.synced();
    if state.m_size == 0 && entity.world.isAuthority()
    {
        state.m_size = plant::MIN_SIZE + entity.world.random(plant::SIZE_SPREAD);
        state.m_hunting = false;

        let _ = entity.setSynced(&state);

        entity.setField(EntityField::Width, (plant::BASE_WIDTH + state.m_size as f32 * plant::WIDTH_PER_SIZE) as f64);
        entity
            .setField(EntityField::Height, (plant::BASE_HEIGHT + state.m_size as f32 * plant::HEIGHT_PER_SIZE) as f64);
    }

    state.m_size
}

struct AechorPlant;

#[entity_behavior]
impl EntityBehavior for AechorPlant
{
    // EntityAechorPlant.onLivingUpdate: rooted in aether grass, spits needles at players in reach
    fn onAi(&self, entity: Entity, info: &EntityInfo)
    {
        let world: World = entity.world;
        let plantSize: i32 = size(entity);
        let below: BlockPos =
            BlockPos::new(info.pos[0].floor() as i32, info.pos[1].floor() as i32 - 1, info.pos[2].floor() as i32);

        if world.block(below) != Palette::load().m_grass
        {
            entity.damage(info.health.unwrap_or(0), None);
            return;
        }

        let _ = entity.setMotion([0.0, info.motion[1], 0.0]);
        let reach: f64 = plant::LOOK + plantSize as f64 / 2.0;
        let target: Option<EntityInfo> = closestPlayer(world, info, plant::SIGHT.min(reach));
        let hunted: Option<EntityInfo> = target.clone();
        let mut reload: i32 = RELOADS.with(|reloads| reloads.borrow().get(&entity.id).copied()).unwrap_or(0);

        match hunted
        {
            Some(player) =>
            {
                let muzzle: [f64; 3] = [info.pos[0], info.pos[1] + plant::MUZZLE, info.pos[2]];
                let seen: bool =
                    world.rayTrace(muzzle, [player.pos[0], player.pos[1] + 1.0, player.pos[2]], false).is_none();

                if reload == plant::READY && seen
                {
                    let [dx, dz] = [player.pos[0] - info.pos[0], player.pos[2] - info.pos[2]];
                    let flat: f64 = (dx * dx + dz * dz + 0.1).sqrt();
                    let lift: f64 = 0.1 + flat * 0.5 + (info.pos[1] - player.pos[1]) * 0.25;
                    let aim: [f64; 3] = [muzzle[0] + dx / flat * 1.5, muzzle[1] + lift, muzzle[2] + dz / flat * 1.5];

                    world.playSound(info.pos, "random.bow", 1.0, 1.2);
                    projectile::fire(
                        world,
                        POISON_NEEDLE,
                        muzzle,
                        aim,
                        plant::SHOT_SPEED + lift * plant::SHOT_LIFT,
                        1.0,
                        Some(entity.id),
                    );
                    reload = plant::RELOAD;
                }

                if reload != plant::READY
                {
                    reload += 1;
                }
            }

            None => reload = 0,
        }

        let hunting: bool = target.is_some();
        let state: PlantState = entity.synced();
        if state.m_hunting != hunting
        {
            let _ = entity.setSynced(&PlantState { m_size: plantSize, m_hunting: hunting });
        }

        RELOADS.with(|reloads| reloads.borrow_mut().insert(entity.id, reload));
    }

    // EntityAechorPlant.interact: an empty skyroot bucket draws poison while the plant has some left
    fn onInteract(&self, entity: Entity, player: Player) -> bool
    {
        let Some(held) = player.held().filter(|held| held.item.is("skyroot_bucket") && entity.world.isAuthority())
        else
        {
            return false;
        };

        let left: i32 = POISON_LEFT.with(|left|
        {
            *left
                .borrow_mut()
                .entry(entity.id)
                .or_insert_with(|| entity.world.random(plant::POISON_SPREAD) + plant::POISON_MIN)
        });
        if left <= 0
        {
            return false;
        }

        let poison: ItemStack = ItemStack::of("skyroot_poison_bucket", 1);

        if held.count == 1
        {
            let _ = player.setHeld(Some(poison));
        }
        else
        {
            let _ = player.setHeld(Some(ItemStack { count: held.count - 1, ..held }));
            let _ = player.give(poison);
        }
        POISON_LEFT.with(|poison| poison.borrow_mut().insert(entity.id, left - 1));
        false
    }

    fn canSpawn(&self, entity: Entity, info: &EntityInfo) -> bool
    {
        crate::config::spawnRoll(entity, "aechor_plant") && onAetherGrass(info)
    }

    // AechorPlantRenderer.preRenderCallback picks the sway and size, AechorPlantModel opens petals and leaves with it
    fn pose(&self, input: &PoseInput) -> Pose
    {
        let state: PlantState = b173_api::fromBytes(&input.synced).unwrap_or_default();

        let sinage: f32 = SINAGE.with(|sinage|
        {
            let mut sinage: RefMut<HashMap<i32, (f32, i32)>> = sinage.borrow_mut();
            let tick: i32 = input.age as i32;
            let entry: &mut (f32, i32) = sinage.entry(input.entity.id).or_insert((0.0, tick));
            while entry.1 < tick
            {
                let step: f32 = if input.hurt_time > 0
                {
                    sway::HURT
                }
                else if state.m_hunting
                {
                    sway::HUNTING
                }
                else
                {
                    sway::IDLE
                };
                entry.0 = (entry.0 + step) % (2.0 * PI);
                entry.1 += 1;
            }

            entry.0
        });

        let hurt: bool = input.hurt_time > 0;
        let wave: f32 = if hurt
        {
            sinage.sin() * sway::HURT_WAVE - sway::HURT_DIP
        }
        else
        {
            sinage.sin() * sway::CALM_WAVE
        };

        let lift: f32 = if hurt
        {
            sway::STEM_REST + (sinage + sway::HURT_PHASE).sin() * sway::HURT_STRETCH
        }
        else
        {
            sway::STEM_REST
        };
        let scale: f32 = sway::BASE_SCALE + state.m_size.max(1) as f32 / sway::SCALE_DIVISOR;
        let yaw: f32 = input.head_pitch.to_radians();
        let mut body: Program = Program::new();
        body.push().translate(0.0, sway::LIFT, 0.0).scale(scale, scale, scale);

        for i in 0..plant::PETALS
        {
            let even: bool = i % 2 == 0;
            let turn: f32 = PI / 5.0 * i as f32;
            body.part(2 * i, [if even { -0.25 } else { -0.4125 } + wave, yaw + turn, 0.0], [0.0, lift, 0.0]);
            body.part(
                2 * i + 1,
                [if even { 0.1 } else { 0.2 } + wave * 0.75, yaw + PI / 10.0 + turn, 0.0],
                [0.0, lift, 0.0],
            );
        }

        for i in 0..plant::STAMENS
        {
            let rotation: [f32; 3] = [0.2 + i as f32 / 15.0 + wave * 0.4, yaw + 0.1 + 2.0 * PI / 3.0 * i as f32, 0.0];
            body.part(model::STAMEN_0 + i, rotation, [0.0, lift + wave * 2.0, 0.0]);
            body.part(model::STAMEN2_0 + i, rotation, [0.0, lift + wave * 2.0, 0.0]);
        }

        body.part(model::HEAD, [0.0, yaw, 0.0], [0.0, lift + wave * 2.0, 0.0]);
        body.part(model::STEM, [0.0, yaw, 0.0], [0.0, lift * 0.5, 0.0]);

        for i in 0..plant::THORNS
        {
            body.part(model::THORN_0 + i, [0.0, yaw, 0.0], [0.0, lift * 0.5, 0.0]);
        }

        body.pop();
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
            size: Some([plant::BASE_WIDTH + plant::WIDTH_PER_SIZE, plant::BASE_HEIGHT + plant::HEIGHT_PER_SIZE]),
            health: Some(20),
            spawn: creature(20).map(|spawn| SpawnSpec { kind: CreatureKind::Monster, ..spawn }),
            // EntityAechorPlant.dropFewItems
            drops: Some(vec![super::drop(item::AECHOR_PETAL, [2, 2])]),
            model: Some(EntityModelSpec { shadow: 0.3, ..model::model() }),
            ..EntitySpec::default()
        },
        Some(std::rc::Rc::new(AechorPlant)),
    )?;
    Ok(())
}
