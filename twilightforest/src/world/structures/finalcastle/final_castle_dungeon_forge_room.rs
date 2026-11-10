// draft from ComponentTFFinalCastleDungeonForgeRoom.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(random: &Random, i: i32, x: i32, y: i32, z: i32, direction: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::FinalCastleDungeonForgeRoom);
    init(&this, random, i, x, y, z, direction);
    Rc::new(this)
}

// ComponentTFFinalCastleDungeonForgeRoom constructor
pub fn init(this: &Piece, random: &Random, i: i32, x: i32, y: i32, z: i32, direction: i32)
{
    this.m_depth.set(i);
    this.m_spawnListIndex.set(3);
    this.setMode(direction);
    this.m_box.set(boxFor2(x, y, z, -15, 0, -15, 50, 30, 50, direction));
}

// ComponentTFFinalCastleDungeonForgeRoom.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    this.fillAir(world, sbb, 0, 0, 0, 50, 30, 50);
    this.placeSign(world, 25, 0, 25, "Mini-boss 2", "Gives talisman", sbb);
    return true;
}
