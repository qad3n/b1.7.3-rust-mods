// the unit vector a yaw and pitch in degrees look along, as EntityLiving.getLook
pub(crate) fn look(yaw: f32, pitch: f32) -> [f64; 3]
{
    let yaw: f64 = f64::from(yaw).to_radians();
    let pitch: f64 = f64::from(pitch).to_radians();
    [-yaw.sin() * pitch.cos(), -pitch.sin(), yaw.cos() * pitch.cos()]
}
