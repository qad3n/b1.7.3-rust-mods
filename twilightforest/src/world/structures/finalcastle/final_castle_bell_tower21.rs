// draft from ComponentTFFinalCastleBellTower21.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(random: &Random, i: i32, x: i32, y: i32, z: i32, direction: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::FinalCastleBellTower21);
    init(&this, random, i, x, y, z, direction);
    Rc::new(this)
}

// ComponentTFFinalCastleBellTower21 constructor
pub fn init(this: &Piece, random: &Random, i: i32, x: i32, y: i32, z: i32, direction: i32)
{
    final_castle_maze_tower13::init9(this, random, i, x, y, z, 8, 1, 1, direction);
    this.m_size.set(21);
    let floors: i32 = 8;
    this.m_height.set((floors * 8) + 1);
    this.m_box.set(boxFor2(
        x,
        y,
        z,
        -6,
        -8,
        (-this.m_size.get()) / 2,
        this.m_size.get() - 1,
        this.m_height.get(),
        this.m_size.get() - 1,
        direction,
    ));
    this.m_openings.borrow_mut().clear();
    this.addOpening(0, 9, this.m_size.get() / 2, 2);
}

// ComponentTFFinalCastleBellTower21.func_74861_a
pub fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    this.m_deco.set((parent).m_deco.get());
    let foundation: Rc<Piece> = final_castle_bell_foundation21::new(random, 4, this);
    list.push(Rc::clone(&foundation));
    foundation.build(this, list, random);
    let roof: Rc<Piece> = final_castle_roof13_crenellated::new(random, 4, this);
    list.push(Rc::clone(&roof));
    roof.build(this, list, random);
}

// ComponentTFFinalCastleBellTower21.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    final_castle_maze_tower13::add(this, world, random, sbb);
    let fieldBlock: Blk = blk("tf:forceField");
    let fieldMeta: i32 = 4;
    for rotation in 0..4
    {
        let y: i32 = 48;
        for x in (5..(this.m_size.get() - 4)).step_by(2)
        {
            this.fillTurned(world, sbb, x, y, 0, x, y + 14, 0, fieldBlock, fieldMeta, rotation);
        }
        let var9: i32 = 24;
        for x in (1..(this.m_size.get() - 1)).step_by(8)
        {
            this.fillTurned(world, sbb, x, var9, 0, x, var9 + 14, 0, fieldBlock, fieldMeta, rotation);
            this.fillTurned(world, sbb, x + 2, var9, 0, x + 2, var9 + 14, 0, fieldBlock, fieldMeta, rotation);
        }
    }
    this.placeSign(world, 7, 9, 8, "Parkour area 2", "mini-boss 1", sbb);
    return true;
}
