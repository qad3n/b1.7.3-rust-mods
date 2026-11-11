use crate::core::{blockTexture, dir, ids, itemTexture, math, own, random, sound, stacks};
use b173_api::prelude::*;
use b173_api::terrain::math::floor;
use std::collections::BTreeSet;
use std::rc::Rc;

// an ExplosionIC2's share of blocks dropped and its entity damage multiplier
#[derive(Clone, Copy)]
pub(crate) struct Blast
{
    pub(crate) m_drop: f32,
    pub(crate) m_damage: f32,
}

// mod_IC2.explodeMachineAt
pub(crate) const MACHINE: Blast = Blast { m_drop: 0.75, m_damage: 0.75 };

// TileEntityLuminator.poof
pub(crate) const LUMINATOR: Blast = Blast { m_drop: 0.85, m_damage: 2.0 };

// TileEntityMatter.injectEnergy on an overload, and a melting reactor
pub(crate) const MATTER: Blast = Blast { m_drop: 0.01, m_damage: 1.5 };

// placed dynamite, apart from the dynamite item of the same IC2 name
const DYNAMITE_BLOCK: &str = "ic2:dynamite_block";
const REMOTE_BLOCK: &str = "ic2:remote_dynamite_block";

// EntityMiningLaser.explode
const LASER: Blast = Blast { m_drop: 0.85, m_damage: 0.55 };

mod ray
{
    // the padding Explosion adds to every block's resistance
    pub const RESIST_PAD: f32 = 0.3;
    // entities within twice the size take damage and a push
    pub const REACH: f32 = 2.0;
    pub const DAMAGE: f64 = 8.0;
    // doExplosionB's bang and its particles
    pub const VOLUME: f32 = 4.0;
    pub const PITCH: f32 = 0.7;
    pub const PITCH_SPREAD: f32 = 0.2;
    pub const PUFF_SPEED: f64 = 0.5;
    pub const PUFF_FALLOFF: f64 = 0.1;
    pub const PUFF_FLOOR: f64 = 0.3;
}

// doExplosionA's second half: everything within twice the size takes damage by distance and cover, and a push
fn hurt(world: World, at: [f64; 3], size: f32, damage: f32)
{
    let reach: f64 = f64::from(size * ray::REACH);
    let low: [f64; 3] = at.map(|value| f64::from(floor(value - reach - 1.0)));
    let high: [f64; 3] = at.map(|value| f64::from(floor(value + reach + 1.0)));

    for info in world.entitiesIn(low, high)
    {
        let offset: [f64; 3] = [info.pos[0] - at[0], info.pos[1] - at[1], info.pos[2] - at[2]];
        let distance: f64 = offset.iter().map(|value| value * value).sum::<f64>().sqrt();
        let near: f64 = distance / reach;
        if near > 1.0 || distance == 0.0
        {
            continue;
        }

        let exposure: f64 = (1.0 - near) * f64::from(world.blockDensity(at, info.id));
        let amount: i32 =
            (((exposure * exposure + exposure) / 2.0 * ray::DAMAGE * reach + 1.0) * f64::from(damage)) as i32;
        let entity: Entity = Entity { world, id: info.id };
        if let Err(error) = entity.damageAs(amount, None, "minecraft:explosion")
        {
            warn(&format!("an IC2 explosion could not hurt entity {}: {error}", info.id));
        }

        let push: [f64; 3] = [0, 1, 2].map(|axis| info.motion[axis] + offset[axis] / distance * exposure);
        let _ = entity.setMotion(push);
    }
}

// doExplosionB: the bang, the puffs and every collected block taken out, last collected first
fn destroy(world: World, at: [f64; 3], size: f32, blocks: &BTreeSet<(i32, i32, i32)>, drop: f32)
{
    let pitch: f32 = (1.0 + (random::float(world) - random::float(world)) as f32 * ray::PITCH_SPREAD) * ray::PITCH;
    world.playSound(at, "random.explode", ray::VOLUME, pitch);

    for &(x, y, z) in blocks.iter().rev()
    {
        let spot: [f64; 3] = [
            f64::from(x) + random::float(world),
            f64::from(y) + random::float(world),
            f64::from(z) + random::float(world),
        ];
        let offset: [f64; 3] = [spot[0] - at[0], spot[1] - at[1], spot[2] - at[2]];
        let distance: f64 = offset.iter().map(|value| value * value).sum::<f64>().sqrt().max(f64::MIN_POSITIVE);
        let speed: f64 = ray::PUFF_SPEED / (distance / f64::from(size) + ray::PUFF_FALLOFF)
            * (random::float(world) * random::float(world) + ray::PUFF_FLOOR);
        let motion: [f64; 3] = offset.map(|value| value / distance * speed);
        world.particle("explode", [(spot[0] + at[0]) / 2.0, (spot[1] + at[1]) / 2.0, (spot[2] + at[2]) / 2.0], motion);
        world.particle("smoke", spot, motion);

        let pos: BlockPos = BlockPos::new(x, y, z);
        if world.block(pos) > 0
            && let Err(error) = world.blastBlock(pos, drop)
        {
            warn(&format!("an IC2 explosion could not take out {pos:?}: {error}"));
        }
    }
}

// ExplosionIC2: Explosion with its own drop rate and entity damage, run by the host so a nuke fits one call
pub(crate) fn blast(world: World, at: [f64; 3], power: f32, blast: Blast)
{
    if !world.isAuthority()
    {
        return;
    }

    if let Err(error) = world.explodeWith(at, power, blast.m_drop, blast.m_damage, false)
    {
        warn(&format!("an IC2 explosion at {at:?} failed: {error}"));
    }
}

// PointExplosion: dynamite's box of blocks, anything an explosion of this size can break
fn pointBlast(world: World, at: [f64; 3])
{
    const POWER: f32 = 1.0;
    const DROP: f32 = 1.0;
    const DAMAGE: f32 = 0.8;
    const RESIST_SCALE: f32 = 10.0;

    let mut center: [i32; 3] = at.map(|value| value as i32);
    for axis in [0, 2]
    {
        if center[axis] < 0
        {
            center[axis] -= 1;
        }
    }

    let mut blocks: BTreeSet<(i32, i32, i32)> = BTreeSet::new();
    for x in center[0] - 1..=center[0] + 1
    {
        for y in center[1] - 1..=center[1] + 1
        {
            for z in center[2] - 1..=center[2] + 1
            {
                let id: i32 = world.block(BlockPos::new(x, y, z));
                let resistance: f32 = if id > 0
                {
                    World::explosionResistance(id)
                }
                else
                {
                    0.0
                };
                if POWER >= resistance / RESIST_SCALE
                {
                    blocks.insert((x, y, z));
                }
            }
        }
    }

    let point: [f64; 3] = center.map(f64::from);
    hurt(world, point, POWER, DAMAGE);
    destroy(world, point, POWER, &blocks, DROP);
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
enum Charge
{
    #[default]
    Industrial,
    Nuke,
}

// BlockITNT.getExplosionEntity: fuse, power, drop rate and entity damage
struct Primed
{
    m_charge: Charge,
    m_block: &'static str,
    m_fuse: i32,
    m_power: f32,
    m_drop: f32,
    m_damage: f32,
}

const PRIMED: [Primed; 2] = [
    Primed
    {
        m_charge: Charge::Industrial,
        m_block: "industrial_tnt",
        m_fuse: 60,
        m_power: 3.0,
        m_drop: 0.9,
        m_damage: 0.3,
    },
    Primed { m_charge: Charge::Nuke, m_block: "nuke", m_fuse: 300, m_power: 40.0, m_drop: 0.05, m_damage: 1.5 },
];

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug, Default)]
struct Fuse
{
    m_charge: Charge,
    m_fuse: i32,
    m_motion: [f64; 3],
    m_started: bool,
}

mod fall
{
    // EntityIC2Explosive.onUpdate: falls like primed TNT
    pub const GRAVITY: f64 = 0.04;
    pub const DRAG: f64 = 0.98;
    pub const GROUND_DRAG: f64 = 0.7;
    pub const BOUNCE: f64 = -0.5;
    pub const POP: f64 = 0.2;
    pub const NUDGE: f64 = 0.02;
    pub const SMOKE_HEIGHT: f64 = 0.5;
}

// EntityIC2Explosive
struct PrimedCharge;

#[entity_behavior]
impl EntityBehavior for PrimedCharge
{
    fn onTick(&self, entity: Entity, info: &EntityInfo)
    {
        let world: World = entity.world;
        if !world.isAuthority()
        {
            return;
        }

        let mut fuse: Fuse = entity.data();
        let primed: &Primed = PRIMED.iter().find(|row| row.m_charge == fuse.m_charge).unwrap_or(&PRIMED[0]);
        let mut motion: [f64; 3] = if fuse.m_started
        {
            fuse.m_motion
        }
        else
        {
            start(world)
        };
        fuse.m_started = true;

        motion[1] -= fall::GRAVITY;
        motion = motion.map(|value| value * fall::DRAG);
        if entity.flag(EntityField::OnGround)
        {
            motion = [motion[0] * fall::GROUND_DRAG, motion[1] * fall::BOUNCE, motion[2] * fall::GROUND_DRAG];
        }

        fuse.m_fuse -= 1;
        if fuse.m_fuse < 0
        {
            entity.kill();
            blast(world, info.pos, primed.m_power, Blast { m_drop: primed.m_drop, m_damage: primed.m_damage });
            return;
        }

        world.particle("smoke", [info.pos[0], info.pos[1] + fall::SMOKE_HEIGHT, info.pos[2]], [0.0; 3]);
        fuse.m_motion = motion;
        let _ = entity.setMotion(motion);
        let _ = entity.setData(&fuse);
    }
}

// the constructor's small random hop
fn start(world: World) -> [f64; 3]
{
    let angle: f64 = random::float(world) * std::f64::consts::TAU;
    [-angle.to_radians().sin() * fall::NUDGE, fall::POP, -angle.to_radians().cos() * fall::NUDGE]
}

fn prime(world: World, pos: BlockPos, charge: Charge, fuse: Option<i32>)
{
    let primed: &Primed = PRIMED.iter().find(|row| row.m_charge == charge).unwrap_or(&PRIMED[0]);
    let Ok(entity) = world.spawnEntity("ic2:primed_explosive", dir::center(pos))
    else
    {
        warn(&format!("could not prime the explosive at {pos:?}"));
        return;
    };

    let state: Fuse =
        Fuse { m_charge: charge, m_fuse: fuse.unwrap_or(primed.m_fuse), m_motion: [0.0; 3], m_started: false };
    let _ = entity.setData(&state);
    world.playSound(dir::center(pos), "random.fuse", 1.0, 1.0);
}

// BlockIC2Explosive: lit by redstone, a flint and steel click or another blast; ITNT also goes off by hand
struct ExplosiveBlock
{
    m_charge: Charge,
}

mod lit
{
    pub const META: u8 = 1;
    pub const CHAIN_SHARE: i32 = 4;
    pub const CHAIN_FLOOR: i32 = 8;
}

#[block_behavior]
impl BlockBehavior for ExplosiveBlock
{
    fn onAdded(&self, ctx: &mut BlockCtx)
    {
        let world: World = ctx.world();
        if ctx.isAuthority() && world.isPowered(ctx.pos())
        {
            prime(world, ctx.pos(), self.m_charge, None);
            let _ = world.setBlock(ctx.pos(), 0, 0);
        }
    }

    fn onNeighborChanged(&self, ctx: &mut BlockCtx, neighbor: i32)
    {
        let world: World = ctx.world();
        if !ctx.isAuthority() || neighbor <= 0 || !world.isPowered(ctx.pos())
        {
            return;
        }

        prime(world, ctx.pos(), self.m_charge, None);
        let _ = world.setBlock(ctx.pos(), 0, 0);
    }

    fn onClicked(&self, ctx: &mut BlockCtx, player: Player)
    {
        if player.held().is_some_and(|held| held.item.is("minecraft:flint_and_steel"))
        {
            ctx.setMeta(lit::META);
        }
    }

    // a short random fuse when another blast reaches it
    fn onDestroyedByExplosion(&self, ctx: &mut BlockCtx)
    {
        let world: World = ctx.world();
        let primed: &Primed = PRIMED.iter().find(|row| row.m_charge == self.m_charge).unwrap_or(&PRIMED[0]);
        let fuse: i32 = world.random(primed.m_fuse / lit::CHAIN_SHARE) + primed.m_fuse / lit::CHAIN_FLOOR;
        prime(world, ctx.pos(), self.m_charge, Some(fuse));
    }

    // onBlockDestroyedByPlayer: a lit block or ITNT starts its fuse, an unlit nuke drops back as an item
    fn onDestroyedByPlayer(&self, ctx: &mut BlockCtx)
    {
        let world: World = ctx.world();
        if !ctx.isAuthority()
        {
            return;
        }

        let lit: bool = ctx.meta() & 1 != 0 || self.m_charge == Charge::Industrial;
        if lit
        {
            prime(world, ctx.pos(), self.m_charge, None);
            return;
        }

        let block: &str = PRIMED.iter().find(|row| row.m_charge == self.m_charge).map_or("nuke", |row| row.m_block);
        stacks::drop(world, ctx.pos(), &ItemStack::of(own(block).as_str(), 1));
    }
}

mod stick
{
    pub const THROWN_FUSE: i32 = 100;
    pub const PLACED_FUSE: i32 = 40;
    pub const CHAIN_FUSE: i32 = 5;
    pub const STICKY_BURN: i32 = 3;
    pub const GROUND_LIFE: i32 = 200;
    pub const SMOKE_BELOW: i32 = 100;
    pub const WATER_DELAY: i32 = 2000;
    pub const DRAG: f64 = 0.98;
    pub const WATER_DRAG: f64 = 0.75;
    pub const GRAVITY: f64 = 0.04;
    pub const BACK_OFF: f64 = 0.05;
    pub const SPREAD: f64 = 0.0075;
    pub const BOUNCE: f64 = -0.3;
    pub const SKID: f64 = 0.75;
    pub const EYE: f64 = 1.62;
    pub const SIDE: f64 = 0.16;
    pub const DROP: f64 = 0.1;
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug, Default)]
struct Throw
{
    m_sticky: bool,
    m_fuse: i32,
    m_motion: [f64; 3],
    m_stuck: Option<[i32; 3]>,
    m_grounded: i32,
}

// EntityDynamite.onUpdate: flies with drag and gravity, bounces off blocks or sticks to them, smokes before it goes
struct Dynamite;

#[entity_behavior]
impl EntityBehavior for Dynamite
{
    fn onTick(&self, entity: Entity, info: &EntityInfo)
    {
        let world: World = entity.world;
        if !world.isAuthority()
        {
            return;
        }

        let mut throw: Throw = entity.data();
        throw.m_fuse -= 1;
        if throw.m_fuse < 0
        {
            entity.kill();
            pointBlast(world, info.pos);
            return;
        }

        if throw.m_fuse < stick::SMOKE_BELOW && throw.m_fuse % 2 == 0
        {
            world.particle("smoke", [info.pos[0], info.pos[1] + fall::SMOKE_HEIGHT, info.pos[2]], [0.0; 3]);
        }

        if let Some([x, y, z]) = throw.m_stuck
        {
            throw.m_grounded += 1;
            if throw.m_grounded >= stick::GROUND_LIFE
            {
                entity.kill();
                return;
            }

            if throw.m_sticky && !world.isAir(BlockPos::new(x, y, z))
            {
                throw.m_fuse -= stick::STICKY_BURN;
                let _ = entity.setMotion([0.0; 3]);
                let _ = entity.setPosition(info.pos);
                let _ = entity.setData(&throw);
                return;
            }
        }

        let motion: [f64; 3] = throw.m_motion;
        let from: [f64; 3] = info.pos;
        let to: [f64; 3] = [from[0] + motion[0], from[1] + motion[1], from[2] + motion[2]];
        let mut next: [f64; 3] = to;
        let mut moving: [f64; 3] = motion;

        match world.rayTrace(from, to, false)
        {
            Some(hit) =>
            {
                let remain: [f64; 3] = [hit.point[0] - from[0], hit.point[1] - from[1], hit.point[2] - from[2]];
                let length: f64 = remain.iter().map(|value| value * value).sum::<f64>().sqrt().max(f64::MIN_POSITIVE);
                next = [0, 1, 2].map(|axis| hit.point[axis] - remain[axis] / length * stick::BACK_OFF);
                moving = [
                    motion[0] * (stick::SKID - random::float(world)),
                    motion[1] * stick::BOUNCE,
                    motion[2] * (stick::SKID - random::float(world)),
                ];
                throw.m_stuck = Some([hit.pos.x, hit.pos.y, hit.pos.z]);
            }

            None => throw.m_stuck = None,
        }

        let wet: bool = entity.flag(EntityField::InWater);
        let drag: f64 = if wet { stick::WATER_DRAG } else { stick::DRAG };
        if wet
        {
            throw.m_fuse += stick::WATER_DELAY;
        }

        moving = moving.map(|value| value * drag);
        moving[1] -= stick::GRAVITY;
        throw.m_motion = moving;
        let _ = entity.setPosition(next);
        let _ = entity.setMotion([0.0; 3]);
        let _ = entity.setData(&throw);
    }
}

fn spawnDynamite(world: World, at: [f64; 3], sticky: bool, motion: [f64; 3], fuse: i32)
{
    let Ok(entity) = world.spawnEntity("ic2:dynamite_stick", at)
    else
    {
        warn(&format!("could not spawn dynamite at {at:?}"));
        return;
    };

    let throw: Throw = Throw { m_sticky: sticky, m_fuse: fuse, m_motion: motion, m_stuck: None, m_grounded: 0 };
    let _ = entity.setData(&throw);
}

// ItemDynamite: thrown on right click, the plain stick placed on a block face
struct DynamiteItem
{
    m_sticky: bool,
}

#[item_behavior]
impl ItemBehavior for DynamiteItem
{
    fn onRightClick(&self, world: World, player: Player, stack: ItemStack) -> Use
    {
        const BOW_VOLUME: f32 = 0.5;
        const BOW_PITCH: f32 = 0.4;
        const PITCH_LOW: f32 = 0.8;
        const PITCH_SPREAD: f32 = 0.4;

        let Some(info) = player.info()
        else
        {
            return Use::Pass;
        };

        let pitch: f32 = BOW_PITCH / (random::float(world) as f32 * PITCH_SPREAD + PITCH_LOW);
        world.playSound(info.pos, "random.bow", BOW_VOLUME, pitch);

        if world.isAuthority()
        {
            let yaw: f64 = f64::from(info.yaw).to_radians();
            let at: [f64; 3] = [
                info.pos[0] - yaw.cos() * stick::SIDE,
                info.pos[1] + stick::EYE - stick::DROP,
                info.pos[2] - yaw.sin() * stick::SIDE,
            ];
            let look: [f64; 3] = math::look(info.yaw, info.pitch);
            let motion: [f64; 3] = look.map(|value| value + random::gaussian(world) * stick::SPREAD);
            spawnDynamite(world, at, self.m_sticky, motion, stick::THROWN_FUSE);
        }

        Use::Done(stacks::less(stack, 1))
    }

    fn onUseOnBlock(&self, world: World, _player: Player, stack: ItemStack, pos: BlockPos, face: i32) -> Use
    {
        if self.m_sticky
        {
            return Use::Pass;
        }

        let at: BlockPos = dir::step(pos, face);
        let Some(block) = ids::block(DYNAMITE_BLOCK)
        else
        {
            return Use::Pass;
        };

        if !world.isAir(at) || !mounted(world, at)
        {
            return Use::Done(Some(stack));
        }

        let _ = world.setBlock(at, block, 0);
        Use::Done(stacks::less(stack, 1))
    }
}

fn solid(world: World, pos: BlockPos) -> bool
{
    let block: i32 = world.block(pos);
    block != 0 && World::blockInfo(block).opaque
}

// BlockDynamite.canPlaceBlockAt: a solid block beside it or below
fn mounted(world: World, pos: BlockPos) -> bool
{
    [dir::WEST, dir::EAST, dir::NORTH, dir::SOUTH, dir::DOWN].iter().any(|face| solid(world, dir::step(pos, *face)))
}

// the face a placed stick leans on, as BlockDynamite stores it: 1 to 4 on a wall, 5 on the floor
fn lean(world: World, pos: BlockPos) -> Option<u8>
{
    const FLOOR: u8 = 5;

    if solid(world, pos.offset(0, -1, 0))
    {
        return Some(FLOOR);
    }

    [dir::WEST, dir::EAST, dir::NORTH, dir::SOUTH]
        .iter()
        .position(|face| solid(world, dir::step(pos, *face)))
        .map(|index| index as u8 + 1)
}

thread_local!
{
    // ItemRemote.remotes: the dynamite each remote frequency sets off
    static REMOTES: std::cell::RefCell<Vec<(i32, BlockPos, i32)>> = const { std::cell::RefCell::new(Vec::new()) };
}

fn linked(world: World, pos: BlockPos) -> bool
{
    REMOTES.with(|remotes| remotes.borrow().iter().any(|(dim, at, _)| *dim == world.dimension && *at == pos))
}

// BlockDynamite: a torch shaped stick that goes off with redstone, a breaking player or a blast
struct DynamiteBlock
{
    m_remote: bool,
}

#[block_behavior]
impl BlockBehavior for DynamiteBlock
{
    fn canPlaceAt(&self, ctx: &mut BlockCtx) -> bool { mounted(ctx.world(), ctx.pos()) }

    fn onAdded(&self, ctx: &mut BlockCtx)
    {
        let world: World = ctx.world();
        if !ctx.isAuthority()
        {
            return;
        }

        if world.isPowered(ctx.pos())
        {
            spawnDynamite(world, dir::center(ctx.pos()), false, [0.0; 3], stick::PLACED_FUSE);
            let _ = world.setBlock(ctx.pos(), 0, 0);
            return;
        }

        match lean(world, ctx.pos())
        {
            Some(meta) => ctx.setMeta(meta),
            None =>
            {
                spawnDynamite(world, dir::center(ctx.pos()), false, [0.0; 3], stick::CHAIN_FUSE);
                let _ = world.setBlock(ctx.pos(), 0, 0);
            }
        }
    }

    // BlockDynamite.updateTick: a remote charge whose remote is gone turns back into plain dynamite
    fn onUpdateTick(&self, ctx: &mut BlockCtx)
    {
        let world: World = ctx.world();
        if self.m_remote
            && ctx.isAuthority()
            && !linked(world, ctx.pos())
            && let Some(plain) = ids::block(DYNAMITE_BLOCK)
        {
            let _ = world.setBlock(ctx.pos(), plain, i32::from(ctx.meta()));
        }
    }

    fn onNeighborChanged(&self, ctx: &mut BlockCtx, neighbor: i32)
    {
        let world: World = ctx.world();
        if !ctx.isAuthority()
        {
            return;
        }

        if neighbor > 0 && world.isPowered(ctx.pos()) || !mounted(world, ctx.pos())
        {
            let fuse: i32 = if mounted(world, ctx.pos())
            {
                stick::PLACED_FUSE
            }
            else
            {
                stick::CHAIN_FUSE
            };
            spawnDynamite(world, dir::center(ctx.pos()), false, [0.0; 3], fuse);
            let _ = world.setBlock(ctx.pos(), 0, 0);
        }
    }

    fn onDestroyedByExplosion(&self, ctx: &mut BlockCtx)
    {
        spawnDynamite(ctx.world(), dir::center(ctx.pos()), false, [0.0; 3], stick::CHAIN_FUSE);
    }

    fn onDestroyedByPlayer(&self, ctx: &mut BlockCtx)
    {
        if ctx.isAuthority()
        {
            spawnDynamite(ctx.world(), dir::center(ctx.pos()), false, [0.0; 3], stick::PLACED_FUSE);
            ctx.world().playSound(dir::center(ctx.pos()), "random.fuse", 1.0, 1.0);
        }
    }
}

// ItemRemote: tags dynamite with its frequency, a right click sets off everything on it
struct Remote;

mod remote
{
    pub const FREQUENCIES: i32 = 9001;
}

#[item_behavior]
impl ItemBehavior for Remote
{
    fn onUseOnBlock(&self, world: World, _player: Player, stack: ItemStack, pos: BlockPos, _face: i32) -> Use
    {
        let block: i32 = world.block(pos);
        let plain: bool = ids::block(DYNAMITE_BLOCK) == Some(block);
        let wired: bool = ids::block(REMOTE_BLOCK) == Some(block);
        if !world.isAuthority()
        {
            return if plain || wired
            {
                Use::Done(Some(stack))
            }
            else
            {
                Use::Pass
            };
        }

        let frequency: i32 = if stack.damage == 0
        {
            world.random(remote::FREQUENCIES)
        }
        else
        {
            stack.damage
        };
        let stack: ItemStack = ItemStack { damage: frequency, ..stack };

        if plain && let Some(remoteBlock) = ids::block(REMOTE_BLOCK)
        {
            let _ = world.setBlock(pos, remoteBlock, world.meta(pos));
        }

        if plain || wired
        {
            REMOTES.with(|remotes|
            {
                let mut remotes = remotes.borrow_mut();
                remotes.retain(|(dim, at, _)| !(*dim == world.dimension && *at == pos));
                remotes.push((world.dimension, pos, frequency));
            });
        }

        Use::Done(Some(stack))
    }

    fn onRightClick(&self, world: World, player: Player, stack: ItemStack) -> Use
    {
        if let Some(info) = player.info()
        {
            world.playSound(info.pos, &sound("remote"), 1.0, 1.0);
        }

        let fired: Vec<BlockPos> = REMOTES.with(|remotes|
        {
            let mut remotes = remotes.borrow_mut();
            let fired: Vec<BlockPos> = remotes
                .iter()
                .filter(|(dim, _, frequency)| *dim == world.dimension && *frequency == stack.damage)
                .map(|(_, at, _)| *at)
                .collect();
            remotes.retain(|(dim, _, frequency)| !(*dim == world.dimension && *frequency == stack.damage));
            fired
        });

        for pos in fired
        {
            let _ = world.setBlock(pos, 0, 0);
            spawnDynamite(world, dir::center(pos), false, [0.0; 3], stick::CHAIN_FUSE);
        }

        Use::Done(Some(stack))
    }
}

mod beam
{
    pub const LIFETIME_SPEED: f64 = 1.0;
    pub const SIDE: f64 = 0.16;
    pub const EYE: f64 = 1.62;
    pub const DROP: f64 = 0.1;
    pub const RESIST_SCALE: f32 = 6.3;
    pub const DROP_CHANCE: f32 = 0.9;
    pub const IGNITE_CHANCE: i32 = 10;
    pub const FIRE_PER_DAMAGE: f64 = 10.0;
    pub const HIT_RADIUS: f64 = 0.3;
    pub const OWNER_GRACE: i32 = 5;
    pub const BLAST: f32 = 2.0;
}

// EntityMiningLaser.unmineableBlocks
const UNMINEABLE: [&str; 8] = [
    "minecraft:brick_block",
    "minecraft:obsidian",
    "minecraft:flowing_lava",
    "minecraft:lava",
    "minecraft:flowing_water",
    "minecraft:water",
    "minecraft:bedrock",
    "minecraft:glass",
];

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug, Default)]
struct Beam
{
    m_range: f32,
    m_explosive: bool,
    m_start: [f64; 3],
    m_heading: [f64; 3],
    m_owner: i32,
    m_age: i32,
}

// EntityMiningLaser: burns through blocks until its range runs out, sets what it hits alight
struct LaserBeam;

fn burnable(block: i32) -> bool
{
    nameOf(RegistryKind::Block, block).is_some_and(|id|
    {
        let tags: Vec<Identifier> = tagsOf(RegistryKind::Block, &id);
        let tagged: bool = tags.iter().any(|tag|
        {
            ["minecraft:logs", "minecraft:leaves", "minecraft:mineable/axe"].contains(&tag.to_string().as_str())
        });
        tagged || id.to_string() == "minecraft:wool"
    })
}

#[entity_behavior]
impl EntityBehavior for LaserBeam
{
    fn onTick(&self, entity: Entity, info: &EntityInfo)
    {
        let world: World = entity.world;
        if !world.isAuthority()
        {
            return;
        }

        let mut beam: Beam = entity.data();
        beam.m_age += 1;
        let travelled: f64 = (0..3).map(|axis| (info.pos[axis] - beam.m_start[axis]).powi(2)).sum::<f64>().sqrt();

        if travelled.sqrt() > f64::from(beam.m_range) || entity.flag(EntityField::InWater)
        {
            if beam.m_explosive && !entity.flag(EntityField::InWater)
            {
                blast(world, info.pos, beam::BLAST, LASER);
            }

            entity.kill();
            return;
        }

        let from: [f64; 3] = info.pos;
        let to: [f64; 3] = [0, 1, 2].map(|axis| from[axis] + beam.m_heading[axis] * beam::LIFETIME_SPEED);
        let exclude: Option<i32> = (beam.m_age < beam::OWNER_GRACE).then_some(beam.m_owner);
        let target: Option<EntityInfo> = world
            .entitiesAlong(from, to, beam::HIT_RADIUS, exclude)
            .into_iter()
            .find(|hit| hit.id != entity.id && hit.health.is_some());

        if let Some(target) = target
        {
            if beam.m_explosive
            {
                blast(world, info.pos, beam::BLAST, LASER);
                entity.kill();
                return;
            }

            let damage: i32 = (f64::from(beam.m_range).powi(2) - travelled).max(0.0).sqrt() as i32;
            let victim: Entity = Entity { world, id: target.id };
            if victim.damage(damage, Some(beam.m_owner))
            {
                let fire: f64 = victim.field(EntityField::FireTicks);
                victim.setField(EntityField::FireTicks, fire + beam::FIRE_PER_DAMAGE * f64::from(damage));
            }

            entity.kill();
            return;
        }

        if let Some(hit) = world.rayTrace(from, to, false)
        {
            if beam.m_explosive
            {
                blast(world, info.pos, beam::BLAST, LASER);
                entity.kill();
                return;
            }

            let block: i32 = world.block(hit.pos);
            if UNMINEABLE.iter().any(|name| ids::block(name) == Some(block))
            {
                entity.kill();
                return;
            }

            beam.m_range -= (World::explosionResistance(block) + ray::RESIST_PAD) / beam::RESIST_SCALE;
            let _ = world.blastBlock(hit.pos, beam::DROP_CHANCE);
            if world.random(beam::IGNITE_CHANCE) == 0
                && burnable(block)
                && let Some(fire) = ids::block("minecraft:fire")
            {
                let _ = world.setBlock(hit.pos, fire, 0);
            }
        }

        let _ = entity.setPosition(to);
        let _ = entity.setMotion([0.0; 3]);
        let _ = entity.setData(&beam);
    }
}

// a laser shot from a player's eye toward a yaw and pitch
pub(crate) fn fireLaser(world: World, shooter: &EntityInfo, aim: [f32; 2], range: f32, explosive: bool)
{
    let yaw: f64 = f64::from(shooter.yaw).to_radians();
    let at: [f64; 3] = [
        shooter.pos[0] - yaw.cos() * beam::SIDE,
        shooter.pos[1] + beam::EYE - beam::DROP,
        shooter.pos[2] - yaw.sin() * beam::SIDE,
    ];
    let Ok(entity) = world.spawnEntity("ic2:mining_laser", at)
    else
    {
        warn("could not fire the mining laser");
        return;
    };

    let heading: [f64; 3] = math::look(aim[0], aim[1]);
    let beam: Beam =
        Beam { m_range: range, m_explosive: explosive, m_start: at, m_heading: heading, m_owner: shooter.id, m_age: 0 };
    let _ = entity.setData(&beam);
}

fn projectile(texture: &str) -> EntitySpec
{
    const SIZE: f32 = 0.5;
    const TRACKING: i32 = 64;
    const UPDATES: i32 = 5;

    EntitySpec
    {
        template: Identifier::parse("minecraft:projectile").ok(),
        texture: Some(texture.to_owned()),
        size: Some([SIZE, SIZE]),
        tracking_range: Some(TRACKING),
        update_frequency: Some(UPDATES),
        send_velocity: Some(true),
        model: Some(EntityModelSpec { shadow: 0.0, ..EntityModelSpec::default() }),
        ..EntitySpec::default()
    }
}

fn explosiveFaces(name: &str) -> FaceTextures
{
    FaceTextures
    {
        top: Some(blockTexture(&format!("{name}_top"))),
        bottom: Some(blockTexture(&format!("{name}_bottom"))),
        side: Some(blockTexture(&format!("{name}_side"))),
        front: None,
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    const DYNAMITE_STACK: i32 = 16;

    ctx.entities().register(
        "primed_explosive",
        projectile(&blockTexture("industrial_tnt_side")),
        Some(Rc::new(PrimedCharge)),
    )?;
    ctx.entities().register("dynamite_stick", projectile(&itemTexture("dynamite")), Some(Rc::new(Dynamite)))?;
    ctx.entities().register("mining_laser", projectile("ic2:entity/laser"), Some(Rc::new(LaserBeam)))?;

    for primed in &PRIMED
    {
        ctx.blocks().register(
            primed.m_block,
            BlockSpecBuilder::template("minecraft:block")
                .material("tnt")
                .sound("grass")
                .hardness(0.0)
                .faces(explosiveFaces(primed.m_block))
                .drops(Vec::new())
                .behavior(ExplosiveBlock { m_charge: primed.m_charge }),
        )?;
    }

    for (name, sprite, remote) in
        [("dynamite_block", "dynamite", false), ("remote_dynamite_block", "remote_dynamite", true)]
    {
        ctx.blocks().register(
            name,
            BlockSpecBuilder::template("minecraft:torch")
                .material("tnt")
                .sound("grass")
                .hardness(0.0)
                .texture(&blockTexture(sprite))
                .drops(Vec::new())
                .tickRandomly()
                .behavior(DynamiteBlock { m_remote: remote }),
        )?;
    }

    ctx.items().register(
        "dynamite",
        ItemSpecBuilder::new()
            .texture(&itemTexture("dynamite"))
            .maxStack(DYNAMITE_STACK)
            .behavior(DynamiteItem { m_sticky: false }),
    )?;
    ctx.items().register(
        "sticky_dynamite",
        ItemSpecBuilder::new()
            .texture(&itemTexture("sticky_dynamite"))
            .maxStack(DYNAMITE_STACK)
            .behavior(DynamiteItem { m_sticky: true }),
    )?;
    ctx.items().register(
        "dynamite_o_mote",
        ItemSpecBuilder::new().texture(&itemTexture("dynamite_o_mote")).maxStack(1).behavior(Remote),
    )?;
    Ok(())
}
