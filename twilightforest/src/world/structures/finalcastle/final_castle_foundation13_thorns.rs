// draft from ComponentTFFinalCastleFoundation13Thorns.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(random: &Random, i: i32, sideTower: &Piece) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::FinalCastleFoundation13Thorns);
    init(&this, random, i, sideTower);
    Rc::new(this)
}

// ComponentTFFinalCastleFoundation13Thorns constructor
pub(crate) fn init(this: &Piece, random: &Random, i: i32, sideTower: &Piece)
{
    final_castle_foundation13::init(this, random, i, sideTower);
    this.m_box.set(StructureBox::new(
        [sideTower.bounds().m_min[0] - 5, sideTower.bounds().m_max[1] - 1, sideTower.bounds().m_min[2] - 5],
        [sideTower.bounds().m_max[0] + 5, sideTower.bounds().m_max[1], sideTower.bounds().m_max[2] + 5],
    ));
}

// ComponentTFFinalCastleFoundation13Thorns.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, _random: &Random, sbb: StructureBox) -> bool
{
    let decoRNG: Random = Random::withSeed(
        (world.seed() + ((this.m_box.get().m_min[0] * 321534781) as i64))
            ^ ((this.m_box.get().m_min[2] * 756839) as i64),
    );

    for i in 0..4
    {
        final_castle_foundation13_thorns::makeThornVine(this, world, &decoRNG, i, sbb);
    }

    true
}

// ComponentTFFinalCastleFoundation13Thorns.makeThornVine
pub(crate) fn makeThornVine(this: &Piece, world: &GenWorld, decoRNG: &Random, rotation: i32, sbb: StructureBox)
{
    let x: i32 = 3 + decoRNG.nextIntBounded(13);
    let z: i32 = 3 + decoRNG.nextIntBounded(13);
    let mut y: i32 = this.m_box.get().ySize() + 5;
    let mut twist: i32 = decoRNG.nextIntBounded(4);
    let twistMod: i32 = 3 + decoRNG.nextIntBounded(3);

    while (this.getTurned(world, x, y, z, rotation, sbb) != blk("tf:deadrock")) && (this.yAt(y) > 60)
    {
        this.placeTurned(world, blk("tf:thorns"), 0, x, y, z, rotation, sbb);

        match twist
        {
            0 =>
            {
                this.placeTurned(world, blk("tf:thorns"), 0, x + 1, y, z, rotation, sbb);
                this.placeTurned(world, blk("tf:thorns"), 0, x, y, z + 1, rotation, sbb);
                this.placeTurned(world, blk("tf:thorns"), 0, x + 1, y, z + 1, rotation, sbb);
            }

            1 =>
            {
                this.placeTurned(world, blk("tf:thorns"), 0, x + 1, y, z, rotation, sbb);
                this.placeTurned(world, blk("tf:thorns"), 0, x, y, z - 1, rotation, sbb);
                this.placeTurned(world, blk("tf:thorns"), 0, x + 1, y, z - 1, rotation, sbb);
            }

            2 =>
            {
                this.placeTurned(world, blk("tf:thorns"), 0, x - 1, y, z, rotation, sbb);
                this.placeTurned(world, blk("tf:thorns"), 0, x, y, z - 1, rotation, sbb);
                this.placeTurned(world, blk("tf:thorns"), 0, x - 1, y, z - 1, rotation, sbb);
            }

            3 =>
            {
                this.placeTurned(world, blk("tf:thorns"), 0, x - 1, y, z, rotation, sbb);
                this.placeTurned(world, blk("tf:thorns"), 0, x, y, z + 1, rotation, sbb);
                this.placeTurned(world, blk("tf:thorns"), 0, x - 1, y, z + 1, rotation, sbb);
            }

            _ =>
            {}
        }

        if (y % twistMod).abs() == 1
        {
            final_castle_foundation13_thorns::makeThornBranch(this, world, x, y, z, rotation, sbb);
        }

        if (y % twistMod) == 0
        {
            twist =
            {
                twist += 1;
                twist
            } % 4;
        }

        y -= 1;
    }
}

// ComponentTFFinalCastleFoundation13Thorns.makeThornBranch
pub(crate) fn makeThornBranch(this: &Piece, world: &GenWorld, x: i32, y: i32, z: i32, rotation: i32, sbb: StructureBox)
{
    let rand: Random = Random::withSeed((world.seed() + ((x * 321534781) as i64)) ^ (((y * 756839) + z) as i64));
    let dir: i32 = rand.nextIntBounded(4);
    let mut dx: i32 = 0;
    let mut dz: i32 = 0;

    match dir
    {
        0 =>
        {
            dx = 1;
        }

        1 =>
        {
            dz = 1;
        }

        2 =>
        {
            dx = -1;
        }

        3 =>
        {
            dz = -1;
        }

        _ =>
        {}
    }

    let dist: i32 = 2 + rand.nextIntBounded(3);
    let destX: i32 = x + (dist * dx);
    let destZ: i32 = z + (dist * dz);
    if (((destX > 0) && (destX < this.m_box.get().xSize())) && (destZ > 0)) && (destZ < this.m_box.get().zSize())
    {
        for i in 0..dist
        {
            let branchMeta: i32 = if (((dir + rotation) + this.m_mode.get()) % 2) == 0
            {
                5
            }
            else
            {
                9
            };

            if i > 0
            {
                this.placeTurned(world, blk("tf:thorns"), branchMeta, x + (dx * i), y, z + (dz * i), rotation, sbb);
            }

            this.placeTurned(world, blk("tf:thorns"), 1, destX, y + i, destZ, rotation, sbb);

            if i > (dist / 2)
            {
                this.placeTurned(
                    world,
                    blk("tf:thorns"),
                    branchMeta,
                    x + (dx * i),
                    (y + dist) - 1,
                    z + (dz * i),
                    rotation,
                    sbb,
                );
            }
        }
    }
}
