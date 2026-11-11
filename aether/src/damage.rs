use b173_api::prelude::*;

// the vanilla damage types closest to the source's DamageSources, other mods read them through damageType
pub mod kind
{
    pub const MOB: &str = "minecraft:mob";
    pub const ARROW: &str = "minecraft:arrow";
    pub const THROWN: &str = "minecraft:thrown";
    pub const FIREBALL: &str = "minecraft:fireball";
    pub const DROWN: &str = "minecraft:drown";
}

pub fn hit(victim: Entity, amount: i32, source: Option<i32>, kind: &str) -> bool
{
    victim.damageAs(amount, source, kind).unwrap_or_else(|error| {
        warn(&format!("damage as {kind}: {error}"));
        false
    })
}

// DamageSource.isProjectile for the hit an attack event or onHurt call is handling
pub fn projectile() -> bool
{
    damageType()
        .is_some_and(|current| [kind::ARROW, kind::THROWN, kind::FIREBALL].contains(&current.to_string().as_str()))
}
