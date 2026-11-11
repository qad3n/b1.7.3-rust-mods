use super::gen_world::GenWorld;
use crate::names::{AIR, Pick};
use b173_api::terrain::Random;
use b173_api::{BlockInfo, World};

const WATER: &str = "minecraft:water";

mod lake
{
    pub const WIDTH: i32 = 16;
    pub const DEPTH: i32 = 8;
    pub const WATER_LAYERS: i32 = 4;
    pub const MIN_BLOBS: i32 = 4;
    pub const BLOB_SPREAD: i32 = 4;
}

fn index(x: i32, z: i32, y: i32) -> usize { ((x * lake::WIDTH + z) * lake::DEPTH + y) as usize }

fn isEdge(shape: &[bool], x: i32, z: i32, y: i32) -> bool
{
    !shape[index(x, z, y)]
        && (x < lake::WIDTH - 1 && shape[index(x + 1, z, y)]
            || x > 0 && shape[index(x - 1, z, y)]
            || z < lake::WIDTH - 1 && shape[index(x, z + 1, y)]
            || z > 0 && shape[index(x, z - 1, y)]
            || y < lake::DEPTH - 1 && shape[index(x, z, y + 1)]
            || y > 0 && shape[index(x, z, y - 1)])
}

fn blobs(random: &Random) -> Vec<bool>
{
    let mut shape: Vec<bool> = vec![false; (lake::WIDTH * lake::WIDTH * lake::DEPTH) as usize];
    let count: i32 = random.nextIntBounded(lake::BLOB_SPREAD) + lake::MIN_BLOBS;
    for _ in 0..count
    {
        let sizeX: f64 = random.nextDouble() * 6.0 + 3.0;
        let sizeY: f64 = random.nextDouble() * 4.0 + 2.0;
        let sizeZ: f64 = random.nextDouble() * 6.0 + 3.0;
        let centerX: f64 = random.nextDouble() * (16.0 - sizeX - 2.0) + 1.0 + sizeX / 2.0;
        let centerY: f64 = random.nextDouble() * (8.0 - sizeY - 4.0) + 2.0 + sizeY / 2.0;
        let centerZ: f64 = random.nextDouble() * (16.0 - sizeZ - 2.0) + 1.0 + sizeZ / 2.0;

        for x in 1..lake::WIDTH - 1
        {
            for z in 1..lake::WIDTH - 1
            {
                for y in 1..lake::DEPTH - 1
                {
                    let dx: f64 = (x as f64 - centerX) / (sizeX / 2.0);
                    let dy: f64 = (y as f64 - centerY) / (sizeY / 2.0);
                    let dz: f64 = (z as f64 - centerZ) / (sizeZ / 2.0);
                    if dx * dx + dy * dy + dz * dz < 1.0
                    {
                        shape[index(x, z, y)] = true;
                    }
                }
            }
        }
    }

    shape
}

// WorldGenLakes.generate with water, which the Aether decorator uses unchanged; its grass and lava passes
// only touch vanilla dirt and lava lakes, so they never fire here
pub(crate) fn waterLake(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32) -> bool
{
    let x: i32 = x - lake::WIDTH / 2;
    let z: i32 = z - lake::WIDTH / 2;
    let mut y: i32 = y;
    while y > 0 && world.isAir(x, y, z)
    {
        y -= 1;
    }

    y -= lake::WATER_LAYERS;

    let shape: Vec<bool> = blobs(random);
    for dx in 0..lake::WIDTH
    {
        for dz in 0..lake::WIDTH
        {
            for dy in 0..lake::DEPTH
            {
                if !isEdge(&shape, dx, dz, dy)
                {
                    continue;
                }
                let block: i32 = world.block(x + dx, y + dy, z + dz);
                let info: BlockInfo = World::blockInfo(block);
                if dy >= lake::WATER_LAYERS && info.liquid
                {
                    return false;
                }
                if dy < lake::WATER_LAYERS && !info.solid && !world.is(x + dx, y + dy, z + dz, WATER)
                {
                    return false;
                }
            }
        }
    }

    for dx in 0..lake::WIDTH
    {
        for dz in 0..lake::WIDTH
        {
            for dy in 0..lake::DEPTH
            {
                if shape[index(dx, dz, dy)]
                {
                    world.put(
                        x + dx,
                        y + dy,
                        z + dz,
                        if dy >= lake::WATER_LAYERS
                        {
                            AIR
                        }
                        else
                        {
                            Pick::of(WATER)
                        },
                    );
                }
            }
        }
    }

    true
}
