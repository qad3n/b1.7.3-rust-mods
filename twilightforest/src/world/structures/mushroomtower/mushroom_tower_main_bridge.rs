// draft from ComponentTFMushroomTowerMainBridge.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, x: i32, y: i32, z: i32, pSize: i32, pHeight: i32, direction: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::MushroomTowerMainBridge);
    init(&this, i, x, y, z, pSize, pHeight, direction);
    Rc::new(this)
}

// ComponentTFMushroomTowerMainBridge constructor
pub fn init(this: &Piece, i: i32, x: i32, y: i32, z: i32, pSize: i32, pHeight: i32, direction: i32)
{
    mushroom_tower_bridge::init(this, i, x, y, z, 11, pHeight, direction);
}

// ComponentTFMushroomTowerMainBridge.makeTowerWing
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
    let mut dx: Vec<i32> = this.offsetTower(x, y, z, 15, direction);
    dx = mushroom_tower_wing::adjustCoordinates(this, dx[0], dx[1], dx[2], 15, direction, list);
    let wing: Rc<Piece> = mushroom_tower_main::new7b(index, dx[0], dx[1], dx[2], 15, wingHeight, direction);
    list.push(Rc::clone(&wing));
    wing.build(&Rc::clone(&list[(0) as usize]), list, random);
    this.addOpening(x, y, z, rotation);
    return true;
}
