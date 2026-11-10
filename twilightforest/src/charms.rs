use crate::ids::item;
use crate::items::ARMOR_FIRST_SLOT;
use b173_api::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

// TFEventListener's charm handling, the charm of life saves a dying player and the charms of keeping hold items
// through death
mod life
{
    pub const FIRST_HEALTH: f64 = 8.0;
    // EntityPlayer.applyArmorCalculations scales by what twenty five armor points leave
    pub const ARMOR_WHOLE: i32 = 25;
}

const MAIN_SLOTS: i32 = 36;
const HOTBAR: i32 = 9;
const ARMOR_SLOTS: i32 = 4;
// beta ItemArmor reduces by three eight six and three for helmet chestplate leggings and boots whatever the material
const ARMOR_POINTS: [i32; 4] = [3, 8, 6, 3];

// InventoryPlayer.consumeInventoryItem
fn consume(player: Player, name: &str) -> bool
{
    for slot in 0..MAIN_SLOTS
    {
        let Some(stack) = player.slot(slot).filter(|stack| stack.item.is(name))
        else
        {
            continue;
        };
        let left: Option<ItemStack> = (stack.count > 1).then(|| ItemStack { count: stack.count - 1, ..stack });
        return player.setSlot(slot, left).is_ok();
    }
    false
}

// boots are slot thirty six, the helmet thirty nine
fn armorValue(player: Player) -> i32
{
    let worn: i32 = (0..ARMOR_SLOTS)
        .filter(|piece| player.slot(ARMOR_FIRST_SLOT + piece).is_some())
        .map(|piece| ARMOR_POINTS[(ARMOR_SLOTS - 1 - piece) as usize])
        .sum();
    worn
}

// TFEventListener.willEntityDie
fn fatal(player: Player, health: f64, amount: i32) -> bool
{
    let left: f64 = f64::from(amount * (life::ARMOR_WHOLE - armorValue(player))) / f64::from(life::ARMOR_WHOLE);
    left.ceil() >= health.floor()
}

// EntityTFCharmEffect, a used charm circles its owner's head twice over, half a turn apart
const CHARM_EFFECT: &str = "charm_effect";

mod orbit
{
    pub const RADIUS: f64 = 1.75;
    pub const SPIN: f64 = 5.0;
    pub const LIFETIME: i32 = 200;
    pub const SIZE: f32 = 0.5;
}

#[derive(Serialize, Deserialize, Default, Clone)]
struct Orbit
{
    m_owner: i32,
    m_offset: f64,
    m_age: i32,
    m_texture: String,
}

fn release(world: World, owner: &EntityInfo, charm: &str)
{
    let (volume, pitch): (f32, f32) = (1.5, 1.0);
    world.playSound(owner.pos, "mob.zombie.unfect", volume, pitch);
    let texture: String = crate::items::texture(&charm.replace('_', ""));
    for offset in [0.0, std::f64::consts::PI]
    {
        let Ok(effect) = world.spawnEntity(&format!("twilightforest:{CHARM_EFFECT}"), owner.pos)
        else
        {
            warn("cannot spawn the charm effect");
            continue;
        };
        let orbit: Orbit = Orbit { m_owner: owner.id, m_offset: offset, m_age: 0, m_texture: texture.clone() };
        if let Err(error) = effect.setSynced(&orbit)
        {
            warn(&format!("charm effect: {}", error.0));
        }
    }
}

struct CharmEffect;

#[entity_behavior]
impl EntityBehavior for CharmEffect
{
    fn onTick(&self, entity: Entity, _info: &EntityInfo)
    {
        let world: World = entity.world;
        if !world.isAuthority()
        {
            return;
        }

        let mut state: Orbit = entity.synced();
        state.m_age += 1;
        let owner: Option<EntityInfo> = Entity { world, id: state.m_owner }.info().filter(|owner| owner.alive);
        let Some(owner) = owner.filter(|_| state.m_age <= orbit::LIFETIME)
        else
        {
            entity.kill();
            return;
        };

        let turn: f64 = f64::from(state.m_age) / orbit::SPIN + state.m_offset;
        let at: [f64; 3] =
            [owner.pos[0] + turn.cos() * orbit::RADIUS, owner.pos[1], owner.pos[2] - turn.sin() * orbit::RADIUS];
        if let Err(error) = entity.setPosition(at)
        {
            warn(&format!("charm effect: {}", error.0));
        }
        if let Err(error) = entity.setSynced(&state)
        {
            warn(&format!("charm effect: {}", error.0));
        }
    }

    // RenderTFCharm, the charm's icon facing the camera at half size
    fn pose(&self, input: &PoseInput) -> Pose
    {
        let state: Orbit = b173_api::fromBytes(&input.synced).unwrap_or_default();
        let mut program: crate::models::Program = crate::models::Program::new();
        program
            .texture(&state.m_texture)
            .faceCamera()
            .scale(orbit::SIZE, orbit::SIZE, orbit::SIZE)
            .normal(0.0, 1.0, 0.0)
            .quad([
                [-0.5, -0.25, 0.0, 0.0, 1.0],
                [0.5, -0.25, 0.0, 1.0, 1.0],
                [0.5, 0.75, 0.0, 1.0, 0.0],
                [-0.5, 0.75, 0.0, 0.0, 0.0],
            ]);
        Pose { pre: Vec::new(), model: program.take() }
    }
}

// TFEventListener.livingHurt, a charm of life in the inventory cancels the killing blow
fn hurt(event: &mut Event)
{
    let EventData::Attack { entity: target, amount, .. } = event.data().clone()
    else
    {
        return;
    };
    let world: World = World { dimension: target.world };
    if target.player.is_none() || !world.isAuthority()
    {
        return;
    }
    let player: Player = Player { id: target.id };
    let entity: Entity = Entity { world, id: target.id };
    let health: f64 = entity.field(EntityField::Health);
    if !fatal(player, health, amount)
    {
        return;
    }
    let second: bool = consume(player, item::CHARM_OF_LIFE_2);
    if !second && !consume(player, item::CHARM_OF_LIFE_1)
    {
        return;
    }
    event.cancel();
    if second
    {
        // the second charm's regeneration, resistance and fire resistance are potions beta lacks, the full heal stays
        entity.setField(EntityField::Health, entity.field(EntityField::MaxHealth));
    }
    else
    {
        // the first charm's short regeneration is a potion beta lacks, its heal stays
        entity.setField(EntityField::Health, life::FIRST_HEALTH);
    }
    release(
        world,
        &target,
        if second
        {
            item::CHARM_OF_LIFE_2
        }
        else
        {
            item::CHARM_OF_LIFE_1
        },
    );
}

// what a player kept through death, by name since the respawned player is a new entity
#[derive(Default)]
struct Kept
{
    m_slots: Vec<(i32, ItemStack)>,
    m_charm: Option<&'static str>,
}

thread_local!
{
    static KEPT: RefCell<HashMap<String, Kept>> = RefCell::new(HashMap::new());
}

fn take(player: Player, slots: impl Iterator<Item = i32>, kept: &mut Kept)
{
    for slot in slots
    {
        if let Some(stack) = player.slot(slot)
            && player.setSlot(slot, None).is_ok()
        {
            kept.m_slots.push((slot, stack));
        }
    }
}

fn heldSlot(player: Player) -> Option<i32>
{
    let held: ItemStack = player.held()?;
    (0..HOTBAR).find(|slot| player.slot(*slot).is_some_and(|stack| stack == held))
}

// TFEventListener.livingDies, the best charm of keeping decides what is set aside, tower keys always are
fn died(event: &mut Event)
{
    let EventData::Death { entity: target, .. } = event.data().clone()
    else
    {
        return;
    };
    let world: World = World { dimension: target.world };
    let Some(name) = target.player.clone().filter(|_| world.isAuthority())
    else
    {
        return;
    };
    let player: Player = Player { id: target.id };
    let mut kept: Kept = Kept::default();
    let armor = || ARMOR_FIRST_SLOT..ARMOR_FIRST_SLOT + ARMOR_SLOTS;
    if consume(player, item::CHARM_OF_KEEPING_3)
    {
        take(player, armor().chain(0..MAIN_SLOTS), &mut kept);
        kept.m_charm = Some(item::CHARM_OF_KEEPING_3);
    }
    else if consume(player, item::CHARM_OF_KEEPING_2)
    {
        take(player, armor().chain(0..HOTBAR), &mut kept);
        kept.m_charm = Some(item::CHARM_OF_KEEPING_2);
    }
    else if consume(player, item::CHARM_OF_KEEPING_1)
    {
        let held: Option<i32> = heldSlot(player);
        take(player, armor().chain(held), &mut kept);
        kept.m_charm = Some(item::CHARM_OF_KEEPING_1);
    }
    let keys: Vec<i32> =
        (0..MAIN_SLOTS).filter(|slot| player.slot(*slot).is_some_and(|stack| stack.item.is(item::TOWER_KEY))).collect();
    take(player, keys.into_iter(), &mut kept);
    if !kept.m_slots.is_empty() || kept.m_charm.is_some()
    {
        KEPT.with(|all| all.borrow_mut().insert(name, kept));
    }
}

// TFEventListener.onPlayerRespawn, the kept items come back where they were
fn respawned(event: &mut Event)
{
    let EventData::Player { player, name, .. } = event.data().clone()
    else
    {
        return;
    };
    let Some(kept) = KEPT.with(|all| all.borrow_mut().remove(&name))
    else
    {
        return;
    };
    let player: Player = Player { id: player };
    for (slot, stack) in kept.m_slots
    {
        if let Err(error) = player.setSlot(slot, Some(stack))
        {
            warn(&format!("kept item for {name}: {}", error.0));
        }
    }
    if let Some(charm) = kept.m_charm
        && let Some(info) = player.info()
    {
        release(World { dimension: info.world }, &info, charm);
    }
}

pub fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.entities().register(
        CHARM_EFFECT,
        EntitySpec
        {
            template: Identifier::parse("minecraft:projectile").ok(),
            size: Some([0.25, 0.25]),
            tracking_range: Some(64),
            update_frequency: Some(1),
            model: Some(EntityModelSpec { shadow: 0.0, ..EntityModelSpec::default() }),
            ..EntitySpec::default()
        },
        Some(std::rc::Rc::new(CharmEffect)),
    )?;
    ctx.events().on::<LivingHurt>(Priority::High, hurt)?;
    ctx.events().on::<LivingDeath>(Priority::High, died)?;
    ctx.events().on::<PlayerRespawn>(Priority::Normal, respawned)?;
    Ok(())
}
