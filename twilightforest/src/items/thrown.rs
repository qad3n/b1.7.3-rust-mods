use super::magic::{held, holdInUse};
use super::{put, texture};
use crate::math::look;
use crate::models::{Program, tf_cube_of_annihilation, tf_spike_block};
use crate::names::{block, item};
use b173_api::math::{blockOf, distance, length, sub};
use b173_api::prelude::*;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;

// ItemTFChainBlock with EntityTFChainBlock and ItemTFCubeOfAnnihilation with EntityTFCubeOfAnnihilation: a spiked
// block on a chain that smashes what it hits and swings back, and a cube that eats through blocks where the player
// looks while they hold it
const CHAIN_BALL: &str = "chain_block_ball";
const CUBE: &str = "cube_of_annihilation_cube";

mod chain
{
    pub const DURABILITY: i32 = 99;
    pub const SPEED: f64 = 1.5;
    pub const GRAVITY: f64 = 0.05;
    pub const REACH: f64 = 16.0;
    pub const CATCH: f64 = 1.0;
    pub const DAMAGE: i32 = 10;
    pub const MAX_SMASH: i32 = 12;
    pub const BOUNCE: f64 = 0.6;
    // EntityTFChainBlock.affectBlocksInAABB smashes blocks under 7 blast resistance, beta's hardness times five
    pub const BREAKABLE: f32 = 1.4;
    pub const RETURN_RATE: f32 = 0.03;
    pub const HIT_RADIUS: f64 = 0.6;
}

mod cube
{
    pub const LOOK: f64 = 16.0;
    pub const MAX_SPEED: f64 = 0.5;
    pub const SLOW: f64 = 0.5;
    pub const CATCH: f64 = 1.5;
    pub const DAMAGE: i32 = 10;
    // EntityTFCubeOfAnnihilation.canAnnihilate keeps to blocks under 8 blast resistance, besides the castle's own
    pub const BREAKABLE: f32 = 1.6;
}

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct Flight
{
    m_owner: i32,
    m_velocity: [f64; 3],
    m_returning: bool,
    m_smashed: i32,
    m_age: i32,
    m_blocked: bool,
}

thread_local!
{
    // ItemTFChainBlock.launchedBlocksMap and the cube's thrown flag, the ball or cube each player has out
    static OUT: RefCell<HashMap<(i32, &'static str), i32>> = RefCell::new(HashMap::new());
}

fn outFor(player: i32, kind: &'static str) -> Option<i32> { OUT.with(|out| out.borrow().get(&(player, kind)).copied()) }

fn setOut(player: i32, kind: &'static str, entity: Option<i32>)
{
    OUT.with(|out|
    {
        let mut out: RefMut<HashMap<(i32, &'static str), i32>> = out.borrow_mut();

        match entity
        {
            Some(id) => out.insert((player, kind), id),
            None => out.remove(&(player, kind)),
        };
    });
}

fn launch(world: World, player: Player, kind: &'static str, speed: f64) -> bool
{
    let Some(info) = player.info()
    else
    {
        return false;
    };

    let alive: bool =
        outFor(player.id, kind).is_some_and(|id| Entity { world, id }.info().is_some_and(|info| info.alive));

    if alive
    {
        return false;
    }

    let aim: [f64; 3] = look(info.yaw, info.pitch);

    let Ok(thrown) = world.spawnEntity(&format!("twilightforest:{kind}"), info.pos)
    else
    {
        warn(&format!("cannot throw {kind}"));
        return false;
    };

    let velocity: [f64; 3] = aim.map(|axis| axis * speed);
    let _ = thrown.setMotion(velocity);
    let _ = thrown.setData(&Flight { m_owner: player.id, m_velocity: velocity, ..Flight::default() });

    setOut(player.id, kind, Some(thrown.id));
    world.playSound(info.pos, "random.bow", 1.0, 1.0 / (crate::math::fraction(world) * 0.4 + 1.2));
    true
}

fn breakable(world: World, pos: BlockPos, limit: f32) -> bool
{
    let block: i32 = world.block(pos);
    let info: BlockInfo = World::blockInfo(block);
    block != 0 && info.hardness >= 0.0 && info.hardness < limit
}

fn smash(world: World, pos: BlockPos)
{
    let (block, meta): (i32, i32) = (world.block(pos), world.meta(pos));
    let _ = world.dropBlockItems(pos, block, meta);
    let _ = world.clear(pos);
    let _ = world.breakEffect(pos, block, meta);
}

fn hitLiving(world: World, from: [f64; 3], to: [f64; 3], owner: i32, damage: i32) -> bool
{
    let hit: Option<EntityInfo> =
        world.entitiesAlong(from, to, chain::HIT_RADIUS, Some(owner)).into_iter().find(|hit| hit.health.is_some());

    hit.is_some_and(|hit|
    {
        crate::combat::hit(Entity { world, id: hit.id }, damage, Some(owner), crate::combat::kind::PLAYER)
    })
}

struct ChainBall;

#[entity_behavior]
impl EntityBehavior for ChainBall
{
    // EntityTFChainBlock.onUpdate and onImpact
    fn onTick(&self, entity: Entity, info: &EntityInfo)
    {
        let world: World = entity.world;
        if !world.isAuthority()
        {
            return;
        }

        let mut state: Flight = entity.data();
        state.m_age += 1;
        // only a player throws the ball, one summoned without a thrower falls apart
        let thrower: Option<Player> = world.players().into_iter().find(|player| player.id == state.m_owner);
        let Some(owner) = thrower.and_then(Player::info)
        else
        {
            entity.kill();
            return;
        };

        let away: f64 = distance(owner.pos, info.pos);
        if !state.m_returning && away > chain::REACH
        {
            state.m_returning = true;
        }

        if state.m_returning && away < chain::CATCH
        {
            setOut(state.m_owner, CHAIN_BALL, None);
            entity.kill();
            return;
        }

        let to: [f64; 3] = info.pos;
        let from: [f64; 3] = sub(to, info.motion);
        if hitLiving(world, from, to, state.m_owner, chain::DAMAGE)
        {
            state.m_age += 60;
        }

        if let Some(hit) = world.rayTrace(from, to, false)
        {
            if !state.m_returning
            {
                world.playSound(to, "random.anvil_land", 0.125, crate::math::fraction(world));
            }

            if state.m_smashed < chain::MAX_SMASH
            {
                if World::blockInfo(world.block(hit.pos)).hardness > 0.3
                {
                    let axis: usize = match hit.face
                    {
                        0 | 1 => 1,
                        2 | 3 => 2,
                        _ => 0,
                    };
                    state.m_velocity = state.m_velocity.map(|axis| axis * chain::BOUNCE);
                    state.m_velocity[axis] = -state.m_velocity[axis];
                }

                if breakable(world, hit.pos, chain::BREAKABLE)
                {
                    smash(world, hit.pos);
                    state.m_smashed += 1;
                }
            }

            state.m_returning = true;
        }

        let motion: [f64; 3] = if state.m_returning
        {
            let back: [f64; 3] = [owner.pos[0] - to[0], owner.pos[1] - 1.2 - to[1], owner.pos[2] - to[2]];
            let length: f64 = length(back).max(1.0e-4);
            let age: f64 = f64::from((state.m_age as f32 * chain::RETURN_RATE).min(1.0));
            [0, 1, 2].map(|axis| state.m_velocity[axis] * (1.0 - age) + back[axis] / length * 2.0 * age)
        }
        else
        {
            state.m_velocity[1] -= chain::GRAVITY;
            state.m_velocity
        };
        let _ = entity.setMotion(motion);
        let _ = entity.setData(&state);
    }

    fn pose(&self, _input: &PoseInput) -> Pose
    {
        let mut program: Program = Program::new();

        for part in 0..tf_spike_block::PIVOTS.len() as u16
        {
            program.rest(part);
        }

        Pose { pre: Vec::new(), model: program.take() }
    }
}

// EntityTFCubeOfAnnihilation.canAnnihilate, the castle's deadrock, bricks, force fields and thorns always go
fn annihilable(world: World, pos: BlockPos) -> bool
{
    let block: i32 = world.block(pos);
    let castle: [&str; 6] = [
        block::SOLID_DEADROCK,
        block::DEADROCK,
        block::CASTLE_BRICK,
        block::BROWN_THORNS,
        block::GREEN_THORNS,
        block::CASTLE_RUNE_BRICK,
    ];
    castle.iter().any(|name| blockIs(block, name)) || breakable(world, pos, cube::BREAKABLE)
}

struct Cube;

#[entity_behavior]
impl EntityBehavior for Cube
{
    // EntityTFCubeOfAnnihilation.onUpdate, it chases the spot 16 blocks along the owner's gaze while they use it
    fn onTick(&self, entity: Entity, info: &EntityInfo)
    {
        let world: World = entity.world;
        if !world.isAuthority()
        {
            return;
        }

        let mut state: Flight = entity.data();
        let player: Option<Player> = world.players().into_iter().find(|player| player.id == state.m_owner);
        let Some((player, owner)) = player.and_then(|player| player.info().map(|info| (player, info)))
        else
        {
            entity.kill();
            return;
        };

        // EntityTFCubeOfAnnihilation.isReturning, it comes back once the owner lets go of the use key
        let using: bool = player.itemInUse() > 0;
        let returning: bool = state.m_blocked || !using;
        if returning && distance(owner.pos, info.pos) < cube::CATCH
        {
            setOut(state.m_owner, CUBE, None);
            entity.kill();
            return;
        }

        let aim: [f64; 3] = look(owner.yaw, owner.pitch);

        let dest: [f64; 3] = if returning
        {
            owner.pos
        }
        else
        {
            [0, 1, 2].map(|axis| owner.pos[axis] + aim[axis] * cube::LOOK)
        };
        let mut motion: [f64; 3] =
            [0, 1, 2].map(|axis| info.motion[axis] - (info.pos[axis] + if axis == 1 { 0.5 } else { 0.0 } - dest[axis]));
        let speed: f64 = length(motion);
        motion = if speed > cube::MAX_SPEED
        {
            motion.map(|axis| axis / (speed / cube::MAX_SPEED))
        }
        else
        {
            motion.map(|axis| axis * cube::SLOW)
        };
        let _ = entity.setMotion(motion);

        let to: [f64; 3] = info.pos;
        let from: [f64; 3] = sub(to, info.motion);
        hitLiving(world, from, to, state.m_owner, cube::DAMAGE);

        let center: BlockPos = blockOf(to);

        for (dx, dy, dz) in (-1..=1).flat_map(|dx| (0..=1).flat_map(move |dy| (-1..=1).map(move |dz| (dx, dy, dz))))
        {
            let pos: BlockPos = center.offset(dx, dy, dz);
            if world.block(pos) == 0
            {
                continue;
            }

            if annihilable(world, pos)
            {
                let _ = world.clear(pos);

                world.playSound(
                    crate::math::center(pos),
                    "random.fizz",
                    0.125,
                    crate::math::fraction(world) * 0.25 + 0.75,
                );
                world.particle("largesmoke", crate::math::center(pos), [0.0; 3]);
            }
            else
            {
                state.m_blocked = true;
            }
        }

        let _ = entity.setData(&state);
    }

    fn pose(&self, _input: &PoseInput) -> Pose
    {
        let mut program: Program = Program::new();

        for part in 0..tf_cube_of_annihilation::PIVOTS.len() as u16
        {
            program.rest(part);
        }

        Pose { pre: Vec::new(), model: program.take() }
    }
}

struct ChainBlock;

#[item_behavior]
impl ItemBehavior for ChainBlock
{
    // ItemTFChainBlock.onItemRightClick, the block flies out and the chain stays raised while the use key is held
    fn onRightClick(&self, world: World, player: Player, _stack: ItemStack) -> Use
    {
        if world.isAuthority() && launch(world, player, CHAIN_BALL, chain::SPEED)
        {
            super::magic::wearHeld(player, 1);
        }

        holdInUse(player, held::LONGEST)
    }

    fn useDuration(&self, _stack: ItemStack) -> i32 { held::LONGEST }

    fn useAction(&self, _stack: ItemStack) -> UseAction { UseAction::Block }
}

struct CubeOfAnnihilation;

#[item_behavior]
impl ItemBehavior for CubeOfAnnihilation
{
    // ItemTFCubeOfAnnihilation.onItemRightClick, the cube flies out and works while the use key is held
    fn onRightClick(&self, world: World, player: Player, _stack: ItemStack) -> Use
    {
        if world.isAuthority()
        {
            launch(world, player, CUBE, cube::MAX_SPEED);
        }

        holdInUse(player, held::LONGEST)
    }

    fn useDuration(&self, _stack: ItemStack) -> i32 { held::LONGEST }

    fn useAction(&self, _stack: ItemStack) -> UseAction { UseAction::Block }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    put(
        ctx,
        item::CHAIN_BLOCK,
        ItemSpecBuilder::new()
            .name("Block and Chain")
            .texture(&texture("chainblock"))
            .maxStack(1)
            .maxDamage(chain::DURABILITY)
            .behavior(ChainBlock),
    )?;
    put(
        ctx,
        item::CUBE_OF_ANNIHILATION,
        ItemSpecBuilder::new()
            .name("Cube of Annihilation")
            .texture(&texture("cubeofannihilation"))
            .maxStack(1)
            .behavior(CubeOfAnnihilation),
    )?;
    for (name, texture, size, model, behavior) in [
        (
            CHAIN_BALL,
            "twilightforest:model/blockgoblin",
            0.6,
            tf_spike_block::model as fn() -> EntityModelSpec,
            Rc::new(ChainBall) as Rc<dyn EntityBehavior>,
        ),
        (
            CUBE,
            "twilightforest:model/cubeofannihilation",
            1.0,
            tf_cube_of_annihilation::model as fn() -> EntityModelSpec,
            Rc::new(Cube) as Rc<dyn EntityBehavior>,
        ),
    ]
    {
        ctx.entities().register(
            name,
            EntitySpec
            {
                template: Identifier::parse("minecraft:projectile").ok(),
                texture: Some(texture.to_owned()),
                size: Some([size, size]),
                tracking_range: Some(64),
                update_frequency: Some(1),
                send_velocity: Some(true),
                model: Some(EntityModelSpec { shadow: 0.3, ..model() }),
                ..EntitySpec::default()
            },
            Some(behavior),
        )?;
    }
    Ok(())
}
