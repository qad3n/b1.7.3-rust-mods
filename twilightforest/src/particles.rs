use crate::ids::{self, NEIGHBORS};
use b173_api::prelude::*;

// TFClientProxy.spawnParticle's own particle classes and the 1.7 particles beta lacks, each a beta particle
// recolored, resized, retimed or weighted toward the source's EntityFX on the loader's mod particles
struct Kind
{
    m_name: &'static str,
    m_base: &'static str,
    m_color: Option<u32>,
    m_scale: Option<f32>,
    m_lifetime: Option<i32>,
    m_gravity: Option<f32>,
}

const fn kind(name: &'static str, base: &'static str) -> Kind
{
    Kind { m_name: name, m_base: base, m_color: None, m_scale: None, m_lifetime: None, m_gravity: None }
}

const WHITE: u32 = 0xFFFFFF;

const KINDS: [Kind; 19] = [
    // EntitySmokeFX at eight times the size
    Kind { m_scale: Some(8.0), ..kind("hugesmoke", "smoke") },
    // EntityTFLargeFlameFX, a white flame five times the size
    Kind { m_scale: Some(5.0), m_lifetime: Some(16), ..kind("largeflame", "flame") },
    // EntityTFGhastTrapFX, a pale pink dust
    Kind { m_color: Some(0xFFE6E6), m_scale: Some(0.75), m_lifetime: Some(20), ..kind("ghasttrap", "reddust") },
    // EntityTFLeafRuneFX, a grey glyph that sinks slowly
    Kind { m_color: Some(0xB3B3CC), m_scale: Some(1.5), m_gravity: Some(0.006), ..kind("leafrune", "portal") },
    // EntityTFProtectionFX, a pale aura speck
    Kind { m_color: Some(WHITE), m_scale: Some(0.5), ..kind("protection", "portal") },
    // EntityTFSnowFX and its warning, guardian and ice beam kin, white flakes of different lengths
    Kind { m_color: Some(WHITE), m_scale: Some(0.75), m_lifetime: Some(6), ..kind("snowstuff", "snowshovel") },
    Kind { m_color: Some(WHITE), m_lifetime: Some(50), ..kind("snowwarning", "snowshovel") },
    Kind { m_color: Some(0xDDDDDD), m_scale: Some(0.75), m_lifetime: Some(17), ..kind("snowguardian", "snowshovel") },
    Kind { m_color: Some(WHITE), m_scale: Some(0.75), m_lifetime: Some(50), ..kind("icebeam", "snowshovel") },
    // EntityTFAnnihilateFX, a white mote that lingers
    Kind { m_color: Some(WHITE), m_scale: Some(0.75), m_lifetime: Some(60), ..kind("annihilate", "snowshovel") },
    // EntityTFFireflyFX, a yellow green glow
    Kind { m_color: Some(0xE6FF00), m_lifetime: Some(53), ..kind("firefly", "reddust") },
    // EntityTFBossTearFX, a heavy falling drop
    Kind { m_lifetime: Some(40), m_gravity: Some(2.0), ..kind("bosstear", "snowballpoof") },
    // the 1.7 vanilla particles beta has no texture for
    Kind { m_color: Some(WHITE), ..kind("crit", "snowshovel") },
    Kind { m_color: Some(0x9966FF), ..kind("magicCrit", "snowshovel") },
    Kind { m_scale: Some(4.0), ..kind("largeexplode", "explode") },
    Kind { m_scale: Some(6.0), ..kind("hugeexplosion", "largesmoke") },
    Kind { m_gravity: Some(0.06), ..kind("dripWater", "splash") },
    Kind { m_color: Some(0x8B8B8B), ..kind("iconcrack_20_0", "snowballpoof") },
    Kind { m_color: Some(0xCCCCCC), ..kind("townaura", "smoke") },
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
            gravity: row.m_gravity,
        };
        ctx.particles().register(row.m_name, spec)?;
    }

    Ok(())
}

// the source's particle names spawn as this mod's particles, beta's own pass through
pub(crate) fn spawn(world: World, name: &str, at: [f64; 3], motion: [f64; 3])
{
    match KINDS.iter().find(|row| row.m_name == name)
    {
        Some(row) => world.particle(&format!("twilightforest:{}", row.m_name), at, motion),
        None => world.particle(name, at, motion),
    }
}

// BlockTFTowerDevice.sparkle, a speck just outside every face that is not covered
pub(crate) fn sparkle(world: World, pos: BlockPos, name: &str)
{
    glint(world, pos, name, [0.0; 3], |at| !World::blockInfo(world.block(at)).opaque);
}

// a speck just outside every face whose neighbor passes the check
pub(crate) fn glint(world: World, pos: BlockPos, name: &str, motion: [f64; 3], open: impl Fn(BlockPos) -> bool)
{
    for [dx, dy, dz] in NEIGHBORS
    {
        if !open(pos.offset(dx, dy, dz))
        {
            continue;
        }

        spawn(world, name, outside(world, pos, [dx, dy, dz]), motion);
    }
}

// a random spot on the face of a block that looks along the step
pub(crate) fn outside(world: World, pos: BlockPos, step: [i32; 3]) -> [f64; 3]
{
    let offset: f64 = 0.0625;
    let mut at: [f64; 3] = [
        f64::from(pos.x) + f64::from(ids::fraction(world)),
        f64::from(pos.y) + f64::from(ids::fraction(world)),
        f64::from(pos.z) + f64::from(ids::fraction(world)),
    ];

    for (axis, step) in step.into_iter().enumerate()
    {
        let base: f64 = f64::from([pos.x, pos.y, pos.z][axis]);

        match step
        {
            1 => at[axis] = base + 1.0 + offset,
            -1 => at[axis] = base - offset,
            _ =>
            {}
        }
    }

    at
}

pub(crate) fn line(world: World, name: &str, from: [f64; 3], to: [f64; 3], count: i32)
{
    let jitter: f64 = 0.005;

    for step in 0..count
    {
        let along: f64 = f64::from(step) / (f64::from(count) - 1.0);

        let at: [f64; 3] = std::array::from_fn(|axis|
        {
            from[axis] + (to[axis] - from[axis]) * along + f64::from(ids::fraction(world)) * jitter
        });
        spawn(world, name, at, [0.0; 3]);
    }
}

// EntityTFTinyFirefly, a glowing speck drifting near a firefly jar
pub(crate) fn tinyFirefly(world: World, at: [f64; 3])
{
    let spread: f64 = 0.3;
    let rise: f64 = 0.4;
    let wobble = || f64::from(ids::fraction(world) - ids::fraction(world));
    spawn(world, "firefly", [at[0] + wobble() * spread, at[1] + wobble() * rise, at[2] + wobble() * spread], [0.0; 3]);
}
