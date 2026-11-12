use b173_api::prelude::*;

// the 1.7 potions the source hands out that beta's attributes can carry, on the loader's status effects; hunger,
// blindness, jump boost, resistance and fire resistance have no beta stat and stay out
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Effect
{
    Slowness,
    Speed,
    Strength,
    Poison,
    Regeneration,
}

// PotionHealth's pulse, every interval ticks the amount of health, and Potion's modifier per level
mod pulse
{
    pub const POISON_TICKS: i32 = 25;
    pub const POISON_HURT: i32 = -1;
    pub const REGENERATION_TICKS: i32 = 50;
    pub const REGENERATION_HEAL: i32 = 1;
    pub const NONE: i32 = 0;
    pub const SLOWER: f64 = -0.15;
    pub const FASTER: f64 = 0.2;
    pub const STRONGER: f64 = 1.3;
}

const SPEED_ATTRIBUTE: &str = "minecraft:movement_speed";
const DAMAGE_ATTRIBUTE: &str = "minecraft:attack_damage";

struct EffectRow
{
    m_effect: Effect,
    m_name: &'static str,
    m_color: u32,
    m_beneficial: bool,
    // Potion's attribute modifier, scaled once more per level
    m_modifier: Option<(&'static str, f64, ModifierOperation)>,
    // PotionHealth ticks, every interval heals a positive amount or hurts a negative one
    m_interval: i32,
    m_amount: i32,
}

// Potion.moveSlowdown, moveSpeed, damageBoost, poison and regeneration with their 1.7 colors and modifiers
const EFFECTS: [EffectRow; 5] = [
    EffectRow
    {
        m_effect: Effect::Slowness,
        m_name: "slowness",
        m_color: 0x5A6C81,
        m_beneficial: false,
        m_modifier: Some((SPEED_ATTRIBUTE, pulse::SLOWER, ModifierOperation::MultiplyTotal)),
        m_interval: pulse::NONE,
        m_amount: pulse::NONE,
    },
    EffectRow
    {
        m_effect: Effect::Speed,
        m_name: "speed",
        m_color: 0x7CAFC6,
        m_beneficial: true,
        m_modifier: Some((SPEED_ATTRIBUTE, pulse::FASTER, ModifierOperation::MultiplyTotal)),
        m_interval: pulse::NONE,
        m_amount: pulse::NONE,
    },
    EffectRow
    {
        m_effect: Effect::Strength,
        m_name: "strength",
        m_color: 0x932423,
        m_beneficial: true,
        m_modifier: Some((DAMAGE_ATTRIBUTE, pulse::STRONGER, ModifierOperation::MultiplyTotal)),
        m_interval: pulse::NONE,
        m_amount: pulse::NONE,
    },
    EffectRow
    {
        m_effect: Effect::Poison,
        m_name: "poison",
        m_color: 0x4E9331,
        m_beneficial: false,
        m_modifier: None,
        m_interval: pulse::POISON_TICKS,
        m_amount: pulse::POISON_HURT,
    },
    EffectRow
    {
        m_effect: Effect::Regeneration,
        m_name: "regeneration",
        m_color: 0xCD5CAB,
        m_beneficial: true,
        m_modifier: None,
        m_interval: pulse::REGENERATION_TICKS,
        m_amount: pulse::REGENERATION_HEAL,
    },
];

fn row(effect: Effect) -> &'static EffectRow
{
    EFFECTS.iter().find(|row| row.m_effect == effect).expect("every effect has a row")
}

// EntityLivingBase.addPotionEffect, level 0 is the first level
pub(crate) fn give(target: Entity, effect: Effect, level: i32, ticks: i32)
{
    let name: String = format!("twilightforest:{}", row(effect).m_name);
    if let Err(error) = target.addEffect(&name, level, ticks)
    {
        warn(&format!("effect {name}: {}", error.0));
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for row in &EFFECTS
    {
        let mut spec: b173_api::Effect = b173_api::Effect::new(row.m_color).every(row.m_interval, row.m_amount);
        if row.m_beneficial
        {
            spec = spec.beneficial();
        }

        if let Some((attribute, amount, operation)) = row.m_modifier
        {
            spec = spec.modifier(attribute, amount, operation);
        }

        ctx.effects().register(row.m_name, spec)?;
    }

    Ok(())
}
