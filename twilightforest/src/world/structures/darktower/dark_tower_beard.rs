// draft from ComponentTFDarkTowerBeard.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, wing: &Piece) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::DarkTowerBeard);
    init(&this, i, wing);
    Rc::new(this)
}

// ComponentTFDarkTowerBeard constructor
pub fn init(this: &Piece, i: i32, wing: &Piece)
{
    this.m_depth.set(i);
    this.setMode(wing.mode());
    this.m_size.set(wing.m_size.get());
    this.m_height.set(this.m_size.get() / 2);
    this.m_box.set(StructureBox::new(
        [wing.bounds().m_min[0], wing.bounds().m_min[1] - this.m_height.get(), wing.bounds().m_min[2]],
        [wing.bounds().m_max[0], wing.bounds().m_min[1], wing.bounds().m_max[2]],
    ));
}

// ComponentTFDarkTowerBeard.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    dark_tower_beard::makeDarkBeard(
        this,
        world,
        sbb,
        0,
        0,
        0,
        this.m_size.get() - 1,
        this.m_height.get() - 1,
        this.m_size.get() - 1,
    );
    return true;
}

// ComponentTFDarkTowerBeard.makeDarkBeard
pub fn makeDarkBeard(
    this: &Piece,
    world: &GenWorld,
    sbb: StructureBox,
    minX: i32,
    minY: i32,
    minZ: i32,
    maxX: i32,
    maxY: i32,
    maxZ: i32,
)
{
    let frameID: Blk = blk("tf:towerWood");
    let frameMeta: i32 = 1;
    for x in minX..=maxX
    {
        for z in minZ..=maxZ
        {
            if (((x == minX) || (x == maxX)) || (z == minZ)) || (z == maxZ)
            {
                let mut length: i32 = ((x - this.m_height.get()).abs() - 1).min((z - this.m_height.get()).abs() - 1);
                if length == (this.m_height.get() - 1)
                {
                    length += 1;
                }
                if length == (-1)
                {
                    length = 1;
                }
                for y in (this.m_height.get() - length..=maxY).rev()
                {
                    this.placeBlock(world, frameID, frameMeta, x, y, z, sbb);
                }
            }
        }
    }
}
