use std::f64::consts::PI;

// an angle difference brought into [-PI, PI) by whole turns, as TextureCompassFX steps it
pub(crate) fn wrapAngle(mut angle: f64) -> f64
{
    while angle < -PI
    {
        angle += PI * 2.0;
    }

    while angle >= PI
    {
        angle -= PI * 2.0;
    }

    angle
}
