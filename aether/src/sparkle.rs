use crate::entities::fraction;
use crate::particles;
use b173_api::prelude::*;

const HALF: f64 = 0.5;
const DRIFT: f64 = 0.5;

// BlockAetherLeaves.randomDisplayTick: the source's leaf motes drift around golden oak, crystal and holiday leaves
pub(crate) struct LeafSparkle
{
    pub m_particle: &'static str,
    pub m_count: i32,
    pub m_spread: f64,
    // one display tick in this many sparkles, one sparkles every tick
    pub m_odds: i32,
}

pub(crate) const GOLDEN_OAK: LeafSparkle =
    LeafSparkle { m_particle: particles::GOLDEN_OAK, m_count: 2, m_spread: 10.0, m_odds: 1 };
pub(crate) const CRYSTAL: LeafSparkle =
    LeafSparkle { m_particle: particles::CRYSTAL, m_count: 6, m_spread: 6.0, m_odds: 5 };
pub(crate) const HOLIDAY: LeafSparkle =
    LeafSparkle { m_particle: particles::HOLIDAY, m_count: 6, m_spread: 8.0, m_odds: 5 };

fn around(world: World, pos: BlockPos, spread: f64) -> [f64; 3]
{
    [pos.x, pos.y, pos.z].map(|axis| f64::from(axis) + (fraction(world) - HALF) * spread)
}

fn drift(world: World) -> [f64; 3] { [0.0; 3].map(|_| (fraction(world) - HALF) * DRIFT) }

#[block_behavior]
impl BlockBehavior for LeafSparkle
{
    fn onRandomDisplayTick(&self, ctx: &mut BlockCtx)
    {
        let world: World = ctx.world();
        if world.random(self.m_odds) != 0
        {
            return;
        }

        for _ in 0..self.m_count
        {
            let at: [f64; 3] = around(world, ctx.pos(), self.m_spread);
            world.particle(self.m_particle, at, drift(world));
        }
    }
}

mod portal
{
    // BlockAetherPortal.randomDisplayTick and ParticleAetherPortal
    pub const MOTES: i32 = 4;
    pub const SOUND_ODDS: i32 = 100;
    pub const VOLUME: f32 = 0.5;
    pub const PITCH: f32 = 0.8;
    pub const PITCH_SPREAD: f64 = 0.4;
    pub const EDGE: f64 = 0.25;
    pub const FLING: f64 = 2.0;
    pub const BRIGHTNESS: f64 = 0.6;
    pub const DARKEST: f64 = 0.4;
    pub const RED_GREEN: f64 = 0.2;
    pub const CHANNEL: f64 = 255.0;
}

// ParticleAetherPortal: a blue mote, red and green at a fifth of a random brightness
fn portalColor(world: World) -> String
{
    let brightness: f64 = fraction(world) * portal::BRIGHTNESS + portal::DARKEST;
    let dim: u32 = (brightness * portal::RED_GREEN * portal::CHANNEL) as u32;
    let blue: u32 = (brightness * portal::CHANNEL) as u32;
    format!("tintportal_{dim:02X}{dim:02X}{blue:02X}")
}

// BlockAetherPortal.randomDisplayTick: the portal hums now and then and throws blue motes from its faces
pub(crate) fn portalTick(ctx: &BlockCtx, portalBlock: &str)
{
    let world: World = ctx.world();
    let pos: BlockPos = ctx.pos();
    let center: [f64; 3] = [pos.x, pos.y, pos.z].map(|axis| f64::from(axis) + HALF);

    if world.random(portal::SOUND_ODDS) == 0
    {
        let pitch: f32 = (fraction(world) * portal::PITCH_SPREAD) as f32 + portal::PITCH;
        world.playSound(center, "portal.portal", portal::VOLUME, pitch);
    }

    let alongX: bool = world.is(pos.offset(-1, 0, 0), portalBlock) || world.is(pos.offset(1, 0, 0), portalBlock);

    for _ in 0..portal::MOTES
    {
        let mut at: [f64; 3] = [pos.x, pos.y, pos.z].map(|axis| f64::from(axis) + fraction(world));
        let mut motion: [f64; 3] = drift(world);
        let side: f64 = if world.random(2) == 0 { -1.0 } else { 1.0 };
        let axis: usize = if alongX { 2 } else { 0 };
        at[axis] = center[axis] + portal::EDGE * side;
        motion[axis] = fraction(world) * portal::FLING * side;
        world.particle(&portalColor(world), at, motion);
    }
}
