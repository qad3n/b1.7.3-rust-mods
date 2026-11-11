// draft from ComponentTFFinalCastleEntranceTower.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(random: &Random, i: i32, x: i32, y: i32, z: i32, direction: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::FinalCastleEntranceTower);
    init(&this, random, i, x, y, z, direction);
    Rc::new(this)
}

// ComponentTFFinalCastleEntranceTower constructor
pub fn init(this: &Piece, random: &Random, i: i32, x: i32, y: i32, z: i32, direction: i32)
{
    final_castle_maze_tower13::init9(this, random, i, x, y, z, 3, 2, 0, direction);
}

// ComponentTFFinalCastleEntranceTower.func_74861_a
pub fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    this.m_deco.set((parent).m_deco.get());
    let foundation: Rc<Piece> = final_castle_foundation13::new(random, 4, this);
    list.push(Rc::clone(&foundation));
    foundation.build(this, list, random);
    let roof: Rc<Piece> = final_castle_roof13_peaked::new(random, 4, this);
    list.push(Rc::clone(&roof));
    roof.build(this, list, random);
    let missingFloors: i32 = (this.m_box.get().m_min[1] - 127) / 8;
    let bottomFloors: i32 = missingFloors / 2;
    let middleFloors: i32 = missingFloors - bottomFloors;
    let mut direction: i32 = 1;
    let howFar: i32 = 20;
    if !final_castle_entrance_tower::buildSideTower(this, list, random, middleFloors + 1, direction, howFar)
    {
        direction = 3;
        if !final_castle_entrance_tower::buildSideTower(this, list, random, middleFloors + 1, direction, howFar)
        {
            direction = 0;
            if !final_castle_entrance_tower::buildSideTower(this, list, random, middleFloors + 1, direction, howFar)
            {}
        }
    }
    let brDirection: i32 = (direction + this.m_mode.get()) % 4;
    let eTower: Rc<Piece> = final_castle_entrance_bottom_tower::new(
        random,
        this.depth() + 1,
        this.m_box.get().m_min[0] + 6,
        this.m_box.get().m_min[1] - (middleFloors * 8),
        this.m_box.get().m_min[2] + 6,
        bottomFloors + 1,
        bottomFloors,
        (brDirection + 2) % 4,
    );
    list.push(Rc::clone(&eTower));
    eTower.build(this, list, random);
    let mut opening: [i32; 3] = this.getValidOpeningCC(random, direction);
    opening[1] -= middleFloors * 8;
    let bc: [i32; 3] = this.offsetTowerAt(opening[0], opening[1], opening[2], 1, brDirection);
    let bridge: Rc<Piece> = final_castle_bridge::new(this.depth() + 1, bc[0], bc[1], bc[2], howFar - 7, brDirection);
    list.push(Rc::clone(&bridge));
    bridge.build(this, list, random);
}

// ComponentTFFinalCastleEntranceTower.buildSideTower
pub fn buildSideTower(
    this: &Piece,
    list: &mut Pieces,
    random: &Random,
    middleFloors: i32,
    mut direction: i32,
    howFar: i32,
) -> bool
{
    let opening: [i32; 3] = this.getValidOpeningCC(random, direction);
    direction += this.m_mode.get();
    direction %= 4;
    let tc: [i32; 3] = this.offsetTowerAt(opening[0], opening[1], opening[2], howFar, direction);
    let eTower: Rc<Piece> = final_castle_entrance_side_tower::new(
        random,
        this.depth() + 1,
        tc[0],
        tc[1],
        tc[2],
        middleFloors,
        middleFloors - 1,
        direction,
    );
    let mut largerBB: StructureBox = eTower.bounds();
    largerBB.m_min[0] -= 6;
    largerBB.m_min[2] -= 6;
    largerBB.m_max[0] += 6;
    largerBB.m_max[2] += 6;
    let intersect: Option<Rc<Piece>> = intersecting(list, largerBB);
    if intersect.is_none()
    {
        list.push(Rc::clone(&eTower));
        eTower.build(this, list, random);
        let bc: [i32; 3] = this.offsetTowerAt(opening[0], opening[1], opening[2], 1, direction);
        let bridge: Rc<Piece> = final_castle_bridge::new(this.depth() + 1, bc[0], bc[1], bc[2], howFar - 7, direction);
        list.push(Rc::clone(&bridge));
        bridge.build(this, list, random);
        this.addOpening(opening[0], opening[1] + 1, opening[2], direction);
        return true;
    }
    else
    {
        return false;
    }
}

// ComponentTFFinalCastleEntranceTower.getValidOpeningCC
pub fn getValidOpeningCC(_this: &Piece, _random: &Random, direction: i32) -> [i32; 3]
{
    if (direction == 0) || (direction == 2)
    {
        let rx: i32 = if direction == 0 { 12 } else { 0 };
        let rz: i32 = 6;
        let ry: i32 = 0;
        return [rx, ry, rz];
    }
    if (direction != 1) && (direction != 3)
    {
        return [0, 0, 0];
    }
    let rx: i32 = 6;
    let rz: i32 = if direction == 1 { 12 } else { 0 };
    let ry: i32 = 0;
    return [rx, ry, rz];
}
