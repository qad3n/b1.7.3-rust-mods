use super::{Kind, Look, base, faces, property, texture, when};
use crate::ids::{self, block, cell};
use b173_api::prelude::*;
use std::rc::Rc;

// BlockTFSlider: a thin pillar that wakes every four seconds while a player is near its column, sends a copy of itself
// sliding out along its axis and back, and hurts whatever it hits
mod slider
{
    pub const HARDNESS: f32 = 2.0;
    pub const RESISTANCE: f32 = 10.0;
    pub const PERIOD: i64 = 80;
    // the low two meta bits stagger sliders of one trap by a second each
    pub const OFFSET: i64 = 20;
    pub const DELAYS: i32 = 4;
    pub const RANGE: f64 = 32.0;
    pub const DAMAGE: i32 = 5;
    pub const INSET: f32 = 5.0;
}

// EntityTFSlideBlock
mod slide
{
    pub const NAME: &str = "slide_block";
    pub const SIZE: f32 = 0.98;
    pub const WARMUP: i32 = 20;
    pub const PUSH: f64 = 0.04;
    pub const DRAG: f64 = 0.98;
    pub const TURN: i32 = 60;
    pub const LOST: i32 = 100;
    pub const EXPIRE: i32 = 600;
    pub const SOUND_EVERY: i32 = 5;
    pub const SOUND: &str = "twilightforest.random.slider";
    pub const HEIGHT: i32 = 128;
}

// the axis values in BlockRotatedPillar meta order, the stagger fills the low two bits
const AXES: [&str; 3] = ["y", "x", "z"];
const DELAYS: [&str; 4] = ["0", "1", "2", "3"];
const AXIS_STEP: i32 = 4;

// where the slide may run along each axis, in BlockTFSlider meta order, tried before any open way
const WAYS: [[[i32; 3]; 4]; 3] = [
    [[-1, 0, 0], [1, 0, 0], [0, 0, -1], [0, 0, 1]],
    [[0, -1, 0], [0, 1, 0], [0, 0, -1], [0, 0, 1]],
    [[0, -1, 0], [0, 1, 0], [-1, 0, 0], [1, 0, 0]],
];

fn axis(meta: i32) -> usize { (meta / AXIS_STEP).clamp(0, AXES.len() as i32 - 1) as usize }

// the direction an axis runs, the column of sliders that share a trap lies along it
fn along(meta: i32) -> [i32; 3]
{
    match axis(meta)
    {
        1 => [1, 0, 0],
        2 => [0, 0, 1],
        _ => [0, 1, 0],
    }
}

fn playerNear(world: World, pos: BlockPos) -> bool
{
    let center: [f64; 3] = ids::center(pos);
    world.players().into_iter().any(|player|
    {
        player.info().is_some_and(|info| crate::entities::ai::distance(info.pos, center) <= slider::RANGE)
    })
}

// BlockTFSlider.isConnectedInRange, a player near this slider or any matching one in a straight run along its axis
fn awake(world: World, pos: BlockPos, meta: i32) -> bool
{
    if playerNear(world, pos)
    {
        return true;
    }

    let [dx, dy, dz] = along(meta);
    let me: i32 = world.block(pos);
    [1, -1].into_iter().any(|sign|
    {
        let mut at: BlockPos = pos.offset(dx * sign, dy * sign, dz * sign);
        while world.block(at) == me && world.meta(at) == meta
        {
            if playerNear(world, at)
            {
                return true;
            }

            at = at.offset(dx * sign, dy * sign, dz * sign);
        }
        false
    })
}

// BlockTFSlider.scheduleBlockUpdate, every slider fires on the same beat shifted by its stagger
fn schedule(world: World, pos: BlockPos, meta: i32)
{
    let offset: i64 = i64::from(meta % slider::DELAYS) * slider::OFFSET;
    let delay: i64 = slider::PERIOD - (world.time() - offset).rem_euclid(slider::PERIOD);
    if let Err(error) = world.scheduleTick(pos, world.block(pos), delay as i32)
    {
        warn(&format!("slider tick at {pos:?}: {}", error.0));
    }
}

fn hurt(world: World, target: &EntityInfo, from: [f64; 3], source: Option<i32>)
{
    let entity: Entity = Entity { world, id: target.id };
    entity.damage(slider::DAMAGE, source);
    if source.is_some()
    {
        return;
    }

    let toward: [f64; 2] = [from[0] - target.pos[0], from[2] - target.pos[2]];
    let _ = entity.setMotion(crate::math::knockback(target.motion, toward));
}

struct Slider;

#[block_behavior]
impl BlockBehavior for Slider
{
    fn onAdded(&self, ctx: &mut BlockCtx)
    {
        if ctx.isAuthority()
        {
            schedule(ctx.world(), ctx.pos(), i32::from(ctx.meta()));
        }
    }

    // BlockRotatedPillar.onBlockPlaced lays the pillar along the clicked face's axis
    fn onPlaced(&self, ctx: &mut BlockCtx, face: i32)
    {
        let axis: i32 = match face
        {
            2 | 3 => 2,
            4 | 5 => 1,
            _ => 0,
        };
        let meta: i32 = axis * AXIS_STEP + i32::from(ctx.meta()) % slider::DELAYS;
        ctx.setMeta(meta as u8);
    }

    fn onUpdateTick(&self, ctx: &mut BlockCtx)
    {
        if !ctx.isAuthority()
        {
            return;
        }

        let (world, pos): (World, BlockPos) = (ctx.world(), ctx.pos());
        let meta: i32 = i32::from(ctx.meta());
        if awake(world, pos, meta)
        {
            launch(world, pos, meta);
        }
        schedule(world, pos, meta);
    }

    fn onEntityCollided(&self, ctx: &mut BlockCtx, entity: Entity)
    {
        if !ctx.isAuthority()
        {
            return;
        }

        let Some(info) = entity.info()
        else
        {
            return;
        };

        if info.health.is_some()
        {
            hurt(ctx.world(), &info, ids::center(ctx.pos()), None);
        }
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct Slide
{
    m_block: i32,
    m_meta: i32,
    m_ticks: i32,
    m_way: [i32; 3],
}

impl Slide
{
    fn stopped(&self) -> bool { self.m_way == [0; 3] }
}

// EntityTFSlideBlock.determineMoveDirection, toward open space away from a wall, else any open way
fn way(world: World, pos: BlockPos, meta: i32) -> [i32; 3]
{
    let ways: &[[i32; 3]; 4] = &WAYS[axis(meta)];
    let open = |step: [i32; 3]| world.isAir(pos.offset(step[0], step[1], step[2]));
    for pair in ways.chunks(2)
    {
        let [first, second] = [pair[0], pair[1]];
        if !open(second) && open(first)
        {
            return first;
        }

        if !open(first) && open(second)
        {
            return second;
        }
    }
    ways.iter().copied().find(|step| open(*step)).unwrap_or([0; 3])
}

fn launch(world: World, pos: BlockPos, meta: i32)
{
    let state: Slide = Slide { m_block: world.block(pos), m_meta: meta, m_ticks: 0, m_way: way(world, pos, meta) };
    let entity: Entity = match world.spawnEntity(&format!("twilightforest:{}", slide::NAME), ids::center(pos))
    {
        Ok(entity) => entity,
        Err(error) =>
        {
            warn(&format!("slider at {pos:?} could not launch: {}", error.0));
            return;
        }
    };

    entity.setField(EntityField::Block, f64::from(state.m_block));
    entity.setField(EntityField::BlockMeta, f64::from(state.m_meta));
    if let Err(error) = entity.setData(&state)
    {
        warn(&format!("slide block state: {}", error.0));
    }
}

// the slide drops as its slider item when it cannot settle
fn spill(world: World, at: [f64; 3])
{
    if let Err(error) = world.spawnItem(at, ItemStack::of(format!("twilightforest:{}", block::SLIDER).as_str(), 1))
    {
        warn(&format!("slide block drop: {}", error.0));
    }
}

// the leading cell the next move would enter, a solid one stops the slide like Entity.isCollided
fn blocked(world: World, at: [f64; 3], motion: [f64; 3]) -> bool
{
    let half: f64 = f64::from(slide::SIZE) / 2.0;
    let lead: [f64; 3] = std::array::from_fn(|axis| at[axis] + motion[axis] + motion[axis].signum() * half);
    let moving: bool = motion.iter().any(|value| *value != 0.0);
    moving && World::blockInfo(world.block(cell(lead))).solid
}

struct SlideBlock;

#[entity_behavior]
impl EntityBehavior for SlideBlock
{
    // EntityTFSlideBlock.onUpdate
    fn onTick(&self, entity: Entity, info: &EntityInfo)
    {
        let world: World = entity.world;
        if !world.isAuthority()
        {
            return;
        }

        let mut state: Slide = entity.data();
        state.m_ticks += 1;
        let here: BlockPos = cell(info.pos);
        if state.m_ticks == 1
        {
            if world.block(here) != state.m_block
            {
                entity.kill();
                return;
            }

            let _ = world.setBlock(here, 0, 0);
        }

        let mut motion: [f64; 3] = info.motion;
        if state.m_ticks > slide::WARMUP
        {
            motion = std::array::from_fn(|axis| motion[axis] + f64::from(state.m_way[axis]) * slide::PUSH);
        }
        if state.m_ticks % slide::SOUND_EVERY == 0
        {
            let pitch: f32 = 0.9 + ids::fraction(world) * 0.4;
            world.playSound(info.pos, slide::SOUND, 1.0, pitch);
        }
        if state.m_ticks == slide::TURN
        {
            motion = [0.0; 3];
            state.m_way = state.m_way.map(|step| -step);
        }

        let hit: bool = blocked(world, info.pos, motion);
        if hit || state.stopped()
        {
            settle(world, entity, here, state);
            return;
        }

        let lost: bool = state.m_ticks > slide::LOST && !(1..=slide::HEIGHT).contains(&here.y);
        if lost || state.m_ticks > slide::EXPIRE
        {
            spill(world, info.pos);
            entity.kill();
            return;
        }

        let _ = entity.setMotion(motion.map(|value| value * slide::DRAG));
        strike(world, entity.id, info.pos);
        let _ = entity.setData(&state);
    }
}

// the slide stops where it is and becomes its slider again, or drops when the spot is taken
fn settle(world: World, entity: Entity, here: BlockPos, state: Slide)
{
    entity.kill();
    let placed: bool = world.isAir(here) && world.setBlock(here, state.m_block, state.m_meta).unwrap_or(false);
    if !placed
    {
        spill(world, ids::center(here));
    }
}

// EntityTFSlideBlock.damageKnockbackEntities, everything alive inside the slide gets hit by it
fn strike(world: World, me: i32, at: [f64; 3])
{
    let reach: f64 = f64::from(slide::SIZE);
    for near in world.entitiesNear(at, reach)
    {
        let inside: bool = (0..3).all(|axis| (near.pos[axis] - at[axis]).abs() < reach);
        if near.id != me && near.health.is_some() && inside
        {
            hurt(world, &near, at, Some(me));
        }
    }
}

// the pillar for one axis, every stagger of that axis shows it
fn pillar(axis: usize) -> Cuboid
{
    let (low, high): (f32, f32) = (slider::INSET, 16.0 - slider::INSET);
    let (from, to): ([f32; 3], [f32; 3]) = match axis
    {
        1 => ([0.0, low, low], [16.0, high, high]),
        2 => ([low, low, 0.0], [high, high, 16.0]),
        _ => ([low, 0.0, low], [high, 16.0, high]),
    };
    let first: i32 = axis as i32 * AXIS_STEP;
    Cuboid { from, to, metas: (first..first + slider::DELAYS).collect(), ..Cuboid::default() }
}

fn lying() -> FaceTextures
{
    FaceTextures
    {
        top: Some(texture("slider_top")),
        bottom: Some(texture("slider_top")),
        side: Some(texture("slider_h")),
        front: None,
    }
}

pub fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    super::put(
        ctx,
        block::SLIDER,
        base("minecraft:block", "Slider", Kind::Metal, slider::HARDNESS)
            .resistance(slider::RESISTANCE)
            .sound("stone")
            .faces(faces(&Look::Column { top: "slider_top", side: "slider_v" }))
            .model(BlockModelSpec { cuboids: (0..AXES.len()).map(pillar).collect(), ..BlockModelSpec::default() })
            .behavior(Slider),
    )?;
    let variants: Vec<StateVariantSpec> = ["x", "z"]
        .into_iter()
        .map(|value| StateVariantSpec { when: when("axis", value), texture: None, textures: Some(lying()) })
        .collect();
    ctx.blocks().states(
        block::SLIDER,
        BlockStatesSpec
        {
            properties: vec![property("axis", &AXES), property("delay", &DELAYS)],
            variants,
            ..BlockStatesSpec::default()
        },
    )?;
    ctx.entities().register(
        slide::NAME,
        EntitySpec
        {
            template: Identifier::parse("minecraft:moving_block").ok(),
            size: Some([slide::SIZE, slide::SIZE]),
            tracking_range: Some(64),
            update_frequency: Some(2),
            send_velocity: Some(true),
            ..EntitySpec::default()
        },
        Some(Rc::new(SlideBlock)),
    )?;
    Ok(())
}
