use crate::ids::{self, NEIGHBORS};
use b173_api::prelude::*;

// TFClientProxy.spawnParticle draws its own particle classes and 1.7 has more vanilla ones, these are the closest
// beta particles
struct Stand
{
    m_name: &'static str,
    m_beta: &'static str,
}

const STANDS: [Stand; 14] = [
    Stand { m_name: "hugesmoke", m_beta: "largesmoke" },
    Stand { m_name: "largeflame", m_beta: "flame" },
    Stand { m_name: "ghasttrap", m_beta: "portal" },
    Stand { m_name: "leafrune", m_beta: "portal" },
    Stand { m_name: "protection", m_beta: "portal" },
    Stand { m_name: "snowstuff", m_beta: "snowshovel" },
    Stand { m_name: "annihilate", m_beta: "portal" },
    Stand { m_name: "firefly", m_beta: "reddust" },
    Stand { m_name: "crit", m_beta: "snowshovel" },
    Stand { m_name: "magicCrit", m_beta: "snowshovel" },
    Stand { m_name: "largeexplode", m_beta: "explode" },
    Stand { m_name: "hugeexplosion", m_beta: "largesmoke" },
    Stand { m_name: "dripWater", m_beta: "splash" },
    Stand { m_name: "iconcrack_20_0", m_beta: "snowballpoof" },
];

pub fn spawn(world: World, name: &str, at: [f64; 3], motion: [f64; 3])
{
    let beta: &str = STANDS.iter().find(|row| row.m_name == name).map_or(name, |row| row.m_beta);
    world.particle(beta, at, motion);
}

// BlockTFTowerDevice.sparkle, a speck just outside every face that is not covered
pub fn sparkle(world: World, pos: BlockPos, name: &str)
{
    glint(world, pos, name, [0.0; 3], |at| !World::blockInfo(world.block(at)).opaque);
}

// a speck just outside every face whose neighbor passes the check
pub fn glint(world: World, pos: BlockPos, name: &str, motion: [f64; 3], open: impl Fn(BlockPos) -> bool)
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
pub fn outside(world: World, pos: BlockPos, step: [i32; 3]) -> [f64; 3]
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

pub fn line(world: World, name: &str, from: [f64; 3], to: [f64; 3], count: i32)
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
pub fn tinyFirefly(world: World, at: [f64; 3])
{
    let spread: f64 = 0.3;
    let rise: f64 = 0.4;
    let wobble = || f64::from(ids::fraction(world) - ids::fraction(world));
    spawn(world, "firefly", [at[0] + wobble() * spread, at[1] + wobble() * rise, at[2] + wobble() * spread], [0.0; 3]);
}
