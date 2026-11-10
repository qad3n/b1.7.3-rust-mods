// draft from ComponentTFFinalCastleEntranceSideTower.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(
    random: &Random,
    i: i32,
    x: i32,
    y: i32,
    z: i32,
    floors: i32,
    entranceFloor: i32,
    direction: i32,
) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::FinalCastleEntranceSideTower);
    init(&this, random, i, x, y, z, floors, entranceFloor, direction);
    Rc::new(this)
}

// ComponentTFFinalCastleEntranceSideTower constructor
pub fn init(
    this: &Piece,
    random: &Random,
    i: i32,
    x: i32,
    y: i32,
    z: i32,
    floors: i32,
    entranceFloor: i32,
    direction: i32,
)
{
    final_castle_maze_tower13::init9(this, random, i, x, y, z, floors, entranceFloor, 0, direction);
    this.addOpening(0, 1, this.m_size.get() / 2, 2);
}

// ComponentTFFinalCastleEntranceSideTower.func_74861_a
pub fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    this.m_deco.set((parent).m_deco.get());
    let foundation: Rc<Piece> = final_castle_foundation13::new(random, 4, this);
    list.push(Rc::clone(&foundation));
    foundation.build(this, list, random);
    let roof: Rc<Piece> = final_castle_roof13_peaked::new(random, 4, this);
    list.push(Rc::clone(&roof));
    roof.build(this, list, random);
}
