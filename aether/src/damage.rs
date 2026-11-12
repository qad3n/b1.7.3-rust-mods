use b173_api::prelude::*;

// the vanilla damage types closest to the source's DamageSources, other mods read them through damageType
pub(crate) mod kind
{
    pub(crate) const MOB: &str = "minecraft:mob";
    pub(crate) const PLAYER: &str = "minecraft:player";
    pub(crate) const ARROW: &str = "minecraft:arrow";
    pub(crate) const THROWN: &str = "minecraft:thrown";
    pub(crate) const FIREBALL: &str = "minecraft:fireball";
    pub(crate) const DROWN: &str = "minecraft:drown";
    pub(crate) const EXPLOSION: &str = "minecraft:explosion";
    pub(crate) const MAGIC: &str = "minecraft:magic";
    pub(crate) const IN_FIRE: &str = "minecraft:in_fire";
    pub(crate) const ON_FIRE: &str = "minecraft:on_fire";
    pub(crate) const LAVA: &str = "minecraft:lava";
    pub(crate) const FALL: &str = "minecraft:fall";
}

pub(crate) fn hit(victim: Entity, amount: i32, source: Option<i32>, kind: &str) -> bool
{
    victim.damageAs(amount, source, kind).unwrap_or_else(|error|
    {
        warn(&format!("damage as {kind}: {error}"));
        false
    })
}

// whether the hit an attack event or onHurt call is handling has one of these types
fn hurtBy(kinds: &[&str]) -> bool { damageType().is_some_and(|current| kinds.contains(&current.to_string().as_str())) }

// DamageSource.isProjectile
pub(crate) fn projectile() -> bool { hurtBy(&[kind::ARROW, kind::THROWN, kind::FIREBALL]) }

// DamageSource.isFireDamage
pub(crate) fn burning() -> bool { hurtBy(&[kind::IN_FIRE, kind::ON_FIRE, kind::LAVA, kind::FIREBALL]) }

// DamageSource.isMagicDamage and isExplosion
pub(crate) fn magic() -> bool { hurtBy(&[kind::MAGIC]) }

pub(crate) fn explosion() -> bool { hurtBy(&[kind::EXPLOSION]) }

pub(crate) fn fall() -> bool { hurtBy(&[kind::FALL]) }

// a player's own blow with what it holds
pub(crate) fn melee() -> bool { hurtBy(&[kind::PLAYER]) }
