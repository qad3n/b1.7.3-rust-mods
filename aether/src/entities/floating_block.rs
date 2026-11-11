use crate::names::handleOf;
use b173_api::math::blockOf;
use b173_api::prelude::*;

const NAME: &str = "floating_block";
const FIRE: &str = "minecraft:fire";

mod float
{
    // BlockFloating and EntityFloatingBlock
    pub const DELAY: i32 = 3;
    pub const LIFT: f64 = 0.04;
    pub const DRAG: f64 = 0.98;
    pub const LIFETIME: f64 = 200.0;
    pub const CENTER: f64 = 0.5;
    pub const REACH: f64 = 1.5;
    pub const SIDE: f64 = 0.49;
    pub const WORLD_TOP: f64 = 128.0;
    // EntitiesAether registers it with a 160 block range and every tick updates
    pub const TRACKING: i32 = 160;
    pub const UPDATES: i32 = 1;
}

// saved by name so a carried block survives a remap of numeric ids
#[derive(Serialize, Deserialize, Default, Clone)]
struct Carried
{
    m_state: Option<StateData>,
}

fn canContinue(world: World, pos: BlockPos) -> bool
{
    world.isAir(pos) || world.is(pos, FIRE) || World::blockInfo(world.block(pos)).liquid
}

// BlockFloating: gravitite ore rises whenever nothing holds it down, enchanted gravitite only while powered
pub(crate) struct Floating
{
    pub m_leveled: bool,
}

impl Floating
{
    fn schedule(ctx: &mut BlockCtx)
    {
        if ctx.isAuthority()
        {
            let _ = ctx.world().scheduleTick(ctx.pos(), ctx.block(), float::DELAY);
        }
    }
}

#[block_behavior]
impl BlockBehavior for Floating
{
    fn onAdded(&self, ctx: &mut BlockCtx) { Self::schedule(ctx); }

    fn onNeighborChanged(&self, ctx: &mut BlockCtx, _neighbor: i32) { Self::schedule(ctx); }

    fn onUpdateTick(&self, ctx: &mut BlockCtx)
    {
        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();
        if !ctx.isAuthority()
            || (self.m_leveled && !world.isPowered(pos))
            || pos.y < 0
            || !canContinue(world, pos.offset(0, 1, 0))
        {
            return;
        }

        lift(world, pos);
    }
}

// the block at a spot becomes a floating block entity rising from there
pub(crate) fn lift(world: World, pos: BlockPos)
{
    let carried: Carried = Carried { m_state: world.blockState(pos) };
    let _ = world.clear(pos);
    let center: [f64; 3] = [pos.x as f64 + float::CENTER, pos.y as f64 + float::CENTER, pos.z as f64 + float::CENTER];
    if let Ok(entity) = world.spawnEntity(&format!("aether:{NAME}"), center)
    {
        show(entity, &carried);

        let _ = entity.setSynced(&carried);
        let _ = entity.setMotion([0.0, float::LIFT, 0.0]);
    }
}

// the entity draws the block it carries, which the engine knows only by numeric id
fn show(entity: Entity, carried: &Carried)
{
    if let Some(state) = &carried.m_state
    {
        entity.setField(EntityField::Block, f64::from(handleOf(state).block()));
    }
}

fn dropAndVanish(entity: Entity, info: &EntityInfo, carried: Carried)
{
    if let Some(state) = carried.m_state
    {
        let _ = entity.world.spawnItem(info.pos, ItemStack::of(ItemRef::Named(state.block), 1));
    }

    entity.kill();
}

// EntityFloatingBlock.onUpdate: speeds upward, lifts whatever stands on it and settles under the first solid block
struct FloatingBlock;

#[entity_behavior]
impl EntityBehavior for FloatingBlock
{
    fn onTick(&self, entity: Entity, info: &EntityInfo)
    {
        let world: World = entity.world;
        let carried: Carried = entity.synced();

        if !world.isAuthority()
        {
            show(entity, &carried);
            return;
        }

        if carried.m_state.is_none()
        {
            entity.kill();
            return;
        }

        let rise: f64 = info.motion[1];
        let _ = entity.setMotion([
            info.motion[0] * float::DRAG,
            rise * float::DRAG + float::LIFT,
            info.motion[2] * float::DRAG,
        ]);

        let bottom: f64 = info.pos[1] - float::SIDE;

        for rider in world.entitiesNear(info.pos, float::REACH)
        {
            let standing: bool = (rider.pos[0] - info.pos[0]).abs() < float::REACH - float::SIDE
                && (rider.pos[2] - info.pos[2]).abs() < float::REACH - float::SIDE
                && rider.pos[1] >= bottom;

            if rider.id == entity.id || !standing || rider.kind.as_ref().is_some_and(|kind| kind.path() == NAME)
            {
                continue;
            }

            let lifted: Entity = Entity { world, id: rider.id };
            lifted.setField(EntityField::FallDistance, 0.0);
            lifted.setField(EntityField::OnGround, 1.0);

            let _ = lifted.setMotion([rider.motion[0], rise.max(0.0), rider.motion[2]]);
        }

        if entity.field(EntityField::TicksExisted) > float::LIFETIME || info.pos[1] > float::WORLD_TOP
        {
            dropAndVanish(entity, info, carried);
            return;
        }

        let here: BlockPos = blockOf(info.pos);
        if !canContinue(world, here.offset(0, 1, 0))
        {
            if let Some(state) = &carried.m_state
            {
                let _ = handleOf(state).place(world, here, true);
            }

            entity.kill();
        }
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.entities().register(
        NAME,
        EntitySpec
        {
            template: Identifier::parse("minecraft:moving_block").ok(),
            tracking_range: Some(float::TRACKING),
            update_frequency: Some(float::UPDATES),
            send_velocity: Some(true),
            ..EntitySpec::default()
        },
        Some(std::rc::Rc::new(FloatingBlock)),
    )?;
    Ok(())
}
