// draft from ComponentTFTowerOutbuilding.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, x: i32, y: i32, z: i32, pSize: i32, pHeight: i32, direction: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::TowerOutbuilding);
    init(&this, i, x, y, z, pSize, pHeight, direction);
    Rc::new(this)
}

// ComponentTFTowerOutbuilding constructor
pub fn init(this: &Piece, i: i32, x: i32, y: i32, z: i32, pSize: i32, pHeight: i32, direction: i32)
{
    tower_wing::init7(this, i, x, y, z, pSize, pHeight, direction);
}

// ComponentTFTowerOutbuilding.makeABeard
pub fn makeABeard(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random) {}

// ComponentTFTowerOutbuilding.makeTowerWing
pub fn makeTowerWing(
    this: &Piece,
    list: &mut Pieces,
    random: &Random,
    index: i32,
    x: i32,
    y: i32,
    z: i32,
    wingSize: i32,
    wingHeight: i32,
    direction: i32,
) -> bool
{
    return if y > 7
    {
        tower_wing::makeTowerWing(this, list, random, index, x, y, z, wingSize, wingHeight, direction)
    }
    else
    {
        false
    };
}

// ComponentTFTowerOutbuilding.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    for x in 0..this.m_size.get()
    {
        for z in 0..this.m_size.get()
        {
            this.fillDown(world, blk("cobblestone"), 0, x, -1, z, sbb);
        }
    }
    return tower_wing::add(this, world, random, sbb);
}
