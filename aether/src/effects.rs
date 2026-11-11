use b173_api::prelude::*;

// the potions the source hands out, on the loader's status effects
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Effect
{
    Inebriation,
    Regeneration,
    Weakness,
}

// PotionHealth's pulse, every interval ticks the amount of health
mod pulse
{
    pub const INEBRIATION_TICKS: i32 = 50;
    pub const INEBRIATION_HURT: i32 = -1;
    pub const REGENERATION_TICKS: i32 = 50;
    pub const REGENERATION_HEAL: i32 = 1;
    pub const NONE: i32 = 0;
    // PotionAttackDamage takes half a point from weakness
    pub const WEAKER: f64 = -0.5;
}

// the hud colours the source's potions draw with
mod colour
{
    pub const INEBRIATION: u32 = 0x51297B;
    pub const REGENERATION: u32 = 0xCD5CAB;
    pub const WEAKNESS: u32 = 0x484D48;
}

struct EffectRow
{
    m_effect: Effect,
    m_id: &'static str,
    m_color: u32,
    m_beneficial: bool,
    m_icon: Option<&'static str>,
    m_modifier: Option<(&'static str, f64, ModifierOperation)>,
    m_interval: i32,
    m_amount: i32,
}

// PotionInebriation and Potion.regeneration and weakness with their 1.7 colours
const EFFECTS: [EffectRow; 3] = [
    EffectRow
    {
        m_effect: Effect::Inebriation,
        m_id: "aether:inebriation",
        m_color: colour::INEBRIATION,
        m_beneficial: false,
        m_icon: Some("aether:gui/inventory/inebriation"),
        m_modifier: None,
        m_interval: pulse::INEBRIATION_TICKS,
        m_amount: pulse::INEBRIATION_HURT,
    },
    EffectRow
    {
        m_effect: Effect::Regeneration,
        m_id: "aether:regeneration",
        m_color: colour::REGENERATION,
        m_beneficial: true,
        m_icon: None,
        m_modifier: None,
        m_interval: pulse::REGENERATION_TICKS,
        m_amount: pulse::REGENERATION_HEAL,
    },
    EffectRow
    {
        m_effect: Effect::Weakness,
        m_id: "aether:weakness",
        m_color: colour::WEAKNESS,
        m_beneficial: false,
        m_icon: None,
        m_modifier: Some(("minecraft:attack_damage", pulse::WEAKER, ModifierOperation::Add)),
        m_interval: pulse::NONE,
        m_amount: pulse::NONE,
    },
];

fn row(effect: Effect) -> &'static EffectRow
{
    EFFECTS.iter().find(|row| row.m_effect == effect).expect("every effect has a row")
}

pub(crate) fn give(target: Entity, effect: Effect, ticks: i32)
{
    let id: &str = row(effect).m_id;
    if let Err(error) = target.addEffect(id, 0, ticks)
    {
        warn(&format!("effect {id}: {error}"));
    }
}

pub(crate) fn remove(target: Entity, effect: Effect)
{
    let id: &str = row(effect).m_id;
    if let Err(error) = target.removeEffect(id)
    {
        warn(&format!("effect {id} removal: {error}"));
    }
}

pub(crate) fn has(target: Entity, effect: Effect) -> bool { target.effectLevel(row(effect).m_id).is_some() }

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for row in &EFFECTS
    {
        let modifiers: Vec<EffectModifierSpec> = match row.m_modifier
        {
            Some((attribute, amount, operation)) => vec![EffectModifierSpec
            {
                attribute: Identifier::parse(attribute).map_err(|error| Error(error.to_string()))?,
                amount,
                operation,
            }],
            None => Vec::new(),
        };

        let spec: EffectSpec = EffectSpec
        {
            color: row.m_color,
            beneficial: row.m_beneficial,
            icon: row.m_icon.map(str::to_owned),
            modifiers,
            interval: row.m_interval,
            amount: row.m_amount,
        };
        ctx.effects().register(row.m_id.trim_start_matches("aether:"), spec)?;
    }

    Ok(())
}
