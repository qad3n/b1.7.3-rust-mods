use b173_api::prelude::*;

// the vanilla damage types closest to the source's DamageSources, other mods read them through damageType
pub mod kind
{
    pub const MOB: &str = "minecraft:mob";
    pub const PLAYER: &str = "minecraft:player";
    pub const ARROW: &str = "minecraft:arrow";
    pub const THROWN: &str = "minecraft:thrown";
    pub const FIREBALL: &str = "minecraft:fireball";
    pub const DROWN: &str = "minecraft:drown";
    pub const EXPLOSION: &str = "minecraft:explosion";
    pub const MAGIC: &str = "minecraft:magic";
    pub const IN_FIRE: &str = "minecraft:in_fire";
    pub const ON_FIRE: &str = "minecraft:on_fire";
    pub const LAVA: &str = "minecraft:lava";
}

pub fn hit(victim: Entity, amount: i32, source: Option<i32>, kind: &str) -> bool
{
    victim.damageAs(amount, source, kind).unwrap_or_else(|error| {
        warn(&format!("damage as {kind}: {error}"));
        false
    })
}

// whether the hit an attack event or onHurt call is handling has one of these types
fn hurtBy(kinds: &[&str]) -> bool { damageType().is_some_and(|current| kinds.contains(&current.to_string().as_str())) }

// DamageSource.isProjectile
pub fn projectile() -> bool { hurtBy(&[kind::ARROW, kind::THROWN, kind::FIREBALL]) }

// DamageSource.isFireDamage
pub fn burning() -> bool { hurtBy(&[kind::IN_FIRE, kind::ON_FIRE, kind::LAVA, kind::FIREBALL]) }

// DamageSource.isMagicDamage and isExplosion
pub fn magic() -> bool { hurtBy(&[kind::MAGIC]) }

pub fn explosion() -> bool { hurtBy(&[kind::EXPLOSION]) }

// a player's own blow with what it holds
pub fn melee() -> bool { hurtBy(&[kind::PLAYER]) }
