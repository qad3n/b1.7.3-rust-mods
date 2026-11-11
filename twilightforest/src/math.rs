use b173_api::terrain::math::{cos, sin};

// java.awt.Color.RGBtoHSB
pub(crate) fn toHsb(red: u32, green: u32, blue: u32) -> (f32, f32, f32)
{
    let max: u32 = red.max(green).max(blue);
    let min: u32 = red.min(green).min(blue);
    let brightness: f32 = max as f32 / 255.0;

    let saturation: f32 = if max == 0
    {
        0.0
    }
    else
    {
        (max - min) as f32 / max as f32
    };

    if saturation == 0.0
    {
        return (0.0, saturation, brightness);
    }

    let span: f32 = (max - min) as f32;
    let redC: f32 = (max - red) as f32 / span;
    let greenC: f32 = (max - green) as f32 / span;
    let blueC: f32 = (max - blue) as f32 / span;

    let mut hue: f32 = if red == max
    {
        blueC - greenC
    }
    else if green == max
    {
        2.0 + redC - blueC
    }
    else
    {
        4.0 + greenC - redC
    };
    hue /= 6.0;
    if hue < 0.0
    {
        hue += 1.0;
    }

    (hue, saturation, brightness)
}

// java.awt.Color.HSBtoRGB without the alpha byte
pub(crate) fn fromHsb(hue: f32, saturation: f32, brightness: f32) -> u32
{
    let scale = |value: f32| (value * 255.0 + 0.5) as u32;

    if saturation == 0.0
    {
        let grey: u32 = scale(brightness);
        return grey << 16 | grey << 8 | grey;
    }

    let sector: f32 = (hue - hue.floor()) * 6.0;
    let fraction: f32 = sector - sector.floor();
    let low: f32 = brightness * (1.0 - saturation);
    let falling: f32 = brightness * (1.0 - saturation * fraction);
    let rising: f32 = brightness * (1.0 - saturation * (1.0 - fraction));

    let (red, green, blue): (f32, f32, f32) = match sector as i32
    {
        0 => (brightness, rising, low),
        1 => (falling, brightness, low),
        2 => (low, brightness, rising),
        3 => (low, falling, brightness),
        4 => (rising, low, brightness),
        _ => (brightness, low, falling),
    };
    scale(red) << 16 | scale(green) << 8 | scale(blue)
}

// Entity.getLook from a yaw and pitch in degrees, through the MathHelper sine table
pub(crate) fn look(yaw: f32, pitch: f32) -> [f64; 3]
{
    let radians: f32 = std::f32::consts::PI / 180.0;
    let turnCos: f32 = cos(-yaw * radians - std::f32::consts::PI);
    let turnSin: f32 = sin(-yaw * radians - std::f32::consts::PI);
    let tiltCos: f32 = -cos(-pitch * radians);
    let tiltSin: f32 = sin(-pitch * radians);
    [f64::from(turnSin * tiltCos), f64::from(tiltSin), f64::from(turnCos * tiltCos)]
}

// Random.nextGaussian by the Box Muller transform from two uniform draws in (0, 1]
pub(crate) fn gaussian(first: f64, second: f64) -> f64
{
    let radius: f64 = (-2.0 * first.max(f64::MIN_POSITIVE).ln()).sqrt();
    radius * (std::f64::consts::TAU * second).cos()
}

// EntityLiving.knockBack, halve the motion then shove away from the hit along the ground and up
pub(crate) fn knockback(motion: [f64; 3], toward: [f64; 2]) -> [f64; 3]
{
    let push: f64 = 0.4;
    let length: f64 = toward[0].hypot(toward[1]);
    let [x, z]: [f64; 2] = if length > 0.0
    {
        [toward[0] / length, toward[1] / length]
    }
    else
    {
        [0.0; 2]
    };
    [motion[0] / 2.0 - x * push, (motion[1] / 2.0 + push).min(push), motion[2] / 2.0 - z * push]
}

pub(crate) fn length(vector: [f64; 3]) -> f64 { vector.iter().map(|axis| axis * axis).sum::<f64>().sqrt() }

pub(crate) fn dot(a: [f64; 3], b: [f64; 3]) -> f64 { (0..3).map(|axis| a[axis] * b[axis]).sum() }

// Vec3.normalize, a vector too short to point anywhere becomes zero
pub(crate) fn normalize(vector: [f64; 3]) -> [f64; 3]
{
    let tiny: f64 = 1.0e-4;
    let size: f64 = length(vector);
    if size < tiny
    {
        [0.0; 3]
    }
    else
    {
        vector.map(|axis| axis / size)
    }
}

// Vec3.rotateAroundY
pub(crate) fn turn(vector: [f64; 3], angle: f64) -> [f64; 3]
{
    let (sin, cos): (f64, f64) = angle.sin_cos();
    [vector[0] * cos + vector[2] * sin, vector[1], vector[2] * cos - vector[0] * sin]
}

// EntityArrow.setArrowHeading's yaw and pitch in degrees for a flight direction
pub(crate) fn heading(motion: [f64; 3]) -> (f64, f64)
{
    let horizontal: f64 = motion[0].hypot(motion[2]);
    (motion[0].atan2(motion[2]).to_degrees(), motion[1].atan2(horizontal).to_degrees())
}

// String.hashCode, the seed a text setting stands for in Java
pub(crate) fn javaHash(text: &str) -> i32
{
    let factor: i32 = 31;
    text.encode_utf16().fold(0, |hash: i32, unit| hash.wrapping_mul(factor).wrapping_add(i32::from(unit)))
}
