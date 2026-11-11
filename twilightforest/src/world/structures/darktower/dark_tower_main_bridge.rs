// draft from ComponentTFDarkTowerMainBridge.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(i: i32, x: i32, y: i32, z: i32, pSize: i32, pHeight: i32, direction: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::DarkTowerMainBridge);
    init(&this, i, x, y, z, pSize, pHeight, direction);
    Rc::new(this)
}

// ComponentTFDarkTowerMainBridge constructor
pub(crate) fn init(this: &Piece, i: i32, x: i32, y: i32, z: i32, _pSize: i32, pHeight: i32, direction: i32)
{
    dark_tower_bridge::init(this, i, x, y, z, 15, pHeight, direction);
}

// ComponentTFDarkTowerMainBridge.makeTowerWing
pub(crate) fn makeTowerWing(
    this: &Piece,
    list: &mut Pieces,
    random: &Random,
    index: i32,
    x: i32,
    y: i32,
    z: i32,
    _wingSize: i32,
    _wingHeight: i32,
    rotation: i32,
) -> bool
{
    let direction: i32 = (this.mode() + rotation) % 4;
    let dx: Vec<i32> = this.offsetTower(x, y, z, 19, direction);
    let wing: Rc<Piece> = dark_tower_main::new7(random, index, dx[0], dx[1], dx[2], direction);
    list.push(Rc::clone(&wing));
    wing.build(this, list, random);
    this.addOpening(x, y, z, rotation);
    true
}
