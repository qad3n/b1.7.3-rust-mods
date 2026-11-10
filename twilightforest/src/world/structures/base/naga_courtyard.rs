// draft from ComponentTFNagaCourtyard.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub const RADIUS: i32 = 46;
pub const DIAMETER: i32 = (2 * RADIUS) + 1;

pub fn new(world: &GenWorld, random: &Random, i: i32, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::NagaCourtyard);
    init(&this, world, random, i, x, y, z);
    Rc::new(this)
}

// ComponentTFNagaCourtyard constructor
pub fn init(this: &Piece, world: &GenWorld, random: &Random, i: i32, x: i32, y: i32, z: i32)
{
    this.m_depth.set(i);
    this.setMode(0);
    this.m_box.set(boxFor(x, y, z, -RADIUS, -1, -RADIUS, RADIUS * 2, 10, RADIUS * 2, 0));
}

// ComponentTFNagaCourtyard.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    for fx in 0..=DIAMETER
    {
        for fz in 0..=DIAMETER
        {
            if random.nextIntBounded(3) == 0
            {
                this.placeBlock(world, blk("double_stone_slab"), 0, fx, 0, fz, sbb);
                if random.nextIntBounded(20) == 0
                {
                    this.placeBlock(world, blk("stone_slab"), 0, fx, 1, fz, sbb);
                }
                else
                {
                    this.placeBlock(world, blk("air"), 0, fx, 1, fz, sbb);
                }
            }
            else
            {
                this.placeBlock(world, blk("grass"), 0, fx, 0, fz, sbb);
            }
        }
    }
    for fx in 0..=DIAMETER
    {
        naga_courtyard::randomBrick(this, world, random, fx, 0, DIAMETER, sbb);
        naga_courtyard::randomBrick(this, world, random, fx, 0, 0, sbb);
        naga_courtyard::randomBrick(this, world, random, fx, 1, DIAMETER, sbb);
        naga_courtyard::randomBrick(this, world, random, fx, 1, 0, sbb);
        naga_courtyard::randomBrick(this, world, random, fx, 2, DIAMETER, sbb);
        naga_courtyard::randomBrick(this, world, random, fx, 2, 0, sbb);
        naga_courtyard::randomBrick(this, world, random, fx, 3, DIAMETER, sbb);
        naga_courtyard::randomBrick(this, world, random, fx, 3, 0, sbb);
        this.placeBlock(world, blk("stone_slab"), 5, fx, 4, DIAMETER, sbb);
        this.placeBlock(world, blk("stone_slab"), 5, fx, 4, 0, sbb);
        match fx % 23
        {
            2 =>
            {
                this.placeBlock(world, blk("tf:nagastone"), 7, fx, 3, DIAMETER, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 7, fx, 3, 0, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 14, fx, 2, DIAMETER, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 14, fx, 2, 0, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 11, fx, 1, DIAMETER, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 11, fx, 1, 0, sbb);
            }
            3 =>
            {
                this.placeBlock(world, blk("tf:nagastone"), 3, fx, 3, DIAMETER, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 3, fx, 3, 0, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 12, fx, 1, DIAMETER, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 12, fx, 1, 0, sbb);
            }
            4 | 8 | 16 | 20 =>
            {
                this.placeBlock(world, blk("tf:nagastone"), 12, fx, 1, DIAMETER, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 12, fx, 1, 0, sbb);
            }
            5 | 9 | 17 =>
            {
                this.placeBlock(world, blk("tf:nagastone"), 7, fx, 3, DIAMETER, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 7, fx, 3, 0, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 14, fx, 2, DIAMETER, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 14, fx, 2, 0, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 10, fx, 1, DIAMETER, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 10, fx, 1, 0, sbb);
            }
            6 | 10 | 14 | 18 =>
            {
                this.placeBlock(world, blk("tf:nagastone"), 12, fx, 3, DIAMETER, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 12, fx, 3, 0, sbb);
            }
            7 | 15 | 19 =>
            {
                this.placeBlock(world, blk("tf:nagastone"), 6, fx, 3, DIAMETER, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 6, fx, 3, 0, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 14, fx, 2, DIAMETER, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 14, fx, 2, 0, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 11, fx, 1, DIAMETER, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 11, fx, 1, 0, sbb);
            }
            // todo falls through to the next case
            11 =>
            {
                this.placeBlock(world, blk("tf:nagastone"), 6, fx, 3, DIAMETER, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 6, fx, 3, 0, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 14, fx, 2, DIAMETER, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 14, fx, 2, 0, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 14, fx, 1, DIAMETER, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 14, fx, 1, 0, sbb);
            }
            12 =>
            {}
            13 =>
            {
                this.placeBlock(world, blk("tf:nagastone"), 7, fx, 3, DIAMETER, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 7, fx, 3, 0, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 14, fx, 2, DIAMETER, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 14, fx, 2, 0, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 14, fx, 1, DIAMETER, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 14, fx, 1, 0, sbb);
            }
            21 =>
            {
                this.placeBlock(world, blk("tf:nagastone"), 2, fx, 3, DIAMETER, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 2, fx, 3, 0, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 12, fx, 1, DIAMETER, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 12, fx, 1, 0, sbb);
            }
            22 =>
            {
                this.placeBlock(world, blk("tf:nagastone"), 6, fx, 3, DIAMETER, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 6, fx, 3, 0, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 14, fx, 2, DIAMETER, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 14, fx, 2, 0, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 10, fx, 1, DIAMETER, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 10, fx, 1, 0, sbb);
            }
            _ =>
            {}
        }
    }
    for fz in 0..=DIAMETER
    {
        naga_courtyard::randomBrick(this, world, random, DIAMETER, 0, fz, sbb);
        naga_courtyard::randomBrick(this, world, random, 0, 0, fz, sbb);
        naga_courtyard::randomBrick(this, world, random, DIAMETER, 1, fz, sbb);
        naga_courtyard::randomBrick(this, world, random, 0, 1, fz, sbb);
        naga_courtyard::randomBrick(this, world, random, DIAMETER, 2, fz, sbb);
        naga_courtyard::randomBrick(this, world, random, 0, 2, fz, sbb);
        naga_courtyard::randomBrick(this, world, random, DIAMETER, 3, fz, sbb);
        naga_courtyard::randomBrick(this, world, random, 0, 3, fz, sbb);
        this.placeBlock(world, blk("stone_slab"), 5, DIAMETER, 4, fz, sbb);
        this.placeBlock(world, blk("stone_slab"), 5, 0, 4, fz, sbb);
        match fz % 23
        {
            2 =>
            {
                this.placeBlock(world, blk("tf:nagastone"), 5, DIAMETER, 3, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 5, 0, 3, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 14, DIAMETER, 2, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 14, 0, 2, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 9, DIAMETER, 1, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 9, 0, 1, fz, sbb);
            }
            3 =>
            {
                this.placeBlock(world, blk("tf:nagastone"), 1, DIAMETER, 3, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 1, 0, 3, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 13, DIAMETER, 1, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 13, 0, 1, fz, sbb);
            }
            4 | 8 | 16 | 20 =>
            {
                this.placeBlock(world, blk("tf:nagastone"), 13, DIAMETER, 1, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 13, 0, 1, fz, sbb);
            }
            5 | 9 | 17 =>
            {
                this.placeBlock(world, blk("tf:nagastone"), 5, DIAMETER, 3, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 5, 0, 3, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 14, DIAMETER, 2, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 14, 0, 2, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 8, DIAMETER, 1, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 8, 0, 1, fz, sbb);
            }
            6 | 10 | 14 | 18 =>
            {
                this.placeBlock(world, blk("tf:nagastone"), 13, DIAMETER, 3, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 13, 0, 3, fz, sbb);
            }
            7 | 15 | 19 =>
            {
                this.placeBlock(world, blk("tf:nagastone"), 4, DIAMETER, 3, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 4, 0, 3, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 14, DIAMETER, 2, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 14, 0, 2, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 9, DIAMETER, 1, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 9, 0, 1, fz, sbb);
            }
            // todo falls through to the next case
            11 =>
            {
                this.placeBlock(world, blk("tf:nagastone"), 4, DIAMETER, 3, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 4, 0, 3, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 14, DIAMETER, 2, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 14, 0, 2, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 14, DIAMETER, 1, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 14, 0, 1, fz, sbb);
            }
            12 =>
            {}
            13 =>
            {
                this.placeBlock(world, blk("tf:nagastone"), 5, DIAMETER, 3, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 5, 0, 3, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 14, DIAMETER, 2, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 14, 0, 2, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 14, DIAMETER, 1, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 14, 0, 1, fz, sbb);
            }
            21 =>
            {
                this.placeBlock(world, blk("tf:nagastone"), 0, DIAMETER, 3, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 0, 0, 3, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 13, DIAMETER, 1, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 13, 0, 1, fz, sbb);
            }
            22 =>
            {
                this.placeBlock(world, blk("tf:nagastone"), 4, DIAMETER, 3, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 4, 0, 3, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 14, DIAMETER, 2, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 14, 0, 2, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 8, DIAMETER, 1, fz, sbb);
                this.placeBlock(world, blk("tf:nagastone"), 8, 0, 1, fz, sbb);
            }
            _ =>
            {}
        }
    }
    let pillarRand: Random =
        Random::withSeed(world.seed() + ((this.m_box.get().m_min[0] * this.m_box.get().m_min[2]) as i64));
    for i in 0..20
    {
        let rx: i32 = 2 + pillarRand.nextIntBounded(DIAMETER - 4);
        let rz: i32 = 2 + pillarRand.nextIntBounded(DIAMETER - 4);
        naga_courtyard::makePillar(this, world, &pillarRand, rx, 1, rz, sbb);
    }
    this.placeBlock(world, blk("tf:bossSpawner"), 0, RADIUS + 1, 2, RADIUS + 1, sbb);
    return true;
}

// ComponentTFNagaCourtyard.makePillar
pub fn makePillar(this: &Piece, world: &GenWorld, random: &Random, x: i32, y: i32, z: i32, sbb: StructureBox) -> bool
{
    let height: i32 = 8;
    this.placeBlock(world, blk("stone_slab"), 0, x - 1, y + 0, z - 1, sbb);
    this.placeBlock(world, blk("stone_slab"), 0, x + 0, y + 0, z - 1, sbb);
    this.placeBlock(world, blk("stone_slab"), 0, x + 1, y + 0, z - 1, sbb);
    this.placeBlock(world, blk("stone_slab"), 0, x - 1, y + 0, z + 0, sbb);
    this.placeBlock(world, blk("stone_slab"), 0, x + 1, y + 0, z + 0, sbb);
    this.placeBlock(world, blk("stone_slab"), 0, x - 1, y + 0, z + 1, sbb);
    this.placeBlock(world, blk("stone_slab"), 0, x + 0, y + 0, z + 1, sbb);
    this.placeBlock(world, blk("stone_slab"), 0, x + 1, y + 0, z + 1, sbb);
    for i in 0..height
    {
        naga_courtyard::randomBrick(this, world, random, x, y + i, z, sbb);
        if (i > 0) && (random.nextIntBounded(2) == 0)
        {
            match random.nextIntBounded(4)
            {
                0 =>
                {
                    this.placeBlock(world, blk("vine"), 8, x - 1, y + i, z + 0, sbb);
                }
                1 =>
                {
                    this.placeBlock(world, blk("vine"), 2, x + 1, y + i, z + 0, sbb);
                }
                2 =>
                {
                    this.placeBlock(world, blk("vine"), 4, x + 0, y + i, z + 1, sbb);
                }
                3 =>
                {
                    this.placeBlock(world, blk("vine"), 1, x + 0, y + i, z - 1, sbb);
                }
                _ =>
                {}
            }
        }
        else if (i > 0) && (random.nextIntBounded(4) == 0)
        {
            match random.nextIntBounded(4)
            {
                0 =>
                {
                    this.placeBlock(world, blk("tf:firefly"), 0, x - 1, y + i, z + 0, sbb);
                }
                1 =>
                {
                    this.placeBlock(world, blk("tf:firefly"), 0, x + 1, y + i, z + 0, sbb);
                }
                2 =>
                {
                    this.placeBlock(world, blk("tf:firefly"), 0, x + 0, y + i, z + 1, sbb);
                }
                3 =>
                {
                    this.placeBlock(world, blk("tf:firefly"), 0, x + 0, y + i, z - 1, sbb);
                }
                _ =>
                {}
            }
        }
    }
    if height == 8
    {
        this.placeBlock(world, blk("stone_slab"), 0, x - 1, y + 8, z - 1, sbb);
        this.placeBlock(world, blk("stone_slab"), 0, x + 0, y + 8, z - 1, sbb);
        this.placeBlock(world, blk("stone_slab"), 0, x + 1, y + 8, z - 1, sbb);
        this.placeBlock(world, blk("stone_slab"), 0, x - 1, y + 8, z + 0, sbb);
        this.placeBlock(world, blk("stone_slab"), 5, x + 0, y + 8, z + 0, sbb);
        this.placeBlock(world, blk("stone_slab"), 0, x + 1, y + 8, z + 0, sbb);
        this.placeBlock(world, blk("stone_slab"), 0, x - 1, y + 8, z + 1, sbb);
        this.placeBlock(world, blk("stone_slab"), 0, x + 0, y + 8, z + 1, sbb);
        this.placeBlock(world, blk("stone_slab"), 0, x + 1, y + 8, z + 1, sbb);
    }
    return true;
}

// ComponentTFNagaCourtyard.randomBrick
pub fn randomBrick(this: &Piece, world: &GenWorld, random: &Random, x: i32, y: i32, z: i32, sbb: StructureBox)
{
    this.placeBlock(world, blk("stonebrick"), random.nextIntBounded(3), x, y, z, sbb);
}
