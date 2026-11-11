use crate::blocks::tower::{Device, deviceName, jetName, nextJet};
use crate::math::NEIGHBORS;
use crate::{names, particles};
use b173_api::math::{distanceSquared, floor};
use b173_api::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub(crate) const SMOKER_TILE: &str = "smoker";
pub(crate) const POPPING_TILE: &str = "popping_jet";
pub(crate) const FLAME_TILE: &str = "flame_jet";
const BUILDER_TILE: &str = "tower_builder";
const REVERTER_TILE: &str = "reverter";
const TRAP_TILE: &str = "ghast_trap_inactive";
const TRAP_ACTIVE_TILE: &str = "ghast_trap_active";
const REACTOR_TILE: &str = "reactor_active";

pub(crate) fn tileFor(device: Device) -> Option<&'static str>
{
    match device
    {
        Device::BuilderActive => Some(BUILDER_TILE),
        Device::Antibuilder => Some(REVERTER_TILE),
        Device::GhastTrap => Some(TRAP_TILE),
        Device::GhastTrapActive => Some(TRAP_ACTIVE_TILE),
        Device::ReactorActive => Some(REACTOR_TILE),
        _ => None,
    }
}

#[derive(Serialize, Deserialize, Default)]
struct Counter
{
    m_ticks: i32,
}

fn bump(ctx: &mut BlockCtx) -> i32
{
    let mut counter: Counter = ctx.tileData();
    counter.m_ticks += 1;

    let _ = ctx.setTileData(&counter);

    counter.m_ticks
}

// TileEntityTFSmoker puffs a column of huge smoke every fourth tick
struct Smoker;

mod smoke
{
    pub const EVERY: i32 = 4;
    pub const SWIRL: f64 = 10.0;
    pub const DRIFT: f64 = 0.05;
    pub const RISE: f64 = 0.25;
    pub const TOP: f64 = 0.95;
}

fn swirl(world: World, pos: BlockPos, ticks: i32)
{
    let phase: f64 = f64::from(ticks) / smoke::SWIRL;
    let at: [f64; 3] = [f64::from(pos.x) + 0.5, f64::from(pos.y) + smoke::TOP, f64::from(pos.z) + 0.5];
    particles::spawn(world, "hugesmoke", at, [phase.cos() * smoke::DRIFT, smoke::RISE, phase.sin() * smoke::DRIFT]);
}

#[tile_behavior]
impl TileBehavior for Smoker
{
    fn onTick(&self, ctx: &mut BlockCtx)
    {
        if ctx.isAuthority()
        {
            return;
        }

        let ticks: i32 = bump(ctx);
        if ticks % smoke::EVERY == 0
        {
            swirl(ctx.world(), ctx.pos(), ticks);
        }
    }
}

// TileEntityTFPoppingJet bubbles lava for four seconds, TileEntityTFFlameJet burns for three
struct JetTile
{
    m_flame: bool,
}

mod jet
{
    pub const POP_TICKS: i32 = 80;
    pub const POP_EVERY: i32 = 20;
    pub const FLAME_TICKS: i32 = 60;
    pub const HURT_EVERY: i32 = 5;
    pub const DAMAGE: i32 = 2;
    pub const BURN_SECONDS: i32 = 15;
    // a flame jet scorches anything two blocks around and four up
    pub const REACH: f64 = 2.0;
    pub const HEIGHT: f64 = 4.0;
    pub const FLAME_RISE: f64 = 0.5;
    pub const FLAME_LEAN: f64 = 0.05;
}

fn finishJet(world: World, pos: BlockPos)
{
    if let Some(next) = nextJet(world, pos)
    {
        let _ = world.place(pos, jetName(next));
    }
}

fn flames(world: World, pos: BlockPos)
{
    let base: [f64; 3] = [f64::from(pos.x) + 0.5, f64::from(pos.y) + 1.0, f64::from(pos.z) + 0.5];
    world.particle("largesmoke", base, [0.0; 3]);

    let lean: f64 = jet::FLAME_LEAN;
    let spots: [([f64; 2], [f64; 2]); 5] = [
        ([0.0, 0.0], [0.0, 0.0]),
        ([-1.0, 0.0], [lean, 0.0]),
        ([0.0, -1.0], [0.0, lean]),
        ([1.0, 0.0], [-lean, 0.0]),
        ([0.0, 1.0], [0.0, -lean]),
    ];

    for (offset, motion) in spots
    {
        particles::spawn(
            world,
            "largeflame",
            [base[0] + offset[0], base[1], base[2] + offset[1]],
            [motion[0], jet::FLAME_RISE, motion[1]],
        );
    }
}

fn scorch(world: World, pos: BlockPos)
{
    let center: [f64; 3] = [f64::from(pos.x) + 0.5, f64::from(pos.y) + jet::HEIGHT / 2.0, f64::from(pos.z) + 0.5];
    for found in world.entitiesNear(center, jet::REACH + jet::HEIGHT)
    {
        let inside: bool = (found.pos[0] - center[0]).abs() <= jet::REACH
            && (found.pos[2] - center[2]).abs() <= jet::REACH
            && found.pos[1] >= f64::from(pos.y)
            && found.pos[1] <= f64::from(pos.y) + jet::HEIGHT;

        if !inside
        {
            continue;
        }

        let entity: Entity = Entity { world, id: found.id };
        if entity.field(EntityField::FireTicks) >= 0.0
            || crate::combat::hit(entity, jet::DAMAGE, None, crate::combat::kind::IN_FIRE)
        {
            entity.setField(EntityField::FireTicks, f64::from(jet::BURN_SECONDS * 20));
        }
    }
}

#[tile_behavior]
impl TileBehavior for JetTile
{
    fn onTick(&self, ctx: &mut BlockCtx)
    {
        let ticks: i32 = bump(ctx);
        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();

        let length: i32 = if self.m_flame
        {
            jet::FLAME_TICKS
        }
        else
        {
            jet::POP_TICKS
        };

        if ticks > length || (!self.m_flame && ticks >= length)
        {
            let _ = ctx.setTileData(&Counter::default());

            if ctx.isAuthority()
            {
                finishJet(world, pos);
            }

            return;
        }

        if !self.m_flame
        {
            if ticks % jet::POP_EVERY == 0
            {
                world.particle(
                    "lava",
                    [f64::from(pos.x) + 0.5, f64::from(pos.y) + 1.5, f64::from(pos.z) + 0.5],
                    [0.0; 3],
                );
                world.playSound(
                    crate::math::center(pos),
                    "liquid.lavapop",
                    0.2 + crate::math::fraction(world) * 0.2,
                    0.9 + crate::math::fraction(world) * 0.15,
                );
            }

            return;
        }

        if ticks % 2 == 0 && !ctx.isAuthority()
        {
            flames(world, pos);
        }

        if ticks % 4 == 0
        {
            world.playSound(
                crate::math::center(pos),
                "mob.ghast.fireball",
                1.0 + crate::math::fraction(world),
                crate::math::fraction(world) * 0.7 + 0.3,
            );
        }
        else if ticks == 1
        {
            world.playSound(
                crate::math::center(pos),
                "fire.ignite",
                1.0 + crate::math::fraction(world),
                crate::math::fraction(world) * 0.7 + 0.3,
            );
        }

        if ctx.isAuthority() && ticks % jet::HURT_EVERY == 0
        {
            scorch(world, pos);
        }
    }
}

// TileEntityTFTowerBuilder lays built blocks toward wherever the nearest player looks
mod builder
{
    pub const RANGE: f64 = 16.0;
    pub const EVERY: i32 = 10;
    pub const MAX_BLOCKS: i32 = 16;
    pub const TIMEOUT: i32 = 60;
    pub const RESTART_DELAY: i32 = 4;
}

#[derive(Serialize, Deserialize, Default)]
struct Builder
{
    m_making: bool,
    m_ticks: i32,
    m_stopped: i32,
    m_made: i32,
    m_last: Option<[i32; 3]>,
    m_player: Option<i32>,
}

struct BuilderTile;

fn nearestPlayer(world: World, at: [f64; 3], range: f64) -> Option<EntityInfo>
{
    world
        .players()
        .into_iter()
        .filter_map(Player::info)
        .filter(|info| info.world == world.dimension)
        .map(|info|
        {
            let distance: f64 = distanceSquared(info.pos, at);
            (distance, info)
        })
        .filter(|(distance, _)| *distance <= range * range)
        .min_by(|left, right| left.0.total_cmp(&right.0))
        .map(|(_, info)| info)
}

// TileEntityTFTowerBuilder.findNextFacing, pitch picks up or down first and yaw picks the side
fn facing(info: &EntityInfo) -> usize
{
    let quarter: f32 = 4.0 / 360.0;
    let pitch: i32 = floor(f64::from(info.pitch * quarter + 1.5)) & 3;
    match pitch
    {
        0 => return 1,
        2 => return 0,
        _ =>
        {}
    }

    match floor(f64::from(info.yaw * quarter + 0.5)) & 3
    {
        0 => 3,
        1 => 4,
        2 => 2,
        _ => 5,
    }
}

// beta Facing offsets for faces down up north south west east
const FACING: [[i32; 3]; 6] = NEIGHBORS;

pub(crate) fn startBuilding(world: World, pos: BlockPos)
{
    let mut state: Builder = world.tileData(pos);
    if state.m_making
    {
        return;
    }

    state.m_making = true;
    state.m_made = 0;
    state.m_last = Some([pos.x, pos.y, pos.z]);

    let _ = world.setTileData(pos, &state);
}

impl BuilderTile
{
    fn step(world: World, pos: BlockPos, state: &mut Builder)
    {
        let center: [f64; 3] = crate::math::center(pos);

        if state.m_player.is_none()
        {
            state.m_player = nearestPlayer(world, center, builder::RANGE).map(|info| info.id);
        }

        let looking: Option<EntityInfo> = state.m_player.and_then(|id| world.entity(id));

        state.m_ticks += 1;

        let mut blocked: bool = false;

        if let (Some(info), Some(last)) = (looking.as_ref(), state.m_last)
            && state.m_ticks % builder::EVERY == 0
        {
            let [dx, dy, dz] = FACING[facing(info)];
            let next: BlockPos = BlockPos::new(last[0] + dx, last[1] + dy, last[2] + dz);
            if state.m_made <= builder::MAX_BLOCKS && world.isAir(next)
            {
                let _ = world.place(next, deviceName(Device::Built));

                world.playSound(crate::math::center(next), "random.click", 1.0, 1.0);
                state.m_last = Some([next.x, next.y, next.z]);
                state.m_made += 1;
            }
            else
            {
                blocked = true;
            }
        }

        if blocked
        {
            state.m_making = false;
            state.m_player = None;
            state.m_stopped = 0;
        }
    }
}

#[tile_behavior]
impl TileBehavior for BuilderTile
{
    fn onTick(&self, ctx: &mut BlockCtx)
    {
        if !ctx.isAuthority()
        {
            return;
        }

        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();
        let mut state: Builder = ctx.tileData();
        if state.m_making
        {
            Self::step(world, pos, &mut state);

            let _ = ctx.setTileData(&state);

            return;
        }

        state.m_player = None;
        state.m_stopped += 1;

        let _ = ctx.setTileData(&state);

        if state.m_stopped == builder::TIMEOUT
        {
            let _ = world.place(pos, deviceName(Device::BuilderTimeout));
            let _ = world.scheduleTick(pos, world.block(pos), builder::RESTART_DELAY);
        }
    }
}

// TileEntityTFReverter snapshots the blocks around it and slowly undoes any change a player makes
mod reverter
{
    pub const RADIUS: i32 = 4;
    pub const RANGE: f64 = 16.0;
    pub const ODDS: i32 = 10;
    pub const SLOW_EVERY: i32 = 20;
    pub const SETTLE: i32 = 20;
    pub const OUTLINE_ODDS: i32 = 10;
    pub const OUTLINES: i32 = 12;
    pub const LINE_PARTICLES: i32 = 16;
}

#[derive(Serialize, Deserialize, Default)]
struct Reverter
{
    m_ticks: i32,
    m_slow: bool,
    m_quiet: i32,
}

// the blocks a reverter remembers, kept in memory as the source keeps them out of its saved data, the ids are only
// good for the content generation they were read in
struct Snapshot
{
    m_generation: i32,
    m_blocks: Vec<(i32, i32)>,
}

thread_local!
{
    static SNAPSHOTS: RefCell<HashMap<(i32, BlockPos), Snapshot>> = RefCell::new(HashMap::new());
}

struct ReverterTile;

fn around(pos: BlockPos) -> impl Iterator<Item = BlockPos>
{
    let radius: i32 = reverter::RADIUS;
    (-radius..=radius).flat_map(move |dx|
    {
        (-radius..=radius).flat_map(move |dy| (-radius..=radius).map(move |dz| pos.offset(dx, dy, dz)))
    })
}

pub(crate) fn deviceBlock(world: World, block: i32) -> bool
{
    let _ = world;

    [
        Device::Reappearing,
        Device::ReappearingActive,
        Device::Vanishing,
        Device::VanishingActive,
        Device::Locked,
        Device::Unlocked,
        Device::Builder,
        Device::BuilderActive,
        Device::BuilderTimeout,
        Device::Antibuilder,
        Device::GhastTrap,
        Device::GhastTrapActive,
        Device::Reactor,
        Device::ReactorActive,
        Device::Ghost,
        Device::GhostActive,
        Device::Built,
        Device::BuiltActive,
        Device::Debris,
        Device::FalseGold,
        Device::FalseDiamond,
    ]
    .iter()
    .any(|device| blockIs(block, deviceName(*device)))
}

// TileEntityTFReverter.isUnrevertable
fn unrevertable(world: World, there: i32, wanted: i32) -> bool
{
    let pair = |left: &str, right: &str| blockIs(there, left) && blockIs(wanted, right);
    deviceBlock(world, there)
        || deviceBlock(world, wanted)
        || pair("minecraft:redstone_lamp", "minecraft:lit_redstone_lamp")
        || pair("minecraft:lit_redstone_lamp", "minecraft:redstone_lamp")
        || blockTagged(there, "minecraft:water")
        || blockTagged(wanted, "minecraft:water")
        || blockIs(wanted, "minecraft:tnt")
}

// TileEntityTFReverter.revertBlock, true when the change is still pending
fn revert(world: World, pos: BlockPos, there: i32, wanted: (i32, i32)) -> bool
{
    if there == 0 && !World::blockInfo(wanted.0).solid
    {
        return false;
    }
    if unrevertable(world, there, wanted.0)
    {
        return false;
    }
    if world.random(reverter::ODDS) != 0
    {
        return true;
    }

    let replace: &'static str = if wanted.0 == 0
    {
        "minecraft:air"
    }
    else
    {
        deviceName(Device::Antibuilt)
    };

    crate::world::palette::withPlain(replace, |state|
    {
        let (block, meta): (i32, i32) = state.numeric();
        if there == 0
        {
            let _ = world.breakEffect(pos, block, meta);
        }
        else if block == 0
        {
            let _ = world.breakEffect(pos, there, world.meta(pos));
            let _ = world.dropBlockItems(pos, there, world.meta(pos));
        }

        let _ = state.place(world, pos, true);
    });

    true
}

impl ReverterTile
{
    fn scan(world: World, pos: BlockPos, blocks: &mut [(i32, i32)]) -> bool
    {
        let mut reverted: bool = false;

        for (index, at) in around(pos).enumerate()
        {
            let there: i32 = world.block(at);
            let wanted: (i32, i32) = blocks[index];
            if wanted.0 == there
            {
                continue;
            }

            if revert(world, at, there, wanted)
            {
                reverted = true;
            }
            else
            {
                blocks[index] = (there, world.meta(at));
            }
        }

        reverted
    }

    fn outline(world: World, pos: BlockPos)
    {
        let radius: f64 = f64::from(reverter::RADIUS);
        let low: [f64; 3] = [f64::from(pos.x) - radius, f64::from(pos.y) - radius, f64::from(pos.z) - radius];
        let high: [f64; 3] =
            [f64::from(pos.x) + radius + 1.0, f64::from(pos.y) + radius + 1.0, f64::from(pos.z) + radius + 1.0];
        let edge: i32 = world.random(reverter::OUTLINES);

        let corner = |bits: i32|
        {
            [
                if bits & 1 != 0 { high[0] } else { low[0] },
                if bits & 2 != 0 { high[1] } else { low[1] },
                if bits & 4 != 0 { high[2] } else { low[2] },
            ]
        };
        // twelve cube edges as pairs of corner bit masks
        let edges: [(i32, i32); 12] =
            [(0, 1), (2, 3), (4, 5), (6, 7), (0, 2), (1, 3), (4, 6), (5, 7), (0, 4), (1, 5), (2, 6), (3, 7)];
        let (from, to): (i32, i32) = edges[edge as usize];
        particles::line(world, "reddust", corner(from), corner(to), reverter::LINE_PARTICLES);
        particles::line(world, "reddust", crate::math::center(pos), corner(to), reverter::LINE_PARTICLES);
    }
}

#[tile_behavior]
impl TileBehavior for ReverterTile
{
    fn onTick(&self, ctx: &mut BlockCtx)
    {
        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();
        let mut state: Reverter = ctx.tileData();

        if nearestPlayer(world, crate::math::center(pos), reverter::RANGE).is_none()
        {
            SNAPSHOTS.with(|snapshots| snapshots.borrow_mut().remove(&(world.dimension, pos)));
            if state.m_ticks != 0
            {
                let _ = ctx.setTileData(&Reverter::default());
            }

            return;
        }

        state.m_ticks += 1;

        if !ctx.isAuthority()
        {
            particles::spawn(
                world,
                "reddust",
                [f64::from(pos.x) + 0.5, f64::from(pos.y) + 0.5, f64::from(pos.z) + 0.5],
                [0.0; 3],
            );

            if world.random(reverter::OUTLINE_ODDS) == 0
            {
                for _ in 0..3
                {
                    Self::outline(world, pos);
                }
            }

            return;
        }

        let generation: i32 = contentGeneration();
        let key: (i32, BlockPos) = (world.dimension, pos);
        let fresh: bool = SNAPSHOTS
            .with(|snapshots| snapshots.borrow().get(&key).is_some_and(|known| known.m_generation == generation));
        if !fresh
        {
            let blocks: Vec<(i32, i32)> = around(pos).map(|at| (world.block(at), world.meta(at))).collect();
            SNAPSHOTS.with(|snapshots|
            {
                snapshots.borrow_mut().insert(key, Snapshot { m_generation: generation, m_blocks: blocks })
            });
            state.m_slow = true;
        }

        if !state.m_slow || state.m_ticks % reverter::SLOW_EVERY == 0
        {
            let reverted: bool = SNAPSHOTS.with(|snapshots|
            {
                snapshots.borrow_mut().get_mut(&key).is_some_and(|known| Self::scan(world, pos, &mut known.m_blocks))
            });
            if reverted
            {
                state.m_slow = false;
                state.m_quiet = 0;
            }
            else
            {
                state.m_quiet += 1;
                state.m_slow = state.m_quiet > reverter::SETTLE;
            }
        }

        let _ = ctx.setTileData(&state);
    }
}

// TileEntityTFGhastTrapInactive charges from mini ghasts dying near it, three make it ready
mod trap
{
    pub const CHARGES: i32 = 3;
    pub const PULL_REACH: [f64; 3] = [6.0, 16.0, 6.0];
    pub const PULL_HEIGHT: f64 = 16.0;
    pub const PULL: f64 = -0.1;
    pub const HURT_ODDS: i32 = 10;
    pub const GHAST_DAMAGE: i32 = 10;
    pub const BOSS_DAMAGE: i32 = 3;
    pub const RUN_TICKS: i32 = 120;
    pub const WARMUP: i32 = 30;
    pub const SPIN: i32 = 80;
    pub const SPIRAL: i32 = 100;
}

#[derive(Serialize, Deserialize, Default)]
struct Charge
{
    m_ticks: i32,
    m_charges: i32,
}

pub(crate) fn trapCharged(world: World, pos: BlockPos) -> bool
{
    world.tileData::<Charge>(pos).m_charges >= trap::CHARGES
}

fn within(at: [f64; 3], center: [f64; 3], reach: [f64; 3]) -> bool
{
    (0..3).all(|axis| (at[axis] - center[axis]).abs() <= reach[axis] + 0.5)
}

struct TrapTile;

#[tile_behavior]
impl TileBehavior for TrapTile
{
    fn onTick(&self, ctx: &mut BlockCtx)
    {
        if ctx.isAuthority()
        {
            return;
        }

        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();
        let mut charge: Charge = ctx.tileData();
        charge.m_ticks += 1;

        let level: i32 = charge.m_charges.min(trap::CHARGES);
        let ticks: i32 = charge.m_ticks;
        let _ = ctx.setTileData(&charge);
        let top: [f64; 3] = [f64::from(pos.x) + 0.5, f64::from(pos.y) + 1.5, f64::from(pos.z) + 0.5];

        if level >= 1 && ticks % 10 == 0
        {
            particles::sparkle(world, pos, "reddust");
            world.playSound(top, "note.harp", 1.0, 1.0);
        }

        if level >= 2
        {
            let at: [f64; 3] = [
                f64::from(pos.x) + 0.1 + f64::from(crate::math::fraction(world)) * 0.8,
                f64::from(pos.y) + 1.05,
                f64::from(pos.z) + 0.5,
            ];
            world.particle("smoke", at, [0.0; 3]);

            if ticks % 10 == 0
            {
                world.playSound(top, "note.harp", 1.2, 0.8);
            }
        }

        if level >= trap::CHARGES
        {
            let at: [f64; 3] = [f64::from(pos.x) + 0.5, f64::from(pos.y) + 1.05, f64::from(pos.z) + 0.5];
            world.particle("largesmoke", at, [0.0, 0.05, 0.0]);
            particles::sparkle(world, pos, "reddust");

            if ticks % 5 == 0
            {
                world.playSound(top, "note.harp", 1.5, 2.0);
            }
        }
    }
}

// TileEntityTFGhastTrapActive pulls every ghast above it down and hurts it for six seconds
struct TrapActiveTile;

fn pullGhasts(world: World, pos: BlockPos)
{
    let center: [f64; 3] = [f64::from(pos.x) + 0.5, f64::from(pos.y) + trap::PULL_HEIGHT + 0.5, f64::from(pos.z) + 0.5];
    for found in world.entitiesNear(center, trap::PULL_REACH[1] + trap::PULL_REACH[0])
    {
        let kind: String = found.kind.as_ref().map(ToString::to_string).unwrap_or_default();
        let boss: bool = kind == crate::entities::UR_GHAST;
        let ghast: bool = boss
            || kind == "minecraft:ghast"
            || kind == crate::entities::MINI_GHAST
            || kind == crate::entities::TOWER_GHAST;

        if !ghast || !within(found.pos, center, trap::PULL_REACH)
        {
            continue;
        }

        let entity: Entity = Entity { world, id: found.id };
        let hover: f64 = if boss { 2.5 } else { 1.5 };
        let motion: [f64; 3] = [
            (found.pos[0] - f64::from(pos.x) - 0.5) * trap::PULL,
            (found.pos[1] - f64::from(pos.y) - hover) * trap::PULL,
            (found.pos[2] - f64::from(pos.z) - 0.5) * trap::PULL,
        ];

        let _ = entity.setMotion(motion);

        if world.random(trap::HURT_ODDS) == 0
        {
            entity.damage(
                if boss
                {
                    trap::BOSS_DAMAGE
                }
                else
                {
                    trap::GHAST_DAMAGE
                },
                None,
            );
        }

        crate::entities::ghastTrapped(entity, boss);
    }
}

fn trapEffects(world: World, pos: BlockPos, ticks: i32)
{
    let top: [f64; 3] = [f64::from(pos.x) + 0.5, f64::from(pos.y) + 1.5, f64::from(pos.z) + 0.5];

    if ticks > trap::SPIRAL && ticks % 4 == 0
    {
        swirl(world, pos, ticks);
    }
    else if ticks < trap::SPIRAL
    {
        let phase: f64 = f64::from(ticks) / smoke::SWIRL;
        let (dx, dz): (f64, f64) = (phase.cos() * 2.5, phase.sin() * 2.5);
        let rise: f64 = 20.0;
        let base: [f64; 3] = [f64::from(pos.x) + 0.5, f64::from(pos.y) + 1.0, f64::from(pos.z) + 0.5];

        for motion in [
            [dx, rise, dz],
            [-dx, rise, -dz],
            [-dx, rise / 2.0, dz],
            [dx, rise / 2.0, -dz],
            [dx / 2.0, rise / 4.0, dz / 2.0],
            [-dx / 2.0, rise / 4.0, -dz / 2.0],
        ]
        {
            particles::spawn(world, "ghasttrap", base, motion);
        }
    }

    let sound: &str = if ticks < trap::WARMUP
    {
        "twilightforest.mob.urghast.trapwarmup"
    }
    else if ticks < trap::SPIN
    {
        "twilightforest.mob.urghast.trapon"
    }
    else
    {
        "twilightforest.mob.urghast.trapspindown"
    };
    world.playSound(top, sound, 1.0, 4.0);
}

#[tile_behavior]
impl TileBehavior for TrapActiveTile
{
    fn onTick(&self, ctx: &mut BlockCtx)
    {
        let ticks: i32 = bump(ctx);
        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();

        if !ctx.isAuthority()
        {
            trapEffects(world, pos, ticks);
            return;
        }

        pullGhasts(world, pos);

        if ticks >= trap::RUN_TICKS
        {
            let _ = world.place(pos, deviceName(Device::GhastTrap));
        }
    }
}

// TileEntityTFCReactorActive cages itself in false ores, eats a growing sphere and blows open three nether bubbles
mod reactor
{
    pub const OFFSET: i32 = 10;
    pub const PRIMARY_START: i32 = 80;
    pub const SECONDARY_START: i32 = 120;
    pub const TERTIARY_START: i32 = 160;
    pub const GROWTH: i32 = 40;
    pub const PRIMARY_END: i32 = 249;
    pub const SHELL_END: i32 = 200;
    pub const BUBBLE_END: i32 = 129;
    pub const NETHER_END: i32 = 160;
    pub const BLOW: i32 = 350;
    pub const SOUND_END: i32 = 250;
    pub const EVERY: i32 = 5;
    pub const SPREAD: i32 = 3;
    pub const GHASTS: i32 = 3;
    pub const POWER: f32 = 2.0;
    pub const FUZZ: i32 = 8;
}

#[derive(Serialize, Deserialize, Default)]
struct Reactor
{
    m_ticks: i32,
    m_secondary: Option<[i32; 3]>,
    m_tertiary: [i32; 3],
}

struct ReactorTile;

fn signedSpread(world: World) -> i32 { reactor::SPREAD * if world.random(2) == 0 { 1 } else { -1 } }

fn cage(world: World, pos: BlockPos)
{
    let gold: StateHandle = names::handle(deviceName(Device::FalseGold));
    let diamond: StateHandle = names::handle(deviceName(Device::FalseDiamond));

    for dx in -1..=1
    {
        for dy in -1..=1
        {
            for dz in -1..=1
            {
                if dx == 0 && dy == 0 && dz == 0
                {
                    continue;
                }

                let edges: i32 = [dx, dy, dz].iter().filter(|value| **value != 0).count() as i32;
                // corners and face centres are false diamond, edges false gold
                let block: &StateHandle = if edges == 2 { &gold } else { &diamond };
                let _ = block.place(world, pos.offset(dx, dy, dz), false);
            }
        }
    }
}

// TileEntityTFCReactorActive.drawBlob, one shell of a rough sphere per call
fn blob(world: World, center: BlockPos, radius: i32, block: &'static str, fuzz: i32, nether: bool)
{
    for dx in 0..=radius
    {
        let fuzzX: i32 = (fuzz + dx) % reactor::FUZZ;

        for dy in 0..=radius
        {
            let fuzzY: i32 = (fuzz + dy) % reactor::FUZZ;

            for dz in 0..=radius
            {
                let longest: i32 = dx.max(dy).max(dz);
                let (mid, low): (i32, i32) = if dx == longest
                {
                    (dy.max(dz), dy.min(dz))
                }
                else if dy == longest
                {
                    (dx.max(dz), dx.min(dz))
                }
                else
                {
                    (dx.max(dy), dx.min(dy))
                };
                let distance: i32 = longest + (f64::from(mid) * 0.5 + f64::from(low) * 0.25) as i32;
                if distance != radius || (dx == 0 && dy == 0 && dz == 0)
                {
                    continue;
                }

                let sx: i32 = if fuzzX & 2 == 0 { dx } else { -dx };
                let sy: i32 = if fuzzX & 4 == 0 { dy } else { -dy };
                let sz: i32 = if fuzzX & 1 == 0 { dz } else { -dz };
                transform(world, center.offset(sx, sy, sz), block, fuzzY, nether);
            }
        }
    }
}

fn transform(world: World, pos: BlockPos, block: &'static str, fuzz: i32, nether: bool)
{
    let there: i32 = world.block(pos);
    if there != 0 && World::blockInfo(there).hardness < 0.0
    {
        return;
    }

    if fuzz == 0 && there != 0
    {
        let _ = world.breakEffect(pos, there, world.meta(pos));
    }

    if nether && there != 0
    {
        let _ = world.place(pos, "minecraft:netherrack");

        if world.isAir(pos.offset(0, 1, 0)) && fuzz % 3 == 0
        {
            let _ = world.place(pos.offset(0, 1, 0), "minecraft:fire");
        }

        return;
    }

    let _ = crate::world::palette::withPlain(block, |state| state.place(world, pos, true));
}

fn spawnGhast(world: World, around: BlockPos)
{
    let spread: f64 = 3.0;
    let at: [f64; 3] = [
        f64::from(around.x) - 1.5 + f64::from(crate::math::fraction(world)) * spread,
        f64::from(around.y) - 1.5 + f64::from(crate::math::fraction(world)) * spread,
        f64::from(around.z) - 1.5 + f64::from(crate::math::fraction(world)) * spread,
    ];

    let _ = world.spawnEntity(crate::entities::MINI_GHAST, at);
}

impl ReactorTile
{
    fn bubble(world: World, at: BlockPos, start: i32, ticks: i32)
    {
        let phase: i32 = ticks - start;
        if phase >= reactor::OFFSET && phase <= reactor::BUBBLE_END
        {
            blob(
                world,
                at,
                (phase - reactor::OFFSET) / reactor::GROWTH,
                "minecraft:air",
                phase - reactor::OFFSET,
                false,
            );
        }

        if (0..=reactor::NETHER_END).contains(&phase)
        {
            blob(world, at, phase / reactor::GROWTH, "minecraft:air", phase, true);
        }
    }

    fn run(world: World, pos: BlockPos, state: &Reactor)
    {
        let ticks: i32 = state.m_ticks;
        let [sx, sy, sz] = state.m_secondary.unwrap_or_default();
        let [tx, ty, tz] = state.m_tertiary;

        if ticks % reactor::EVERY == 0
        {
            if ticks == reactor::EVERY
            {
                cage(world, pos);
            }

            let primary: i32 = ticks - reactor::PRIMARY_START;
            if primary >= reactor::OFFSET && primary <= reactor::PRIMARY_END
            {
                blob(
                    world,
                    pos,
                    (primary - reactor::OFFSET) / reactor::GROWTH,
                    "minecraft:air",
                    primary - reactor::OFFSET,
                    false,
                );
            }

            if primary <= reactor::SHELL_END
            {
                blob(world, pos, primary / reactor::GROWTH, deviceName(Device::Debris), ticks, false);
            }

            Self::bubble(world, pos.offset(sx, sy, sz), reactor::SECONDARY_START, ticks);
            Self::bubble(world, pos.offset(tx, ty, tz), reactor::TERTIARY_START, ticks);
        }

        if ticks >= reactor::BLOW
        {
            for _ in 0..reactor::GHASTS
            {
                spawnGhast(world, pos.offset(sx, sy, sz));
                spawnGhast(world, pos.offset(tx, ty, tz));
            }

            let _ = world.explode(crate::math::center(pos), reactor::POWER);
            let _ = world.clear(pos);
        }
    }
}

#[tile_behavior]
impl TileBehavior for ReactorTile
{
    fn onTick(&self, ctx: &mut BlockCtx)
    {
        let world: World = ctx.world();
        let mut state: Reactor = ctx.tileData();
        state.m_ticks += 1;

        if state.m_secondary.is_none()
        {
            let secondary: [i32; 3] = [signedSpread(world), signedSpread(world), signedSpread(world)];
            let mut tertiary: [i32; 3] = [signedSpread(world), signedSpread(world), signedSpread(world)];
            if tertiary == secondary
            {
                tertiary = tertiary.map(|value| -value);
            }

            state.m_secondary = Some(secondary);
            state.m_tertiary = tertiary;
        }

        let _ = ctx.setTileData(&state);

        if ctx.isAuthority()
        {
            Self::run(world, ctx.pos(), &state);
        }
        else if state.m_ticks % reactor::EVERY == 0 && state.m_ticks <= reactor::SOUND_END
        {
            let loud: f32 = state.m_ticks as f32 / 100.0;
            world.playSound(crate::math::center(ctx.pos()), "portal.portal", loud, loud);
        }
    }
}

pub(crate) fn registerTiles(ctx: &mut RegisterCtx) -> Result<()>
{
    let tiles: [(&str, bool, Rc<dyn TileBehavior>); 8] = [
        (SMOKER_TILE, false, Rc::new(Smoker)),
        (POPPING_TILE, false, Rc::new(JetTile { m_flame: false })),
        (FLAME_TILE, false, Rc::new(JetTile { m_flame: true })),
        (BUILDER_TILE, false, Rc::new(BuilderTile)),
        (REVERTER_TILE, false, Rc::new(ReverterTile)),
        (TRAP_TILE, true, Rc::new(TrapTile)),
        (TRAP_ACTIVE_TILE, false, Rc::new(TrapActiveTile)),
        (REACTOR_TILE, false, Rc::new(ReactorTile)),
    ];

    for (name, sync, behavior) in tiles
    {
        ctx.tileEntities().register(
            name,
            TileEntitySpec { inventory: 0, ticks: true, sync, behavior: None },
            Some(behavior),
        )?;
    }

    Ok(())
}
