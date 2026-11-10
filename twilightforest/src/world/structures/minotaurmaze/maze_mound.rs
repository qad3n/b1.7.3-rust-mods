// draft from ComponentTFMazeMound.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub const DIAMETER: i32 = 35;

pub fn new(i: i32, random: &Random, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::MazeMound);
    init(&this, i, random, x, y, z);
    Rc::new(this)
}

// ComponentTFMazeMound constructor
pub fn init(this: &Piece, i: i32, random: &Random, x: i32, y: i32, z: i32)
{
    this.m_depth.set(i);
    this.m_averageGroundLevel.set(-1);
    this.m_mode.set(random.nextIntBounded(4));
    this.m_box.set(StructureBox::new([x, y, z], [x + 35, y + 8, z + 35]));
}

// ComponentTFMazeMound.func_74861_a
pub fn build(this: &Piece, structurecomponent: &Piece, list: &mut Pieces, random: &Random)
{
    *this.m_mazeAbove.borrow_mut() = Some(maze_upper_entrance::new(
        3,
        random,
        this.m_box.get().m_min[0] + 10,
        this.m_box.get().m_min[1] + 0,
        this.m_box.get().m_min[2] + 10,
    ));
    list.push(this.m_mazeAbove.borrow().piece().clone());
    this.m_mazeAbove.borrow().piece().clone().build(this, list, random);
}

// ComponentTFMazeMound.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    if this.m_averageGroundLevel.get() < 0
    {
        this.m_averageGroundLevel.set(this.groundLevel(world, sbb));
        if this.m_averageGroundLevel.get() < 0
        {
            return true;
        }
        let offset: i32 = ((this.m_averageGroundLevel.get() - this.m_box.get().m_max[1]) + 8) - 1;
        this.boxed(|bounds| bounds.shift([0, offset, 0]));
        if let Some(above) = this.m_mazeAbove.borrow().as_ref()
        {
            above.boxed(|bounds| bounds.shift([0, offset, 0]));
        }
    }
    for x in 0..35
    {
        for z in 0..35
        {
            let cx: i32 = x - 17;
            let cz: i32 = z - 17;
            let dist: i32 = ((((cx * cx) + (cz * cz)) as f64).sqrt()) as i32;
            let hheight: i32 = ((((dist) as f64 / 35.0) * std::f64::consts::PI).cos() * 11.0) as i32;
            if ((((cx > 2) || (cx < (-1))) || (cz > 2)) || (cz < (-1)))
                && ((((cx > 2) || (cx < (-1))) && ((cz > 2) || (cz < (-1)))) || (hheight > 6))
            {
                this.placeBlock(world, blk("grass"), 0, x, hheight, z, sbb);
                if ((cx <= 2) && (cx >= (-1))) || ((cz <= 2) && (cz >= (-1)))
                {
                    if hheight > 6
                    {
                        this.fill(world, sbb, x, 6, z, x, hheight - 1, z, blk("dirt"), blk("air"), false);
                    }
                }
                else
                {
                    this.fillDown(world, blk("dirt"), 0, x, hheight - 1, z, sbb);
                }
            }
        }
    }
    return true;
}

// ComponentTFMazeMound.getAverageGroundLevel
pub fn getAverageGroundLevel(this: &Piece, world: &GenWorld, sbb: StructureBox) -> i32
{
    let mut totalHeight: i32 = 0;
    let mut totalMeasures: i32 = 0;
    for var5 in this.m_box.get().m_min[2]..=this.m_box.get().m_max[2]
    {
        for var6 in this.m_box.get().m_min[0]..=this.m_box.get().m_max[0]
        {
            if sbb.contains(var6, 64, var5)
            {
                totalHeight += world.topSolid(var6, var5).max(crate::world::terrain::SEA_LEVEL);
                totalMeasures += 1;
            }
        }
    }
    return if totalMeasures == 0
    {
        -1
    }
    else
    {
        totalHeight / totalMeasures
    };
}
