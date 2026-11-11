// draft from ComponentTFMazeRoomCollapse.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(i: i32, random: &Random, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::MazeRoomCollapse);
    init(&this, i, random, x, y, z);
    Rc::new(this)
}

// ComponentTFMazeRoomCollapse constructor
pub(crate) fn init(this: &Piece, i: i32, random: &Random, x: i32, y: i32, z: i32)
{
    maze_room::init(this, i, random, x, y, z);
}

// ComponentTFMazeRoomCollapse.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    maze_room::add(this, world, random, sbb);

    for x in 1..14
    {
        for z in 1..14
        {
            let dist: i32 = (7.0
                / (((7.5 - (x) as f64) * (7.5 - (x) as f64)) + ((7.5 - (z) as f64) * (7.5 - (z) as f64))).sqrt())
            .round() as i32;

            let mut gravel: i32 = random.nextIntBounded(dist);
            let root: i32 = random.nextIntBounded(dist);

            if gravel > 0
            {
                this.fill(
                    world,
                    sbb,
                    x,
                    1,
                    z,
                    x,
                    {
                        gravel += 1;
                        gravel
                    },
                    z,
                    blk("gravel"),
                    blk("air"),
                    false,
                );
                this.fillAir(world, sbb, x, gravel, z, x, gravel + 5, z);
            }
            else if root > 0
            {
                this.fill(world, sbb, x, 5, z, x, 5 + root, z, blk("dirt"), blk("air"), true);
                this.fillMeta(
                    world,
                    sbb,
                    x,
                    5 - random.nextIntBounded(5),
                    z,
                    x,
                    5,
                    z,
                    blk("tf:plant"),
                    14,
                    blk("air"),
                    0,
                    false,
                );
            }
            else if random.nextIntBounded(dist + 1) > 0
            {
                this.fillAir(world, sbb, x, 5, z, x, 5, z);
            }
        }
    }

    true
}
