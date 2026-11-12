use crate::effects::Effect;
use crate::entities::lookOf;
use crate::names::numeric;
use b173_api::math::{Face, offset};
use b173_api::prelude::*;

mod reach
{
    // ItemBucket traces five blocks ahead of the eyes
    pub const BUCKET: f64 = 5.0;
    // a milked cow stands within a player's arm of whoever holds the bucket
    pub const MILKING: f64 = 6.0;
}

const SKYROOT_BUCKET: &str = "skyroot_bucket";
const MILK_BUCKET: &str = "skyroot_milk_bucket";
const COW: &str = "minecraft:cow";
const MAIN_SLOTS: i32 = 36;

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
// ItemSkyrootBucket.onBucketUsed, the remedy's setCured
const REMEDY_CURE: i32 = 200;

fn empty() -> Option<ItemStack> { Some(ItemStack::of(SKYROOT_BUCKET, 1)) }

// AetherEventHandler.onEntityInteract: an empty skyroot bucket milks cows and flying cows, one bucket of a stack at
// a time
pub(crate) fn milk(player: Player) -> bool
{
    let Some(held) = player.held().filter(|held| held.item.is(SKYROOT_BUCKET))
    else
    {
        return false;
    };

    let milked: ItemStack = ItemStack::of(MILK_BUCKET, 1);
    if held.count <= 1
    {
        let _ = player.setHeld(Some(milked));

        return true;
    }

    let _ = player.setHeld(Some(ItemStack { count: held.count - 1, ..held }));
    if let Err(error) = player.give(milked)
    {
        warn(&format!("skyroot milk bucket: {error}"));
    }

    true
}

// the item hook names no player, the milker is the nearest one holding an empty skyroot bucket
fn milker(world: World, cow: &EntityInfo) -> Option<Player>
{
    let reach = |player: &Player| -> Option<f64>
    {
        let info: EntityInfo = player.info()?;
        let held: bool = player.held().is_some_and(|held| held.item.is(SKYROOT_BUCKET));
        Some(b173_api::math::distance(info.pos, cow.pos)).filter(|distance| held && *distance <= reach::MILKING)
    };
    world
        .players()
        .into_iter()
        .filter_map(|player| reach(&player).map(|distance| (player, distance)))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(player, _)| player)
}

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
                let source = |hit: &RayHit| world.fluidAt(hit.pos).is_some_and(|fluid| fluid.source);
                let Some(hit) =
                    trace(world, &info, true).filter(|hit| world.tagged(hit.pos, "minecraft:water") && source(hit))
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
                    let _ = world.clear(hit.pos);
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
                let Some(face) = Face::fromIndex(hit.face)
                else
                {
                    return Use::Pass;
                };

                let at: BlockPos = offset(hit.pos, face);

                // AetherEventHandler.onFillBucket: skyroot water lights a glowstone frame as a water bucket does
                if crate::portal::trySpawn(world, at)
                {
                    return Use::Done(empty());
                }

                if !world.isAir(at)
                {
                    return Use::Pass;
                }

                if world.isAuthority()
                {
                    let _ = world.place(at, "minecraft:flowing_water");
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

    fn onInteractEntity(&self, _stack: ItemStack, target: Entity)
    {
        let world: World = target.world;
        if self.m_kind != Bucket::Empty || !world.isAuthority()
        {
            return;
        }

        let cow: Option<EntityInfo> =
            target.info().filter(|info| info.kind.as_ref().is_some_and(|kind| kind.to_string() == COW));
        if let Some(player) = cow.and_then(|cow| milker(world, &cow))
        {
            milk(player);
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
            Bucket::Remedy => crate::player::cure(entity, REMEDY_CURE),

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
    // PlayerAether.onUpdate opens a carried parachute below this fall speed
    pub const PLUMMET: f64 = -2.0;
    pub const TERMINAL: f64 = -4.0;
    pub const PUFFS: i32 = 1;
}

const PARACHUTE: &str = "parachute";

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct Chute
{
    m_rider: i32,
    m_golden: bool,
}

// EntityParachute over a player, drawn as the cloud its item is made of
fn open(world: World, player: Player, info: &EntityInfo, golden: bool)
{
    let Ok(chute) =
        world.spawnEntity(&format!("aether:{PARACHUTE}"), [info.pos[0], info.pos[1] + chute::ABOVE, info.pos[2]])
    else
    {
        return;
    };

    let cloud: &str = if golden
    {
        "golden_aercloud"
    }
    else
    {
        "cold_aercloud"
    };
    chute.setField(EntityField::Block, f64::from(numeric(cloud)));

    let _ = chute.setSynced(&Chute { m_rider: player.id, m_golden: golden });
}

// what is left of a parachute after one opening, the cold one is used up and the golden one wears
fn spent(stack: ItemStack, golden: bool) -> Option<ItemStack>
{
    if golden
    {
        let worn: ItemStack = ItemStack { damage: stack.damage + 1, ..stack };
        return (worn.damage < chute::GOLDEN_USES).then_some(worn);
    }

    (stack.count > 1).then(|| ItemStack { count: stack.count - 1, ..stack })
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
        {
            open(world, player, &info, self.m_golden);
        }

        Use::Done(spent(stack, self.m_golden))
    }
}

// the height each player stood at a tick ago, a server never learns a player's own motion
static LAST_Y: EntityState<Option<f64>> = EntityState::new("last_height");

// PlayerAether.activateParachute: a player falling faster than two blocks a tick opens the first parachute carried
pub(crate) fn openFalling(player: Player, info: &EntityInfo)
{
    let entity: Entity = Entity { world: World { dimension: info.world }, id: player.id };
    let before: Option<f64> = LAST_Y.get(entity);
    crate::entities::store(&LAST_Y, entity, &Some(info.pos[1]));

    // a drop past terminal speed is a teleport, never a fall
    let plummeting = |before: f64| (chute::TERMINAL..chute::PLUMMET).contains(&(info.pos[1] - before));
    if !before.is_some_and(plummeting)
    {
        return;
    }

    let carried = |slot: i32| -> Option<(i32, ItemStack, bool)>
    {
        let stack: ItemStack = player.slot(slot)?;
        let golden: bool = stack.item.is("golden_parachute");
        (golden || stack.item.is("cold_parachute")).then_some((slot, stack, golden))
    };
    let Some((slot, stack, golden)) = (0..MAIN_SLOTS).find_map(carried)
    else
    {
        return;
    };

    open(entity.world, player, info, golden);

    let _ = player.setSlot(slot, spent(stack, golden));
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
        entity.setField(EntityField::Block, f64::from(numeric(name)));

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
