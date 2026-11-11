use crate::core::area::Area;
use b173_api::prelude::*;

// FillerFillStairs: a staircase of layers climbing away from the filler, straight when it stands beside the box
// and corner to corner when it stands off a corner
fn layer(world: World, x: [i32; 2], height: i32, z: [i32; 2], stack: &mut Option<&mut ItemStack>) -> bool
{
    super::filler::fillLayer(world, [x[0], height, z[0]], [x[1], height, z[1]], stack.as_deref_mut())
}

pub(crate) fn iterate(world: World, pos: BlockPos, area: &Area, mut stack: Option<&mut ItemStack>) -> bool
{
    let [xMin, yMin, zMin] = area.m_min;
    let [xMax, yMax, zMax] = area.m_max;
    let sizeX: i32 = area.size(0);
    let sizeZ: i32 = area.size(2);
    let climbing: bool = pos.y <= yMin;
    let heightStep: i32 = if climbing { 1 } else { -1 };
    let mut height: i32 = if climbing { yMin } else { yMax };
    let mut steps: [i32; 4] = [0; 4];
    let mut diagonal: bool = false;

    if pos.x == xMin - 1
    {
        steps[0] = 1;
    }
    else if pos.x == xMax + 1
    {
        steps[1] = 1;
    }
    else if pos.z == zMin - 1
    {
        steps[2] = 1;
    }
    else if pos.z == zMax + 1
    {
        steps[3] = 1;
    }
    else
    {
        diagonal = true;
    }

    if !diagonal
    {
        let (mut x1, mut x2, mut z1, mut z2): (i32, i32, i32, i32) = (xMin, xMax, zMin, zMax);

        if !climbing
        {
            if steps[0] == 1
            {
                x1 = xMax - sizeX + 1;
                x2 = x1;
            }

            if steps[1] == 1
            {
                x2 = xMin + sizeX - 1;
                x1 = x2;
            }

            if steps[2] == 1
            {
                z1 = zMax - sizeZ + 1;
                z2 = z1;
            }

            if steps[3] == 1
            {
                z2 = zMin + sizeZ - 1;
                z1 = z2;
            }
        }

        while x2 - x1 + 1 > 0
            && z2 - z1 + 1 > 0
            && x2 - x1 < sizeX
            && z2 - z1 < sizeZ
            && (yMin..=yMax).contains(&height)
        {
            if !layer(world, [x1, x2], height, [z1, z2], &mut stack)
            {
                return false;
            }

            if climbing
            {
                x1 += steps[0];
                x2 -= steps[1];
                z1 += steps[2];
                z2 -= steps[3];
            }
            else
            {
                x2 += steps[0];
                x1 -= steps[1];
                z2 += steps[2];
                z1 -= steps[3];
            }

            height += heightStep;
        }

        return true;
    }

    let x: i32 = if pos.x <= xMin
    {
        xMin
    }
    else if pos.x >= xMax
    {
        xMax
    }
    else
    {
        0
    };
    let z: i32 = if pos.z <= zMin
    {
        zMin
    }
    else if pos.z >= zMax
    {
        zMax
    }
    else
    {
        0
    };
    let (mut dimX, mut dimZ, stepX, stepZ): (i32, i32, i32, i32) = if climbing
    {
        (sizeX - 1, sizeZ - 1, -1, -1)
    }
    else
    {
        (0, 0, 1, 1)
    };

    while (0..sizeX).contains(&dimX) && (0..sizeZ).contains(&dimZ) && (yMin..=yMax).contains(&height)
    {
        let (x1, x2): (i32, i32) = match (climbing, pos.x >= xMax, pos.x <= xMin)
        {
            (true, true, _) => (x - sizeX + 1, x - sizeX + 1 + dimX),
            (true, false, _) => (x + sizeX - 1 - dimX, x + sizeX - 1),
            (false, _, true) => (x, x + dimX),
            (false, _, false) => (x - dimX, x),
        };
        let (z1, z2): (i32, i32) = match (climbing, pos.z >= zMax, pos.z <= zMin)
        {
            (true, true, _) => (z - sizeZ + 1, z - sizeZ + 1 + dimZ),
            (true, false, _) => (z + sizeZ - 1 - dimZ, z + sizeZ - 1),
            (false, _, true) => (z, z + dimZ),
            (false, _, false) => (z - dimZ, z),
        };

        if !layer(world, [x1, x2], height, [z1, z2], &mut stack)
        {
            return false;
        }

        dimX += stepX;
        dimZ += stepZ;
        height += heightStep;
    }

    true
}
