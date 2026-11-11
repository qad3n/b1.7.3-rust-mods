use b173_api::prelude::*;
use b173_api::terrain::Random;
use b173_api::terrain::math::{cos, floor, sin};

mod shape
{
    // WorldGenMinable: the vein runs from size/8 either side of the block centre and rises 2 to 4 above y
    pub const CENTER: i32 = 8;
    pub const REACH: f32 = 8.0;
    pub const RISE_SPREAD: i32 = 3;
    pub const RISE: i32 = 2;
    pub const WIDTH: f64 = 16.0;
    pub const HALF: f64 = 0.5;
}

fn inside(offset: f64, radius: f64) -> f64 { offset / (radius / 2.0) }

// WorldGenMinable.generate: a blob of ore replacing stone, drawn with the same random draws as vanilla
pub(crate) fn generate(world: World, random: &Random, ore: &StateHandle, size: i32, at: [i32; 3])
{
    let [x, y, z] = at;
    let angle: f32 = random.nextFloat() * std::f32::consts::PI;
    let spread: f32 = size as f32 / shape::REACH;
    let x0: f64 = f64::from((x + shape::CENTER) as f32 + sin(angle) * spread);
    let x1: f64 = f64::from((x + shape::CENTER) as f32 - sin(angle) * spread);
    let z0: f64 = f64::from((z + shape::CENTER) as f32 + cos(angle) * spread);
    let z1: f64 = f64::from((z + shape::CENTER) as f32 - cos(angle) * spread);
    let y0: f64 = f64::from(y + random.nextIntBounded(shape::RISE_SPREAD) + shape::RISE);
    let y1: f64 = f64::from(y + random.nextIntBounded(shape::RISE_SPREAD) + shape::RISE);

    for i in 0..=size
    {
        let part: f64 = f64::from(i) / f64::from(size);
        let cx: f64 = x0 + (x1 - x0) * part;
        let cy: f64 = y0 + (y1 - y0) * part;
        let cz: f64 = z0 + (z1 - z0) * part;
        let scale: f64 = random.nextDouble() * f64::from(size) / shape::WIDTH;
        let bulge: f64 = f64::from(sin(i as f32 * std::f32::consts::PI / size as f32) + 1.0);
        let wide: f64 = bulge * scale + 1.0;
        let tall: f64 = bulge * scale + 1.0;

        for bx in floor(cx - wide / 2.0)..=floor(cx + wide / 2.0)
        {
            let dx: f64 = inside(f64::from(bx) + shape::HALF - cx, wide);
            if dx * dx >= 1.0
            {
                continue;
            }

            for by in floor(cy - tall / 2.0)..=floor(cy + tall / 2.0)
            {
                let dy: f64 = inside(f64::from(by) + shape::HALF - cy, tall);
                if dx * dx + dy * dy >= 1.0
                {
                    continue;
                }

                for bz in floor(cz - wide / 2.0)..=floor(cz + wide / 2.0)
                {
                    let dz: f64 = inside(f64::from(bz) + shape::HALF - cz, wide);
                    let pos: BlockPos = BlockPos::new(bx, by, bz);
                    if dx * dx + dy * dy + dz * dz < 1.0 && world.tagged(pos, "minecraft:base_stone_overworld")
                    {
                        let _ = ore.place(world, pos, false);
                    }
                }
            }
        }
    }
}
