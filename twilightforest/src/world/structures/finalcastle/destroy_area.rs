use crate::world::structures::prelude::*;

// DestroyArea, a column of the damaged castle towers knocked out from a height down
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct DestroyArea
{
    pub m_box: StructureBox,
}

mod size
{
    pub const DEPTH: i32 = 10;
    pub const WIDTH: i32 = 4;
    pub const ATTEMPTS: i32 = 100;
}

impl DestroyArea
{
    pub fn new(tower: StructureBox, random: &Random, y: i32) -> DestroyArea
    {
        let x: i32 = tower.m_min[0] - 2 + random.nextIntBounded(tower.xSize());
        let z: i32 = tower.m_min[2] - 2 + random.nextIntBounded(tower.zSize());
        DestroyArea { m_box: StructureBox::new([x, y - size::DEPTH, z], [x + size::WIDTH, y, z + size::WIDTH]) }
    }

    pub fn isEntirelyAbove(&self, y: i32) -> bool { self.m_box.m_min[1] > y }

    pub fn isVecInside(&self, x: i32, y: i32, z: i32) -> bool { self.m_box.contains(x, y, z) }

    // DestroyArea.intersectsWith, which tests the other area's max x as its max z
    fn intersectsWith(&self, other: &DestroyArea) -> bool
    {
        let bounds: StructureBox = other.m_box;
        self.m_box.intersectsXZ(bounds.m_min[0] - 1, bounds.m_min[2] - 1, bounds.m_max[0] + 1, bounds.m_max[0] + 1)
    }
}

// DestroyArea.createNonIntersecting, nothing when every try overlaps the areas already chosen
pub fn createNonIntersecting(
    tower: StructureBox,
    random: &Random,
    y: i32,
    others: &[Option<DestroyArea>],
) -> Option<DestroyArea>
{
    let mut area: Option<DestroyArea> = None;
    let mut tries: i32 = 0;
    while tries < size::ATTEMPTS && area.is_none()
    {
        let test: DestroyArea = DestroyArea::new(tower, random, y);
        if others.is_empty() || others.iter().any(|other| other.is_none_or(|other| !test.intersectsWith(&other)))
        {
            area = Some(test);
        }
        tries += 1;
    }
    area
}
