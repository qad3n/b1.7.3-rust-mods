// draft from ComponentTFFinalCastleEntranceBottomTower.java by tools/structgen.py
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
    let this: Piece = Piece::blank(Kind::FinalCastleEntranceBottomTower);
    init(&this, random, i, x, y, z, floors, entranceFloor, direction);
    Rc::new(this)
}

// ComponentTFFinalCastleEntranceBottomTower constructor
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
}

// ComponentTFFinalCastleEntranceBottomTower.func_74861_a
pub fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    this.m_deco.set((parent).m_deco.get());
    this.addStairs(list, random, this.depth() + 1, this.m_size.get() - 1, 1, this.m_size.get() / 2, 0);
    this.addStairs(list, random, this.depth() + 1, 0, 1, this.m_size.get() / 2, 2);
    this.addStairs(list, random, this.depth() + 1, this.m_size.get() / 2, 1, 0, 3);
    this.addStairs(list, random, this.depth() + 1, this.m_size.get() / 2, 1, this.m_size.get() - 1, 1);
}

// ComponentTFFinalCastleEntranceBottomTower.addStairs
pub fn addStairs(
    this: &Piece,
    list: &mut Pieces,
    random: &Random,
    index: i32,
    x: i32,
    y: i32,
    z: i32,
    rotation: i32,
) -> bool
{
    this.addOpening(x, y, z, rotation);
    let direction: i32 = (this.mode() + rotation) % 4;
    let dx: [i32; 3] = this.offsetTowerAt(x, y, z, 0, direction);
    let stairs: Rc<Piece> = final_castle_entrance_stairs::new(index, dx[0], dx[1], dx[2], direction);
    list.push(Rc::clone(&stairs));
    stairs.build(&Rc::clone(&list[(0) as usize]), list, random);
    return true;
}

// ComponentTFFinalCastleEntranceBottomTower.hasAccessibleRoof
pub fn hasAccessibleRoof(this: &Piece) -> bool { return false; }
