use crate::bosses::Home;
use crate::ids::{self, item};
use crate::models::{DEGREES_PER_RADIAN, Program, WALK_RATE, quadruped, tf_quest_ram};
use crate::progress::{self, Gate};
use b173_api::prelude::*;
use b173_api::terrain::math;
use std::rc::Rc;

// EntityTFQuestRam, fed one wool of every color it pays out blocks and a crumble horn, it eats loose wool it finds
const NAME: &str = "quest_ram";
const WOOL: &str = "minecraft:wool";
const COLORS: usize = 16;
const ALL_COLORS: u32 = (1 << COLORS) - 1;
const SOUND: &str = "mob.sheep";

mod tuning
{
    // EntityAITFFindLoose and EntityAITempt reach
    pub const LOOSE_REACH: f64 = 16.0;
    pub const TEMPT_REACH: f64 = 10.0;
    pub const EAT_REACH: f64 = 1.5;
    pub const SEARCH_CHANCE: i32 = 20;
    pub const SPEED: f32 = 1.0;
    // EntityTFQuestRam keeps to thirteen blocks of its grove
    pub const HOME_RANGE: f64 = 13.0;
    pub const HOME_CHANCE: i32 = 95;
    pub const REWARD_REACH: f64 = 16.0;
    pub const FED_SPARKS: i32 = 50;
    pub const DONE_SPARKS: i32 = 5;
    pub const SPREAD: f64 = 1.5;
    pub const VOLUME: f32 = 5.0;
    pub const PITCH: f32 = 0.7;
    pub const PITCH_SPREAD: f32 = 0.2;
}

mod body
{
    // ModelTFQuestRam.setLivingAnimations pushes the rear half back two pixels per color
    pub const REAR_BASE: f32 = 2.0;
    pub const LEGS_BASE: f32 = 11.0;
    pub const SEGMENT_BASE: f32 = 2.0;
    pub const SEGMENT_STEP: f32 = 2.0;
    pub const LEG_DAMPING: f32 = 0.5;
}

const SIZE: [f32; 2] = [1.25, 2.9];
const HEALTH: i32 = 70;
// beta's pig walks at 0.7 for the 1.7 attribute's 0.25, the ram's 0.23 scaled the same
const SPEED: f32 = 0.23 * 0.7 / 0.25;
const SHADOW: f32 = 1.0;

// ModelTFQuestRam.colorOrder, the order the colored bands stack along the body
const ORDER: [usize; COLORS] = [0, 8, 7, 15, 14, 1, 4, 5, 13, 3, 9, 11, 10, 2, 6, 12];

// EntitySheep.fleeceColorTable
const FLEECE: [[f32; 3]; COLORS] = [
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

// EntityTFQuestRam.rewardQuest, beta has no emerald block so a second diamond block stands in
const REWARDS: [&str; 6] = [
    "minecraft:diamond_block",
    "minecraft:iron_block",
    "minecraft:diamond_block",
    "minecraft:gold_block",
    "minecraft:lapis_block",
    item::CRUMBLE_HORN,
];

// the data watcher's color flags and rewarded byte, mirrored for the bands
#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct Fleece
{
    m_colors: u32,
    m_rewarded: bool,
}

impl Fleece
{
    fn has(self, color: usize) -> bool { self.m_colors & (1 << color) != 0 }

    fn count(self) -> u32 { self.m_colors.count_ones() }
}

fn sparkle(world: World, info: &EntityInfo, color: usize, count: i32)
{
    let [red, green, blue]: [f32; 3] = FLEECE[color];
    for _ in 0..count
    {
        let width: f64 = f64::from(SIZE[0]) * tuning::SPREAD;
        let at: [f64; 3] = [
            info.pos[0] + (f64::from(ids::fraction(world)) - 0.5) * width,
            info.pos[1] + f64::from(ids::fraction(world)) * f64::from(SIZE[1]) * tuning::SPREAD,
            info.pos[2] + (f64::from(ids::fraction(world)) - 0.5) * width,
        ];
        world.particle("reddust", at, [f64::from(red), f64::from(green), f64::from(blue)]);
    }
}

fn bleat(world: World, info: &EntityInfo)
{
    let pitch: f32 = (ids::fraction(world) - ids::fraction(world)) * tuning::PITCH_SPREAD + tuning::PITCH;
    world.playSound(info.pos, SOUND, tuning::VOLUME, pitch);
}

// EntityTFQuestRam.setColorPresent with animateAddColor
fn feed(entity: Entity, info: &EntityInfo, color: usize) -> bool
{
    let mut fleece: Fleece = entity.synced();
    if color >= COLORS || fleece.has(color)
    {
        return false;
    }
    fleece.m_colors |= 1 << color;
    if let Err(error) = entity.setSynced(&fleece)
    {
        warn(&format!("quest ram colors: {}", error.0));
        return false;
    }
    sparkle(entity.world, info, color, tuning::FED_SPARKS);
    bleat(entity.world, info);
    true
}

fn reward(entity: Entity, info: &EntityInfo)
{
    let world: World = entity.world;
    for name in REWARDS
    {
        if let Err(error) = world.spawnItem(info.pos, ItemStack::of(name, 1))
        {
            warn(&format!("quest ram reward {name}: {}", error.0));
        }
    }
    for player in world.players()
    {
        let near: bool =
            player.info().is_some_and(|other| super::ai::distance(other.pos, info.pos) <= tuning::REWARD_REACH);
        if near
        {
            progress::grant(player, Gate::QuestRam);
        }
    }
}

fn block(pos: [f64; 3]) -> BlockPos
{
    BlockPos::new(pos[0].floor() as i32, pos[1].floor() as i32, pos[2].floor() as i32)
}

fn woolColor(stack: &ItemStack) -> Option<usize>
{
    stack.item.is(WOOL).then_some(stack.damage.clamp(0, COLORS as i32 - 1) as usize)
}

// EntityAITFFindLoose and EntityAITFEatLoose, walks to a dropped wool of a missing color and eats it
fn eatLoose(entity: Entity, info: &EntityInfo, fleece: Fleece) -> bool
{
    let world: World = entity.world;
    let loose: Option<(EntityInfo, usize)> = world
        .entitiesNear(info.pos, tuning::LOOSE_REACH)
        .into_iter()
        .filter_map(|other|
        {
            let color: usize = Entity { world, id: other.id }.itemStack().as_ref().and_then(woolColor)?;
            (!fleece.has(color)).then_some((other, color))
        })
        .min_by(|a, b| super::ai::distance(a.0.pos, info.pos).total_cmp(&super::ai::distance(b.0.pos, info.pos)));
    let Some((wool, color)) = loose
    else
    {
        return false;
    };

    if super::ai::distance(wool.pos, info.pos) > tuning::EAT_REACH
    {
        entity.walkTo(block(wool.pos), tuning::SPEED);
        return true;
    }
    if feed(entity, info, color)
    {
        Entity { world, id: wool.id }.kill();
    }
    true
}

// EntityAITempt, it follows a player holding wool
fn tempted(entity: Entity, info: &EntityInfo) -> bool
{
    let tempter: Option<EntityInfo> = entity.world.players().into_iter().find_map(|player|
    {
        let holding: bool = player.held().is_some_and(|stack| stack.item.is(WOOL));
        let other: EntityInfo = player.info()?;
        (holding && super::ai::distance(other.pos, info.pos) <= tuning::TEMPT_REACH).then_some(other)
    });
    let Some(tempter) = tempter
    else
    {
        return false;
    };
    entity.walkTo(block(tempter.pos), tuning::SPEED);
    true
}

// EntityTFQuestRam.updateAITasks keeps it near its grove
fn goHome(entity: Entity, info: &EntityInfo)
{
    let Home { m_home: Some([x, y, z]) } = entity.data()
    else
    {
        return;
    };
    let home: [f64; 3] = [f64::from(x), f64::from(y), f64::from(z)];
    if super::ai::distance(home, info.pos) > tuning::HOME_RANGE
    {
        entity.walkTo(BlockPos::new(x, y, z), tuning::SPEED);
    }
}

struct Ram;

#[entity_behavior]
impl EntityBehavior for Ram
{
    fn onTick(&self, entity: Entity, info: &EntityInfo)
    {
        let world: World = entity.world;
        let fleece: Fleece = entity.synced();
        let done: bool = fleece.m_colors == ALL_COLORS && !fleece.m_rewarded;
        // checkAndAnimateColors runs on the client, the reward on the server
        if !world.isAuthority()
        {
            if done
            {
                sparkle(world, info, world.random(COLORS as i32) as usize, tuning::DONE_SPARKS);
            }
            return;
        }

        if done
        {
            reward(entity, info);
            if let Err(error) = entity.setSynced(&Fleece { m_rewarded: true, ..fleece })
            {
                warn(&format!("quest ram reward flag: {}", error.0));
            }
            return;
        }
        let searching: bool = world.random(tuning::SEARCH_CHANCE) == 0;
        if searching && (tempted(entity, info) || eatLoose(entity, info, fleece))
        {
            return;
        }
        if world.random(tuning::HOME_CHANCE) == 0
        {
            goHome(entity, info);
        }
    }

    // EntityTFQuestRam.interact takes one wool of a color it lacks
    fn onInteract(&self, entity: Entity, player: Player) -> bool
    {
        let (Some(stack), Some(info)) = (player.held(), entity.info())
        else
        {
            return false;
        };
        let Some(color) = woolColor(&stack)
        else
        {
            return false;
        };
        if !entity.world.isAuthority() || !feed(entity, &info, color)
        {
            return true;
        }

        let left: Option<ItemStack> = (stack.count > 1).then(|| ItemStack { count: stack.count - 1, ..stack });
        if let Err(error) = player.setHeld(left)
        {
            warn(&format!("quest ram wool: {}", error.0));
        }
        true
    }

    // ModelTFQuestRam.setLivingAnimations and setRotationAngles, the bands of the colors fed stack along the back
    fn pose(&self, input: &PoseInput) -> Pose
    {
        let fleece: Fleece = b173_api::fromBytes(&input.synced).unwrap_or_default();
        let count: f32 = fleece.count() as f32;
        let pivots: &[[f32; 3]] = &tf_quest_ram::PIVOTS;
        let pivot = |part: u16| pivots[part as usize];
        let shifted = |part: u16, z: f32| [pivot(part)[0], pivot(part)[1], z];

        let mut body: Program = Program::new();
        let yaw: f32 = input.head_yaw / DEGREES_PER_RADIAN;
        let swing: f32 =
            math::cos(input.walk * WALK_RATE) * quadruped::LEG_SWING * input.walk_speed * body::LEG_DAMPING;
        let back: f32 = math::cos(input.walk * WALK_RATE + std::f32::consts::PI)
            * quadruped::LEG_SWING
            * input.walk_speed
            * body::LEG_DAMPING;
        let legsZ: f32 = body::LEGS_BASE + body::SEGMENT_STEP * count;
        body.rest(tf_quest_ram::FRONTBODY);
        body.part(
            tf_quest_ram::REARBODY,
            [0.0; 3],
            shifted(tf_quest_ram::REARBODY, body::REAR_BASE + body::SEGMENT_STEP * count),
        );
        for (leg, haunch, angle, moved) in [
            (tf_quest_ram::LEG1, tf_quest_ram::HAUNCH1, swing, true),
            (tf_quest_ram::LEG2, tf_quest_ram::HAUNCH2, back, true),
            (tf_quest_ram::LEG3, tf_quest_ram::HAUNCH3, back, false),
            (tf_quest_ram::LEG4, tf_quest_ram::HAUNCH4, swing, false),
        ]
        {
            let legPivot: [f32; 3] = if moved
            {
                shifted(leg, legsZ)
            }
            else
            {
                pivot(leg)
            };
            let haunchPivot: [f32; 3] = if moved
            {
                shifted(haunch, legsZ)
            }
            else
            {
                pivot(haunch)
            };
            body.part(leg, [angle, 0.0, 0.0], legPivot);
            body.part(haunch, [angle, 0.0, 0.0], haunchPivot);
        }
        body.part(tf_quest_ram::NECK, [0.0, yaw, 0.0], pivot(tf_quest_ram::NECK));
        body.part(tf_quest_ram::HEAD, [input.head_pitch / DEGREES_PER_RADIAN, yaw, 0.0], pivot(tf_quest_ram::HEAD));

        let mut offset: f32 = body::SEGMENT_BASE;
        for color in ORDER.into_iter().filter(|color| fleece.has(*color))
        {
            let [red, green, blue]: [f32; 3] = FLEECE[color];
            let part: u16 = tf_quest_ram::SEGMENTS_0 + color as u16;
            body.color(red, green, blue, 1.0).part(part, [0.0; 3], shifted(part, offset));
            offset += body::SEGMENT_STEP;
        }
        body.color(1.0, 1.0, 1.0, 1.0);
        Pose { pre: Vec::new(), model: body.take() }
    }
}

pub fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.entities().register(
        NAME,
        EntitySpec
        {
            template: Identifier::parse("minecraft:pig").ok(),
            texture: Some("twilightforest:model/questram".to_owned()),
            size: Some(SIZE),
            health: Some(HEALTH),
            speed: Some(SPEED),
            name: Some("Questing Ram".to_owned()),
            sounds: super::sounds(Some(SOUND), SOUND, SOUND),
            model: Some(EntityModelSpec { shadow: SHADOW, ..tf_quest_ram::model() }),
            persistent: true,
            ..EntitySpec::default()
        },
        Some(Rc::new(Ram)),
    )?;
    Ok(())
}
