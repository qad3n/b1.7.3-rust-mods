use super::{creature, onAetherGrass};
use crate::ids::Palette;
use crate::models::{Program, combine, quadruped, quadrupedPose, sheepuff, sheepuff_wool, sheepuffed};
use b173_api::prelude::*;
use b173_api::terrain::math;

const NAME: &str = "sheepuff";
const TEXTURE: &str = "aether:entity/sheepuff/sheepuff";
const FUR: &str = "aether:entity/sheepuff/fur";
const SHEARS: &str = "minecraft:shears";
const DYE: &str = "minecraft:dye";
const WOOL: &str = "minecraft:wool";
const SHEEP_SAY: &str = "mob.sheep";

// EntitySheep.fleeceColorTable
const FLEECE: [[f32; 3]; 16] = [
    [1.0, 1.0, 1.0],
    [0.95, 0.7, 0.2],
    [0.9, 0.5, 0.85],
    [0.6, 0.7, 0.95],
    [0.9, 0.9, 0.2],
    [0.5, 0.8, 0.1],
    [0.95, 0.7, 0.8],
    [0.3, 0.3, 0.3],
    [0.6, 0.6, 0.6],
    [0.3, 0.6, 0.7],
    [0.7, 0.4, 0.9],
    [0.2, 0.4, 0.8],
    [0.5, 0.4, 0.3],
    [0.4, 0.5, 0.2],
    [0.8, 0.3, 0.3],
    [0.1, 0.1, 0.1],
];

mod puff
{
    // a puffed sheepuff floats down and bounds high
    pub const JUMP: f64 = 1.8;
    pub const JUMP_SPREAD: f64 = 0.5;
    pub const MAX_FALL: f64 = -0.05;
    pub const NORMAL_JUMP: f64 = 0.42;
    // SheepuffAIEatAetherGrass runs one time in a thousand
    pub const EAT_CHANCE: i32 = 1000;
    pub const EAT_TICKS: f32 = 40.0;
    pub const EATS_TO_PUFF: u8 = 2;
    pub const MIN_SHEAR: i32 = 2;
    pub const SHEAR_SPREAD: i32 = 3;
    pub const DEATH_WOOL_SPREAD: i32 = 2;
}

// EntitySheepuff.getRandomFleeceColor: a roll under the bound picks the color, otherwise white or rarely purple
const COLOR_ROLL: i32 = 100;
const COLOR_ROLLS: [(i32, u8); 4] = [(5, 3), (10, 9), (15, 5), (18, 6)];
const PURPLE_ODDS: i32 = 500;
const PURPLE: u8 = 10;
const WHITE: u8 = 0;
const COLORS: u8 = 15;

// ModelSheep head animation while grazing
mod graze
{
    pub const LOWER_END: f32 = 4.0;
    pub const RAISE_START: f32 = 36.0;
    pub const NOD_TICKS: f32 = 32.0;
    pub const NOD_SPEED: f32 = 28.7;
    pub const NOD_ANGLE: f32 = 0.2199115;
    pub const HEAD_REST_Y: f32 = 6.0;
    pub const HEAD_DROP: f32 = 9.0;
    pub const JITTER_STEPS: i32 = 1000;
    // a puffed sheepuff takes two dyes, a plain one takes one
    pub const PUFFED_DYE: i32 = 2;
}

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct Fleece
{
    m_ready: bool,
    m_color: u8,
    m_sheared: bool,
    m_puffed: bool,
    m_eaten: u8,
    // ticksExisted when the last grazing started, the head animation runs from it
    m_eatStart: i32,
}

// EntitySheepuff.getRandomFleeceColor
fn randomColor(world: World) -> u8
{
    let roll: i32 = world.random(COLOR_ROLL);
    match COLOR_ROLLS.iter().find(|(bound, _)| roll < *bound)
    {
        Some((_, color)) => *color,
        None if world.random(PURPLE_ODDS) != 0 => WHITE,
        None => PURPLE,
    }
}

fn fleece(entity: Entity) -> Fleece
{
    let mut state: Fleece = entity.synced();
    if !state.m_ready && entity.world.isAuthority()
    {
        state = Fleece { m_ready: true, m_color: randomColor(entity.world), ..state };
        let _ = entity.setSynced(&state);
    }

    state
}

fn ticks(entity: Entity) -> i32 { entity.field(EntityField::TicksExisted) as i32 }

// EntitySheepuff.getHeadRotationPointY and getHeadRotationAngleX for a grazing timer counting down from 40
fn headPose(timer: f32, pitch: f32) -> (f32, f32)
{
    let drop: f32 = if timer <= 0.0
    {
        0.0
    }
    else if (graze::LOWER_END..=graze::RAISE_START).contains(&timer)
    {
        1.0
    }
    else if timer < graze::LOWER_END
    {
        timer / graze::LOWER_END
    }
    else
    {
        -(timer - puff::EAT_TICKS) / graze::LOWER_END
    };

    let angle: f32 = if timer > graze::LOWER_END && timer <= graze::RAISE_START
    {
        std::f32::consts::PI / 5.0
            + graze::NOD_ANGLE * math::sin((timer - graze::LOWER_END) / graze::NOD_TICKS * graze::NOD_SPEED)
    }
    else if timer > 0.0
    {
        std::f32::consts::PI / 5.0
    }
    else
    {
        pitch.to_radians()
    };
    (drop, angle)
}

struct Sheepuff;

#[entity_behavior]
impl EntityBehavior for Sheepuff
{
    // EntitySheepuff.onUpdate and the grazing AI
    fn onLivingUpdate(&self, entity: Entity, info: &EntityInfo)
    {
        let mut state: Fleece = fleece(entity);
        let [motionX, motionY, motionZ] = info.motion;

        if state.m_puffed
        {
            entity.setField(EntityField::FallDistance, 0.0);

            if motionY > puff::NORMAL_JUMP - 0.01 && motionY < puff::NORMAL_JUMP + 0.01
            {
                let spread = ||
                {
                    (entity.world.random(graze::JITTER_STEPS) as f64 / graze::JITTER_STEPS as f64 - 0.5)
                        * 2.0
                        * puff::JUMP_SPREAD
                };
                let _ = entity.setMotion([motionX + spread(), puff::JUMP, motionZ + spread()]);
            }
            else if motionY < puff::MAX_FALL
            {
                let _ = entity.setMotion([motionX, puff::MAX_FALL, motionZ]);
            }
        }

        if !entity.world.isAuthority() || entity.world.random(puff::EAT_CHANCE) != 0
        {
            return;
        }
        let feet: BlockPos =
            BlockPos::new(info.pos[0].floor() as i32, info.pos[1].floor() as i32 - 1, info.pos[2].floor() as i32);
        let palette: Palette = Palette::load();
        if entity.world.block(feet) != palette.m_grass
        {
            return;
        }

        let _ = entity.world.setBlock(feet, palette.m_dirt, 0);

        state.m_eatStart = ticks(entity);
        state.m_eaten += 1;
        if state.m_eaten >= puff::EATS_TO_PUFF && !state.m_sheared && !state.m_puffed
        {
            state.m_puffed = true;
            state.m_eaten = 0;
        }

        if state.m_eaten == 1 && state.m_sheared && !state.m_puffed
        {
            state = Fleece { m_sheared: false, m_color: WHITE, m_eaten: 0, ..state };
        }

        let _ = entity.setSynced(&state);
    }

    // EntitySheepuff.interact, shears take the fleece and dye recolors it
    fn onInteract(&self, entity: Entity, player: Player) -> bool
    {
        let Some(held) = player.held()
        else
        {
            return false;
        };
        let mut state: Fleece = fleece(entity);

        let Some(info) = entity.info()
        else
        {
            return false;
        };

        if held.item.is(SHEARS) && !state.m_sheared
        {
            if entity.world.isAuthority()
            {
                state.m_sheared = true;
                state.m_puffed = false;

                let count: i32 = puff::MIN_SHEAR + entity.world.random(puff::SHEAR_SPREAD);
                for _ in 0..count
                {
                    let _ = entity.world.spawnItem(info.pos, ItemStack::of(WOOL, 1).withDamage(state.m_color as i32));
                }

                let _ = entity.setSynced(&state);
            }

            let _ = player.setHeld(Some(ItemStack { damage: held.damage + 1, ..held }));

            return true;
        }

        let dyeColor: u8 = COLORS - held.damage.clamp(0, COLORS as i32) as u8;
        if held.item.is(DYE) && !state.m_sheared && state.m_color != dyeColor
        {
            let cost: i32 = if state.m_puffed { graze::PUFFED_DYE } else { 1 };
            if held.count >= cost
            {
                state.m_color = dyeColor;

                let _ = entity.setSynced(&state);
                let left: i32 = held.count - cost;
                let _ = player.setHeld((left > 0).then_some(ItemStack { count: left, ..held }));
            }

            return true;
        }

        false
    }

    fn onDeath(&self, entity: Entity, _source: Option<i32>)
    {
        let state: Fleece = fleece(entity);
        if let Some(info) = entity.info()
            && !state.m_sheared
            && entity.world.isAuthority()
        {
            let count: i32 = 1 + entity.world.random(puff::DEATH_WOOL_SPREAD);
            let _ = entity.world.spawnItem(info.pos, ItemStack::of(WOOL, count).withDamage(state.m_color as i32));
        }
    }

    fn canSpawn(&self, entity: Entity, info: &EntityInfo) -> bool
    {
        crate::config::spawnRoll(entity, "sheepuff") && onAetherGrass(info)
    }

    // SheepuffRenderer: the shorn body, then the fur layer in the fleece color unless sheared
    fn pose(&self, input: &PoseInput) -> Pose
    {
        let state: Fleece = b173_api::fromBytes(&input.synced).unwrap_or_default();
        let timer: f32 = (puff::EAT_TICKS - (input.age - state.m_eatStart as f32)).max(0.0);
        let (drop, angle) = headPose(if state.m_eatStart > 0 { timer } else { 0.0 }, input.head_pitch);
        let mut body: Program = Program::new();

        let lowered = |pivots: &[[f32; 3]]|
        {
            let mut pivots: Vec<[f32; 3]> = pivots.to_vec();
            pivots[quadruped::HEAD as usize][1] = graze::HEAD_REST_Y + drop * graze::HEAD_DROP;
            pivots
        };
        quadrupedPose(&mut body, 0, &lowered(&sheepuff_wool::PIVOTS), input, Some(angle));

        if !state.m_sheared
        {
            let [red, green, blue] = FLEECE[(state.m_color & COLORS) as usize];
            body.texture(FUR).color(red, green, blue, 1.0);

            let (base, pivots) = if state.m_puffed
            {
                (2 * quadruped::PARTS, lowered(&sheepuffed::PIVOTS))
            }
            else
            {
                (quadruped::PARTS, lowered(&sheepuff::PIVOTS))
            };
            quadrupedPose(&mut body, base, &pivots, input, Some(angle));
            body.color(1.0, 1.0, 1.0, 1.0);
        }

        Pose { pre: Vec::new(), model: body.take() }
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    let say: String = format!("{SHEEP_SAY}.say");
    ctx.entities().register(
        NAME,
        EntitySpec
        {
            template: Identifier::parse("minecraft:sheep").ok(),
            texture: Some(TEXTURE.to_owned()),
            size: Some([0.9, 1.3]),
            health: Some(8),
            spawn: creature(10),
            drops: Some(Vec::new()),
            sounds: Some(EntitySounds { living: Some(say.clone()), hurt: Some(say.clone()), death: Some(say) }),
            model: Some(EntityModelSpec
            {
                shadow: 0.7,
                ..combine(vec![sheepuff_wool::model(), sheepuff::model(), sheepuffed::model()])
            }),
            ..EntitySpec::default()
        },
        Some(std::rc::Rc::new(Sheepuff)),
    )?;
    Ok(())
}
