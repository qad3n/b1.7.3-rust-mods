// draft from ComponentTFMazeUpperEntrance.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(i: i32, random: &Random, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::MazeUpperEntrance);
    init(&this, i, random, x, y, z);
    Rc::new(this)
}

// ComponentTFMazeUpperEntrance constructor
pub(crate) fn init(this: &Piece, i: i32, random: &Random, x: i32, y: i32, z: i32)
{
    this.m_depth.set(i);
    this.m_averageGroundLevel.set(-1);
    this.m_mode.set(random.nextIntBounded(4));
    this.m_box.set(StructureBox::new([x, y, z], [x + 15, y + 4, z + 15]));
}

// ComponentTFMazeUpperEntrance.func_74861_a
pub(crate) fn build(_this: &Piece, _structurecomponent: &Piece, _list: &mut Pieces, _random: &Random) {}

// ComponentTFMazeUpperEntrance.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    this.fillRandomly(world, sbb, random, 0.7, 0, 5, 0, 15, 5, 15, blk("tf:mazestone"), blk("air"), true);
    this.fillMeta(world, sbb, 0, 0, 0, 15, 0, 15, blk("tf:mazestone"), 6, blk("air"), 0, false);
    this.fillMeta(world, sbb, 0, 1, 0, 15, 1, 15, blk("tf:mazestone"), 3, blk("air"), 0, true);
    this.fillMeta(world, sbb, 0, 2, 0, 15, 3, 15, blk("tf:mazestone"), 1, blk("air"), 0, true);
    this.fillMeta(world, sbb, 0, 4, 0, 15, 4, 15, blk("tf:mazestone"), 3, blk("air"), 0, true);
    this.fillRandomly(world, sbb, random, 0.2, 0, 0, 0, 15, 5, 15, blk("gravel"), blk("air"), true);
    this.fill(world, sbb, 6, 1, 0, 9, 4, 0, blk("fence"), blk("air"), false);
    this.fillAir(world, sbb, 7, 1, 0, 8, 3, 0);
    this.fill(world, sbb, 6, 1, 15, 9, 4, 15, blk("fence"), blk("air"), false);
    this.fillAir(world, sbb, 7, 1, 15, 8, 3, 15);
    this.fill(world, sbb, 0, 1, 6, 0, 4, 9, blk("fence"), blk("air"), false);
    this.fillAir(world, sbb, 0, 1, 7, 0, 3, 8);
    this.fill(world, sbb, 15, 1, 6, 15, 4, 9, blk("fence"), blk("air"), false);
    this.fillAir(world, sbb, 15, 1, 7, 15, 3, 8);
    this.fillAir(world, sbb, 1, 1, 1, 14, 4, 14);
    this.fillMeta(world, sbb, 5, 1, 5, 10, 1, 10, blk("tf:mazestone"), 3, blk("air"), 0, false);
    this.fillMeta(world, sbb, 5, 4, 5, 10, 4, 10, blk("tf:mazestone"), 3, blk("air"), 0, false);
    this.fillRandomly(world, sbb, random, 0.7, 5, 2, 5, 10, 3, 10, blk("iron_bars"), blk("air"), false);
    this.fillAir(world, sbb, 6, 0, 6, 9, 4, 9);
    true
}

// ComponentTFMazeUpperEntrance.getAverageGroundLevel
pub(crate) fn getAverageGroundLevel(this: &Piece, world: &GenWorld, sbb: StructureBox) -> i32
{
    let mut var3: i32 = 0;
    let mut var4: i32 = 0;

    for var5 in this.m_box.get().m_min[2]..=this.m_box.get().m_max[2]
    {
        for var6 in this.m_box.get().m_min[0]..=this.m_box.get().m_max[0]
        {
            if sbb.contains(var6, 64, var5)
            {
                var3 += world.topSolid(var6, var5).max(crate::world::terrain::SEA_LEVEL);
                var4 += 1;
            }
        }
    }

    if var4 == 0 { -1 } else { var3 / var4 }
}
