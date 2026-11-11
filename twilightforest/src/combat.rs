use crate::items::armorWorn;
use crate::names::item;
use b173_api::prelude::*;
use std::cell::Cell;

mod weapon
{
    // ItemTFFierySword and ItemTFFieryPick set what they hit alight for fifteen seconds
    pub const BURN_TICKS: i32 = 15 * 20;
    // ItemTFKnightlySword ItemTFKnightlyPick and ItemTFKnightlyAxe hit two harder against the right target
    pub const KNIGHTLY_BONUS: i32 = 2;
    pub const FLAMES: i32 = 20;
    pub const FLAKES: i32 = 20;
    // ItemTFIceSword's chill
    pub const CHILL: i32 = 2;
    pub const CHILL_TICKS: i32 = 200;
}

mod aura
{
    // TFEnchantment.getFieryAuraLevel counts five for every fiery piece worn, a mob burns on a roll under it in 25
    pub const FIERY_PER_PIECE: i32 = 5;
    pub const FIERY_ROLL: i32 = 25;
    // TFEnchantment.getChillAuraLevel counts the yeti pieces worn, the attacker slows that many levels for five ticks
    // a level and five more
    pub const CHILL_TICKS_PER_PIECE: i32 = 5;
    pub const CHILL_BASE_TICKS: i32 = 5;
}

// the vanilla damage types the source's DamageSources stand for, other mods read them through damageType
pub(crate) mod kind
{
    pub(crate) const MOB: &str = "minecraft:mob";
    pub(crate) const PLAYER: &str = "minecraft:player";
    pub(crate) const THROWN: &str = "minecraft:thrown";
    pub(crate) const FIREBALL: &str = "minecraft:fireball";
    pub(crate) const EXPLOSION: &str = "minecraft:explosion";
    pub(crate) const IN_FIRE: &str = "minecraft:in_fire";
    pub(crate) const ON_FIRE: &str = "minecraft:on_fire";
    pub(crate) const LAVA: &str = "minecraft:lava";
    pub(crate) const IN_WALL: &str = "minecraft:in_wall";
    pub(crate) const CACTUS: &str = "minecraft:cactus";
    pub(crate) const MAGIC: &str = "minecraft:magic";
}

thread_local!
{
    // the block a fiery pick just broke, its drops come out smelted
    static SMELT_AT: Cell<Option<(i32, BlockPos)>> = const { Cell::new(None) };
}

fn armored(target: &EntityInfo) -> bool
{
    if target.player.is_some()
    {
        let player: Player = Player { id: target.id };
        return (0..4).any(|slot| player.slot(crate::items::ARMOR_FIRST_SLOT + slot).is_some());
    }

    crate::tags::entityTagged(target, crate::tags::ARMORED)
}

fn holding(player: Player, name: &str) -> bool { player.held().is_some_and(|stack| stack.item.is(name)) }

fn burn(world: World, target: Entity, info: &EntityInfo)
{
    for _ in 0..weapon::FLAMES
    {
        world.particle("flame", [info.pos[0], info.pos[1] + 1.0, info.pos[2]], [0.0; 3]);
    }

    target.setField(EntityField::FireTicks, f64::from(weapon::BURN_TICKS));
}

// LivingHurt for every attack a player makes with twilight weapons
fn weaponHit(event: &mut Event, target: &EntityInfo, attacker: Player)
{
    let world: World = World { dimension: target.world };
    let entity: Entity = Entity { world, id: target.id };

    if holding(attacker, item::FIERY_SWORD) || holding(attacker, item::FIERY_PICK)
    {
        burn(world, entity, target);
    }

    // ItemTFIceSword.hitEntity, ten seconds of slowness three
    if holding(attacker, item::ICE_SWORD)
    {
        crate::effects::give(entity, crate::effects::Effect::Slowness, weapon::CHILL, weapon::CHILL_TICKS);

        for _ in 0..weapon::FLAKES
        {
            crate::particles::spawn(world, "snowstuff", target.pos, [0.0; 3]);
        }
    }

    let EventData::Attack { amount, .. } = event.data()
    else
    {
        return;
    };

    let bonus: bool = ((holding(attacker, item::KNIGHTLY_SWORD) || holding(attacker, item::KNIGHTLY_PICK))
        && armored(target))
        || (holding(attacker, item::KNIGHTLY_AXE) && !armored(target));

    if bonus
    {
        let raised: i32 = amount + weapon::KNIGHTLY_BONUS;
        event.setAmount(raised);
    }
}

// TFEventListener.entityHurts, a mob's own blow on the wearer sets it alight under fiery armor and chills it under
// yeti armor
fn armorAura(target: &EntityInfo, attacker: &EntityInfo)
{
    if !hurtBy(kind::MOB)
    {
        return;
    }

    let world: World = World { dimension: target.world };
    let wearer: Player = Player { id: target.id };
    let mob: Entity = Entity { world, id: attacker.id };
    let fire: i32 = armorWorn(wearer, "fiery") as i32 * aura::FIERY_PER_PIECE;
    if fire > 0 && world.random(aura::FIERY_ROLL) < fire
    {
        mob.setField(EntityField::FireTicks, f64::from(fire / 2 * 20));
    }

    let chill: i32 = armorWorn(wearer, "yeti") as i32;
    if chill > 0
    {
        let ticks: i32 = chill * aura::CHILL_TICKS_PER_PIECE + aura::CHILL_BASE_TICKS;
        crate::effects::give(mob, crate::effects::Effect::Slowness, chill, ticks);
    }
}

fn hurt(event: &mut Event)
{
    let EventData::Attack { entity: target, source: Some(source), .. } = event.data().clone()
    else
    {
        return;
    };
    let world: World = World { dimension: target.world };
    if !world.isAuthority()
    {
        return;
    }
    let Some(attacker) = world.entity(source)
    else
    {
        return;
    };

    if attacker.player.is_some()
    {
        weaponHit(event, &target, Player { id: attacker.id });
    }
    else if target.player.is_some()
    {
        armorAura(&target, &attacker);
    }
}

// ItemTFFieryPick marks the block it breaks, the harvest then smelts what drops
pub(crate) fn markSmelt(world: World, pos: BlockPos) { SMELT_AT.set(Some((world.dimension, pos))); }

fn smeltDrops(event: &mut Event)
{
    let Some((dimension, at)) = SMELT_AT.take()
    else
    {
        return;
    };
    let EventData::Drops { world, pos, drops, .. } = event.dataMut()
    else
    {
        return;
    };
    if *world != dimension || *pos != at
    {
        return;
    }

    let recipes: Vec<RecipeView> = recipeList();

    for drop in drops.iter_mut()
    {
        let smelted: Option<&StackData> = recipes.iter().find_map(|recipe| match &recipe.shape
        {
            RecipeShape::Smelting { input }
                if input.id == drop.id && (input.damage == drop.damage || input.damage < 0) =>
            {
                Some(&recipe.output)
            }

            _ => None,
        });
        if let Some(result) = smelted
        {
            *drop = StackData { id: result.id, count: drop.count, damage: result.damage };
        }
    }
}

pub(crate) fn hit(victim: Entity, amount: i32, source: Option<i32>, kind: &str) -> bool
{
    victim.damageAs(amount, source, kind).unwrap_or_else(|error|
    {
        warn(&format!("damage as {kind}: {}", error.0));
        false
    })
}

// a body part passes its hit on to the body under the type the hit came with
pub(crate) fn forward(victim: Entity, amount: i32, source: Option<i32>) -> bool
{
    match damageType()
    {
        Some(current) => hit(victim, amount, source, &current.to_string()),
        None => victim.damage(amount, source),
    }
}

// the type of the hit an onHurt call is handling, as the source compares its DamageSource
pub(crate) fn hurtBy(kind: &str) -> bool { damageType().is_some_and(|current| current.to_string() == kind) }

// DamageSource.isFireDamage
pub(crate) fn burning() -> bool { [kind::IN_FIRE, kind::ON_FIRE, kind::LAVA, kind::FIREBALL].into_iter().any(hurtBy) }

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.events().on::<LivingHurt>(Priority::Normal, hurt)?;
    ctx.events().on::<BlockHarvestDrops>(Priority::Normal, smeltDrops)?;
    Ok(())
}
