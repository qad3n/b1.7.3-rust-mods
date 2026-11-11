use crate::effects::Effect;
use crate::entities::lookOf;
use crate::ids::blockId;
use b173_api::prelude::*;

mod reach
{
    // ItemBucket traces five blocks ahead of the eyes
    pub const BUCKET: f64 = 5.0;
    pub const FACES: [[i32; 3]; 6] = [[0, -1, 0], [0, 1, 0], [0, 0, -1], [0, 0, 1], [-1, 0, 0], [1, 0, 0]];
}

fn trace(world: World, info: &EntityInfo, liquids: bool) -> Option<RayHit>
{
    let look: [f64; 3] = lookOf(info);
    let to: [f64; 3] = [
        info.pos[0] + look[0] * reach::BUCKET,
        info.pos[1] + look[1] * reach::BUCKET,
        info.pos[2] + look[2] * reach::BUCKET,
    ];
    world.rayTrace(info.pos, to, liquids)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Bucket
{
    Empty,
    Water,
    Poison,
    Remedy,
    Milk,
}

// ItemSkyrootBucket: fills from water, pours it out, and the poison, remedy and milk buckets are drunk
struct SkyrootBucket
{
    m_kind: Bucket,
}

// ItemSkyrootBucket.getMaxItemUseDuration for the buckets that are drunk
const DRINK_TICKS: i32 = 32;

fn empty() -> Option<ItemStack> { Some(ItemStack::of("skyroot_bucket", 1)) }

impl SkyrootBucket
{
    fn drunk(&self) -> bool { matches!(self.m_kind, Bucket::Poison | Bucket::Remedy | Bucket::Milk) }
}

#[item_behavior]
impl ItemBehavior for SkyrootBucket
{
    fn onRightClick(&self, world: World, player: Player, stack: ItemStack) -> Use
    {
        let Some(info) = player.info()
        else
        {
            return Use::Pass;
        };

        // ItemSkyrootBucket stacks empties to sixteen, only a single one fills
        match self.m_kind
        {
            Bucket::Empty =>
            {
                let water: [i32; 2] = [blockId("minecraft:water"), blockId("minecraft:flowing_water")];
                let Some(hit) = trace(world, &info, true)
                    .filter(|hit| water.contains(&world.block(hit.pos)) && world.meta(hit.pos) == 0)
                else
                {
                    return Use::Pass;
                };
                if stack.count > 1
                {
                    return Use::Pass;
                }

                if world.isAuthority()
                {
                    let _ = world.setBlock(hit.pos, 0, 0);
                }

                Use::Done(Some(ItemStack::of("skyroot_water_bucket", 1)))
            }

            Bucket::Water =>
            {
                let Some(hit) = trace(world, &info, false)
                else
                {
                    return Use::Pass;
                };
                let [dx, dy, dz] = reach::FACES[hit.face.clamp(0, 5) as usize];
                let at: BlockPos = hit.pos.offset(dx, dy, dz);
                if !world.isAir(at)
                {
                    return Use::Pass;
                }

                if world.isAuthority()
                {
                    let _ = world.setBlock(at, blockId("minecraft:flowing_water"), 0);
                }

                Use::Done(empty())
            }

            Bucket::Poison | Bucket::Remedy | Bucket::Milk =>
            {
                if let Err(error) = player.setItemInUse(DRINK_TICKS)
                {
                    warn(&format!("skyroot bucket drink: {error}"));
                }

                Use::Done(Some(stack))
            }
        }
    }

    fn useDuration(&self, _stack: ItemStack) -> i32 { if self.drunk() { DRINK_TICKS } else { 0 } }

    fn useAction(&self, _stack: ItemStack) -> UseAction
    {
        if self.drunk()
        {
            UseAction::Drink
        }
        else
        {
            UseAction::None
        }
    }

    // ItemSkyrootBucket.onBucketUsed: poison inebriates, the remedy cures it and milk clears the vanilla potions
    fn onUseFinish(&self, world: World, player: Player, _stack: ItemStack) -> Use
    {
        let entity: Entity = Entity { world, id: player.id };

        match self.m_kind
        {
            Bucket::Poison => crate::player::poison(entity),
            Bucket::Remedy => crate::player::cure(entity),

            Bucket::Milk if world.isAuthority() =>
            {
                crate::effects::remove(entity, Effect::Regeneration);
                crate::effects::remove(entity, Effect::Weakness);
            }

            Bucket::Milk =>
            {}
            Bucket::Empty | Bucket::Water => return Use::Pass,
        }

        Use::Done(empty())
    }
}

const BUCKETS: [(&str, Bucket); 5] = [
    ("skyroot_bucket", Bucket::Empty),
    ("skyroot_water_bucket", Bucket::Water),
    ("skyroot_poison_bucket", Bucket::Poison),
    ("skyroot_remedy_bucket", Bucket::Remedy),
    ("skyroot_milk_bucket", Bucket::Milk),
];

pub(crate) fn bucket(name: &str) -> Option<impl ItemBehavior>
{
    BUCKETS.iter().find(|(item, _)| *item == name).map(|(_, kind)| SkyrootBucket { m_kind: *kind })
}

mod chute
{
    // EntityParachute keeps its rider falling no faster than this
    pub const FALL: f64 = -0.25;
    pub const ABOVE: f64 = 1.5;
    pub const GOLDEN_USES: i32 = 20;
    pub const PUFFS: i32 = 1;
}

const PARACHUTE: &str = "parachute";

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct Chute
{
    m_rider: i32,
    m_golden: bool,
}

// ItemAetherParachute: opens a cloud over a falling player, the cold one is used up, the golden one wears
struct ParachuteItem
{
    m_golden: bool,
}

#[item_behavior]
impl ItemBehavior for ParachuteItem
{
    fn onRightClick(&self, world: World, player: Player, stack: ItemStack) -> Use
    {
        let Some(info) = player.info()
        else
        {
            return Use::Pass;
        };
        let entity: Entity = Entity { world, id: player.id };
        if entity.flag(EntityField::OnGround) || entity.flag(EntityField::InWater)
        {
            return Use::Pass;
        }

        if world.isAuthority()
            && let Ok(chute) = world
                .spawnEntity(&format!("aether:{PARACHUTE}"), [info.pos[0], info.pos[1] + chute::ABOVE, info.pos[2]])
        {
            let cloud: &str = if self.m_golden
            {
                "golden_aercloud"
            }
            else
            {
                "cold_aercloud"
            };
            chute.setField(EntityField::Block, blockId(cloud) as f64);

            let _ = chute.setSynced(&Chute { m_rider: player.id, m_golden: self.m_golden });
        }

        if self.m_golden
        {
            let worn: ItemStack = ItemStack { damage: stack.damage + 1, ..stack };
            return Use::Done((worn.damage < chute::GOLDEN_USES).then_some(worn));
        }

        Use::Done((stack.count > 1).then(|| ItemStack { count: stack.count - 1, ..stack }))
    }
}

// EntityParachute: a cloud hanging over its rider until the rider lands or swims
struct Parachute;

#[entity_behavior]
impl EntityBehavior for Parachute
{
    fn onTick(&self, entity: Entity, info: &EntityInfo)
    {
        let world: World = entity.world;
        let chute: Chute = entity.synced();
        let name: &str = if chute.m_golden
        {
            "golden_aercloud"
        }
        else
        {
            "cold_aercloud"
        };
        let cloud: i32 = blockId(name);
        entity.setField(EntityField::Block, cloud as f64);

        let rider: Entity = Entity { world, id: chute.m_rider };
        let Some(seat) = rider.info().filter(|seat| seat.alive)
        else
        {
            if world.isAuthority()
            {
                entity.kill();
            }

            return;
        };

        if rider.flag(EntityField::OnGround) || rider.flag(EntityField::InWater)
        {
            if world.isAuthority()
            {
                for _ in 0..chute::PUFFS
                {
                    world.particle("explode", info.pos, [0.0; 3]);
                }

                entity.kill();
            }

            return;
        }

        let _ = entity.setPosition([seat.pos[0], seat.pos[1] + chute::ABOVE, seat.pos[2]]);
        let _ = entity.setMotion(seat.motion);
        let local: bool = localPlayer().is_some_and(|player| player.id == chute.m_rider);
        if local || world.isAuthority()
        {
            if seat.motion[1] < chute::FALL
            {
                let _ = rider.setMotion([seat.motion[0], chute::FALL, seat.motion[2]]);
            }

            rider.setField(EntityField::FallDistance, 0.0);
        }
    }
}

pub(crate) fn parachute(name: &str) -> Option<impl ItemBehavior>
{
    match name
    {
        "cold_parachute" => Some(ParachuteItem { m_golden: false }),
        "golden_parachute" => Some(ParachuteItem { m_golden: true }),
        _ => None,
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.entities().register(
        PARACHUTE,
        EntitySpec
        {
            template: Identifier::parse("minecraft:moving_block").ok(),
            tracking_range: Some(160),
            update_frequency: Some(1),
            send_velocity: Some(true),
            ..EntitySpec::default()
        },
        Some(std::rc::Rc::new(Parachute)),
    )?;
    Ok(())
}
