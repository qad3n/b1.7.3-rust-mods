// draft from ComponentTFMazeEntranceShaft.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, random: &Random, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::MazeEntranceShaft);
    init(&this, i, random, x, y, z);
    Rc::new(this)
}

// ComponentTFMazeEntranceShaft constructor
pub fn init(this: &Piece, i: i32, random: &Random, x: i32, y: i32, z: i32)
{
    this.m_depth.set(i);
    this.m_averageGroundLevel.set(-1);
    this.m_mode.set(random.nextIntBounded(4));
    this.m_box.set(StructureBox::new([x, y, z], [(x + 6) - 1, y + 14, (z + 6) - 1]));
}

// ComponentTFMazeEntranceShaft.func_74861_a
pub fn build(_this: &Piece, _structurecomponent: &Piece, _list: &mut Pieces, _random: &Random) {}

// ComponentTFMazeEntranceShaft.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, _random: &Random, sbb: StructureBox) -> bool
{
    if this.m_averageGroundLevel.get() < 0
    {
        this.m_averageGroundLevel.set(this.groundLevel(world, sbb));
        if this.m_averageGroundLevel.get() < 0
        {
            return true;
        }
        this.boxed(|bounds|
        {
            bounds.shift([0, ((this.m_averageGroundLevel.get() - this.m_box.get().m_max[1]) + 15) - 1, 0])
        });
    }
    this.fillMeta(world, sbb, 0, -10, 0, 5, 30, 5, blk("tf:mazestone"), 1, blk("air"), 0, true);
    this.fillAir(world, sbb, 1, -10, 1, 4, 30, 4);
    let _var8: i32 = this.xAt(0, 0);
    let _var9: i32 = this.yAt(0);
    let _var10: i32 = this.zAt(0, 0);
    return true;
}

// ComponentTFMazeEntranceShaft.getAverageGroundLevel
pub fn getAverageGroundLevel(this: &Piece, world: &GenWorld, sbb: StructureBox) -> i32
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
    return if var4 == 0 { -1 } else { var3 / var4 };
}
