use b173_api::World;

// java.util.Random's nextFloat and nextGaussian shapes over the world's random, for the drop motions machines use
const RESOLUTION: i32 = 1 << 24;

pub(crate) fn float(world: World) -> f64 { f64::from(world.random(RESOLUTION)) / f64::from(RESOLUTION) }

// Box-Muller from two uniform draws
pub(crate) fn gaussian(world: World) -> f64
{
    let first: f64 = float(world).max(f64::MIN_POSITIVE);
    let second: f64 = float(world);
    (-2.0 * first.ln()).sqrt() * (std::f64::consts::TAU * second).cos()
}
