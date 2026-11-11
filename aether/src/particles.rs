use b173_api::prelude::*;

// the source's own EntityFX classes, each a beta particle recoloured, resized or retimed on the loader's mod particles
struct Kind
{
    m_name: &'static str,
    m_base: &'static str,
    m_color: Option<u32>,
    m_scale: Option<f32>,
    m_lifetime: Option<i32>,
}

pub(crate) const GOLDEN_OAK: &str = "aether:golden_oak_leaves";
pub(crate) const CRYSTAL: &str = "aether:crystal_leaves";
pub(crate) const HOLIDAY: &str = "aether:holiday_leaves";
pub(crate) const WHIRLY: &str = "aether:whirly";
pub(crate) const EVIL_WHIRLY: &str = "aether:evil_whirly";

// the colours the leaf motes and whirlies draw in
mod colour
{
    pub const GOLDEN_OAK: u32 = 0xF9BE00;
    pub const CRYSTAL: u32 = 0x00A4BE;
    pub const HOLIDAY: u32 = 0xFFFFFF;
    // ParticlePassiveWhirly's grey of 0.7 to 1 and ParticleEvilWhirly's of 0 to 0.3, at their middle
    pub const WHIRLY: u32 = 0xD9D9D9;
    pub const EVIL_WHIRLY: u32 = 0x262626;
}

const KINDS: [Kind; 5] = [
    // ParticleGoldenOakLeaves, ParticleCrystalLeaves and ParticleHolidayLeaves, tinted portal motes
    Kind { m_name: GOLDEN_OAK, m_base: "portal", m_color: Some(colour::GOLDEN_OAK), m_scale: None, m_lifetime: None },
    Kind { m_name: CRYSTAL, m_base: "portal", m_color: Some(colour::CRYSTAL), m_scale: None, m_lifetime: None },
    Kind { m_name: HOLIDAY, m_base: "portal", m_color: Some(colour::HOLIDAY), m_scale: None, m_lifetime: None },
    // ParticlePassiveWhirly, a pale puff about two and a half times smoke that lives some thirty ticks
    Kind { m_name: WHIRLY, m_base: "smoke", m_color: Some(colour::WHIRLY), m_scale: Some(2.5), m_lifetime: Some(29) },
    // ParticleEvilWhirly, a dark puff at three and a half times three quarters of smoke for some forty six ticks
    Kind
    {
        m_name: EVIL_WHIRLY,
        m_base: "smoke",
        m_color: Some(colour::EVIL_WHIRLY),
        m_scale: Some(2.6),
        m_lifetime: Some(46),
    },
];

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for row in &KINDS
    {
        let spec: ParticleSpec = ParticleSpec
        {
            base: row.m_base.to_owned(),
            color: row.m_color,
            scale: row.m_scale,
            lifetime: row.m_lifetime,
            gravity: None,
        };
        ctx.particles().register(row.m_name.trim_start_matches("aether:"), spec)?;
    }

    Ok(())
}
