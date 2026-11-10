// draft from ComponentTFDarkTowerEntranceBridge.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, x: i32, y: i32, z: i32, pSize: i32, pHeight: i32, direction: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::DarkTowerEntranceBridge);
    init(&this, i, x, y, z, pSize, pHeight, direction);
    Rc::new(this)
}

// ComponentTFDarkTowerEntranceBridge constructor
pub fn init(this: &Piece, i: i32, x: i32, y: i32, z: i32, pSize: i32, pHeight: i32, direction: i32)
{
    dark_tower_bridge::init(this, i, x, y, z, pSize, pHeight, direction);
}

// ComponentTFDarkTowerEntranceBridge.makeTowerWing
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
    rotation: i32,
) -> bool
{
    let direction: i32 = (this.mode() + rotation) % 4;
    let dx: Vec<i32> = this.offsetTower(x, y, z, wingSize, direction);
    let wing: Rc<Piece> = dark_tower_entrance::new(index, dx[0], dx[1], dx[2], wingSize, wingHeight, direction);
    list.push(Rc::clone(&wing));
    wing.build(this, list, random);
    this.addOpening(x, y, z, rotation);
    return true;
}
